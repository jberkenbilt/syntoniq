
## check_pitch

Check that all pitches are the same. If multiple parts are specified, all specified notes must
exist in all the parts' tunings. All parameters may be repeated.

**Parameters**:
* **note (repeatable)** — Notes compare
* **var (repeatable)** — Variables to compare
* **pitch (repeatable)** — Pitches to compare
* **part (repeatable)** — Which parts check; if none given, the default tuning is checked.

## csound_global_instrument

Indicate the name or number of a Csound instrument number that must be
turned on at the beginning and must remain on for the direction of the
score. You may optionally provide a value for `tail`, which is a number of
beats beyond the total duration to leave the instrument on. This is useful
for effect instruments, like reverb. This is only useful when combined with
a custom Csound template that defines the instrument.

**Parameters**:
* **number (optional)** — Csound instrument number
* **name (optional)** — Csound instrument name
* **tail (optional)** — Number of beats beyond the end of the piece to add to the instrument's
duration. Defaults to 3 beats.

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

## csound_template

Specify the name of a file, relative to the score file, that contains the
Csound template to use. This can still be overridden from the command line.

**Parameters**:
* **path** — Path to template, interpreted as relative to the score file

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

## restore_pitch

Tune the given parts so that the named note has the pitch that was previously saved to the
given variable.

**Parameters**:
* **note** — Name of the note whose pitch is to be set
* **var** — Name of the variable that contains the pitch
* **part (repeatable)** — Which parts to transpose; if not specified, the default tuning is updated.

## save_pitch

Save the pitch of a note to a variable that can be used with
`restore_pitch`. If no part is given, the note's pitch is retrieved from the
global tuning. If more than one part is specified, the note must have the
same pitch in all the parts. This can be used as a quick sanity check when
saving a note's pitch. See also `restore_pitch` and `check_pitch`.

**Parameters**:
* **note** — Name of the note whose pitch is to be saved
* **var** — Name of the variable to save the note's pitch into
* **part (repeatable)** — Which parts' tuning to get the note's pitch from; if more than one specified, the note
must have the same pitch in all tunings.

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
transpose up a whole step in 12-TET with `transpose(written=c pitch_from=d)`.
This method of specifying transposition is easily reversible even in non-EDO
tunings by simply swapping `written` and `pitch_from`. This can be applied
to multiple parts or to the default tuning. The parts do not all have to be
using the same scale as long as they are all using scales that have both
named notes.

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
