# Zagreus

Both a reference to the rebirth of minor greek deity and the rogue-ness of the videogame he protaganizes, Zagreus is a featureful library
for retries, backoffs, and logging.

Please note this library still has some features missing and needs both examples and tests written.

# Features:

- `async`hronous API
- Runtime Independent & Zero Dependencies
- Logging
- Short circuiting on unrecoverable errors
- Interval randomization
- Zero Dependencies!

# Usage:

There are four traits with which users of Zagreus must familiarize themselves:

- `BackoffStrategy`: Defines the algorithm used to create retry intervals.
- `Randomizer`: Adds randomization to intervals generated by the BackoffStrategy. 
- `BackoffLogger`: Exposes methods for logging errors encountered both within and at the end of the attempt loop.
- `BackoffHandler`: Generates (and sleeps for) the amount of time produced by the BackoffStrategy as randomized by the Randomizer.

Start by creating a BackoffStrategy. This is very simple and involves only defining a `limit()` to the number of attempts iterations
and an `interval()` method to calculate the time between said iterations.

Then, create a `Randomizer`. Use the library, bounds, and source of randomness of your choosing: implementors
need only expose the `randomize` method, which takes an un-randomized Duration and returns a new one with the desired variance.

Finally, the implementation of a `BackoffLogger` will determine how, when, and which errors encountered during the attempt loop will be logged.

From this, each implementation will either be pased to the `BackoffHandler` or composed as a member of it to generate the desired behavior.

While the API surface is large for what it is, this crate is intended to be used as a building block to cover the majority of behavior one may desire
in a retrying implementation. To wrangle this complexity it is recommended to hide implementors of BackoffHandler within another (new)type which
has the parameters for `handle()` predefined and stored within itself for each given usecase. 

# FAQ:

Q: How may the interval between retries be capped?

A: Return `None` from `BackoffStrategy::interval()`.
There are too many distinct ways to track time than would be sound for this crate to opine upon. Instead, implementors of BackoffStrategy
may track the time from the first call to `interval()` to the final, and return `None` when this value exceeds some maximum that the developer is comfortable with. 

While this means any cost associated with timing requests is opt-in, it also makes cancelling in-flight requests that exceed the time alotted
by a given `BackoffStrategy` impossible.

Cancelling in-flight requests is outside the scope of this library and should be handled elsewhere.

Q: Why are `BackoffHandler` and `BackoffStrategy` separate traits?

A: It may be convenient to make a handler implement distinct strategies for distinct `fallible`s. Say one endpoint rate limits requests made sooner than one second apart and another two seconds. 
For yet another, performance contraints may demand the interval between attempts be capped at 500ms.

Additionally, If `BackoffHandler` and `BackoffStrategy` were one and the same each strategy used to address these constraints would need its own source of randomness: either contained within itself (and exposed by `BackoffHandler::randomizer()`)
or part of the the program's global state. Both of these solutions have negative design and performance implications for asynchronous, highly concurrent applications. 
Splitting these traits in two allows for easy customization of behavior per-endpoint without resorting to sourcing per-endpoint randomness.

Q: Zagreus is so small (~200 LOC) but the API is so big/ugly/unweildy. Why?

A: This library is meant to expose a very broad amount of functionality in a manner that is both lightweight and runtime-independent.
The onus of wrangling the APIs into something less annoying is on the user. Do not expect consumers of your types to interface with these traits directly, instead, implement types which
wrap them, encapsulating common functionality and exposing only the subset of what's left.
