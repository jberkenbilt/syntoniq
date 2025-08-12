2025-07-19

# To Do

Other:
* Cabbage with wave form with filter, LFO, maybe stereo, detune, pan knobs as basic example

See also
* [design](./design.md)

# Documentation Notes

* All shifts and transpositions are reset on clear.
* Up/Down arrows transpose up/down octaves by changing the base pitch of the scale.
* The `Note` key transposes.
* `Note`, optional layout change, note, ..., note1, note1 sets the base pitch of the layout that was selected when `Note` was pressed to the pitch of note1 in whatever layout it appears. After transposition, the transposed (original) layout is selected automatically. Transpose is made final when the same note is pressed twice in a row.
  * Example: to set the tonic of 19-EDO to E from 12-EDO
    * Select 19-EDO
    * Press `Note`
    * Select 12-EDO
    * Press E twice
  * Note: if in sustain mode, touching the note behaves as usual, so if you transpose by double touching a note that is on, you will turn it off. It's not clear what the best behavior would be, but a good policy would be that if you touch a note in sustain mode to transpose to and it's the wrong one, touch `Note` to cancel transpose and then turn the note off before proceeding. If you touch a note to transpose that it is already on, it will turn off. Either cancel and try again, or turn the note back on by touching the tonic of the transposed scale.
* Shift layout
  * With shift key down, press two different notes. The layout is shifted so that the first note is now in the second note's position.
  * The shift key is sticky if pressed and released without touching other notes, so the following are equivalent, where "touch" is "press and release":
    * press shift, touch note 1, touch note 2, release shift
    * touch shift, touch note 1, touch note 2, touch shift

# Maybe Someday

* In web, track whether a key "down" or "up" just as a web only extraction. Each click sends the appropriate event and toggles the state which also sends an SSE event. No need to reset state -- click once and hit reset is the same as pressing clear while holding a note, and it prevents two successive on or two successive off events, which can't happen on the actual device. It means double-click for sustain mode, but that's fine.
* Tuned MIDI output mode: generate a Scala file for a layout that can be loaded into something like Surge-XT, which has tuning awareness. In EDO mode, we can just use notes starting from 60 up to the number of notes in the scale. For generic, we can create a 64-note scale and map each square to a note number in a consistent way, probably 60 to 123.
* Create note-entry CLI that uses HTTP interface; can be a separate tool to not interfere with logs.
* Session record/replay: record non-synthetic key events with timestamps. We can have a replay mode that will transmit key events at specific times to effectively replay an entire session. This would run instead of the device controller.

# Remember

* Up and Down arrows transpose by octaves, regardless of cycle size.
* You can send key events with `curl http://localhost:8440/key -d key=k -d velocity=v`
* Find QLaunchPad MIDI output port
  ```
  aconnect -l
  aseqdump -p 128:0
  ```
