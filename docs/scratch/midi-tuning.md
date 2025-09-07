ChatGPT(gpt-5-mini/General)> I'm an advanced developer with decades of experience working on low-level systems. I have written mini languages, parsers, and software that consumes or generates binary files. I have been working on a system for microtonal music and have software that generates MIDI notes using two different approaches: MPE (note per channel, note + pitch bend) and something similar to MTS-ESP where I reassign the frequency for each note. These both work in my little demo setup, but I want to do some experimentation with Reaper and some VST plugins as well as command-line tools like timidity and fluidsynth. I have code in rust using midly and midir. That's the background. For this chat, I don't want you to generate code unless I specifically ask.

I'm looking for a tool that I can use to work with MIDI files in a text editor. I'm the author of qpdf, so I will use it for analogy. qpdf provides two different ways to work with PDF in editor: QDF and qpdf JSON. In both cases, you can convert a PDF file into something editable, edit it, and convert it back.  My ideal tool would be something that can export MIDI into something that can be edited in a text editor and round-tripped. The intermediate file format can be anything editable: xml, json, the tool's own format, etc. I use Linux and need this to work in a Linux environment. Do you know of anything that does this?
<shell-maker-end-of-prompt>
Short answer: yes — the classic tool that most closely matches your qpdf/QDF-style workflow is midicsv (CSVmidi). Beyond that there are a few other reasonable approaches (MusicXML via MuseScore or music21, LilyPond’s midi2ly, or one of the JS/Python MIDI→JSON libraries). I’ll summarize the options, the tradeoffs for microtonal work (MPE / pitch‑bend / tuning SysEx), and a recommended path.

1) midicsv (CSV ↔ Standard MIDI file)
- What it is: a CLI tool that converts Standard MIDI Files to a human‑readable CSV text format and back. It is the de facto tool for text‑editing SMF data.
- Why it fits your qpdf analogy: it provides a faithful textual representation of every MIDI event (track, delta/abs time, event type, channel, controllers, pitch-bend, SysEx, meta events) and a round‑trip CLI to rebuild a .mid from the edited text.
- Strengths:
  - Very low level and faithful: you get raw events including SysEx and meta events.
  - Easy to diff/patch in text form and to edit with an editor or scripts.
  - Widely used and available on Linux (package name often midicsv, CSVmidi).
- Caveats:
  - The textual CSV is not a “semantic” musical notation (no beams/voices); it’s event stream level.
  - Round‑trip preserves musical events, but the exact binary encoding (running status, chunk layout, exact delta‑length encoding) may differ. That usually doesn’t matter.
  - If you need to express higher‑level constructs (score layout, parts) you’ll need to edit at the event level.
- Useful for microtonal/MPE:
  - You can edit pitch‑bend events, RPN/NRPN controller events, per‑channel program changes and channel assignment directly.
  - SysEx tuning messages (Standard MIDI Tuning SysEx or proprietary MTS messages) appear as SysEx lines and can be edited or injected.
- Example usage (typical):
```bash
midicsv input.mid > input.csv
# edit input.csv in your editor
csvmidi input.csv > output.mid
```
(Exact binary names vary by distro; the package is commonly called midicsv / csvmidi.)

2) MusicXML via MuseScore / music21
- What it is: convert MIDI → MusicXML (notation‑oriented XML) → edit in text editor / notation tool → export back to MIDI.
- Strengths:
  - If you want a notation view or want to edit pitch spelling, stems, voices, etc., MusicXML is higher level than raw events.
- Caveats:
  - Conversion between MIDI and MusicXML is lossy for performance data: timing nuances, pitch‑bend trickery (MPE) and per‑note pitch variants often get flattened or reinterpreted as microtonal accidentals or nonstandard notation.
  - Not ideal if you want exact pitch‑bend/MPE stream preservation.
- Tools:
  - MuseScore has good import/export and a CLI front-end on Linux; music21 (Python) can convert and manipulate MusicXML and MIDI programmatically.

3) LilyPond’s midi2ly / ly2midi
- midi2ly turns MIDI into LilyPond input (editable text score). ly2midi can generate MIDI back.
- Use case: notation editing where the LilyPond text representation is preferred. Same caveat: performance details and pitch‑bend MPE approaches are not preserved in their original event stream.

4) JSON/JS/Python converters
- There are a few projects that convert SMF → JSON (for use in JS amplifiers like Tone.js, midi-parser-js, midi-json-parser, or python libraries that serialize into JSON).
- Pros: JSON is convenient for programmatic editing (structured representation), good for building custom editors or pipelines.
- Cons: Not necessarily a standard format; tools for converting back to .mid are not as ubiquitous as midicsv; fidelity depends on implementation.
- If you prefer JSON, you can either use one of these libraries/CLIs or write a tiny converter based on a stable MIDI parser (your Rust midly would be great for this if you want a tailored exporter).

5) Rolling your own (Rust with midly)
- Given your background and the midly crate you already use, a small converter that turns MIDI into a stable JSON or YAML representation that you design (with clear fields for SysEx, tempo maps, absolute times, per‑note frequency/cents if you want) is straightforward. That gives you the qpdf-like control: canonical, editable, round‑trip guaranteed by your own writer.
- If you want this, I can help design the JSON schema and the round‑trip rules (no code unless you ask).

Practical recommendations for microtonal / MPE work
- For raw event‑level editing (pitch bends, channel assignment, tuning SysEx): start with midicsv. It’s the simplest, battle‑tested, and gives you exact access to the events you need to tweak for MPE or per‑note pitch‑bend schemes.
- If you need a more structured representation (JSON) for writing scripts to reassign channels, recompose pitch‑bend streams into per‑note frequency overrides, or generate complex SysEx tuning dumps, consider:
  - Using an existing JS/Python MIDI→JSON tool, or
  - Implementing a tiny exporter in Rust (midly) to a schema you control.
- For notation or score edits: use MusicXML (MuseScore, music21), but do not expect perfect preservation of MPE/pitch‑bend tricks.

A couple more practical tips
- Keep the tempo map and timebase in mind: midicsv shows delta times; editing absolute/relative timing is possible but be explicit about ticks per quarter when interpreting CSV.
- When editing pitch‑bend for microtonal notes, watch for:
  - RPN/NRPN messages that set pitch‑bend range (these are events you must preserve).
  - Channel assignment: MPE depends on channel per note; midicsv makes channel numbers explicit.
  - Tuning SysEx: standard MTS SysEx and proprietary SysEx show up in the CSV and can be injected.
- Use version control on the text representation. That’s the big win: diffs, patches, and automated transformations.

If you want next steps:
- I can walk through a sample MIDI file and show how specific MPE / per‑note pitch‑bend sequences look in midicsv and how to modify them for MTS‑ESP-style reassignment.
- If you prefer JSON as the intermediate, I can propose a JSON schema and show how to map MIDI concepts (absolute times, note-on/off, pitchbend, controllers, SysEx) into it and back — and we can discuss edge cases for round‑trip fidelity.

Which of these approaches would you like to explore first? Midicsv example CSV layout and the mapping for pitchbend/SysEx, or a proposed JSON schema for a round‑trip exporter?

ChatGPT(gpt-5-mini/General)> midicsv fits the bill. Your suggestion of showing MPE -> MTS-ESP is a great place to go next. Let's take a simple midi file that just plays middle C followed a beat later by a note that's one degree up in a 17-EDO scale. One note up in 17-EDO from middle C is 70.6 cents, so with MPE, that would likely be represented as a pitch bend down from note 61, where, if my understanding is correct, MTS-ESP would most likely just call it note 61 and change the tuning for note 61 to be the right value.
<shell-maker-end-of-prompt>
Great — below I’ll show two minimal midicsv (CSVMIDI) examples you can paste into a .csv and round‑trip with midicsv/csvmidi.

Notes/assumptions
- Middle C = MIDI 60.
- 17‑EDO step = 1200 * log2(17)/17 ≈ 70.588235... cents above C.
- To minimize pitch‑bend magnitude we use the nearest semitone (C# = 61). The target is +70.588 cents above C, which is −29.4118 cents relative to C# (so a pitch‑bend DOWN from note 61 by ~29.412 cents).
- Pitch‑bend math (14‑bit, center = 8192, symmetric ±8192):
  - desired semitone offset (relative to the chosen MIDI note) = −29.4118 cents = −0.294117647 semitones
  - if pitch‑bend range R = 2 semitones (typical MPE setting), normalized = Δsemis / R = −0.1470588235
  - 14‑bit value = round(8192 + normalized * 8192) ≈ round(8192 − 1204.7059) = 6987
- midicsv uses 0‑based channel numbers (channel 0 = MIDI channel 1).

1) MPE-style: two channels, second note is note 61 with a pitch‑bend applied
- Here we explicitly send an RPN sequence (to set pitch‑bend range = 2 semitones) on the note's channel, emit the pitch‑bend value, then Note_on for that channel. (You can omit the RPN if the synth already has the desired range.)

Save this as mpe.csv and convert with: midicsv mpe.csv > mpe.mid

```text
0, 0, Header, 1, 1, 480
1, 0, Start_track
1, 0, Title_t, "MPE example"
1, 0, Time_signature, 4, 2, 24, 8
1, 0, Tempo, 500000
# --- Channel 0: middle C (note 60) no bend ---
1, 0, Program_c, 0, 0
1, 0, Note_on_c, 0, 60, 90
1, 480, Note_off_c, 0, 60, 0
# --- Channel 1: 17-EDO step realized by playing MIDI note 61 and pitching down ~29.4118 cents ---
# Set RPN 0 (pitch-bend sensitivity) on channel 1 to 2 semitones (Data Entry MSB = 2, LSB = 0)
1, 0, Control_c, 1, 101, 0
1, 0, Control_c, 1, 100, 0
1, 0, Control_c, 1, 6, 2
1, 0, Control_c, 1, 38, 0
1, 0, Control_c, 1, 101, 127
1, 0, Control_c, 1, 100, 127
# At time=480: set pitch-bend on channel 1, then play note 61
1, 480, Pitch_bend_c, 1, 6987
1, 480, Note_on_c, 1, 61, 90
1, 960, Note_off_c, 1, 61, 0
1, 960, End_track
```

Comments:
- The Pitch_bend_c value 6987 is the computed 14‑bit word (center 8192) for the −29.4118 cent bend with a ±2‑semitone PB range.
- MPE conventionally assigns each note to its own channel; I used channel 0 and 1 here simply to illustrate two independent channels. In a real MPE patch you’d use channels 2–16 (or per your MPE zone).

2) MTS‑ESP / MIDI Tuning Standard approach: play note 61 with no pitch‑bend, but change (or load) a tuning that maps key 61 to the required pitch
- Here the idea is to send a MIDI Tuning Standard (MTS) SysEx that either:
  - sets a single‑note tuning (single‑note tuning change) for MIDI key 61, or
  - uploads a full tuning program / bulk tuning dump that encodes the entire 17‑EDO mapping (recommended if many notes need remapping).
- midicsv will show the SysEx as a System_exclusive line with the raw bytes. The exact SysEx byte sequence depends on the MTS message variant (RealTime vs Non‑RealTime, Single Note vs Bulk), and on whether the synth accepts frequencies expressed as cents, absolute Hz, or a fixed 3‑byte format from the spec. I’ll show a placeholder SysEx line and explain what to fill in.

Save this as mts_esp.csv and convert with: midicsv mts_esp.csv > mts_esp.mid

```text
0, 0, Header, 1, 1, 480
1, 0, Start_track
1, 0, Title_t, "MTS-ESP example (conceptual)"
1, 0, Time_signature, 4, 2, 24, 8
1, 0, Tempo, 500000
# --- SysEx: Single-note tuning change or bulk tuning program that maps key 61 to +70.588235 cents relative to C
# (Replace the bytes below with a properly formatted MTS SysEx for your synth.)
# Example conceptual placeholder (not actual spec bytes):
1, 0, System_exclusive, 0, 240, 126, 0, 1, 8, 2, 0, 1, 61, <tuning-bytes-for-+70.588c>, 247

# Now play MIDI note 61 on a single channel (no pitch bend)
1, 0, Program_c, 0, 0
1, 0, Note_on_c, 0, 61, 90
1, 480, Note_off_c, 0, 61, 0
1, 480, End_track
```

What to do next for a correct MTS SysEx
- If you want a concrete, synth‑compatible SysEx payload I can produce the exact bytes for either:
  - Single Note Tuning Change (Non‑Real‑Time) [Universal Non‑Realtime: 0xF0 0x7E ... 0x08 0x02 ... F7], or
  - Bulk Tuning Dump (set the whole 128‑key map), or
  - Realtime variants (0x7F instead of 0x7E).
- For each option I’ll need to know:
  - target (device ID in SysEx: e.g., 0x7F = all‑call / some synthesizers accept this),
  - which MTS format your target VST/synth accepts (single note vs bulk; whether it expects frequency in "decimal frequency" format or the 3‑byte MTS format),
  - whether you want tuning expressed as cents or absolute Hz (some forms use frequency in Hz encoded as a 3‑byte 21‑bit fraction of a reference).

Practical notes / tradeoffs
- MPE approach:
  - Pros: works with any synth that responds to pitch‑bend; straightforward with MPE‑aware instruments.
  - Cons: per‑note channel assignment complexity, limited number of per‑note channels (16 channels total), requires managing pitch‑bend range and RPN messages per channel, pitch‑bend resolution is finite (14‑bit).
- MTS‑ESP approach:
  - Pros: you can play the same MIDI note number and change the tuning mapping for that note (no per‑note channels needed); better semantic match for scale-based retuning (upload a 17‑EDO map once and play normally).
  - Cons: requires the synth/VST to support the MIDI Tuning Standard or a vendor‑specific SysEx tuning API (not all do). Different synths accept different SysEx formats—some accept Scala files, some accept MTS bulk dumps, some don't accept any tuning SysEx at all.
- For DAW and plugin testing:
  - Reaper: many plugins will respond if the plugin itself implements MTS or Scala import (or if the plugin exposes its own retuning API). Reaper does not itself do MTS conversion; it simply passes SysEx through.
  - timidity/fluidsynth: support depends on the synth engine / soundfont. fluidsynth has some support for Scala (via the –scaletempo? actually it has a --midi-tuning or --load-scl option in some versions); check the fluidsynth version and features.

If you want, next I can:
- produce an exact, spec‑correct SysEx byte sequence for:
  - single‑note tuning change for key 61 to +70.588 cents (or),
  - a full 17‑EDO 128‑key bulk dump (so you can retune the whole keyboard),
  - or a tiny Rust routine (pseudocode or actual code if you ask) that emits the correct MTS SysEx bytes from cent values (useful since you already use midly).
Which of those would you like me to generate?

