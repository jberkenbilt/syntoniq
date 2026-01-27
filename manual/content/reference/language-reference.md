+++
title = "Language Reference"
weight = 20
sort_by = "weight"
+++

This section provides precise descriptions of all the features of the Syntoniq Language.

# Compatibility Contract

This section describes Syntoniq's forward and backward compatibility contract.

**Pre-1.0 Note:** The compatibility contract is not enforced until version 1.0.0.

The first directive in a Syntoniq file must be
```syntoniq
syntoniq(version=1)
```

Here are the specific guarantees. Each is followed by a few examples, but not an exhaustive list.

Within a specific version, we provide the following guarantees
* No previously valid syntax will become invalid unless the old behavior was a bug.
  * We won't remove characters from valid note names.
  * We might create an error message for code that used to create incorrect musical output because of a bug.
* No previously valid score will be valid but mean something semantically different.
  * We will not re-interpret dynamics or pitches.
  * We will not change the meanings of notes in generated scales.
  * We won't change the meanings or default values of directive parameters.

Within a version, we may do any of the following:
* Add new features so that a valid file won't work with an *older release* of Syntoniq.
  * We might add a new directive.
  * We might add new optional parameters to directives.
* Make improvements to the MIDI or Csound generation.
  * We might change how notes whose frequencies are out of range are rendered.
  * We might make improvements to channel allocation or use of specific MIDI instructions to improve the experience of DAW users based on feedback.
* Change the implementation of the Csound instrument, though we would try to make it backward compatible.
  * We might pass more granular information about the timeline to the instrument.
  * We might improve how the instrument handles polyphony.

If we change anything about the MIDI or Csound generation, release notes will describe the changes in detail, including what you need to do if you are migrating. If a change to MIDI or Csound output is too invasive, we may allow the old behavior to be selected.

Rationale for allowing some MIDI/Csound changes: while Syntoniq intentionally creates Csound and MIDI that are designed for additional manual enhancement, it is assumed that, people will not adopt a workflow where they are actively composing in Syntoniq and iterating by regenerating output and replacing it in a DAW or Csound file, and that people who *are* doing this would be able to handle the changes. We optimize for the case of having the unmodified output of Syntoniq being as good as possible.

# Syntax

## Spaces and Comments

A comment is started by `;` and proceeds until the end of the line.

White space and newlines are ignored and are allowed in all contexts with the following exceptions:
* Comments are terminated by newlines
* Blank lines terminate score blocks

## Strings

A Syntoniq string is delimited by double quotes (`"`) and may contain any valid UTF-8 text. To include `\` or `"` in a string, precede with `'`. Strings may not contain embedded newlines.

Examples (preceded by directive parameters to be syntactically valid):
```syntoniq
f(
   s1="piano"
   s2="a \" and a \\"
   s3="π♯"
)
```

## Numbers

A number may be
* Any integer value
* A rational number whose numerator is a positive decimal of up to three decimal places and whose denominator is a positive integer

Examples (preceded by directive parameters to be syntactically valid):
```syntoniq
f(
   n1=0
   n2=261.626
   n3=1.5/12
   n4=3/2
   n5=31
)
```

## Pitches

A pitch is one or more *factors* separated and optionally preceded by `*`.

In the description below, `[x]` indicates that `x` is optional. Letters represent numerical values. All other characters, including `^`, `/`, and `|` are literal.

A factor represents either a rational number or a rational number raised to a rational power. It must be non-empty and adhere to the following pattern:
```
[a[/b]][^c|d]
```
where
* `a` is a positive integer or decimal of up to three decimal places
* `b` is a positive integer
* `c` is any integer, including negative numbers or zero
* `d` is a positive integer

Since a pitch must be non-empty, at least one of `a/[b]` or `^c|d` must be present.

If `a[/b]` is omitted, it takes the value of `2`.

If `a` is present and `[/b]` is omitted, `b` has the value of `1`.

See [Pitch Primer](../../microtonality/pitch-primer/) for a detailed semantic description of pitches.

