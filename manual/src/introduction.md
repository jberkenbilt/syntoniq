# Syntoniq

<img src="assets/syntoniq-logo.svg" alt="Syntoniq Logo" style="height: 10em; vertical-align: middle;">

{{#include hexboard.html}}

{{#include hexboard2.html}}

{{#include launchpad.html}}

{{#include launchpad2.html}}

TODO:
* Include LOGO using an img tag. Will need build logic to populate the src/assets directory.
* To get a keyboard HTML file, get the keyboard in the right state, then run `curl http://localhost:8440/board` and save to a file. Make sure this is in manual/README.md along with populating assets.
* Feature Summary; mention videos with internal link
* Build and installation
* Link to other parts of the manual
* Show a sample input file with audio
* go through docs/scratch/ and make sure it's all here

This is the manual for [Syntoniq](https://github.com/jberkenbilt/syntoniq). Syntoniq converts musical notation in text files to CSound or MIDI output. Its purpose is to allow you to "code" score-like Music files and generate musical output suitable for final use or further manipulation in other tools.

Syntoniq's main feature is first-class support of arbitrary scales and tunings. Pitches are represented with a lossless notation. A score allows creation of scales and tunings dynamically with an array of transposition options available.

## What does it do?

* You work with a text file containing musical notation. Syntoniq "compiles" it into a musical timeline and converts it to one more or output formats.
* You generate one of several outputs:
  * A [CSound](https://csound.com) file
  * A Standard MIDI file
  * A JSON dump of the timeline

## What does it not do?

In the first iteration, Syntoniq does not create printed scores. It's possible that a future version of Syntoniq may generate MusicXML or LilyPond notation, depending on interest and time.

## Who is it for?

If you like creating audio with (LilyPond)[https://lilypond.org/] and are not trying to create printed scores, or you create music directly with CSound and are experimenting with microtonal music, you may like using Syntoniq. You can think of it is as a programmer's musical notation system. It's higher-level and more tightly focused than CSound. Syntoniq can be used to create a finished musical product, but it's designed to be more of a helper. Syntoniq creates note events CSound files that you can drop into your own template, thus freeing you from computing frequencies and so forth. The goal for MIDI output is that you should be able to import Syntoniq's MIDI files into whatever MIDI workflow you have and do additional fine-tuning.