ChatGPT(gpt-5-mini/General)> Can you educate me a little on the MTS-ESP SysEx command semantics? You mentioned single-note vs. bulk and real-time vs. non-real-time. You can either give me a high-level, generic description of these, or if you prefer, I can describe my intended application/workflow. Note that I have decades of architectural experience and I am not seeking architectural suggestions (e.g. if you're doing x, you should do y) as I can work that out on my own. What I want to understand is what the expressive capability of MTS-ESP is. I'm not worried about trying to be compatible with things. I can pick my plugins and tools around the desired capability. For example, using Reaper with Pianoteq will definitely support MTS-ESP on Linux. I'm not sure about Numa from Studiologic. I'm hoping I can find some low-cost or free VST instruments that support MTS-ESP if I decide to go that route. Anyway, I thinking getting you to describe the range of things MTS can do without knowing my application or workflow may contain less bias. I don't need help on MPE -- I have a fully working implementation.
<shell-maker-end-of-prompt>
Good — I’ll focus on the expressive capability of the MIDI Tuning Standard (MTS / “MTS‑ESP” as people often call the family of SysEx variants) in practical, un‑opinionated terms: what you can express, the different message families, how they scope and persist, and the typical numerical encodings you’ll encounter. I’ll avoid low‑level byte recipes here (I can produce those on request) and concentrate on what the standard lets you describe and control.

Summary up front
- MTS lets you retune MIDI keys (0–127) in several ways: tuning a single key, uploading a full 128‑key map, defining a repeating scale/octave pattern, and selecting/activating tuning programs/banks.  
- There are two transport flavors: Non‑Real‑Time (for file or editor style bulk uploads and storage) and Real‑Time (for immediate live updates). Conceptually they do the same jobs but are handled differently on the receiver.  
- You can express tuning as absolute frequency (Hz) or as offsets (cents) relative to some reference; implementations usually support high precision (micro‑ or sub‑cent practical behavior depends on device).  
- MTS tuning tables are global to the receiving synth/device (not per MIDI channel). If you need per‑channel per‑note tuning you’d typically combine MTS with routing (multiple instances) or use per‑voice retuning built into the synth.  
- For programmatic use (e.g., 17‑EDO), you either (A) upload a scale definition that is mapped/repeated across the keyboard, or (B) construct and upload a full 128‑key map (bulk) — both are supported by the standard.

Message families and their semantics
1) Single‑Note Tuning Change (one key at a time)
- Purpose: change the pitch of a single MIDI key number (0–127).
- Use cases: occasional microtuning tweaks, live retuning of individual notes, quick per‑key correction.
- Behavior:
  - Targets a specific key number and provides the desired pitch for that key.
  - Can be encoded as an absolute frequency or as a cents offset (depending on receiver support).
  - Two transport modes: Real‑Time (processed immediately) or Non‑Real‑Time (suitable for file/patch programming).
- Persistence/Scope:
  - On many synths, a single‑note change is applied immediately and persists until changed or until a new tuning program is selected; behavior depends on the synth (some clear tuning on program change).

2) Scale / Octave (small table that repeats)
- Purpose: define a scale with N degrees (N can be other than 12) and have it repeat across the keyboard (useful for EDOs).
- Use cases: non‑12 equal temperaments (EDO), custom scales defined by a sequence of cents or ratios, simple Scala‑like definitions.
- Semantics:
  - You send one table of M values (M = number of scale degrees). The implementation repeats (maps) that scale across the full 128 key range according to a specified repeat interval (usually an octave, but the spec permits other mappings).
  - The message can include: reference key (which MIDI key anchors the pattern), scale degree cent offsets, and a description of how to apply the pattern across octaves/keyboard.
- Practical advantage:
  - Very compact for 17‑EDO: you send the 17 degree offsets once and the synth repeats them.
- Caveats:
  - Exact interpretation can vary slightly between implementations (how they treat reference key, coarse mapping rules, etc.), so testing on the target synth is wise.

3) Bulk Tuning Dump (full 128‑key map)
- Purpose: upload a complete tuning table for all MIDI keys 0–127.
- Use cases: arbitrary tuning across the whole keyboard; most general and unambiguous way to describe complex tunings (including scales that don’t map neatly by repetition).
- Semantics:
  - You send an explicit frequency (or offset) for each of the 128 keys.
  - This fully defines the keyboard mapping; no further algorithmic interpretation is needed by the receiver.
- Persistence:
  - Bulk dumps are typically stored as a “tuning program” or replace the active tuning map, depending on the device. Some devices can store named tuning banks, others treat the dump as ephemeral.

4) Program/Bank selection and naming
- Purpose: manage multiple named tuning programs/banks; select which tuning the synth should use.
- Semantics:
  - MTS defines messages for selecting a tuning program or a bank/program pair. Implementations may also allow naming and storing tunings (user memory).
- Workflow:
  - Upload (bulk or scale); optionally store as program/bank; then send a “select tuning program” message to activate it.
- Note:
  - Naming/storing semantics are implementation dependent (how many programs, permanence, whether files are saved to disk, etc.).

Real‑Time vs Non‑Real‑Time: what that means practically
- Real‑Time (Universal Real‑Time SysEx family)
  - Intended for live performance where immediate effect is required.
  - Devices that implement Real‑Time MTS are expected to apply the tuning without breaking timing and may be optimized for low latency.
  - Some receivers restrict the complexity of real‑time messages they accept to avoid voice stealing or glitching.
- Non‑Real‑Time (Universal Non‑Real‑Time SysEx family)
  - Intended for non‑urgent configuration (DAW loading, patching, bulk updates).
  - Often used in MIDI files or during patch load sequences; receivers may treat these as “store” operations.
- Practically: use Non‑Real‑Time for file/bulk uploads (MIDI files, patch data), and Real‑Time when you must alter tuning while the instrument plays.

Numeric encodings and precision
- MTS commonly supports:
  - Absolute frequency (Hz): the most direct and unambiguous representation; some MTS messages encode frequency as a multi‑byte fixed‑point number with good precision (enough for sub‑cent resolution).
  - Cents offsets: provide cent‑scale offsets relative to a reference note (e.g., cents relative to A4/440 or relative to the equal‑tempered mapping). A few MTS variants allow per‑key cent offsets in a compact form.
- Precision:
  - The standard was designed with musical microtuning in mind — sub‑cent resolution is typical in practice (synth behavior depends on internal oscillator resolution).
  - If you need exact formulas: convert cents → frequency via f = f_ref * 2^(cents/1200). For 17‑EDO, a step is ≈ 70.588235 cents; repeated across octaves gives you a deterministic frequency for every key.
- Implementation notes:
  - When you provide absolute frequencies you remove ambiguity. When you provide offsets/cents you depend on the receiver’s anchoring/reference convention.

Scope (global vs per‑channel vs per‑voice)
- MTS applies to the device/instance (global tuning map for the instrument) — not to a specific MIDI channel.
- Consequence:
  - You cannot use a single device instance to host two different tunings simultaneously unless the device supports multiple internal voices/instances or per‑voice tunings—most conventional MIDI synths treat tuning as a single active map.
  - For per‑note differing tunings (like MPE but without channels), you either:
    - use multiple synth instances/tracks each with its own tuning, or
    - rely on synths that support per‑voice retuning internally (some modern VSTs do).
- Pianoteq: as you noted, supports MTS-style maps and per‑instrument retuning, so it can be used as a single synth with arbitrary mappings.

Typical workflows for microtonal/EDO use
- Small number of scales (e.g., testing 17‑EDO across a project):
  - Upload a scale definition or a small tuning program and select it. If the synth supports scale messages, that’s compact and expressive.
- Wide or dynamic retuning:
  - Upload full 128‑key bulk dumps, or generate and upload per‑program maps as needed.
- Live dynamic retuning (rarely used, but possible):
  - Use Real‑Time single‑note or small-scale messages; beware of audible artifacts for actively sounding notes.

What MTS cannot (directly) do
- Per‑MIDI‑channel tuning maps (the standard targets keys, not channels).
- Guaranteed cross‑device semantics for non‑standard extensions (different synths may implement additional features).
- Implicit per‑note voice stealing/voice assignment rules — how a synth reassigns or retunes currently sounding voices when you change tuning is implementation specific.

Examples of representing a 17‑EDO scale
- Option A — scale/octave message: send 17 degree offsets (in cents) and a reference key so the device repeats the 17‑step pattern across the keyboard. Compact and expressive if the synth implements this scale type.
- Option B — bulk 128‑key map: compute absolute frequency for each MIDI key using your reference (e.g., key 69 = 440 Hz) and the 17‑EDO step formula, then upload the full table. Unambiguous and universal.
- Conversion formula (useful when making a bulk table):
  - step_cents = 1200 * log2(17)/17 ≈ 70.588235...
  - If you want key k to correspond to some number of 17‑EDO steps relative to reference key r (e.g., r = 60), then cents offset = (steps_from_r * step_cents) + 1200 * octaves_from_r
  - frequency = 440 * 2^((midi_key_distance_in_cents + key_specific_offset)/1200) where midi_key_distance_in_cents is the equal‑tempered cents for the key relative to A4 and key_specific_offset is your 17‑EDO offset relative to equal temperament, or compute directly by counting EDO steps and octaves.

Implementation/compatibility notes (practical)
- Most DAWs (including Reaper) simply pass SysEx through to VSTs; the VST must implement MTS or accept Scala files or equivalent import. Pianoteq, some Kontakt instruments, and a few free synths support it; many sample‑based SF2/SFZ engines do not unless they explicitly include Scala support.
- For offline work (MIDI files), Non‑Real‑Time bulk dumps are commonly used to embed tuning in the file; receivers that honor MTS will apply the embedded tuning when they load the file.
- Some modern plugins accept Scala (.scl/.kbm) directly, which is often more convenient than raw SysEx; but Scala → MTS translation is straightforward if you need to send SysEx.

If you want exact, spec‑correct SysEx payloads next
- I can generate the precise SysEx bytes for:
  - a Single‑Note tuning change that sets MIDI key 61 to a specific frequency or cents, or
  - a Scale/Octave definition for 17‑EDO (compact message), or
  - a Bulk Tuning Dump for the full 128‑key map for a specified reference (e.g., A4=440 Hz), and
  - variant bytes for Real‑Time vs Non‑Real‑Time universal SysEx headers and for device ID (all‑call vs specific).
- Tell me which payload you want (single key / scale / bulk), whether you prefer real‑time or non‑real‑time, and whether you want cent offsets or absolute Hz. If you want absolute Hz, tell me the reference mapping rule (e.g., key 69 = 440 Hz). I’ll produce the exact bytes and note any implementation caveats (e.g., how to format the multi‑byte frequency field).

ChatGPT(gpt-5-mini/General)> Okay, this is helpful. My microtonal project is venturing into what appears to be somewhat novel territory. I have developed a lossless pitch notation using factors of ratios or (a/b)^(c/d) and the ability to chain tuning systems, e.g., dynamically specify a 17-EDO scale whose root note is some arbitrary degree of a different scale. I've been exploring microtonal music, and most microtonal music picks a scale and sticks to it. I think it would be fascinating to play around with "modulating" from one scale to another. To me, many of the different EDOs have different flavors -- especially when it comes to whether certain intervals are a little sharp or a little flat -- and I think it would be fun to play around in that space. None of the existing systems I've seen for representing scales and pitches lend themselves to this kind of experimentation. Another application would be experimenting with Just Intonation but shifting it around different key changes. My system allows you to create a simple scale with specific ratios, and then to transpose it dynamically, preserving semantic information. For example, you could have a piece that shifts around to different "keys" in a tuning system like 17-EDO, modulating by the 2-step or 4-step interval and then do something with JI based on whatever step of 17-EDO you're in.

I have this all working (very prototype-y, but working) using csound as a backend where I bypass MIDI entirely. My system, being lossless, does not accumulate floating point rounding errors but rather resolves a note to a specific frequency at the last moment. It also canonicalizes pitches so you can tell that two pitches are the same even if you arrived at them through different paths. Transpose up 5 steps in 17-EDO and down 10 in 34-EDO? My system will know you're back where you started. Using csound, there are no tuning, limited channels, or any of the other headaches of MIDI. Everything is perfectly smooth. I can generate as many instruments and voices as I want, and I can play as many notes as I want in any tuning system.

The downside of all this is that csound, while an amazing system, is not very non-programmer friendly (fine with me -- I do everything with code)....but more fundamentally, it's not easy to do human expression with it. I've been envisioning a workflow where I would use my system but, instead of generating csound, I would generate some kind of MIDI product that could be loaded into a DAW and manually tweaked. My system is fully expressive in terms of pitch, but it is quite minimal in terms of instruments, control messages, etc. Expanding my system to support these things is a non-goal. Instead, I'd rather be able to generate something that can be loaded into a DAW and tweaked manually.

What I envision is generating separate "MIDI items" for each region of music in a particular tuning system. Let's take a simplistic example: say you have a harmonic passage based in 17-EDO with a simultaneous melodic passage in 7-EDO. For this, what I imagine is generating two separate MIDI items, each with its own tuning map. The advantage of this over MPE is that you could load these as two separate tracks in Reaper and then revoice them, modify individual notes, change instrumentation, add effects, etc. This kind of manual workflow would be likely intractable with MPE, but with MTS-ESP, you could work with notes in the local scale, and by generating completely separate MIDI items that are intended to be mixed in the DAW, I think you can get the desired effect.

Are you tracking so far? There's more to come.
<shell-maker-end-of-prompt>
Yes — I’m tracking, and this is a neat direction. Short summary of what I understand, the key technical implications for using MTS in your workflow, and a few practical options and questions to guide the next step.

What I understand you want
- You have a lossless, canonical pitch representation (ratios / exponentials / chained tuning systems) and can resolve any pitch to an exact frequency at the last moment.  
- You want to generate MIDI “items” that can be imported into a DAW (Reaper) for manual editing, mixing, and effect processing.  
- Each MIDI item represents a region whose notes are defined in a particular tuning system (e.g., 17‑EDO, 7‑EDO, JI derived from a scale degree of another scale).  
- You want each region/track to carry its own tuning map so the DAW/plugin plays the notes at the correct frequencies while you keep the ability to edit notes/velocities/etc. in the DAW.  
- You’re happy to pick plugins that fully support MTS (Pianoteq works for you) and you plan to use separate instrument instances/tracks to keep tunings independent.

Key MTS implications and recommended patterns
- Scope: MTS tuning is global to the receiving synth instance (not per MIDI channel). Therefore:
  - To have multiple simultaneous, different tunings, route each MIDI item to a separate instrument instance (separate plugin instance or synth instance) in Reaper. This matches your plan to generate separate MIDI items/tracks.
