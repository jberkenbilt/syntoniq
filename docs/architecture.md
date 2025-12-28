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

# Maybe Someday

* Tuned MIDI output (MTS) mode: we could generate a tuning file (Scala or Tun) based on Syntoniq scale.
* Score printing: we could potentially generate MusicXML or LilyPond if we included additional metadata.
