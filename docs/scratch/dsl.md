# Syntoniq DSL

The goal is to create an ASCII/UTF-8 file format for describing music with arbitrary tuning systems.

Goals:
* Use a score-like layout with the ability to automatically align notes and check for voice synchronization
* Represent any scale, pitch, or tuning using my pitch representation with support for chaining
* Represent notes, chords, rhythms, and dynamics with good granularity
* Support strumming and morphing
* Support multiple voices
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

# MIDI thoughts

See ~/Q/tasks/reference/midi.md

General strategy:
* Generate a separate track per voice
* Bulk load MTS non-real-time at time 0 in the track for the initial scale; send real-time bulk reload if the scale changes. If morphing, use pitch bend before switching, then clear pitch bend. Note that morphing from one tuning to another can be supported by the synth, so whether or not to try to morph manually might be a configuration option.
* Assume separate routing per track
  * If we use non-overlapping channels AND all tracks using the same tuning for the whole time, then our output would work with all tracks routed to the same output; maybe make this an option? This would also make it possible to use timidity. Perhaps the option would be to generate a separate MIDI file for each group of tracks that have to be routed separately. In that case, if all tracks had the same tuning and didn't require a total of more than 15 channels (excluding 10 for percussion -- no reason to avoid channel 0), the whole thing could go to a single MIDI file which could be played with timidity.

DAW usage:
* If > 16 separate tracks, generate multiple MIDI files
* Within each MIDI file, assume each track has to be routed separately (but see above)
* Load MIDI file, route tracks, edit as needed

Project: write/find something for working with MIDI files and figure out what works well as input for timidity and also Reaper. See whether other things like Ardour or Surge can work with these. Validate all of the above assumptions.

# Syntax

Work in progress; all syntax is subject to change.

## Basic Syntactic Rules

Leading whitespace is stripped. Any remark about what a line starts with refers to after any leading whitespace.

The comment character is `;`. Everything from `;` to the end of the is removed.

Blank lines are ignored except when they terminate score blocks.

A file consists of a sequence of the following, excluding comments and non-functional blank lines:

* Global directives: lines consisting of operations, which look like function calls, and have global scope
* Score blocks: a sequence of one or more lines, each starting with [voice.note], both preceded by and followed by a blank line
* A macro definition
* Possibly an escape hatch if needed, but hopefully not

## Operations

* `name(k=v, k=v, ...)`
* must be contained within a single line
* multiple may occur on one line
* If not in a score block, scope is global; otherwise, scope is for the voice

## Score Blocks

Each line starts with `[voice]` or `[voice.n]`, where `n` is a note number. `n` may be omitted if there is only a single note, in which case the note number is `0`. If `n` is omitted, whatever is present refers to all notes on the line. Some operations, such as tuning, are only allowed to apply to the entire voice.

A score block must be both preceded and followed by a blank line, the beginning of the file, or the end of the file.

See examples below.

## Notes

```
note ::= [$beats:]$note_name[$octave][~]
note_name ::= <see below>
octave ::= `'`[n] | `,`[n]
beats ::= rational-or-decimal
```

If `beats` is omitted, take from the previous note on the same line. It is mandatory for the first note on the line. Note that these are beats as in with csound, not LilyPond-style note durations. 2:c is twice as long as 1:c, and quarter-note triplets would have 2/3 beats each.

Beats may be `a`, `a/b`, or `a.b`.

If a note ends with `~`, it is not turned off at the end of its duration. This can be used to implement ties when a pitch is held for a long time.

The note `~` by itself does nothing, making useful as a rest, continuation of a tied note, or way to position a dynamic.

The `|` symbol by itself may be used as an alignment check. It doesn't have to match a bar line in the traditional sense as there is no enforced time signature. (TODO: consider whether there should be a time signature that forces bar checks to align with bar lines.)

Pitches are absolute (possibly transposed). No relative octaves.

TODO: work out valid characters in note names. Note names should avoid any of `~:=<>@,';` but can contain numbers other special characters, including `^`, `*`, `/`, `|`, and `.`, making it possible to use pitches as note names.

I would like to be able to morph smoothly from one pitch to another, e.g., to implement a glissando. Ideally, it should be possible to notate Fabio Costa's Etude on Minor Thirds as well as Elegy Waltz in EDO-17. I plan to use parts of these for demonstration purposes if I can get permission.

By convention, these ASCII symbols are used for accidentals.

* # = ♯ (diatonic sharp)
* % = ♭ (diatonic flat)
* + = ↑ (single scale step up)
* - = ↓ (single scale step down)

The default scale is 12-EDO with note names c, c#|d%, d, d#|e%, ...

I need some mechanism for defining custom scales (similar to the launchpad controller) but with support for enharmonics.