- Best transport for your use case:
  - Non‑Real‑Time Bulk Tuning Dump (a full 128‑key map) is the most general and unambiguous way to represent arbitrary tunings produced by your system (including chained systems and mixed JI/EDO). It’s compact enough to embed in the MIDI item/file and guarantees the synth knows the exact frequency for each key.
  - Scale/Octave (repeating small table) messages are compact and efficient for pure repeated EDOs (e.g., 17‑EDO); use them if your target plugin implements the repeat‑scale message and you want the compactness/semantic clarity of a degree list rather than 128 explicit entries.
  - Single‑Note messages (real‑time or non‑realtime) are useful for local, punctual changes, but for your chaining/modulating schemes a bulk map or a scale message will be more straightforward and less fragile.
- Where to put the SysEx:
  - Embed the Non‑Real‑Time SysEx bulk/scale as the very first MIDI event(s) in each MIDI item (time 0 of the item). That ensures the plugin instance sees and installs the tuning before Note_on events in that item.
  - Use the Non‑Real‑Time (Universal Non‑Real‑Time) MTS header for file/patch loading semantics; many plugins treat Non‑Real‑Time messages as configuration/load-time data and will install them when the MIDI item is loaded.
  - If you need dynamic retuning inside an item (mid‑item modulation), use Real‑Time single‑note or small-scale messages at precise timestamps — but beware audible artifacts if notes are held during retune.
- Numeric representation:
  - For absolute and lossless fidelity from your pipeline, prefer absolute frequencies (Hz) in the MTS bulk dump. That’s unambiguous and avoids reliance on receiver anchoring rules.
  - If you use scale messages for EDOs, represent degrees in cents relative to a reference; those are compact and semantically clear.
- Round‑trip / metadata preservation:
  - MTS encodes absolute tuning, but not your high‑level canonical expressions (ratios, chained definitions). If you want round‑trip editing back into your system, embed your canonical representation as metadata in the MIDI item too:
    - Put a Text or Lyric meta event (or a maker‑specific SysEx blob) at the start of the item containing a canonical textual serialization of your ratio/chaining spec. Reaper keeps MIDI item text and the meta events travel with the item in a Standard MIDI file export, so you can re-import that metadata when you want to reconstruct the exact lossless definition.
    - Alternatively, use a private vendor SysEx with a compact binary serialization of your representation; advantage: exact and compact. Disadvantage: you must arrange the tooling to parse it out later.
- DAW/plugin startup/load semantics:
  - Some plugins only process SysEx during initialization or when specifically told to “read tuning” — test your chosen VSTs. Pianoteq is known to accept MTS/Scala reliably; many others vary.
  - When loading the MIDI file or moving items around, make sure the plugin gets the SysEx before note playback. Embedding the SysEx at time 0 of the item is the usual practical approach.

Practical trade-offs and why bulk is often best for you
- Your pipeline can compute arbitrary, lossless frequencies. Bulk dumps let you express that exact mapping for each MIDI key; recipient synths will reproduce the intended frequencies precisely (subject to synth resolution).
- Scale messages are elegant for EDO/periodic systems and save space, but chaining arbitrary scale transforms (e.g., dynamic root mapped to a different degree in another scale) may be easier to export as an explicit 128‑key bulk map for each region, because it avoids relying on device-specific scale-application semantics.
- Embedding metadata is essential if you care about round‑tripping to your canonical system (ratios, chained rules). MIDI alone won’t preserve your algebraic descriptions unless you deliberately put them into meta events or SysEx.

Questions / choices to make (I can help with any)
- Do you prefer to embed absolute Hz in bulk dumps (preferred for losslessness) or to use a compact scale message when the region is a simple repeating EDO? (I can produce both types of exact SysEx payload examples.)
- How will you structure items in the DAW: one item → one track/plugin instance (my assumption), or multiple differently‑tuned items routed to the same plugin instance? (If the latter, we need to discuss per‑voice plugin behavior and likely impossibility without multiple synth instances.)
- Do you want the canonical representation embedded in a human‑readable text meta event (easy to inspect/edit) or as a binary private SysEx chunk (compact and exact)? I can recommend formats and show how to embed them in midicsv.
- Do you want to see a concrete midicsv example showing:
  - a Non‑Real‑Time bulk dump (absolute Hz) followed by note events for a MIDI item, and/or
  - a compact scale‑message for a 17‑EDO item, and/or
  - how to include your canonical textual metadata as a Text meta event in the same item?

Next step
Pick one (or tell me which combination):
- Produce exact SysEx bytes (Non‑Real‑Time bulk with absolute Hz) plus a midicsv snippet that shows embedding it at the start of an item; or
- Produce a compact MTS Scale/Octave message for 17‑EDO plus midicsv; or
- Describe a metadata serialization for your canonical ratio/chaining representation (text or binary) and show how to embed it in the MIDI item for round‑trip.

Which would you like first?

ChatGPT(gpt-5-mini/General)> You mentioned dynamic retuning, which is where I was going next. I hadn't considered round-trip as a goal. That's an interesting thought. I was more thinking people would work out the notes using the csound backend. But round-trip is interesting...I'm not sure I would make that a goal right now. I'd have to see what is preserved by, say, Reaper's MIDI editor. But I think I'm okay with having to make a note change in the original and send it through. The non-destructive editing I think should make this possible, but I'll have to try some things.

I have the concept of being able to morph from one tuning system to another. Surge-XT can do this. There's a specific piece I have in mind: Fabio Costa's "Etude on Difference Tones: Minor Thirds". I know for a fact (I have talked with him and he documents it in his YouTube page) that he produced this using Reaper and Surge-XT. A fundamental, core property of that piece is the dynamic retuning, and the notes in that piece are not based on any scale. They are arithmetically spaced JI ratios. I don't know exactly how he created that piece, but I want my notational system to be rich enough to generate that. I haven't integrate it, but I have a stand-alone csound file that I made that will morph a note from one pitch to another following an envelope. Anyway, I have listened to that piece, and I know what software was used to create it, so I know what I have in mind is doable. I know that some of the microtonal music I have listened to (I lean contemporary classical in my taste) was performed and edited.

A common workflow for microtonal composers these days is to play the music on a lumatone and then do additional work in the DAW. I know there are lots of lumatone mappings for keys. The lumatone has more than 128 keys, so it can't be purely MIDI based. Do you know what the lumatone sends to the DAW? Being a performance instrument, it seems to me that MPE would be a better fit. If the lumatone limits its MIDI output to 15-note polyphony, the lumatone software can just track which notes are on and keep a channel mapping...this is what my existing MPE software does. I coded a microtonal keyboard based on Launchpad MK3 Pro. It's a good study keyboard, but it's not really a good performance keyboard. But that's what it does -- it just tracks which pitches are down and keeps a mapping of pitch bind to channel, discarding when a pitch bend is no longer in use. This works okay, but both Surge-XT and Vital have bugs in processing these. In both cases, it's possible to reproduce an issue where you send three distinct pitches on channels, 2, 3, and 4 with distinct pitch bends, and sending a note on channel 4 causes the note on either channel 2 or channel 3 to change. Surge-XT and Vital both have this bug, though the pitches change differently. So anyway, MPE seems like a good solution for an instrument that someone is playing, but MTS-ESP seems like a better fit for my batch mode work. Prior to my traversing down this path, my workflow was lilypond -> midi, and I never took it past that. Either it was performed live or else the output of timidity was as good as it got.

Not that it matters, but I'm not a "real composer" -- I've only composed a few very small things that aren't very good. I'm a better arranger, but I'm a professional software engineer and amateur musician, and I'm not plugged into anything where anything I create would be performed if I didn't do it myself. So this may or may not actually go anywhere...I'm just playing with ideas. I've been chipping away at this project on weekends. I think it would be fun to just get these ideas out there. I don't know how much of this I will build. I am definitely planning on building out the rest of the csound part. How far I go down the MIDI path is an open question.

Anyway, do you know what lumatone generates? Do you know what kind of workflow you would have editing a MIDI file with pitch bend instructions in Reaper vs. editing one with regular notes and an alternative tuning system loaded?
<shell-maker-end-of-prompt>
Short answer up front
- I don’t have a canonical, model‑unambiguous specification for “the Lumatone” in my current knowledge; different firmware/editor versions and modes can emit different things (MPE, plain MIDI note maps, SysEx from the editor, or multi‑port/multi‑channel schemes). I wouldn’t confidently assert one exact behavior.
- You can determine exactly what it emits by monitoring its MIDI output with a MIDI monitor on Linux; I’ll give precise commands and what to look for.
- Workflow comparison (editing pitch‑bend event streams vs. using MTS SysEx + ordinary note numbers): each has clear tradeoffs I’ll summarize and give practical experiments to try so you can choose how to generate/export from your system.

How to find out what the Lumatone actually sends
1) Tools (Linux)
- aplaymidi / arecordmidi / amidi / aconnect / aseqdump / amidi
- Useful GUI/CLI monitors: kmidimon, qmidinet, joel's midimon, or simply aseqdump (ALSA) for event view and amidi for raw dumps.

Quick command examples (pick one):

- List ports:
```bash
aconnect -l
amidi -l
```

- Monitor MIDI events in a readable stream (ALSA sequencer):
```bash
aseqdump -p "Lumatone:Midi Out"   # replace with the port name from aconnect -l
```

- Dump raw SysEx bytes from a raw device:
```bash
amidi -p hw:1,0,0 -d > sysex_dump.bin   # replace hw:id with the device path shown by amidi -l
hexdump -C sysex_dump.bin | head
```

- Or record MIDI activity during a simple performance to a file:
```bash
arecordmidi -p "Lumatone:Midi Out" out.mid
# then inspect out.mid with midicsv/csvmidi or a seq editor
midicsv out.mid | less
```

2) What to look for (signatures)
- MPE-style output:
  - Repeated note_on/note_off events on different channels with accompanying pitchbend events (Pitch_bend messages) per channel and occasional RPN/NRPN controller sequences to set pitch-bend ranges. If you see notes on different channels concurrently with pitch-bend control for each channel, it’s likely MPE.
- Simple mapping:
  - Plain Note_on/Note_off messages with different MIDI note numbers, possibly a “keymap” SysEx at startup from the editor that assigns which physical key → which MIDI note.
- Extended mapping (more than 128 unique physical keys):
  - The device may map its many physical keys down to the 0–127 MIDI range using multiple layers (shift layers) or use its editor/driver to stream SysEx definitions of layouts rather than sending unique note numbers directly. The editor might translate extended layouts into sequences of MIDI on/off using only standard note numbers.
- SysEx and editor traffic:
  - On startup or on layout change, look for System_exclusive messages (0xF0 … 0xF7). Those are often editor upload/downloads (mappings, presets, etc.) and may contain the mapping for the physical grid. If present, capture it — that’s the device’s exact format for mappings.
- Channel/port strategies:
  - If more than 128 logical keys are required, some devices:
    - Use multiple MIDI ports or device IDs (so the host sees different ports with the same note number ranges),
    - Use multiple channels and layer mappings (e.g., different ranges on different channels),
    - Use SysEx to forward custom messages (nonstandard).

If you want, capture 10–30 seconds of typical playing and paste the midicsv or hexdump here; I can interpret the exact behavior and advise next steps.

Practical comparison: Editing pitch‑bend (MPE/per‑note bend) vs. editing ordinary notes with MTS tuning maps
I’ll compare them in categories you’ll care about: editability, DAW behavior, plugin behavior, mixing, morphing, and preservation.

A) Editor ergonomics / note editing
- Pitch‑bend per‑note (MPE)
  - What you see in DAW: notes are normal Note_on events, but pitch is implemented via pitch‑bend events per channel/voice. Most DAWs show pitch‑bend as an automation lane or CC lane.
  - Editing implications:
    - If you move a note, you must also move the pitch‑bend events precisely to keep the intended pitch. Many editors do not automatically keep pitch‑bend glued to Note_on (some do for MPE-aware lanes, but behavior varies).
    - If you quantize/resize/duplicate, beware: pitch‑bend segments can get left behind or duplicated incorrectly.
    - Complex per‑note pitch envelopes are possible (you can shape glides, vibrato, etc.), but editing them in a piano-roll-like interface can be fiddly unless the DAW has explicit MPE-aware features.
- MTS (tuning table + ordinary note numbers)
  - What you see in DAW: notes are ordinary MIDI note numbers at the pitches the synth maps via the tuning table; no special per‑note bend events required.
  - Editing implications:
    - Moving notes or changing pitch in the piano roll simply changes the MIDI note number — no associated pitch-bend events to manage. This is the most editor-friendly approach for conventional DAW workflows.
    - If you change the tuning map you can globally affect many notes without touching per-note automation.

B) Reliability and plugin behavior
- Pitch‑bend approach:
  - Pros: works on any synth that accepts pitch‑bend; expressive continuous motion possible.
  - Cons: depends on correct channel assignment & PB range; some synths have bugs (you noted Surge-XT and Vital issues). MPE requires the plugin to be MPE-aware for best editing experience (some treat pitch bend as global or share behavior across channels).
- MTS approach:
  - Pros: if plugin supports MTS/Scala/bulk tuning, the pitch mapping is handled internally and reliably for all notes. No per-note envelope artifacts when editing notes.
  - Cons: plugin must implement tuning SysEx/Scala support. Retuning a playing voice depends on the synth’s implementation — some synths instantly retune pitch oscillators, others may produce glitches or not retune existing held voices.

C) Dynamic retuning / morphing
- Pitch bend (MPE)
  - Smooth continuous morphs are straightforward: interpolate pitch‑bend values over time per note or per controller, and the synth will produce a continuous frequency sweep (subject to pitch‑bend resolution and synth processing). This supports continuous slides and per‑note pitch envelopes well.
  - MPE is the natural fit for frequent per‑note continuous interpolation during performance.
- MTS dynamic retuning
  - MTS provides real‑time Single‑Note or Bulk messages; whether morphing works smoothly depends on the synth:
    - Many synths apply the new tuning to newly triggered notes but do not smoothly interpolate already sounding voices to the new frequency (they either retune oscillator tables, leading to a step, or they might support internal smoothing).
    - Some synths plug in “morph tuning” features (e.g., Surge‑XT can morph) and will interpolate between tunings if they were implemented to do so.
  - For smooth morphs with MTS, you need a synth that either:
    - Accepts continuous parameter interpolation between two tuned states, or
    - Is willing to accept fine-grained incremental tuning updates (small, frequent bulk or single‑note updates) and internally smooth changes.
  - Practical hybrid: use MTS for base static maps and use MPE-like controllers for micro dynamic sweeps you want to smoothly interpolate.

D) Project maintainability / round‑trip and DAW behavior
- MPE:
  - DAW MIDI items contain many pitch‑bend events. Copy/paste and editing can be fragile if the DAW doesn’t treat the pitch-bend data atomically with note events.
  - Exporting/importing between DAWs can expose compatibility issues (some strip or quantize pitch bends).
