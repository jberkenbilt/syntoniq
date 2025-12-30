
* https://www.getzola.org/
* https://www.getzola.org/themes/deepthought/ (not used, for reference)
* https://www.getzola.org/themes/book/
* https://github.com/getzola/book

```sh
cargo install --locked --git https://github.com/getzola/zola
```

```
zola init
zola build
zola serve
```

```
cd manual/themes
git clone https://github.com/getzola/book
```

```
commit 4ee06ce568e1c9f6d19f53bf521fb267603bc6c4 (HEAD -> master, origin/master, origin/HEAD)
Author: Miguel Pimentel <contact@miguelpimentel.do>
Date:   Fri Mar 14 12:12:57 2025 -0700
```

TODO: after cleaning commits, make a vendor branch off the book theme.

```
+++
title = "??TITLE??"
weight = 0
sort_by = "weight"
+++
```

Note ./ordering for tweaking section order
* git commit
* Run `./ordering --current >| /tmp/a`
* Edit /tmp/a to put the sections in order
* Run `./ordering --apply /tmp/a`

Remember to use get_url for absolute paths in the Zola templates -- see manual/templates/index.html.

# Generated Content

There is a magic comment `<!-- generate ... -->` that can appear in markdown sources. It has a very exact syntax that is recognized by `./autogen`.

Generated sections are always delimited with
```
<!-- generate k=v k=v ... -->
# generated material
<!-- end-generate -->
```

Valid operations:
* `include=file checksum=...` -- include the contents of `static-src/file` verbatim. The checksum is updated if the file changes so we can avoid gratuitously updating files. This can be used to include source examples or other things. Files in `static-src` can be generated or manual. The script knows to quote .stq files with ` ```syntoniq ` and may have other special case logic.

Audio files can be automatically generated from stq files for the manual. You have to add them to `manual/static-src/Taskfile.yml`.

----------

TODO:
* Include LOGO using an img tag. Will need build logic to populate the src/assets directory.
* To get a keyboard HTML file, get the keyboard in the right state, then run `curl http://localhost:8440/board` and save to a file. Make sure this is in manual/README.md along with populating assets.
* Feature Summary; mention videos with internal link
* Build and installation
* Link to other parts of the manual
* Show a sample input file with audio
* go through docs/scratch/ and make sure it's all here
* Embed KeTeX rather than getting from a CDN

# Doc outline

For CONTRIBUTING.md, not the manual:

* Various sections on how Syntoniq is implemented including pub/sub architecture, MIDI port/channel allocation strategies, CSound polyphony logic, others
* Deeper dives on Syntoniq's architecture, written as blog-style articles; many will become blog posts (like my particular use of winnow, `ToStatic` pattern for keyboard layout reload, bindgen for csound library)
* Testing -- how to run the automated tests, coverage analysis, how to listen to generated audio from the test suite, how to compare actual vs. expected MIDI files for both MTS and MPE
* Managing the docs: including HTML keyboard dumps, generated content, etc.


# TODO

- [Generated Scales](generated_scales.md)
- [Keyboard](keyboard.md)
- [Pitch Notation](pitch_notation.md)
- Creating Layouts
- [Examples](examples.md)

# Design and Implementation Notes

- Syntoniq Generator
  - [Testing](testing.md)
  - [Parser Infrastructure](parser_infrastructure.md)
  - Pass 1 Tokenizer]
  - Pass 2 Parser
  - Pass 3 Output
  - Timeline
  - [Layout Engine](layout_engine.md)
  - [Owned Layouts](owned_layouts.md)
  - [Directives](directives.md)
  - [Data Blocks](data_blocks.md)
  - [Generators](generators.md)
    - [CSound Generator](csound_generator.md)
    - [MIDI Generators](midi_generators.md)
- Keyboard Architecture
   - Event System
   - [Keyboard Core Components](keyboard_core_components.md)
   - [Web UI](web_ui.md)
   - [CSound Playback](csound_playback.md)
   - [MIDI Playback](midi_playback.md)
   - [Device Isolation](device_isolation.md)
   - [Lauchpad Specifics](launchpad.md)
   - [HexBoard Specifics](hexboard.md)

# Appendices

- [Syntoniq Name](syntoniq_name.md)
- [Syntoniq Logo](syntoniq_logo.md)
- [Roadmap](roadmap.md)
