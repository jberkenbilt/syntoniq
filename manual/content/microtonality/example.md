+++
title = "Complete Example"
weight = 80
sort_by = "weight"
+++

In previous sections, we've seen most of the features of the Syntoniq language. This section covers the remaining features by walking through a complete example.

# Quick Reminders

Here are a few things to keep in mind. I mention them up front to avoid repeating them throughout the section.

* The layout definition parts of the syntoniq language are not covered in this section. They are discussed in [Layout Engine](../../keyboard/layout-engine/)
* See the [Language Reference](../../reference/language-reference/) section for detailed syntax and a complete list of directives. This includes descriptions of each available parameter. We may not show every parameter to every directive in this section.
* You can run `syntoniq doc` for a summary of all directives. That text and the directive section of the language reference come from the same source and will always be in sync, though `syntoniq doc` will contain documentation for the version of `syntoniq` you are actually running.

# An Example Score

We'll use the score below to demonstrate remaining features.

<!-- generate include=full-example.stq checksum=c3f7c562285bd180ac03ce301ce6a3d5c36f7cc9f3c290527fe97586bafc62b2 -->
```syntoniq
syntoniq(version=1)
; Define scales with different octave divisions
define_generated_scale(scale="gen-17" divisions=17)
define_generated_scale(scale="gen-5" divisions=5)
; Use gen-17 for both chords and bass
use_scale(scale="gen-17" part=chords part=bass)
; Use gen-5 for the melody
use_scale(scale="gen-5" part=melody)
; Transpose melody up an octave
set_base_pitch(relative=2 part=melody)
; Transpose bass down an octave
set_base_pitch(relative=0.5 part=bass)
; Set a global tempo
tempo(bpm=80)

mark(label="opening")
[chords.0] 1:A    A   2:A    | 1:A    MA   2:A:&~
[chords.1] 1:JK   I   2:JK   | 1:JK   MJK  2:JK:&~
[chords.2] 1:C    D   2:C    | 1:C    MC   2:C:&~
[chords.3] 1:CJK  DJK 2:CJK  | 1:CJK  MCJK 2:CJK:&~
[chords.4] 1:I'   A'  2:I'   | 1:I'   MI'  2:I':&~
[chords] 127@0 |
[bass.0] 2/3:A Bi, C, 2:A,:~ | 1:A,:~ 1:A,:&~ 2:A:&
[bass] 127@0 |

mark(label="transition")
transpose(part=chords part=bass written=A pitch_from=A1)
[chords.0] 1:A    A   2:A    | 1:A    MA   2:A
[chords.1] 1:JK   I   2:JK   | 1:JK   MJK  2:JK
[chords.2] 1:C    D   2:C    | 1:C    MC   2:C
[chords.3] 1:CJK  DJK 2:CJK  | 1:CJK  MCJK 2:CJK
[chords.4] 1:I'   A'  2:I'   | 1:I'   MI'  2:I'
[chords]   127@0> | 64@2
[bass.0] 2/3:A Bi, C, 2:A,:~ | 4:A,
[bass]   127@0> | 64@2

mark(label="a")
reset_tuning(part=chords)
use_scale(scale="gen-17" part=chords)
transpose(part=bass written=A1 pitch_from=A0)
[chords.0] 1:A    A   2:A    | 1:A    MA   2:A
[chords.1] 1:JK   I   2:JK   | 1:JK   MJK  2:JK
[chords.2] 1:C    D   2:C    | 1:C    MC   2:C
[chords.3] 1:CJK  DJK 2:CJK  | 1:CJK  MCJK 2:CJK
[chords.4] 1:I'   A'  2:I'   | 1:I'   MI'  2:I'
[melody.0] 1/2:~ A' A4 A3 A4:. A3 A2 A1 | A a1 1/2:A A1 A2 A1 A2:^ A3:>
[melody] 127@0 |
[bass.0] 2:A, Cy, | 4:A,
mark(label="b")
repeat(start="a" end="b")

mark(label="out")
tempo(bpm=80 start_time=1 end_bpm=60 duration=3)
[chords.0] 8:A,
[chords.1] 8:C,
[chords.2] 1:I IM I Im 4:I
[chords.3] 1:C CM C Cm 4:C
[chords] 64@4< 127@8
[melody.0] 4/5:A A1 A2 A3 A4 1/2:A' A1' A' a1' 2:C!17
```
<!-- generate-end -->

{{ audio(src="full-example-csound.mp3", caption="Example Score") }}

# Overall Structure

Throughout most of the manual, we've been using generic part names like "p1" and "p2". You can use any alphanumeric characters or underscores in your part names as long as they start with a letter. Here, we are using the more descriptive part names "melody" and "chords". You can use instrument names, voice names, etc.

This selection starts with some chords as an opening introduction. They use a generated scale in 17-EDO. The material is placed once, then repeated up a step, as in the transposition example from the previous section. After that, we repeat the chord sequence with a melodic section in 5-EDO. This is repeated using a `repeat` directive. Then we have a closing passage. The remaining sections will look at this a block at a time.

