2025-07-19

# To Do

Other:
* Add a logger. This can subscribe to PlayNote events and use the optional note to log. It can record actual semantic note information as well as key events. We can have a replay mode that will transmit key events at specific times to effectively replay an entire session. This can be an alternate controller.
* The logger might want to track note names in addition to pitches. At this moment, note names are not visible to the PlayNote event, so a separate LogEvent might be in order. Can/should we add layout to Note?
* Cabbage with wave form with filter, LFO, maybe stereo, detune, pan knobs as basic example
* Create note-entry CLI that uses HTTP interface; can be a separate tool to not interfere with logs.
* Alternatives to the MIDI input controller
  * Replay recorded session

See also
* [design](./design.md)

# Documentation Notes

* All shifts and transpositions are reset on clear.
* Up/Down arrows transpose up/down octaves by changing the base pitch of the scale.
* The `Note` key triggers transpose or shift.
* `Note`, note1, note2 moves note1 to note2's position
* `Note`, optional layout change, note1, note1 sets the base pitch of the layout that was selected when `Note` was pressed to the pitch of note1 in whatever layout it appears. After transposition, the transposed (original) layout is selected automatically.
  * Example: to set the tonic of EDO-19 to E from EDO-12
    * Select EDO-19
    * Press `Note`
    * Select EDO-12
    * Press E twice

# Maybe Someday

* In web, track whether a key "down" or "up" just as a web only extraction. Each click sends the appropriate event and toggles the state which also sends an SSE event. No need to reset state -- click once and hit reset is the same as pressing clear while holding a note, and it prevents two successive on or two successive off events, which can't happen on the actual device. It means double-click for sustain mode, but that's fine.

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
