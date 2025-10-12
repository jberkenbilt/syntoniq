# TODO

Tempo, then Reaper. And doc cleanup.

----------

Oseh Shalom as possible JI demo piece: `1:g, c c b%, | 4:c | 1:g, b%, b%, a, | 4:b%, |` ...
Anything with modulations.

```
1:g  b% a% g 4:f
2:e%    e%   4:e%
2:c     c    4:c
2:~     b%,  4:a%,
```

Fuzz testing. A file that ends in the middle of a directive panics. The panic would have been clearer with context as well.

Finish doc so `syntoniq doc` gives more than just directives.

Current state:
* Next steps
  * Timeline
    * Backend order: JSON, MIDI, csound; target MTS instruments, use velocity for attack only, and play with either CC#7 or CC#11 or both for dynamics.
  * Work on reformatter
    * Should be able to work from token stream but only after validation has passed
  * Generate music via timeline
* Remaining syntax
  * tempo, accel, decel
  * morph, strum
  * define_instrument, use_instrument
  * marks, regions, repeats

Next, I would like create a timeline capable of expressing everything we have so far, perhaps taking it end to end through csound and midi. Then, as new features are added, they can be added end to end. This will make it easier to test new features and result in more incremental or less refactoring.

See
* ~/Downloads/MIDI Tuning Updated Specification.pdf
* ~/source/examples/rust/midi

Basic strategy: figure out total number of channels needed for 10 octaves of a scale, then track the number of simultaneous (instrument, scale, base_pitch) triples. This determines number of simultaneous channels, which in turn determines number of ports. Try to keep all channels for a part on the same port.

XXX Document highlight-next-span in a dev/test README -- can refer to comment in code

# Keyboard thoughts

We could replace the toml file for configuring the keyboard with something based on stq syntax.

Add a layout definition.
```
define_isomorphic_layout(
    shape="square" ; or "hex"
    rows=n cols=n
    scale="scale"
    base-row=n base-col=n ; see below for row/col
    steps-h=n steps-v=h   ; closest to h/v in the upper-left quadrant
)

define_manual_layout(
    shape="square" ; or "hex"
    rows=n cols=n
    scale="scale"
    base-row=n base-col=n ; see below for row/col
    cycle-h=n cycle-v=n   ; see below
) <<< ... >>>
```

With isomorphic keyboards, for square keyboards, `steps-h` and `steps-v` are self-explanatory, row 1 is the bottom row, and column 1 is the left column.

For hex keyboards, consider these two layouts:

```
                           ⌈S⌉   ⌈X⌉   ⌈γ⌉
<NN><OO><PP><QQ><RR>       ⌊S⌋|U⌉⌊X⌋⌈Z⌉⌊γ⌋
  |JJ><KK><LL><MM>         ⌈R⌉⌊U⌋⌈W⌉⌊Z⌋⌈β⌉
<EE><FF><GG><HH><II>       ⌊R⌋|T⌉⌊W⌋⌈Y⌉⌊β⌋
  <AA><BB><CC><DD>         ⌈Q⌉⌊T⌋⌈V⌉⌊Y⌋⌈α⌉
                           ⌊Q⌋   ⌊V⌋   ⌊α⌋
```

To assign rows and columns unambiguously, we create a staggered grid where either every column has only even/odd rows or every row has only even/odd columns. In the left layout, keys have immediate horizontal neighbors. In this case, odd rows have only even columns, and even rows have only odd columns.

* A = row 1, column 2
* B = row 1, column 4
* E = row 2, column 1
* J = row 3, column 2
* N = row 4, column 1

Horizontal would be in the direction from A to B, and vertical would be the closest to vertical in the upper-left quadrant, so it would be the direction from A to F.

In the right layout, keys have immediate vertical neighbors. In this case, odd rows have only odd columns, and even rows have only even columns

* Q = row 1, column 1
* R = row 3, column 1
* S = row 5, column 1
* T = row 2, column 2
* U = row 2, column 4

Vertical would be in the direction from Q to R, and horizontal would be from Q to T.

I feel like this is the only sensible way to do it unambiguously. Otherwise, the coordinates depend on whether start with column first or row first.

