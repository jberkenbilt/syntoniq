+++
title = "Shift and Transpose"
weight = 40
sort_by = "weight"
+++

All the material in this section is demonstrated in the video linked below.

{{ youtube(id="hQWnnIIHgWM?si=4DIhmbuE9djknIUN", caption="Shift and Transpose", script="keyboard-shift-transpose") }}

Now that we've covered the basics of using the keyboard to play notes and chords, it's time to talk about the Syntoniq keyboard's shift and transpose features. These are designed to complement the flexible transposition system of the Syntoniq language.

## Overview

In a nutshell, the features do the following:
* Both features are activated by entering a mode, then selecting two keys in sequence.
* **Shift** — shifts the positions of all the keys on the keyboard so that the first key "moves" to the second position. After the move, pressing the second key does whatever pressing the first key used to do, and all the other keys have moved accordingly. It's like sliding the whole layout around. This makes it possible for you to reach notes that are off the edges of the keyboard.
* **Transpose** — transposes the pitches of all the keys on the keyboard so that the first key's pitch is assigned to the second key. All other notes in the scale are transposed so the relative pitches of the notes stay the same. This operation is effectively the same as `transpose(from_pitch="first-key" written="second-key")` in the Syntoniq language.

Noticing that both operations transfer some property of the first key to the second key can help you remember how to use the features: shift transfers the physical position, and transpose transfers the pitch.

On the Launchpad, there's actually a "Shift" key, and while intended as a general modifier, we grab it for the shift feature because of the matching label. We use the Launchpad "Note" key for transpose.

On the HexBoard, shift and transpose are assigned to the sixth and seventh command keys respectively. As a mnemonic to remember which is which, recall that shift precedes transpose alphabetically.

# Shifting Layouts

The next section will dive more deeply into layouts, but here are a few things you need to know now.

In the [previous section](../notes-and-chords/), we introduced the idea of isomorphic and manual layouts. I'll provide a little more information here. This is a bit technical, but it will help you understand what's going on.

In an isomorphic layout, we establish the location for an *anchor pitch*, which is the tonic of the scale. We also establish a number of *scale degrees* to go up when we move one key to the right and one key up (which, on a hexagonal keyboard, we define as "up and to the left"). On the HexBoard, we've been using the default 12-EDO layout, which is called "12-EDO-h2v5". This is just a naming convention in the default keyboard file indicating "two steps horizontal" (h2) and "five steps vertical" (v5). That means you can calculate the scale degree of any note knowing its offset from the anchor key. For example, in 12-EDO with that configuration, going up two rows and to the right one column brings you $2\times 5+2 = 12$ steps up, which is one octave. Moving one row up and two columns to the left brings you $5-2\times 2 = 1$ step up. Moving two rows down five steps to the right brings you $-2\times 5 + 5\times 2 = 0$ steps away and is a fixed point on the keyboard. You can see all these patterns if you look at the layout on the web UI. In this layout, you can find two locations for `c#` key (among others) on both the HexBoard and Launchpad keyboards.

When we perform a shift operation on an isomorphic keyboard, we are doing nothing other than moving the position of the anchor pitch by the same amount as the delta between the first and second keys. That means that, if you wanted to reach a note two columns off the right edge of the keyboard, you could just use shift to slide everything two or more columns to the left, and the missing note would appear. It's like scrolling around on an infinite document, where the keyboard is showing you however much of the document fits in the available space.

With manual layouts, things are a bit more complicated, and we'll discuss it in a later section...but here's a teaser. Unlike with isomorphic layouts, the Syntoniq keyboard can't calculate the note of a key so easily. In the layout specification, which we will discuss in the subsequent sections, you can define a manual layout by providing a rectangular grid of note names, indicating which one is the anchor, and stating where to place the note on the keyboard. This enables the Syntoniq keyboard to compute the note belonging to any key in that grid. What about notes off the edges? The Syntoniq keyboard allows manual layouts to *tile*. When you define the mapping, you can give a relative pitch offset to be applied to each note as the entire grid is repeated horizontally and vertically. So you can shift a manual layout as well! For the rest of this section, we'll focus on isomorphic layouts, but we will cover manual layouts in their full glory later in the manual!

