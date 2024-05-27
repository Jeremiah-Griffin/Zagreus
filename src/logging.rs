use std::{error::Error, marker::PhantomData};

///TODO: Document this entire api.

//take cell implements new<T>() -> TakeCellState<T, UnOccupied>;
///takecell: TakeCell<T, TakeCellState::Occupied> exposes  a method: take.
//takecell: TakeCell<T, TakeCellState::UnOccupied> exposes a method: set.
//
//TakeCellState::Occupied implments deref and derefmut to &T and &mut T

///should lender and Logger be the same?
///it should look like this
///handle{
///let lender = Lender;
///
///`retry loop {
///    let loan = lender::lend();
///    
///    loan gets dropped and runds its implementation of drop (maybe pushes its error type to the lender? The lender stores it? or can throw it away)
///  }
///  lender.finalize(loan)
use crate::BackoffError;

///TODO: we dont need the Lender types as their only purpose was to store state.
///We dont need to store state as we can just call log on drop of the loan type.
///we implement filtering not by iterating over stored errors, but by switching on a filter
///within the implementation of drop.

///A `Logger` is passed into the Handle method. The `log()` trait method deterines
///what the `Logger` will do with the error, such as sending it to stdout or writing it to disk.
trait Logger<E: Error> {
    type Lender: Lender<E>;

    fn log(&self, error: &BackoffError<E>);
}

#[allow(drop_bounds)]
///We use the `Drop` bound to implement behavior at the end of each retry loop iteration.
///For example, if we want every error emitted within the loop to get pushed to a buffer, the `Loan` should keep a mutable reference
///to that buffer and and push its error to it. On the call to finalize - since we need to call log *before* the Loan gets dropped, we take a reference
///to the contained BackoffError<E>
trait ErrorLoan<E: Error>: Drop {
    fn error(&self) -> &BackoffError<E>;
}

///A `Lender` describes when a `Logger`'s `log` method gets invoked.
///Every `BackoffError` emitted internally to the retry loop will ge converted into a `Loan` implementing `ErrorLoan`.
trait Lender<E: Error>: Sized {
    type Loan: ErrorLoan<E>;
    fn new() -> Self;

    ///Creates the loan type.
    fn lend(&mut self, error: BackoffError<E>) -> Self::Loan;

    fn finalize(self, loan: Self::Loan, logger: &impl Logger<E>);
}

///A lender that logs only the final error
struct FinalErrorLender<E: Error> {
    phantom: PhantomData<E>,
}

impl<E: Error> Lender<E> for FinalErrorLender<E> {
    type Loan = FinalErrorLoan<E>;
    fn new() -> Self {
        //the lack of fields should make this a no op.
        FinalErrorLender {
            phantom: PhantomData::default(),
        }
    }

    fn lend(&mut self, error: BackoffError<E>) -> FinalErrorLoan<E> {
        FinalErrorLoan { error }
    }

    fn finalize(self, loan: FinalErrorLoan<E>, logger: &impl Logger<E>) {
        logger.log(loan.error());
    }
}

struct FinalErrorLoan<E: Error> {
    error: BackoffError<E>,
}

impl<E: Error> ErrorLoan<E> for FinalErrorLoan<E> {
    fn error(&self) -> &BackoffError<E> {
        &self.error
    }
}

impl<E: Error> Drop for FinalErrorLoan<E> {
    fn drop(&mut self) {
        ()
    }
}

impl<'a, E: Error> Lender<E> for AllErrorsLender<E>
where
    E: 'a,
{
    type Loan = AllErrorsLoan<'a, E>;
    fn new() -> Self {
        todo!()
    }

    fn lend(&'a mut self, error: BackoffError<E>) -> AllErrorsLoan<'a, E> {
        AllErrorsLoan {
            error: Some(error),
            lender: self,
        }
    }

    fn finalize(self, loan: AllErrorsLoan<'a, E>, logger: &impl Logger<E>) {
        //we call drop to make sure AllErrorsLoan gets pushed to the buffer.
        drop(loan);

        let Some(errors) = self.errors else {
            return;
        };

        errors.into_iter().for_each(|e| logger.log(&e));
    }
}

struct AllErrorsLender<E: Error> {
    errors: Option<Vec<BackoffError<E>>>,
}

struct AllErrorsLoan<'a, E: Error> {
    error: Option<BackoffError<E>>,
    lender: &'a mut AllErrorsLender<E>,
}

impl<'a, E: Error> ErrorLoan<E> for AllErrorsLoan<'a, E> {
    fn error(&self) -> &BackoffError<E> {
        &self.error.as_ref().unwrap()
    }
}

impl<'a, E: Error> Drop for AllErrorsLoan<'a, E> {
    fn drop(&mut self) {
        if self.lender.errors.is_none() {
            self.lender.errors = Some(Vec::new());
        }

        let errors = self.lender.errors.as_mut().unwrap();
        errors.push(self.error.take().unwrap())
    }
}

/*
struct FilteringLender<E: Error> {
    errors: Option<Vec<BackoffError<E>>>,
    filter: fn(error: &BackoffError<E>) -> bool,
}

struct FilteringLoan<'a, E: Error> {
    error: Option<BackoffError<E>>,
    lender: &'a mut FilteringLender<E>,
}

impl<'a, E: Error> ErrorLoan<E> for FilteringLoan<'a, E> {
    fn error(&self) -> &BackoffError<E> {
        self.error.as_ref().unwrap()
    }
}

impl<'a, E: Error> Drop for FilteringLoan<'a, E> {
    fn drop(&mut self) {
        if self.lender.errors.is_none() {
            self.lender.errors = Some(Vec::new());
        }

        let errors = self.lender.errors.as_mut().unwrap();
        errors.push(self.error.take().unwrap())
    }
}
*/
