use std::{error::Error, marker::PhantomData};

use crate::errors::BackoffError;

use super::{strategies::AllErrorsStrategy, ErrorLoan};

pub(crate) struct FinalErrorLoan<'a, E: Error + Send> {
    pub error: BackoffError<E>,
    pub phantom: PhantomData<&'a str>,
}

impl<'a, E: Error + Send> ErrorLoan<'a, E> for FinalErrorLoan<'a, E> {
    fn error(&self) -> &BackoffError<E> {
        &self.error
    }
}

impl<'a, E: Error + Send> Drop for FinalErrorLoan<'a, E> {
    fn drop(&mut self) {
        ()
    }
}

pub(crate) struct AllErrorsLoan<'a, E: Error + Send> {
    pub error: Option<BackoffError<E>>,
    pub lender: &'a mut AllErrorsStrategy<'a, E>,
}

impl<'a, E: Error + Send> ErrorLoan<'a, E> for AllErrorsLoan<'a, E> {
    fn error(&self) -> &BackoffError<E> {
        &self.error.as_ref().unwrap()
    }
}

impl<'a, E: Error + Send> Drop for AllErrorsLoan<'a, E> {
    fn drop(&mut self) {
        if self.lender.errors.is_none() {
            self.lender.errors = Some(Vec::new());
        }

        let errors = self.lender.errors.as_mut().unwrap();
        errors.push(self.error.take().unwrap())
    }
}
