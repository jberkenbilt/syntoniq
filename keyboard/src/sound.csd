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

instr AmpControl
  iBaseVoices = 5

  ; number of currently-active instances of instr 1
  kN active 1
  ; peak-hold voice count while any notes are active
  kPeak init iBaseVoices
  if (kN <= 0) then
    kPeak = iBaseVoices
  else
    kPeak max kN, kPeak
  endif
  kAmpTarget = 0.6 / sqrt(kPeak)

  ; Glide to the new amplitude
  kAmp portk kAmpTarget, 0.05
  chnset kAmp, "amp"
endin

instr 1
  iFreq = p4
  kAmp chnget "amp"

  iRel = 0.15
  xtratim iRel
  aEnv madsr 0.05, 0, 1, iRel
  aMain poscil3 1, iFreq, 1
  aSine poscil3 1, iFreq
  aTriangle vco2 1, iFreq, 12
  aHigh = (aSine * 0.5) + (aTriangle * 0.5)

  iLowThresh = 2000
  iHighThresh = 4000
  iInterp linlin iFreq, 1, 0, iLowThresh, iHighThresh
  iMainMix limit iInterp, 0, 1

  iHighMix = 1 - iMainMix
  aSignal = ((aHigh * iHighMix) + (aMain * iMainMix)) * aEnv * kAmp
  aOut moogladder aSignal, 2000, 0.1
  outs aOut
endin

</CsInstruments>

<CsScore>
f 0 31536000 ; keep csound running until stopped or this number of seconds elapses
f 1 0 32768 10 1 .4 .3 .2 .1 .05 .02

; Start the amp control instrument.
i "AmpControl" 0 -1

; Remaining score events come from the API.

e
</CsScore>

</CsoundSynthesizer>
