use std::{
    cell::{Cell, OnceCell},
    error::Error,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

//take cell implements new<T>() -> TakeCellState<T, UnOccupied>;
///takecell: TakeCell<T, TakeCellState::Occupied> exposes  a method: take.
//takecell: TakeCell<T, TakeCellState::UnOccupied> exposes a method: set.
//
//TakeCellState::Occupied implments deref and derefmut to &T and &mut T

///should lneder and Logger be the same?
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

///finlize runs the code which sends the error to the logger. Maybe this is printing to eprintln? if so, a ManyLogger
///would flush all the errors stored. a FinalError would print onyl the loan pased in.
///}
use crate::BackoffError;

trait Logger {
    fn log<E>(error: BackoffError<E>) {}
}

struct FinalErrorLender<E: Error, L: Logger> {
    phantom: PhantomData<(E, L)>,
}

trait Lender<E: Error> {
    type Loan;

    fn new() -> Self;

    ///Creates the loan type.
    fn lend(&mut self, error: BackoffError<E>) -> Self::Loan;

    fn finalize() {}
}

impl<E: Error, L: Logger> Lender<E> for FinalErrorLender<E, L> {
    type Loan = FinalErrorLoan<E>;

    fn new() -> Self {
        //the lack of fields should make this a no op.
        FinalErrorLender {
            phantom: PhantomData::default(),
        }
    }

    fn lend(&mut self, error: BackoffError<E>) -> Self::Loan {
        FinalErrorLoan { error }
    }
}

struct FinalErrorLoan<E: Error> {
    error: BackoffError<E>,
}

impl<E: Error> FinalErrorLoan<E> {
    pub fn new(error: BackoffError<E>) -> Self {
        FinalErrorLoan { error }
    }

    pub fn error(&self) -> &BackoffError<E> {
        &self.error
    }

    pub fn into_error(self) -> BackoffError<E> {
        self.error
    }
}

struct AllErrorsLender<E: Error> {
    errors: Option<Vec<BackoffError<E>>>,
}

impl<E: Error> AllErrorsLender<E> {
    pub fn new() -> Self {
        //the lack of fields should make this a no op.
        AllErrorsLender { errors: None }
    }

    pub fn lend<'a>(&'a mut self, error: BackoffError<E>) -> AllErrorsLoan<'a, E> {
        AllErrorsLoan {
            error: Some(error),
            logger: self,
        }
    }
}

struct AllErrorsLoan<'a, E: Error> {
    error: Option<BackoffError<E>>,
    logger: &'a mut AllErrorsLender<E>,
}

impl<'a, E: Error> AllErrorsLoan<'a, E> {
    pub fn new(error: BackoffError<E>, logger: &'a mut AllErrorsLender<E>) -> Self {
        AllErrorsLoan {
            error: Some(error),
            logger,
        }
    }
}

impl<'a, E: Error> Drop for AllErrorsLoan<'a, E> {
    fn drop(&mut self) {
        if self.logger.errors.is_none() {
            self.logger.errors = Some(Vec::new());
        }

        let errors = self.logger.errors.as_mut().unwrap();
        errors.push(self.error.take().unwrap())
    }
}

///TODO figure out a trait solution that will log an error only once,
///and one will log all errors accumulated ina  store. Both will implement a method to collect
///types, and both will impl a method to present types (either as a slice of BackoffError<E> or a single BackoffError<E>)
///
///depending on those traits - as parameter of the logger - the implementation of drop will call log. In the case of the single error, only one E will be stored and tht one (the latest) will get logge.
///in the other one,  it will call log on each error in the slice.
pub trait BackoffLogger<E: Error> {
    fn log(error: BackoffError<E>);
}

struct BackoffErrorLogger<E: Error> {
    error: E,
}

impl<E: Error> Drop for BackoffErrorLogger<E> {
    fn drop(&mut self) {}
}
