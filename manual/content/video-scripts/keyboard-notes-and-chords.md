+++
title = "Keyboard Initialization"
weight = 10
sort_by = "weight"
+++

[This video accompanies [SYNTONIQ KEYBOARD -> Notes and Chords](../../keyboard/notes-and-chords/).]

[Present on the screen: a video of one of the controllers when in use, the web UI, and a terminal window.]

In this video, we'll go over playing notes and chords. We will describe how to interpret note colors, the web display, and console output. We will cover regular notes, the sustain feature, turning off all notes, using the octave shift keys, and a few features specific to each keyboard controller.

I'm going to use the HexBoard for most of this video. At the end, I'll show you how to do the same things on the Launchpad.

[Clear the terminal and start syntoniq-kbd. Refresh the web UI.]

Here you can see the HexBoard in its initial state, displaying the logo, and you can see the corresponding web display. The console indicates that the HexBoard is initialized, so we're ready to start.

I'll select the first layout, 12-EDO, and then I'll give you the guided tour. [Select the first layout.]

Take a look at the physical keyboard and at the web display. While the color matching is not perfect, you can basically see the same color layout on the physical keyboard and the web display. You may see slightly different colors depending on your monitor and on whether a future version of the keyboard application improves upon the color matching between the keyboard and the web UI, but it should be possible to follow along.

Before I dive in, I'll review a little terminology. The Syntoniq keyboard has two kinds of layouts: manual and isomorphic. The word "isomorphic" literally means "same shape." We will cover layouts in more depth later on. This video will focus only on isomorphic layouts. In an isomorphic layout, the interval between two keys is based solely on the relative physical orientations of the keys. This means you can play any chord or melodic passage in any key by simply adjusting your starting position. I'll show you how this works as I explain.

Let's talk about colors. I'm going to press this yellow key. [Press `c`]. As I press the key, notice that two things happen: the key lights up on the keyboard, and the corresponding hexagon lights up on the web display. Sorry that the yellows look pretty green on the web display for this video. That's something I hope to address.

On the Syntoniq keyboard, the color of a note tells you something about its position in the octave, or technically, I should say, "in the cycle" since we allow scales whose repeating cycle is other than an octave. For now, we'll stick with octave-based scales. We use the following color scheme:

* The tonic is yellow. [Play `c` again.] This is because yellow is easy to find on the keyboard.
* A minor third is red. [Play `c` and `e%` together.] Red is a harsher color, like the minor third.
* The reciprocal interval of a minor third is a major sixth since a major sixth is a minor third *below* the tonic. We use a similar color, orange, for the major sixth. [Play `c` and `a,` together.] Notice that the relative position of `c` and `a,` is the same as between `e%` and `c`. This is because this is an isomorphic keyboard. I'll explain that more in a minute.
* For the major third and its reciprocal interval, the minor sixth, we use pink and purple. These are gentler colors to go with the gentler intervals. [Play `c` and `e`, then `c` and `a%,`.] Notice again the symmetry in the key positions.
* The fourth and fifth are green and blue. [Play `c`, then add `f`, then add `g`.]
* One color that is not related to the interval is the single-step color, which is cyan. In any isomorphic layout, we use that color for one step above the tonic. In this case, it's a semitone. [Play `c`, then `c#`.] I'll come back to that more when we get to transposition and layout creation in another video.
* Everything else is gray.

Now that you've seen the colors, let's look at the web display and the console output. They convey slightly different information. The web display is all about helping you find the right note on the keyboard. Its job is to provide the information you need but that we can't display on the keyboard itself because of limitations in the hardware. The console output is about telling you exactly what notes you're playing and giving you information about their pitches. Its job is to give you what you need to know so you can use these notes in a Syntoniq language score.

Take a look at middle `c` again. [Play `c`.] On the web, you can see this has a two-row label. The top row is `c`. That's the note name. The second row is `1`. That's the relative pitch. The tonic of a scale always has a relative pitch of `1`. To the right, we have the note `d`. Its pitch is `^1|6`. That's syntoniq pitch notation for two raised to the power of one sixth, which is the same as two twelfths. This is because `d` is two chromatic steps above `c` in the 12-EDO scale, which is our regular Western scale consisting of 12 equal divisions of the octave. EDO stands for Equal Divisions of the Octave. You can see similar pitch notations for other notes. Let's go up a couple of rows. Here you see a note labeled `d'` [say "d apostrophe"], which has the same pitch marker. In this case the apostrophe indicates that the note is up one octave. As you look up and down the rows, you can see that and the `,` [say comma] indicating the number of octaves away, but the pitches remain the same. Relative pitches are normalized to be within a single scale cycle. This makes it easier to find the note with a given pitch.

Let's take a look at the console output. I'm going to play `c`, then `e`, then `g`. You can see a bunch of information printed about the note. Let's look at this line, the `e`. [Draw attention to it on the terminal.] We see the note name `e`, the relative pitch `^1|3`, the computed pitch as `base` multiplied by the relative pitch, the scale, and the base pitch, which is 220 times two to the power of one fourth. Since one fourth is three twelfths and 220 is half of 440, the frequency of A440, our base is the usual pitch for middle `c`. This would about 261 Hertz, but Syntoniq shows all pitches in this exact form. This is also what makes it possible for Syntoniq to compare two pitches and know that they are *exactly the same*, even if they were not written the same way.