The very last note in the melody uses the note `C!17`. While the melody is in 5-EDO, the last note is coerced into 17-EDO so it is in tune with the corresponding note on the chord. We could have used `define_scale` to create a scale, as discussed in [Defining Scales](../scales/), but this provides a useful example of the flexibility of the generated scale system once you get used to it!

# Top Section

In the top section of the score, we define scales using `define_generated_scale`. You can recognize scale-related directives from previous sections. Note the duplication of `part` in `use_scale` for `"gen-17"` to use that scale for more than one part with a single directive.

The `tempo` directive is used to set the tempo to 80 beats per minute. We use `set_base_pitch` with a relative pitch of `2`, indicating an octave, to raise the pitch of the `melody` part by one octave. We use it with a relative pitch of `0.5` to lower the pitch of `bass` by an octave. We could have used `1/2`, but we used `0.5` to demonstrate the use of decimals. Syntoniq allows ratios (and numerators of ratios) to contain up to three decimal places.

# Bar Checks

These score blocks use bar checks. This is the `|` character. For note lines, Syntoniq ensures that each line has the same number of bar checks, and that each "bar" has the same number of beats. This divides a line up into something like measures in a conventional score, except there is no time signature, so Syntoniq just checks consistency within lines in a block.

# Dynamics

The score blocks contain lines that start with bracketed identifiers without note numbers, such as `[chords]` and `[bass]`. These are *dynamic lines*. A dynamic consists of `level@beat`, where `level` is a number from 0 to 127, and `beat` is a beat offset from the previous bar check or, if no bar checks, the beginning of the line. Syntoniq ensures that the beat offset does not exceed the number of beats in that bar-check-separated region.

In the block that follows `mark(label="opening")`, we set the dynamics for `chords` and `bass` to full volume. We'll talk about marks and repeats below.

In the block following `mark(label="transition")`, you can see dynamics followed by `>`. This indicates a diminuendo. Syntoniq enforces that there is a subsequent dynamic marking whose volume is lower. The volume is linearly scaled from the old to the new dynamic. For MIDI, the instrument has to support this, and if you are using MIDI in a digital audio workstation (DAW), you may also need to ensure that the DAW is not blocking changes. Syntoniq uses velocity to indicate volume. For Csound, the velocity is scaled to a number from 0 to 1 and is passed as a parameter to the instrument. You can use it with custom instruments as you wish. Custom Csound instruments are discussed below.

In the last score block, you can see a crescendo. This is a dynamic followed by `<`. Here, Syntoniq ensures that there is a subsequent dynamic whose value is greater.

# Tempo

There are various `tempo` directives. The first one sets the initial tempo. The second one, preceding the final score block, indicates a start time, a duration, and an end tempo. The start time is an offset from the current moment in the score, which is the time offset at the beginning of the next score block. This indicates a gradual (linear) tempo change over the number of beats specified in the duration.

# Tie and Glide

The end of the first block glides smoothly into the beginning of the second block. The bass note is first glided up an octave over one beat. Then all notes glide smoothly up one 17-EDO step. The bass note is re-articulated because the glide is not combined with a tie. This demonstrates the use of glide across a tuning change.

# Polyrhythms

Notice the use of durations like `4/5` and `2/3` in various places in the score. This is how to do tuplets in Syntoniq. Setting a beat length of `4/5` creates a quintuplet. `2/3` creates a triplet. That's because 5 notes of duration `4/5` take $\frac{4}{5} \times 5 = 4$ beats. The same logic applies to `2/3`. You can use fractional beats like this to create arbitrarily complex polyrhythms.

# Reset Tuning

We have previously encountered various directives for transposing and setting pitch. Immediately after `mark(label="a")`, you can see that we use the `reset_tuning` directive to completely clear the tuning for the `chords` part. This is gratuitous and is just to show `reset_tuning`. To get us back to "gen-17", we had to call `use_scale` again. We could have just passed `part=bass part=chords` to the transpose we use to reverse the transposition of the `bass` part.

# Articulation

In the melody line, you see notes followed by `:.`, `:^`, and `:>`. The first of these shortens the note. It is mnemonically related to staccato, but its behavior is to shorten a note by a quarter of a beat. The `^` and `>` are like marcato and accent. They slightly increase the velocity. These are discussed in more detail in the language reference.

# Marks and Repeats

This example uses several marks and repeats. A mark has a label and just indicates a point in the timeline. The `repeat` directive repeats the material between the specified start and end marks *as musical timeline events*. It is important to understand that repeat is *not lexical*. That means that whatever *sound* was generated between the marks is repeated. Any tuning in effect in that region is preserved. There is a technical explanation of the rationale at the end of this section.

# Controlling Playback

In addition to using marks for repeats, you can also use them from the `syntoniq` command line to start and/or end the playback at specific locations.

