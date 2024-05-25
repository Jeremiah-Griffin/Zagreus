pub use errors::{BackoffError, BackoffErrorKind};
pub use handler::BackoffHandler;
pub use logging::BackoffLogger;
pub use random::Randomizer;
pub use strategy::BackoffStrategy;

mod errors;
mod handler;
mod random;
mod strategy;
mod logging {
    use std::error::Error;

    use crate::BackoffError;

    pub trait BackoffLogger<E: Error> {
        fn log(error: BackoffError<E>);
    }
}
mod examples {
    //TODO: examples for each trait.
}
pub mod types;