For isomorphic layouts, the above is sufficient to fully describe the keyboard. For non-isomorphic layouts, we need some way of assigning notes to keys. Also, we can assign keys to only a fraction of the keyboard, and then repeat the pattern n cycles up in a given direction. The repetition period would be determined by the number of rows and columns defined in the layout. For hex keyboards, there needs to be an even number of rows if cycle-v is specified and an even number of columns if cycle-h is specified. In all cases, a rectangular region has to be specified, meaning each row has to have the same number of specified columns.

I'm thinking `<<<` and `>>>` could be keyboard layout delimiters, and the syntax could be note names or `~` for unused in the layout of the keyboard.

For example, this toml:

```toml
[[scale]]
name = "JI-11"
type = "Generic"
base_pitch = "220*^3|12"
pitches = [
  "[61]*2", "[62]*2", "[63]*2", "[64]*2", "[65]*2", "[66]*2", "[67]*2", "[68]*2",
  "[51]*2", "[52]*2", "[53]*2", "[54]*2", "[55]*2", "[56]*2", "[57]*2", "[58]*2",
  "17/16",  "16/15",   "6/5",   "11/8",   "17/12",  "8/5",     "7/4",   "16/9",
   "1",      "9/8",    "5/4",    "4/3",    "3/2",   "5/3",    "15/8",    "2",
  "[61]*1/2", "[62]*1/2", "[63]*1/2", "[64]*1/2", "[65]*1/2", "[66]*1/2", "[67]*1/2", "[68]*1/2",
  "[51]*1/2", "[52]*1/2", "[53]*1/2", "[54]*1/2", "[55]*1/2", "[56]*1/2", "[57]*1/2", "[58]*1/2",
  "[41]*1/2", "[42]*1/2", "[43]*1/2", "[44]*1/2", "[45]*1/2", "[46]*1/2", "[47]*1/2", "[48]*1/2",
  "[31]*1/2", "[32]*1/2", "[33]*1/2", "[34]*1/2", "[35]*1/2", "[36]*1/2", "[37]*1/2", "[38]*1/2",
]
note_names = [
 "C♯'", "D♭'", "E♭'", "H11'", "F♯'", "A♭'", "H7'", "B♭'",
 "C'",  "D'",  "E'",  "F'",   "G'",  "A'",  "B'",  "C'",
 "C♯", "D♭", "E♭", "H11", "F♯", "A♭", "H7", "B♭",
 "C",  "D",  "E",  "F",   "G",  "A",  "B",  "C",
 "C♯,", "D♭,", "E♭,", "H11,", "F♯,", "A♭,", "H7,", "B♭,",
 "C,",  "D,",  "E,",  "F,",   "G,",  "A,",  "B,",  "C,",
 "C♯,,", "D♭,,", "E♭,,", "H11,,", "F♯,,", "A♭,,", "H7,,", "B♭,,",
 "C,,",  "D,,",  "E,,",  "F,,",   "G,,",  "A,,",  "B,,",  "C,,",
]

[[layout]]
name = "Just Intonation (11)"
scale_name = "JI-11"
```

would be replaced with this:
```
define_scale(name="JI-11") <<
 1     c  | 17/16  c#  | 16/15  d%
 9/8   d  |  6/5   e%
 5/4   e
 4/3   f  | 11/8   h11 | 17/12  f#
 3/2   g  |  8/5   a%
 5/3   a  |  7/4   h7  | 16/9   b%
15/8   b
>>
define_generic_layout(
   shape="square" rows=8 cols=8
   base-row=5 base-col=1
   scale="JI-11" cycle-v=1
) <<<
c# d% e% h11 f# a% h7 b%
c  d  e  f   g  a  b  c'
>>>
```

We can ignore layouts in scores and pay attention to only scales and layouts for the keyboard. That way you can define layouts for the scales and tunings used in a particular file, and the transposition and shift features in the keyboard can work to replace the idea of tunings. We could even potentially have the keyboard output tuning directives in response to transposition events, which makes the keyboard even more useful for transcribing notes.

This also makes the keyboard generalizable to other keyboard layouts. You just have to write a driver for the specific keyboard. When using a particular keyboard, only look at layouts that can be applied to that keyboard. A keyboard driver can accept or reject a given layout.

