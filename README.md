2025-07-19

Next tasks, in undefined order, are:
* Get the web UI up
  * Download htmx locally
* Vertical arrow keys (70, 80) shift pitch up and down by a cycle (or octave if no cycle)
* Pitch shift; see below
* The logger might want to track note names in addition to pitches

Other:
* Pitch/scale overflows: handle gracefully for moving up/down octaves
* `shift` key behavior
  * When pressed, toggle
  * When released, if any other key was touched, turn off; otherwise no action
  * Implies shift down, shift up, shift down, key, shift up => effect shift is ignored
* Implement shift to move keyboard or transpose:
  * shift + non-note = reserved for future use
  * shift + (note1, note2)  = move note2 to the position of note1
  * shift + (note1, note1) = move base pitch to pitch of that note in the octave closest to it
  * shift + (note1) = do nothing
  * UI:
    - note1 stays flashing until second note is pressed or shift is released
    - second note cancels shift
  * When we set a scale, we can have an optional shift and transpose that can be used to set the notes. There may need to be a way to persist this so it stays when we switch back to a scale.
* General keyboard layout. Probably don't bother with specific JI/Harmonic layouts
  * define 64-element array of relative pitches and 64-element array of names
* Cabbage with wave form with filter, LFO, maybe stereo, detune, pan knobs as basic example

See also
* [design](./design.md)

Remember:

```
# Find QLaunchPad as output port
aconnect -l
aseqdump -p 128:0
```
