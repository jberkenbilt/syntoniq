+++
title = "Keyboard: Manual Layout Mappings"
weight = 40
sort_by = "weight"
+++

[This video accompanies [SYNTONIQ KEYBOARD -> Manual Layout Mappings](../../keyboard/manual-mappings/).]

[Present on the screen: a video of one of the controllers when in use, the web UI, and a terminal window.]

In this video, I'm going to explain how manual layouts work with the Syntoniq keyboard. Before I dive in I need to clarify some terminology. In previous videos, I've used the term "layout" to refer to the keys on the keyboard and how they map to notes and pitches. The Syntoniq keyboard distinguishes between a layout and a mapping. The term "mapping" refers to the mapping between keys and notes, and it is actually the thing that can be isomorphic or manual. A layout is created by placing mappings on the physical keyboard. Up to now, all our layouts have contained a single mapping, so I have used the term "layout" exclusively. In this video, I will be more precise. When I say "layout", I'm talking about the whole keyboard. When I say "mapping," I'm talking about the specific part of the layout definition that tells the system which key goes to which note. We'll start off with a layout that contains a single manual mapping, but I will be demonstrating a layout with more than one mapping later in this video.

In this video, I'll stick to the HexBoard. I won't be introducing any new user interface features, so nothing about this is specific to the HexBoard. If you have a Launchpad, the layouts I use here are available there as well. The manual, which you can find at syntoniq.cc, also includes diagrams of the layouts for the layouts used in this video for both HexBoard and Launchpad.

For this part of the video, I'll use the layout called "JI". We can find it as the 13th layout. [Load the layout.] There's a lot going on here, so I'll explain a few things. The first thing you might notice is that this layout has weird looking note names like EK, I, and Bh. There are also arrows. I'll talk about both. This layout uses the built-in just intonation scale called JI. It is a Syntoniq *generated scale*. The manual talks about generated scales in depth, but if your first exposure to Syntoniq is through these videos, you may not have seen it yet. I'll do a quick crash course with just enough information for you to follow along.

Let me say up front that this is a little more technical than what we've seen before. If you don't follow this explanation, don't worry; you can still get something from the video. The main thing you need to keep in mind is that each of these notes plays some pitch. You don't have to actually understand the meanings of the notes to follow the video. But let me explain anyway as it will help.

The generated scale system in Syntoniq allows notes to be constructed on the fly by combining letters. Each letter represents an interval ratio from just intonation. The root pitch of the scale is always `A`. Each uppercase letter from `B` to `Y` represents a ratio of the form [say n over n minus 1] $n/n-1$, and each lowercase letter represents $n-1/n$. The value of $n$ comes from the position of the letter in the alphabet, so `B` is 2, `C` is 3, `D` is 4, etc. That means `C` represents the ratio 3/2, `E` is 5/4, and so on. When you place two letters together, it means to multiply. So `EK` is [say five fourths times eleven tenths, which is eleven eighths] $\frac{5}{4} \times \frac{11}{10} = \frac{11}{8}$. `Bh` is two times seven eighths, or seven fourths. This allows us to play some notes that include the seventh and eleventh harmonics without having to invent special note names. There's a lot more to the Syntoniq generated note system. If you want the details, I refer you to the manual. For now, you can see the ratios in the notes here [point to the untiled section]. These notes, which have no arrows on them, show you the ratios associated with each pitch, so you can see hear that `EK` is 11/8 and `Bh` is 7/4. Here's what they sound like. [Play A, E, C, Bh, EK→] This is a nice 11th-ish chord but with low harmonics. It's the kind of thing you can do with the Syntonic system.

Another thing you'll notice here is that there are arrows. The arrows indicate tiling. In an isomorphic layout, you can always compute the note on any key by knowing its offset from the anchor pitch. We talked about this in the last video. For manual mappings, the Syntoniq keyboard knows which note is the anchor pitch and where to place it, and then it can find other notes within that region. It also allows you to specify tiling. A manual mapping always defines a rectangular region. These regions are staggered on a hexagonal keyboard so rows stack vertically. The entire rectangular region is tiled. When you place a manual mapping in layout, you tell the keyboard what to do with the pitches when you tile horizontally or vertically. In this layout, the horizontal tile factor is 2. That means that each right arrow indicates the pitch going up by an octave. Here's `A` [play `A`] and `A→` [play `A→`]. They are an octave apart. You can also see that both keys are yellow, and both keys show the base-relative pitch to be `1`. This is because we always normalize base-relative pitch to be within the cycle, which in this case is an octave. The color of the key is also yellow, indicating the same pitch class as the tonic.