Next, let's take a look at the octave shift keys. These are the fourth and fifth command keys. When I press these keys, there is no visual change to the web display, but the pitch changes. I'll press octave down one time and play `c` again. [Do that.] The pitch you hear is now an octave lower. Take a look at the console output. [Draw attention to the new `c` line.] Now the pitch shows as `110*^1|4`. This is half of the previous value. We can also see this indication [point to the transposition block] indicating that the base pitch of `110*^1|4` is the result multiplying our original base pitch, `220*^1|4`, by a transposition factor of one half.

Here are some other features of the octave shift key. If I press it while holding a note, the effect is as if I had released and re-pressed the note. Listen. [Press and hold `c`. Press octave up. Observe that the low note turns off and the high note turns on. Press octave up again, then octave down.] You can hear the different notes being played, and you can see the console output, which shows that we selected a new layout (it's the same layout, but this is how Syntoniq internally applies a transposition), and we can see the notes indicated with their transpositions: absent when 1 or displayed as 2 or one half.

Let's take a look at the sustain key now. This third command key [point] is currently red. When I touch it [touch it], it turns green, indicating that the sustain feature is on. Notice here on the web display where it lists the meanings of the command keys and tells us that 3 is toggle sustain.

In sustain mode, when you press a note, it toggles on or off. See, if I press and release `c` [press and release `c`], it continues to sound after I release. When I touch it again [do it] it turns off. I'll build a C major triad, one note at a time. [Play `c`, then `e`, then `g`.] I'm going to turn all the notes off now by touch sustain twice so we don't have to keep hearing the chord. [Do that.] If you turn sustain off and back on without touching any notes, it clears all notes. Notice on the console output [indicate] that it tells us, each time turn a note on or off, about all the notes currently playing. This is very useful for grabbing this information later to use in a Syntoniq language score.

Here's another fun thing you can do with sustain mode. If you turn it off while notes are sustained, other keys work normally. Let's say I want to build a triad. I can enter sustain and play `c` and `g` [Press `c` and `g` since we're already in sustain mode]. Now I can turn off sustain mode [do that], and the notes continue to play. Now I can press `e` or `e%` [go back and forth a few times] to build chords. If I want to add one of the notes, I can turn sustain back on [turn it on and add `e`], turn it off and add more notes [turn it off, play `b` and `d'`, then hit sustain three times to clear]. Sustain even interacts as you would expect with the octave shift keys: it continues to act like you released and pressed again. Watch this. [Hold `c`, hit down, then up, then up, then down, then release. Wait a second, then touch twice to clear.] We had all three octaves playing. You can also see this on the "Current notes" output on the console: we see all three notes with their respective transpositions.

I'll show you one more feature specific to the HexBoard, then we'll switch to the Launchpad. Note on my list of layouts [indicate the web display] that many of the layouts are labeled with `hexboard-60`. I'll pick layout 2, which is 12-EDO with hexboard-60. Before I do, let's sustain a C major triad. [Do that.] Now I'll select layout 2. [Do that.] Take a look. The lights are on in the new layout because Syntoniq knows what the pitches are. Let's turn those off. [Hit sustain three times to turn off and exit sustain.] In the 60-degree layout, our rows and columns are rotated 60 degrees counterclockwise. You can see the octaves are at an angle now. [Play some Cs going up and down the octaves.] This layout gives a wider center rows. [Play all the notes in the diagonal containing `c`.] Sometimes this makes it possible to reach different notes or chords, especially in scales with lots of notes per octave or cycle.

There's a lot more we can do with shift and transpose, which give you ways of reaching notes that are not current displayed. We'll come to that in the next video.

For now, you've seen all the features. Let's briefly switch to the Launchpad so you can see what it looks like there.

[Exit out, clear, restart the keyboard with the Launchpad device. Refresh the web UI.]

I'm going to select the first layout. [Do that.] You can see that the layout is a little different just because we're on a rectangular grid instead of a hexagonal one, but the colors and labels are the same. Everything works exactly the same.

We can play regular notes the same way [just play a C major triad]. The sustain button is up here where it says `Chord` on the physical keyboard [point] and `Sustain` on the web display. As before, it's red until I touch it, then it turns green. [Touch it.] You can see we are in sustain mode with exactly the same behavior. [Touch `c`, `e`, `g`, then touch twice to cancel and once more to turn it off.]

There's one extra thing we can do with the Launchpad, since it has so many more keys. This button, labeled "Capture MIDI" on the physical keyboard and "Show Notes" on the web display will print all the notes just like sustain mode, but we can do it any time. I'll play a chord in regular mode and then press the button. Notice that we get our list of notes in the console.

That about wraps it up. Come back for the next video where we'll look at shift and transpose and get our first taste of some microtonal scales on the Syntoniq keyboard.