## Timeline

Score parser should generate a timeline that is fully ready for generating various types of output. That's where the responsibility of the parsing module in the common crate should end.

The timeline should contain enough information to generate csound, midi, and some kind of JSON dump that we can use for testing and debugging. Things in the timeline:

* Header: source file
* Timeline event:
 * span of triggering event in source
 * offset in ticks
 * metadata
 * similar to MIDI tracks, we have a global timeline with things like tempo and a timeline for each part
 * also global section for tunings and instruments
 * sorted by time, then span
* Tuning
 * Need to capture the chain of tunings for the base pitch so we can have a comment like `17-EDO, up ^2|19 from 'e' from default` or show the base pitch as `220*^1|4*^1|3*^2^19` or both.
 * Pitches should be shown in comments factor from base pitch, e.g. `*9/8` or `*^2|19`
 * Perhaps have a catalog of unique tunings, perhaps in order of appearance, and set a tuning based on that to reduce duplication and make it easier to generate tuning files. If we keep a uuid from an atomic counter on tune generation, this becomes easy; we just have to keep the predecessor's ID when `base_factor` or `base_note` is used.
* Notes
 * organized by part
 * name
 * pitch as
   * factor of base
   * Hz
   * scale degree with zero as base pitch and pitches sorted numerically +/- cycle. If we allow pitches lower than the base pitch, I think it's okay. The base pitch still gets zero. So if you had a scale like `<< ^-1|3 q | 1 r | ^1|3 s >>`, then `q` would be -1, `r` would be 0, and `s` would be 1. Then `q'` would be 2, and `s,` would be -2. This can be used to map to midi note numbers
 * on|off + "velocity"
 * other details attached to the note
* Dynamics -- need a way to represent crescendo, decrescendo
* Instrument assignment, metadata changes (strum, morph)
* Comments from source are not preserved here but are preserved by the reformatter

End goal: output generators should be able to go straight to output without encountering any error conditions.

Example:
* Csound: can create a separate instrument for each source instrument or maybe for each part if we can share csound instruments easily. Our instruments have to support certain things...will have to design that part, but that can likely be done after generating the timeline.
* MIDI: can allocate tracks and channels based on number of simultaneous notes, instruments, etc.
* Dump/Json: just serialize the timeline

# Syntoniq DSL

The goal is to create an ASCII/UTF-8 file format for describing music with arbitrary tuning systems.

Goals:
* Use a score-like layout with the ability to automatically align notes and check for synchronization across parts and monophonic voices ("notes") within a part
* Represent any scale, pitch, or tuning using my pitch representation with support for chaining
* Represent notes, chords, rhythms, and dynamics with good granularity
* Support strumming and morphing
* Support multiple parts and multiple voices per part
* Generate
  * Live csound playback including with selected regions or voices
  * csound files
  * some kind of MIDI; research required; see below.
* Represent time signatures or bar numbers as optional

Possible goals:
* Use scala files

Non-goals:
* Generate printed scores
* Be opinionated about any particular system of notation
* Produce fully nuanced, performance-ready files -- if you want that, use a MIDI target and do further work in a DAW
* Drums, at least for now

# Broad Terminology

* *Part*: something akin to a score line, e.g., an instrument, one staff of a piano score, etc. At a any given time, a part may be assigned a single tuning, instrument, and dynamic, and there are other part-specific properties like strum rate. These can be inherited from global properties but may not be overridden at the note level.
* *Note*: a single rendered pitch. Chords are represented as multiple notes within a part. You can think of a note as a single, monophonic sound within a part.

A note about the word *voice*: I previously used *voice* to refer to what I now call *part*, but I abandoned this terminology because a part may be polyphonic, and it is useful to be able to use the word "voice" to refer to a single monophonic voice within a part. The word "voice" no longer refers to a semantic or syntactic element within the DSL.

# MIDI thoughts

See ~/Q/tasks/reference/midi.md

General strategy:
* Generate a separate track per part
* Bulk load MTS non-real-time at time 0 in the track for the initial scale; send real-time bulk reload if the scale changes. If morphing, use pitch bend before switching, then clear pitch bend. Note that morphing from one tuning to another can be supported by the synth, so whether or not to try to morph manually might be a configuration option.
* Assume separate routing per track
  * If we use non-overlapping channels AND all tracks using the same tuning for the whole time, then our output would work with all tracks routed to the same output; maybe make this an option? This would also make it possible to use timidity. Perhaps the option would be to generate a separate MIDI file for each group of tracks that have to be routed separately. In that case, if all tracks had the same tuning and didn't require a total of more than 15 channels (excluding 10 for percussion -- no reason to avoid channel 0), the whole thing could go to a single MIDI file which could be played with timidity.

DAW usage:
* If > 16 separate tracks, generate multiple MIDI files
* Within each MIDI file, assume each track has to be routed separately (but see above)
* Load MIDI file, route tracks, edit as needed


MIDI volume: CC#7 is channel volume, CC#11 is channel expressiveness, often interpreted as a percentage of volume, and then there's note-level velocity and channel and polyphonic (note-level) after-touch. Polyphonic after-touch is often unsupported. The list below is a summary from an AI chat of how I could represent dynamics in MIDI:

* Encode part-level dynamics as CC#7 (Channel Volume) on every MIDI channel used by that part (send before first note and on changes). This gives DAW users a clear, editable track-level automation lane.
* Use a neutral baseline note velocity for every note (configurable default). 72 (decimal) is a fine sensible default.
* Encode accents by increasing per-note velocity above the baseline. Reasonable defaults: baseline = 72, simple accent = 96, strong accent (marcato/sfz) = 108. Make these configurable.
* Optionally also emit CC#11 Expression if you want a finer control layer (but it’s optional and not universally honored).
* To make files immediately playable in sample-based synths (fluidsynth, timidity), you may want to map major written dynamics to both CC#7 (for DAW automation) and scaled velocities (for sample-layer selection), or at least ensure velocity accents are present. Many SF2 instruments rely on velocity for sample selection/attack.

I will probably want configurable defaults for
* Whether to use CC7, velocity, or both for dynamics
* How to map accents to velocity

Project: write/find something for working with MIDI files and figure out what works well as input for timidity and also Reaper. See whether other things like Ardour or Surge can work with these. Validate all of the above assumptions.

# Syntax

## Basic Syntactic Rules

Leading whitespace is stripped. Any remark about what a line starts with refers to after any leading whitespace.

The comment character is `;`. Everything from `;` to the end of the is a comment.

Comments are allowed in most places. There are a few exceptions:
* Inside multi-line directives, comments are allowed after the initial `(` and after each parameter, but comments on their own lines are errors.
* Inside multi-line scale blocks, comments are allowed after the initial `<<` and at the end of each line, but comments on their own lines are errors.

Blank lines are ignored except when they terminate score blocks.

A file consists of a sequence of the following, excluding comments and non-functional blank lines:

* Directives: declarative statements that look like function calls
* Score blocks: a sequence of one or more lines, each starting with `[part]` or `[part.note]`. Score blocks are ended by blank lines or things that are not comments, dynamic lines, or note lines.
* Scale definitions, described below

## Directives

* `name(k=v k=v ...)`
* may be divided across lines
  ```
  name(
    k=v
    k=v
  )
  ```
* a keyword may be repeated, e.g. `tempo(bpm=60 part="p1" part="p2")`
* multiple may occur on one line
* comments allowed after `(` and on the same line as a parameter but not on lines between parameters

Parameters can have one of these types:
* Pitch: the pitch notation
* Number: an integer, rational, or fixed-point decimal
* String: a double-quoted string with `\` as a quoting character only for `\"` and `\\`

## Score Blocks

A score block consists of one or more note lines and zero or more dynamic lines. A note line starts with `[part.n]`, and a dynamic line starts with `[part]`. This is defined further below.

## Scales

The `define_scale` directive is followed by scale definition blocks.
```
define_scale(name=... base_pitch=... cycle_ratio=...)
```
* `name` is mandatory
* `base_pitch` defaults to `220*^1|4` (middle C based on 440 Hz, 12-TET)
* `cycle_ratio` defaults to `2` (octave)

An scale definition block has the format
```
<<
pitch-factor name name name
pitch-factor name name name
...
>>
```

or
```
<< pitch-factor name name | pitch-factor name name >>
```

The names are enharmonic names for the scale degree. Examples:
```
define_scale(name="19-EDO")
<<
 ^0|19 c |  ^1|19 c# d%% |  ^2|19 d% c##
 ^3|19 d |  ^4|19 d# e%% |  ^5|19 e% d##
 ^6|19 e |  ^7|19 e# f%
 ^8|19 f |  ^9|19 f# g%% | ^10|19 g% f##
^11|19 g | ^12|19 g# a%% | ^13|19 a% g##
^14|19 a | ^15|19 a# b%% | ^16|19 b% a##
^17|19 b | ^18|19 b# c%
>>

define_scale(name="11-JI-partial")
<<
1     c
17/16 c#
9/8   d
6/5   e%
5/4   e
4/3   f
11/8  h11    ; 11th harmonic
45/32 f#-d   ; major third above d
17/12 f#-c#  ; fourth above c#
3/2   g
8/5   a%
5/3   a
7/4   h7     ; 7th harmonic
16/9  b%
15/8  b
>>
```

I had considered having a special "equal division" scale type and using an index instead of a pitch factor, but allowing `0` to be used creates a lot of parsing headache, and the only thing different about EDO scales is that you can note shift them. But we don't have to disallow note shift for non-EDO scales, so there's no reason to have the distinction.

## Notes

```
note ::= [$beats:]$note_name[$octave][(mods)][~|>]
note_name ::= <see below>
octave ::= `'`[n] | `,`[n]
beats ::= rational-or-decimal
```

If `beats` is omitted, take from the previous note on the same line. It is mandatory for the first note on the line. Note that these are beats as in with csound, not LilyPond-style note durations. 2:c is twice as long as 1:c, and quarter-note triplets would have 2/3 beats each.

Beats may be `a`, `a/b`, or `a.b`.

If a note ends with `~`, it is not turned off at the end of its duration. This can be used to implement ties when a pitch is held for a long time. If a note ends with `>`, it means its pitch should slide to the next pitch; see morphing below.

The note `~` by itself does nothing, making useful as a rest, continuation of a tied note, or way to position a dynamic.

Modifiers can potentially be used for accents or length modifiers. We could support `>` and `^` for accents and `.` and `_` for legato, though it's not entirely clear what these would do. Maybe I can have a parameter that sets space between notes that can be locally modified with `.` and `_`. Probably not in the first iteration.

The `|` symbol by itself may be used as an alignment check. It doesn't have to match a bar line in the traditional sense as there is no enforced time signature. (TODO: consider whether there should be a time signature that forces bar checks to align with bar lines.)

Pitches are absolute (possibly transposed). No relative octaves.

Note names must start with a letter and may contain any alphanumeric character or any of the following: `_*^/.|+-!#%&`. The broad alphabet for note names allows various symbols to be used as accidentals and also allows ratios and pitch notation to be embedded in note names. We avoid any of `()~:=<>@,';` to avoid syntactic ambiguity.

I would like to be able to morph smoothly from one pitch to another, e.g., to implement a glissando. Ideally, it should be possible to notate Fabio Costa's Etude on Minor Thirds as well as Elegy Waltz in EDO-17. I plan to use parts of these for demonstration purposes if I can get permission. See morhping and strumming below.

By convention, these ASCII symbols are used for accidentals.

* # = ♯ (diatonic sharp)
* % = ♭ (diatonic flat)
* + = ↑ (single scale step up)
* - = ↓ (single scale step down)

The default scale is 12-EDO with note names c, c#|d%, d, d#|e%, ... It is called `default`.

Examples:
* `c` -- play middle C (C4) for a single beat
* `3:e'~` -- play E5 for 3 beats and then leave the note on
* `2:~ g` -- wait 2 beats, resting or sustaining as appropriate, then play G4 for two beats.

## Dynamics

* Expressed as a numerical value from 0 to 127 (for consistency with MIDI), where `0` is silent.
* `dynamic@beat`
* `n` -- set the volume immediately to `n`
* `n<` -- start a crescendo; the next dynamic must be more than `n`. Volume is linearly interpolated.with m < n
* `n>` -- start a decrescendo; treated like a crescendo, but the next dynamic must be lower.
* Default volume is 72.
* The `<` or `>` is the last character.

Can only be expressed at the part level.

## Parts

A part maps (approximately) to a track for MIDI and an instrument for csound. In some cases, a part may have to be represented by more than one MIDI track, and it may often be possible to use the same csound instrument for more than one part.

# Version

The first directive must be `syntoniq(version=n)`. This is a file format version, not a software/semantic version. It increments whenever there is a non-backward compatible change. The contract is that you should always use the latest software that supports version `n` since operations, optional operation named parameters, or other syntactical changes that would have been errors in an older version are permitted without incrementing `n`.

# Tuning

Tuning
```
use_scale(name="..." part="...")
set_base_pitch(absolute=... relative=... part=...)
transpose(written="..." pitch_from="..." part=...)
reset_tuning(part=...)
```
* Default tuning is 12-EDO with the base pitch of `220*1|4`
* For set_base_pitch, exactly one of `absolute` or `relative` is required.
* `absolute` sets the base pitch of the scale to an absolute pitch
* `relative` sets the base pitch by adjusting relative to the current base pitch
* Transpose transposes the tuning so that the note named by `written` will have the pitch previously assigned to the note named in `pitch_from`.
  * `transpose(written="d" pitch_from=c")` would transpose down a whole step in 12-EDO.
* When no parts are specified, this sets the default tuning. If one or more parts are specified, it sets the tuning for the specified parts.
* `reset_tuning`: with no parts, resets all tunings. With parts, it resets each part's tuning to use the default tuning. There is no way to reset the default tuning without clearing the tuning table. If you need it, `tune(scale="default")` will do it.

Examples:
```
transpose(written="c" pitch_from="3")
use_scale(scale="17-EDO")
set_base_pitch(relative="*2|17")
set_base_pitch(absolute=264 part="p1")
```

# Tempo

Tempo changes are global and can be delayed relative to the current beat wherever they appear in the score.

Set the tempo to 60 beats per minute starting 2.5 beats after the current moment:
```
tempo(bpm=60 when=2.5)
```

Accelerate linearly, starting immediately from 60 to 90 beats per minute over the next 8 beats:
```
accel(start=60 end=90 for=8)
```

Decelerate linearly, starting in two beats from 90 to 60 beats per minute over 4 beats:
```
decel(start=90 end=60 when=2 for=4)
```

# Examples

The opening two bars of my rainbow medley with each part as a separate part. Note the use of the bar check and the alignment, which is optional and can be automated. There is an implicit bar check at the end of each line. If a bar check appears on any line, it must appear on all lines. For dynamics, the bar check just serves as the anchor point for beat offsets.

```
[p1] 1/2:e g e g e g e   g |   f g     f g   f g    f  g
[p2]   1:d   c   d   c     |   e   1/2:d e   d e    d  e
[p3]   2:~     1:b,  b%,   | 2:c             b,
[p4]   4:~                 | 2:~             a,
[p5]   4:~                 | 2:a,          1:g, 1/2:f, e,
```

The same thing but with a single part containing more than one note per part with a dynamic swell:

```
[p1.0]  1/2:e g e g e g e   g |      f g     f g   f g    f  g
[p1.1]    1:d   c   d   c     |      e   1/2:d e   d e    d  e
[p1.2]    2:~     1:b,  b%,   |    2:c             b,
[p1.3]    4:~                 |    2:~             a,
[p1.4]    4:~                 |    2:a,          1:g, 1/2:f, e,
  [p1]   64@0    64@2<        |   96@0>         64@2
```

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

# Morphing

One way to support morphing might be to allow a note to start or end with `>`, e.g.
```
[p1] 1:c> >e
```
If this is done, we probably need a morph directive, like `morph(before=1/8 after=0 when=2)`, indicating to start the morph 1/8 of a beat before the note change and end exactly at the new note, effective 2 beats after the current moment. This makes it similar to accel/decel.

# Strumming

To indicate strumming, we can use something like `strum(gap=beats on_beat=n)` where `n` is a note number, and notes are strummed in order. For example:
```
  [p1] strum(gap=1/12 on_beat=0)
[p1.0] 1:c
[p1.1] 1:e
[p1.2] 1:g
```
would strum a C major chord with the `c` on the beat and the `e` and `g` following 1/12 and 2/12 of a beat later.
```
  [p1] strum(gap=1/12 on_beat=2)
[p1.0] 1:c
[p1.1] 1:e
[p1.2] 1:g
```
would be similar but the strumming would start before the beat with the `g` on the beat.

Consider how to handle strumming that precedes the first beat. Maybe make it an error and require a rest? That way, we can generate better metadata about marks or file positions and their time offsets in the output.

# Metadata

Key signature and time signature are not needed for this application, but time signature can be helpful for MIDI piano rolls or notation. Key signature can be useful for notation as well, but it's not really useful with microtonal music. It would be useful to provide mechanisms to output this information into the MIDI file even if it is not otherwise use. We can also include ways to include creator and title. Looking at a `midicsv` file as generated by Lilypond or other applications can provide a useful template.

# Instruments

If we support midi and csound, we need different ways to represent instruments. It should be general so additional backends could be added.

```
define_instrument(name=... midi=... csound=...)
use_instrument(name=... part=...) ; default or within a part
```
* name: generic
* midi: program/instrument number
* csound: instrument number or name

# Marks, Regions, and Repeats

```
mark(name="x")
repeat(from="mark1" to="mark2")
skip_repeats()
```

It would be useful to be able to play from a mark to another mark, probably just as command-line arguments (`--start-at-mark`, `--end-at-mark`). We can maintain full state so notes that are still on are playing, etc.

It would be useful to be able to generate data about time and/or beat offsets for lines and marks.

# Software Design

Follow comments in the code, starting from parsing.rs.

Directives are defined using a derive macro that generates help from doc comments.

For csound, we should design some kind of instrument template that allows dynamic volume change, but changes to instruments can just be a part to instrument mapping change in the software. For MIDI, we will have to maintain some kind of track/channel mappings, but it shouldn't be very hard. Probably the rule should be to put as many notes as possible in one channel, moving out to additional channels if we have differences in channel-specific settings like volume or pitch bend.

Everything should have simple defaults. The following should be a valid file:
```
syntoniq(version=1)
[p1.0] 1:c
```

Defaults:
* tempo: 60 bpm
* instrument: csound instrument 1, MIDI program 0
* tuning: 12-EDO (from C), base pitch = `220*1|4`

Write a tree-sitter grammar and an LSP server. See this chat: https://gemini.google.com/app/665ad5eb23ae0417

Summary:
* LSP Server crates:
  * lsp-server: This is the core engine. It handles the low-level JSON-RPC communication protocol over stdin/stdout and provides a simple event loop for receiving messages from the editor.
  * lsp-types: This crate contains all the Rust structs and enums that correspond directly to the Language Server Protocol specification (e.g., Diagnostic, CompletionItem, Position). It saves you from defining these data structures yourself.
* Tree-sitter Grammar Reference
  * Tree-sitter uses a static grammar definition; it is a library that editors use directly, not a server with a protocol.
  * The core of your work will be creating a grammar.js file. This single file defines your language's syntax using a JavaScript DSL. The Tree-sitter CLI tool then uses this file to generate a C parser (parser.c), which gets compiled into a shared library that editors can load.
  * The best reference for getting started is the official Tree-sitter documentation: [Creating Parsers](https://tree-sitter.github.io/tree-sitter/creating-parsers)

## Reformatting

Suggested reformatting rules:
* Collapse multiple blank lines to single blank lines, and remove leading and trailing blank lines
* Remove trailing white space
* In a multi-line structure (score block, scale definition block, multiline directive), keep comments aligned and offset by two spaces from the longest line
* Remove spaces from around `=` in directive parameters
* If a directive with any trailing comment exceeds 100 columns, move the trailing comment to the preceding line. If still over 100 columns, break the directive to one parameter per line.
* If a directive that contains no parameter-level comments fits on one line in <= 100 columns, reformat as a single line. Never move a preceding comment to after a single-line directive.
* Apply alignment to score blocks as above
* Within scale definition blocks, right-justify pitches or indices with columns, then align and left-justify note names

## Considerations

A scale like 31-EDO will overflow 128 notes. It doesn't matter for csound. For MIDI, can we should automatically split the tracks.
