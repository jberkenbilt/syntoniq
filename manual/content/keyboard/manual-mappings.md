+++
title = "Manual Layout Mappings"
weight = 50
sort_by = "weight"
+++

All the material in this section is demonstrated in the video linked below.

{{ youtube(id="IzjyinrwGnM?si=StIP6uHvQ9G7NayK", caption="TODO Placeholder Video", script="keyboard-manual-mappings") }}

In this section, I will introduce the concept of manual layouts. The next section on the [layout engine](../layout-engine/) includes a more technical and rigorous description and tells you how to create your own layouts.

The built-in keyboard configuration includes one manual layout called "JI" and one hybrid layout that includes an isomorphic layout and manual layout occupying different regions of the keyboard. The accompanying video includes demonstrations of everything I describe here.

Before we dive in, I need to improve the precision of the terms I've been using. The Syntoniq keyboard defines the concept of a *layout* which consists of a number of *mappings*. A *mapping* is the thing that maps notes to keys. Throughout the manual, I have been treating layouts like they contain a single mapping and thus using the terms interchangeably. It's time to stop doing that. When I refer to *layout*, I'm referring to the entire collection of keys on the screen. When I refer to *mapping*, I'm talking about the exact way in which the layout is defined. For a lot of this section, they are the same, but you can create hybrid layouts with multiple mappings, and then it matters.

Below, you will see a view of the web UI for the JI layout. Here is the HexBoard:

{{ include(path="hexboard-ji.html", caption="HexBoard with JI Layout") }}

Here is the Launchpad:

{{ include(path="launchpad-ji.html", caption="Launchpad with JI Layout") }}

There is a lot of new stuff here, so let's explain. As I explain, you can find what I'm talking about in both diagrams.

The first thing to know is that this layout uses Syntoniq's generated note names. These are not your familiar diatonic note names. Please review [Using Generated Scales](../../microtonality/generated-scales/). With generated scales note `A` (also `a`, `A0`, and `a0`) is the tonic of the scale. On the HexBoard, you can find the yellow note with `A` on its top row and `1` on its bottom row by counting up to the seventh row from the bottom and the second column from the left. On the Launchpad, it is row 3, column 2. This particular manual mapping is defined to be two rows high and five rows wide. I will reveal the actual definition in [layout engine](../layout-engine/) section. For now, here's what you need to know:
* The top row is `p F EK e' Bh`
* The bottom row is `A I E D C`
* The anchor note is the `A` at the lower-left.
* The horizontal tile factor is 2.
* The vertical tile factor is 3/2.

As you examine the diagrams, you can see this pattern of notes as stated along with their ratios, computed using the generated note syntax. For example, the note `Bh` has the ratio of 7/6, and the note `e'` is 8/5. In this case, the apostrophe indicating the octave mark *appears explicitly in the mapping's definition*. It is not inferred by Syntoniq. In general, it is not possible to map a tiled note to cycle marks.

The next thing to notice is the arrows. This layout includes the characters ←, →, ↑, and ↓, sometimes followed by a number. These indicate the number of horizontal or vertical tilings of the mapping. For example, if you look 5 keys to the right of the `A` key, you will find a note labeled `A→`. Its ratio is also shown as `1`. If you play these notes one after the other, you will see the following in the keyboard's console output:

```
Note: A (base × 1 = 264), scale=JI, base=264
Note: A→ (base × 1 × 2 = 528), scale=JI, base=264
```

Observe:
* In the web UI, the ratio of both notes is `1`. This is because, as with isomorphic mappings, the web UI always shows relative pitches *normalized* to within the scale's cycle, which in this case is an octave. With manual mappings, relative pitches are *relative to the pitch of the untiled anchor*.
* The note names with the tiling arrows are displayed.
* The pitch has an extra factor for `A→` and is shown as `(base × 1 × 2 = 528)` indicating that the pitch, 528, is the result of multiplying the base frequency (shown as 264) first by 1 (the base-relative pitch) and then by 2, which is the tile factor.

