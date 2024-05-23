use std::{future::Future, time::Duration};

use crate::{BackoffError, BackoffErrorKind, BackoffStrategy};

pub trait Randomizer {
    fn randomize_interval(interval: Duration) -> Duration;
}

///TODO: is this better to be in backoff handler or strategy? No real clue.
///A Randomizer implementation that doesn't actually do anything.
///While randomization is useful to prevent concurrent requests to an endpoint
///exacerbating network issues in a storm, it may involve a syscall or otherwise be more expensive
///than a no op if it is not necessary.
pub struct NotRandom {}

impl Randomizer for NotRandom {
    fn randomize_interval(interval: Duration) -> Duration {
        interval
    }
}

//I dont know if async_fn_in_trait will be a problem or not as we don't care about auto trait bounds Will check to be sure.
//TODO: should Strategy be a supertrait for handler or a generic parameter?
#[allow(private_bounds, async_fn_in_trait)]
pub trait BackoffHandler: BackoffStrategy + Send {
    ///The method that will be called to automatically log the BackoffError that will be returned.
    fn log<E>(e: &BackoffError<E>);

    ///Some errors are unrecoverable, and it makes sense to short circuit on them.
    ///Fallible is the function which will be retried upon failure,
    ///is_recoverable is used to determine whether a try should be attempted.
    async fn handle<T, E, Fal, Fut, R, S>(
        &self,
        //the fallible closure that may be retryied upon its failure.
        mut fallible: Fal,
        //returns true if the error may be solved by retrying, false otherwise.
        is_recoverable: fn(&E) -> bool,
        //This is useful in cases like an html retry_after header which requests a retry come later.
        //this does *not* change the behavior of the backoff_strategy for any future iteration.
        //if this function returns None, retrying will be immediately aborted - useful in a case where the extended duration is too great. Otherwise, the next retry will come
        //after the term determined by the strategy PLUS the returned Duration.
        //TODO: should we  allow the caller to see the duration planned by the backoff strategy to permit better decisions about maximum rety times?
        //TODO: this is currently unused
        extend_retry: fn(&E) -> Option<Duration>,
        //The function that will be used to sleep
        //the caller, i.e. std::thread::sleep.
        sleep: fn(Duration) -> S,
    ) -> Result<T, E>
    where
        Fal: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        R: Randomizer,
        S: Future<Output = ()>,
    {
        let log_and_return = |e: BackoffError<E>| -> E {
            Self::log(&e);
            e.error
        };
        //index from 1 so that number off attempts is reported acccurately and that attempts passed to inteveral is never 0.
        //we iterate to limit exclusive so that the final retry is the limit, nth retry.
        for attempts in 1..self.limit().get() {
            match fallible().await {
                Ok(t) => return Ok(t),
                Err(e) => match is_recoverable(&e) {
                    true => sleep(R::randomize_interval(self.interval(attempts))).await,
                    false => {
                        let e = BackoffError {
                            error: e,
                            kind: BackoffErrorKind::Unrecoverable(attempts),
                        };

                        return Err(log_and_return(e));
                    }
                },
            }
        }

        fallible()
            .await
            .map_err(|e| match is_recoverable(&e) {
                true => BackoffError {
                    error: e,
                    kind: BackoffErrorKind::ExhaustedLimit(self.limit()),
                },
                false => BackoffError {
                    error: e,
                    kind: BackoffErrorKind::UnrecoverableAndExhaustedLimit(self.limit()),
                },
            })
            .map_err(|e| log_and_return(e))
    }
}
