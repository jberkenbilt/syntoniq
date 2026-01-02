+++
title = "Notes and Chords"
weight = 30
sort_by = "weight"
+++

This section shows you how to play notes and chords with the Syntoniq keyboard.

Here are a few important takeaways:
* In addition to behaving like a normal keyboard, the Syntoniq keyboard has a *sustain mode*. When sustain mode is on, keys become toggles: when you touch a key, it turns a note on or off. You can combine sustain mode with normal mode for experimenting and building chords, and you can turn all notes off. Sustain mode works across layouts, allowing you to combine notes from different layouts in a single chord.
* The Syntoniq keyboard supports multiple layouts. Syntoniq knows about exact pitches. When you play a note, if there is any visible key that makes that pitch, it will be lit up. Because of the Syntoniq keyboard's transposition and shift features, it is possible that a note may be sounding that is not present on the keyboard.
* The Syntoniq keyboard shows you information on the read-only web UI and also outputs a lot of information to the console. The web UI is geared toward helping you find notes and providing additional information that can't be displayed on the keyboards themselves. The console output is geared toward giving you semantic information about the notes you play and makes the keyboard a good helper for the Syntoniq language. Additionally, each note's color conveys additional meaning. We aim for a consistent color scheme across supported devices subject to the devices' hardware capabilities.

All the material in this section is demonstrated in the video linked below.

{{ youtube(id="Oc_HkZVupjw?si=ObzOvNzB7gsY_KcD", caption="TODO Placeholder Video", script="keyboard-notes-and-chords") }}

Please review the information in the [previous section](../initialization/) for starting the keyboard application and selecting layouts. We'll start off by selecting the `12-EDO-h2v5` layout. This layout is located on the lower-left layout key on the Launchpad. It is assigned to the upper-left layout button on the hexboard.

This is what you see on the web UI for each keyboard.

Launchpad:

{{ include(path="launchpad-layout-1.html") }}

HexBoard:

{{ include(path="hexboard-layout-1.html") }}

Additionally, on the HexBoard, the third, sixth, and seventh (counting from 1, top to bottom) are lit up red, which is not shown on the web UI. Here's what everything means.

## Layout Terminology

We're going to discuss Syntoniq's flexible layout system in a later section, but for now, you have to know the following terms to follow along. The Syntoniq has two kinds of keyboard layouts:
* **Isomorphic Layout** &mdash; "isomorphic" literally means *same shape*. In an isomorphic layout, the interval between two keys is always based solely on the relative positions of those keys. Isomorphic layouts are ideal for scales based on even divisions of some interval (equal-step tunings), though the Syntoniq keyboard allows you to use isomorphic layouts for other kinds of scales...your mileage may vary.
* **Manual Layout** &mdash; this is a kind of layout where you define a section of the keyboard and state exactly which note is assigned to which key. Manual layouts are more complicated but also let you work with tunings that wouldn't work well with an isomorphic layout. *The remainder of this section focuses solely on isomorphic layouts.* We will come back to manual layouts in a later section.

## Colors and Labels

Syntoniq keyboard colors are based on intervals from the base pitch of the scale. The colors have the following meaning based on being close to specific just intonation ratios:

| ratio | interval | color       |
|-------|----------|-------------|
| 1     | yellow   | tonic       |
| 6/5   | red      | minor third |
| 5/4   | pink     | major third |
| 4/3   | green    | fourth      |
| 3/2   | blue     | fifth       |
| 8/5   | purple   | minor sixth |
| 5/3   | orange   | major sixth |
| 2     | yellow   | octave      |
| -     | cyan     | one step    |

Everything else is gray. Colors are brighter when the note is being played and dimmer when the note is off.

The cyan color for one step is used with *isomorphic layouts*, which we will cover later. This helps you orient yourself on the keyboard. The colors were chosen for the following reasons:
* Yellow is the most easily visible color and helps you find the tonic.
* red and orange are similar and are harsher colors, suitable for the minor third and its reciprocal, the major sixth
* pink and purple are similar and are softer colors, suitable for the gentler major third and its reciprocal, the minor sixth
* green and blue are centered colors for the fourth and fifth
* cyan has good contrast with yellow for the single step, and while it's easily confused with blue and green, it typically won't be adjacent to either on the keyboard

