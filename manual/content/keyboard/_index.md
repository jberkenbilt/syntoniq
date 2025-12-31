+++
title = "SYNTONIQ KEYBOARD"
weight = 50
sort_by = "weight"
+++

Include several very short/low-resolution videos that can be checked and included in the docs.

Video format:
* Keyboard on left
* Web UI and text output on right

Videos to record for each keyboard:
* Startup, logo
* Select 12-EDO layout
* Play notes and chords
* Enable sustain
* Play sustained notes
* Clear all notes
* Use shift, modal and modifier including cancel
* Use transpose, modal and modifier
* Use octave transpose
* Use octave transpose while holding notes in sustain and not
* Change layouts
* Reset
* Use transpose across layouts
* Use sustain across layouts
* Show manual layout with tiling
* Show two layouts together with "transposition bar"
* Launchpad
  * Show minor thirds
  * Print notes
* Hexboard
  * Show 60 degree

This is all various notes cut and pasted from other things. See also docs/scratch/keyboard-design.md

Make sure to call out that this is optional and distinct from the language.

* Hardware Requirements
* System Requirements (loopmidi on Windows, a synth, etc.)
* Keyboard web view
* Keyboard "UI" -- how to transpose, sustain, etc. -- covers all keyboard features
* Standard output of keyboard and using it to assist with Syntoniq score creation
* Using MIDI vs. CSound for playback including syntoniq-loop for Windows

TODO:
* Command line
* UI features
  * double sustain
  * sustain
  * transpose, shift as modifier and not
  * check engine for full behavior
* color scheme

Note somewhere that we show the normalized base interval on the web UI but the full base factor on print. The rationale is that, when looking for notes to play on the keyboard, it's easier to find the normalized interval, and it also keeps the second line of note labels manageable. On stdout, we want the full information. In both cases, the number of cycles is encoded in octave markers. This is potentially confusing, but it feels like the right call.

Make sure to fully explain the semantics of the octave marks and arrows on the keyboard. Octave marks show the pitch offset in cycles relative to the note in the original scale and are used with isomorphic mappings. In manual mappings, notes are always shown as they appear in the mapping. Arrows indicate the tiling offsets. The displayed pitch is within cycle and relative to the base pitch of the original, *non-tiled* note. So a note that's naturally up 3/2 from the base and additionally up another 3/2 from tiling will show 9/8, not 9/4.

Layout engine

The Syntoniq Keyboard (`syntoniq-kbd`) creates a *study keyboard* using either a [Novation Launchpad MK3 Pro](https://novationmusic.com/products/launchpad-pro-mk3) or a [HexBoard MIDI Controller](https://shapingthesilence.com/tech/hexboard-midi-controller/).

While you could use `syntoniq-kbd` for performance, it is mainly intended for study. It has certain features, such as dynamic transposition and layout shift, as well as note sustain mode, that are more useful to non-real-time work, such as transcribing, composing or arranging, and it lacks features to vary sound quality, though it can be used as a MIDI input device.

The keyboard also has a read-only web UI that shows you an annotated version of the keyboard. You can reach this on `http://localhost:8440`. (Get it? 440?) It also generates output on the terminal giving additional information and logging what notes are pressed.

# Documentation Notes

These are notes on the keyboard UI so I remember to cover these when I write the real docs. These have not been verified (2025-12).

* Reset: turns off all notes, draws the logo, and reloads the configuration file. The configuration is a syntoniq score file containing layout directives.
* Layout selection: a button is assigned to each layout. When the button is pressed, the layout is loaded.
* Color scheme: the color scheme is based on the interval from the tonic pitch of the scale. Notes close to a minor third/major sixth are one color, major third/minor sixth are a different color, and fourths/fifths are a third color. Each of those intervals has an "on" and an "off" color. The tonic has its own colors. For isomorphic layouts, a special color is assigned to the note that is one scale degree away from the tonic. All other notes use the same color. At initial release, the color scheme is not configurable.
* Octave shift: scales can be configured with a "cycle" size that's other than an octave, but on the keyboard, the octave shift keys always move the pitch up and down by an octave, and they effect the entire layout. This is as opposed to shift and transpose, which effect only a single mapping within the layout. Why do the keys always work in octaves? It's possible for a layout to have more than one mapping with different cycle sizes. It's possible for a scale with a non-octave cycle size to not include an octave pitch, but you can always use transpose to transpose by a cycle.
* Shift: touch two notes to move the first note to the second note's position. Both notes must belong to the same mapping in the same layout.
* Transpose: touch two notes to assign the first note's pitch to the second note. The two notes may be in different layouts, so you can touch one note, switch to a completely different layout, and assign the note's pitch there.
* Both shift and transpose are triggered by pressing a specific key. You can either press and release the key, then press and release the two note keys, or you can press and hold shift or transpose while pressing the other note keys.
* Sustain: In sustain mode, pressing a note turns it on or off. This is very useful for constructing complex chords, transcribing music, or playing chords with notes from different layouts. In sustain mode, the keyboard's terminal output displays the complete collection of notes played, including any transposition currently in effect.
* Launchpad only: the "record midi" button also prints all current notes.

# Remember

* Syntoniq creates various virtual MIDI ports. On Windows, use [loopMIDI](https://www.tobias-erichsen.de/software/loopmidi.html) and create a port called `syntoniq-loop`.
* On Linux, you can watch Syntoniq's MIDI output with `aseqdump`, e.g.:
  ```sh
  aconnect -l
  aseqdump -p 128:0
  ```

* hexboard details including firmware (unless/until accepted)
