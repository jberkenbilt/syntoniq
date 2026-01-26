# To-do List

This is general TODO across internal docs, manual, and software.

# Glide

Modify NoteEvent to contain an array of pitches with time offsets from the beginning of the note.

# Pre-1.0

These are proposed pre-1.0 items. Details are below for many.

* Create a minimal emacs mode
* Glide
* Fix edge cases -- see copilot-initial-review.md
* calc -- from scripts in misc
* Maybe: interactive chord builder

# Build/CI

* Resolve manual generation in CI
* Do the lychee check in build_all

# Video

* Delete ~/Q/video/syntoniq/rendered/old/ when sure
* Could add TOC. Example format:

```
00:00 Introduction
02:15 Connecting the HexBoard
05:40 The Web Interface
```

## Keyboard

When transposition is in effect, it is not indicated on the web UI. There should be some indication of transposition, perhaps in the area, which can show the mappings in effect with their transposition. Trying to work it into the note name makes the note name too long and busy.

Bug: hexboard HTML doesn't look good in light mode. Maybe I should hard-code dark mode since it matches the hardware.

# Software

* See [Copilot Initial Review](copilot-initial-review.md) for things found by GitHub copilot. Some of these are worth doing. All are already on my radar.
* Morphing
  * The note modifier `&` (?), mutually exclusive with `~` means to glide exponentially (perceptually linearly) from the pitch of this note to the pitch of the next note over the specified duration. The Csound instrument syntax has room for this, and it should be easy with Csound. For MIDI, we'll need to ramp with pitch bend.
* Create the interactive chord builder -- see below
* Expand scripts in misc to support other than octave
* Consider bringing misc/exponent-to-ratio and misc/scale-semitones into the main CLI as a separate subcommand like `syntoniq calc`. If so, mention in the microtonal section of the manual. Also add something to show the base pitch of a generated note, e.g. `JK!17` should show `^5|17` or `jI` should show `81/80`.
* Csound: maybe: interpret accents with envelope, then figure out what this does to articulation adjustment.
* Articulation adjustment directives:
  * four factors: default of each plus modifier for each option
    * default velocity (72)
    * accent velocity (96)
    * marcato velocity (108)
    * staccato shorten amount (1/4 beat)
  * Can be applied globally or at the part level
* MIDI:
  * generate tuning files for midi by port and channel
  * generate summaries of part -> track/port/channel, etc.
* Note: not tested (generator):
  * MPE: more than 16 channels; multi-port
* Editing experience
  * Write LSP
  * Reformatting -- see below

# Documentation

* Have something that checks link integrity (internal and external)
* Embed KeTeX rather than getting from a CDN
* Pay special attention to "on active" or "on octave" instead of "an octave" and "ration" instead of "ratio"
* Find all occurrences of `TODO` in the docs.
* Tweak theme for better colors
* Figure out where to document the stuff in misc. Somewhere in the docs directory
* Remember https://gemini.google.com/app/81c4b4fb40317cdf for parsing blog. Gemini stuck something in Google Keep. Main thrust is justification for 100% code coverage

# Release

```
cargo build --workspace --all-targets --release
cargo build --target-dir target.x86 --target x86_64-apple-darwin --workspace --all-targets --release
lipo -create -output syntoniq-kbd target/release/syntoniq-kbd target.x86/x86_64-apple-darwin/release/syntoniq-kbd
```

* Use cargo-dist for creating distributions.

# Reminders

On Linux, you can watch Syntoniq's MIDI output with `aseqdump`, e.g.:
```sh
aconnect -l
aseqdump -p 128:0
```

# Reformatter

These notes predate parser implementation.

* Use a lossless token stream for the reformatter.
* Reformatter: Two-pass (Parse-to-AST, then Token-Walk + AST-Peek).

The reformatter will first fully validate an input file. Then it will drive the formatting from pass1 tokens, peeking at later parsing results for semantic information as needed.

Suggested reformatting rules:
* Collapse multiple blank lines to single blank lines, and remove leading and trailing blank lines
* Remove trailing white space
* In a multi-line structure (score block, scale definition block, multiline directive), keep comments aligned and offset by two spaces from the longest line
* Remove spaces from around `=` in directive parameters
* If a directive with any trailing comment exceeds 100 columns, move the trailing comment to the preceding line. If still over 100 columns, break the directive to one parameter per line.
* If a directive that contains no parameter-level comments fits on one line in <= 100 columns, reformat as a single line. Never move a preceding comment to after a single-line directive.
* Apply alignment to score blocks as above
* Within scale definition blocks, right-justify pitches or indices with columns, then align and left-justify note names

It would be nice to have tool support for alignment. Within a score block, align notes so the beginning of the pitch part of notes or the location part of dynamics are aligned rhythmically after any beat markers as in the above examples. See below for an algorithm.

The DSL interpreter should have some commands to check and align. I could run C-c C-f on a score line, and it could either reformat or generate output with embedded comments containing any error messages. No reason to integrate with flycheck, etc.

Other notes:
* If there are bar checks, this algorithm can be applied to each "bar" and spaces can be added before the bar checks to force the bar checks to align.
* To align, calculate total number of discrete ticks (GCD of duration denominators * total beats)
* For each note, get number of characters before and after alignment point; `:`, `@` count as before since some notes won't have an explicit mark
* prepend/append space so all notes are the same width and have the alignment point in the same spot
* prepend each note with one extra space
* place notes based on numerator of n/GCD
* shrink vertical columns of spaces to width of 1
* align `]` of part names, prepending leading space

Example:

```
[treble] 1:e a 2/3:g f d
[bass] 2:c, 1:f, g,
```
* max characters before alignment marker = 4 (`2/3:`)
* max after alignment marker = 2 (`c,`)
* combined width = 4 + 2 = 6
* total beats = 4
* gcd of denominators = 3
* discrete ticks = 12. Each beat is 3 ticks.
* each note width, including leading space, is 7 (1+2+6)
* beat marker goes at position 4 (from 0)
* spaces except separator space shown below as `_`

Step 1: place each padded note based on its start position

```
|0     |1     |2     |3     |4     |5     |6     |7     |8     |9     |10    |11
 __1:e_               ____a_               2/3:g_        ____f_        ____d_
 __2:c,                                    __1:f_               ____g,
```

Step 2: shrink space columns
```
 1:e  a 2/3:g f   d
 2:c,     1:f  g,
```

Step 3: prepend `]`-aligned part names:
```
[treble] 1:e  a 2/3:g f   d
  [bass] 2:c,     1:f  g,
```

For dynamics, like the `@` up with the beat but only add spaces as necessary to keep dynamics from colliding so they don't make things too wide. See examples/hello.stq.

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
