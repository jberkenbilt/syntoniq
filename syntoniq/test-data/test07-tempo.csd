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
; [p1.4] => 1.5
; [p1.5] => 1.6
; [p1.6] => 1.7
; [p1.7] => 1.8
; [p1.8] => 1.9
; [p1.9] => 1.10

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 10
t 0 60 5 60 5 72 9 108
; 2:e@109
i "SetPartParam" 0 2 1 "freq_1" 327.18
; 2:e @109
i 1.01 0 2 1 1 0.567
; 2:g@123
i "SetPartParam" 0 2 1 "freq_2" 391.266
; 2:g @123
i 1.02 0 2 1 2 0.567
; e-@113
i "SetPartParam" 2 2 1 "freq_1" 319.945
; e- @113
i 1.01 2 2 1 1 0.567
; g+@127
i "SetPartParam" 2 2 1 "freq_2" 400.113
; g+ @127
i 1.02 2 2 1 2 0.567
; 10:c,@217
i "SetPartParam" 4 10 1 "freq_1" 130.813
; 10:c, @217
i 1.01 4 10 1 1 0.567
; 9:g,@234
i "SetPartParam" 5 9 1 "freq_2" 195.633
; 9:g, @234
i 1.02 5 9 1 2 0.567
; 8:d@250
i "SetPartParam" 6 8 1 "freq_3" 292.572
; 8:d @250
i 1.03 6 8 1 3 0.567
; 7:a@265
i "SetPartParam" 7 7 1 "freq_4" 437.547
; 7:a @265
i 1.04 7 7 1 4 0.567
; 6:e'@280
i "SetPartParam" 8 6 1 "freq_5" 654.36
; 6:e' @280
i 1.05 8 6 1 5 0.567
; 5:e-,@296
i "SetPartParam" 9 5 1 "freq_6" 159.973
; 5:e-, @296
i 1.06 9 5 1 6 0.567
; 4:b-,@313
i "SetPartParam" 10 4 1 "freq_7" 239.242
; 4:b-, @313
i 1.07 10 4 1 7 0.567
; 3:f#@330
i "SetPartParam" 11 3 1 "freq_8" 365.881
; 3:f# @330
i 1.08 11 3 1 8 0.567
; 2:c#'@346
i "SetPartParam" 12 2 1 "freq_9" 547.182
; 2:c#' @346
i 1.09 12 2 1 9 0.567
; 1:g#'@363
i "SetPartParam" 13 1 1 "freq_10" 818.32
; 1:g#' @363
i 1.10 13 1 1 10 0.567
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
