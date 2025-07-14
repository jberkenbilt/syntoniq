# TODO

This has scratch notes in varying degrees of completeness.

# Color Choices

off/off interval colors

2D/15  4th  5th  blue/green
35/38 +3rd -6th  magenta/pink
06/09 -3rd +6th  red/orange
25/0D root       cyan/yellow
01/03 others     gray/white

# Pitch Colors

color.py is a Python script for manually iterating with colors. To use it, first put the device in programmer mode with `./scripts/progmode`. You can then use it to set the colors based on the values by number to match the image on page 10 (also in `./colors.png`), a manual table, or a failed attempt at HSV. See above for the choices.

You can run `qlaunchpad colors` to see the colors in action. Along the bottom row, all the colors are displayed. Above, there is a scale. Touch any note to turn it and the other of the same pitch from the off to the on color.

With real scales, colors are determined by closeness to Just intervals. We tolerate a specific range, initially ±15 cents.

Use these ratios only:
* Fifth: 3/2
* Forth: 4/3
* Major third: 5/4
* Minor sixth: 8/5
* Minor third: 6/5
* Major sixth: 5/3

With the scheme, EDO-{12,19,31} will show all the intervals. EDO-17 will only show fourths and fifths. You will be able to tell at a glance which intervals are good in a given tuning system. I don't think it's worth trying to distinguish "okay" (like 12 cents) with "excellent" (like < 3 cents) with color, but it would be useful to provide that in some other fashion.

# Pitch specification

* `freq*a/b` = obvious
* `freq*a\b\c/d` = `c/d^(a/b)`, default for c = 2, default for d = 1
* Can chain multiplications

Examples:
- `440*3/5` = 264 = middle C with Just intonation with A 440
- `220*1\3` = middle C with EDO-12 with A 440 (300 cents above the A below middle C)
- `220*1\3*4\7` = 4th EDO-7 step starting from EDO-12 middle C
- `220*1\31\4` = 1 step of the division of two octaves into 31 equal divisions
- `264*9/8*6/5` = Just minor third above Just whole tone above middle C

Scale:
```yaml
- scale: name
  tonic: pitch
  octave: n  # steps in an octave, optional
  # one of
  step: step-multiplier
  notes: [multiplier, ...]  # start with `1` for the tonic
  names: [...]
```

* step-multiplier may contain n as the number of steps above base and may include `ak[+-]b` with count
* when octave is specified, len(names) must equal octave, and names repeat
* when octave is not specified, when you run out of names, it will just the be the multiplier

Examples:

EDO-31 scale with middle C based on EDO-12, A 440
```yaml
- scale: EDO-31
  tonic: 220*1\3
  octave: 31
  step: 1\31
  names: ["C", "C+", "C#", "D%", "D-", "D", "D+", "D#", "D%", "D-", ...]
```

Harmonic sequence starting with E 2.5 octaves below A 440
```yaml
- scale: Harmonic-Sequence
  tonic: 110*2/3
  step: n
  # names would be 1, 2, 3, 4, etc.
```

Sample just intonation
```yaml
- scale: key-of-C
  tonic: 264
  octave: 12
  notes:
  - 1
  - 17/16
  - 9/8
  - 6/5
  - 5/4
  - 4/3
  - 17/12
  - 3/2
  - 8/5
  - 5/3
  - 16/9
  - 15/8
  names:
  - "C"
  - "C#"
  - "D"
  - "E%"
  - "E"
  - "F"
  - "F#"
  - "G"
  - "A%"
  - "A"
  - "B%"
  - "B"
```

Layout:
- layout: name
  ll: x0,y0
  ur: x1, y1
  base: x, y
  scale: scale-name
  x-steps: n
  y-steps: n

Examples:

EDO-31 isomorphic with middle C at 5,5; 5 steps horizontal, 3 steps vertical covering entire grid
- layout: edo-31-functional
  ll: 1,1
  ur: 8,8
  base: 5, 5
  scale: EDO-31
  x-steps: 5
  y-steps: 3

This would let us divide up the keyboard any way we want, e.g., we could have stacks of just Intonation intervals like the minor third etude, an EDO-31 scale over six rows and an EDO-7 scale over two rows, etc. When we shift, it can just be within the grid. For octave shift, it's probably everything.

It might be useful to have an automatic layout for EDO. The only parameter would octave divisions, which we could get from the scale. The x-steps and y-steps parameters can always be computed as follows:
* x-steps = closest integer to octave steps/6
* y-steps = closest integer to octave steps/12
* if x == y, y += 1

See above for colors. I'm undecided about whether colors should be automatic based on various schemes or whether there should manual color. Specifying color manually will be hard because of launchpad color mapping.

The brightness should increase when the note is on. When a note is on, the same exact pitch (not just pitch class) should light up everywhere.

Other ideas:
* To shift the keyboard layout in real time
   - hit a move key, touch a key in its old spot; the key flutters, touch the key in its new spot; the whole keyboard moves
* key change: touch a key to make it the tonic without changing the pitch name
* transpose: same as key change except the note names change
* shift by octave up or down

Config file can define scales and layouts. We can assign them to control keys along the bottom, potentially flipping through pages using one of the arrow buttons for an unlimited number of scales.

Sustain mode: when in sustain mode, pitches toggle. When we shift around, the pitches stay as they are, making it easier to create big chords.

A clear button can turn off all notes.

Maybe we can save chords.

# Architecture

See [programmer's reference](~/Q/instruction-manuals/launchpad_pro_mk3.pdf)

See source/qlaunchpad. Written in rust with midly and midir. Uses programmer mode.

Rather than having separate tools, have a single tool that can be run with different subcommands for isolation.

Other abstractions:
* Scale
  * EDO
    * horizontal, vertical step size
    * divisions
    * set pitch of tonic as frequency or relative to a chain of tunings, e.g. EDO-7, tonic = 10th step of EDO-31 whose tonic is E in a 12-tone scale with A 440.
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
