+++
title = "SYNTONIQ LANGUAGE"
weight = 50
sort_by = "weight"
+++

Gemini suggestions

# PART 2: CONCEPTS AND WORKFLOW

* **Lossless Pitch & Generated Scales**
    * The "Why" (Semantic meaning vs. Frequency)
    * The "How" (Math of generated scales, Intervals)
* **The Score File**
    * File structure
    * Tracks, Parts, and Voices
    * Directives and metadata
* **Transposition & Pivoting**
    * Logic of moving pitch centers
    * Relative vs. Absolute transposition

# PART 4: REFERENCE

* **Command Line Usage**
    * `syntoniq` arguments and flags
* **Language Reference**
    * How to read the reference
    * Directives (likely generated)
    * Marks and Decorations
    * Syntax summary
* **Error Messages**
    * Common errors
    * How to read and fix them

----------------------------------------------------------------------

I don't want these split this way, but make sure all the above is there in a sensible order so the flow between reference and tutorial is clear.

My notes

TODO:
 * Command line
 * File format

Built-in scales: 12-EDO (default), 19-EDO, 31-EDO, JI (generated)

* Summary of Syntoniq Language Syntax
  * Directives, including accessing built-in help
  * Notes
  * Dynamics
  * Data blocks: layout and scale definition; refer to keyboard for layouts
* Examples: show examples and give instructions on how to play them
* Understanding and fixing syntoniq error messages
* Lossless Pitch Notation
* Defining and Using Scales
* Defining and Using Generated Scales
  * See outdated scratch docs and help for define_generated_scale.
* Transposition
* Marks and Repeated Sections
* Language Reference


The Syntoniq Language, represents a musical score as a temporal sequence of notes and dynamics using user-defined scales and tuning. The output of the `syntoniq` command is either a Csound file or a MIDI file. You can create a MIDI file that includes embedded MTS SysEx codes or that uses MPE. The MTS MIDI can be interpreted by Timidity++. The MPE MIDI file is suitable for importing into a DAW (Digital Audio Workstation) for further editing. The Csound file can be played directly. When syntoniq creates a Csound file, it inserts the notes into a template containing instruments and any other logic. Its output is also a template, so you can continue to rerun `syntoniq` on its previous output to update the notes.

Note that `syntoniq` does not generate a printable score.

csound generator: including templates and using output as template
playback for MTS and MPE
