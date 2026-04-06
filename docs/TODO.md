# To-do List

This is general TODO across internal docs, manual, and software.

# Early Issues

* Need volume control for syntoniq-kbd regular and prompt.
* Erroneous "unknown part" error when part appears outside of playing region as a dynamic line that doesn't otherwise appear in the region but appears before it.
* When starting in the middle of a sustained note with a mark, the note should be on; advance the start time of notes in progress. When ending in the middle of a sustained note, the truncate the duration.
* Allow mark to take an offset
* Would be nice to override the instrument for the keyboard.

# Pre-1.0

These are proposed pre-1.0 items. Details are below for many.

* Create a minimal emacs mode

# Build/CI

* Do the lychee check in build_all

# Software

* See [Copilot Initial Review](copilot-initial-review.md) for things found by GitHub copilot. Some of these are worth doing. All are already on my radar.
* Csound: maybe: interpret accents with envelope, then figure out what this does to articulation adjustment.
* Articulation markers control note length, attack velocity, and release velocity.
    * default: full length, 72 attack, 64 release
    * accent:  96 attack
    * marcato: 108 attack, 96 release
    * staccato: each repetition shortens note by 1/4 beat and adds 32 to release, capping at 127
    * tenudo: each repetition subtracts 32 from release as long as >= 0
  * Csound: these translate to channels and are normalized from 0.0 to 1.0 and are up to the instrument to interpret. The default instrument uses attack velocity to control the length and peak of the attack phase and the release to control the length and slope of the release phase.
  * MIDI: these translate to velocity on note on and note off events.
  * Add directives to change the numbers globally and at the part level
* MIDI:
  * generate tuning files for midi by port and channel
  * generate summaries of part -> track/port/channel, etc.
* Note: not tested (generator):
  * MPE: more than 16 channels; multi-port
* Editing experience
  * Write LSP
  * Reformatting -- see below

## Keyboard

When transposition is in effect, it is not indicated on the web UI. There should be some indication of transposition, perhaps in the area, which can show the mappings in effect with their transposition. Trying to work it into the note name makes the note name too long and busy.

Bug: hexboard HTML doesn't look good in light mode. Maybe I should hard-code dark mode since it matches the hardware.

# Documentation

* Clean up docs/architecture.md
* Have something that checks link integrity (internal and external)
* Embed KeTeX rather than getting from a CDN
* Pay special attention to "on active" or "on octave" instead of "an octave" and "ration" instead of "ratio"
* Tweak theme for better colors
* Figure out where to document the stuff in misc. Somewhere in the docs directory
* Remember https://gemini.google.com/app/81c4b4fb40317cdf for parsing blog. Gemini stuck something in Google Keep. Main thrust is justification for 100% code coverage

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
