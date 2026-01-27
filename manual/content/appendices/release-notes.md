+++
title = "Release Notes"
weight = 10
sort_by = "weight"
+++

This section includes release notes for the Syntoniq software. See also [docs/TODO.md](https://github.com/jberkenbilt/syntoniq/blob/main/docs/TODO.md) for the latest roadmap.

# v0.1.1 - not released

## Improvements

* Upgrade manual to Zola version 0.22. The syntax highlighting was simple enough for a successful AI conversion.
* Refactor Csound instrument to be more future proof. New parameters are varied through channels intead of arguments to the instrument.
* Drop support for MTS MIDI. The previous implementation was incompatible with glide, and writing MTS SysEx codes into a MIDI file is not the usual way of MTS. I have also not found any tool that supports MTS and MPE together. This means we also drop TiMidity++, which doesn't support MPE. We can use FluidSynth instead for simple SoundFont-based rendering, or use Csound's SoundFont opcodes.
* Implement pitch glide.

# v0.1.0 - January 22, 2026

This is the initial release of Syntoniq.

## Issues to Resolve Before 1.0

* Update Zola to 0.22 (manual, syntax highlighting); possible VSCode syntax highlighting (should use same format as manual)
* Implement `syntoniq calc` to cover items from `misc` scripts and a few others
* Fix a few MIDI generation edge cases/overflow conditions and improve pitch overflow handling
* Implement pitch glide
* Remove disclaimer about compatibility contract not being enforced before version 1.0.0

## Enhancements for After 1.0

* Interactive chord builder (CLI tool to type notes into)
* Syntoniq formatter (`syntoniq fmt`)
* LSP (Language Server Protocol) server with full syntax highlighting