- MTS:
  - Embedding a Non‑Real‑Time SysEx bulk dump at item/track start is usually more robust: it’s a single place to set the tuning and then notes are plain note numbers. However, behavior depends on whether the DAW retains SysEx attached to the item and whether the plugin processes the SysEx. Reaper tends to pass SysEx to plugins and keeps SysEx meta events in takes, but testing is essential.
  - If you want subsequent round‑trip editing back to your canonical ratio data, you must embed that metadata (e.g., a text meta event or private SysEx) because MTS only conveys frequencies, not your lossless chain descriptors.

E) Instrument selection and parallel tuned material
- If you want multiple simultaneous, different tunings (your 17‑EDO harmony vs 7‑EDO melody), with MTS you’ll typically use separate plugin instances, each loaded with its own tuning map (this is exactly your plan). This is straightforward in a DAW: separate tracks, separate plugin instances, separate inputs. With MPE you might be able to use one instance if it supports multi‑timbral per‑voice retuning (rare), but in practice multiple instances are easier and more predictable.

Practical experiments I recommend you run (fast, low effort)
1) Snapshot the Lumatone output while you play single notes, glissandi and layout changes. Paste capture here (midicsv or hexdump) and I’ll decode it for you.
2) In Reaper:
   - Create two tracks, load the same plugin twice (e.g., Pianoteq or Surge-XT if available).
   - On track A: send a non‑real‑time bulk tuning SysEx (I can generate exact bytes) at the start and import a simple note sequence (midicsv) that uses note numbers for 17‑EDO.
   - On track B: send the same sequence but encoded as MPE (separate channels + pitch bends), and compare editing experience, playback fidelity, and plugin reaction.
3) Test dynamic retuning:
   - Generate a small MIDI item that sends a Non‑Real‑Time bulk tuning at t=0, plays a sustained note, then at t=1s sends another real‑time single‑note or bulk update to a new tuning that shifts the note. Observe whether the plugin glides, snaps, or produces artifacts.
   - Repeat with pitch‑bend interpolation for the same note (MPE approach) and compare smoothness.

If you want help generating assets
- I can produce:
  - Exact Non‑Real‑Time bulk MTS SysEx bytes for absolute-frequency bulk dumps (e.g., fill a 128‑key table from your resolver given a reference like A4=440 Hz).
  - Small example midicsv files that embed the SysEx and a few notes (so you can csv→mid and import into Reaper).
  - An MPE-style midicsv example with per‑note channel assignment and pitch‑bend envelopes for the same musical material so you can compare.

Closing / offer
- If you capture a few seconds of Lumatone output (either an arecordmidi file, aseqdump output, or a raw SysEx hexdump) I will decode it and tell you exactly which mode it’s using and how you might best integrate it with your pipeline.
- If you want to experiment with the MTS route now, tell me if you prefer bulk dumps in absolute Hz (recommended for losslessness) or scale‑style messages (compact for EDOs) and I’ll generate a concrete midicsv example for you to load into Reaper.

ChatGPT(gpt-5-mini/General)> Alas, I don't have a lumatone. This gives me what I need though, so let's switch gears. I'm looking at midi files that were generated by lilypond. They're pretty minimal. I notice that channel 0 is used only for the header, and that each separate voice gets its own track. Actually, track 1 just has metadata, time signature, and tempo, then the voices start at track 2 (using zero-based numbering). I know that, with MPE, the convention (rule?) is 1 note per channel, but it seems to me that you could continue to have multiple notes in a channel at the same time as long as they had the same pitch bend. Anyway, this is what my MPE implementation does, which works well for playing two notes in octaves, for example, or would be useful if you were in any EDO that was multiples of 3 and 4. I also notice that this midi file doesn't have any note off events: it just uses note on events with velocity of 0 to turn off notes. I would probably organize my midi files more or less the same way: bulk tuning tables at the beginning, then essentially one channel per "instrument". Now I'm a little unclear on this use of channel 0 and channel 1, and actually, in general, I'm not that clear about certain things like zones and sequences. Most likely I don't have to care about this for my purposes as I'm not using MIDI with physical instruments...I'm just using it as a way of getting notes into the DAW. There's also the term "transport", which I also see used with audacity. MIDI clearly has a "mental model" with it based on physical instruments and 12-tone tuning. I know it's a decades-old system. My impulse would be to stick with note-on-velocity-0 to mean off (this is actually what the launchpad sends as well, so it must be reasonably standard) and one track per voice allowing chords within a single voice. If I were to import a midi file into Reaper and want to add things like pedal or instrument-specific control messages, would these typically be on channel 0? So am I best staying out of channel 0? I could follow lilypond, using 0 only for header, 1 for tempo and time information, and 2 through 16 for notes. What's the expectation here?
<shell-maker-end-of-prompt>
Good questions — there’s a lot of historical baggage and a few gotchas that trip people up. Below I’ll separate concepts (tracks vs channels vs meta events), point out common conventions (and why they exist), explain MPE specifics (zones/master vs per-note channels), and finish with concise recommendations tailored to your workflow (MTS per-track tuning maps, simple DAW-friendly MIDI, etc.).

Key distinctions and common confusions
- MIDI channel (0–15 in raw bytes; usually called “channels 1–16” by humans)
  - This is the per-event channel value encoded in channel Voice messages (Note_on, CC, Pitch_bend, Program_change, etc.).
  - Many tools display channels 1–16; some programmatic tools (midicsv, some libs) show 0–15. Be careful about off‑by‑one when looking at raw dumps.
- MIDI track (in a Standard MIDI File)
  - Logical container inside an SMF (Type 0,1,2). A single track may contain events for many MIDI channels, or all tracks may contain events for the same channel — no hard binding.
  - Common DAW practice: one MIDI track per instrument instance. That makes editing and routing easier.
- Meta events (time signature, tempo, text)
  - These are not channel messages. They are stored as Meta events in SMF and apply to the whole file/sequence/track. In Type 1 files the conventional place for global meta events (tempo, TS) is the first track (sometimes called the “tempo track” or header track).
  - They have nothing to do with MIDI channel 0 (the apparent coincidence you noticed).

Why you saw “channel 0 used only for header”
- That’s almost certainly a coincidental conflation of track numbering and channel numbering:
  - A LilyPond-generated Type‑1 SMF usually places tempo/time meta events in the first track (midicsv might show that as track 1 or 0 depending on zero/one indexing).
  - That track contains meta events, not “channel 0 note events.” The file’s separate musical voices are in subsequent tracks. So don’t interpret “track 1 has header stuff on channel 0” as meaning channel 0 is reserved for header — it’s just a track whose events are meta events.

MIDI Note Off vs Note_on with velocity 0
- Both are valid per the MIDI spec. Many devices, editors and DAWs prefer (or default to) Note_on with velocity 0 as note-off because it’s compact and aligns with running status optimizations.
- It’s fine to continue to emit note-on(vel=0) to turn off notes; almost every DAW and synth will accept it. If you need to be pedantic, you can emit explicit Note_off events, but it’s unnecessary in most workflows.

General MIDI channel conventions worth knowing
- Channel 10 (zero‑based channel 9) is traditionally the percussion channel in General MIDI. Many softsynths ignore this unless you are targeting GM behaviors; you can use it, but some hosts/plugins treat it specially.
- Avoid hard‑coding semantics into a particular channel number unless you’re interoperating with a particular device expecting it.

MPE semantics (important if/when you use MPE)
- MPE reserves contiguous blocks of channels as a zone:
  - Lower MPE zone: master channel = lowest numbered channel in the zone; member channels = the higher‑numbered channels.
  - Upper MPE zone: master channel = highest numbered channel in the zone; member channels = the lower‑numbered channels.
  - A master channel is used for global/per‑zone controllers (e.g., CC for expression that applies to the zone). Member channels are the per‑note channels where each sounding note gets a dedicated channel and per‑note pitch‑bend/pressure/etc.
- In practice:
  - Use one MIDI channel per simultaneous note for fully expressive MPE. You can put more than one note on the same channel if they share the exact same pitch‑bend and per‑note controllers, but that defeats the purpose of per‑note control and invites voice‑management bugs in synths.
  - The MPE spec is explicit about which channels are master vs member so synths can treat master controllers as global for the zone.

MIDI “zones” and “splits”
- Some synths define keyboard zones (ranges assigned to different channels or patches). That’s a synth feature (keyboard splitting, layering) and is not part of the SMF format. If you want multiple tunings simultaneously, you’ll typically just run multiple synth instances (one per track) rather than rely on hardware zone routing inside a single plugin.

Where to put controllers/pedals, program changes, and SysEx
- Controllers (CCs), sustain pedal (CC64), program changes and pitch‑bend should go on the same MIDI channel as the notes they are intended to affect — that is the default expected behavior for channel-based synths.
  - Practical approach in a DAW: put them in the same MIDI track (and same channel) as the instrument notes.
- SysEx (e.g., MTS bulk tuning dumps)
  - Put SysEx for a particular instrument instance on the same track (or as the first event in the take) routed to the plugin instance that you want to receive it.
  - Many DAWs (including Reaper) will pass SysEx through to the plugin, but some hosts/settings may filter SysEx; check the host preferences if you don’t see it applied.
  - Non‑Real‑Time bulk dumps are best placed at the start (timestamp 0) of the MIDI item/take so the plugin gets the tuning before notes play.

Transport, sequences, file types — brief clarification
- Transport: in DAW terms this is the play/stop/locate control. In hardware MIDI it may also refer to MIDI Clock/Start/Stop (synchronization messages). For your purposes (SMF import into Reaper) it’s just DAW transport.
- SMF types:
  - Type 0: single-track containing everything (channels interleaved).
  - Type 1: multiple tracks, synchronous playback relative to common tempo/PPQ. Common for DAW-style workflow (one track per instrument).
  - Type 2: multiple independent sequences (rare).
- Use Type 1 for your DAW imports — it’s the most compatible for separate instrument tracks.

Practical policy / recommendations for your workflow
You described a strategy that fits standard DAW practice: one MIDI item per region/tuning, one plugin instance per item/track. Here are concise rules to follow:

1) File format and structure
- Use a Type‑1 MIDI file (multiple tracks). Put global meta events (tempo, time signature) in the first track (tempo track). Put each instrument/region in its own track.
- In each musical track/take:
  - Put the MTS SysEx bulk dump or scale message at timestamp 0 of that track/take (Non‑Real‑Time bulk is recommended for preloading tuning).
  - Follow with ordinary Note_on / Note_on(vel=0) style note events. Keep all CC and Program Change events for that instrument on the same channel.

2) Channel usage
- Allocate one MIDI channel per instrument instance in the DAW. You can keep multiple voices/chords on that channel; that’s normal. For example:
  - Track A → plugin instance A → channel 1 (zero-based 0).
  - Track B → plugin instance B → channel 2.
- Avoid using channel 10 (zero‑based 9) for pitched instruments if you don’t want percussion semantics.
- If you use MPE:
  - Follow the MPE zone rules and assign a block of channels appropriately. Use a separate plugin instance configured for MPE if the plugin supports it. Don’t try to mix MPE and non‑MPE notes for a single instance unless the plugin explicitly supports it.

3) Note-off representation
- Stick with Note_on with vel=0 if that’s what your generator produces. It’s standard and well supported. No need to emit explicit Note_off unless you prefer the clarity.

4) Controllers and pedal data
- Put CCs and pedals in the same track and channel as the corresponding notes (same plugin instance). This keeps DAW editing intuitive and plugin behavior predictable.

5) SysEx and tuning/metadata
- Put MTS (or vendor tuning) SysEx at the start of the item/take.
- If you care about round‑trip or preserving your canonical lossless descriptors, also embed a textual meta event or a private SysEx blob containing your ratio/chaining description at the start of the item (this won’t bother the synth and will be preserved in the MIDI file). Reaper keeps these meta events with “takes” and they survive export/import.

6) If you want simultaneous different tunings
- Use separate plugin instances/tracks — easiest and most robust. Each instance gets its own tuning SysEx. That maps perfectly to your “one MIDI item per region/tuning” idea.

7) DAW-specific notes (Reaper)
- Reaper will usually pass SysEx to plugins and stores SysEx in takes, but confirm:
  - Preferences → MIDI → “Allow SysEx” or similar options must be enabled if there are filters.
  - Test by loading a SysEx-embedded MIDI into Reaper and verify the plugin sees and applies it at playhead 0.

Short checklist you can follow when generating MIDIs
- [ ] SMF Type 1
- [ ] Track 0: tempo/time meta events
- [ ] For each region:
   - [ ] Put a Non‑Real‑Time MTS bulk/scale SysEx at time 0 on the track (and optionally the canonical metadata text/SysEx).
   - [ ] Use one DAW track / one plugin instance for that region.
   - [ ] Put Note_on and Note_on(vel=0) for notes; CC/pedal/program change on same track/channel.
   - [ ] Avoid channel 10 if you want pitched instruments.
- [ ] If you use MPE: reserve the MPE zone channels correctly and use an MPE-aware plugin instance.

Final notes / small gotchas
- Plugins sometimes have fragile SysEx handling or only ingest SysEx at plugin load time. If you find a plugin doesn’t accept the SysEx you sent, test sending it via a separate utility or confirm the plugin docs for loading/importing tunings.
- If you plan to export MIDI from Reaper back into your toolchain and want to reingest it, check which meta events Reaper preserves. Embedding your canonical text at the start of each take is the simplest insurance policy.
- If you ever need to examine what LilyPond produced, midicsv is a great way to inspect the tracks/channels/meta events precisely (watch out for zero/one indexing).

If you want, I can:
- Produce a minimal midicsv template that matches these recommendations (Type‑1, tempo track, per‑region track with an MTS bulk SysEx at time 0, notes as Note_on vel=0 off‑events) so you can round‑trip with midicsv and import to Reaper for testing — no code, just a text CSV you can feed to csvmidi.

ChatGPT(gpt-5-mini/General)> Actually, I was mixing up "channel" and "track", and a lot of your clarification missed that I was making that mistake. Some times I said channel when I meant track. Let me correct:
* lilypond has header in TRACK 0
* lilypond puts tempo and time in TRACK 1
* instruments appear starting with TRACK 2
I'm murky then on the exact meaning of track and channel. There are 16 channels. You've mentioned percussion is usually on channel 10. Anyway, can you clarify what's a track, what's a channel, and what are the limitations? How many tracks can you have in a type 1 file? In type 1, all the tracks are synchronized, right? Let's say you wanted a full orchestral score that had more than 16 distinct instruments and more than 16 simultaneous pitches. Can that be represented in a type 1 standard midi file? How do you decide what's a track and what's a channel? I guess this is pretty basic stuff, but this is the first time I've dug into the guts of midi files, and it's surprisingly difficult to find clear, lucid descriptions of this stuff. Everything I read is written for the guy plugging his guitar into his amp, not to the person who's trying to do something novel and use MIDI files as a means of conveying information to a DAW.
<shell-maker-end-of-prompt>
Good — let’s be explicit and concrete so you can reason about MIDI files as data containers rather than as hardware wiring.

