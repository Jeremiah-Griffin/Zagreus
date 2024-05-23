pub use errors::{BackoffError, BackoffErrorKind};
pub use handler::{BackoffHandler, BackoffStrategy, NotRandom, Randomizer};

mod errors;
mod handler;
mod strategy;
