;; This file is a copy of csound-template.csd with the instrument name
;; changed to "potato" and replaced with an audibly different sound.
;; There's a Reverb instrument for exercising global instrument logic.

<CsoundSynthesizer>

<CsOptions>
-odac
</CsOptions>

<CsInstruments>

sr = 44100
ksmps = 32
nchnls = 2
0dbfs = 1

; Each part has associated channels:
; - p<n>_amp -- a volume level from 0 to 1 inclusive
; - p<n>_notes -- the maximum number of notes ever "on" for the part's instrument
; These are set using the "SetPartParam" and "SetPartParamRamp" control
; instruments.

instr SetPartParam
  iPartNum = p4
  SParam = p5
  iValue = p6
  SChan sprintf "p%d_%s", iPartNum, SParam
  chnset iValue, SChan
endin

instr SetPartParamRamp
  iDuration = p3
  iPartNum = p4
  SParam = p5
  iStart = p6
  iEnd = p7
  SChan sprintf "p%d_%s", iPartNum, SParam
  kValue expseg iStart, iDuration, iEnd
  chnset kValue, SChan
endin

; A single instrument may be used for multiple parts. Any additional
; instrument must accept the same parameters to be a target for
; syntoniq notes. By design, the instrument's parameters only include
; required parameters (instrument, start time, duration) and
; identification of part and note numbers. This allows arbitrary new
; parameters or changes in behavior, such as ramping previously
; constant values, without breaking backward compatibility.

; Global audio buses for reverb send
ga_rev_L init 0
ga_rev_R init 0

instr Reverb
  ; reverbsc parameters
  iRevFeedback = 0.65
  iRevCutoff   = 6000   ; internal HF damping

  ; Equalizer parameters
  iPeakFreq    = 401     ; resonant peak frequency
  iPeakGain    = 12
  iPeakQ       = 0.7     ; peak bandwidth

  aRevL, aRevR reverbsc ga_rev_L, ga_rev_R, iRevFeedback, iRevCutoff

  ; Resonant peak boost
  aRevL pareq aRevL, iPeakFreq, ampdb(iPeakGain), iPeakQ, 0
  aRevR pareq aRevR, iPeakFreq, ampdb(iPeakGain), iPeakQ, 0

  outs aRevL, aRevR

  ga_rev_L = 0
  ga_rev_R = 0
endin

instr potato
  ; p1..p3 are always instrument, start time, duration
  iPartNum = p4
  iNoteNum = p5
  iVelocity = p6 ; 0 to 1

  ; Oscillator mix proportions
  iTriMix   = 1.00
  iPulseMix = 0.26
  iSawMix   = 0.22

  ; Pulse width
  iPulseWidth = 0.5257

  ; Unison: 4 voices with detuning and stereo pan positions
  ; Two voices at pitch, two detuned.
  ; Pan values: 0 = hard left, 0.5 = center, 1 = hard right
  iDetune1 = 0
  iDetune2 = 0
  iDetune3 = 6
  iDetune4 = -6
  iPan1 = 0.35
  iPan2 = 0.65
  iPan3 = 0.2
  iPan4 = 0.8

  ; Filter
  iFilterCutoff = 1600

  ; Reverb send level (0 to 1)
  iRevSend = 0.4

  SFreqChan sprintf "p%d_freq_%d", iPartNum, iNoteNum
  SAmpChan sprintf "p%d_amp", iPartNum
  SNotesChan sprintf "p%d_notes", iPartNum
  kBaseVol chnget SAmpChan
  kNoteCount chnget SNotesChan
  kFreq chnget SFreqChan

  kNoteCount = (kNoteCount == 0 ? 1 : kNoteCount)
  kAmp = kBaseVol * iVelocity
  ; Attenuate based on polyphony
  kFinalAmp = kAmp / sqrt(kNoteCount)
  aEnv madsr 0.05, 0.05, 0.9, 0.15

  ; Oscillator: 4 unison voices x 3 waveforms, panned in stereo
  kFreq1 = kFreq * cent(iDetune1)
  kFreq2 = kFreq * cent(iDetune2)
  kFreq3 = kFreq * cent(iDetune3)
  kFreq4 = kFreq * cent(iDetune4)

  ; Voice 1
  aTri1   vco2 iTriMix,   kFreq1, 12
  aPulse1 vco2 iPulseMix, kFreq1, 2, iPulseWidth
  aSaw1   vco2 iSawMix,   kFreq1, 0
  aVoice1 = aTri1 + aPulse1 + aSaw1

  ; Voice 2
  aTri2   vco2 iTriMix,   kFreq2, 12
  aPulse2 vco2 iPulseMix, kFreq2, 2, iPulseWidth
  aSaw2   vco2 iSawMix,   kFreq2, 0
  aVoice2 = aTri2 + aPulse2 + aSaw2

  ; Voice 3
  aTri3   vco2 iTriMix,   kFreq3, 12
  aPulse3 vco2 iPulseMix, kFreq3, 2, iPulseWidth
  aSaw3   vco2 iSawMix,   kFreq3, 0
  aVoice3 = aTri3 + aPulse3 + aSaw3

  ; Voice 4
  aTri4   vco2 iTriMix,   kFreq4, 12
  aPulse4 vco2 iPulseMix, kFreq4, 2, iPulseWidth
  aSaw4   vco2 iSawMix,   kFreq4, 0
  aVoice4 = aTri4 + aPulse4 + aSaw4

  ; Pan each voice and sum into stereo mix, normalize by 4 voices
  aMixL = (aVoice1 * (1 - iPan1) \
         + aVoice2 * (1 - iPan2) \
         + aVoice3 * (1 - iPan3) \
         + aVoice4 * (1 - iPan4)) * 0.25
  aMixR = (aVoice1 * iPan1 \
         + aVoice2 * iPan2 \
         + aVoice3 * iPan3 \
         + aVoice4 * iPan4) * 0.25

  kFilterCutoff = iFilterCutoff + (kFreq * 0.5)

  ; Filter
  aFilteredL butterlp aMixL, kFilterCutoff
  aFilteredR butterlp aMixR, kFilterCutoff

  ; Amplitude and envelope
  aOutL = aFilteredL * aEnv * kFinalAmp
  aOutR = aFilteredR * aEnv * kFinalAmp

  ; Output: dry to speakers, copy to reverb bus
  aDryL = aOutL * (1 - iRevSend)
  aDryR = aOutR * (1 - iRevSend)
  outs aDryL, aDryR

  ga_rev_L = ga_rev_L + aOutL * iRevSend
  ga_rev_R = ga_rev_R + aOutR * iRevSend
endin

</CsInstruments>
<CsScore>

; i instr start duration [params...]

;; NOTE: for comments that end with @nnn, nnn is the byte offset of
;; the item in the original file.

;; BEGIN SYNTONIQ

; Generated data goes here

;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
