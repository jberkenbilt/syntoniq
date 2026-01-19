+++
title = "Layout Engine"
weight = 60
sort_by = "weight"
+++

This section describes the Syntoniq keyboard's layout engine. The primary objective is to explain how to create your own layouts. We cover
* The concepts of layouts and mappings
* How to define isomorphic mappings
* How to define manual mappings
* How to place mappings on layouts
* The built-in default keyboard configuration file

# Defining Manual Mappings

Manual mappings are defined with the `define_manual_mapping` directive. This directive must be followed by a valid layout definition block. A layout definition block contains rows of note names, optionally followed by cycle markers (`'` or `,` optionally followed by a number). These represent a rectangular region of keys. You may use the value `~` to leave a specific key unmapped. Exactly one note (or `~`) must be preceded by `@`. This marks it as the *anchor note*, which is placed in a layout using `place_mapping`. In the example below, the anchor note is the lower-left note, but it can be any note.

When you place keys with manual mapping, you are free to use whitespace in whatever way works best. In the `example` mapping shown below, we stagger the keys so they are easier to visualize on a hexagonal grid. This has no syntactic significance, and it has no effect on a rectangular grid, but it can help you to lay things out visually so it looks more like it will actually appear on a hexagonal keyboard. This is entirely optional. The Syntoniq parser does not consider spaces other than to separate notes from each other. A future version of Syntoniq may include a reformatter that can help with aligning layout definitions. (FUTURE: update this section if we write the formatter.)

```syntoniq
define_manual_mapping(
    mapping="example"
    scale="JI"
    v_factor=3/2
    h_factor=2
) <<
EK  e'  Bh
  p   JK   F
D   C   ~
  @A  I   E
>>
```

# Built-in Configuration

This is the built-in keyboard configuration. You can retrieve this by running `syntoniq-kbd default-config`. After the code, we will describe what's going on with some examples.