If you're running the keyboard application now, you can try a shift. Just touch the shift key and then touch two notes. You should see all the lights jump to the new positions, and whatever sound and color were on the old key will now be on the new key. This is demonstrated in the accompanying video.

Here are a few more things to know about shift:
* When you activate shift, the shift key turns from red to green. When you have pressed the first key, the shift key turns purple.
* You can use the shift key *modally*, meaning that you press and release shift, then press and release each of the two keys.
* You can also use the shift key as a *modifier*, meaning that you can hold the shift key down while you press and release the two note keys.
* You can only shift by pressing two keys in the same *mapping*. We haven't talked about mappings yet, but we'll cover that in the section about the [layout engine](../layout-engine/).
* You can cancel shift by pressing the shift key again before the second note. If you are using the shift key as a modifier, releasing the key before the second note will cancel the operation.

# Transpose

The transpose key's user interface is exactly like the shift key in that it can be used modally or as a modifier. Cancellation works the same way. The main difference is that transpose can be used *across layouts*. That means you can switch layouts between the first and second keys of a transpose operation. To show that off, we can transpose the 19-EDO scale up a single 12-EDO step! Here's how it works.

* Select the 19-EDO layout. On the Launchpad with the built-in layouts, it's the second layout. On the HexBoard, it's the third layout.
* In this layout, you can find `c` on one of the yellow keys near the center. On the Launchpad, it's row 4 (counting up from row 1 at the bottom), column 3. On the HexBoard, it's the only yellow key in row 8 from the bottom. That's the row that's vertically aligned with the top command key.
* The "one step up" note is cyan and can be found two rows up and one column to the left. (This layout is called "19-EDO-h3v2", and $2\times 2 - 3 = 1$).
* Play the `c` and `c#` (`^1|19`) keys to get the sound of the pitches in your ear.
* Press the transpose key. This is labeled "Note" on the Launchpad and is the bottom command key on the HexBoard. Notice that it turns green.
* Press the cyan key belonging to `c#`. Notice that the transpose key turns purple.
* Select the first layout using the first layout key on the Launchpad or the layout selection followed by the first key on the HexBoard. The transpose key remains purple after the new layout is selected. (On the HexBoard, all the command keys turn off when you press the layout key, and you can cancel layout selection by hitting the layout key again.)
* Press the key assigned to `c` on the 12-EDO layout.
* Transpose turns red again.
* Press the `c` key. You should now hear the pitch previously assigned to the 19-EDO `c#` key!

As of the initial release, the web UI doesn't give any indication about transposition, but you can see all the details in the console output. Try playing the `c` and `d` keys. You will see the following on the console:
```
Note: c (base × 1 = 220*^23|76), scale=12-EDO, base=220*^23|76 (transposition: 220*^1|4 × ^1|19)
Note: d (base × ^1|6 = 220*^107|228), scale=12-EDO, base=220*^23|76 (transposition: 220*^1|4 × ^1|19)
```
Let's unpack the `d` line:
* `Note: d` — the note name, which remains the same
* `(base × ^1|6 = 220*^107|228)` — the absolute pitch but still indicating that the pitch was computed as `^1|6` over the base; the semantic information about the pitch is preserved
* `scale=12-EDO, base=220*^23|76` — tells us that we are in the 12-EDO scale with a base pitch of `220*^23|76`
* `(transposition: 220*^1|4 × ^1|19)` — tell us that the base pitch was computed as `220*^1|4`, which was our original base pitch, multiplied by `^1|19`, the transposition amount

That's a lot of information! And if we said that the `d` was 163.158¢ over the base pitch or that its frequency was 304.576 Hz, none of that semantic information would be conveyed.

Using one scale to transpose another scale is an example of what you can do with the Syntoniq keyboard and the Syntoniq language as well. When we cover more advanced layout concepts, you can see how this can make it possible for you to experiment with complex just intonation tunings or scales with more notes than you can fit on the keyboard. That's coming up in the next sections!