Examples (preceded by directive parameters to be syntactically valid):
```syntoniq
f(
    p1=2
    p2=*3/2
    p4=^7|17
    p5=220*3^2|13
    p6=3/2^-6|15
    p7=261.3/2*2^1|2*3^1|3
)

## Directives

Directives take the form
```syntoniq
identifier(param="value" repeatable="value1" repeatable="value2")
```

Directive syntax details:
* There is no separator between parameters. Think of them as being like XML attributes rather than function arguments.
* All parameters are named. Parameters can be repeated, as in `repeatable` above. Repeating a parameter makes its value a list.
* Parameter values may be numbers, pitches, or strings.
* Newlines may appear anywhere in a directive definition except between a parameter name and value.
* Spaces may optionally surround `=`
* Comments may appear inside a directive.

Example (syntactically valid but not valid Syntoniq):
```syntoniq
identifier ( ; opening comment
   one   = 1
   two   = 22/7
   three = "🥔♭"
   three = "π♯"  ; repeated parameter
   four  = *3^-2|31*3/2
)
```

See [Directive Reference](#directive-reference) below for the list of valid directives and their parameters. You can also run `syntoniq doc`.

## Note Names

Note names must start with an ASCII alphabetic character and may contain the following characters:
* alphanumeric characters
* any of `_*^/.|+-!\#%&`

## Scale Definitions

The directive `define_scale` must be followed by a scale definition. A scale definition is delimited by `<<` and `>>` and consists of sequence of pitches followed by note names.

You may define more than one note on a line. You may assign multiple names to a pitch. It is an error to have a duplicated pitch. Instead, add all the notes to the single definition of the pitch. This is based on the *value* of the pitch, not the representation. If you tried to add both `^2|12` and `^1|6`, you would get an error message.

Scale definitions are described in detail with examples in [Defining Scales](../../microtonality/scales/).

## Layout Definitions

The directive `define_manual_mapping` must be followed by a layout definition. A layout definition is delimited by `<<` and `>>` and consists of a rectangular grid of note names optionally followed by cycle (e.g. octave) markers. A note name may be replaced by `~` to indicate an unmapped key. Exactly one item (either a note or `~`) must be preceded by `@` to indicate that it is the anchor note.

Layout definitions are described in detail with examples in [Layout Engine](../../keyboard/layout-engine/).

## Score Blocks

Score blocks consist of groups of contiguous *note lines* and *dynamic lines*. A score block is terminated by a blank line or a line containing a directive.

You can find examples of score blocks throughout the manual including in the [Complete Example](../../microtonality/example/) section. Here's a simple example, repeated from the [Quick Start](../../introduction/quickstart-12-edo/) section.

<!-- generate include=hello.stq checksum=f7c2a15b5a54491b3b9f9e1c471b01ed442e3f2f5d1f75691ebc9e8c9bd4631e -->
```syntoniq
syntoniq(version=1)

; Here is some music
[p1.0]  1:g a    b  c'
[p1.1]  1:e f    g  g
[p1.2]  2:c    1:f  e
[p1.3]  2:~    1:d  d
[p1.4]  1:~ a,   g, c,
  [p1] 64@0<    127@4
```
<!-- generate-end -->

{{ audio(src="hello-csound.mp3", caption="Audio Created with Csound") }}

### Parts

A Syntoniq *part* corresponds approximately to a part in a score. It is a container for notes and dynamics. A part may contain any number of simultaneous notes (subject to limitations of the instrument). In Syntoniq, certain properties apply at the part level, such as the following:
* dynamics
* tuning
* Csound or MIDI instrument assignment

