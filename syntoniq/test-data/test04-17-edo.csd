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
; [part] => csound part
; [p1] => 1
; [part.note] => instr.note
; [p1.0] => "potato".3
; [p1.1] => "potato".2
; [p1.2] => "potato".1

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 3
t 0 72
; 6:c,2@472
i "SetPartParam" 0 6 1 "freq_1" 65.406
; 6:c,2 @472
i "potato.1" 0 6 1 1 0.567
; 1:g,@456
i "SetPartParam" 2 1 1 "freq_2" 196.665
; 1:g, @456
i "potato.2" 2 1 1 2 0.567
; 3:g@441
i "SetPartParam" 3 3 1 "freq_3" 393.33
; 3:g @441
i "potato.3" 3 3 1 3 0.567
; 3:c@461
i "SetPartParam" 3 3 1 "freq_2" 261.626
; 3:c @461
i "potato.2" 3 3 1 2 0.567
; 6:c,2@532
i "SetPartParam" 6 6 1 "freq_1" 65.406
; 6:c,2 @532
i "potato.1" 6 6 1 1 0.567
; 1:g,@510
i "SetPartParam" 7 1 1 "freq_2" 196.665
; 1:g, @510
i "potato.2" 7 1 1 2 0.567
; a%,@515
i "SetPartParam" 8 1 1 "freq_2" 213.374
; a%, @515
i "potato.2" 8 1 1 2 0.567
; 3:g@495
i "SetPartParam" 9 3 1 "freq_3" 393.33
; 3:g @495
i "potato.3" 9 3 1 3 0.567
; c@519
i "SetPartParam" 9 1 1 "freq_2" 261.626
; c @519
i "potato.2" 9 1 1 2 0.567
; 2:d@521
i "SetPartParam" 10 2 1 "freq_2" 295.667
; 2:d @521
i "potato.2" 10 2 1 2 0.567
; 6:c,2@572
i "SetPartParam" 12 6 1 "freq_1" 65.406
; 6:c,2 @572
i "potato.1" 12 6 1 1 0.567
; 1:g,@550
i "SetPartParam" 13 1 1 "freq_2" 196.665
; 1:g, @550
i "potato.2" 13 1 1 2 0.567
; a%,@555
i "SetPartParam" 14 1 1 "freq_2" 213.374
; a%, @555
i "potato.2" 14 1 1 2 0.567
; d@559
i "SetPartParam" 15 1 1 "freq_2" 295.667
; d @559
i "potato.2" 15 1 1 2 0.567
; e@561
i "SetPartParam" 16 1 1 "freq_2" 334.138
; e @561
i "potato.2" 16 1 1 2 0.567
; d@563
i "SetPartParam" 17 1 1 "freq_2" 295.667
; d @563
i "potato.2" 17 1 1 2 0.567
; 5:c,2@615
i "SetPartParam" 18 5 1 "freq_2" 65.406
; 5:c,2 @615
i "potato.2" 18 5 1 2 0.567
; 1:g,@590
i "SetPartParam" 19 1 1 "freq_3" 196.665
; 1:g, @590
i "potato.3" 19 1 1 3 0.567
; a%,@595
i "SetPartParam" 20 1 1 "freq_3" 213.374
; a%, @595
i "potato.3" 20 1 1 3 0.567
; d@599
i "SetPartParam" 21 1 1 "freq_3" 295.667
; d @599
i "potato.3" 21 1 1 3 0.567
; e@601
i "SetPartParam" 22 1 1 "freq_3" 334.138
; e @601
i "potato.3" 22 1 1 3 0.567
; 4:c#@603
i "SetPartParam" 23 4 1 "freq_3" 272.513
; 4:c# @603
i "potato.3" 23 4 1 3 0.567
; 4:b%,3@632
i "SetPartParam" 23 4 1 "freq_2" 60.284
; 4:b%,3 @632
i "potato.2" 23 4 1 2 0.567
; global instruments
i "Reverb" 0 30
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
