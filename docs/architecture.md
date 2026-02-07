# Architecture

Future work:
* reformat
* LSP

TODO:
* unsafe code in csound as pattern for talking to csound library including Sync/Send and threading in rust instead of C
* Keyboard core components/device isolation
  * engine
  * controller
  * device
  * keyboard
  * web
* Parser: raw directives, proc macro, and FromRawDirective trait
* ToStatic, owned data in layouts section, unsafe code in ArcContext
* Parser architecture
  * winnow
  * diagnostics
  * helpers
  * degraded modes
* web UI including templates, HTMX, and SSE



* Reformat
* LSP
* Tree-sitter
* Printed scores (maybe)

Notes about mark/repeat for architectural docs. They are based on the timeline, not the token stream. This has both advantages and disadvantages.

Mention scripts in misc for launchpad and hexboard

----------------------------------------------------------------------

Below is very early (2025-07) from keyboard-design.md. Clean it up bring it up to date.

# Architecture

Partly obsolete -- fold into docs/architecture.md

See [programmer's reference](~/Q/instruction-manuals/launchpad_pro_mk3.pdf)

See source/syntoniq. Written in rust with midly and midir. Uses programmer mode.

Rather than having separate tools, have a single tool that can be run with different subcommands for isolation.

Other abstractions:
* Scale
  * EDO
    * horizontal, vertical step size
    * divisions
    * set pitch of tonic as frequency or relative to a chain of tunings, e.g. EDO-7, tonic = 10th step of 31-EDO whose tonic is E in a 12-tone scale with A 440.
    * color based on metadata, e.g., diatonic scale for Key of G+ should be white with one color for relative steps in different directions; this makes it easier to play in a key but still get notes reported properly for notation. So if I want to play in D major, I can shift the colors to be for D major. Might also want completely other coloring schemes. Probably need to know steps relative to octave, steps relative to nearest diatonic with specified tonic, ...? Could also want "black keys" colored differently, e.g., in D, D♯ is in the chromatic scale while E♭ isn't, and it may be useful to indicate beyond just that they are both two steps away from a diatonic. Not sure how much of this is really useful once I get used to the layout.
    * above implies we should be able to transpose everything but also not transpose and just shift colors as if transposed. These solve different problems.
  * Just, arbitrary
    * Might be interesting to stack Just intonation scales or just create arbitrary pitches
    * Might be interesting to divide the keyboard into two sections, or possibly use control keys to shift around between different tuning/color systems.

Sustain mode: when we are playing a note by holding or in sustain mode, it and all its equally pitched notes should be lit if visible. We should be able to transpose without turning pitches off for easier polytonality and for playing wide chords that cover more range that we can display at once.

Other to do:
* Add the ability to send notes to a midi out port so this can drive Surge-XT. We can map buttons to arbitrary notes and generate new events rather than passing them through so the isomorphic keyboard can work. Just make sure we send the right note number for the tuning system. This may not be worth bothering with once csound is wired up.

Older notes below.

# Launchpad Controller

The controller will have some kind of interface, possible socket, http, stdin/stdout prompt, or some combination. It's sole function will be to send information about raw events, accept lighting changes, dump the entire configuration, and enter/exit this mode. Once done, this program should require very little further maintenance. It would be great if we can use something like server events so people can subscribe to notifications. This would make it possible to create web application that can reflect the real-time state of the device with metadata. Alternatively, the controller may just communicate with socket/CLI, and the main application may have the http interface.

# Main application

This contains the "business logic." Each square should have a bunch of information about it. My thought is that pressing or releasing any button can cause any other button to change state. A button can have a number of different states, perhaps numbered, or perhaps named, or maybe both. Examples:

* The chord mode button could toggle whether in sustain mode.
* Button 55 could be middle C. It could have several states:
  * Off: {label, color, metadata}
  * On: {label, color, metadata}
  * Active: ...
  Where Active is the state when the finger is actually on the key and maybe can respond to other data like velocity or pressure (I don't know what velocity is, but I think it's some midi sequencer metadata and may not be applicable). I'll have to see what the messages from the controller look like.

If in sustain mode, pressing a button might toggle between on or off. If not, pressing the button will turn it on, and releasing will turn it off. We probably want some kind of `id` to be assigned to each note so we can treat duplicated notes identically...when you paint one, the other one changes color. As such, it might make more sense to define notes as separate things with ids and then to map notes to buttons.

Metadata would be useful so we can write color rules. It might be useful to just hand-generate complete layouts with maybe some static helper code to do it.

It should also be possible for buttons to change the layout, such as transposing pitch, changing the key, which could change colors, physically shifting the keys, changing the color scheme, etc. I'll have to think through what some options are.

We can't put a label on the key on the launchpad itself, but if there were a graphical display, we could label notes on it. One can imagine a graphical display that mirrors the state of the launchpad and is enhanced with better labels.

I think the controller is probably stateless, and the main application is responsible for logging.

The main application is probably also responsible for interfacing with csound.

I think the main application could be written in rust or go. I see nothing about it that requires Python. The controller should probably be in Python because it will be the most ergonomic way to interact with Midi, but maybe the whole thing can be go. I doubt it would be worth using rust for this.

# UI Console

My intention is for this to be a read-only view. See ~/source/examples/rust/http-server-axum/ for a basic HTMX/SSE framework that should work well for this.
