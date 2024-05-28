use std::time::Duration;
///Types implementing `Randomizer`.
pub mod randomizers;

///Provides the interface by which BackoffHandlers can add randmoness to their retries.
///Randomization is useful to prevent multiple requests to an endpoint which fail concurrently
///from retrying the endpoint concurrently, which may exacerbate the problem or even trigger DoS protections.
pub trait Randomizer {
    ///Given an interval, applies a function to randomize it, returning the randomized interval.
    ///
    ///Most RNGs require a mutable reference to self. Implementors should use a synchronization
    ///primitive like mutex to provide interior mutability. A RwLock is discouraged
    ///as `BackoffHandler` never takes shared/immutable reference to a `Randomizer`,
    ///rendering the overhead of a more complex primitive wasted.    
    fn randomize(&self, interval: Duration) -> Duration;
}
