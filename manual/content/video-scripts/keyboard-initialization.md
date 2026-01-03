+++
title = "Keyboard: Initialization"
weight = 10
sort_by = "weight"
+++

[This video accompanies [SYNTONIQ KEYBOARD -> Initialization](../../keyboard/initialization/).]

[Present on the screen: a video of one of the controllers when in use, the web UI, and a terminal window.]

This video shows you how to start the Syntoniq keyboard application with both the Novation Launchpad Pro Mark 3 and the HexBoard MIDI controller. I will demonstrate how to start the keyboard from the command line, how to select a layout, and playing a few notes using both the default Csound output mode and the MIDI output. Please note that there is a lot of detail in the display. We show a web UI and a terminal window. You will likely want to watch this video on a full-sized display.

Let's get started with the Launchpad.

The Launchpad has a midi port called "Launchpad Pro Mark 3". When you run the Syntoniq keyboard application, you have to specify a string that appears in the port name. We can get a list of valid ports by giving any invalid port name.

[Type `syntoniq-kbd run --port=potato`]

Here, you can see that I have typed a port name that doesn't correspond to any of my MIDI devices. The output shows you the Launchpad ports as well as the RP2040 port, which is the HexBoard. Future versions of the HexBoard may present differently. Let's run this again. I'll just give "MK3" [em kay three] as the port.

[Type `syntoniq-kbd run --port=MK3`]

Notice the output tells you to exit with control C, gives you the location of the "view HTTP server", and tells you that the keyboard is initialized. Depending on the version of the software, you may see something slightly different. I'll also point out that I'm running the application with its built-in default configuration. A major feature of the Syntoniq keyboard is its flexible scale and layout system. These will be covered in a later video.

Here in my browser, I will connect to localhost port 8440. You can see a drawing of the keyboard with some extra labels and information. Here on the right, you can see an indication of the selected layout, currently empty, and a list of 9 possible layouts. Notice along the bottom that there are 8 numbered buttons. Pressing a button from 1 to 8 selects the corresponding layout. To get to layout 9, press the button labeled "Scroll layouts". [Press the button.] After you do this, you will notice that only the first LED is lit up in the layout selection row and that it is now labeled with the number 9. Pressing the scroll button again [Press the button] returns to the previous configuration.

This is a good time to mention that the Launchpad's keys are labeled for its intended use as a controller for Ableton Live, and it is not possible to change the labels programmatically. I have picked keys that are somewhat mnemonically related to their Syntoniq keyboard functions. The Scroll layouts key on the physical keyboard says "Print to Clip", but for us, it's "Scroll Layouts". The web UI is labeled with the Syntoniq keyboard's use of the key.

Let's select the first layout. [Press the layout 1 button.] You can now see that the lights have changed both on the keyboard itself and on the web display. I'll cover what the labels and colors mean in a subsequent video. For now, I'll just play a few notes to make sure everything's working. [Play C, D, E, F, G, then C-E-G triad.] Great.

A lot of information is being printed to the console here. I'll explain all that in a subsequent video.

Before I exit, I'm hitting the `Clear` key, which performs reset. [Hit Clear.] You can see that this brings us back to the logo screen. If we had specified a configuration file containing custom layouts, it would also reload that file.

Next, I'll show you the same thing with the HexBoard. I'm hitting Control C at the terminal to exit from the keyboard. [Hit CTRL-C.] Now, I'll start the keyboard application for the HexBoard. I'm just going to use RP as an abbreviation.

[Type `syntoniq-kbd run --port=RP`]

I'll refresh my browser [refresh browser], and now you can see the Syntoniq logo drawn on the HexBoard. It's quite a bit more recognizable here with the 133-button hexagonal grid than with the 8x8 rectangular grid of the Launchpad! For the HexBoard, the command keys are not shown in the web UI, but you can see here at the top right that there is a list of command key functions from top to bottom. The second key is labeled "Select Layout." When I press that, all the lights turn off except the top row and part of the second row. This is reflected on the web UI as well, where the buttons are labeled with numbers. These numbers indicate which layout will be selected. I'll press the upper-left button [press the button] to select the first layout. Now I'm ready to play some notes. Play [C, G E C', adding notes to the chord.] There. While the Launchpad is a great device, the HexBoard gives you a lot more keys and a more compact and convenient layout. I highly recommend the HexBoard. You can get yours at shaping the silence dot com. You can find links in the Syntoniq manual at syntoniq (that's s-y-n-t-o-n-i-q) dot cc.

I'll show you the reset function, bound to the topmost command key [press reset], which you can see brings us back to the logo screen.

I'm going to hit control C again to get out. Before we wrap it up, I'll show you how you can use the Syntoniq keyboard as a MIDI keyboard. For this, I'm running Surge XT, a great free synthesizer that you can easily find with a web search. To start in midi mode, I use the same command but pass the dash dash MIDI option on the command line.

[Type `syntoniq-kbd run --port=RP --midi`]

Here's my Surge XT window. [Switch video focus to Surge XT window.] I'm running this as a stand-alone application in Linux, but you can also run it inside a digital audio workstation, and there are other tools you could use instead. If you just start pressing keys now, there's a good chance you'll either hear nothing, the wrong note, or two notes at once. This is because the actual hardware itself is a MIDI device, and the Syntoniq keyboard is also a MIDI device. For this to work, you need to make sure you go into your options and select "Syntoniq Keyboard" as your active MIDI input and clear the actual hardware device. You can see that I've done that here. [Open audio options and show it.] For the patch, I've chosen the Analog Brass patch by Luna. Now when I press keys [press some keys], I hear sounds from this excellent patch. If you are accustomed to using MIDI keyboards through a synthesizer or audio workstation, you may prefer this as the sound quality will likely be much better than my "study tone" Csound instrument. Please keep in mind that Csound is an extremely capable tool, and you should not judge the tool by the quality of the default sound used by the Syntoniq keyboard. If you want better sounds with the keyboard, use MIDI. For the Syntoniq language, you can use your own Csound instruments in place of the study tone.

I hope you've enjoyed this introduction. Please check out the Syntoniq manual and other videos to see the rest of the features of the keyboard, and remember: the keyboard is only half the story. Syntoniq comes with a language that lets you create microtonal scores, so if that's your thing, be sure to visit syntoniq.cc for the whole story.
