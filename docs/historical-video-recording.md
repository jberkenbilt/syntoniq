# Video Recording Notes

The manual includes verbatim video scripts. The notes below outline what I did to create the screencast video. After completing these steps, there was quite a lot of editing work. This is kept for historical reference or in case I ever want to re-record parts of the video. This was made by reading through the scripts and notating things I would need to do in the correct sequence. If doing it again, completely removing my head from view between sections would have helped with some of the editing. It's also easier to freeze than to omit sections, so avoid holding a static position for too long.

## Initialization

Set up about:blank on the browser and a clear terminal. Focus video on keyboards in their factory default settings. Touch the launchpad to clear the screensaver before starting the video. Make sure dark reader is disabled.

Start screen recorder. Pause a few seconds between each step and the next.

Run `syntoniq-kbd run --port=potato`

Draw attention to the list of devices

Touch launchpad so factory tiles are visible.

Start video recording.

Run `syntoniq-kbd run --port=MK3`

Click on link to open MK3 in the browser. Adjust zoom.

Draw attention to first the selected layout, then the list of layouts, then the numbered buttons.

Press the scroll layouts button.

Show on the browser that 9 is visible in the layouts area.

Press the scroll layouts button again.

Press layout button 1.

Play C, D, E, F, G, then C-E-G.

Hit Clear.

Terminal, hit CTRL-C

Type `syntoniq-kbd run --port=RP`

Refresh browser, adjust zoom.

Draw attention to list of command keys.

Press layout selection command key

On web, show the labeled buttons.

Select the first layout.

Play C, D, E, F, G, C-E-G.

Hit CTRL-C

Run `syntoniq-kbd run --port=RP --midi`

Start Surge XT. Set it to always on top and bring it over the browser.

Go to audio settings -- also needs to be always on top. Show correct devices. Show patch.

Hide Surge.

Play C, D, E, F, G, then C, G, E C D' E' G' or similar.

Hit CTRL-C

End video and screen capture.

## Notes and Chords

Clear terminal

Unblank launchpad

Start screen recording and video

Run `syntoniq-kbd run --port=RP`

Select first layout

Play `c` and hold for a few seconds. Do this two or three times.

Play `c` and `e%` together.

Play `c` and `a,` together.

Play `c` and `e`.

Play `c` and `a%,`.

Play `c`, then `f`, then `g`.

Play `c`, then `c#`.

Play `c`, then `d`.

Play `c`, then `e`, then `g`.

Draw attention to the console output.

Press octave down, then `c`.

Draw attention to the console output.

Draw attention to the transposition statement

Press and hold `c`. Press octave up. Press octave up. Press octave down.

Touch sustain.

Show 3 = sustain on the web display.

Touch `c` to turn on

Touch `c` to turn off

Touch `c`, then `e`, then `g`.

Touch sustain twice to clear.

Touch `c` and `g`.

Turn off sustain.

Touch `e`, then `e%`. Repeat.

Turn sustain on.

Touch `e`.

Turn sustain off.

Touch `b` and `d'`.

Hit sustain 3 times to clear and leave on.

Hold `c`. Press octave down, up, up, down. Release `c`.

Sustain twice to clear.

Draw attention to hexboard-60 in layout 2.

Play `c`, `e`, `g`

Select layout 2.

Clear sustain and turn off.

Play some Cs going up and down the octaves.

Play all the notes in the diagonal containing `c`.

CTRL-C, CTRL-L, run `syntoniq-kbd run --port=MK3`

Refresh the web.

Select layout 1

Play C, E, G

Touch Chord

Play C, E, G.

Touch Chord three times to clear and turn off.

Press C, E, G together in regular mode. While holding, press Show Notes.

Reset

End video and recording

## Shift and Transpose

Start the video and screen recording

Run syntoniq-kbd run --port=RP

Load the first layout

Draw attention to its name as 12-EDO-h2v5.

Play `c`, pause, Play `d`

Play `c`, pause, Play `f`

Play `c`, pause, `f`, `b%`, pause briefly, play `c'`

Play `c`, pause, play `f`, pause, play `e%`, `c#`

Play `c#` two or three times

Play `c`, `d`, `e`, `f#`, `a%`, `b%`, then down to `f`, then hover over the edge.

Touch the spot where the `c` would be

Point to the purple `a%,` key two columns to the left.

Show the shift assignment in the web UI

(modal shift)