<!-- generate include=keyboard.stq checksum=aca8046550445a440a13940977350c06750f809142e852d8844454b20c53b0d1 -->
```syntoniq
syntoniq(version=1)

define_isomorphic_mapping(mapping="12-EDO-h2v5" steps_h=2 steps_v=5)
place_mapping(layout="12-EDO-h2v5" mapping="12-EDO-h2v5" keyboard="launchpad" anchor_row=4 anchor_col=3)
place_mapping(layout="12-EDO-h2v5" mapping="12-EDO-h2v5" keyboard="hexboard" anchor_row=8 anchor_col=8)
place_mapping(layout="12-EDO-h2v5" mapping="12-EDO-h2v5" keyboard="hexboard-60" anchor_row=9 anchor_col=8)

define_isomorphic_mapping(mapping="19-EDO-h3v2" scale="19-EDO" steps_h=3 steps_v=2)
place_mapping(layout="19-EDO-h3v2" mapping="19-EDO-h3v2" keyboard="launchpad" anchor_row=4 anchor_col=3)
place_mapping(layout="19-EDO-h3v2" mapping="19-EDO-h3v2" keyboard="hexboard" anchor_row=8 anchor_col=7)
place_mapping(layout="19-EDO-h3v2" mapping="19-EDO-h3v2" keyboard="hexboard-60" anchor_row=9 anchor_col=8)

define_isomorphic_mapping(mapping="31-EDO-h5v3" scale="31-EDO" steps_h=5 steps_v=3)
place_mapping(layout="31-EDO-h5v3" mapping="31-EDO-h5v3" keyboard="launchpad" anchor_row=4 anchor_col=3)
place_mapping(layout="31-EDO-h5v3" mapping="31-EDO-h5v3" keyboard="hexboard" anchor_row=7 anchor_col=8)
place_mapping(layout="31-EDO-h5v3" mapping="31-EDO-h5v3" keyboard="hexboard-60" anchor_row=8 anchor_col=7)

define_generated_scale(scale="17-EDO" divisions=17)
define_isomorphic_mapping(mapping="17-EDO-h3v2" scale="17-EDO" steps_h=3 steps_v=2)
place_mapping(layout="17-EDO-h3v2" mapping="17-EDO-h3v2" keyboard="launchpad" anchor_row=4 anchor_col=3)
place_mapping(layout="17-EDO-h3v2" mapping="17-EDO-h3v2" keyboard="hexboard" anchor_row=8 anchor_col=7)
place_mapping(layout="17-EDO-h3v2" mapping="17-EDO-h3v2" keyboard="hexboard-60" anchor_row=9 anchor_col=8)

define_generated_scale(scale="41-EDO" divisions=41)
define_isomorphic_mapping(mapping="41-EDO-h7v3" scale="41-EDO" steps_h=7 steps_v=3)
place_mapping(layout="41-EDO-h7v3" mapping="41-EDO-h7v3" keyboard="launchpad" anchor_row=4 anchor_col=3)
place_mapping(layout="41-EDO-h7v3" mapping="41-EDO-h7v3" keyboard="hexboard" anchor_row=7 anchor_col=8)
place_mapping(layout="41-EDO-h7v3" mapping="41-EDO-h7v3" keyboard="hexboard-60" anchor_row=8 anchor_col=7)

define_generated_scale(scale="53-EDO" divisions=53)
define_isomorphic_mapping(mapping="53-EDO-h9v4" scale="53-EDO" steps_h=9 steps_v=4)
place_mapping(layout="53-EDO-h9v4" mapping="53-EDO-h9v4" keyboard="launchpad" anchor_row=4 anchor_col=3)
place_mapping(layout="53-EDO-h9v4" mapping="53-EDO-h9v4" keyboard="hexboard" anchor_row=7 anchor_col=8)
place_mapping(layout="53-EDO-h9v4" mapping="53-EDO-h9v4" keyboard="hexboard-60" anchor_row=9 anchor_col=7)

define_manual_mapping(
    mapping="JI"
    scale="JI"
    v_factor=3/2
    h_factor=2
) <<
p    F   EK    e'   Bh
  @A   I    E    D    C
>>
place_mapping(
    layout="JI"
    base_pitch=264
    keyboard="hexboard"
    mapping="JI"
    anchor_row=7
    anchor_col=5
)
place_mapping(
    layout="JI"
    base_pitch=264
    keyboard="launchpad"
    mapping="JI"
    anchor_row=3
    anchor_col=2
)
place_mapping(
    layout="JI-19-EDO"
    keyboard="launchpad"
    mapping="JI"
    anchor_row=1
    anchor_col=1
    rows_above=3
    rows_below=0
)
place_mapping(
    layout="JI-19-EDO"
    keyboard="launchpad"
    mapping="19-EDO-h3v2"
    anchor_row=6
    anchor_col=3
)
place_mapping(
    layout="JI-19-EDO"
    keyboard="hexboard"
    mapping="JI"
    anchor_row=1
    anchor_col=2
    rows_above=3
    rows_below=0
)
place_mapping(
    layout="JI-19-EDO"
    keyboard="hexboard"
    mapping="19-EDO-h3v2"
    anchor_row=9
    anchor_col=9
)

define_scale(scale="harmonics") <<
 1 h1    2 h2    3 h3    4 h4    5 h5    6 h6    7 h7    8 h8
 9 h9   10 h10  11 h11  12 h12  13 h13  14 h14  15 h15  16 h16
17 h17  18 h18  19 h19  20 h20  21 h21  22 h22  23 h23  24 h24
25 h25  26 h26  27 h27  28 h28  29 h29  30 h30  31 h31  32 h32
33 h33  34 h34  35 h35  36 h36  37 h37  38 h38  39 h39  40 h40
41 h41  42 h42  43 h43  44 h44  45 h45  46 h46  47 h47  48 h48
49 h49  50 h50  51 h51  52 h52  53 h53  54 h54  55 h55  56 h56
57 h57  58 h58  59 h59  60 h60  61 h61  62 h62  63 h63  64 h64
>>

define_manual_mapping(mapping="harmonics" scale="harmonics") <<
h57 h58 h59 h60 h61 h62 h63 h64
h49 h50 h51 h52 h53 h54 h55 h56
h41 h42 h43 h44 h45 h46 h47 h48
h33 h34 h35 h36 h37 h38 h39 h40
h25 h26 h27 h28 h29 h30 h31 h32
h17 h18 h19 h20 h21 h22 h23 h24
h9  h10 h11 h12 h13 h14 h15 h16
@h1 h2  h3  h4  h5  h6  h7  h8
>>
place_mapping(
    layout="harmonics"
    mapping="harmonics"
    base_pitch=50
    keyboard="launchpad"
    anchor_row=1
    anchor_col=1
)
place_mapping(
    layout="harmonics"
    mapping="harmonics"
    base_pitch=50
    keyboard="hexboard"
    anchor_row=4
    anchor_col=4
    rows_above=7
    rows_below=0
    cols_right=7
    cols_left=0
)

define_generated_scale(scale="13-ED3" cycle_ratio=3 divisions=13)
define_isomorphic_mapping(mapping="13-ED3-h2v3" scale="13-ED3" steps_h=2 steps_v=3)
place_mapping(layout="13-ED3-h2v3" mapping="13-ED3-h2v3" keyboard="hexboard" anchor_row=7 anchor_col=8)

define_generated_scale(scale="27-ED3" cycle_ratio=3 divisions=27)
define_isomorphic_mapping(mapping="27-ED3-h3v5" scale="27-ED3" steps_h=3 steps_v=5)
place_mapping(layout="27-ED3-h3v5" mapping="27-ED3-h3v5" keyboard="hexboard" anchor_row=7 anchor_col=8)
```
<!-- generate-end -->