Using meaningful colors for intervals helps you find notes and chords on an unfamiliar layout. It can also give you immediate insight about the features of a scale. For example, when we get to other layouts, you'll see that 17-EDO has yellow, cyan, blue, and green, but doesn't have red, orange, pink, or purple. That's because that scale doesn't have any notes that are close enough to thirds and sixths.

If you press notes on the keyboard, you should see LEDs get brighter on the physical keyboard and colors get brighter on the web UI. The colors won't match up perfectly because of differences in color representations. It's possible that what you see may be better or worse than what's in the manual depending on your own hardware and on possible future improvements to the on-screen color matching with the device LEDs.

Let's unpack the labels on the keys. Each button has two rows in its label. The top row is the *note name*, and the bottom note is the *base-relative pitch* of the note in the scale. The rest of this gets more technical&mdash;but this is a microtonal keyboard application, so some complexity is expected! You may want to review the section on [syntoniq pitch notation](../../microtonality/pitch-primer/) if you aren't following. Start with the note labeled `c`. On the Launchpad, this is row 4 (numbered from the *bottom*) and column 3 (numbered from the *left*). On the hexboard, count up from the bottom to the 8th row, then count from the left to the 4th column.

Note 1: Why do we number rows from the bottom? This matches what you'd expect for a keyboard: going up in row number and going up vertically correspond to going up in pitch.

Note 2: I'm glossing over this detail, but counting row and column numbers is complicated for the hexagonal grid. I'll come back to that in the layout section. That's why I used slightly different terminology to help you locate `c` on the keyboards.

For the `c`, you notice that the top row says `c` and the bottom row says `1`. The relative pitch of the tonic in the scale is always `1`. If you look to the right (on either keyboard), you will find `d` with a pitch of `^1|6`. In Syntoniq pitch notation, this represents the value $2^\frac{1}{6} = 2^\frac{2}{12}$, indicating two scale degrees in the 12-EDO scale. On the Launchpad, you can find `f` directly above `c`. On the HexBoard, it's *up and to the left*. On the hexagonal grid, "up" means "up and to the left". Why? It has to mean either "up and to the left" or "up and to the right", and the math is easier in the layout engine if it means "up and to the left!" Both ways would work, and there is no real standard for this. (As it is written, "The nice thing about standards is that there are so many to choose from.") You can see that `f` has the pitch `^5|12`, consistent with its being the fifth chromatic scale degree, counting from 0, of the 12-EDO scale.

Lets go up two rows and find the note labeled `f'`. On the Hexboard, this is two rows above `f`. On the launchpad, it's two rows up and one column to the right because the mappings land slightly differently on a rectangular grid from how the land on a hexagonal grid. Here, you see that the pitch is still `^5|12`. That's because the pitch is *normalized to the cycle size*. This makes it easier to find where a note lands in the scale, at least once you get used to reading Syntoniq pitch notation! The `f'` indicates that the note is *an octave higher* than plain `f`. (If this scale had a cycle size of other than an octave, it would indicate *one cycle higher*; more on that in later sections.)

This is what everything on the screen means for isomorphic layouts. Manual layouts are more complicated, so we'll postpone for now.

## Console Output

Next, let's take a look at the keyboard's console output. If you press and release the `c`, `e`, and `g` keys in order, you will see something like the following output on the console:
```
Note: c (base × 1 = 220*^1|4), scale=12-EDO, base=220*^1|4
Note: e (base × ^1|3 = 220*^7|12), scale=12-EDO, base=220*^1|4
Note: g (base × ^7|12 = 220*^5|6), scale=12-EDO, base=220*^1|4
```
The line for each note will be printed as you play the note. Let's unpack this. I'll use the second line (for `e`) to explain.
* `Note: e` &mdash; this tells you the note name (`e`)
* `(base × ^1|3 = 220*^7|12)` &mdash; this tells you that the pitch is the scale base multiplied by $2^\frac{1}{3} = 2^\frac{4}{12}$, consistent with `e` being the fourth chromatic step of a 12-EDO scale. Then it shows you that the actual pitch is `220*^7|12`. That is 7 semitones above 220 Hz, which is itself an octave below A 440. This is what you'd expect: `e` is a fifth (7 semitones) above `a`.
* `scale=12-EDO, base=220*^1|4` &mdash; this tells you the scale is `12-EDO` and the base pitch is `220*^1|4`, which is the canonical Syntoniq pitch representation of middle C.

