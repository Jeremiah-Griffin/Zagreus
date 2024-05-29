#[cfg_attr(feature = "nightly_can_backoff", feature(negative_impls))]
#[cfg_attr(feature = "nightly_can_backoff", feature(auto_traits))]
pub use errors::BackoffError;
pub use handler::BackoffHandler;
pub use logging::loggers;
pub use logging::BackoffLogger;
pub use random::randomizers;
pub use random::Randomizer;
pub use strategy::strategies;
pub use strategy::BackoffStrategy;

mod errors;
mod handler;
mod logging;
mod random;
mod strategy;