Touch shift.

Touch the `f`, point to the shift key, touch two spaces to the left of the `f`.

Play the new `c` twice.

(modifier shift)

Press and hold shift.

Touch the new c, wiggle finger on shift, touch one column to the right, release shift

(cancel modal)

Touch shift

Touch the `c`

Touch shift.

(cancel modifier)

Press and hold shift.

Touch c.

Release shift.

Reset

Show 19-EDO-h3v2 on the web UI

Select 19-EDO-h3v2.

Play c

Play e

Play g

Play the triad.

Touch the c# to the right and below.

Go back and forth between c and c# a few times.

Pause

Play c, then c#

Touch transpose

Touch c#

Point to transpose

Pause

Select first layout

Press c

Point to transpose

(after transpose)

Press c

Show console output.

Play d

Show console output.

Show `Note: d`

Show `^1|6`

Show  `220*^107|228`

Show the scale and transpose part.

CTRL-C

Unblank the launchpad

Run `syntoniq-kbd run --port=MK3`

Select 12-EDO

Play `c`

Play `f`

Play `c#`

Point to shift on the web and physical keyboard.

Press shift, then `c`, then the note to the right.

Hold shift, touch `c`, then the note to the left.

Select 19-EDO.

Play `c` and `c#`

Point to Note on the web and keyboard.

Press Note.

Press `c#`. Point to Note.

Select the first layout.

Touch `c`.

Point to the console.

## Manual Mapping

Start `syntoniq-kbd run --port=RP`

Show JI on the web

Select JI layout

Point out EK and Bh on the web.

Point out arrows.

Play A.

Play C.

Play E.

Play A-C-E.

Play EK.

Play Bh.

Play A, E, C, Bh, I→ EK→.

Pause.

Play A.

Play A→.

Show on the web UI that they are both 1.

Play A↑.

Show 3/2 on the web UI.

Play A↑→. Show it as 3/2.

Draw attention to `Note: A↑→ (base × 1 × 3 = 792), scale=JI, base=264` on the console.

Show 792.

Show times 1, then times 3. Pause.

Play I↑→

Show console: `Note: I↑→ (base × 9/8 × 3 = 891), scale=JI, base=264`

Show 891, then 9/8, then times 3.

Show 27/16 on the web UI.

Play E↓

Play E↓ and I↑→.

Play E↓ and I↑→ again, pointing out other lit up notes.  *** add

Play A, I, E, D, C, pause, then A→, etc to D→. Pause.

Shift one to the left.

Play C→.

Shift to the right.

Play A, E, C.

Transpose I to A.

Play A, E, C.

Draw attention to console.

[Console shows]
```
Note: A (base × 1 = 297), scale=JI, base=297 (transposition: 264 × 9/8)
Note: E (base × 5/4 = 371.25), scale=JI, base=297 (transposition: 264 × 9/8)
Note: C (base × 3/2 = 445.5), scale=JI, base=297 (transposition: 264 × 9/8)
```

On C line, show 445.5, 3/2, 297, 264, and 9/8.

Play E↑→: `Note: E↑→ (base × 5/4 × 3 = 1113.75), scale=JI, base=297 (transposition: 264 × 9/8)`

Show 5/4, 3, 1113.75, 297, 264, 9/8.

Show JI-19-EDO in the web and select the layout

Indicate the bottom for rows, then the top rows.

Play A, E, C on the bottom row.

Play A, E, C again, drawing attention to the yellow on top.

Point to the cyan note near the middle of the keyboard.

Play A, E, C.

Play c#.

Hit Transpose, then c#, then A.

Play the triad again.

Play it one more time, drawing attention to the cyan on top.

[Console output]
```
Note: A (base × 1 = 220*^23|76), scale=JI, base=220*^23|76 (transposition: 220*^1|4 × ^1|19)
Note: E (base × 5/4 = 275*^23|76), scale=JI, base=220*^23|76 (transposition: 220*^1|4 × ^1|19)
Note: C (base × 3/2 = 330*^23|76), scale=JI, base=220*^23|76 (transposition: 220*^1|4 × ^1|19)
```

Draw attention to console output. Show `330*^23|76`

Show harmonics layout and select.

Show notes h1 through h64.

Play the first four notes.

Draw attention to the console output.

Play the remaining notes of the bottom two rows.

Point to the console again.

Play the top row, top to bottom.

Show the console.

Reset.

End video.
