# PRE-RELEASE

December 2025

*Warning:* This software has not been released and is still undergoing active development. Every aspect of it is still subject to change. The documentation is likely to be outdated. If you see this and want to play, we probably need to be in direct communication. It is my intention to get this ready for a production release, which will include detailed documentation and demo videos. The documentation below is more of a reminder to my future self and is not complete enough for you to actually use the software.

# Syntoniq

![Syntoniq](./logo/syntoniq-logo.svg)

Syntoniq is a system for creating microtonal music. It can also be used for regular 12-tone equal temperament. Syntoniq has two components: a DSL (Domain-specific Language) for representing music and a keyboard application.

## Major Features

* Define arbitrary scales using flexible note names and lossless pitch representation. You can specify a pitch as a product of rational numbers and rational numbers raised to rational powers. This makes it possible to play with Just Intonation, equal divisions of any interval, or to combine them. Syntoniq does not support Scala or other tuning files. The intention is to create scales that include semantically meaningful pitch descriptions.
* Generalized transposition. You can define a scale and then create a *tuning* with the scale by specifying a base pitch. You can specify an absolute base pitch, or you can transpose by multiplying a relative pitch factor with the base pitch or by assigning the pitch of one note to another note. This makes it possible to pivot from one tuning to another around a pivot note and to reverse any transposition. Flexible transposition and scale creation are available in the keyboard and DSL.
* Flexible layout engine. The keyboard allows you to create isomorphic layouts, where you specify the number of scale steps in each of two directions, or manual layouts, where you assign notes explicitly to grid locations. A layout can include multiple mappings and can combine manual and isomorphic mappings. Layouts can be "shifted" as well as transposed, meaning you can effectively slide the keys over. Shifting works with isomorphic mappings, allowing you to extend beyond what fits on the keys. It also works with manual mappings, where the entire mapped region is "tiled" horizontally and vertically with optional pitch shifting. Being able to create complex and combined layouts with shift and transpose allows you do things like create Just Intonation tunings and transpose them to different keys.

## Syntoniq Language

The Syntoniq Language, which we refer to the *Syntoniq DSL* (DSL = Domain-Specific Language), represents a musical score as a temporal sequence of notes and dynamics using user-defined scales and tuning. The output of the `syntoniq` command is either a CSound file or a MIDI file. You can create a MIDI file that includes embedded MTS SysEx codes or that uses MPE. The MTS MIDI can be interpreted by Timidity++. The MPE MIDI file is suitable for importing into a DAW (Digital Audio Workstation) for further editing. The CSound file can be played directly. When syntoniq creates a CSound file, it inserts the notes into a template containing instruments and any other logic. Its output is also a template, so you can continue to rerun `syntoniq` on its previous output to update the notes.

Note that `syntoniq` does not generate a printable score.

## Syntoniq Keyboard

The Syntoniq Keyboard (`syntoniq-kbd`) creates a *study keyboard* using either a [Novation Launchpad MK3 Pro](https://novationmusic.com/products/launchpad-pro-mk3) or a [HexBoard MIDI Controller](https://shapingthesilence.com/tech/hexboard-midi-controller/).

While you could use `syntoniq-kbd` for performance, it is mainly intended for study. It has certain features, such as dynamic transposition and layout shift, as well as note sustain mode, that are more useful to non-real-time work, such as transcribing, composing or arranging, and it lacks features to vary sound quality, though it can be used as a MIDI input device.

The keyboard also has a read-only web UI that shows you an annotated version of the keyboard. You can reach this on `http://localhost:8440`. (Get it? 440?) It also generates output on the terminal giving additional information and logging what notes are pressed.

# Documentation Notes

These are notes on the keyboard UI so I remember to cover these when I write the real docs.

* Reset: turns off all notes, draws the logo, and reloads the configuration file. The configuration is a syntoniq score file containing layout directives.
* Layout selection: a button is assigned to each layout. When the button is pressed, the layout is loaded.
* Color scheme: the color scheme is based on the interval from the tonic pitch of the scale. Notes close to a minor third/major sixth are one color, major third/minor sixth are a different color, and fourths/fifths are a third color. Each of those intervals has an "on" and an "off" color. The tonic has its own colors. For isomorphic layouts, a special color is assigned to the note that is one scale degree away from the tonic. All other notes use the same color. At initial release, the color scheme is not configurable.
* Octave shift: scales can be configured with a "cycle" size that's other than an octave, but on the keyboard, the octave shift keys always move the pitch up and down by an octave, and they effect the entire layout. This is as opposed to shift and transpose, which effect only a single mapping within the layout. Why do the keys always work in octaves? It's possible for a layout to have more than one mapping with different cycle sizes. It's possible for a scale with a non-octave cycle size to not include an octave pitch, but you can always use transpose to transpose by a cycle.
* Shift: touch two notes to move the first note to the second note's position. Both notes must belong to the same mapping in the same layout.
* Transpose: touch two notes to assign the first note's pitch to the second note. The two notes may be in different layouts, so you can touch one note, switch to a completely different layout, and assign the note's pitch there.
* Both shift and transpose are triggered by pressing a specific key. You can either press and release the key, then press and release the two note keys, or you can press and hold shift or transpose while pressing the other note keys.
* Sustain: In sustain mode, pressing a note turns it on or off. This is very useful for constructing complex chords, transcribing music, or playing chords with notes from different layouts. In sustain mode, the keyboard's terminal output displays the complete collection of notes played, including any transposition currently in effect.
* Launchpad only: the "record midi" button also prints all current notes.

# Maybe Someday

* Tuned MIDI output (MTS) mode: we could generate a tuning file (Scala or Tun) based on Syntoniq scale.
* Score printing: we could potentially generate MusicXML or LilyPond if we included additional metadata.

# Remember

* Syntoniq creates various virtual MIDI ports. On Windows, use [loopMIDI](https://www.tobias-erichsen.de/software/loopmidi.html) and create a port called `syntoniq-loop`.
* On Linux, you can watch Syntoniq's MIDI output with `aseqdump`, e.g.:
  ```sh
  aconnect -l
  aseqdump -p 128:0
  ```
