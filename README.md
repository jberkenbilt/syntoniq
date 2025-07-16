2025-07-13

Next tasks, in undefined order, are:
* Implement response to key presses when playing. Add sustain mode.
* Update play_midi to send a note just based on scale degree: 60 ± steps from middle C. This should work with Scala files loaded into Surge-XT. See comments in midi_player.rs. This doesn't need to be its own command. For `run`, we can have --midi as a flag that overrides the default (csound).
* Do the actual csound integration
* Get the web UI up

See also
* [design](./design.md)
