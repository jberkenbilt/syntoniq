+++
title = "Hardware"
weight = 10
sort_by = "weight"
+++

* Supported devices:
  * [Novation Launchpad MK3 Pro](https://novationmusic.com/products/launchpad-pro-mk3)
  * [HexBoard MIDI Controller](https://shapingthesilence.com/tech/hexboard-midi-controller/)
* Have custom HexBoard firmware as download and provide instructions, noting that the HexBoard team plans to include the changes in the next firmware release. As of HexBoard firmware version 1.2, custom firmware is required. It is expected that the firmware mods will be incorporated into the main firmware. In the meantime, the changes are at <https://github.com/jberkenbilt/HexBoard/tree/delegated-control>; check status at <https://github.com/shapingthesilence/HexBoard/pull/6>.
* Show csound and MIDI output
* Mention MIDI system Requirements (loopmidi/syntoniq-loop on Windows, a synth, etc.)
* Show how to use Syntoniq as a MIDI input device and how this may require disabling input from the hardware to avoid hearing two notes (select input from the syntoniq device, not the hardware device)
  * Syntoniq creates various virtual MIDI ports. On Windows, use [loopMIDI](https://www.tobias-erichsen.de/software/loopmidi.html) and create a port called `syntoniq-loop`.
  * On Linux, you can watch Syntoniq's MIDI output with `aseqdump`, e.g.:
    ```sh
    aconnect -l
    aseqdump -p 128:0
    ```
