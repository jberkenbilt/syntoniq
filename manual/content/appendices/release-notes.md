+++
title = "Release Notes"
weight = 10
sort_by = "weight"
+++

This section includes release notes for the Syntoniq software. See also [docs/TODO.md](https://github.com/jberkenbilt/syntoniq/blob/main/docs/TODO.md) for the latest roadmap.

<!-- See issues to resolve ... below -->

# v0.4.1 - April 2, 2026

## Enhancements

* Add the `csound_global_instrument` directive for enabling global effect instruments, such as reverb, when using custom Csound templates.
* Add the `csound_template` directive, which takes a relative path, for specifying a Csound template. This can still be overridden from the command line.
* Add prompt syntax for saving pitches to and restoring pitches from variables.

## Bug Fixes

* Remove false "unknown part" error in Csound processor dynamics lines that appear in parts that don't otherwise appear in the region.

# v0.4.0 - April 2, 2026

## Bug Fix/Breaking Change

* A generated note with a `!n` override that specified only a number of divisions, not an interval, was interpreted to divide the scale's default interval rather than dividing the octave. This was inconsistent with documented behavior and also resulted in the `!` override creating a context-dependent pitch, which defeats the purpose.

## Enhancements

* This version introduces `syntoniq-kbd prompt`, an interactive chord builder that allows you to construct chords by typing generated note names at a command prompt.

# v0.3.1 - February 7, 2026

* Add the `syntoniq calc` command, which implements various pitch calculators.

# v0.3.0 - February 1, 2026

## Breaking Changes

* The transpose directive takes note names, which may now include octave/cycle markers, rather than strings for its `written` and `pitch_from` parameters. All examples have been updated.
* All directives that take `part` now take a raw identifier.

## Improvements

* Directives parameter values can now be note names and identifiers (such as part names) in addition to strings, pitches, and numbers. This paves the way for future semantic checks on note and part options.
* New directives added: `save_pitch`, `restore_pitch`, and `check_pitch`

# v0.2.0 - January 27, 2026

## Improvements

* Upgrade manual to Zola version 0.22. The syntax highlighting was simple enough for a successful AI conversion.
* Refactor Csound instrument to be more future proof. New parameters are varied through channels instead of arguments to the instrument.
* Drop support for MTS MIDI. The previous implementation was incompatible with glide, and writing MTS SysEx codes into a MIDI file is not the usual way of MTS. I have also not found any tool that supports MTS and MPE together. This means we also drop TiMidity++, which doesn't support MPE. We can use FluidSynth instead for simple SoundFont-based rendering, or use Csound's SoundFont opcodes.
* Implement pitch glide.
* Rework MPE channel allocation to use a pair of channels per note line so we can properly avoid immediate channel reuse. This means we only get 7 simultaneous notes per track, but we still allocate as many ports and tracks as required.
* Initialize volume and instruments for each channel unconditionally. This makes it easier to work with sending multiple outputs to the same MIDI device.

# v0.1.0 - January 22, 2026

This is the initial release of Syntoniq.

# Issues to Resolve Before 1.0

* Fix a few MIDI generation edge cases/overflow conditions and improve pitch overflow handling
* Remove disclaimer about compatibility contract not being enforced before version 1.0.0

# Enhancements for After 1.0

* Syntoniq formatter (`syntoniq fmt`)
* LSP (Language Server Protocol) server with full syntax highlighting
