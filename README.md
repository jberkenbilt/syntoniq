2025-07-19

# To Do

Other:
* Add a button that prints information about all the notes that are currently on. Will need to store last_note_for_pitch in transient state and remove entries when we turn off the pitch.
* Add a logger. This can subscribe to PlayNote events and use the optional note to log. It can record actual semantic note information as well as non-synthetic key events. We can have a replay mode that will transmit key events at specific times to effectively replay an entire session. This can be an alternate controller.
* Cabbage with wave form with filter, LFO, maybe stereo, detune, pan knobs as basic example
* Create note-entry CLI that uses HTTP interface; can be a separate tool to not interfere with logs.
* Replay session: don't run device controller; instead just synthesize the key events in the right order.

See also
* [design](./design.md)

# Documentation Notes

* All shifts and transpositions are reset on clear.
* Up/Down arrows transpose up/down octaves by changing the base pitch of the scale.
* The `Note` key transposes.
* `Note`, optional layout change, note, ..., note1, note1 sets the base pitch of the layout that was selected when `Note` was pressed to the pitch of note1 in whatever layout it appears. After transposition, the transposed (original) layout is selected automatically. Transpose is made final when the same note is pressed twice in a row.
  * Example: to set the tonic of EDO-19 to E from EDO-12
    * Select EDO-19
    * Press `Note`
    * Select EDO-12
    * Press E twice
* Shift layout
  * With shift key down, press two different notes. The layout is shifted so that the first note is now in the second note's position.
  * The shift key is sticky if pressed and released without touching other notes, so the following are equivalent, where "touch" is "press and release":
    * press shift, touch note 1, touch note 2, release shift
    * touch shift, touch note 1, touch note 2, touch shift

# Maybe Someday

* In web, track whether a key "down" or "up" just as a web only extraction. Each click sends the appropriate event and toggles the state which also sends an SSE event. No need to reset state -- click once and hit reset is the same as pressing clear while holding a note, and it prevents two successive on or two successive off events, which can't happen on the actual device. It means double-click for sustain mode, but that's fine.
* Tuned MIDI output mode: generate a Scala file for a layout that can be loaded into something like Surge-XT, which has tuning awareness. In EDO mode, we can just use notes starting from 60 up to the number of notes in the scale. For generic, we can create a 64-note scale and map each square to a note number in a consistent way, probably 60 to 123.

# Remember

* Up and Down arrows transpose by octaves, regardless of cycle size.
* You can send key events with `curl http://localhost:8440/key -d key=k -d velocity=v`
* Find QLaunchPad MIDI output port
  ```
  aconnect -l
  aseqdump -p 128:0
  ```

# Static Assets

Static assets are served from `static` with the help of `rust-embed`. The htmx files were downloaded:
```
cd static
wget https://cdn.jsdelivr.net/npm/htmx.org@2.0.6/dist/htmx.min.js
wget https://cdn.jsdelivr.net/npm/htmx-ext-sse@2.2.2
```
