# Ideas

This is a repository of ideas that may or may not ever be worth building.

# Printed Scores

Muse about printed notation hints. We could potentially generate MusicXML, LilyPond, or add enough metadata to the timeline JSON dump that someone could do their own notation from it. This is probably a non-goal though, especially with focus on harmonic sequence.

# Piano Keyboard Layout

This idea could be usable for any scale that maps nicely to diatonic (12, 17, 19, 31, 24, 36, 41, 48, 51, 53, 72, a few others) as long as max(scale-degrees-to-semitone, ceil(scale-degrees-to-whole-tone/2)) < number of octaves on the piano.

* note 60 (middle C) is middle base pitch
* half step (white key to black key above it, e to f, b to c) is diatonic semitone
* whole step is whole step
* octave is single step

This gives you something kind of almost like an isomorphic keyboard in feel. For example, in 31-EDO, note 60 would be C, note 61 would be 3 scale degrees up, note 62 would be 5 scale degrees up, ..., note 72 would be one scale degree up, etc. What you'd have with this layout scheme is that the middle C octave would play like a "regular" scale, and the octave to the right would be up one step, etc.

Using my notation of A0 = root, A1 = one step, A2 = 2 steps, etc., 19-EDO would map to the middle C octave and the one above it like this:
```
  A2  A5      A10  A13  A26       A3  A6      A11  A14  A27
A0  A3  A6  A8  A11  A14  A17   A1  A4  A7  A9  A12  A15  A18
```

This would not be the same experience as a hex grid, but it allow you to play 12-tone-like music in different tuning systems. Take the simplest case of 24-EDO: the middle C/note 60 octave would be as it is on a regular keyboard. The octave above would be the same octave shifted up a quarter tone. The octave behind that would be one octave above middle C, etc. For a piano keyboard, this would make those particular EDO flavors manageable.

This would require a different type of layout that wasn't a grid, and we'd have to know about half-step and whole-step intervals, but it would be a relatively light lift.

We'd have to solve function keys (reset, layout selection, octave shift, sustain, shift, transpose), but this could be done in various ways. We have pedals and other things that generate MIDI events that could be mapped.
