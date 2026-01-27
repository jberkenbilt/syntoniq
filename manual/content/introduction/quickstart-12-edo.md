+++
title = "Quick Start: Syntoniq Language"
weight = 20
sort_by = "weight"
+++

In this section, we will discuss the basics of the Syntoniq language and produce some simple music using the default scale, which is the "regular" 12-tone equal tempered scale with unsurprising note names. Later sections will describe using Syntoniq for microtonal music.

Here's a simple Syntoniq score file:

<!-- generate include=hello.stq checksum=f7c2a15b5a54491b3b9f9e1c471b01ed442e3f2f5d1f75691ebc9e8c9bd4631e -->
```syntoniq
syntoniq(version=1)

; Here is some music
[p1.0]  1:g a    b  c'
[p1.1]  1:e f    g  g
[p1.2]  2:c    1:f  e
[p1.3]  2:~    1:d  d
[p1.4]  1:~ a,   g, c,
  [p1] 64@0<    127@4
```
<!-- generate-end -->

{{ audio(src="hello-csound.mp3", caption="Audio Created with Csound") }}

In this example:
* `syntoniq(version=1)` is a *directive*. The `syntoniq` directive has to appear before any other content (except comments, spaces, and blank lines).
* The line starting with `;` is a comment.
* The group of contiguous lines that start with bracketed identifiers are a *score block*.
* The lines that start with `[p1.n]` for some $n$ are *note lines*. `p1` is the *part name* and `n` is the *note number*.
* The line that starts with `[p1]` is a *dynamic line*.
* The lines are aligned, but this is not a requirement.

The rest of this section will explain in more detail.

FUTURE: Update this section when the reformatter is done.

# Structure of a Syntoniq File