High level summary
- Track = an SMF container, an editing/organizational unit inside the MIDI file. Tracks carry a sequence of timed events (meta events, MIDI events, SysEx).
- Channel = the 4‑bit address encoded in MIDI voice messages (Note_on, CC, Pitch_bend, ProgChange, etc.). There are 16 channels per MIDI port (numbered 0–15 in raw bytes, shown 1–16 in most UIs).
- Tracks are arbitrary containers; channels are addressed to synths/devices. A track may contain events for many channels, and many tracks may contain events for the same channel.
- Type 1 files are the normal DAW-style format: multiple tracks, a single synchronized timeline (they use the same tempo/PPQ map). Type 0 collapses everything into a single track. Type 2 is rarely used (independent sequences).

Now the details.

1) What a track actually is
- Technical: In the SMF file format, a Track is a chunk (MTrk) that stores delta‑time stamped events in sequential order. The delta times are local to that track; the player or sequencer merges tracks by converting deltas to absolute times and interleaving events across tracks.
- Practical: Tracks are the unit editors and DAWs use as “this is one instrument” or “this is one part.” They’re for human organization and for putting meta events (like markers, lyrics, text) where you want them.
- Limits: The spec does not set a small fixed max number of tracks. Practically you’re limited only by file size and the player implementation. Modern DAWs and tools happily handle hundreds or thousands of tracks in a file (subject to memory and implementation limits).

2) What a MIDI channel actually is
- Technical: Every MIDI voice message carries a channel nibble (0x0–0xF) that the receiver uses to distinguish different control streams. The spec defines 16 channels per MIDI port.
- Practical: Channels are the addressing layer for synths. A single synth instance usually listens on one or more channels. Channel messages affect the synth’s current voice(s) assigned to that channel (Program Change, CC, pitch-bend, etc.).
- Limits: 16 channels per port. You can have multiple ports to get more total channels; in hardware this is multiple physical MIDI cables or logical ports, in software it is multiple virtual ports/plugin instances.

3) Relationship and mapping: tracks vs channels
- Tracks contain events (some meta, some channel messages). Channels identify the target stream. There is no one-to-one mandated mapping: it’s conventional to place one instrument per track and use one channel for that instrument, but that is a convention, not a file format requirement.
- Example patterns:
  - DAW‑friendly: one track per instrument, one channel per track. Clean for editing and routing.
  - Compact: put multiple instruments’ data into one track but with different channel numbers (rare for DAW workflows).
  - MIDI file generation: separate tracks for voices makes importing into DAW straightforward (tracks become DAW tracks/takes).

4) Type 0 vs Type 1 vs Type 2 (and sync behavior)
- Type 0: single track that contains all events interleaved. Useful for very simple playback targets and legacy devices. No multi-track organization.
- Type 1: multiple tracks that are time‑synchronized. A player merges tracks into a single time base using the tempo/time signature/PPQ. This is the format you want for DAW import/edit workflows.
- Type 2: multiple independent sequences (rare, not used by DAWs much).
- In Type 1, tracks are synchronized because the player uses the same tempo map and PPQ and merges events. Tempo/time meta events are usually in the first track, but they can appear elsewhere; conventionally track 0 holds global tempo/TS markers.

5) How to represent an “orchestra” with >16 distinct instruments / independent controllers
- Polyphony vs channel count:
  - Polyphony (many simultaneous notes) is not limited by channels. A single synth/channel can play many simultaneous notes (its internal voice count is the limit).
  - Channels are about independent control/addressing — independent program changes, independent CC streams, per-channel pitch-bend ranges, or per-note expression when using one channel per note (MPE style).
- If you need >16 independent control streams (e.g., 40 separate instruments each needing own CCs/ProgCh):
  - Option A (common in DAWs): use multiple synth/plugin instances. Each plugin instance is a separate “device” in the DAW that you can route one track to; each instance can use the same channel numbers independently because the DAW routes MIDI to instances separately.
  - Option B: use multiple MIDI ports (physical cables or virtual ports). Each port has 16 channels -> more total channels.
  - Option C: use multi‑timbral synths that accept multiple channels on the same instance (one instance can be multi‑timbral by listening on channels 1..16 and producing distinct patches per channel).
  - Option D (less common): encode extra addressing in SysEx or meta events + a plugin that understands them.
- For file interchange: the SMF can contain many tracks, but the channels encoded remain 0–15. How a DAW maps tracks to synth instances/channels decides how you get more than 16 independent “instruments.”

6) Practical reconciliation: “how do I decide what’s a track and what’s a channel?”
- Rule of thumb for DAW-oriented files:
  - Track = editing/routing unit (usually one instrument/one plugin instance/one MIDI lane).
  - Channel = the address you send to the synth/plugin that will play that track.
- Implementation guidance:
  - If your DAW workflow will map each track to its own plugin instance, use the same MIDI channel for each track (e.g., channel 1 on track A, channel 1 on track B) — the DAW will route track A’s events to instance A and track B’s events to instance B. The fact both use channel 1 is irrelevant because routing isolates them.
  - If instead you want one multi‑timbral synth instance to respond to many parts, assign distinct channels for each part within a single track or across tracks and keep routing aimed at that one instance.

7) Other useful practical facts and gotchas
- Percussion: General MIDI convention assigns channel 10 (zero‑based 9) to percussion; some synths/patch sets treat it specially. Avoid if you don’t want percussion semantics.
- Note_off vs Note_on vel=0: both are acceptable. Many tools write the latter; playback engines treat them equivalently.
- Program Changes and CCs are channel‑addressed, so to change instruments independently you either need distinct channels or distinct plugin instances (where DAW routes only track events to that instance).
- MIDI port meta event: SMF supports a MIDI Port meta event (0x21) to indicate intended port number. Many DAWs ignore or don’t use this; virtual‑port routing in DAWs is often done separately.
- SysEx: not channel-specific (but usually sent to a device), used for bulk tuning (MTS), vendor patches, layout dumps, etc. Put SysEx at track start for plugin-specific configuration.
- MPE: uses a contiguous set of channels as a zone; MPE semantics require special handling by synths.

8) Concrete examples to cement the concept
- Single plugin instance per DAW track (recommended for your use case):
  - SMF: create Track N; at time 0 put SysEx tuning for that plugin (targeted to that plugin instance when imported into DAW), then normal Note_on/Note_off on channel 0 (or 1). When imported, Reaper maps Track N to plugin instance N and sends those channel messages only to that instance.
- Single multitimbral synth for whole orchestra:
  - SMF: multiple tracks, each track uses a unique channel (0..15) to address patches within that synth instance. If you need more than 16 parts, either use multiple synth instances or multiple ports.

9) Answering your specific questions succinctly
- “How many tracks can you have in a Type 1 file?” Practically unlimited (file size / implementation limit). You can have many tracks — DAWs commonly import and map them to separate tracks.
- “In Type 1, all tracks are synchronized, right?” Yes — they share the same timebase and tempo map; the player merges them by absolute time.
- “Can an orchestral score with >16 instruments and >16 simultaneous pitches be represented in Type 1?” Yes — simultaneous pitches are not limited by channels; use either:
  - many tracks routed to separate plugin instances (each instance handles polyphony), or
  - multi‑timbral synths + separate channels (but limited to 16 channels per port), or
  - multiple ports/instances to get more channels.
- “How do I decide track vs channel?” Track = editing/routing unit; Channel = synth addressing for control. For DAW workflows, prefer one track per DAW instrument instance and reuse the same channel number across tracks, letting the DAW handle routing.

10) Final practical recommendation for your pipeline
- Generate Type 1 MIDI files with:
  - Track 0 (or 1 by LilyPond convention) carrying tempo/time meta events.
  - Each musical region as its own track (one per intended DAW track/plugin instance).
  - Put MTS SysEx bulk/scale at time 0 in the corresponding track so the plugin instance gets the tuning before notes play.
  - Use Note_on(vel=0) off semantics or explicit Note_off — either is fine.
  - Use channels consistently within a track for controllers/ProgramChange; don’t worry about reusing the same channel number across different tracks if you intend separate plugin instances.
- If you need more than 16 independent control streams for a single host synth, plan to either instantiate multiple synths or use multiple ports (virtual or physical).

If you want, I can:
- produce a minimal Type 1 midicsv template that follows these recommendations (tempo track, separate tracks for two different tunings each with a sample SysEx tuning at t=0, notes using Note_on vel=0 to turn off) so you can inspect and import into Reaper; or
- walk through a concrete example mapping a 24‑part orchestral layout into a DAW using plugin instances and channel/port decisions. Which would you prefer?

ChatGPT(gpt-5-mini/General)> What is the program change message used for? Other than percussion, when would you use more than one channel per track? I guess one example would be with pitch bend. What about instrument changes, say, like adding a mute on a trumpet or switching between plucked and bowed for a stringed instrument? Let's say I had a midi file for a trumpet solo with piano accompaniment in regular 17-EDO and I was using MPE, just so I understand. I might have global information (tempo) on track 1, piano on track 2, and trumpet on track 3. Track 2's channels would all be devoted to the piano, but I would use different channels with different pitch bends. For the trumpet, let's say I was using a bulk tuning system so I didn't need pitch bends for pitch. (In real life, I would not mix and match. I'm just trying to make sure I understand the structure.) Here, I might use one channel for open and another for mute, again recognizing that the channel space within one track is distinct from one within another track. Maybe the trumpet part has some pitch bending representing lip movements. Here I could encode pitch bend momentarily on either the open or muted channel. The trumpet is a single-pitch instrument, so there are no chords or anything, and I don't need to worry about tracking notes to pitch bends. I can just use one if I need to bend the note. Does this sound sensible? And to clarify, is it the case that channels are scoped to a track? Your previous answer was contradictory on this point, stating that "A track may contain events for many channels, and many tracks may contain events for the same channel" (suggesting that channels are somehow shared) but also suggesting channels are per track. The first statement makes sense if I assume you are just saying that different tracks may use overlapping *channel numbers*.
<shell-maker-end-of-prompt>
Good set of questions — you’re thinking about the right tradeoffs. I’ll be explicit and concrete and avoid hand‑waving.

1) What Program Change is for
- Program Change (status byte 0xC0 + program number) is the per‑channel message that tells a receiving synth which patch/voice/program to use on that channel. It’s the standard way to switch timbres (e.g., trumpet→muted trumpet, violin pizzicato→violin arco) on a single channel.
- Bank Select (typically CC 0 and CC 32, sometimes CC 0/32 + MSB/LSB variants) is used in combination with Program Change to address large banks of patches on modern synths/samplers.
- Practical use:
  - If a single channel represents a single instrument part and you want that part to change articulation/timbre at a time boundary, send a Program Change (and bank select if needed) before the next notes.
  - If you want two different timbres sounding simultaneously, program change alone won’t help because it switches the program for that channel globally; you need two channels (or two instances).

2) Are channels scoped to tracks?
Short answer: No — channels are not scoped to tracks by the MIDI file format; a track is just a container of events. The XML:

- Track: a file/container concept (MTrk chunk in SMF). It stores time‑stamped events. There can be any number of tracks.
- Channel: a 4‑bit address in the MIDI voice messages (0–15). Channels are a property of each event, not a property of a track. In the raw SMF any track can contain events for any channel.
- In practice in DAWs: each track is routed to a particular plugin instance or MIDI output, and that routing typically isolates that track’s events to that destination. Because of that mapping, it feels like “channels are per track” when you use one plugin instance per track — but that is a DAW routing convention, not a file format rule.

So your reading of my earlier statement is correct: I meant “different tracks may use the same channel numbers” (because channels are only channel numbers in the file; routing decides where they land).

3) Why/when you’d use more than one channel in one track
Use multiple channels in the same track when you need independent channel-scoped state or simultaneous, independent patches that are naturally routed to the same MIDI output/device instance:

- Multi‑timbral synth instance: the synth instance can produce multiple patches simultaneously by listening on multiple channels. If you want two patches inside the same synth instance you use two channels.
- Simultaneous distinct articulations: if the same musical part needs to play two different articulations at the same time (e.g., backing pad + solo line, or crossfading open and muted trumpet samples simultaneously), send them on distinct channels so each can have its own program/CC.
- Independent per‑channel controllers: controllers like pitch‑bend, CCs, program changes, PB range (RPN/NRPN) are per channel. If you need different PB streams concurrently (different PB values) you need different channels.
- Per‑note pitch control (non‑MPE): if you need more than one independent pitch‑bend stream at the same time, you either use MPE (one channel per sounding note/voice) or allocate channels manually to voices.
- Voice management tricks: some composers synthesize instantaneous doubling or layering by duplicating the same notes on several channels with slightly different patches.

4) Your trumpet + piano example — practical mappings and options
Scenario recap (your examples): Track 1 tempo; Track 2 piano (17‑EDO); Track 3 trumpet (bulk tuning; occasional pitch bend for lip slides). You consider using separate channels for open vs muted trumpet.

Option A — Simple, single channel per instrument (most DAW‑friendly)
- Track 2 (piano) → send all piano notes on channel 1.
- Track 3 (trumpet) → send all trumpet notes on channel 2.
- Program Change on channel 2 when you need trumpet→muted trumpet, placed before the next note event (if the plugin switches timbre immediately).
- Pitch bend applied on channel 2 for any bends.
Pros:
  - Clear editing: controller lanes and program changes live with the track.
  - DAW routing straightforward; one plugin instance per track works well.
Cons:
  - You cannot have open and muted trumpet sounding simultaneously on the same channel without re‑triggering and program switching (the program change replaces the program on that channel).

Option B — Two channels for trumpet to allow simultaneous articulations
- Track 3 contains two logical trumpet streams:
  - Channel 2 for open trumpet patches,
  - Channel 3 for muted trumpet patches.
- Notes that should sound muted go to channel 3 (with its own program change set to muted trumpet). Notes that should be open go to channel 2.
- If you want occasional bends on a sounding note, send pitch‑bend on the same channel as the note.
Pros:
  - Allows open and muted to sound simultaneously without program changes.
  - Controller data on each channel is independent.
Cons:
  - Slightly more complexity: you must decide channel assignment when generating MIDI and track editor might show all channels interleaved.
  - If your DAW routes Track 3 to a single plugin instance, that instance will get channel 2 and 3 messages and must be multi‑timbral (or the DAW must route the different channels appropriately to separate instances).

Option C — Dedicated plugin instances per articulation (DAW approach)
- Create two separate instrument tracks in the DAW, each with its own plugin instance:
  - Track 3A → trumpet open (channel 1 on that instance),
  - Track 3B → trumpet muted (channel 1 on that instance).
