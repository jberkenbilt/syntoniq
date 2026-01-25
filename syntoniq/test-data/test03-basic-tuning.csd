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
; [p1.2] => 1.3
; [p1.3] => 1.4

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 4
t 0 72
; 1:p @236
i "SetPartParam" 0 1 1 "freq_1" 261.626
i 1.1 0 1 1 1 0.567
; 1:r @264
i "SetPartParam" 0 1 1 "freq_2" 345.217
i 1.2 0 1 1 2 0.567
; 1:t @292
i "SetPartParam" 0 1 1 "freq_3" 455.517
i 1.3 0 1 1 3 0.567
; q @240
i "SetPartParam" 1 1 1 "freq_1" 300.529
i 1.1 1 1 1 1 0.567
; s @268
i "SetPartParam" 1 1 1 "freq_2" 396.55
i 1.2 1 1 1 2 0.567
; p' @296
i "SetPartParam" 1 1 1 "freq_3" 523.251
i 1.3 1 1 1 3 0.567
; r @243
i "SetPartParam" 2 1 1 "freq_1" 345.217
i 1.1 2 1 1 1 0.567
; t @271
i "SetPartParam" 2 1 1 "freq_2" 455.517
i 1.2 2 1 1 2 0.567
; q' @299
i "SetPartParam" 2 1 1 "freq_3" 601.058
i 1.3 2 1 1 3 0.567
; s @246
i "SetPartParam" 3 1 1 "freq_1" 396.55
i 1.1 3 1 1 1 0.567
; p' @274
i "SetPartParam" 3 1 1 "freq_2" 523.251
i 1.2 3 1 1 2 0.567
; r' @302
i "SetPartParam" 3 1 1 "freq_3" 690.434
i 1.3 3 1 1 3 0.567
; t @249
i "SetPartParam" 4 1 1 "freq_1" 455.517
i 1.1 4 1 1 1 0.567
; q' @277
i "SetPartParam" 4 1 1 "freq_2" 601.058
i 1.2 4 1 1 2 0.567
; s' @305
i "SetPartParam" 4 1 1 "freq_3" 793.1
i 1.3 4 1 1 3 0.567
; p' @252
i "SetPartParam" 5 1 1 "freq_1" 523.251
i 1.1 5 1 1 1 0.567
; r' @280
i "SetPartParam" 5 1 1 "freq_2" 690.434
i 1.2 5 1 1 2 0.567
; t' @308
i "SetPartParam" 5 1 1 "freq_3" 911.033
i 1.3 5 1 1 3 0.567
; 1:c @431
i "SetPartParam" 7 1 1 "freq_1" 261.626
i 1.1 7 1 1 1 0.567
; 1:e @446
i "SetPartParam" 7 1 1 "freq_2" 329.628
i 1.2 7 1 1 2 0.567
; 1:g @460
i "SetPartParam" 7 1 1 "freq_3" 391.995
i 1.3 7 1 1 3 0.567
; c' @435
i "SetPartParam" 8 1 1 "freq_1" 523.251
i 1.1 8 1 1 1 0.567
; e @450
i "SetPartParam" 8 1 1 "freq_2" 329.628
i 1.2 8 1 1 2 0.567
; g, @464
i "SetPartParam" 8 1 1 "freq_3" 195.998
i 1.3 8 1 1 3 0.567
; c, @479
i "SetPartParam" 8 1 1 "freq_4" 130.813
i 1.4 8 1 1 4 0.567
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
