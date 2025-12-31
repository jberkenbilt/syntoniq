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

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 4
t 0 72
i 1.1 0 1 1 261.626 0 0.567 ; 1:p @236
i 1.2 0 1 1 345.217 0 0.567 ; 1:r @264
i 1.3 0 1 1 455.517 0 0.567 ; 1:t @292
i 1.1 1 1 1 300.529 0 0.567 ; q @240
i 1.2 1 1 1 396.55 0 0.567 ; s @268
i 1.3 1 1 1 523.251 0 0.567 ; p' @296
i 1.1 2 1 1 345.217 0 0.567 ; r @243
i 1.2 2 1 1 455.517 0 0.567 ; t @271
i 1.3 2 1 1 601.058 0 0.567 ; q' @299
i 1.1 3 1 1 396.55 0 0.567 ; s @246
i 1.2 3 1 1 523.251 0 0.567 ; p' @274
i 1.3 3 1 1 690.434 0 0.567 ; r' @302
i 1.1 4 1 1 455.517 0 0.567 ; t @249
i 1.2 4 1 1 601.058 0 0.567 ; q' @277
i 1.3 4 1 1 793.1 0 0.567 ; s' @305
i 1.1 5 1 1 523.251 0 0.567 ; p' @252
i 1.2 5 1 1 690.434 0 0.567 ; r' @280
i 1.3 5 1 1 911.033 0 0.567 ; t' @308
i 1.1 7 1 1 261.626 0 0.567 ; 1:c @431
i 1.2 7 1 1 329.628 0 0.567 ; 1:e @446
i 1.3 7 1 1 391.995 0 0.567 ; 1:g @460
i 1.1 8 1 1 523.251 0 0.567 ; c' @435
i 1.2 8 1 1 329.628 0 0.567 ; e @450
i 1.3 8 1 1 195.998 0 0.567 ; g, @464
i 1.4 8 1 1 130.813 0 0.567 ; c, @479
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