- Route notes intended for open trumpet to 3A and muted to 3B.
Pros:
  - Very simple file model: each track/instance uses the same single channel internally.
  - Most DAWs make this extremely easy and editing is straightforward.
Cons:
  - More plugin instances; more CPU, but usually acceptable.

Which is most sensible?
- Since trumpet is monophonic, Option A is simplest if you never need simultaneous two different articulations sounding. If you sometimes want both (e.g., blend muted and open), Option B or C are appropriate. Option C is the cleanest in DAWs: separate tracks/instances for separate simultaneous timbres.

5) Pitch bend specifics in the context
- Pitch bend is per channel. If a channel has multiple overlapping notes, a PB change affects all notes sounding on that channel (unless the synth supports per‑voice retuning).
- For a monophonic trumpet track that uses occasional bends, a single channel with PB is perfect. For polyphonic parts that require independent per‑note bends you need either:
  - MPE (one channel per sounding note), or
  - careful manual channel allocation so each currently sounding voice has its own channel.

6) Program changes and live/sustained notes
- Be careful: many softsynths change the patch immediately, and the new patch may or may not affect already‑sounding voices. Behavior varies by synth:
  - Some synths will change the timbre of existing voices in place.
  - Others will apply the program change only to subsequently triggered voices (the existing voice keeps its old patch).
- If you need the articulation to change while a note is held (e.g., change mute state mid‑sustain), you usually re‑trigger the note on the new program, or use plugin automation if the plugin exposes articulation as a parameter.

7) Practical generation rules for your pipeline
- If you will import into a DAW and use one plugin instance per track, you can safely reuse the same channel number across tracks (the DAW will isolate them). That is the simplest mental model: Track→Plugin Instance→Channel 1 (local).
- If you want to create a single multi‑timbral file that target a single multi‑patch synth instance, then allocate distinct channels for each simultaneous patch you need on that synth (and keep track of channel usage).
- If you need per‑note independent pitch bends: use MPE or distribute voices to channels so each voice has unique channel.
- If you want separate simultaneous articulations for the same “part”, either:
  - allocate multiple channels and route them to a single multi‑timbral synth instance, or
  - create multiple DAW tracks/instances and route accordingly (recommended for DAW workflows).

8) A few gotchas and tips
- Avoid relying on program change to immediately change articulation of held notes (synth behaviour varies). Test on your target plugin.
- Channel 10 (zero‑based 9) is conventionally percussion — avoid it for pitched instruments, unless you intentionally want percussion mapping.
- When generating MIDI for DAW import, your life is easiest if you model “one track per logical instrument/instance” and put CCs/ProgramChange/PB on the same track/channel. If you need simultaneous variations, use separate tracks or channels intentionally.
- When embedding MTS tuning SysEx or other SysEx, place it at time 0 in the track/take so the synth receives it before note events.

9) Short examples to illustrate common choices (no code)
- Single trumpet, no overlap of different articulations: single track, single channel, program change when you want change.
- Trumpet needing muted and open simultaneously: either two channels in one track targeted at same synth instance, or two DAW tracks with separate instances — pick the DAW tracks approach if you want simpler editing.
- Polyphonic instrument needing per‑note bends: MPE (one channel per note) or channel allocation scheme where each voice gets its own channel.

If you want, I can produce a compact midicsv example showing:
- A single trumpet track using Program Change for mute,
- The same trumpet implemented with two channels (open + mute),
- Or an MPE-style snippet showing per-note pitch bends on per-note channels,
so you can see the exact event sequences and how they differ. Which would you prefer to inspect?

ChatGPT(gpt-5-mini/General)> In your option A, you said:
Option A — Simple, single channel per instrument (most DAW‑friendly)
- Track 2 (piano) → send all piano notes on channel 1.
- Track 3 (trumpet) → send all trumpet notes on channel 2.

Why would you not put trumpet on channel *1* of track 3 instead of channel 2? This is where I'm still confused about tracks and channels. Would it be that sometimes two tracks are routed to the same synth? port? and that's why you'd use different channels?
<shell-maker-end-of-prompt>
Short answer
- Yes — you could put the trumpet on channel 1 of track 3. Technically there is no rule forbidding that. Whether you should depends on how the DAW routes tracks to synths/ports.
- Use different channel numbers only when the destination (a single synth instance or a single MIDI port) needs channels to distinguish simultaneous or independent parts. If each DAW track is routed to its own plugin/synth instance, you can reuse the same channel number across tracks with no problem.

Expanded explanation (concrete, no hand‑waving)

1) Basic facts
- Track = a container inside the MIDI file (MTrk). It just holds timed events.
- Channel = the 4‑bit address embedded in voice messages (0–15 in raw bytes; commonly shown 1–16). Channels are a property of events, not of tracks.
- A track may contain events for any channel(s). Many tracks can contain events that use the same channel numbers; whether they conflict depends on how those tracks are routed at playback.

2) Why I used channel 1 for piano and channel 2 for trumpet in Option A
- I was illustrating the case where you imagine a single multitimbral synth instance (or a single MIDI port) receiving messages for both instruments. If both piano and trumpet were to be played by the same synth instance that distinguishes its parts by channel, you must assign different channel numbers so the synth knows which patch/voice for each part.
- If, instead, you route the piano track to plugin instance A and the trumpet track to plugin instance B, each instance receives only its track’s events. In that case channel numbers inside the track are mostly irrelevant: both tracks could use channel 1 with no conflict because the DAW routes them to different destinations.

3) Concrete scenarios and what to do
- DAW track → single plugin instance per track (typical DAW workflow)
  - Recommendation: Put notes and CCs on the same channel inside that track (e.g., channel 1). It doesn’t matter if every track uses channel 1 because the DAW isolates the routing to separate plugin instances.
  - Result: Simple, intuitive editing. Program changes and CCs on the track affect just that plugin instance.
- Multiple tracks routed to the same synth instance (multi‑timbral synth)
  - Recommendation: Assign distinct channels per part (e.g., piano → ch 1, trumpet → ch 2) so the single synth can produce separate patches and manage per‑channel controllers independently.
  - Result: Channels act as the addressing mechanism for the synth instance.
- External MIDI hardware with one physical port
  - You are limited to 16 channels per port. To support more simultaneous independent parts, either use multiple ports or multiple device instances.
- MPE or per‑note pitch bend
  - You need multiple channels for per‑note control (one channel per sounding note in MPE, or manual channel allocation for multiple simultaneous independent pitch‑bend streams).

4) Practical guidance for your generator
- If your intended DAW workflow will have one plugin instance per musical region/track (your plan of one MIDI item per tuning/region and one plugin instance per item), then:
  - Keep it simple: emit a single channel (e.g., channel 0 or 1) per track; put all note, CC, program changes for that track on that channel.
  - Reuse the same channel number across tracks; the DAW will keep them separate.
- If you intend to target a single multi‑timbral synth instance or a single MIDI port where a single synth must handle multiple parts, then:
  - Decide and document a channel allocation plan (which channel number is for which part) and emit those channel numbers in each track.
  - Optionally add a textual meta event or SysEx in the track indicating the intended mapping so human editors and your future self aren’t confused.
- If you want to support both workflows robustly:
  - Default to one channel per track (channel 1). If you later want to route multiple tracks to one synth, you can remap channels in the DAW or regenerate the file with a channel allocation table.

5) Example to make it concrete
- Case A (two plugin instances, isolated):
  - Track 2: channel 1, Piano notes → routed to Plugin A
  - Track 3: channel 1, Trumpet notes → routed to Plugin B
  - No conflict because Plugin A receives only Track 2, Plugin B only Track 3.
- Case B (single multi‑timbral synth instance):
  - Track 2: channel 1, Piano notes
  - Track 3: channel 2, Trumpet notes
  - Both tracks are routed to the same synth instance; the synth demultiplexes by channel.

6) Final checklist (quick rules of thumb)
- One plugin instance per DAW track → reuse channel numbers freely (simplest).
- One synth instance for multiple parts → use distinct channel numbers to separate parts.
- Need multiple simultaneous articulations or per‑note independent controllers → allocate multiple channels (or multiple instances).
- Always document allocations (meta text or SysEx) if the mapping is nontrivial.

If you like, I can produce a tiny midicsv snippet that shows:
- two tracks both using channel 1 but routed to different plugin instances (DAW‑style), and
- the same two parts using channels 1 and 2 but routed to a single multitimbral synth,
so you can see the exact event streams and compare. Want that?

ChatGPT(gpt-5-mini/General)> I am understanding more clearly that a lot of this is specific to the synth/instrument. I believe Surge-XT can be a multi-timbral synth. You can set input for a MIDI port to "all channels" or a specific channel. Most examples I've seen select "all channels." If I were targeting a known, specific instrument, I might code to its particular constraints. Otherwise, I probably wouldn't assume multi-timbral. I can imagine two different cases for multi-timbral. Once is where you could have a synth that specific says "channel 1 is flute, channel 2 is violin", etc. Another might be where a single note plays two different instruments at the same time, a common setting on keyboards. My Yamaha DGX 670 can do that (piano + pad, etc.). I notice that Lilypond uses one track per voice and one non-overlapping channel per track. This seems like the most flexible. If you have the numbers to spare, it guarantees no conflicts without any assumptions about routing or properties of the synth. With timidity, midi instrument numbers are mapped to instruments in a sound font using "standard" instrument numbers, but this is definitely not always true. Many sound fonts are all piano, all brass, etc. In that case, you have to use instrument numbers that map to the sounds based on knowledge of the sound font. In that case, you would say timidity is a multi-timbral synth, map a channel to an index in the sound font, and get different voices by *channel*, with tracks being a different logical division. If I am specifically creating MIDI whose purpose in life is to be loaded into a DAW, I would have to specify (or make this configurable) what my strategy is. For example, I could have a default be to use a separate track for each instrument and to assume isolated channel spaces for each track, and that would require the DAW user to set up a separate synth/plugin for each track and route them separately, but if I followed what I see with lilypond and tried to use non-overlapping channel numbers, that would give the DAW user the potentially to simplify by routing all the tracks to the same multi-timbral synth. Now if the DAW user *wanted* to take one of the tracks and make it do something else, like have each note play on multiple instruments or add various polyphonic effects, the DAW user would freely be able to create a separate instance and then break free from the constraint on channel numbers. Even with completely separate virtual ports, there's no reason why the DAW user couldn't decide to use the same synth configuration, which would mean the channels would have the same meanings. But if I wanted more than 16 simultaneous instruments, it might be that channel 3 in track 3 over here pointed to one thing and channel 3 in track 3 over there pointed to something else....but that would be something the DAW user would have to actually do in the routing configuration. Timidity doesn't seem like it could actually do that on its own (fine), so if I were generating standard midi files for something like that, there would necessarily be some additional mixing step. For example, I could create a 20-part score in lilypond and set up the main file's voice to midi mapping to generate multiple outputs with different subsets of tracks, just as you do when you're splitting parts. Then you could pull the two separate rendered audio streams into audacity to mix them. Is my mental model converging?

TRACK -> a collection of events
CHANNEL -> a property of the event
Mapping of CHANNEL to instrument: a property of the ROUTING TARGET that the TRACK is routed to -- not represented in the MIDI file at all other than that the midi file may contain program changes/instrument number assignments. But, for example, if TRACK 3 switched CHANNEL 4 from instrument 15 to instrument 16, what that actually means is 100% determined by the synth that track 3 is routing to and how that synth interprets what instrument 15 and instrument 16 mean.
<shell-maker-end-of-prompt>
Yes — your mental model is converging correctly. Short confirmation, a few clarifications, and a compact checklist you can use when generating files.

Succinct confirmation
- TRACK = a sequence/container in the SMF file that carries timed events (meta, SysEx, MIDI channel events). It’s an editing/routing unit for humans/DAWs.
- CHANNEL = a 4‑bit field on voice messages (Note_on, CC, PB, Program Change, etc.). It’s an address interpreted by the receiver(s).
- Mapping channel→instrument is not encoded by the SMF except via Program Change/Bank Select messages and (optionally) MIDI Port meta events / SysEx. The actual sound or patch that corresponds to a program number is defined by the receiving synth/plugin (or the soundfont) and by how the DAW routes tracks to plugin instances or hardware ports.

Key clarifications and implications

1) Channels are event properties, not track properties
- Any track can contain events on any channel. The SMF format doesn’t “bind” a channel to a track.
- DAWs typically create a 1:1 mapping between MIDI track and plugin instance and therefore treat a track’s channel use as local; but that’s a host routing convention, not a file rule.

2) When different channel numbers matter
- Use distinct channels when the single destination synth instance distinguishes parts by channel (multi‑timbral synth), or when you need concurrent independent per‑channel state (separate Program/Bank, separate CC streams, independent pitch‑bend values, MPE per‑note channels, etc.).
- If each DAW track routes to its own plugin instance, channel numbers across tracks can be safely reused.

3) Program Change & Bank Select
- Program Change (and Bank Select via CC 0/32 or MSB/LSB) tells the receiver which patch number to use on that channel. How the receiver maps program numbers to actual sounds is implementation/soundfont dependent.
- Program Change is per channel. If you want two timbres simultaneously, either use two channels (same instance multitimbral) or two plugin instances.

4) SoundFont / Timidity semantics
- A SoundFont maps (bank, program) to an internal instrument sample. Many SFs are single‑timbre, some contain GM mappings. You must know the SF’s mapping (or let the DAW user choose fonts) to get intended timbres.
- Timidity (and other renderers) can be treated as a multi‑timbral synth: channels are addressed to different preset indices within the SF, but the actual mapping depends on the SF contents and the renderer’s config.

5) Multi‑timbral vs layered/doubled voices
- Multi‑timbral: single synth instance listens on multiple channels and produces different patches per channel.
- Layering/doubling: the same note may trigger multiple patches (either by sending the note to multiple channels or by the synth internally layering patches). This is a synth feature and not encoded directly in the SMF except by sending duplicate notes on multiple channels/tracks.

6) More than 16 independent control streams
- A single MIDI port has 16 channels. To exceed 16 independent addressable channels, use:
  - multiple virtual/physical ports, or
  - multiple plugin instances in the DAW (each instance receives the track’s events and can interpret channel numbers in its own context).

7) Practical generator strategies (what to pick)
Two robust generator modes many people use:

A — DAW‑friendly default (recommended if target is DAW import)
- Emit Type‑1 MIDI with: tempo in the tempo track, one SMF track per logical part/region.
- Put all events for each part on a single channel within the track (e.g., channel 1).
- Reuse channel numbers across tracks — the DAW will isolate them when you route each track to its own plugin instance.
- Put SysEx (MTS tuning) at time 0 of the track to configure the plugin instance beforehand.

