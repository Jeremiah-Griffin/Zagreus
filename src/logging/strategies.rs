use std::{error::Error, marker::PhantomData};

use crate::errors::BackoffError;

use super::{
    loans::{AllErrorsLoan, FinalErrorLoan},
    BackoffLogger, ErrorLoan, LoggingStrategy,
};

///A strategy that logs only if all retry attempts fail
pub struct FinalErrorStrategy<E: Error + Send> {
    phantom: PhantomData<E>,
}

impl<'a, E: Error + Send> LoggingStrategy<'a, E> for FinalErrorStrategy<E> {
    type Loan = FinalErrorLoan<'a, E>;
    fn new() -> Self {
        //the lack of fields should make this a no op.
        FinalErrorStrategy {
            phantom: PhantomData::default(),
        }
    }

    fn lend(&mut self, error: BackoffError<E>) -> FinalErrorLoan<'a, E> {
        FinalErrorLoan {
            error,
            phantom: PhantomData::default(),
        }
    }

    fn finalize(self, loan: FinalErrorLoan<'a, E>, logger: &impl BackoffLogger<'a, E>) {
        logger.log(loan.error());
    }
}

impl<'a, E: Error + Send> LoggingStrategy<'a, E> for AllErrorsStrategy<'a, E>
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

    fn finalize(self, loan: AllErrorsLoan<'a, E>, logger: &impl BackoffLogger<'a, E>) {
        //we call drop to make sure AllErrorsLoan gets pushed to the buffer.
        drop(loan);

        let Some(errors) = self.errors else {
            return;
        };

        errors.into_iter().for_each(|e| logger.log(&e));
    }
}

//replace this with a cell if possible
//take cell implements new<T>() -> TakeCellState<T, UnOccupied>;
///takecell: TakeCell<T, TakeCellState::Occupied> exposes  a method: take.
//takecell: TakeCell<T, TakeCellState::UnOccupied> exposes a method: set.
//
//TakeCellState::Occupied implments deref and derefmut to &T and &mut T
///A strategy that logs all errors encountered in the retry loop.
pub struct AllErrorsStrategy<'a, E: Error + Send> {
    pub(crate) errors: Option<Vec<BackoffError<E>>>,
    phantom: PhantomData<&'a str>,
}
