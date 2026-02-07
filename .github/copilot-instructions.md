# Copilot Coding Agent Instructions for Syntoniq

## Repository Summary

Syntoniq implements a domain-specific language for creating microtonal music from a text file. It uses the MIT license. Syntoniq includes
* A compiler that outputs MIDI or Csound
* A keyboard program that works with a very small number of physical keyboards
* A detailed manual written with Zola, including custom syntax highlighting

Syntoniq is implemented in rust.

As a text-based microtonal music production system, Syntoniq will have a very small audience of very interested and technically sophisticated users. This is intended to be a very robust implementation. It was written by someone with deep math, coding, and music theory background and other successful open source projects. Given the niche audience and the fact that this is a hobby project, there is a limit to the amount of polish I can handle.

There are TODO items in the code and documentation. Some parts of the manual are marked with FUTURE.

## Build Instructions

For Linux, you must have csound and alsa development files. See `ci-build/build-linux` for the whole story, but that also performs cross compilation. For Ubuntu, the following is minimal, assuming a working rust toolchain with clang for bindgen:

```
sudo apt-get update
sudo apt -y install --no-install-recommends csound \
     libasound2-dev libcsound64-dev
cargo build
cargo test
```

You can use
```sh
./build_all
```
to build everything.

## Build and CI

Given the technical audience and the fact that this is a command-line tool written in rust, I have opted for zip/tar-based distributions. Given the small audience and infrequent release schedule, I plan to create releases manually. CI creates a distributions.zip file, which I can then archive and upload to create a GitHub release. I don't intend to further automate this for the time being.

## Manual Hosting

I maintain my own website on <https://syntoniq.cc>.

## Review Instructions

* Start with the top-level README.
* Look at the contents of the docs/ directory, especially TODO.md.
* Read the manual in `manual`.

### Noteworthy Code Items

* Complex Material
  * This is a microtonal music system. There's a lot of math and complex material, which is why I suggest reviewing the manual first. I have found current (2025/26) large language models to be capable of tracking with my system and successfully reasoning about it. As far as I know, a lot of what I'm doing here is novel.
* Unsafe code
  * The csound wrapper (keyboard/src/csound/wrapper.rs), using bindgen, has unsafe code. We use rust for threading, rather than the Csound library. The rust code sets things up and the interacts with the Csound library in a single OS thread. Passing the pointer wrapper into the thread is done with an unsafe Sync/Send implementation, and unsafe code is necessarily used to interact with the C code.
  * common/src/parsing/score_helpers.rs has a tiny bit of unsafe code to assist with converting borrowed to owned for certain data structures containing `Cow`. The unsafe code is used to maintain referential integrity for `Arc<T>` within the conversion.
* Parser
  * The code uses a series of Winnow-based parser combinators to perform parsing in three passes. The parser code is thoroughly commented. Considerable effort was made to use borrows rather than copies. The parser is very efficient.
  * Rather than using Winnow's error handling (cut, etc.), we use a `Diagnostics` object to maintain errors. Error messages are very detailed with plenty of context. They are intended to be extremely high quality.
  * docs/testing.md includes some information about test coverage. Overall, high test coverage is not necessarily a goal for this project, but there are parts of the parser that maintain 100% coverage. I have not automated the coverage checks, but I have a script that I use to check whenever I modify that part of the code.
* Keyboard
  * Most of the tricky logic is implemented in the keyboard through automated tests. There are no automated tests for the hardware layers. For this project, it's not worth building emulators, etc. I have an instance of each keyboard type and test manually.

## Use of AI

I have a firm rule against AI-generated documentation content. AI-generated code is allowed only when explicitly requested and must be marked as such. I appreciate AI review. As such, I prefer suggestions over rewrites, and I am much more likely to manually incorporate suggestions than accept a pull request. That said, suggested corrections to obvious typos or grammatical errors are welcome. Small code changes in pull requests are also fine when illustrative of an obvious error. Here are a few mistakes I make:
* `active` instead of `octave`, including `on active` instead of `an octave`
* `ration` instead of `ratio`
* `as` instead of `has`
* `is` instead of `as`

I am a native English speaker with good writing skills, but I sometimes move quickly and leave out or repeat words, etc.