If we play `A↑` [do that], we hear a pitch a fifth above the tonic. You can see its ratio is 3/2, which is the vertical tile interval in this layout. You can also see that the note's color is blue, which is the color for a fifth. The color and ratios are normalized to within the cycle *and* they are adjusted to be relative to the pitch of the untiled anchor note. This makes it easier for you to figure out what note you're going to hear. What if you tiled both ways? Here's `A↑→`. [Play that.] It's an octave and a fifth above the tonic. It's color is blue, and its ratio is 3/2.

Let's take a look at the console output. Here you can see the note we just played, `A↑→`. [Show that. It says `Note: A↑→ (base × 1 × 3 = 792), scale=JI, base=264`.] You see the base pitch as 792, but this time, there's an additional factor. This shows as base times 1, which is the base-relative pitch of the tonic, times 3, which is the tile factor resulting from one horizontal tiling (factor 2) and one vertical tiling (factor 3/2). Since three halves times two is three, we see a factor of 3. Let's take the note to the right of this: `I↑→`. Since `I` is the 9th letter of the alphabet, its ratio is 9/8. When I play that note [play the note; console says `Note: I↑→ (base × 9/8 × 3 = 891), scale=JI, base=264`], you can see its frequency of 891 as being base times 9/8 times 3. You might think its color should indicate a sixth, but its adjusted ratio of 27/16 is just a little too far away from 5/3 to get the major sixth color. You can find that color down here with `E↓`. [Play `E↓`.] Let's play the notes together. [Play `E↓` and `I↑→` together.] Hear that? They're not the same pitch. Welcome to the world of just intonation.

It may seem strange at first that the information in the web UI differs from the console output. Here's why. The purpose of the web UI is to help you find notes and give you an idea of what you're going to hear. The purpose of the console output is to deconstruct the pitch and how it was derived as well as to give you what you need to drop notes into a Syntoniq language score file. We can also cram a lot more information in the console output than on the web view.

I'll demonstrate that shift and transpose work as expected. Let's start with shift. If I start on `A` and go to the right, you can hear that I'm going up four scale degrees to the fifth [Play notes start at A and going to the end fo the row], then starting over with `A→` but that I run out of notes at `D→`. If you think `C→` should be to the right, you're correct, and we can reveal it by shifting. The last video explained shift and transpose. Here I'm just going to press shift and slide everything over to the left. [Shift to the left one column, revealing `C→`, then play it] And there it is. Let me put that back. [Shift to the right one column.]

Let's look at transpose. Here's a perfect C major triad. [Play A, E, C.] What if I want a D major triad? Well, I don't have one, at least not in that octave. But if I take the pitch of D, which is represented by the letter `I` in this system, and move it over to the tonic like this [transpose, I, A], now those same notes play the D major triad. Take a look at the console output.

[Console shows]
```
Note: A (base × 1 = 297), scale=JI, base=297 (transposition: 264 × 9/8)
Note: E (base × 5/4 = 371.25), scale=JI, base=297 (transposition: 264 × 9/8)
Note: C (base × 3/2 = 445.5), scale=JI, base=297 (transposition: 264 × 9/8)
```

Looking at the note `C`, we see the frequency of 445.5 Hertz as base times three halves, and we see that the base frequency, 297, is 264 times nine eighths. All the information is there. To see everything, let's play `E↑→`. This time we see this [play it: `Note: E↑→ (base × 5/4 × 3 = 1113.75), scale=JI, base=297 (transposition: 264 × 9/8)`]. We have base times 5/4, which is what `E` is times 3, our tile factor for a frequency of 1113.75 Hertz. We also see that the base frequency of 297 is the original base, 264, times 9/8, our transposition factor. It's a lot of information, but it's all there, and you can deconstruct it. We haven't lost all the history by just looking at the number 1113.75.

