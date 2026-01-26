+++
title = "INTRODUCTION"
weight = 20
sort_by = "weight"
+++

<div style="padding: 2ex;">
<center>
<img src="../syntoniq-logo.svg" alt="Syntoniq Logo" style="height: 10em; vertical-align: middle;">
</center>
</div>

<!-- This opening paragraph also appears in ../start/_index.md -->
This is the manual for [Syntoniq](https://syntoniq.cc/). Syntoniq converts musical notation in text files to [Csound](https://csound.com) or MIDI output. It was designed from the beginning to represent music in any tuning system, which makes it ideal for use with microtonal music. Syntoniq can generate MIDI using MPE (MIDI Polyphonic Expression) with pitch-bend, specifically designed to be friendly to import into a Digital Audio Workstation for further refinement.

# Components

Syntoniq consists of two components:
* A *music creation language*, which allows you to "code" score-like Music files and generate musical output suitable for final use or further manipulation in other tools
* A *keyboard* that allows you to program a very small number of keyboards with arbitrary layouts and scales for experimenting with different tuning systems

# Features

**Syntoniq's main features include**
* Score-like layout of musical input text files that give you a dedicated line for each voice in a part, giving you full control of how notes transition. These notes also map to MIDI channels in a way that allows generated MIDI to be more editable.
* The ability to define arbitrary scales using flexible note names (including enharmonics) and lossless pitch representation. You can specify a pitch as a product of rational numbers and rational numbers raised to rational powers, which makes them *lossless* (no accumulating rounding errors). This makes it possible to play with Just Intonation, equal divisions of any interval, or to combine them. Syntoniq does not support Scala or other tuning files. The intention is to create scales that include *semantically meaningful* pitch definitions.
* Three built-in EDO (equal division of the octave) scales: 12-EDO, 19-EDO, and 31-EDO, that use "conventional" letter note names (`a` through `g` with "regular" sharps and flats). Adding many built-in scales is a *non-goal* of Syntoniq. This is about creating and exploring scales. Other tools allow you to use large libraries of existing scales.
* A *generated scale* concept that allows you to construct note names by chaining intervals. This is designed to make it easier to work with pure Just Intonation or overlays of Just Intonation on scales based on divided intervals. This is an advanced feature but is also arguably the most exciting and versatile feature.
* Generalized transposition. You can define a scale and then create a *tuning* with the scale by specifying a base pitch. You can specify an absolute base pitch, or you can transpose by multiplying a relative pitch factor with the base pitch or by assigning the pitch of one note to another note. This makes it possible to pivot from one tuning to another around a pivot note and to reverse any transposition. Flexible transposition and scale creation are available in the Syntoniq language and in the keyboard.
* Flexible layout engine. The keyboard allows you to create isomorphic layouts, where you specify the number of scale steps in each of two directions, or manual layouts, where you assign notes explicitly to grid locations. A layout can include multiple mappings and can combine manual and isomorphic mappings. Layouts can be "shifted" as well as transposed, meaning you can effectively slide the keys over. Shifting works with isomorphic mappings, allowing you to extend beyond what fits on the keys. It also works with manual mappings, where the entire mapped region is "tiled" horizontally and vertically with optional pitch shifting. Being able to create complex and combined layouts with shift and transpose allows you do things like create Just Intonation tunings and transpose them to different keys.

# Use Cases

**You might use Syntoniq to...**
* Transcribe microtonal music for study. You can listen to passages of microtonal music, pick out notes using the keyboard, and notate them in a score file.
* Experiment with harmonies in different tuning systems. You can define whatever scale you want (You want 22 divisions of the interval 13/8? Syntoniq can do it!), create one or more keyboard layouts, and poke around. Then create scores using notes from that system.
* Play with Just Intonation. Using pure Just Intonation usually requires calculating lots of ratios and making lots of decisions about exactly which note to use. Are you looking for a perfect fifth from a particular scale degree? Do you want chords to sound perfectly still, or are you intentionally picking a wolf interval or out-of-tune note for effect? Syntoniq's generated note system frees you from a lot of explicit calculation of ratios and makes it much easier to iterate. You still have to think about intervals and ratios as this is an inherent part of Just Intonation. Generated scales significantly reduce the friction of doing so.
* Use Syntoniq as a proxy for playing live music. If you are better as a composer or arranger than as an instrumentalist, or if you don't have an instrument that can produce the notes in your head, Syntoniq can help you produce a MIDI file that you can load into a workstation and edit. You can think of Syntoniq as a non-real-time musical instrument. The same applies if your workflow is Csound-based. Syntoniq frees you from calculating frequencies without taking away your freedom to use Csound's full capabilities to create sounds.

# Target User

**You might like using Syntoniq if...**
* You like creating music in [LilyPond](https://lilypond.org/). Like LilyPond, Syntoniq scores are plain text files. The `syntoniq` command-line tool uses Syntoniq files to generate a [Csound](https://csound.com) file, a standard MIDI file with MPE pitch bend, or a JSON file containing details about the timeline of musical events.
* You like creating music with [Csound](https://csound.com). Some ideas from Syntoniq are similar to Csound, and Syntoniq can generate Csound files or add timeline events to existing Csound files.
* You are interested in microtonal music. Syntoniq represents pitches using a *lossless notation* that represents pitches *exactly* and with *semantic meaning*. It does not use cents. Syntoniq allows you to create arbitrary scales and name the notes however you want. It also has its own native note naming convention that constructs pitches from *just intonation ratios* and/or *even interval division steps*. It breaks free of the baggage of traditional 12-tone constructs.
* You are primarily interested in arranging, composition, transcription, or study and are willing to use other tools to create a finished product. By design, Syntoniq doesn't have all the things you need to create finished musical works. It is concerned with pitch, rhythm, and dynamics. Syntoniq can create a MIDI file that's ready to import into any microtonal-capable DAW (Digital Audio Workstation), and it can create Csound events that you can use with the provided (minimal) orchestra file or combine with your own.
* You are comfortable working with command-line tools like compilers or content generators. The `syntoniq` command-line tool operates like a compiler: it validates your input, provides *clear, detailed error messages* if there are mistakes, and then generates output.

# Not For Everyone

**You might want to look elsewhere if...**
* You are looking for a single tool to bring you from idea to finished product. Syntoniq is not intended to do that, instead focusing on the hardest part (notating pitches for microtonal music), and leaving final production details to tools like the DAW. Syntoniq aims to supplement other tools, not to replace them.
* You are looking for a graphical interface for composing or arranging. Syntoniq doesn't have a GUI. You have to edit text files in an editor and "compile" them into MIDI or Csound.
* You want to use Scala or TUN files. Syntoniq has its own pitch notation and doesn't currently have support for Scala or TUN files. A future version of Syntoniq might *generate* scala or TUN files, but it is not likely to read them for the simple reason that Syntoniq's pitch notation aims to *improve upon* Scala and TUN files by providing semantic information about how a pitch is constructed and representing pitches *exactly* using a lossless notation (no floating point rounding errors).
* You are primarily interested in live performance or live coding. Syntoniq's keyboard component can be used for live performance, but it's not what it's made for, and it supports a very small number of devices.
* You want a printed score. Syntoniq does not generate printed scores. The Syntoniq syntax is designed to look score-like, but it doesn't substitute for a real printed score. Syntoniq has no opinion about how printed music should be represented, and it allows you to work with scales or arbitrary pitches for which there is no standard notation.

The rest of this manual describes Syntoniq's features in depth. It contains links to video files and sample audio, reference material, tutorials, and blog-style articles.
