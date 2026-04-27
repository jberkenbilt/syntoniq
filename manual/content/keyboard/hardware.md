+++
title = "Hardware"
weight = 10
sort_by = "weight"
+++

The Syntoniq keyboard works with the following hardware keyboards:
* [Novation Launchpad MK3 Pro](https://novationmusic.com/products/launchpad-pro-mk3)
* [HexBoard MIDI Controller](https://shapingthesilence.com/tech/hexboard-midi-controller/)

The Launchpad MK3 Pro works "out of the box" with the Syntoniq keyboard. The HexBoard also works out of the box with firmware version 1.3 or later.

Once you have the device connected, you can run the keyboard application using the `syntoniq-kbd` command. Run `syntoniq-kbd --help` for help.

When you run `syntoniq-kbd`, it scans the list of available MIDI devices and picks the first one whose name contains the string you pass as the option to `--port`. If it doesn't find one, it will tell you which ports it found. You can run one of the following

```sh
# for Launchpad
syntoniq-kbd run --port=MK3
# for HexBoard
syntoniq-kbd run --port=RP2040
```

## Using Syntoniq Keyboard with MIDI

By default, `syntoniq-kbd` links with the Csound library and uses Csound for its output. As an alternative, you can make the keyboard behave as a MIDI output device and connect it to your Digital Audio Workstation (DAW) or a synth such as [VITAL](https://vital.audio/) or [Surge XT](https://surge-synthesizer.github.io/). You might want to do this if you prefer to pick your own sound over the functional but dry "study tone" Syntoniq uses by default. If you find that Csound doesn't work well for you, or you built `syntoniq-kbd` yourself and disabled Csound, you can also use the MIDI feature. Syntoniq sends MPE-compliant (MPE = MIDI Polyphonic Expression) pitch bend commands and uses a standard MPE-compliant channel allocation strategy, so you should enable MPE if you have that option, and you should listen on all channels. If you do not use a synth/instrument capable of receiving MIDI pitch bend commands, you will not hear the correct pitches with any tuning other than 12-EDO.

If you use the Syntoniq keyboard as a MIDI device, there are several things to know:

* The `syntoniq-kbd` application creates a MIDI port called "Syntoniq Keyboard". In your synthesizer, you will see that input device, and you will also see the input device for the hardware. At the time of this writing, the Launchpad presents itself as "Launchpad Pro MK3", and the HexBoard presents as "RP2040". The hardware is sending MIDI commands, which `syntoniq-kbd` receives. In MIDI mode, `syntoniq-kbd` sends out its own MIDI note events. *You must unselect the hardware device in your synthesizer and select "Syntoniq Keyboard", or else you will hear two notes: one sent by the hardware and one sent by `syntoniq-kbd`.
* On Linux and macOS, you shouldn't have to do anything special to use `syntoniq-kbd` as a MIDI device. On Windows, you need to install [loopMIDI](https://www.tobias-erichsen.de/software/loopmidi.html) and create a port called `syntoniq-loop`. The "Syntoniq Keyboard" device writes its output to that port since Windows doesn't support dynamic creation of virtual MIDI devices the way macOS and Linux do.

In the next section, we'll go over starting `syntoniq-kbd` and making sure the device works.
