use std::{num::NonZeroU32, time::Duration};
pub mod strategies;

pub trait BackoffStrategy: Send + Sync {
    ///The interval computed for the attempts + 1 backoff.
    ///Returning Some(_) will progress the attempt loop once more with the contained Duration.
    ///Returning None will case the loop to halt immediately. This can be useful for implementing timeouts.
    fn interval(&self, attempts: u32) -> Option<Duration>;

    ///The maximum number of retries that may be attempted by a handle using this Strategy.
    fn limit(&self) -> NonZeroU32;
}
