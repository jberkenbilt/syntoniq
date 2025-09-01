<CsoundSynthesizer>

<CsOptions>
-odac
</CsOptions>

<CsInstruments>
sr = 44100
ksmps = 32
nchnls = 1
0dbfs = 1

instr SetChan  ; contoller instrument: sets a channel's value
  iValue = p4
  SChannelName = p5
  chnset iValue, SChannelName
endin

instr 1
  iFreq = p4
  kAmp chnget "amp"

  ; Amplitude is controlled by the channel now
  aTone oscil3 kAmp, iFreq, 1
  aVoc moogladder aTone, 2000, 0.1
  out aVoc
endin

</CsInstruments>

<CsScore>
f 0 31536000 ; keep csound running until stopped or this number of seconds elapses
f 1 0 32768 10 1 .8 .6 .4 .2 .2 .1

; Set the amplitude initially to 1. It can be reset.
i "SetChan" 0 -1 .4 "amp"

; Remaining score events come from the API.

e
</CsScore>

</CsoundSynthesizer>
