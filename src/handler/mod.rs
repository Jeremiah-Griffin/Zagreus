use crate::{
    errors::{BackoffError, BackoffErrorKind},
    logging::BackoffLogger,
    random::Randomizer,
    strategy::BackoffStrategy,
};
use std::{error::Error, future::Future, time::Duration};
mod test;

///Provides the interface for  retries and backoffs.
pub trait BackoffHandler: Send {
    ///At scale randomization can be somewhat expensive. It is therefore encouraged that an RNG be stored inside the implementor
    ///and a reference to it returned by this method so that it may be reused.
    fn randomizer(&mut self) -> &mut impl Randomizer;

    ///Runs a function and attempts retries based on its type as well as the type parameters provided:
    ///
    ///# Function Parameters
    ///- fallible: the fallible function that will potentially be retried.
    ///- is_recoverable: determines whether the error returned by Fallible (if any) is able to be recovered: for example,
    ///  HTTP 503 (Service Unavailable) may resolve itself by being retried, while an HTTP 404 (Not Found) is highly unlikely to be so.
    ///  in the former case `true` should be returned and in the latter, `false`.
    ///- peek_retry: Allows the BackoffHandler to decide whether to to terminate early based on fallible's Error value *or* the next retry interval planned by the Handler.
    ///  this can be useful i.e. when receiving an HTTP 429 (Retry Later), and overriding the planned_duration, or terminating early if planned_duration is too long.
    ///  When this returns `Some(_)`, the returned duration will be used as the interval between retries.
    ///  When it returns `None` the Handler will terminate all retries immediately and return a `BackoffError` with a `BackoffErrorKind:::EarlyTermination` as its `kind` value.
    ///- sleep: used to make the current thread/task/etc sleep for the duration calculated by the BackoffStrategy and peek_retry.
    ///- strategy: The `BackoffStrategy` which will be used to generate retry intervals for this call.
    ///- logg: The `BackoffLogger` which will log errors generated by this call.
    ///
    ///
    ///# Generic Parameters
    ///- `T`: The success value of `fallible`.
    ///- `E`: The error value of `fallible`.
    ///- `F`: The generic type of `fallible` itself.
    ///- `S`: The `Future` returned by `sleep`.
    fn handle<T: Send, E: Send + Error, F, S>(
        &mut self,
        //Make sure that ONLY fallible is accepted as a closure:
        //When nightly_auto_trait is implemented it limits the number of structural checks for the implementors of CanBackoff
        //this means that if any of the other callbacks take a BackoffHandler a nested call could occur, but that is an unlikely
        //usecase for any of those methods, while allowing paterns that dont lend themselves to auto trait implementation checking (like dyn trait) to be used
        //in those callbacks, if necessary
        mut fallible: impl FnMut() -> F + Send,
        is_recoverable: fn(error: &E) -> bool,
        peek_retry: fn(error: &E, planned_interval: Duration, attempt: u32) -> Option<Duration>,
        sleep: fn(to_sleep: Duration) -> S,
        strategy: impl BackoffStrategy,
        logger: impl BackoffLogger<E>,
    ) -> impl Future<Output = Result<T, E>> + Send
    where
        F: Future<Output = Result<T, E>> + Send,
        S: Future<Output = ()> + Send,
    {
        fn log_and_return<Err: Error>(
            error: Err,
            kind: BackoffErrorKind,
            logger: &impl BackoffLogger<Err>,
        ) -> Err {
            let backoff_error = BackoffError::new(error, kind);
            logger.log_terminal(&backoff_error);
            backoff_error.into_error()
        }
        async move {
            let limit = strategy.limit();

            //index from 1 so that number off attempts is reported acccurately and that attempts passed to inteveral is never 0.
            //we iterate to limit exclusive so that the final retry is the limit, nth retry.
            for attempt in 1..limit.get() {
                let res = fallible().await;
                let Err(error) = res else {
                    return res;
                };

                //if an error is not recoverable we terminate iteration
                if is_recoverable(&error) == false {
                    return Err(log_and_return(
                        error,
                        BackoffErrorKind::Unrecoverable(attempt),
                        &logger,
                    ));
                };

                //interval can terminate iteration
                let Some(interval) = strategy.interval(attempt) else {
                    return Err(log_and_return(
                        error,
                        BackoffErrorKind::IntervalTerminated(attempt),
                        &logger,
                    ));
                };

                //peek_retry can terminate iteration.
                match peek_retry(&error, self.randomizer().randomize(interval), attempt) {
                    Some(i) => {
                        logger.log_nonterminal(&error, attempt);
                        sleep(i).await
                    }
                    None => {
                        return Err(log_and_return(
                            error,
                            BackoffErrorKind::PeekTerminated(attempt),
                            &logger,
                        ));
                    }
                }
            }

            //We don't bother to call peek_retry with this iteration as the prior iteration wil have already done so.
            fallible().await.map_err(|e| match is_recoverable(&e) {
                true => log_and_return(e, BackoffErrorKind::ExhaustedLimit(limit), &logger),
                false => log_and_return(
                    e,
                    BackoffErrorKind::UnrecoverableAndExhaustedLimit(limit),
                    &logger,
                ),
            })
        }
    }
}
