use std::time::Duration;

use crate::random::Randomizer;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
///A Randomizer that does nothing.
///This can save cycles and memory if randomization is deemed unnecessary by the developer.
pub struct NoRandomization {}

impl Default for NoRandomization {
    fn default() -> Self {
        NoRandomization {}
    }
}

impl Randomizer for NoRandomization {
    ///NotRandom always returns the interval it is given, doing nothing.
    fn randomize(&mut self, interval: Duration) -> Duration {
        interval
    }
}
