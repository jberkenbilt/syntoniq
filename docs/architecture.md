# Architecture

Noteworthy features:

* Unsafe code in csound as pattern for talking to csound library including Sync/Send and threading in rust instead of C
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
* Start mark, end mark, skip beats, skip repeat are based on final timeline, not token stream. They include interpolation of in-flight changes for precise carving while iterating on arrangements.
* Use midly and midir for MIDI, though there is some manual MIDI code generation as well
* Launchpad: consult Launchpad developer docs. We use programmer mode.
* HexBoard: use delegated control; added in Firmware 1.3 (contributed from this project)

These are mostly documented in the code themselves.
