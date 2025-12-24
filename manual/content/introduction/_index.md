+++
title = "Introduction"
weight = 1
sort_by = "weight"
+++

```syntoniq
syntoniq(version=1)
tempo(bpm=60)

; Define two 5-note scales with different pitches and partial overlap
; in note names.
define_scale(

  ; comment and blank line here

  scale="5-EDO"

) <<
  ^0|5 p
  ; comment in scale definition
  ^1|5 q

  ; also blank lines

  ^2|5 r
  ^3|5 s
  ^4|5 t
     2 w  ; same pitch class as p but out of cycle
  ^7|6 x  ; out of cycle, unique pitch class
>>
define_scale(scale="5-JI" cycle_ratio=2) <<
    1 p
  5/4 q
  4/3 r
  3/2 u
  7/4 v
>>

[p1.0]  1:c d e' f,

use_scale(scale="5-EDO" part="p1")

[p1.0]  1:p'2 q,2:> r:^ s:~
[p2.1]  1:c   d:.   e:> f:.^

; Transpose to set the base pitch to the current pitch of `d`, then
; switch to the 5-JI scale for p2.
transpose(written="c" pitch_from="d" part="p2") use_scale(scale="5-JI" part="p2")

mark(label="a")
tempo(bpm=72 start_time=1 end_bpm=96 duration=2)
[p1.0]  1:p q r s
[p2.1]  1:p q r u
[p3.0]  1:a e ~ 1/2:b% b#
  [p3]  64@2< 96@3
mark(label="b")

; Exercise scale degree logic more thoroughly by having notes that
; extend outside the cycle (beyond b#). This ensures notes are sorted
; properly. These go up by fifths.
define_scale(scale="fifths" cycle_ratio=2) <<
   1    c cc
   3/2  g
   9/4  d  ;  9/8
  27/8  a  ; 27/16
  81/16 e  ; 81/64
>>
define_scale(scale="fifths-3" cycle_ratio=3) <<
   1    c! c!!
   1/2  g!  ;  3/2  * cycle
   9/4  d!
  27/8  a!  ;  9/8  * cycle
  81/16 e!  ; 27/16 * cycle
>>
use_scale(scale="fifths" part="p1") set_base_pitch(absolute=264 part="p1")
use_scale(scale="fifths-3" part="p2") set_base_pitch(absolute=264 part="p2")
[p1.1] 2/5:c d, e,2 1/5:g:~ g 2/5:a,
[p2.1] 2/5:c!' a! g!'2 e! d!'

repeat(start="a" end="b")

; Create a manual mapping that has all the notes from 5-EDO
define_manual_mapping(mapping="m1" scale="5-EDO") <<
t ~ w x p'
p @q r s p,
>>
define_isomorphic_mapping(mapping="m2" steps_h=2 steps_v=5)
; Place the manual mapping in two distinct spots with different parameters.
place_mapping(
    layout="l1"
    mapping="m1"
    base_pitch=400
    keyboard="k"
    anchor_row=5
    anchor_col=4
    rows_above=3
    rows_below=1
    cols_left=2
    cols_right=6
)
place_mapping(
    layout="l1"
    mapping="m1"
    base_pitch=500
    keyboard="k"
    anchor_row=1
    anchor_col=8
    rows_above=1
    rows_below=0
    cols_left=1
    cols_right=2
)
; Place the isomorphic mapping to cover the whole keyboard
place_mapping(
    layout="l1"
    mapping="m2"
    base_pitch=300
    keyboard="k"
    anchor_row=12
    anchor_col=7
)
```



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


# Syntoniq

<img src="/syntoniq-logo.svg" alt="Syntoniq Logo" style="height: 10em; vertical-align: middle;">

```rs
if let Some(potato) = salad && "a".is_empty() {
    // TODO
}
```

```syntoniq
; This is a comment
nothing
sample_directive (x = "potato \"salad\"" y=3 
z=5/2 w=*^2|3*2/3*^-5|6  )
   [potato.1]  asdf
   [potato.2]   1:as3/df':~ ; comment
[salad] 12@1/2> ; potato 

mark(label="a")
tempo(bpm=72 start_time=1 end_bpm=96 duration=2)
[p1.0]  1:p q r s
[p2.1]  1:p q r u
[p3.0]  1:a e ~ 1/2:b% b#
  [p3]  64@2< 96@3
mark(label="b")

define_scale(scale="5-JI" cycle_ratio=2) <<
    1 p | 5/4 q
  4/3 r
  3/2 u
  7/4 v
>>

define_manual_mapping(mapping="m1" scale="5-EDO") <<
t ~ w x p'
p @q r s p,
>>

```

{{ include(path="introduction/hexboard.html") }}

{{ include(path="introduction/hexboard2.html") }}

{{ include(path="introduction/launchpad.html") }}

{{ include(path="introduction/launchpad2.html") }}

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
