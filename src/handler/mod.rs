use crate::{
    errors::{BackoffError, BackoffErrorKind},
    logging::{BackoffLogger, LoggingStrategy},
    random::Randomizer,
    strategy::BackoffStrategy,
};
use std::{error::Error, future::Future, time::Duration};

//I dont know if async_fn_in_trait will be a problem or not as we don't care about auto trait bounds Will check to be sure.
pub trait BackoffHandler: Send {
    //since it's meant to be overrided and the default is to do nothing we really should be unused.
    #[allow(unused)]
    ///Allows the implementor to "hook" into the retry loop and log the final returned error internal to a BackoffHandler.
    ///default implementation is a no-op. Feel free to override this as necessary.
    fn log<E: Error + Send>(e: &BackoffError<E>) {}

    ///At scale randomziation can be somewhat expensive. It is therefore encouraged that an RNG be stored inside the implementor
    ///and a reference to it returned by this method.
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
    ///- Randomizer: implementor of `Randomizer` that will be used to add random error to interval spacing.
    ///- sleep: used to make the current thread/task/etc sleep for the duration calculated by the BackoffStrategy and peek_retry.
    ///
    ///# Generic Parameters
    ///- `T`: The success value of `fallible`.
    ///- `E`: The error value of `fallible`.
    ///- `F`: The generic type of `fallible` itself.
    ///- `S`: The `Future` returned by `sleep`.
    fn handle<'a, T: Send, E: Send + Error, F, L: BackoffLogger<'a, E>, S>(
        &mut self,
        //Make sure that ONLY fallible is accepted as a closure:
        //When nightly_auto_trait is implemented it limits the number of structural checks for the implementors of CanBackoff
        //this means that if any of the other callbacks take a BackoffHandler a nested call could occur, but that is an unlikely
        //usecase for any of those methods, while allowing paterns that dont lend themselves to auto trait implementation checking (like dyn trait) to be used
        //in those callbacks, if necessary
        mut fallible: impl FnMut() -> F + Send,
        is_recoverable: fn(error: &E) -> bool,
        peek_retry: fn(error: &E, planned_interval: Duration, attempt: u32) -> Option<Duration>,
        backoff_strategy: impl BackoffStrategy,
        sleep: fn(to_sleep: Duration) -> S,
        logger: &'a L,
    ) -> impl Future<Output = Result<T, E>> + Send
    where
        F: Future<Output = Result<T, E>> + Send,
        S: Future<Output = ()> + Send,
    {
        async move {
            let mut logging_strategy = L::Strategy::new();
            let limit = backoff_strategy.limit();

            let log_and_return = |e: E, kind: BackoffErrorKind| -> E {
                let backoff_error = BackoffError::new(e, kind);

                Self::log(&backoff_error);
                backoff_error.into_error()
            };
            //index from 1 so that number off attempts is reported acccurately and that attempts passed to inteveral is never 0.
            //we iterate to limit exclusive so that the final retry is the limit, nth retry.
            for attempt in 1..limit.get() {
                let res = fallible().await;
                let Err(error) = res else {
                    return res;
                };

                let loan = match fallible().await {
                    Ok(r) => return Ok(r),
                    Err(e) => logging_strategy.lend(e),
                };

                //if an error is not recoverable we terminate iteration
                if is_recoverable(&error) == false {
                    return Err(log_and_return(
                        error,
                        BackoffErrorKind::Unrecoverable(attempt),
                    ));
                };

                //interval can terminate iteration
                let Some(interval) = backoff_strategy.interval(attempt) else {
                    return Err(log_and_return(
                        error,
                        BackoffErrorKind::IntervalTerminated(attempt),
                    ));
                };

                //peek_retry can terminate iteration.
                match peek_retry(&error, self.randomizer().randomize(interval), attempt) {
                    Some(i) => sleep(i).await,
                    None => {
                        return Err(log_and_return(
                            error,
                            BackoffErrorKind::PeekTerminated(attempt),
                        ));
                    }
                }
            }

            //We don't bother to call peek_retry with this iteration as the prior iteration wil have already done so.
            fallible().await.map_err(|e| match is_recoverable(&e) {
                true => log_and_return(e, BackoffErrorKind::ExhaustedLimit(limit)),
                false => log_and_return(e, BackoffErrorKind::UnrecoverableAndExhaustedLimit(limit)),
            })
        }
    }
}
