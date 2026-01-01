# To-do List

This is general TODO across internal docs, manual, and software.

# Known Issues

## Keyboard

When transposition is in effect, it is not indicated on the web UI. There should be some indication of transposition, perhaps in the area, which can show the mappings in effect with their transposition. Trying to work it into the note name makes the note name too long and busy.

Bug: hexboard doesn't look good in light mode. Maybe I should hard-code dark mode since it matches the hardware.

# Software

* Create the interactive chord builder -- see below
* Create a `demo` mode. Embed examples/microtonal-hello.stq. Generate the stq, csound, mpe MIDI, and mts MIDI files, and suggest ways to play them back.
* Expand scripts in misc to support other than octave
* Consider bringing misc/exponent-to-ratio and misc/scale-semitones into the main CLI as a separate subcommand like `syntoniq calc`. If so, mention in the microtonal section of the manual.

# Documentation

* Pay special attention to "on active" or "on octave" instead of "an octave" and "ration" instead of "ratio"
* Find all occurrences of `TODO` in the docs.
* Tweak theme for better colors
* Remember not to use "DSL" in the docs.
* Figure out where to document the stuff in misc. Somewhere in the docs directory

# Release

* Use cargo-dist for creating distributions.

# Chord Builder

Consider an interactive mode for building chords using generated notes. For now, let this just be a CLI tool. It can be an alternative subcommand to syntoniq-kbd since it will share all its output options and use the same internal messaging system, or we can still have it be `run` and can make `--port` optional if `--prompt` is given.

```
!n              -- divisions = n (or ! for pure JI)
= pitch         -- set_base_pitch(absolute=pitch)
* pitch         -- set_base_pitch(relative=pitch)
note1 > note2   -- transpose(pitch_from=note1 written=note2)
note            -- play note
- n             -- stop playing note n
n < note        -- replace note n with a note
0               -- silence all notes
```
Each time a note is played, print a numbered list of all the notes currently played with useful metadata.

Example
```
>> A
1. A (base=220*^1|4)
>> E
1. A (relative=1 base=220*^1|4)
2. E (relative=3/2 base=220*^1|4)
>> C
1. A (relative=1 base=220*^1|4)
2. E (relative=5/4 base=220*^1|4)
3. C (relative=3/2 base=220*^1|4)
>> 2 < F
1. A (relative=1 base=220*^1|4)
2. F (relative=6/5 base=220*^1|4)
3. C (relative=3/2 base=220*^1|4)
>> * 9/8
Transposition is now 9/8.
>> A
1. A (relative=1 base=220*^1|4)
2. F (relative=6/5 base=220*^1|4)
3. C (relative=3/2 base=220*^1|4)
4. A (relative=1 base=220*^1|4 transpose=9/8)
>> = 264
Base pitch is now 264.
>> A
1. A (relative=1 base=220*^1|4)
2. F (relative=6/5 base=220*^1|4)
3. C (relative=3/2 base=220*^1|4)
4. A (relative=1 base=220*^1|4 transpose=9/8)
5. A (relative=1 base=264)
>> - 2
1. A (relative=1 base=220*^1|4)
2. C (relative=3/2 base=220*^1|4)
3. A (relative=1 base=220*^1|4 transpose=9/8)
4. A (relative=1 base=264)
>> 0
>> = 440*^-9|12
Base pitch is now 220*^1|4.
>> d > A
Transposition is now 3/4.
>> A
1. A (relative=1 base=220*^1|4 transpose=3/4)
>> !17
Divisions is now 17.
>> D
1. A (relative=1 base=220*^1|4 transpose=3/4)
2. D (relative=^7|17 base=220*^1|4 transpose=3/4)
```
