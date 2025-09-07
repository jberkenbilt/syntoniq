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

A line may contain a global operation, e.g.

```
base_pitch=220*^1|3
tempo=60
```

If a line starts with `[voice]`, it pertains to that voice. A group of contiguous voice lines are treated as simultaneous. Use a blank line to move forward. This is similar to how a score is. If the same voice line appears more than once a score line, the effect is to concatenate.

Note format:
```
$note = [$beats:](`<`($single_note( $single_note)*)`>` | $single_note)
single-note ::= note-name octave
octave ::= `'`[n] | `,`[n]
beats ::= rational
```

Pitches are absolute (possibly transposed). No relative octaves.

If `beats` is omitted, take from the previous note of the same voice. If omitted for first note, it is `1`. Note that these are beats as in with csound, not LilyPond-style note durations. 2:c is twice as long as 1:c, and quarter-note triplets would have 2/3 beats each.

In addition to constructing chords with `<notes...>`, I want to support "strumming" and also the ability to morph smoothly from one note to another. Ideally, it should be possible to notate Fabio Costa's Etude on Minor Thirds as well as Elegy Waltz in EDO-17. I plan to use parts of these for demonstration purposes if I can get permission.

By convention, these ASCII symbols are used for accidentals.

* # = ♯ (diatonic sharp)
* % = ♭ (diatonic flat)
* + = ↑ (single scale step up)
* - = ↓ (single scale step down)

The default scale is EDO-12 with note names c, c#|d%, d, d#|e%, ...

I need some mechanism for defining custom scales (similar to the launchpad controller) but with support for enharmonics.

Examples:
* `c` -- play middle C for a single beat
* `3:<c, c g c' e' g' c'2>` -- play a chord for three beats from the C below middle C to the C two octaves above middle C

Note names should avoid any of `!@:=,'<>;` but can contain numbers other special characters, including `^`, `*`, `/`, `|`, and `.`, making it possible to use pitches as note names.

Use `!` for a rest.

The comment character is `;`

Voice names can be arbitrary except not contain `[]`.

Voice commands:
* `reset_base` -- sets the base pitch of the voice to the global base pitch
* `scale=...` -- sets the scale; base stays the same
* `base=note@` -- transposes to set the base of the voice to the pitch of another note in the same scale
* `base=note@scale` -- transposes to set the base of the voice to the pitch of another note in a different scale
* `base=pitch` -- transposes to set the base of the voice to a specific pitch
* These are approximate. Other commands can be used for strumming, volume, instrument selection, etc.

Examples:

Using an EDO-19 scale transposed so written `c` is `e`, play some notes:
```
base_pitch=220*^1|3 ; set base pitch to middle C relative to A 440
[v1] scale=edo-12   ; set scale to edo-12
[v1] base=e@        ; transpose so E becomes the new tonic, e.g., written C sounds like E
[v1] scale=edo-19   ; reset the scale to edo-19, retaining the base pitch
[v1] c e 2:<g b%> <c, c'>  ; play some notes
```

You can create a local scale. Leading whitespace continues the line.

```
new_scale=e19 scale=edo-12 base=e scale=edo-19
```

Then the previous example could be written as
```
[v1] scale=e19 c e 2:<g b%> <c, c'>
```

Setting a scale without a voice sets it globally. `reset_scale` for a voice resets the scale for that voice to the global scale.

Example score:

```
[v1] 1/2:e g e g e g e  g
[v2]   1:d   c   b   b%

[v1] 1/2:f g f g   f g     f g
[v2]   2:a       1:g   1/2:f e
```

In lieu of bar checks, at the beginning of each score line, the timing has to line up.

Alignment is visual only. You can continue a voice line within one score line. These are the same:

```
[v1] 1/8:c d e f g a b c' c' b a g f e d c
[v2]   1:f                e
```
```
[v1] 1/8:c d e f g a b c'
[v1] c' b a g f e d c
[v2] 1:f e
```

It would be nice to have tool support for alignment. Within a score line, align notes so the beginning of the pitch part of notes are aligned rhythmically after any beat markers as in the above examples. See below for an algorithm.

```
[v1] 1:<c e g> <c f a>   <c e g> <b, d g> 4:<c e g>
[v2] 2:c,              1:f,      g,       4:c,
```

The DSL interpreter should have some commands to check and align. I could run C-c C-f on a score line, and it could either reformat or generate output with embedded comments containing any error messages. No reason to integrate with flycheck, etc.

Other notes:
* Allow non-breaking space for additional visual alignment
* To align, calculate total number of discrete events (GCD)
* For each note, get number of characters before and after alignment point; : counts as before
* prepend/append space so all notes are the same width and have the alignment point in the same spot
* prepend each note with one extra space
* place notes based on numerator of n/GCD
* join all notes with spaces; keep any non-breaking spaces between notes; place immediately before following note
* shrink vertical columns of spaces to width of 1
* align `]` of voice names

Example:

```
[treble] 1:<c e g> <c f a> 2/3:g f d
[bass] 2:c, 1:f, g,
```
* max before alignment = 4 (`2/3:`)
* max after alignment = 7 (`<c e g>`)
* total width = 12
* gcd: 3, so notes go at (zero-numbered beat position * 3 * 12)
* each note width is 12 (1+4+7)
```
|0          |1          |2          |3          |4          |5          |6          |7          |8          |9          |10         |11
 _1:<c e g>                          ____<c f a>                         2/3:g______             ____f______             ____d______
__2:c,_____                                                              __1:f,_____                         ____g,_____
```
```
 1:<c e g> <c f a> 2/3:g  f    d
 2:c,                1:f,   g,
```
```
[treble] 1:<c e g> <c f a> 2/3:g  f    d
  [bass] 2:c,                1:f,   g,
```

It's not clear what to do when a voice spans across multiple lines.

It's possible that we may want bar checks within a single line for compact enough lines.

For strumming morphing, consider something like `<(strum) ...>` or `<(morph> ...>`, or maybe we can have some compact way that can be specified, like `<1: ... >`. When morphing, you can only morph from one chord to another if they have the same number of notes. Notes will be aligned in order of appearance. You can use `!` or repeated notes. When morphing to or from `!`, treat `!` as silent. Possible example:
```
[v1] <c d f a> <2: c e e g> <2: d ! ! f>
```
would start with `<c d f a>`, then `c` stay the same, `d` and `f` would both slide to `e`, and `a` would slide to `g`. Then `c` would slide to `d` and `g` to `f` while `e` faded out. Duplicating the note should not cause its amplitude to increase.

Unresolved:
* How do you specify the timing for strum and morph, including how long to spend morphing, when to start, and when to end?
* I have not done anything about dynamics. There probably needs to be a command for it.
* There may need to be instrument-specific escape hatches for communicating with the orchestra parts of the file or allowing hand-coded sections.

