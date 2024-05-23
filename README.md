# Zagreus
Both a reference to the rebirth of minor greek deity and the rogue-ness of the videogame he protaganizes, Zagreus is a featureful library
for backoffs in use at [Ceres](www.ceres.us) and in some of my personal projects.

The first version will be published very soon but for the moment this repo is just a stub.


# Usage:

Start by creating a BackoffStrategy. This is very simple and involves only defining a `limit()` to the number of attempts iterations
and an `interval()` method to calculate the time between said iterations.

Then, create a `Randomizer`. Use the library and source of randomness of your choosing: implementors
need only expose the `randomize` method, which takes an un-randomized Duration and returns a new one with the desired variance.

Once you have done this, the blanket implementation of BackoffHandler will generate an `handl()` method for your strategy. From there,
call `handle()` as needed.

While the API surface is large for what it is, this crate is intended to be used as a building block to cover the majority of behavior one may desire
in a retrying implementation. To wrangle this complexity it is recommended to hide implementors of BackoffHandler within another (new)type which
has the parameters for `handle()` predefined and stored within itself for each given usecase. 

