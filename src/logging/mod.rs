use std::error::Error;

mod test;

use crate::errors::BackoffError;

///Types implementing `BackoffLogger`
pub mod loggers;

#[allow(private_bounds)]
pub trait BackoffLogger<E: Error>: Send {
    ///Called for errors where attempt < limit *and* the error is not found to be unrecoverable
    ///by the `is_recoverable` and `peek_retry` callbacks on `BackoffHandler::handle()`;
    fn log_nonterminal(&mut self, error: &E, attempt: u32);

    ///Called for errors that are found to be unrecoverable either by the `is_recoverable` or `peek_retry` callbacks on
    ///`BackoffHandler::handle()` or when the `BackoffStrategy` has exhausted its retries.
    fn log_terminal(&mut self, error: &BackoffError<E>);
}
