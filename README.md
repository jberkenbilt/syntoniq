# Syntoniq

<img src="logo/syntoniq-logo.svg" alt="Syntoniq Logo" style="height: 10em; vertical-align: middle; padding: 2ex;">

Welcome to *Syntoniq*. Syntoniq converts musical notation in text files to CSound or MIDI output. It was designed from the beginning to represent music in any tuning system, which makes it ideal for use with microtonal music. Syntoniq can generate MIDI using MTS (Midi Tuning System) with custom tunings and also MPE (MIDI Polyphonic Expression) with pitch-bend, specifically designed to be friendly to import into a Digital Audio Workstation for further refinement.

Major Features:
* Score-like layout of musical text files
* Ability to define arbitrary scales with a semantically meaningful and lossless pitch notation (no cents)
* Generated scales, allowing dynamic naming of notes based on intervals for pure Just Intonation and overlay of Just Intonation on scales based on interval division (EDO)
* Generalized transposition: transpose by absolute frequency, relative pitch multiplier, or by pivot notes in the scale
* Optional keyboard program with a flexible layout engine.

More details can be found in the [manual](https://syntoniq.cc/manual/)!

# PRE-RELEASE

December 2025

This software has not been released and is still undergoing active development. Every aspect of it is still subject to change, though I am working toward a 1.0 release.

# Resources

* [The WebSite](https://syntoniq.cc) which includes a detailed [User Manual](https://syntoniq.cc/manual/). If you want to learn about Syntoniq and how and why to use it, start there.
* [Manual Sources](./manual/content/) -- Zola/Markdown source code to the User Manual
* [Internal Docs](./docs/) -- developer-facing documentation, architectural notes, etc.
* Videos -- Coming. I am working on a few introductory videos. They will also be referenced from the manual.
