use std::{fmt::Display, num::NonZeroU32};

#[derive(Debug)]
pub struct BackoffError<E> {
    pub error: E,
    pub kind: BackoffErrorKind,
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
}

impl<E: std::error::Error> Display for BackoffError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
        BackoffErrorKind::Unrecoverable(i) => write!(f, "After {i} attempts Encountered unrecoverable error: {:?}", self.error),
        BackoffErrorKind::ExhaustedLimit(i) => write!(f, "Limit of {} was exhausted. Error: {:?}", i.get(), self.error),
        BackoffErrorKind::UnrecoverableAndExhaustedLimit(i ) => write!(f, "An unrecoverable error was encountered and the limit of {} was exhausted. Error: {:?}", i.get(), self.error),
    }
    }
}
impl<E: std::error::Error> std::error::Error for BackoffError<E> {}
