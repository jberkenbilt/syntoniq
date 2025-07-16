2025-07-13

Next tasks, in undefined order, are:
* Build the whole pitch/scale/layout engine.
  * initialize closest_just_interval (bookmark 0)
  * translate layout to mapping from keys to notes (bookmark 1)
* Update play_midi to send a note just based on scale degree: 60 ± steps from middle C. This should work with Scala files loaded into Surge-XT. See comments in midi_player.rs. This doesn't need to be its own command. For `run`, we can have --midi as a flag that overrides the default (csound).
* Do the actual csound integration
* Get the web UI up

See also
* [design](./design.md)
