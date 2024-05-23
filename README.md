# Zagreus
Both a reference to the rebirth of minor greek deity and the rogue-ness of the videogame he protaganizes, Zagreus is a featureful library
for backoffs in use at [Ceres](www.ceres.us) and in some of my personal projects.

The first version will be published very soon but for the moment this repo is just a stub.

# Features:
- Asynchronous API
- Short circuiting on errors that are unrecoverable
- Interval randomization
- Error and interval interception


# Usage:

There are three traits with which users of this crate must familiarize themselves:
`BackoffStrategy`: Defines the algorithm used to create backoff intervals.
`Randomizer`: Adds randomization to intervals generated by the BackoffStrategy. 
`BackoffHandler`: Generates (and sleeps for) the amount of time produced by the BackoffStrategy as randomized by the Randomizer.


Start by creating a BackoffStrategy. This is very simple and involves only defining a `limit()` to the number of attempts iterations
and an `interval()` method to calculate the time between said iterations.

Then, create a `Randomizer`. Use the library, bounds, and source of randomness of your choosing: implementors
need only expose the `randomize` method, which takes an un-randomized Duration and returns a new one with the desired variance.

Additionally, you may find it useful to override the default implementation of `BackoffHandler::log()` to hook your logging infrastructure into the Retry loop.

While the API surface is large for what it is, this crate is intended to be used as a building block to cover the majority of behavior one may desire
in a retrying implementation. To wrangle this complexity it is recommended to hide implementors of BackoffHandler within another (new)type which
has the parameters for `handle()` predefined and stored within itself for each given usecase. 