Let's go two rows above that. That note is labeled `A↑→`, and on the keyboard its relative pitch shows 3/2, and the color is blue, our color for fifths. If you play it, you can hear that it is, in fact, an octave and a fifth (which, normalized to the octave, has ratio 3/2) above the tonic. You will also notice that its note name includes `A`, not `C`, which would be 3/2. Why is that? In this case, it happens that `A↑→` lands on a pitch corresponding to a note in the scale, but this doesn't always happen. If you look just one character to the right, you will find the note `I↑→` with the ratio 27/16. While you can spell that note as a generated note (one way is `CI`), the note `CI` doesn't appear in the mapping. If our tile factor were something like `^1|12`, then the tiled note wouldn't land in the JI scale at all (though it could still be written as a generated note).

Quick aside: while it would have been possible to design this so that we always presented a generated note, there are two big reasons not to do this. The main one is that manual mappings don't have to use generated notes at all! The second one is that, even if we said we'd only do this with generated scales, note names would get very long and complex, and there are an infinite number of ways of writing any note, so we'd have to pick some way to do it, and it wouldn't necessarily reflect the harmonic intent! Make sense? If not, no worries...you just accept that there are good reasons we don't try to generate notes outside the mapping!

When we play the note `A↑→`, we see the following output:
```
Note: A↑→ (base × 1 × 3 = 792), scale=JI, base=264
```
Here we have a tile factor of `3`, which comes from one horizontal tiling (factor 2) and one vertical tiling (factor 3/2) since $2 \times \frac{3}{2} = 3$. If you play `I↑→`, you see this:
```
Note: I↑→ (base × 9/8 × 3 = 891), scale=JI, base=264
```
This is showing us that the frequency of 891 Hz comes from the base (264) multiplied by 9/8 (the base-relative pitch) multiplied by 3 (the tile factor). If we had transposition in play, that would also show up here.

This seems complicated, but there are reasons it's done the way it is (and I can assure you, a great deal of thought and iteration went into it!).
* The web UI is about finding the note. When you've defined a manual mapping, you want to see note names that match what you defined, and as explained, there often is no note name that matches the right base-relative pitch.
* The tile factors might not be related to the scale. They might even include `1` to indicate a simple wrap-around. The only way to clearly show tiling is to introduce a new symbol.
* The console output is about knowing the details of the pitch. Tile factor is a different thing from either cycles (octaves, etc.) or transposition. It needs to be shown its own way.

For all these reasons, we've introduced this additional notation. While dense, once you get used to it, it really does pack in the information you need.

I'll wrap up this section by showing you a few more layouts. Here's one that contains more than one mapping. The layout entitled "JI-19-EDO" is shown below. For HexBoard:

{{ include(path="hexboard-ji-19-edo.html", caption="HexBoard with JI-19-EDO Layout") }}

For Launchpad:

{{ include(path="launchpad-ji-19-edo.html", caption="Launchpad with JI-19-EDO Layout") }}

In these mappings, notice that the bottom four rows have generated names using the same mapping as our "JI" layout, and the rows above that contain notes from 19-EDO.

Here's one that uses a completely custom scale that defines 64 notes: one for each of the first 64 steps of the harmonic sequence. The notes are named `h1` through `h64`. On the HexBoard, it leaves part of the layout unmapped. Here it is for HexBoard:

{{ include(path="hexboard-harmonics.html", caption="HexBoard with Harmonics Layout") }}

And here it is for Launchpad:

{{ include(path="launchpad-harmonics.html", caption="Launchpad with Harmonics Layout") }}

The accompanying video will demonstrate both of those layouts.

The shift and transpose features work as you'd expect since tiling of manual mappings still creates an infinite space where you can always know what note should be on a key. The accompanying video demonstrates this as well, including showing how it works with the hybrid layout. In the next section, we'll unveil the full complexity and power of the layout engine and show you how to create your own custom mappings and to create layouts containing multiple mappings.
