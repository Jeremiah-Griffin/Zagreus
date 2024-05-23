use std::{num::NonZeroU32, time::Duration};

pub trait BackoffStrategy: Send {
    fn interval(&self, attempts: u32) -> Duration;

    fn limit(&self) -> NonZeroU32;
}
///Retry strategy with backoffs that grow with n * i where n is the interval, and i is the number of requests.
pub struct Linear {
    ///The constant factor of the backoff. The first backoff interval will be this long.
    ///All successive intervals will be this value multiplied by the number of attempts.
    constant: Duration,
    ///The maximum number of retries that may be attempted.
    limit: NonZeroU32,
}

impl Linear {
    pub fn new(constant: Duration, limit: NonZeroU32) -> Self {
        Linear { constant, limit }
    }
}
impl BackoffStrategy for Linear {
    fn interval(&self, attempts: u32) -> Duration {
        self.constant.mul_f64(attempts as f64)
    }

    fn limit(&self) -> NonZeroU32 {
        self.limit
    }
}

///Retry strategy with backoffs that do not grow with time.
pub struct Constant {
    ///The fixed amount by which the backoffs will be spaced
    constant: Duration,
    ///The maximum number of retries that may be attempted.
    limit: NonZeroU32,
}

impl BackoffStrategy for Constant {
    fn interval(&self, _attempts: u32) -> Duration {
        self.constant
    }

    fn limit(&self) -> NonZeroU32 {
        self.limit
    }
}

impl Constant {
    pub fn new(constant: Duration, limit: NonZeroU32) -> Self {
        Constant { constant, limit }
    }
}

///Classic expontential backoff.
pub struct Exponential {
    ///The first backoff will take this long. Further backoffs
    ///will be multiplied by `factor` n times up to `limit`.
    constant: Duration,
    ///The factor by which the constant will grow.
    factor: NonZeroU32,
    ///The maximum number of retries that may be attempted.
    limit: NonZeroU32,
}

impl Exponential {
    pub fn new(constant: Duration, factor: NonZeroU32, limit: NonZeroU32) -> Self {
        Exponential {
            constant,
            factor,
            limit,
        }
    }
}

impl BackoffStrategy for Exponential {
    fn interval(&self, attempts: u32) -> Duration {
        self.constant.mul_f64((self.factor.get() * attempts) as f64)
    }

    fn limit(&self) -> NonZeroU32 {
        self.limit
    }
}