B — Multi‑timbral friendly (if targeting a single synth instance)
- Allocate non‑overlapping channels across tracks (or across logical parts) so the single synth can demultiplex by channel.
- Emit Program Change / Bank Select messages where appropriate.
- Document mapping (meta text / SysEx or README) since the mapping is synth/host dependent.

You can support both by making channel allocation configurable (single channel per track by default; optional non‑overlap allocation).

8) Small gotchas to watch for
- Channel 10 (human index 10, zero‑based 9) is GM percussion — avoid it for pitched parts unless intentional.
- Note_on with velocity 0 vs explicit Note_off: both work; most hosts accept either.
- Program Change timing/held notes: some synths don’t apply program changes to already sounding voices; behavior varies — test your target synth.
- MPE: reserves blocks of channels and imposes master/member semantics; MPE semantics are meaningful only if the synth is MPE‑aware.

Compact checklist for your file generator (practical)
- Decide default routing mode (A or B above).
- Output Type‑1 MIDI.
- Track 0 (or 1 depending on your zero/one convention): global meta events (tempo, TS).
- For each DAW region/part:
  - Create a separate SMF track.
  - Put MTS SysEx (or your metadata SysEx/text) at delta-time 0 of the track.
  - Emit NOTE/CC/ProgramChange events on a single channel for that track (default), or use assigned distinct channels if targeting multi‑timbral synth.
- Provide a simple map/README with channel→intended part mapping when you choose non‑overlapping channels.

If you want, I can:
- Produce two tiny midicsv examples (one DAW‑friendly with channel reuse, one multi‑timbral with distinct channel allocations and Program Changes) so you can inspect the exact event streams and import them into Reaper to see the routing behavior. Want me to generate those?

ChatGPT(gpt-5-mini/General)> The midicsv file has track 0 with the header and track 1 with tempo, etc. Is that literally track 0 with the header, or is the format of the something that just has 0 in the place where the track is?

Also, can you fill me on what these "percussion semantics" are that you keep talking about? Do you mean, for example, that different note numbers refer to different percussion sounds rather than pitches? Or are there some other percussion semantics that you are talking about? I tend not to use percussion, but my daughter is a percussionist, so maybe I should start incorporating that into my arrangements. :-)
<shell-maker-end-of-prompt>
Good questions — both items deserve a crisp, unambiguous answer.

1) midicsv “Header” / track numbering

- The Standard MIDI File (SMF) format has a distinct header chunk and then one or more track chunks (MThd and MTrk chunks). The header chunk is not itself a track — it contains global file info (format, number of tracks, division/PPQ).

- midicsv represents that header chunk as a CSV row labelled "Header". midicsv prints that Header row using the track index 0. The actual track chunks (MTrk) follow and midicsv numbers them starting at 1. So when you see a CSV like:

  0, 0, Header, 1, 3, 480
  1, 0, Start_track
  1, 0, Title_t, "Something"
  2, 0, Start_track
  ...

  — the "0" row is the SMF header chunk (not an MTrk). Track 1, track 2, etc. correspond to the actual track chunks in the file. midicsv chose this convention so the header has a row, and it’s unambiguous; other tools might enumerate differently, but this is what midicsv does.

- Practical implications:
  - Track numbering in midicsv: Header = track 0; the first real track chunk = track 1; subsequent tracks increment from 2, 3, ...
  - Many human‑facing tools or docs talk about “track 1” as the first musical track; watch for off‑by‑one differences between tools.
  - The order of the MTrk chunks in the file is the order midicsv will show them (and that order is what a simple parser uses when reassembling or merging events).

2) “Percussion semantics” — what that actually means

When I say “percussion semantics” I mean that on one particular MIDI channel (by convention channel 10, which is zero‑based channel 9 in raw bytes) the meaning of MIDI note numbers is usually different: they index drum/percussion sounds instead of chromatic pitches. Concretely:

- General MIDI convention:
  - Channel 10 is the dedicated percussion/drum channel.
  - On channel 10, each MIDI note number (e.g., 35, 36, 38, 42, 46, etc.) corresponds to a particular drum sound (bass drum, snare, closed hi‑hat, open hi‑hat, etc.) according to the GM percussion map. So note number = instrument (not pitch).
  - Program Change on channel 10 typically selects a drum kit (a collection of mappings/sample sets), and the note→instrument mapping is defined by that kit.

- Practical differences from pitched channels:
  - Pitch meaning: On pitched channels, a note number implies a pitch (frequency). On the percussion channel, note numbers typically do not map to pitched notes (although some drums have tuned pitches); they map to samples/sounds.
  - Pitch bend: Many drum/percussion implementations ignore pitch-bend or treat it differently; behavior varies by synth. Bending a snare by pitch‑bend is often meaningless unless the drum sampler supports it.
  - Program/Bank: Program Changes on channel 10 may choose different drum kits; a soundfont’s drum kits are often in a specific bank/program area.
  - Velocity semantics: Percussion velocities map to sample dynamics/hit strength; velocity mapping can be quite different from expressive pitch instruments.

- SoundFont / Sampler specifics:
  - Some soundfonts place drum kits in a special bank (often bank 128 or a designated percussion bank) and interpret whatever channel you use as percussion if routed to that bank/region. Many renderers and DAWs treat channel 10 specially to map to drum kits automatically.
  - Not all synths strictly follow GM; drum mappings differ between instruments and authors. So channel 10 is a convention that is broadly supported, but exact sounds will vary across soundfonts and VSTs.

- Why it matters for file generation / DAW workflow:
  - If you put a drum pattern on channel 10 and the target synth is GM‑compatible, the DAW/plugin will interpret the notes as drum hits rather than pitched notes.
  - If you are using a sampler or soundfont where drums are arranged differently, the same note numbers might trigger different samples or you may need to use Program Change/Bank Select to pick the correct kit.
  - If you don’t intend percussion, avoid channel 10 so you don’t accidentally trigger a drum kit on a GM synth.

Examples of percussion use cases
- Notation/percussion parts: Put percussion parts in their own track and give that track channel 10 (or let the DAW route it to a drum instrument). Use GM note‑numbers to specify which drum sound to strike.
- Drum kit selection: Use Program Change (on channel 10) or Bank Select to change drum kits.
- Hybrid cases: Some modern samplers allow pitched drums and accept pitch-bend/filters; in those cases percussion channel semantics can be richer.

Summary advice
- midicsv: Header row is printed as track 0; actual tracks begin at 1. Treat that as a presentation convention — the header is a distinct chunk, not a musical track.
- Percussion: channel 10 is the conventional percussion channel — note numbers index drum instruments, not pitches; program/bank chooses drum kit. If you don’t want drum semantics, don’t use channel 10 (or be explicit about Program/Bank so target synth knows what you mean).

If you want, I can:
- Show a tiny midicsv snippet illustrating a percussion track (channel 10) vs a pitched track, or
- Give the GM percussion mapping table (note numbers → drum instrument names) for quick reference. Which would you prefer?

ChatGPT(gpt-5-mini/General)> This is all good. I've written a summary for my future self. Can you check it for correctness? I'm omitting information I don't care about, but feel free to mention obvious omissions.

# MIDI Summary

In a nutshell:
* An *event* that references a *channel* in a *track* gets mapped to a particular thing happening on a particular instrument as follows:
  * The track is routed to a target synth within the rendering environment (DAW, timidity, etc.)
  * The synth gets the program currently assigned to the channel along with other channel-specific information, like pitch bend
  * The synth interprets that program in its own way

## SMF (Standard Midi File)

* tool `midicsv` can map between a CSV file and a MIDI file; great for looking at/editing MIDI
* `midicsv` shows first row as track 0 for the header line, but it's not a real track
* file type is 0, 1, or 2. Type 1 is what we care about: multiple, time-synchronized tracks

## Tracks, channels

A track is a sequence of events. A SMF can contain any number of tracks. By convention, use track 1 (the first actual track) for global metadata, tempo changes, time signatures. Start with track 2 for notes.

A track is just a container for organizing events. Tracks may appear in the DAW as separate DAW tracks and may be routed separately to different plugins/synths. Use different tracks for different logical voices.

A channel is an index within the synth. They are numbered internally as 0 to 15 and referred to as 1 to 16; be careful about off-by-one. A channel gets a program via a program change message. The program effectively says what instrument sound notes get on that channel at that time.

Within the DAW (or rendering system), there is a many-to-many mapping between tracks and destinations. The destination interprets the meaning of the channel. If two tracks both use channel 1, their channel 1 instances will sound the same if the tracks are routed to the same destination instance. For maximum flexibility for use of multi-timbral synths (that allow different channels to be mapped to different instruments), use non-overlapping channel numbers within tracks, but reuse of channel numbers is normal and works fine if tracks are routed to different instances.

There are certain channel-wide settings, such as pitch bend. If using MPE, you would distribute notes across channels based on pitch bend information, and all the channels would likely have the same "program" (instrument sound). If you wanted to change timbre within a voice (e.g. adding a mute), you would usually send a program change command for the channel rather than using a different channel.

A good DAW-friendly strategy:
* Use a separate track per voice
* Use a separate, non-overlapping channel per track

This will work well with single-timbre or multi-timbre synths. Reasons to deviate:
* If you are using channels for MPE, each track will likely need its own channel space
* If you have more than 16 instruments, it will be necessary to route different tracks to different places

The mapping of track/channel to actual instrument is a function of where the track is routed and how the destination interprets the program mapped to the specific channel.

An alternative DAW strategy is to assume isolated channel numbers per track. This scales better for more instrument sounds but requires each track to be routed separately and is thus potentially more work to set up.

## Other notes

It is common to use "note on" with velocity 0 to turn a note off. If you use "note off", it should have velocity 0.

Channel 10 (from 1, 9 from 0) is usually percussion. For percussion, note numbers map to different drum sounds rather than different pitches, and pitch bend information is often ignored. Avoid channel 10 for non-percussion use.

MTS-ESP allows tuning banks to be loaded within a track. MTS tuning banks apply to all instruments in the instance, so for two tracks to meaningfully use different tunings, they have to be routed to different instances.

<shell-maker-end-of-prompt>
This is a very good summary — you’ve captured the essentials and the right mental model. A few small corrections, clarifications, and a couple of obvious omissions you might want to note for your future self.

I’ll list line‑by‑line corrections / clarifications, then a compact, slightly revised “best practice” recommendation.

Corrections and clarifications
- midicsv header numbering
  - midicsv prints the file header as a CSV row using track index 0; the first actual MTrk chunk is printed as track 1. So your statement is correct but be explicit: the “track 0” row in midicsv is the header chunk, not an MTrk.

- “Start with track 2 for notes”
  - That’s a common convention (LilyPond does this), but not a file requirement. It’s conventional to put global meta events (tempo/time signatures) in the first track chunk; some tools put meta events in the same track as music. So treat the “track 1 = tempo/meta, tracks 2+ = parts” pattern as a practical convention rather than a rule.

- Note-offs and velocities
  - Note_on with velocity = 0 is equivalent to Note_off (both are legal). However: explicit Note_off messages may carry a non‑zero release velocity (many synths accept/rely on that), so it’s not correct to say if you use Note_off it should have velocity 0. In practice, using Note_on(vel=0) is perfectly standard and widely supported; explicit Note_off often uses release velocity meaningfully.

- Channels are per MIDI port
  - Channels are 4‑bit values (0–15) and belong to a MIDI port. If you have multiple MIDI ports, you get 16 channels per port. Also some SMF files include a MIDI Port meta event — but many hosts ignore it and DAW routing is done outside the file.

- Program Change and Bank Select
  - Program Change alone selects a program number on a channel. Many synths require Bank Select (CC 0 MSB / CC 32 LSB, or Bank MSB/LSB conventions) to address larger banks. Program/Bank semantics are entirely dependent on the receiving synth/soundfont.

- Percussion semantics (channel 10)
  - Your summary is right: channel 10 (human 10, zero‑based 9) is conventionally the percussion channel in GM. On that channel, note numbers usually map to drum/instrument samples rather than chromatic pitches; pitch bend is often ignored or meaningless. Different synths/soundfonts map note numbers to different drum sounds, so the exact mapping is not universal.

- MTS‑ESP: scope and types
  - Good note: MTS SysEx tuning messages apply to the synth instance (global per instance), not to channels. Also mention: MTS has Real‑Time and Non‑Real‑Time variants, single‑note, scale (repeating degree lists), and bulk (128‑key) dumps. Use Non‑Real‑Time bulk dumps at time 0 of a track to preload a tuning for that instance.

- MPE nuance
  - MPE reserves a contiguous block of channels (a zone) with a master channel and member channels. Member channels are per‑note channels. MPE channel allocation (one note per channel) is the usual approach if you need independent per‑note controllers; it’s not mandatory to have one note per channel, but that is how many synths expect MPE inputs.

Minor wording nitpicks
- “If you use Note_off, it should have velocity 0” — change to: “Note_off may carry a release velocity; Note_on with velocity 0 is equivalent to Note_off and is commonly used.”

Obvious omissions you may want in the summary
- MIDI ports and multi‑port setups (how to exceed 16 channel addresses).
- Bank Select CC numbers (CC 0 and CC 32 conventionally used for MSB/LSB).
- The MIDI Port meta event in SMF and that many hosts ignore it in favor of explicit DAW routing.
- That tracks are merged by absolute time when playing a Type‑1 file; tempo/meta events affect playback globally (conventionally in the first track).
- RPN/NRPN and other per‑channel state (pitch‑bend range via RPN 0 is important for microtonal PB use).
- Running status and delta‑time encoding are implementation details that affect binary encoding but not musical semantics.

Refined “best practice” recommendations (compact)
- If your target is DAW import with one plugin instance per DAW track:
  - Default: create one SMF track per logical part, put that part’s events (notes, CC, Program Change) on a single channel within the track (e.g., channel 1). Reuse channel numbers across tracks freely — the DAW will route each track to its own instance and isolate the channels.
  - Place any MTS SysEx or instrument‑specific SysEx at time 0 in the track so the instance receives configuration before notes.
- If your target is a single multi‑timbral synth instance (or a renderer that expects channel addressing):
  - Allocate non‑overlapping channels for parts that must be distinguished by the synth; put Program Change/Bank Select on each channel as needed.
- If you need simultaneous independent pitch‑bend streams (per‑note pitch control):
  - Use MPE (one channel per sounding note) or intentionally allocate separate channels for each voice so pitch‑bend/RPN state doesn’t corrupt other voices.
- If you need more than 16 independent addressing streams:
  - Use multiple MIDI ports or multiple plugin instances.

