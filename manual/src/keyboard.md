TODO:
* Command line
* UI features
  * double sustain
  * sustain
  * transpose, shift as modifier and not
  * check engine for full behavior
* color scheme

Note somewhere that we show the normalized base interval on the web UI but the full base factor on print. The rationale is that, when looking for notes to play on the keyboard, it's easier to find the normalized interval, and it also keeps the second line of note labels manageable. On stdout, we want the full information. In both cases, the number of cycles is encoded in octave markers. This is potentially confusing, but it feels like the right call.

Make sure to fully explain the semantics of the octave marks and arrows on the keyboard. Octave marks show the pitch offset in cycles relative to the note in the original scale and are used with isomorphic mappings. In manual mappings, notes are always shown as they appear in the mapping. Arrows indicate the tiling offsets. The displayed pitch is within cycle and relative to the base pitch of the original, *non-tiled* note. So a note that's naturally up 3/2 from the base and additionally up another 3/2 from tiling will show 9/8, not 9/4.
