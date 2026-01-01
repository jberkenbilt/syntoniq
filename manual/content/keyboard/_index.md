+++
title = "SYNTONIQ KEYBOARD"
weight = 50
sort_by = "weight"
+++

Video format:
* Keyboard on left
* Web UI and text output on right

* Intro
  * Call out independence of keyboard and language except for use of language for defining keyboard layouts
  * Position this as more of a study keyboard than performance keyboard, but note that can it can be used for performance and provides a way to use the HexBoard or Launchpad with scales you design yourself
  * Mention intent to add an interactive CLI-based chord builder that shares syntonq-kbd's output generation but doesn't require a hardware device.

* [Hardware](hardware.md)
  * Supported devices:
    * [Novation Launchpad MK3 Pro](https://novationmusic.com/products/launchpad-pro-mk3)
    * [HexBoard MIDI Controller](https://shapingthesilence.com/tech/hexboard-midi-controller/)
  * Have custom HexBoard firmware as download and provide instructions, noting that the HexBoard team plans to include the changes in the next firmware release. As of HexBoard firmware version 1.2, custom firmware is required. It is expected that the firmware mods will be incorporated into the main firmware. In the meantime, the changes are at <https://github.com/jberkenbilt/HexBoard/tree/delegated-control>; check status at <https://github.com/shapingthesilence/HexBoard/pull/6>.
  * Show csound and MIDI output
  * Mention MIDI system Requirements (loopmidi/syntoniq-loop on Windows, a synth, etc.)
  * Show how to use Syntoniq as a MIDI input device and how this may require disabling input from the hardware to avoid hearing two notes (select input from the syntoniq device, not the hardware device)
    * Syntoniq creates various virtual MIDI ports. On Windows, use [loopMIDI](https://www.tobias-erichsen.de/software/loopmidi.html) and create a port called `syntoniq-loop`.
    * On Linux, you can watch Syntoniq's MIDI output with `aseqdump`, e.g.:
      ```sh
      aconnect -l
      aseqdump -p 128:0
      ```
* [Initialization](initialization.md)
  * Use demo config; show logos
  * Mention reset
  * Exit with ctrl-c
  * Connect to web UI (http://localhost:8440); point out "440"
* [Notes and Chords](notes_and_chords.md)
  * Select 12-EDO layout and at least one other, sticking to isomorphic for now
  * Demonstrate reset
  * On the hexboard, show a 60 degree layout
  * Throughout, highlight information in console output
  * Explain color scheme with interval-based colors and note that it is not configurable at initial release
    * With the scheme, EDO-{12,19,31} will show all the intervals. 17-EDO will only show fourths and fifths. You will be able to tell at a glance which intervals are good in a given tuning system.
  * Play regular notes and chords
  * Explain that all keys with the exact same pitch (not pitch class) light up even across layouts (made possible by canonical pitch representation)
  * Enable sustain
  * Build a partial chord
  * Launchpad specific: demonstrate "print notes" button
  * Disable sustain
  * Play individual notes
  * Switch layouts; have at least one common note
  * Add new notes
  * Clear all notes with double sustain
* [Shift and Transpose Intro](shift_transpose.md)
  * Draw attention to the similarity in UI: both do some form of "move the first thing to the second thing"
  * Use shift to shift the layout (moves the first note to the second note's position)
  * Show shift as modifier and also as modal; also cancel in-flight
  * Do the same with transpose (assigns the first note's pitch to the second note; point out direct mapping to the syntoniq language's `transpose` directive)
  * Show octave transpose
  * Use octave transpose while holding notes in sustain and not
  * Do transpose across layouts
* [Manual Layouts](manual_layouts.md)
  * Introduce a manual layout and explain how to interpret the UI
    * Explain use of octave marks, tiling arrows, and meaning of base-relative pitches and colors
    * Show difference between information as given on the web UI and what's printed to the console with rationale (on web UI, it's about finding the note; on console, it's about getting what you need to replicate the note in a score)
      * Text written earlier (note to self: check for accuracy): Make sure to fully explain the semantics of the octave marks and arrows on the keyboard. Octave marks show the pitch offset in cycles relative to the note in the original scale and are used with isomorphic mappings. In manual mappings, notes are always shown as they appear in the mapping. Arrows indicate the tiling offsets. The displayed pitch is within cycle and relative to the base pitch of the original, *non-tiled* note. So a note that's naturally up 3/2 from the base and additionally up another 3/2 from tiling will show 9/8, not 9/4.
    * Contrast use of octave markings with isomorphic; in isomorphic they match the notes because the keyboard can compute; in manual, they show only what's in the layout since it's user-selected
  * Demonstrate tiling with shift
  * Demonstrate transposition and reversible transposition
  * Show a layout with multiple mappings
    * Shift only works when you select two notes in the same mapping, so a mapping must be at least two rows high to shift vertically or two columns wide to shift horizontally
    * Transpose works across mappings just as it works across layouts as will be shown later
    * Octave transpose applies consistently to all mappings and transposes by an actual octave regardless of the scale's configured cycle. Its purpose is more about getting notes into a comfortable audible range.
  * Show independence of shift and transpose by mapping
  * Demonstrate tiling from shifting manual layouts
  * Demonstrate use of "transposition bar" with a high EDO and/or JI scale
  * Launchpad-specific: show minor third difference tones layout
* [Layout Engine](layout_engine.md)
  * This section probably doesn't get an accompanying video as it is very technically dense and more coding oriented.
  * Go over syntoniq language features for creating custom isomorphic and manual mappings and combining mappings to create layouts, including row and column layout for rectangular and hexagonal grids (with rationale for hexagonal)
  * Mention that, while the software doesn't prevent you from using isomorphic mappings with uneven tunings, it might create a confusing situation, but there are use cases if you "know what you're doing", such as dealing with regular but uneven tunings or intentionally experimenting with out-of-tune keys in JI
