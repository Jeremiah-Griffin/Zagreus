use std::error::Error;

mod test;

mod loans;
pub mod strategies;

use crate::errors::BackoffError;

//as long as the lending mechanism is ill-documented/reliant on Drop.
//TODO: to remove the Drop bound, we can create a lona wrapper type which calls the ErrorLoan's implementation some function (on_drop?) in its drop impl.
//I dont want BackoffLogger implementable by dependents.
#[allow(private_bounds)]
///A logger is comprised of two parts, a `LoggingStrategy` and an implementation of `Log`;
pub trait BackoffLogger<'a, E: Error> {
    ///The strategy used by this logger.
    type Strategy: LoggingStrategy<'a, E>;

    ///Invokes the `BackoffLogger` for the given implementation.
    fn log(&self, error: &BackoffError<E>);
}

#[allow(drop_bounds)]
///We use the `Drop` bound to implement behavior at the end of each retry loop iteration.
///For example, if we want every error emitted within the loop to get pushed to a buffer, the `Loan` should keep a mutable reference
///to that buffer and and push its error to it. On the call to finalize - since we need to call log *before* the Loan gets dropped, we take a reference
///to the contained BackoffError<E>
trait ErrorLoan<'a, E: Error>: Drop {
    fn error(&self) -> &BackoffError<E>;
}

///A `LoggingStrategy` describes when a `Logger`'s `log` method gets invoked.
///Every `BackoffError` emitted internally to the retry loop will ge converted into a `Loan` implementing `ErrorLoan`.
trait LoggingStrategy<'a, E: Error>: Sized {
    type Loan: ErrorLoan<'a, E>;
    fn new() -> Self;

    ///Creates the loan type.
    fn lend(&'a mut self, error: BackoffError<E>) -> Self::Loan;

    fn finalize(self, loan: Self::Loan, logger: &impl BackoffLogger<'a, E>);
}
