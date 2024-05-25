use std::{error::Error, fmt::Display, num::NonZeroU32};

#[derive(Debug)]
pub struct BackoffError<E: Error> {
    error: E,
    kind: BackoffErrorKind,
}

impl<E: Error> BackoffError<E> {
    pub fn new(error: E, kind: BackoffErrorKind) -> BackoffError<E> {
        BackoffError { error, kind }
    }

    ///Get a reference to the contained Error value.
    pub fn error(&self) -> &E {
        &self.error
    }

    ///Get a reference to the contained BackoffErrorKind.
    pub fn kind(&self) -> &BackoffErrorKind {
        &self.kind
    }

    ///Take the contained error and kind from this instance.
    pub fn into_error_and_kind(self) -> (E, BackoffErrorKind) {
        (self.error, self.kind)
    }

    ///Take the contained error from this instance.
    pub fn into_error(self) -> E {
        self.error
    }

    ///Take the contained kind from this instance.
    pub fn into_kind(self) -> BackoffErrorKind {
        self.kind
    }
}

#[derive(Debug)]
///Describes all the error states that can induce a backoff
pub enum BackoffErrorKind {
    ///When the discriminator supplied to a given strategy's handler
    ///calls the strategy to short circuit because the error is unrecoverable
    ///this is returned for logging.
    Unrecoverable(u32),
    ///Returned when there are no more retries left.
    ExhaustedLimit(NonZeroU32),
    ///The final call in a retry loop may return an unrecoverable error, in which case it is both Unrecoverable and ExhaustedLimit.
    UnrecoverableAndExhaustedLimit(NonZeroU32),
    ///A call to peek_retry returned None, meaning it requested further attempts to be cancelled.
    PeekTerminated(u32),
    ///A call to BackoffStrategy::interval returned None, meaning it requested further attempts to be cancelled.
    IntervalTerminated(u32),
}

impl<E: Error> Display for BackoffError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            BackoffErrorKind::Unrecoverable(i) => write!(
                f,
                "After {i} attempt(s) the following unrecoverable error was encountered: {}",
                self.error
            ),
            BackoffErrorKind::ExhaustedLimit(i) => {
                write!(f, "Limit of {} was exhausted. {}", i.get(), self.error)
            }
            BackoffErrorKind::UnrecoverableAndExhaustedLimit(i) => write!(
                f,
                "An unrecoverable error was encountered and the limit of {} was exhausted. {}",
                i.get(),
                self.error
            ),
            BackoffErrorKind::PeekTerminated(i) => {
                write!(
                    f,
                    "After {i} attempt(s), retrying was terminated by peek_retry"
                )
            }
            BackoffErrorKind::IntervalTerminated(i) => write!(
                f,
                "After {i} attempt(s), retrying was terminated by BackoffStrategy::interval()."
            ),
        }
    }
}
impl<E: Error> Error for BackoffError<E> {}