In the [Directive Reference](#directive-reference) section, some directives take a `part` parameter to set part-specific parameters.

A *part name* must start with an alphabetic character and may contain only alphanumeric characters or underscore (`_`).

### Note Lines

Note lines begin with `[part_name.n]`, where, in this case *the `[` and `]` characters are literal* (not indicating an optional value) and `n` is a non-negative integer value (0 or positive) indicating a note number. Note that the note number is interpreted as a numerical value, ignoring leading zeroes. This is probably the least surprising behavior unless you are used to Csound. If you are accustomed to Csound, keep in mind that, since Syntoniq treats note numbers as *numeric values*, `part.1` and `part.01` refer to the *same note*. This is different from Csound, which treats these like floating point fractional parts. In Csound, `part.1` and `part.10` would be the same. We believe that, for anyone except a Csound user, it is less surprising to view the note line leader as a `.`-separated part name and numerical value.

After the line leader (`[part_name.n]`), a line consists of any of a sequence of
* notes
* holds
* bar checks

#### Notes

In this description of a note, the `[` and `]` characters represent optional values. The general syntax of a note is `[duration:]name[cycle-markers][:modifiers]`. Note pitches are absolute in Syntoniq. If you are coming from LilyPond, you might be accustomed to LilyPond's relative pitch mode. Syntoniq intentionally does not support relative pitch mode as this creates a lot of confusion when rearranging notes in a score. It is also impractical to define it in a meaningful way with arbitrary note names and pitches. We believe absolute pitch notation is the only sensible approach with Syntoniq, but it can be a source of momentary confusion if you are accustomed to using relative pitch notation in LilyPond.

Duration is mandatory for the first note in each note line. If omitted in subsequent notes, the value is repeated from the previous note. Default duration values do not carry across lines. This reduces surprises when splitting, joining, or otherwise rearranging lines in a score.

Duration is a rational number or decimal with up to three decimal places. It is measured in beats. This is similar to Csound. If you are used to LilyPond, notice the difference. In Syntoniq, `4:a` means to play `a` for four beats. In LilyPond, `a4` indicates the note `a` as a quarter note. In this case, Syntoniq's use of beat counts aligns it more with Csound. Note that, since Syntoniq durations are rational numbers, you can represent tuplets with perfect precision. For example: `1/3:c d e` would be similar to eighth note triplets.

`name` is the note name, whose syntax is discussed above.

`cycle-markers` may be one of `'` (one cycle up), `,` (one cycle down), `'n` ($n$ cycles up) or `,n` ($n$ cycles down). A cycle is usually an octave, but it may be defined to be any other interval using any of the scale definition directives. (See [Directive Reference](#directive-reference) and [Defining Scales](../../microtonality/scales/).)

* Modifiers are characters that modify some aspect of a note's behavior. The default behavior of a note is that it sounds for the full duration. The following modifiers are available:
  * `>` — slightly increases the velocity (MIDI) or amplitude (Csound) of the note; corresponds to an accent.
  * `^` — like `>` but more; corresponds to marcato.
  * `.` - may be repeated; shortens the note by one quarter of a beat as long as duration remains at least one quarter of a beat. This roughly corresponds to staccato. It is a shortcut and behaves the same regardless of the note length. For more precise control, you can use full-length notes with specific durations, such as 7/8.
  * `~` — tie: sustains the note, holding the pitch constant across any subsequent holds (discussed below). If the subsequent note has the same pitch, this implements a tie. For Csound, if the next pitch is different, this acts like a slur, changing the pitch of the note without releasing and retriggering the note.
  * `&` — glide: sustains the note indicating the pitch should glide smoothly to the pitch of the next note. Like with `~`, intervening holds extend its duration. The following note is re-articulated by default, but you can combine tie and glide to create chains of continuous pitch glides. For Csound, this implements smooth pitch changes. With MIDI, it causes several pitch-bend changes per second.

#### Holds

You can indicate *hold* with `~`. The `~` character can be preceded by a duration and must be preceded by a duration if it is the first item in the line. A *hold* means "keep doing what you're doing." That means that, following a tied note, a *hold* means to keep holding the pitch. Following a glide it extends the duration over which the pitch is changed. Following a non-sustained note, or as the first thing, it is a rest.

#### Bar Checks

The `|` character may occur in any position in a note line except the beginning or end. When a bar check appears, Syntoniq performs the following validations:
* Each line in the score block must contain the same number of bar checks.
* The duration between a bar check and its neighbors (another bar check, the beginning of the line, of the end of the line) must be consistent across lines.

If you are coming from LilyPond, this is similar to LilyPond bar checks, but since Syntoniq (intentionally) doesn't have the concept of time signatures, they are just alignment checks and visual separators.

#### Other Things to Know

Syntoniq ensures that every note line in a score block is the same length. If you make a mistake, the compiler will give you enough information to fix the mistake by telling you the computed duration of each line so you can easily find the error. Bar checks can help.

When generating MIDI output with MPE, every note with a given note number is always assigned to the same channel. This makes it easier to perform edits. When generating Csound output, there is a fixed mapping between Syntoniq note numbers and Csound note numbers that takes into consideration the differences between how each system uses note numbers.

Note numbers do not have to be sequential or contiguous.

Note numbers do not have to be in any particular order, but you may have only a single line per score block with a given part and note number.

If a score block doesn't sound any notes for a note number, you don't need a line for it. If the last occurrence of the note was sustained, the sustain eventually has to be resolved, but you can skip one or more score blocks.

The Syntoniq compiler is very thorough and gives clear errors with copious context. If you get it wrong, the compiler will help you fix it.

### Dynamic Lines

Dynamic lines start with `[part_name]` where, as with the note leader, `[` and `]` *appear literally*. Dynamic lines consist of a sequence of dynamics and bar checks.

A dynamic has the form (where `[` and `]` mean "optional") `level@offset[change]`.
* `level` is a value from 0 to 127 inclusive
* `offset` is a number of beats as an offset from the beginning of the line or the most recent bar check
* `change`, if present, may be `<` to indicate a crescendo. or `>` to indicate a diminuendo.

When bar checks appear in note lines, Syntoniq validates that there are the same number as in the note blocks and that all offsets fall within the duration of the corresponding region of notes. Here again, if you get it wrong, the compiler will give you copious information to help you fix it.

If you use `<` or `>`, Syntoniq will enforce that there is a subsequent dynamic and that it is greater than (for crescendo) or less than (for diminuendo) the previous volume. This serves as an extra check. Volumes of `0` are allowed.

# Directive Reference

Below is an alphabetical list of directives. You can get this by running `syntoniq doc`.

<!-- generate include=directive_doc.md checksum=6f430836449aed3a6ae50bfe83b3a35b7b88b204de0240849d76bda5da2b42ab -->

## csound_instrument

Set the Csound instrument number or name for zero or more parts. If no part
is specified, this becomes the default instrument for all parts without a
specific instrument. It is an error to name a part that doesn't appear
somewhere in the score. You must specify exactly one of number or name.

**Parameters**:
* **number (optional)** — Csound instrument number
* **name (optional)** — Csound instrument name
* **part (repeatable)** — Which parts use this instrument; if not specified, all unassigned parts
use it

## define_generated_scale

Define a generated scale. Note pitches are generated according to the
following rules:
- Notes consist of letters, numbers, `+`, `-`, `#`, `%`, `!`, and `/`.
- `A` and `a` represent the root of the scale
- `B` through `Y` represent n/n-1 where n is the ordinal position of the
  letter (B=2, C=3/2, D=4/3, etc.)
- `b` through `y` are n-1/n, the reciprocal of their upper-case
  counterparts (b=1/2, c=2/3, d=3/4, etc.)
- `Z` followed by a number ≥ 2 represents n/n-1 (e.g. Z30 = 30/29)
- `z` followed by a number ≥ 2 represents n-n/n (e.g. z30 = 29/30)
- All factors are multiplied to create the base pitch; e.g, (Bh = 2×7/8 =
  7/4, Cl = 3/2×11/12 = 11/8)

When `divisions` is specified, the following additional rules apply, noting that the divided
interval can be explicitly given and defaults to the cycle ratio, which defaults to 2.
- `An` represents `n` scale steps up (divided_interval^n|divisions)
- `an` represents `n` scale steps down (divided_interval^-n|divisions)
- `+` is short for `A1` (raises the pitch by one scale degree)
- `-` is short for `a1` (lowers the pitch by one scale degree)
- If `tolerance` is not specified or the pitch is within tolerance of its
  nearest scale degree, the pitch is rounded to the nearest scale degree,
  and the `#` and `%` characters have no effect on the pitch.
- If `tolerance` is specified and the pitch is farther away from its nearest
  scale degree than `tolerance`:
  - `#` forces the pitch to the next highest scale degree
  - `%` forces the pitch to the next lowest scale degree

The specified divisions or divided_interval can be overridden by appending `!` followed
by optional numbers separated by `/`. This causes the following additional changes:
- `!` — forces the exact ratio to be used, allowing pure ratios to be mixed with equal
  divisions
- `!n` — interprets the note as if `divisions=n` where specified
- `!a/n` — interprets the notes as if `divided_interval=a divisions=n` where specified
- `!a/b/n` — interprets the notes as if `divided_interval=a/b divisions=n` where specified

Example: with `divisions=17` and tolerance of 4¢:
- `E` is `^5|17` because 5/4 is between steps 5 and 6 (zero-based) but is
  slightly closer to step 5
- `E%` is also `^5|17`
- `E#` is `^6|17`
- `E!` is `5/4`
- `E!12` is `^4|12`

See the manual for more details and examples.

**Parameters**:
* **scale** — scale name
* **cycle_ratio (optional)** — ratio to be applied by the octave marker; default is 2 (one octave)
* **divisions (optional)** — number steps to divide the divided interval into; omit for a pure just intonation scale
* **divided_interval (optional)** — interval to divide when `divisions` is given or specified as a single digit in an override;
defaults to cycle_ratio
* **tolerance (optional)** — tolerance for `#` and `%` — `#` and `%` are ignored if computed pitch
is within `tolerance` of a scale degree; applies only when `divisions` is given

## define_isomorphic_mapping

Define an isomorphic mapping for a tuning. The mapping is placed into a
layout with the 'place_mapping' directive.

**Parameters**:
* **mapping** — Name of mapping
* **scale (optional)** — Scale; if omitted, use the current default scale
* **steps_h** — Number of scale degrees to go up in the horizontal direction
* **steps_v** — Number of scale degrees to go up in the vertical or up-right direction

## define_manual_mapping

Define a manual mapping of notes to keyboard positions. The mapping is
placed into a layout with the 'place_mapping' directive.

This directive must be followed by a layout block.

**Parameters**:
* **mapping** — Name of mapping
* **scale (optional)** — Scale; if omitted, use the current default scale
* **h_factor (optional)** — Factor to multiply by the pitches for horizontal tiling of the mapping;
default is 1
* **v_factor (optional)** — Factor to multiply by the pitches for vertical tiling of the mapping;
default is 2

## define_scale

Define a scale. The scale called "default" is pre-defined and corresponds to
12-EDO.

This directive must be followed by a scale block.

**Parameters**:
* **scale** — scale name
* **cycle_ratio (optional)** — ratio to be applied by the octave marker; default is 2 (one octave)

## mark

Mark a moment in the score. The mark may be used for repeats or to generate
a subset of musical output. There are no restrictions around the placement
of marks, but there are restrictions on what marks may be used as repeat
delimiters. See the `repeat` directive.

**Parameters**:
* **label** — The mark's label

## midi_instrument

Set the MIDI instrument number for zero or more parts. If no part is
specified, this becomes the default instrument for all parts without a
specific instrument. It is an error to name a part that doesn't appear
somewhere in the score.

**Parameters**:
* **instrument** — Midi instrument number from 1 to 128
* **bank (optional)** — Optional bank number from 1 to 16384
* **part (repeatable)** — Which parts use this instrument; if not specified, all unassigned parts
use it

## place_mapping

Place a mapping onto a layout for a keyboard.

**Parameters**:
* **layout** — Name of layout
* **mapping** — Name of mapping
* **base_pitch (optional)** — Base pitch; defaults to the base pitch of the default tuning
* **keyboard** — Name of keyboard
* **anchor_row** — Row of the base note for isomorphic layouts or the anchor note for
manual layouts
* **anchor_col** — Column of the base note for isomorphic layouts or the anchor note for
manual layouts
* **rows_above (optional)** — Number of rows *above* the anchor position to include in the region
containing the mapping; default is to extend to the top of the keyboard.
May be 0.
* **rows_below (optional)** — Number of rows *below* the anchor position to include in the region
containing the mapping; default is to extend to the bottom of the
keyboard. May be 0.
* **cols_left (optional)** — Number of columns to the *left* of the anchor position to include in the
region containing the mapping; default is to extend to the leftmost
column of the keyboard. May be 0.
* **cols_right (optional)** — Number of columns to the *right* of the anchor position to include in
the region containing the mapping; default is to extend to the rightmost
column of the keyboard. May be 0.

## repeat

Repeat a section of the timeline delimited by two marks. The start mark must
strictly precede the end mark. No tied notes or pending dynamic changes may
be unresolved at the point of the end mark.

**Parameters**:
* **start** — Label of mark at the beginning of the repeated section
* **end** — Label of mark at the end of the repeated section
* **times (optional)**

## reset_tuning

Reset tuning. If no parts are specified, clears all tunings. Otherwise,
resets the tuning for each specified part to use the global tuning.

**Parameters**:
* **part (repeatable)** — Which parts to tune; if not specified, all parts are tuned

## set_base_pitch

Change the base pitch of the current tuning for the named parts, or if no
parts are named, for the default tuning. If `absolute`, use the pitch as the
absolute base pitch. If `relative`, multiply the base pitch by the given
factor. Example: `set_base_pitch(relative="^1|12")` would transpose the
tuning up one 12-TET half step. Only one of `absolute` or `relative` may be
given.

**Parameters**:
* **absolute (optional)** — Set the base pitch of the current tuning to this absolute pitch value
* **relative (optional)** — Multiply the base pitch of the current tuning by the specified factor
* **part (repeatable)** — Which parts to tune; if not specified, all parts are tuned

## syntoniq

Set the syntoniq file format version. This must be the first functional item
in the file.

**Parameters**:
* **version** — syntoniq file format version; supported value: 1

## tempo

Set tempo, with possible accelerando or ritardando (gradual change).

**Parameters**:
* **bpm** — Tempo in beats per minute
* **start_time (optional)** — Optional effective time relative to the current score time. This can be
useful for inserting a tempo change part way through a score line.
Defaults to 0.
* **end_bpm (optional)** — Optional end tempo; if specified, duration is required. Indicates that
the tempo should change gradually from `bpm` to `end_bpm` over
`duration` beats.
* **duration (optional)** — Must appear with `end_bpm` to indicate the duration of a gradual tempo
change.

## transpose

Change the base pitch of the scale in a way that makes the new pitch of
`written` equal to the current pitch of `pitch_from`. For example, you could
transpose up a whole step in 12-TET with `transpose(written="c"
pitch_from="d")`. This method of specifying transposition is easily
reversible even in non-EDO tunings by simply swapping `written` and
`pitch_from`. This can be applied to multiple parts or to the default
tuning. The parts do not all have to be using the same scale as long as they
are all using scales that have both named notes.

**Parameters**:
* **written** — Name of note used as anchor pitch for transposition. In the new tuning,
this note will have the pitch that the note in `pitch_from` has before
the transposition.
* **pitch_from** — Name of the note in the existing tuning whose pitch will be given to the
`written` note after transposition.
* **part (repeatable)** — Which parts to tune; if not specified, all parts are tuned

## use_scale

Change the scale for the specified parts. If no parts are specified, change
the scale used by parts with no explicit scale. This creates a tuning with
the specified scale and the current base pitch.

**Parameters**:
* **scale** — Scale name
* **part (repeatable)** — Which parts to tune; if not specified, all parts are tuned
<!-- generate-end -->