Now let's take a look at two more layouts. First, I'll show you a layout that uses two mappings. This one, JI-19-EDO [select the layout] has the same JI mapping on the bottom four rows [indicate] and the 19-EDO mapping on the top. We can go crazy here and transpose JI up one step of 19-EDO. This works because shift and transpose apply to the *mapping*, not to the layout. The exception to this is that the octave shift keys apply to the whole layout. In this layout, we have our perfect triad on the bottom row. [Play it.] Notice when I play it that a bunch of other yellow keys light up. That's because the `c` is repeated several times and has exactly the same pitch across these two completely different mappings. [Play it again.] Also notice that no other colors light up on top. That's because the third and fifth of the pure just intonation triad don't appear in 19-EDO or any other scale that equally divides the octave. Let's transpose the just intonation mapping up one 19-EDO step. We can do this by picking the cyan note near any of the 19-EDO `c` notes we just saw light up. Let's pick this one. [Point to the one near the middle of the keyboard.] Here's the original chord again. [Play the original triad.] Now I'll hit transpose, the cyan `c#` from the top mapping, and the `A` key on the bottom mapping. [Do that.] Now if I play the triad again [do it], it's been transposed up a 19-EDO step. Also notice that the cyan notes light up on the top side since it's now that note that shares a pitch with the tonic of scale. [Do it again and point it out.] I'll also mention that the 19-EDO note is still cyan because that is still the single step note in that mapping, and the tonic in the just intonation section is still yellow because that's still the tonic in *that* section. I know...it's confusing. You might have to watch this a few times or play with it yourself. But honestly, can you think of any other system that lets you do this? The complexity is inherent. This tool makes it possible.

Back to our chord. What does the console say?

[Console output]
```
Note: A (base × 1 = 220*^23|76), scale=JI, base=220*^23|76 (transposition: 220*^1|4 × ^1|19)
Note: E (base × 5/4 = 275*^23|76), scale=JI, base=220*^23|76 (transposition: 220*^1|4 × ^1|19)
Note: C (base × 3/2 = 330*^23|76), scale=JI, base=220*^23|76 (transposition: 220*^1|4 × ^1|19)
```
I'm not going to read this all to you, but you can see, for example [say three thirty times two to the 23 over 76] `330*^23|76` appears as the frequency of the note `C`, and you can see it all deconstructed through the transposition. This is a very powerful feature that allows you to create pitches that would be quite hard to achieve in most systems, and you can do it all without thinking about cents or calculating frequencies.

Let's close with a completely custom layout using a custom scale. This layout only maps part of the keyboard. It uses a scale with notes called `h1` through `h64` that map to the first 64 steps of the harmonic sequence. I'll select it now. [Select layout]. Here, you can see on the web UI that only some of the keys are mapped, and on the keyboard, only some of the keys are lit. On the Launchpad, this fully covers the 64-note grid. Let's play a few notes. [Start at the beginning and play the first two rows.] There's the harmonic sequence. The ratios are all normalized on the web UI as usual. What does the console say?

[Console:]
```
Note: h1 (base × 1 = 50), scale=harmonics, base=50
Note: h2 (base × 2 = 100), scale=harmonics, base=50
Note: h3 (base × 3 = 150), scale=harmonics, base=50
Note: h4 (base × 4 = 200), scale=harmonics, base=50
Note: h5 (base × 5 = 250), scale=harmonics, base=50
Note: h6 (base × 6 = 300), scale=harmonics, base=50
Note: h7 (base × 7 = 350), scale=harmonics, base=50
Note: h8 (base × 8 = 400), scale=harmonics, base=50
Note: h9 (base × 9 = 450), scale=harmonics, base=50
Note: h10 (base × 10 = 500), scale=harmonics, base=50
Note: h11 (base × 11 = 550), scale=harmonics, base=50
Note: h12 (base × 12 = 600), scale=harmonics, base=50
Note: h13 (base × 13 = 650), scale=harmonics, base=50
Note: h14 (base × 14 = 700), scale=harmonics, base=50
Note: h15 (base × 15 = 750), scale=harmonics, base=50
Note: h16 (base × 16 = 800), scale=harmonics, base=50
```
It shows us those frequencies increasing arithmetically by 50 Hertz each step, exactly what you'd expect for the harmonic sequence. I'll play the top few notes so you see how close together they are. [Play the top row, top to bottom.] If you don't hear anything, perhaps your speakers aren't good enough, or maybe the video compression is ruining the audio. But the notes are there, and if you try it yourself, you should be able to hear them.

Now you've seen what there is to see. The manual describes how to create your own layouts. That material is technically dense, and there is no accompanying video, but there are also no UI new features. You've seen everything the Syntoniq keyboard application can do.