A Syntoniq file represents a timeline of musical events. Its syntax takes some inspiration from both [Csound](https://csound.com) and [LilyPond](https://lilypond.org/), but it is unique to Syntoniq.

In Syntoniq, musical notes and dynamics belong to *parts*. A part can be assigned to exactly one instrument and can play multiple notes at once. At any given time, a part has a specific *tuning*, which consists of a *scale* and a *base pitch*. A part's tuning can be changed at any time.

A Syntoniq file expresses music in *score blocks*. A score block consists of *note lines* and *dynamic lines*.

A Syntoniq file can also includes *scale definitions* and, if you are using the keyboard, *layout definitions*.

# Syntax Basics

Syntoniq files usually end with the `.stq` extension, but this is not a requirement.

A Syntoniq file consists of the following things:

* Comments: the `;` introduces a comment. Comments last until the end of the line.
* Directives: these look kind of like function calls and provide general instructions to Syntoniq.
* Scale definitions: these allow you to define custom scales. You can either use Syntoniq's [generated scale](../../microtonality/generated-scales/) feature, or you can create your own completely custom scale where you give the pitches and note names.
* Layout definitions: if you are using the [Syntoniq keyboard](../../keyboard/), you can create layouts to place the notes of your scales on the keyboard.
* Score blocks: the heart of the language. Score blocks contain *note lines*, which include the notes and rhythms, and *dynamic lines*, which specify dynamics.

For a complete description of the Syntoniq language, see the [Language Reference](../../reference/language-reference/). Here are a few basics so you know what you're looking at.

A note consists of up to three parts: `duration:name:modifiers`. The duration is a *number of beats*. If you use csound, this will be familiar. If you are used to LilyPond, it is different. In LilyPond, `4` is a quarter note, `2` is a half note, etc. In Syntoniq, `1` is a beat, `2` is two beats, etc. Syntoniq doesn't have any concept of quarter notes, etc., as it breaks free from the usual notational conventions of Western music. Durations in Syntoniq can be fractions of a beat, but we'll come back to that later.

The note name always starts with a letter and may contain a wide range of characters. A note name may end with `'` followed by an optional number or `,` followed by an optional number. These indicate the number of *cycles* to go up (`'`) or down (`,`). These are similar to octave marks in LilyPond with some differences. If you want to go up or down two octaves, use `'2` or `,2` rather than repeating the mark. Also, notice that we used the term *cycle*, not *octave*. The term *cycle* refers to the interval over which a scale repeats. The default cycle size is the octave, but you can use other intervals&mdash;more on that later!

The symbol `~` is a *hold*. It means "keep doing what you were doing." In this example, it indicates a rest, but it can also mean "keep sustaining a sustained note".

This example doesn't include any note modifiers. Syntoniq supports a handful of modifiers to change articulation and note length and to sustain notes. There is no intention to add a wide range of modifiers to Syntoniq. Syntoniq's goal is not to be a complete musical production system. Its goal is to create music with fully specified pitch and dynamics. For experimentation, study, and working out melodic and harmonic ideas, this is often all you need. For complete musical production, you can take the output that Syntoniq generates and add further refinements in Csound or the Digital Audio Workstation of your choice.

# Playing the File

Use the `syntoniq` command-line tool to convert a file to musical output. Run `syntoniq --help` to see the available options and subcommands.

Save the above example to a file called `hello.stq`. Then you can run
```sh
syntoniq generate --score=hello.stq \
   --csound=hello.csd \
   --midi=hello.midi \
   --json=hello.json
```

You should see
```
syntoniq score 'hello.stq' is valid
JSON output written to hello.json
MIDI output written to hello.midi
Csound output written to hello.csd
```

The `--score` option is required. If no other options are given, `syntoniq` will just validate the score and report any errors it finds. The other options all tell `syntoniq` to generate a certain type of output.

The file `hello.csd` contains [Csound](https://csound.com) output. If you want to use Csound, you can install it from its website. Then just run `csound hello.csd` to hear the file. You can create your own Csound instruments to use with Syntoniq. By default, it includes a simple instrument with a simple wave form that's good for clearly hearing pitches and intervals...but you probably wouldn't want to listen to a piece of music with it! *Please note: you can create much better audio with csound. This is a limitation of Syntoniq's default instrument, not csound itself!*

{{ audio(src="hello-csound.mp3", caption="Audio Created with Csound") }}

The file `hello.midi` is a standard MIDI file with MPE (Midi Polyphonic Expression) compatible pitch bend statements. In this example, which uses regular 12-tone pitches, there won't be any, but for microtonal music, these are essential. A file like this can be loaded into a Digital Audio Workstation (DAW) or consumed by other MIDI tools. You can play this with a MIDI player of your choice. You can also render it with [FluidSynth](https://www.fluidsynth.org). The command `fluidsynth -iq -F a.wav a.midi` converts `a.midi` to `a.wav`.

<!--
To generate
* fluidsynth -iq -F /tmp/a.wav /tmp/a.midi
* convert to mp3 using same lame as in static-src/Taskfile.yml
-->
{{ audio(src="introduction/hello-fluid.mp3", caption="Audio Created by FluidSynth") }}

For better quality with fluidsynth, you can run `fluidsynth` interactively and give it the `interp 7` command. Then you send MIDI files to its input port. See documentation for `fluidsynth` for more help.

Another way to hear this on Linux is to install a synth tool, such as Surge XT, and to send the file to it using a tool such as `aplaymidi`. For example:
```sh
# start Surge XT and set up audio
aplaymidi --port='Midi Through' hello.midi
```
On other platforms, you can just load this into your favorite DAW or MIDI player.

<!--
To generate
* Start Surge XT
* Select Luna -> Analog Brass
* Start sox `rec` command with input set to monitoring the output device of Surge XT
* Use `aplaymidi --port='Midi Through'`
* Stop `rec`
* Trim the with with audacity and convert to mp3 using same lame as in static-src/Taskfile.yml
-->
{{ audio(src="introduction/hello-surge.mp3", caption="Audio Created by Surge XT with Luna/Analog Brass") }}

The file `hello.json` contains complete information about the timeline that `syntoniq` generated. You can use this for study, or it could be the basis for creating other ways to render the audio without modifying the Syntoniq software. All the information that the Csound and MIDI generators use is encoded in this JSON file.