Examples:
* `c` -- play middle C (C4) for a single beat
* `3:e'~` -- play E5 for 3 beats and then leave the note on
* `2:~ g` -- wait 2 beats, resting or sustaining as appropriate, then play G4 for two beats.

## Dynamics

* Expressed as a numerical value from 0 to 127 (for consistency with MIDI)
* `dynamic@beat`
* `=n` -- set the volume immediately to `n`
* `m<` -- start a crescendo; the next dynamic must be `<n`. Volume is linearly interpolated by m and n, with m < n
* `m>` -- start a decrescendo; the next dynamic must be `>n`. Volume is linearly interpolated by m and n, with m > n

## Macros

Tentative. Not sure if this is a good idea.

Single-line macro. `n` is the number of parameters and `,` is the separator. Within the macro, `$n` is replaced by the argument.
```
$name(n,) { .... }
```

Example:
```
; define
$transpose(1,) { tune(base_note=$1) }
; invoke
$transpose(e)
```

Multi-line macros are the same but are defined as
```
$name(n,) {
...
}
```
and the invocation of a multi-line macro must be on a line by itself.

Macros may call other macros and are lexically expanded from top down. They can only reference previously defined macros. Macros may not define macros.

## Voices

A voice maps to a track for midi and an instrument for csound.

# Tuning

Tuning
```
tune(base_pitch=..., base_note=..., scale=...)
reset_tuning()
```
* Default tuning is 12-EDO with the base pitch of `220*1|4`
* At most one of `base_pitch` or `base_note` is allowed
* `base_pitch` sets the base pitch of the scale to an absolute pitch
* `base_note` sets the base pitch to a specified note in the *current* scale
* `scale` sets the new scale
* In global scope, this sets the default tuning. In a voice scope, it sets the tuning for the voice.
* `reset_tuning`: in global scope, resets the default to 12-EDO with `220*1|4`. In voice scope, it resets the voice's tuning to use the default tuning

# Tempo

Tempo changes are global and can be delayed relative to the current beat wherever they appear in the score.

Set the tempo to 60 beats per minute starting 2.5 beats after the current moment:
```
tempo(bpm=60, when=2.5)
```

Accelerate linearly, starting immediately from 60 to 90 beats per minute over the next 8 beats:
```
accel(start=60, end=90, for=8)
```

Decelerate linearly, starting in two beats from 90 to 60 beats per minute over 4 beats:
```
decel(start=90, end=60, when=2, for=4)
```

# Examples

The opening two bars of my rainbow medley with each part as a separate voice. Note the use of the bar check and the alignment, which is optional and can be automated. There is an implicit bar check at the end of each line. If a bar check appears on any line, it must appear on all lines. For dynamics, the bar check just serves as the anchor point for beat offsets.

```
[v1] 1/2:e g e g e g e   g |   f g     f g   f g    f  g
[v2]   1:d   c   d   c     |   e   1/2:d e   d e    d  e
[v3]   2:~     1:b,  b%,   | 2:c             b,
[v4]   4:~                 | 2:~             a,
[v5]   4:~                 | 2:a,          1:g, 1/2:f, e,
```

The same thing but with a single voice containing more than one note per voice, a dynamic swell affecting all but note 0, and a fixed dynamic for note 0:

```
[v1.0]  1/2:e g e g e g e   g |      f g     f g   f g    f  g
[v1.1]    1:d   c   d   c     |      e   1/2:d e   d e    d  e
[v1.2]    2:~     1:b,  b%,   |    2:c             b,
[v1.3]    4:~                 |    2:~             a,
[v1.4]    4:~                 |    2:a,          1:g, 1/2:f, e,
[v1.0] =112@0                 |
  [v1]  =64@0   64>@2         | >96>@0         >64@2
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
* align `]` of voice names, prepending leading space

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

Step 3: prepend `]`-aligned voice names:
```
[treble] 1:e  a 2/3:g f   d
  [bass] 2:c,     1:f  g,
```

# Morphing

One way to support morphing might be to allow a note to start or end with `>`, e.g.
```
[v1] 1:c> >e
```
If this is done, we probably need a morph directive, like `morph(before=1/8, after=0, when=2)`, indicating to start the morph 1/8 of a beat before the note change and end exactly at the new note, effective 2 beats after the current moment. This makes it similar to accel/decel.

# Strumming

To indicate strumming, we can use something like `strum(gap=beats, on_beat=n)` where `n` is a note number, and notes are strummed in order. For example:
```
  [v1] strum(gap=1/12, on_beat=0)
[v1.0] 1:c
[v1.1] 1:e
[v1.2] 1:g
```
would strum a C major chord with the `c` on the beat and the `e` and `g` following 1/12 and 2/12 of a beat later.
```
  [v1] strum(gap=1/12, on_beat=2)
[v1.0] 1:c
[v1.1] 1:e
[v1.2] 1:g
```
would be similar but the strumming would start before the beat with the `g` on the beat.
