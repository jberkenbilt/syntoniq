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
instr 1
  ; p1..p3 are always instrument, start time, duration
  iPartNum = p4
  iNoteNum = p5
  iVelocity = p6 // 0 to 1

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

  ; For most of the frequency range, we use a custom sound mixed with
  ; specific harmonics. At higher frequency ranges, we fall back to a
  ; sine/triangle mix for fewer artifacts.
  aMain poscil3 1, kFreq, 1

  ; blend sine and triangle
  aSine poscil3 0.9, kFreq
  aTriangle vco2 0.9, kFreq, 12
  aHigh = (aSine * 0.5) + (aTriangle * 0.5)

  ; For frequencies in the range of iLowThresh to iHighThresh,
  ; interpolate how much of the main mix we want. It drops to 0
  ; through that range.
  iLowThresh = 2000
  iHighThresh = 4000
  ; map iLowThresh, iHighThresh -> 1, 0 and clamp
  kInterp linlin kFreq, 1, 0, iLowThresh, iHighThresh
  kMainMix limit kInterp, 0, 1

  ; blend
  kHighMix = 1 - kMainMix
  aSignal = (aHigh * kHighMix) + (aMain * kMainMix) * aEnv * kFinalAmp
  aOut moogladder aSignal, 2000, 0.1
  outs aOut, aOut
endin

</CsInstruments>
<CsScore>

; function table for oscilator
f 1 0 32768 10 1 .4 .3 .2 .1 .05 .02

; i instr start duration [params...]

;; NOTE: for comments that end with @nnn, nnn is the byte offset of
;; the item in the original file.

;; BEGIN SYNTONIQ
; [part] => csound part
; [p1] => 1
; [part.note] => instr.note
; [p1.0] => 1.1

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 1
t 0 120 3 120 3 150 9 160 15 160 15 180 21 180 21 180
; 1:g@52
i "SetPartParam" 0 1 1 "freq_1" 391.995
; 1:g @52
i 1.1 0 1 1 1 0.567
; f@56
i "SetPartParam" 1 1 1 "freq_1" 349.228
; f @56
i 1.1 1 1 1 1 0.567
; d@58
i "SetPartParam" 2 1 1 "freq_1" 293.665
; d @58
i 1.1 2 1 1 1 0.567
; mark 'verse-start' @'[80,93)
; 1:c@161
i "SetPartParam" 3 1 1 "freq_1" 261.626
; 1:c @161
i 1.1 3 1 1 1 0.567
; e@165
i "SetPartParam" 4 1 1 "freq_1" 329.628
; e @165
i 1.1 4 1 1 1 0.567
; g@167
i "SetPartParam" 5 1 1 "freq_1" 391.995
; g @167
i 1.1 5 1 1 1 0.567
; f@169
i "SetPartParam" 6 1 1 "freq_1" 349.228
; f @169
i 1.1 6 1 1 1 0.567
; e@171
i "SetPartParam" 7 1 1 "freq_1" 329.628
; e @171
i 1.1 7 1 1 1 0.567
; d@173
i "SetPartParam" 8 1 1 "freq_1" 293.665
; d @173
i 1.1 8 1 1 1 0.567
; c@177
i "SetPartParam" 9 1 1 "freq_1" 261.626
; c @177
i 1.1 9 1 1 1 0.567
; e@179
i "SetPartParam" 10 1 1 "freq_1" 329.628
; e @179
i 1.1 10 1 1 1 0.567
; g@181
i "SetPartParam" 11 1 1 "freq_1" 391.995
; g @181
i 1.1 11 1 1 1 0.567
; 2:c@183
i "SetPartParam" 12 2 1 "freq_1" 261.626
; 2:c @183
i 1.1 12 2 1 1 0.567
; mark 'chorus-main-start' @'[212,231)
; 1:c@255
i "SetPartParam" 15 1 1 "freq_1" 261.626
; 1:c @255
i 1.1 15 1 1 1 0.567
; e@259
i "SetPartParam" 16 1 1 "freq_1" 329.628
; e @259
i 1.1 16 1 1 1 0.567
; g@261
i "SetPartParam" 17 1 1 "freq_1" 391.995
; g @261
i 1.1 17 1 1 1 0.567
; mark 'chorus-main-end' @'[274,291)
; 2:a@315
i "SetPartParam" 18 2 1 "freq_1" 440
; 2:a @315
i 1.1 18 2 1 1 0.567
; repeat start 'chorus-main-start' @'[350,369)
; 1:c@255
i "SetPartParam" 21 1 1 "freq_1" 261.626
; 1:c @255
i 1.1 21 1 1 1 0.567
; e@259
i "SetPartParam" 22 1 1 "freq_1" 329.628
; e @259
i 1.1 22 1 1 1 0.567
; g@261
i "SetPartParam" 23 1 1 "freq_1" 391.995
; g @261
i 1.1 23 1 1 1 0.567
; repeat end 'chorus-main-end' @'[374,391)
; 2:c@416
i "SetPartParam" 24 2 1 "freq_1" 261.626
; 2:c @416
i 1.1 24 2 1 1 0.567
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
