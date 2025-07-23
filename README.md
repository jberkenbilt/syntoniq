2025-07-19

# To Do

Before doing move (transpose/shift), implement automated testing for engine.
* Should be able to just start it with events and monitor events and internal state.

Desired behavior for move (transpose/shift) (different from below):
* Press Note
* Move state moves to Pending
  * Pressing Note key again cancels the operation; resets any note color changes
* Press note1
  * Move state switches to FirstSelected
  * save note, position, and current layout
  * selected color (if possible, or color of Note button itself) changes to NoteSelected
* Press note2
  * Make sure either canceling or completing restores the color of note1
  * If same as note1 (same layout, same position, same note), set base pitch of the layout that was active when note1 was pressed to the pitch of note2
  * Otherwise, if different layout or selected layout is not EDO, cancel
  * Otherwise, adjust the base position of the originally selected layout by the delta between note1 and note2 positions
* Note about note colors:
  * When sending note event, check to see whether the current layout and position matches what was stored in the move state of FirstSelected. If so, override with NoteSelected color. This would require encapsulating the call to note.light_event so it's only called in one place and can be overridden.

* There are TODO comments in all the other places.
* Probably should update side-info with in-progress and result. We
  probably need a "current operation" or something.

Next tasks, in undefined order, are:
* Vertical arrow keys (70, 80) shift pitch up and down by a cycle (or octave if no cycle)
* Pitch shift; see below
* The logger might want to track note names in addition to pitches. At this moment, note names are not visible to the PlayNote event, so a separate LogEvent might be in order.

Shifting and transposition mute the scales. When we reset, scales loaded from the config file and are owned by the engine. Octave shifting and transposition change the base pitch. Note shifting changes the base position.

Pitch shift:
* Pitch/scale overflows: handle gracefully for moving up/down octaves
* `shift` key behavior
  * When pressed, toggle
  * When released, if any other key was touched, turn off; otherwise no action
  * Implies shift down, shift up, shift down, key, shift up => effect shift is ignored
* For non-EDO: octave shift with ^/v keys
* For EDO
  * ^/v keys shift by a cycle
  * shift, note1, note2 moves note1 to the position of note2

Transposition:
* Press Note key
* Note key flashes; currently selected scale is pending
* The pitch of next note pressed becomes the tonic of the pending scale.
* Example:
  * Select a Just Intonation scale
  * Press the "Note" key; note flashes
  * Switch to EDO-19
  * Touch the key for step 1
  * Note turns off; JI layout is selected
  * Now the "C" of the JI scale has the pitch aligned with EDO-19's step 1.

Other:
* Add a logger. This can subscribe to PlayNote events and use the optional note to log. It can record actual semantic note information as well as key events. We can have a replay mode that will transmit key events at specific times to effectively replay an entire session. This can be an alternate controller.
* General keyboard layout. Probably don't bother with specific JI/Harmonic layouts
  * define 64-element array of relative pitches and 64-element array of names
* Cabbage with wave form with filter, LFO, maybe stereo, detune, pan knobs as basic example
* Create note-entry CLI that uses HTTP interface; can be a separate tool to not interfere with logs.
* Alternatives to the MIDI controller
  * Replay recorded session
  * No-op; just use HTTP

See also
* [design](./design.md)

# May Someday

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
