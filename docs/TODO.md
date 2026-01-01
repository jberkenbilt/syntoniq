# To-do List

Work in things from docs/scratch

## Software

* Create a `demo` mode. Embed examples/microtonal-hello.stq. Generate the stq, csound, mpe MIDI, and mts MIDI files, and suggest ways to play them back.
* Expand scripts in misc to support other than octave

## Documentation

* To cover
  * Other features
    * Mention about decimal syntax
    * Dynamics
    * Mark, repeat, start/end mark, skip-beats; mention about timeline, not token stream. It has to be understood that repeats are temporal repeats, not lexical repeats. If you think about it this way, the constraints are logical.
      * Disadvantages
        * All logic around resolving pending ties and dynamic changes have already been completed, which means a tie in effect at the end mark would already have been resolved to something after the repeat. This is okay except it complicates things like having a tie right before a repeat and a matching tie at the first ending. This can be handled in other ways, but it might be possible to make the logic more sophisticated. It might be possible to do dynamic/tie resolution and overlapping tempo detection as a post-processing step, which could give us a middle-ground between what is there now and making repeats lexical. This would create the need for additional timeline events, so we would probably want to create a parallel set of events and have `into_timeline` only return events that are intended for the generators. This may not be worth ever doing.
      * Advantages
        * A repeated section gets to stay in whatever tuning it has.
        * There's never any question about whether something may be syntactically or semantically valid in a repeat. This is probably enough of an advantage to override the disadvantages.
    * Sustain note across tuning changes
    * Articulation
    * Gradual tempo change
    * Multiple parts
* Keyboard -- see keyboard/_index.md

* Pay special attention to "on active" or "on octave" instead of "an octave" and "ration" instead of "ratio"
* Find all occurrences of `TODO` in the docs.
* Tweak theme for better colors
* Remember not to use "DSL" in the docs.
* Figure out where to document the stuff in misc

## Release

* Use cargo-dist for creating distributions.
