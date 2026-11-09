# To-do List

This is general TODO across internal docs, manual, and software.

# Build/CI

* Do the lychee check (for valid HTML links in manual -- see [manual/README.md](../manual/README.md)) in build_all. Or at least remember about it when adding new external links to the documentation.

# Software

* Periodically search for `jetbrains.com/issue` to see if we can remove workarounds.
  * timeline.rs has a workaround for RUST-20121
* Need volume control for syntoniq-kbd regular and prompt.
* Csound: maybe: interpret accents with envelope, then figure out what this does to articulation adjustment.
* Improve pitch overflow handling
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
  * generate summaries of part -> track/port/channel, etc.
  * Fix a few MIDI generation edge cases/overflow conditions
* Editing experience
  * Create a minimal emacs mode
  * Write LSP
  * Reformatting -- see below

## Keyboard

When transposition is in effect, it is not indicated on the web UI. There should be some indication of transposition, perhaps in the area, which can show the mappings in effect with their transposition. Trying to work it into the note name makes the note name too long and busy.

Bug: hexboard HTML doesn't look good in light mode. Maybe I should hard-code dark mode since it matches the hardware.

# Documentation

* Clean up docs/architecture.md
* Embed KeTeX rather than getting from a CDN
* Pay special attention to "on active" or "on octave" instead of "an octave" and "ration" instead of "ratio"
* Tweak theme for better colors

# Reformatter

These notes predate parser implementation.

Work in: make the representation more compact. Make sure the `:` or `@` markers are in order but don't space things out so much.

* Use a lossless token stream for the reformatter.
* Reformatter: Two-pass (Parse-to-AST, then Token-Walk + AST-Peek).

The reformatter will first fully validate an input file. Then it will drive the formatting from pass1 tokens, peeking at later parsing results for semantic information as needed.

Suggested reformatting rules:
* Collapse multiple blank lines to single blank lines, and remove leading and trailing blank lines
* Remove trailing white space
* In a multi-line structure (score block, scale definition block, multiline directive), keep comments aligned and offset by two spaces from the longest line
* Remove spaces from around `=` in directive parameters
* If a directive with any trailing comment exceeds 100 columns, move the trailing comment to the preceding line. If still over 100 columns, break the directive to one parameter per line.
* If a directive that contains no parameter-level comments fits on one line in <= 100 columns, reformat as a single line. Never move a preceding comment to after a single-line directive or collapse to a single line if there are embedded comments.
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

# Easy Scale Enhancements

There are two easy enhancements we could make:
* define_merged_scale: create a scale by merging other non-generated scales that have no overlapping note names
* snap_to_scale="scale" as a parameter to define_generated_scale; instead of snapping to equal divisions, snap to pitches present in the named scale

This would make it possible to define any manual scale, including an uneven scale, and then snap to it using generated scale notation.

# Stretched/Compressed Tunings

We could add

```
stretch(from=x to=y voice=z ...)
```

with `from` and `to` both defaulting to 2. The result is to multiply the relative pitch by `log_x(y)` before multiplying by the base frequency. So `stretch(to=2.1)` would create a stretch tuning where the octave is 2.1. A note X would be (n/(n-1))^(log_x(y))*base_freq. For semantically written music where we transpose the base pitch to the melodic tonic, when combined with a timbre whose harmonics are designed to align with the stretch, it should create something recognizable. Making this a property of the voice is correct because stretched tunings would be used with specific timbres that are designed to support them.

We should just attach a closed-form expression of the stretch values, along with the base frequency, to the voice and not try to extend the pitch syntax with some log_a(b) syntax. The final pitch would be `(pitch / base) × log_x(y) × pitch`. This basically transposes the pitch by an amount that's linearly related in *scale distance* from the base pitch of the scale at that time.

