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
; syntoniq notes.
instr 1
  ; p1..p3 are always instrument, start time, duration
  iPartNum = p4
  iFreq = p5
  iEndFreq = p6  // place-holder
  iVelocity = p7 // 0 to 1

  SAmpChan sprintf "p%d_amp", iPartNum
  SNotesChan sprintf "p%d_notes", iPartNum
  kBaseVol chnget SAmpChan
  kNoteCount chnget SNotesChan

  kNoteCount = (kNoteCount == 0 ? 1 : kNoteCount)
  kAmp = kBaseVol * iVelocity
  ; Attenuate based on polyphony
  kFinalAmp = kAmp / sqrt(kNoteCount)
  aEnv madsr 0.05, 0.05, 0.9, 0.15

  ; For most of the frequency range, we use a custom sound mixed with
  ; specific harmonics. At higher frequency ranges, we fall back to a
  ; sine/triangle mix for fewer artifacts.
  aMain poscil3 1, iFreq, 1

  ; blend sine and triangle
  aSine poscil3 0.9, iFreq
  aTriangle vco2 0.9, iFreq, 12
  aHigh = (aSine * 0.5) + (aTriangle * 0.5)

  ; For frequencies in the range of iLowThresh to iHighThresh,
  ; interpolate how much of the main mix we want. It drops to 0
  ; through that range.
  iLowThresh = 2000
  iHighThresh = 4000
  ; map iLowThresh, iHighThresh -> 1, 0 and clamp
  iInterp linlin iFreq, 1, 0, iLowThresh, iHighThresh
  iMainMix limit iInterp, 0, 1

  ; blend
  iHighMix = 1 - iMainMix
  aSignal = (aHigh * iHighMix) + (aMain * iMainMix) * aEnv * kFinalAmp
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
i 1.01 0 2 1 327.18 0 0.567 ; 2:e @109
i 1.02 0 2 1 391.266 0 0.567 ; 2:g @123
i 1.01 2 2 1 319.945 0 0.567 ; e- @113
i 1.02 2 2 1 400.113 0 0.567 ; g+ @127
i 1.01 4 10 1 130.813 0 0.567 ; 10:c, @217
i 1.02 5 9 1 195.633 0 0.567 ; 9:g, @234
i 1.03 6 8 1 292.572 0 0.567 ; 8:d @250
i 1.04 7 7 1 437.547 0 0.567 ; 7:a @265
i 1.05 8 6 1 654.36 0 0.567 ; 6:e' @280
i 1.06 9 5 1 159.973 0 0.567 ; 5:e-, @296
i 1.07 10 4 1 239.242 0 0.567 ; 4:b-, @313
i 1.08 11 3 1 365.881 0 0.567 ; 3:f# @330
i 1.09 12 2 1 547.182 0 0.567 ; 2:c#' @346
i 1.10 13 1 1 818.32 0 0.567 ; 1:g#' @363
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