In addition to starting and ending at specified places, you can use the `syntoniq` command to limit which parts are played and to apply a global multiplier to the tempo. These can help you iterate over a particular passage and may be easier to use than the corresponding features of your DAW or of the `csound` command. Please run `syntoniq --help` for more details on available command-line arguments.

# Using different MIDI instruments

The `midi_instrument` directive can be used to cause Syntoniq to generate a *control change* event to set the instrument number for a given part. The meaning of the MIDI instrument number is dependent on your tools. Syntoniq is not opinionated about this. It just passes the bank and instrument numbers through. This is most useful if you are using something like [FluidSynth](https://www.fluidsynth.org/) to render MIDI files with a SoundFont. In that case, you should set the instrument (and, if used, bank) number to whatever matches your sound font. If you are using MPE-based MIDI directly with a synth (like Surge XT), you will probably have only a single instrument. If you are using Syntoniq to generate MIDI with MPE and are going to do further work in a digital audio workstation (DAW), MIDI instrument specification in Syntoniq is probably not useful as you will have to assign instruments in the DAW. You can still set the instrument number in Syntoniq and make use of it in whatever way works for your workflow.

Syntoniq tries to generate MIDI that is useful for further processing. If you are trying to use Syntoniq to generate MIDI that you are using in a DAW but find that it falls short in some way, please open a [GitHub issue](https://github.com/jberkenbilt/syntoniq/issues/). The source code of the MIDI generators in `syntoniq` contain detailed information about exactly how `syntoniq` assigns parts and notes to ports, tracks, and channels.

# Defining Additional Csound Instruments

The `csound_instrument` directive works similarly to the `midi_instrument` directive in that it allows you to assign some or all parts to a different Csound instrument. This allows you to specify numbered or named Csound instruments. Creating a Csound instrument is out of scope for this manual, but here are the important details.

Run `syntoniq csound-template` to get a copy of the Csound template that `syntoniq` uses. It has `BEGIN SYNTONIQ` and `END SYNTONIQ` comments. Any Csound file with those comments can be used as a Csound template. The `syntoniq` application replaces whatever is between those markers with its output. When Syntoniq generates Csound, it *leaves the markers in place*. This makes it possible to iterate on the other parts of the file and use the same file as a template when replacing the notes. The Csound template contains comments describing how it works and what the constraints are on any instrument you design. See that output for the ground truth. In a nutshell, your instrument must take the same parameters as the default instrument, but beyond that, you can do anything you want. Remember that you can create an adapter instrument for Syntoniq's use that just passes data to other instruments you define. If you have a developed Csound workflow and want to use `syntoniq` to generate notes but find that it falls short in some way, please open a [GitHub issue](https://github.com/jberkenbilt/syntoniq/issues/).

# Technical Note on Mark/Repeat

This section is entirely optional! Read it if you want to know some rationale behind why mark and repeat were designed the way they were.

Earlier in the section, we mentioned that mark and repeat work on the *timeline*, not on the lexical aspects of the file. This makes them more *semantic* and less like macros.

When designing marks and repeats, there are two sensible approaches:
* Repeat the musical note events generated between the start and end marker (what we do)
* Lexically repeat the text between the start and end marker

The choice of repeating musical note events makes Syntoniq behave more like how a printed score would behave. In a printed score, if you have a D.S. mark that carries you back to a part of the music in a different key or different tempo, you are back in the key/tempo of the repeated section. There are certain things you generally avoid, like tying notes across repeat boundaries, though this is sometimes encountered. To reduce surprise, the Syntoniq languages disallows certain constructs, such as unresolved tied notes, dynamic changes, or in-process tempo changes, from spanning across repeat boundaries. That ensures that the actual sounds that are made during a repeated section are fixed across repetitions.

With syntoniq, the *valid note names* vary based on what tuning you're in. Having repeat work like a macro could be very confusing as a section may be syntactically valid in one context and syntactically invalid in another context after the part was retuned to a different scale. Treating repeated sections as already baked sound greatly simplifies the validation logic that has to be performed by the `syntoniq` compiler...but that's not just laziness. It also reduces cognitive load for the user. If you repeated a section lexically in a new section of the score that had different valid notes, the error messages would be quite confusing. If you want to do that, it's better to just cut and paste.

Note that this approach also has a compelling advantage for the kind of music that Syntoniq enables. Suppose you have a piece that has a repeated section in a certain tuning but cross into other tunings through transposition or otherwise throughout the piece. When you repeat a section in Syntoniq, you always know you are going to hear the same thing. The key, tunings, transpositions, meanings of note names, etc. are all the same.

If you really want to repeat sections lexically, my suggestion would be to use something else to generate your Syntoniq score. For example, you could use a Jinja template (or pick your favorite template engine) or any other kind of macro language (`m4` anyone? `/lib/cpp`? Am I dating myself?) to handle lexically repeated sections. I had considered adding a macro system to Syntoniq but decided not to. This is a solved problem through one of these other mechanisms, but the logic around a timeline/musical-event-based repeat system is complex and can only be implemented by Syntoniq itself.