Small example checklist to follow when generating files
- Use Type‑1 SMF for DAW import.
- Put tempo/time meta events in the header/tempo track (midicsv prints header as track 0; tempo track is usually track 1).
- Per musical part: one SMF track, one channel per track (unless targeting multi‑timbral synth), SysEx at time 0 for tuning or config, notes follow.
- Use Note_on(vel=0) or explicit Note_off as you prefer; be aware release velocities exist.
- Avoid channel 10 unless the part is percussion (or you intentionally want GM drum mapping).

If you want, I can:
- Tidy the summary into a slightly edited version you can paste into your notes (I’ll keep it short), or
- Produce two short midicsv examples showing the two generator modes (DAW‑friendly reuse of channel numbers across tracks, and multi‑timbral single‑instance with distinct channels). Which would you prefer?

ChatGPT(gpt-5-mini/General)> Revised version. In this case, if you mentioned something above as an omission and I have still not included it, it means it's either obvious to me or I have decided that it's irrelevant for my purposes. This time, I'd like your response to be focused on accuracy. No need to repeat omissions or offer changes in wording. I've got a vast number of summaries of this type, and I have refined over many years how I like to present information for my future self. This is the right level of detail for me, so I just want to make sure it doesn't contain misinformation at this point.

MIDI Summary

In a nutshell:
* An *event* that references a *channel* in a *track* gets mapped to a particular thing happening on a particular instrument as follows:
  * The track is routed to a target synth within the rendering environment (DAW, timidity, etc.)
  * The synth gets the program currently assigned to the channel along with other channel-specific information, like pitch bend
  * The synth interprets that program in its own way

## SMF (Standard Midi File)

* tool `midicsv` can map between a CSV file and a MIDI file; great for looking at/editing MIDI
* Each row of midicsv starts with a track number. `midicsv` shows a 0 in the track position for the first row, but this does not indicate a track 0. The first track is actually track 1.
* file type is 0, 1, or 2. Type 1 is what we care about: multiple, time-synchronized tracks

## Tracks, channels

A track is a sequence of events. A SMF can contain any number of tracks. A common convention is use track 1 for global metadata, tempo changes, time signatures, and to start with track 2 for notes, but it is not necessary to do this as global information can be in any track.

A track is just a container for organizing events. Tracks may appear in the DAW as separate DAW tracks and may be routed separately to different plugins/synths. Use different tracks for different logical voices.

A channel is an index within the PORT. They are numbered internally as 0 to 15 and referred to as 1 to 16; be careful about off-by-one. A channel gets a program via a program change message. The program effectively says what instrument sound notes get on that channel at that time. If you want more than 16 channels, use more than one port. There are also banks that can be selected with bank select. Programs live in banks. Program/bank semantics are dependent on the receiving synth/sound font.

Within the DAW (or rendering system), there is a many-to-many mapping between tracks and destinations. The destination interprets the meaning of the channel. If two tracks both use channel 1, their channel 1 instances will sound the same if the tracks are routed to the same destination instance. For maximum flexibility for use of multi-timbral synths (that allow different channels to be mapped to different instruments), use non-overlapping channel numbers within tracks, but reuse of channel numbers is normal and works fine if tracks are routed to different instances.

There are certain channel-wide settings, such as pitch bend. If using MPE, you would distribute notes across channels based on pitch bend information, and all the channels would likely have the same "program" (instrument sound). If you wanted to change timbre within a voice (e.g. adding a mute), you would usually send a program change command for the channel rather than using a different channel.

A good DAW-friendly strategy:
* Use a separate track per voice
* Use a separate, non-overlapping channel per track

This will work well with single-timbre or multi-timbre synths. Reasons to deviate:
* If you are using channels for MPE, each track will likely need its own channel space
* If you have more than 16 instruments, it will be necessary to route different tracks to different places

The mapping of track/channel to actual instrument is a function of where the track is routed and how the destination interprets the program mapped to the specific channel.

An alternative DAW strategy is to assume isolated channel numbers per track. This scales better for more instrument sounds but requires each track to be routed separately and is thus potentially more work to set up.

## Other notes

It is common to use "note on" with velocity 0 to turn a note off. Note on with velocity 0 is equivalent to note off with velocity 0, but some synths support the idea of a release velocity, in which case note off has a distinct meaning.

Channel 10 (from 1, 9 from 0) is usually percussion. For percussion, note numbers map to different drum sounds rather than different pitches, and pitch bend information is often ignored. Avoid channel 10 for non-percussion use.

MTS-ESP allows tuning banks to be loaded within a track. MTS tuning banks apply to all instruments in the instance, so for two tracks to meaningfully use different tunings, they have to be routed to different instances. MTS supports single-note, scale (with repetition), and bulk (all 128 notes) settings as well as real-time and non-real-time. Use non-real-time bulk up front to assign a frequency to each note. Absolute frequency is not the only method, but it is the most reliable and simplest.

There are other nuances of MPE (zones with master and member channels); come back to this if I ever want to use MPE. For DAW-friendly, microtonal note editing, MTS will be much more ergonomic.

<shell-maker-end-of-prompt>
This is accurate overall — nicely concise and well focused. A couple of narrowly factual corrections/clarifications only (no style or omission changes):

1) Program Change and already‑sounding notes  
- Program Change selects the program for a channel, but whether it affects voices already sounding on that channel is implementation dependent. Many synths only apply the new program to subsequently triggered notes; some change existing voices in place. So don’t assume an instantaneous timbre swap for held notes.

2) Note_on velocity 0 vs Note_off  
- Per the MIDI spec, Note_on with velocity=0 is defined as equivalent to Note_off. However, Note_off events can carry a release velocity value (nonzero), which some synths use. So Note_on(0) ≡ Note_off semantically, but Note_off can convey additional release‑velocity info.

3) MTS SysEx scope (small wording nitpick)  
- Placing MTS SysEx in a track is the right way to deliver tuning to a synth instance, but remember the SysEx effects are on the receiving device/instance (not “scoped” to the track itself). Practically: put the Non‑Real‑Time bulk (or other tuning SysEx) at time 0 of the track so the plugin/instance receives it before notes play.

Everything else in your summary is correct as written.

ChatGPT(gpt-5-mini/General)> I've made a few minor tweaks based on your latest feedback. The only thing I'm unclear about is banks. I think I've captured it enough, but just so I understand, my assumption is that the program number for program change is limited in size (7 bits?) so if you had a sound font with more than 128 things in it (or whatever the limit is), you would use banks? How are banks used in practice? Might you have a synth that had a bunch of piano sounds, guitar sounds, brass sounds, etc., all in separate banks, with program numbers for specific instruments numbered distinctly within each bank? If that's how they're used, it would match my intuition, and I don't need to say anything further about it in my summary.
<shell-maker-end-of-prompt>
Your intuition is basically right. Concise, accurate points about Program Change and banks:

- Program Change size: Program Change is a 7‑bit value, so program numbers range 0–127 (humanly shown 1–128 in many UIs). That alone gives you 128 programs per “bank.”

- Bank Select concept: To address more than 128 programs, MIDI uses Bank Select to choose a bank, then Program Change selects a program within that bank. Bank Select is implemented with two Control Change messages:
  - CC#0 = Bank Select MSB (7 bits, 0–127)
  - CC#32 = Bank Select LSB (7 bits, 0–127)
  Together they form a 14‑bit bank index (0..16383) if the receiver uses both MSB and LSB.

- Typical usage/order: Send Bank Select MSB (CC#0), then Bank Select LSB (CC#32) if needed, then send Program Change. Many devices only consult MSB and ignore LSB; many accept only CC#0 and omit CC#32. Best practice: set MSB, then LSB (if used), then Program Change.

- Practical semantics (how synths use it):
  - A synth/sampler interprets bank+program according to its own mapping. Commonly, banks partition instrument families (e.g., several piano banks, several guitar banks, percussion banks) and program numbers index specific presets within that bank.
  - SoundFonts (SF2) and many samplers implement banks this way: a bank selects a collection/zone/group and program selects a preset inside it. By convention many SoundFonts use bank MSB = 128 (or other bank numbers) for percussion/drum kits, but this is a convention of SF2/GM mapping, not a universal rule.
  - Some devices implement extra conventions (e.g., treat certain MSB values as GM banks, use LSB for articulations/variations, or use completely different SysEx-based bank schemes). Some synths ignore bank select entirely and only heed Program Change.

- Edge cases & compat details:
  - If the device only uses MSB, you effectively have 128 banks × 128 programs but only 128 useful banks (0–127) because LSB is unused.
  - If the device supports both MSB and LSB, you get up to 16384 distinct bank+program combinations.
  - Some DAWs/plugins present a “patch” dialog that sends CC#0/32 and Program Change for you; others may expose arbitrary mapping in the plugin UI.
  - There are vendor-specific/sysex ways to select presets; some hardware requires SysEx for preset selection (especially for large/specialized patch libraries).

- Practical recommendation (if you care about interoperability):
  - When you need >128 presets, use Bank Select MSB (CC#0) and then Program Change. Optionally send CC#32 (LSB) for finer selection, but be aware many targets ignore LSB.
  - Document the intended bank/program mapping for your user (or embed it in meta text/SysEx), because the actual sound produced depends on the target synth/soundfont.

So yes: think of “banks” as the coarse partition and Program Change as the index within that partition. Most common real‑world setups: several banks grouping similar instruments, program numbers 0–127 inside each bank, with many synths only paying attention to the MSB bank.

ChatGPT(gpt-5-mini/General)> Other than reserving channel 10 for percussion, if not using formal MPE (since pitch bend can still be used without MPE), are there other channels to avoid? Any reason, for example, to avoid channel 0?
<shell-maker-end-of-prompt>
Short answer: No — there’s nothing special about channel 0 that you must avoid. Beyond the GM percussion convention (channel 10, i.e. human channel 10 / zero‑based 9) there are no universally reserved channels in the MIDI spec. Any special treatment of particular channels is device/implementation specific, not a general rule.

A few precise points and caveats to keep in mind:

- Channel numbering: MIDI voice messages carry a 4‑bit channel 0–15. UIs usually display them as 1–16. When people say “channel 10” they mean human 10 (zero‑based 9). When you see dumps from libraries/tools they may show 0–15 — mind the off‑by‑one.

- The one broad convention to avoid for pitched material is channel 10 (human 10, zero‑based 9): General MIDI treats it as the percussion/drum channel, where note numbers map to drum sounds rather than pitches. If you don’t want drum semantics, don’t put pitched parts there (or explicitly select a non‑percussion bank/program).

- MPE considerations: MPE defines a contiguous block of channels (a zone) with a master channel and member channels. If you will interoperate with MPE‑aware instruments, be aware your chosen channel numbers might fall into an MPE zone expected by that synth. If you’re not using MPE, this is not a problem — but if you later route tracks into an MPE destination, you may need to remap channels to avoid colliding with an MPE zone.

- Device/host conventions trump general rules: some synths, soundfonts, or studio setups may reserve or expect specific channel mappings (e.g., a workstation that assumes channel 1 ↦ patch A, channel 2 ↦ patch B). Those are local conventions — not MIDI protocol constraints. When targeting such a device you must follow its mapping.

- Bank/program interplay: if a synth treats a channel as percussion via Bank Select/Program or some other flag, you can still use channel 10 for pitched notes provided you send the appropriate Bank Select/Program Change (but many GM‑style hosts will treat channel 10 as percussion by default).

- System messages are separate: System Exclusive, System Common, and System Real‑Time messages are not per‑channel; their behavior is independent of the 16 voice channels.

Practical recommendations
- Don’t avoid channel 0. It’s perfectly fine to use channels 0–15 (human 1–16) as you like.
- Avoid using channel 10 for pitched parts if you’re targeting a GM/standard soundfont or unknown synths.
- If you plan to support both non‑MPE and MPE workflows, document or make configurable your channel allocations so the DAW user can remap if needed.
- If targeting a particular synth or soundfont, follow that synth’s recommended channel/bank mappings (they override generic advice).

If you tell me your intended target(s) (e.g., Pianoteq, Surge‑XT, a particular SF2), I can call out any vendor conventions to avoid.

ChatGPT(gpt-5-mini/General)> Do you know if Studiologic Numa instruments understand MTS?
<shell-maker-end-of-prompt>
I don’t have a definitive, up‑to‑date list that says “All Studiologic Numa models support MTS” — models differ (some Numa products are controllers only, others have internal sound engines) and vendor firmware/feature sets change. So I can’t truthfully assert “yes” or “no” for your particular unit without the exact model and firmware.

How you can determine it (fast, practical checks)

1) Read the manual / product page
- Search the model’s manual for keywords: “MTS”, “MIDI Tuning Standard”, “Scala”, “microtuning”, “tuning”, “SysEx”, “bulk tuning”, or “tuning table”.
- Also look at the editor application (if Studiologic supplies one) — an editor that imports .scl or has “tuning” menus is a strong hint.

2) Look for Scala support or “tuning” dialogs in the editor/UI
- If the instrument/editor advertises Scala (.scl/.kbm) import or an on‑keyboard tuning menu, that’s a practical sign it supports retuning — often via MTS or a vendor SysEx equivalent.

3) Empirical test (recommended)
- Send a known MTS SysEx tuning message (single‑note or small bulk/scale) to the device and see whether the pitch changes. This is the most definitive test.

On Linux you can do this reliably:
- Find the device port:
  - aconnect -l
  - amidi -l
- Monitor incoming MIDI while you send the message:
  - aseqdump -p "YourDevice:PortName"
- Send the SysEx via a DAW (route a MIDI item with the SysEx at time 0 to the device) or use a CLI tool (amidi can send a hex SysEx string to a device).

A good minimal test is:
- Create a tiny SMF (or midicsv) that contains:
  - a Non‑Real‑Time MTS single‑note tuning change or a very small bulk/scale message at t=0
  - then a sustained Note_on for a single key (e.g., middle C)
- Send/import that file, and listen/watch for the pitch shift.

If you want, I can:
- produce exact MTS SysEx payloads for testing (single‑note, scale, or a tiny bulk dump) and a midicsv snippet you can csvmidi → .mid and then send to the keyboard or import into your DAW.
- give you the precise amidi/arecordmidi/aseqdump commands to run the test on Linux for your MIDI port.

4) Check for vendor SysEx alternatives
- Some instruments don’t implement MTS but have a vendor SysEx protocol for tuning or accept Scala via their editor. If you see any SysEx upload features in the editor, capture/inspect what they send — that’s the device’s tuning API.

5) Ask Studiologic or community
- If the manual is ambiguous, a quick email to Studiologic support or a search/post in user forums for the exact model name + “MTS” / “Scala” will usually give a clear answer.

If you want to proceed now, tell me:
- the exact Studiologic model (e.g., “Numa X”, “Numa Concert”, or exact name), and
- whether you prefer a quick single‑note MTS test or a 128‑note bulk dump test,

and I’ll generate the appropriate SysEx bytes and the Linux/CSV test steps you can paste and run.

ChatGPT(gpt-5-mini/General)> 