An alternative would be to add some log_a(b) syntax as a pitch factor. While not as bad as Tenney Weights (see [Generalized Tempered Scales](#generalized-tempered-scales), this would be ugly and have limited usefulness, so I prefer attaching it as its own thing directly the voice. That's also much easier to implement.

# Generalized Tempered Scales

Interesting Reddit post on magic temperament: https://www.reddit.com/r/microtonal/comments/1xpnmcf/piece_piece_using_magic_7_temperament/
Gemini chat where I worked out my mental model: https://gemini.google.com/app/ae0b860f86f74e4f

## Temperament

Temperament shrinks or stretches intervals to make a particular comma disappear. *Meantone temperament* specifically removes the syntonic (81/80) comma.

Create a lattice where each dimension represents multiplying pitch by a particular prime. Example:

ratio = 2^x + 3^y + 5^z

Four perfect fifths minus two octaves = 81/64 = (-6, 4, 0)
A major third = 5/4 = (-2, 0, 1)
The ratio is 81/80 (the syntonic comma).

A pitch on this lattice is

The coefficients are "generators". Tempering ratio means replacing the coefficients with

```
g_x^x * g_y^y * g_z^z
```

two points that should have the same value are the same. For example

```
2^x * 2^(19/12)y * 2^(28/12)z
```

replaces the ratio 3 with a 12-EDO octave and a fifth and 5 with 2 octaves and a major third.

Any g_x, g_y, g_z that evaluates the same for (-6, 4, 0) and (-2, 0, 1) is a meantone temperament. It solves the equations

```
g_x^(-6) * g_y^4 = g_x^(-2) * g_z ==>
g_x^(-4) * g_y^4 = g_z
```

Fixing `g_x = 2` ensures that the octave remains pure, giving

```
2^(-4) * g_y^4 = g_z ==>
g_y^4 = 16 g_z ==>
```
Does it work for 12-EDO? Yes -- both sides evaluate to 2^(76/12). It also works for 19-EDO and 31-EDO, showing that these are all meantone temperaments. You could also do something like fix the third and calculate the fift. This would fix `g_z = 5`, giving

```
g_y = 80^(1/4) =~ 2.99
```

which slightly shrinks the fifth. Or fix the fifth and compute the third:
```
3^4 = 16 g_z ==> g_z = 81/16 =~ 5.0625
```
for a slightly sharp major third. Any collection of values that satisfies this is a meantone temperament. There are various methods, such as Tenney Weighting (see Gemini chat) that minimizes the RMS delta with logarithmic scaling between each interval and its perfect interval. An equal temperament is just one in which a generated scale has intervals that are all the same size. It would have the property that 4*steps-in-fifth = steps in 2 octaves plus major third. 12-EDO: `4*7 = 24 + 4`. 19-EDO: `4*11 = 38 + 6`. 31-EDO: `4*18 = 62 + 10`. But 17-EDO and 41-EDO are not meantone temperaments.

You can temper other commas in exactly the same way. Consider five 8/7 intervals = 32768/16807, which is an octave above 16384/16807. With

2^x + 3^y + 5^z + 7^w, this is (14, 0, 0, 0) = (0, 0, 0, 5). For this, 5-EDO works:

```
g_x = 2
g_w = 2^(14/5) =~ 6.9644
2^14 = 2^(14/5)*5
```

So 5-EDO tempers away 16384/16807.

The magic temperament from the Reddit post is 3125/3072, which is tempered away by 5-ED3. It is removing the comma from comparing 5 perfect major thirds with an octave and a perfect fifth.

The Reddit poster defined a scale with these intervals.
```
0 – 322 – 381 – 703 – 762 – 1084 – 1142 – 1201
```

If you generate a scale based on 5-ED3, you can get this:

```
1           0.000
1/2*3^4|5   321.564
3^1|5       380.391
1/2*3^5|5   701.955
3^2|5       760.782
1/2*3^6|5   1082.346
3^3|5       1141.173
```

The Reddit post comes from the Tenney Weight optimization, which stretches the octave to 1201 cents (from 1200) and the fifth from 702 to 703, but the 5-ED3 result is perceptually indistinguishable.

The Gemini chat outlines how to compute the poster's numbers, but the resulting pitches require 2 to the power of rational functions of products of (k + L_n) where L_n is log_2(nth prime). We could potentially support this, but it doesn't seem worth it.

We could create tempered scales by imposing exactly the right number of constraints between specifying commas and fixed values. For example, comma: 81/80=1, g_x=2, and some fixed value for g_y OR g_z provide enough information to compute the other generator. It could also be done two commas as long as they don't contradict each other. If Syntoniq had a "lattice solver", it could generate arbitrary temperaments. That said, finding an EDO that supports the temperament is the easiest solution.

It would also be possible to reproduce the Reddit post's scale by defining a manual scale either with the 3-ED5 pitches pulled into the octave and the snap to it using generated scale notation or even to define the poster's exact ratios by using `^x|1200` to specify cents. Of course, we could also just assign note names to the pitches in the magic-7 scale.
