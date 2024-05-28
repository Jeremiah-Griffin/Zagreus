use std::error::Error;

use crate::{BackoffError, BackoffLogger};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
///A logger that does nothing and costs nothing.
pub struct NoLogging {}

impl Default for NoLogging {
    fn default() -> Self {
        NoLogging {}
    }
}

impl<E: Error> BackoffLogger<E> for NoLogging {
    fn log_nonterminal(&self, _error: &E, _attempt: u32) {
        ()
    }

    fn log_terminal(&self, _error: &BackoffError<E>) {
        ()
    }
}
