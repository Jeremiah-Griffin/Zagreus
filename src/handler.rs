use std::{future::Future, num::NonZeroU32, time::Duration};

use crate::{random::Randomizer, strategy::BackoffStrategy, BackoffError, BackoffErrorKind};

//I dont know if async_fn_in_trait will be a problem or not as we don't care about auto trait bounds Will check to be sure.
#[allow(async_fn_in_trait)]
pub trait BackoffHandler: Send {
    //since it's meant to be overrided and the default is to do nothing we really should be unused.
    #[allow(unused)]
    ///Allows the implementor to "hook" into the retry loop and log the final returned error internal to a BackoffHandler.
    ///default implementation is a no-op. Feel free to override this as necessary.
    fn log<E>(e: &BackoffError<E>) {}

    ///Runs a function and attempts retries based on its type as well as the type parameters provided:
    ///
    ///# Function Parameters
    ///- fallible: the fallible function that will potentially be retried.
    ///- is_recoverable: determines whether the error returned by Fallible (if any) is able to be recovered: for example,
    ///  HTTP 503 (Service Unavailable) may resolve itself by being retried, while an HTTP 404 (Not Found) is highly unlikely to be so.
    ///  in the former case `true` should be returned and in the latter, `false`.
    ///- peek_retry: Allows the BackoffHandler to decide whether to to terminate early based on fallible's Error value *or* the next retry interval planned by the Handler.
    ///  this can be useful i.e. when receiving an HTTP 429 (Retry Later), and overriding the planned_duration, or terminating early if planned_duration is too long.
    ///  When this returns Some(_), the returned duration will be used as the interval between retries.
    ///  When it returns None the Handler will terminate all retries immediately and return a BackoffError::EarlyTermination error.
    ///- sleep: used to make the current thread/task/etc sleep for the duration calculated by the BackoffStrategy and peek_retry.
    ///
    ///# Generic Parameters
    ///- `T`: The success value of `fallible`.
    ///- `E`: The error value of `fallible`.
    ///- `F`: The generic type of `fallible` itself.
    ///- `S`: The `Future` returned by `sleep`.
    ///Note: parameters T, E, Fal, Fut, and S, are very likely to be inferred by the compiler. However, R might not.
    async fn handle<T, E, F, S>(
        &self,
        mut fallible: impl FnMut() -> F,
        is_recoverable: fn(error: &E) -> bool,
        peek_retry: fn(error: &E, planned_interval: Duration) -> Option<Duration>,
        randomizer: &mut impl Randomizer,
        strategy: impl BackoffStrategy,
        sleep: fn(to_sleep: Duration) -> S,
    ) -> Result<T, E>
    where
        F: Future<Output = Result<T, E>>,
        S: Future<Output = ()>,
    {
        let log_and_return = |e: BackoffError<E>| -> E {
            Self::log(&e);
            e.error
        };

        let limit = strategy.limit();
        //index from 1 so that number off attempts is reported acccurately and that attempts passed to inteveral is never 0.
        //we iterate to limit exclusive so that the final retry is the limit, nth retry.
        for attempts in 1..limit.get() {
            let res = fallible().await;
            let Err(error) = res else {
                return res;
            };

            if is_recoverable(&error) == false {
                return Err(log_and_return(BackoffError {
                    error,
                    kind: BackoffErrorKind::Unrecoverable(attempts),
                }));
            };

            let interval = randomizer.randomize(strategy.interval(attempts));

            match peek_retry(&error, interval) {
                Some(i) => sleep(i).await,
                None => {
                    return Err(log_and_return(BackoffError {
                        error,
                        kind: BackoffErrorKind::ExplicitlyTerminated(attempts),
                    }));
                }
            }
        }

        //We don't bother to call peek_retry with this iteration as the prior iteration wil have already done so.
        fallible()
            .await
            .map_err(|e| match is_recoverable(&e) {
                true => BackoffError {
                    error: e,
                    kind: BackoffErrorKind::ExhaustedLimit(limit),
                },
                false => BackoffError {
                    error: e,
                    kind: BackoffErrorKind::UnrecoverableAndExhaustedLimit(limit),
                },
            })
            .map_err(|e| log_and_return(e))
    }
}
