use crate::errors::BackoffError;
use std::error::Error;
///Types implementing `BackoffLogger`
pub mod loggers;

///Defines the behavior for logging errors in the retry loop.
///It is strongly recommended that implementors either be cheap to construct or, if shared state is needed, it be wrapped in an `Arc` or similar.
#[allow(private_bounds)]
pub trait BackoffLogger<E: Error>: Send {
    ///Called for errors where attempt < limit *and* the error is not found to be unrecoverable
    ///by the `is_recoverable` and `peek_retry` callbacks on `BackoffHandler::handle()`;
    fn log_nonterminal(&self, error: &E, attempt: u32);

    ///Called for errors that are found to be unrecoverable either by the `is_recoverable` or `peek_retry` callbacks on
    ///`BackoffHandler::handle()` or when the `BackoffStrategy` has exhausted its retries.
    fn log_terminal(&self, error: &BackoffError<E>);
}
