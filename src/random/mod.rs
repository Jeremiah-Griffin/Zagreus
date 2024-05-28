use std::time::Duration;
pub mod randomizers {
    use std::time::Duration;

    use crate::random::Randomizer;

    ///A Randomizer that does nothing.
    ///This can save cycles and memory if randomization is deemed unnecessary by the developer.
    pub struct NoRandomization {}

    impl NoRandomization {
        pub fn new() -> Self {
            NoRandomization {}
        }
    }

    impl Randomizer for NoRandomization {
        ///NotRandom always returns the interval it is given, doing nothing.
        fn randomize(&mut self, interval: Duration) -> Duration {
            interval
        }
    }
}

///Provides the interface by which BackoffHandlers can add randmoness to their retries.
///Randomization is useful to prevent multiple requests to an endpoint which fail concurrently
///from retrying the endpoint concurrently, which may exacerbate the problem or even trigger DoS protections.
pub trait Randomizer {
    ///Given an interval, applies a function to randomize it, returning the randomized interval.
    fn randomize(&mut self, interval: Duration) -> Duration;
}
