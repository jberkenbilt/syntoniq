+++
title = "Hardware"
weight = 10
sort_by = "weight"
+++

The Syntoniq keyboard works with the following hardware keyboards:
* [Novation Launchpad MK3 Pro](https://novationmusic.com/products/launchpad-pro-mk3)
* [HexBoard MIDI Controller](https://shapingthesilence.com/tech/hexboard-midi-controller/)

The Launchpad MK3 Pro works "out of the box" with the Syntoniq keyboard.

As of version 1.2 of the HexBoard firmware (released October 23, 2025), the features required by the `syntoniq-kbd` application are not yet included in the official firmware, but the HexBoard maintainer has indicated his intention to include it in the next official release. In the meantime, you can use this [unofficial firmware](../../HexBoard_syntoniq.uf2). This was built from the changes on [GitHub in the syntoniq branch of jberkenbilt/HexBoard](https://github.com/jberkenbilt/HexBoard/tree/syntoniq), which includes preliminary support for device identity and addition of a "delegated control" mode. The changes were submitted for inclusion on this [pull request](https://github.com/shapingthesilence/HexBoard/pull/6). To use the unofficial firmware, [download the unofficial firmware](../../HexBoard_syntoniq.uf2) and follow the instructions on the [HexBoard GitHub Repository page](https://github.com/shapingthesilence/HexBoard?tab=readme-ov-file#flashing-the-firmware). The version will show as `1.2.0+q1`.

Once you have the device connected, you can run the keyboard application using the `syntoniq-kbd` command. Run `syntoniq-kbd --help` for help.

TODO: finalize demo.stq and fix commands below

When you run `syntoniq-kbd`, it scans the list of available MIDI devices and picks the first one whose name contains the string you pass as the option to `--port`. If it doesn't find one, it will tell you which ports it found. You can run one of the following

```sh
# for Launchpad
syntoniq-kbd run --port=MK3 --score=keyboard/configs/demo.stq
# for HexBoard
syntoniq-kbd run --port=RP2040 --score=keyboard/configs/demo.stq
```

## Using Syntoniq Keyboard with MIDI

By default, `syntoniq-kbd` links with the Csound library and uses Csound for its output. As an alternative, you can make the keyboard behave as a MIDI output device and connect it to your Digital Audio Workstation (DAW) or a synth such as [VITAL](https://vital.audio/) or [Surge XT](https://surge-synthesizer.github.io/). You might want to do this if you prefer to pick your own sound over the functional but dry "study tone" Syntoniq uses by default. If you find that Csound doesn't work well for you, or you built `syntoniq-kbd` yourself and disabled Csound, you can also use the MIDI feature. Syntoniq sends MPE-compliant (MPE = MIDI Polyphonic Expression) pitch bend commands and uses a standard MPE-compliant channel allocation strategy, so you should enable MPE if you have that option, and you should listen on all channels. If you do not use a synth/instrument capable of receiving MIDI pitch bend commands, you will not hear the correct pitches with any tuning other than 12-EDO.

If you use the Syntoniq keyboard as a MIDI device, there are several things to know:

* The `syntoniq-kbd` application creates a MIDI port called "Syntoniq Keyboard". In your synthesizer, you will see that input device, and you will also see the input device for the hardware. At the time of this writing, the Launchpad presents itself as "Launchpad Pro MK3", and the HexBoard presents as "RP2040". The hardware is sending MIDI commands, which `syntoniq-kbd` receives. In MIDI mode, `syntoniq-kbd` sends out its own MIDI note events. *You must unselect the hardware device in your synthesizer and select "Syntoniq Keyboard", or else you will hear two notes: one sent by the hardware and one sent by `syntoniq-kbd`.
* On Linux and macOS, you shouldn't have to do anything special to use `syntoniq-kbd` as a MIDI device. On Windows, you need to install [loopMIDI](https://www.tobias-erichsen.de/software/loopmidi.html) and create a port called `syntoniq-loop`. The "Syntoniq Keyboard" device writes its output to that port since Windows doesn't support dynamic creation of virtual MIDI devices the way macOS and Linux do.

# Video Demonstration

This video shows
* Starting the keyboard using the built-in demo configuration with Csound
* Selecting the first layout
* Playing a few notes
* Starting the keyboard using the built-in demo configuration with MIDI
* Selecting the first layout
* Playing a few notes

{{ youtube(id="Oc_HkZVupjw?si=ObzOvNzB7gsY_KcD", caption="TODO Placeholder Video", script="TODO") }}
