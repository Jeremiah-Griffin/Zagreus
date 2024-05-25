use std::time::Duration;

///Provides the interface by which BackoffHandlers can add randmoness to their retries.
///Randomization is useful to prevent multiple requests to an endpoint which fail concurrently
///from retrying the endpoint concurrently, which may exacerbate the problem or even trigger DoS protections.
pub trait Randomizer {
    ///Given an interval, applies a function to randomize it, returning the randomized interval.
    fn randomize(&mut self, interval: Duration) -> Duration;
}