That gives you everything there is to know about the pitch. When we come to transposition and tiled manual layouts later, you'll see some additional information that is not displayed here, but for now, that's it.

## Sustain Mode

The sustain key is the `Chord` key on the Launchpad and the third-to-top command key on the HexBoard. Initially, the key color is red, indicating that it is inactive. When you touch the key, it turns green, and you are in sustain mode. While in sustain mode, when you touch a note, it stays on until you touch it again.

If you turn sustain mode on, play some notes, and turn it off, those notes remain on. You can then play other notes in regular mode. This lets you do things like play part of a chord and experiment with other notes to add to the chord. If you find one you like, you can turn sustain mode on, add the note to the chord, and turn it off again.

If you turn sustain off and back on again without pressing any intervening notes, all notes are turned off. From time to time, because of timing issues or possible bugs in the keyboard application, a note may get stuck on. If this happens, you can usually turn it off by entering sustain mode and touching the key. You can also hit reset. As a last resort, exit from the keyboard application and restart.

Another thing that happens in sustain mode is that, when you play a note, the console output shows all the notes that are currently on. For example, if you enter sustain mode and play `c`, `e`, and `g`, you will see something like this:

```
----- Current Notes (2026-01-02 17:50:24 -05:00) -----
Scale: 12-EDO, base=220*^1|4
  Note: c (base × 1 = 220*^1|4)
----- Current Notes (2026-01-02 17:50:25 -05:00) -----
Scale: 12-EDO, base=220*^1|4
  Note: c (base × 1 = 220*^1|4)
  Note: e (base × ^1|3 = 220*^7|12)
----- Current Notes (2026-01-02 17:50:26 -05:00) -----
Scale: 12-EDO, base=220*^1|4
  Note: c (base × 1 = 220*^1|4)
  Note: e (base × ^1|3 = 220*^7|12)
  Note: g (base × ^7|12 = 220*^5|6)
```

If you find a specific chord, you can notate the time so you can come back to it. You can also use this to help you create notes for the Syntoniq language.

On the Launchpad only, you can use the `Capture MIDI` button, which is labeled "Show Notes" in the web UI, to force printing of the current notes at any time whether in sustain mode or not. This feature is not available on the HexBoard.

## 60-degree Layouts (HexBoard)

In the HexBoard only, you will notice that some of the layouts are labeled `hexboard` and some are labeled `hexboard-60`. For the `hexboard-60` layouts, the 60-degree diagonal from bottom left to top right is considered "horizontal". From that angle, "up and left" corresponds to the normal horizontal direction. The 60-degree orientation of the HexBoard gives you wider rows in the center and makes a different subset of notes reachable, especially on scales with many notes per cycle. You can experiment and decide which way you prefer.

## Octave Shift

On the Launchpad, the octave shift keys are the up and down triangular arrow keys. On the HexBoard, they are the fourth and fifth command keys. When you press an octave key, all the keys on the keyboard are shifted up or down an octave. Here are a few things to know about the octave shift keys:

* They apply a special type of transposition, discussed below.
* If you are holding a note down while operating the octave shift keys, the effect is if you released and re-pressed the key. This works in sustain mode as well, so if you are holding a button down in sustain mode while pressing octave keys, additional notes will sound or turn off depending on their current state.
* If you reach the end of the range, the octave keys will cause notes to generate unplayable frequencies. This usually results in silence in Csound mode. In MIDI mode, it will play the highest or lowest note of the same scale degree.

Since the octave keys apply transposition, you can see how it affects console output and web display.

As of the initial release, there is no indication of transposition on the web display at all. The transposition is shown in the console output. For example, if you press `c` after touching the octave down button, you will hear the `c` below middle C, and the console will show

```
Note: c (base × 1 = 110*^1|4), scale=12-EDO, base=110*^1|4 (transposition: 220*^1|4 × 1/2)
```
The text `(transposition: 220*^1|4 × 1/2)` indicates that the base, in this case `110*^1|4`, is derived by taking the original base and multiplying it by a transposition factor of `1/2`. We will see more about this in subsequent sections.