# Note Computation Examples

This section is technically dense. Understanding it is not essential to using the keyboard. Feel free to skim or skip, picking up below with [Other Considerations](#other-considerations). When using the Syntoniq keyboard, the calculations are done manually. The main thing you have to understand is that manual layouts map notes in a *physically rectangular region* even on a hexagonal keyboard. You can develop a feel for this by playing around with defining manual layouts to see what they look like on a keyboard. This section explains it fully. We discuss
* Determining which of multiple mappings in a layout is responsible for mapping a particular key
* Calculating notes in an isomorphic layout based on offsets from the anchor
* Calculating notes in a manual layout, including tiling and *stagger*, which only applies to manual mappings on a hexagonal grid.

Here are a few examples of computing pitches and notes. We'll use the `JI-19-EDO` layout because this is the most general, including both an isomorphic and a manual mapping. If you can do this one, you can do any of them.

Some general notes, which you can verify from the keyboard configuration shown above.

* The `JI` mapping
  * has `v_factor=3/2` and `h_factor=2`
  * is 5 columns wide
  * is 2 rows high
  * specifies the lower-left note as the anchor (using `@` before the note name)
* The `19-EDO-h3v2` mapping is an isomorphic mapping with `steps_h=3` and `steps_v=2`, hence `h3v2`.

Below are some walk-throughs of computing notes on the various keyboards. You can verify these using the diagrams.

## Launchpad Computations

Here is a diagram of the JI-19-EDO layout on the Launchpad:

{{ include(path="launchpad-ji-19-edo.html", caption="Launchpad with JI-19-EDO Layout") }}

On the Launchpad:

* **Row 7, Column 5**:
  * The first `place_mapping` for `JI-19-EDO` with `keyboard="launchpad"` places the `JI` mapping with its anchor at `row=1`, `col=1`, and specifies `rows_above=3`, and `rows_below=0`. That means all columns (infinite in both directions) and rows 1 (anchor row: 1 - rows below: 0) through 4 (anchor row: 1 + rows above: 3), inclusive, are claimed by that mapping. Since row 7 is not between 1 and 4 inclusive, it doesn't belong to that mapping.
  * The second mapping places `19-EDO-h3v2` with its anchor at `row=6`, `col=3`. That means row 7, column 5 is one row above and two columns to the right of the anchor. That means its scale degree is $1\times 2$ (from one row above) $+ 2 \times 3$ (from two columns to the right), which is 8, so its base pitch should be `^8|19`. Consulting the built-in scales, that would have the note name `f`.
* **Row 8, Column 8**:
  * Row 8 is outside the range for `JI`.
  * Row 8, column 8 is two rows above and five columns to the right of the anchor. Its scale degree is $5 \times 3 + 2 \times 2 = 19$. This is one complete cycle above the root, giving it pitch `c'`.
* **Row 3, Column 7**:
  * Row 3 is in the range 1 to 4, inclusive, so it is handled by JI.
  * Row 3 is two rows above the anchor. Since the mapping is two rows high and located in row 1, row 3 is the same as row 1 but with one vertical tiling.
  * Column 7 is 6 columns to the right of the anchor. Since the mapping is 5 columns wide, this is one horizontal tiling plus one column to the right of the anchor.
  * One column to the right of the anchor is `I`.
  * Since we have one horizontal and one vertical tiling, this note is `I↑→`.
  * Since `I` is `9/8`, the horizontal tile factor is `2`, and the vertical tile factor is `3/2`, this notes relative pitch is `27/8`.
  * Normalizing `27/8` to within the cycle (which is 2, an octave) brings us to `27/16`, so this is note `I↑→` with pitch `27/18`.

## HexBoard Computations

Here is a diagram of the JI-19-EDO layout on the HexBoard:

{{ include(path="hexboard-ji-19-edo.html", caption="HexBoard with JI-19-EDO Layout") }}

Before we can work with this using the HexBoard, we have to introduce the concept of *stagger*. As discussed above, the *up* direction on a hexagonal keyboard is *up and to the left*. This is fine with isomorphic layouts, but with manual layouts, we always define a group of keys in a rectangular layout and tile them rectangularly. We effectively have a rectangular grid of groups of hexagonal keys. For this reason, we *stagger* columns for purposes of finding notes in a manual layout. Specifically, we take the *Euclidean quotient* of the *number of rows above the anchor row* and 2 and subtract that from the column. The Euclidean quotient is the integer part of the result of Euclidean division, which takes a quotient and a remainder that is always between 0 and the denominator. For positive numbers, it's the same as integer division. For negative numbers, you move toward negative infinity rather than toward 0. This is usually written as $\lfloor \frac{a}{b}$\rfloor$. This "corrects" for the columns drifting to the right. Because of stagger, you will have the best results for manual layouts if you ensure they are *an even number of rows high*. Note that you can leave some keys in a manual mapping blank by using the note `~` in a given spot.

To make it concrete, observe that the note `A` appears in the above diagram on row 1, column 2. Two physical, rectangular rows above that, meaning two rows above in the 90° *vertical* direction, not the *up and left* direction, we would want to find the note `A↑` because we want manual layout tiling to be vertical, not skewed to the left. You can see `A↑` in that spot on the keyboard. It would be easier if the vertical neighbor two rows above row 1, column 2 were row 3, column 2, but it's not: it's row 1, *column 3* because of the up/left direction of column numbering. To account for this, we introduce the concept of stagger. The stagger here is 1 ($\lfloor \frac{3-1}{2} \rfloor = 1$), which means we want to place all notes one column to the right over where they would land if we just added to the row. In other words, if we look at row 3, column 3, we should find the note that is two rows above row 1, *column 2*, which is $3 - 1$. That means we need to *subtract* the stagger amount from the column on the keyboard to get the column to search in the mapping. It's a bit much, but you can convince yourself by counting it out. If we didn't have stagger, manual mappings taller than two rows would be shaped like parallelograms, and we would tile that way as well. This doesn't work very well when you have a physically rectangular keyboard with hexagonal keys! This scheme works perfectly with the [HexBoard](../../images/hexboard.jpg), but it would work with larger, more complex hexagonal keyboards (like the [Lumatone](../../images/lumatone.png)) as well.

* **Row 7, Column 6**:
  * The first `place_mapping` for `JI-19-EDO` with `keyboard="hexboard"` places the `JI` mapping with its anchor at `row=1`, `col=2`, and specifies `rows_above=3`, and `rows_below=0`. That means all columns (infinite in both directions) and rows 1 (anchor row: 1 - rows below: 0) through 4 (anchor row: 1 + rows above: 3), inclusive, are claimed by that mapping. Since row 7 is not between 1 and 4 inclusive, it doesn't belong to that mapping.
  * The second mapping places `19-EDO-h3v2` with its anchor at `row=9`, `col=9`. That means row 7, column 6 is one row below and three columns to the left of the anchor. That means its scale degree is $-2\times 2$ (from two rows below) $+ -3 \times 3$ (from three columns to the left), which is $-13$. We add 19 to get this in the range from 0 to 18, meaning that this is degree 6 one octave below. It's base pitch should be `^6|19`. Consulting the built-in scales, that would have the note name `e`, so our final note is `e,` with scale degree `^6|19`.
* **Row 4, Column 10**:
  * Row 4 is in the range 1 to 4, inclusive, so it is handled by JI.
  * Row 4 is two rows above the anchor. Since the mapping is two rows high and located in row 1, row 4 is the same as row 2 but with one vertical tiling.
  * Since this is a hexagonal keyboard, we have to take stagger into account. Row 4 is 3 rows above the anchor row we take the Euclidean quotient of 3 by 2, or $\lfloor \frac{3}{2} \rfloor$, which is 1. This is our stagger. We subtract this from the column and consider column 9 in row 2.
  * Column 9 is 7 columns to the right of the anchor column, column 2. Since this mapping is 5 columns wide, we are one horizontal tiling and two columns to the right of the anchor. This means we are looking at the note in the third column of the second row (from bottom to top), which is `EK`.
  * `E` is 5/4 and `K` is 11/10, which makes `EK` 11/8. Our final note is `EK↑→` with a base-relative pitch of 11/8.

# Other Considerations

## Isomorphic Keyboard Coverage

An isomorphic layout will always repeat. If you go to the right by the `steps_v` amount and down by the `steps_h` amount (or, equivalently, up by `steps_v` and to the left by `steps_h`) you will always land back at the same note because you are computing $(h \times v) + (h \times -v)$.

There is no guarantee that you will cover all the notes. To cover all the notes, at least one of the following must be true:
* Number of scale degrees and `steps_v` have no common factors
* Number of scale degrees and `steps_h` have no common factors
* `steps_v` and `steps_h` have no common factors
* One of the step sizes is 1

This is just modular arithmetic, so I won't "prove" that this is true here, but you can count it out and see that it's true. For example, if you had a 12-EDO with `steps_h=2` and `steps_v=4`, you would find repeating patterns, but you would only be able to reach the even steps. For very large EDOs, you can use this intentionally. For example, one way to create a 72-EDO layout would be to use step sizes `steps_h=4`, `steps_v=10`, which would give you only the even keys (since $2$ is the greatest common factor of 4, 10, and 72). You could then have a separate mapping covering part of the keyboard (maybe just one row or even just part of one row) that had `steps_h=1`. You could use that row as a *transposition bar* when building chords. This would make it cumbersome to play arbitrary `72-EDO` intervals, but you could construct chords with sustain mode by adding the even notes, transposing, and adding the odd notes. A similar trick could work with `106-EDO` by creating offsets that only cover the even notes and using a single-step 106-EDO transposition bar.

## Isomorphic Mappings and Uneven Divisions

An isomorphic mapping works best with even divisions of an interval. This is the only way you get the exact same interval with the same relative position. Syntoniq does not enforce this: you are free to create scales with uneven divisions (such as any just intonation scale) and use an isomorphic layout with it. Notes are assigned to keys based on *scale degree*, not based on pitch. This might be useful in some cases, such as if you are intentionally playing with out-of-tune intervals in just intonation or if you are using some other uneven arrangement and want the convenience of not manually laying out the notes and are willing to tolerate that chord shapes will not be truly isomorphic.

# Example: 27-ED3

Let's wrap up with an interesting scale: *27-ED3*. This is 27 divisions of the *tritave*, ratio 3, which is an octave and a just intonation perfect fifth. This is the third harmonic. Some scales, such as Bohlen-Pierce (13-ED3), are based on the tritave. Below is a diagram of 27-ED3 on the HexBoard.

{{ include(path="hexboard-27-ed3.html", caption="HexBoard with 27-ED3 Layout") }}

Here are several things to notice:
* You can see yellow, cyan, green, and blue, but there is no red, orange, pink, or purple. The reason is that no notes in this scale (which is very similar to 17-EDO as, in 17-EDO, step 27 is very close to a tritave) are sufficiently close to a major or minor third, but we have good approximations of fourth and fifth. Cyan is present because we always use cyan for the single step note in an isomorphic layout.
* We have the note `B`, which has the ratio 2/1. This is an octave over `A`. We don't see `B` by itself in an octave-based scale as the octave would just be `A'`, but here, `A'` would be three times the frequency of `A`. The pitch of `B` is `3^17|27`, which is $3^{\frac{17}{27}}$, which is 1.997145, or 1,197¢. This is only 3¢ below 1,200, which puts is close enough to the octave to color the note yellow. This signals that we have a close octave in this tritave-based tuning.
