# Syntoniq

<img src="logo/syntoniq-logo.svg" alt="Syntoniq Logo" style="height: 10em; vertical-align: middle; padding: 2ex;">

Welcome to *Syntoniq*. Syntoniq converts musical notation in text files to Csound or MIDI output. It was designed from the beginning to represent music in any tuning system, which makes it ideal for use with microtonal music. Syntoniq generates MIDI using MPE (MIDI Polyphonic Expression) with pitch-bend, specifically designed to be friendly to import into a Digital Audio Workstation for further refinement.

Major Features:
* Score-like layout of musical text files
* Ability to define arbitrary scales with a semantically meaningful and lossless pitch notation (no cents)
* Generated scales, allowing dynamic naming of notes based on intervals for pure Just Intonation and overlay of Just Intonation on scales based on interval division (EDO)
* Generalized transposition: transpose by absolute frequency, relative pitch multiplier, or by pivot notes in the scale
* Optional keyboard program with a flexible layout engine

More details can be found in the [manual](https://syntoniq.cc/manual/)!

# PRE-RELEASE

January 2026

This software has not been released and is still undergoing active development. Every aspect of it is still subject to change, though I am working toward a 1.0 release.

# Resources

* [User Manual](https://syntoniq.cc/manual/) — everything you need to know to use Syntoniq
* [Syntoniq YouTube Channel](https://www.youtube.com/channel/UCxLdne2sP1iOPxpq8j49YIw) — contains demonstration videos that accompany the manual; all videos are linked from the manual as well
* [Sources](https://github.com/jberkenbilt/syntoniq/)
* [The WebSite](https://syntoniq.cc) — a good jumping off point
* [Manual Sources](./manual/content/) — Zola/Markdown source code to the User Manual
* [Internal Docs](./docs/) — developer-facing documentation, architectural notes, etc.

# Download and Install

You can find binary distributions of Syntoniq at the [GitHub Releases page](https://github.com/jberkenbilt/syntoniq/releases). The binary distributions are built in GitHub actions, which means the Linux versions are built on a recent Ubuntu LTS release. They are not tested on older Linux versions.

For detailed instructions, check the [manual section](https://syntoniq.cc/manual/introduction/installation/).

# Building from Source

Building Syntoniq from source is mainly just running `cargo build`, but there are some things you have to do around Csound. The manual is built using `zola`. You can look at [build_all](./build_all) or the CI scripts invoked from [GitHub Actions](.github/workflows/main.yml) for the full story on building everything.

If you have Csound installed and clang installed to support [bindgen](https://github.com/rust-lang/rust-bindgen), `cargo build` should usually "just work", but there may be extra steps you need to take.

## Csound

The `syntoniq` language compiler does not use Csound. It generates Csound output, but it doesn't use the Csound libraries to do this.

The `syntoniq-kbd` application build uses the `csound64` library by default. If you are only going to use the keyboard as a MIDI device, you can build with `--no-default-features`, and Csound will be skipped entirely. Otherwise, the keyboard build tries to figure out where Csound is located. It should succeed if you have installed a Csound binary distribution for Mac or Windows. It should also succeed on Mac if you have installed Csound with HomeBrew. On Linux, any installation that puts Csound in the standard include and library paths should work, including (on Debian-derived systems) `apt-get install libcsound64-dev`.

If you have Csound installed in a location where the Syntoniq build can't find it, you can set the following environment variables:

* `CSOUND_LIBDIR` — directory containing the csound library
* `CSOUND_LIB` — name of the library without prefixes or suffixes, usually `csound64`
* `CSOUND_INCLUDE` — name of the directory containing `csound.h`

At the time of initial Syntoniq release, Csound 7 is still in beta. It is expected to work without changes with `syntoniq-kbd`, but it hasn't been tested.

You can look at the scripts in [ci-build](./ci-build/) for examples.

Cross-compiling with `csound` is tricky because of issues with `bindgen` and the need to have Csound libraries for the foreign platform. It is possible to cross-compile Linux to Windows with `--no-default-features`, but this is not regularly tested. You can try `cargo build --config .cargo/windows-cross.toml --no-default-features`.

### Linux

You need development libraries for `csound` and `alsa`. On Debian-derived systems like Ubuntu, install `libasound2-dev` and `libcsound64-dev`.

### Windows

As of Csound 6.18.1, you have to install Csound using the MSI. The 64-bit Windows binary zip file doesn't contain all the required headers (probably a packaging error).

If you have a rust toolchain installed with MSVC and have added its clang support, you probably need to set `LIBCLANG_PATH` to enable `bindgen` to find it. You can try `cargo build --config .cargo/windows-cross.toml`.

### Mac

Both HomeBrew and the official installer should work. I've had better luck with the official installer, which includes universal binaries.
