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
; [p1.1] => 1.2

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 2
t 0 90
; 1:c @137
i "SetPartParam" 0 1 1 "freq_1" 261.626
i 1.1 0 1 1 1 0.567
; 31:c, @232
i "SetPartParam" 0 31 1 "freq_2" 130.813
i 1.2 0 31 1 2 0.567
; c+ @141
i "SetPartParam" 1 1 1 "freq_1" 267.541
i 1.1 1 1 1 1 0.567
; c# @144
i "SetPartParam" 2 1 1 "freq_1" 273.591
i 1.1 2 1 1 1 0.567
; d% @147
i "SetPartParam" 3 1 1 "freq_1" 279.777
i 1.1 3 1 1 1 0.567
; d- @150
i "SetPartParam" 4 1 1 "freq_1" 286.103
i 1.1 4 1 1 1 0.567
; d @153
i "SetPartParam" 5 1 1 "freq_1" 292.572
i 1.1 5 1 1 1 0.567
; d+ @155
i "SetPartParam" 6 1 1 "freq_1" 299.188
i 1.1 6 1 1 1 0.567
; d# @158
i "SetPartParam" 7 1 1 "freq_1" 305.953
i 1.1 7 1 1 1 0.567
; e% @161
i "SetPartParam" 8 1 1 "freq_1" 312.871
i 1.1 8 1 1 1 0.567
; e- @164
i "SetPartParam" 9 1 1 "freq_1" 319.945
i 1.1 9 1 1 1 0.567
; e @167
i "SetPartParam" 10 1 1 "freq_1" 327.18
i 1.1 10 1 1 1 0.567
; e+ @169
i "SetPartParam" 11 1 1 "freq_1" 334.578
i 1.1 11 1 1 1 0.567
; f- @172
i "SetPartParam" 12 1 1 "freq_1" 342.143
i 1.1 12 1 1 1 0.567
; f @175
i "SetPartParam" 13 1 1 "freq_1" 349.88
i 1.1 13 1 1 1 0.567
; f+ @177
i "SetPartParam" 14 1 1 "freq_1" 357.791
i 1.1 14 1 1 1 0.567
; f# @180
i "SetPartParam" 15 1 1 "freq_1" 365.881
i 1.1 15 1 1 1 0.567
; g% @183
i "SetPartParam" 16 1 1 "freq_1" 374.154
i 1.1 16 1 1 1 0.567
; g- @186
i "SetPartParam" 17 1 1 "freq_1" 382.614
i 1.1 17 1 1 1 0.567
; g @189
i "SetPartParam" 18 1 1 "freq_1" 391.266
i 1.1 18 1 1 1 0.567
; g+ @191
i "SetPartParam" 19 1 1 "freq_1" 400.113
i 1.1 19 1 1 1 0.567
; g# @194
i "SetPartParam" 20 1 1 "freq_1" 409.16
i 1.1 20 1 1 1 0.567
; a% @197
i "SetPartParam" 21 1 1 "freq_1" 418.412
i 1.1 21 1 1 1 0.567
; a- @200
i "SetPartParam" 22 1 1 "freq_1" 427.872
i 1.1 22 1 1 1 0.567
; a @203
i "SetPartParam" 23 1 1 "freq_1" 437.547
i 1.1 23 1 1 1 0.567
; a+ @205
i "SetPartParam" 24 1 1 "freq_1" 447.441
i 1.1 24 1 1 1 0.567
; a# @208
i "SetPartParam" 25 1 1 "freq_1" 457.558
i 1.1 25 1 1 1 0.567
; b% @211
i "SetPartParam" 26 1 1 "freq_1" 467.904
i 1.1 26 1 1 1 0.567
; b- @214
i "SetPartParam" 27 1 1 "freq_1" 478.484
i 1.1 27 1 1 1 0.567
; b @217
i "SetPartParam" 28 1 1 "freq_1" 489.303
i 1.1 28 1 1 1 0.567
; b+ @219
i "SetPartParam" 29 1 1 "freq_1" 500.367
i 1.1 29 1 1 1 0.567
; c' @222
i "SetPartParam" 30 1 1 "freq_1" 523.251
i 1.1 30 1 1 1 0.567
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
