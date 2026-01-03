+++
title = "Manual Layouts"
weight = 50
sort_by = "weight"
+++

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
