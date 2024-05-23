use std::{num::NonZeroU32, time::Duration};

pub trait BackoffStrategy {
    fn interval(&self, attempts: u32) -> Duration;

    fn limit(&self) -> NonZeroU32;
}

///Retry strategy with backoffs that grow with n * i where n is the interval, and i i sthe number of requests.
pub struct Linear {
    constant: NonZeroU32,
    limit: NonZeroU32,
}

impl Default for Linear {
    fn default() -> Self {
        Self {
            constant: NonZeroU32::new(10).unwrap(),
            limit: NonZeroU32::new(10).unwrap(),
        }
    }
}

impl BackoffStrategy for Linear {
    fn interval(&self, attempts: u32) -> Duration {
        Duration::from_millis(self.constant.get() as u64 * attempts as u64)
    }

    fn limit(&self) -> NonZeroU32 {
        self.limit
    }
}

///Retry strategy with backoffs that do not grow with time.
pub struct Constant {
    ///The fixed amount by which the backoffs will be spaced
    interval: Duration,
    ///The number of retries which will be attempted
    limit: NonZeroU32,
}

impl Default for Constant {
    fn default() -> Self {
        Constant {
            interval: Duration::from_millis(25),
            limit: NonZeroU32::new(10).unwrap(),
        }
    }
}

impl BackoffStrategy for Constant {
    fn interval(&self, _attempts: u32) -> Duration {
        self.interval
    }

    fn limit(&self) -> NonZeroU32 {
        self.limit
    }
}

pub struct Exponential {
    ///The first backoff will take this man milliseconds. Further backoffs
    ///will be multiplied by th factor n times up to limit.
    constant_millis: NonZeroU32,
    ///The factor by which the constant will grow.
    factor: NonZeroU32,
    ///The maximum number of retries which will be attempted
    limit: NonZeroU32,
}

impl BackoffStrategy for Exponential {
    fn interval(&self, attempts: u32) -> Duration {
        Duration::from_millis((self.constant_millis.get() * (self.factor.get() * attempts)) as u64)
    }

    fn limit(&self) -> NonZeroU32 {
        self.limit
    }
}

impl Default for Exponential {
    fn default() -> Self {
        Exponential {
            constant_millis: NonZeroU32::new(5).unwrap(),
            factor: NonZeroU32::new(2).unwrap(),
            limit: NonZeroU32::new(5).unwrap(),
        }
    }
}
