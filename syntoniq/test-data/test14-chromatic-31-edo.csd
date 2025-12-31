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

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 2
t 0 90
i 1.1 0 1 1 261.626 0 0.567 ; 1:c @137
i 1.2 0 31 1 130.813 0 0.567 ; 31:c, @232
i 1.1 1 1 1 267.541 0 0.567 ; c+ @141
i 1.1 2 1 1 273.591 0 0.567 ; c# @144
i 1.1 3 1 1 279.777 0 0.567 ; d% @147
i 1.1 4 1 1 286.103 0 0.567 ; d- @150
i 1.1 5 1 1 292.572 0 0.567 ; d @153
i 1.1 6 1 1 299.188 0 0.567 ; d+ @155
i 1.1 7 1 1 305.953 0 0.567 ; d# @158
i 1.1 8 1 1 312.871 0 0.567 ; e% @161
i 1.1 9 1 1 319.945 0 0.567 ; e- @164
i 1.1 10 1 1 327.18 0 0.567 ; e @167
i 1.1 11 1 1 334.578 0 0.567 ; e+ @169
i 1.1 12 1 1 342.143 0 0.567 ; f- @172
i 1.1 13 1 1 349.88 0 0.567 ; f @175
i 1.1 14 1 1 357.791 0 0.567 ; f+ @177
i 1.1 15 1 1 365.881 0 0.567 ; f# @180
i 1.1 16 1 1 374.154 0 0.567 ; g% @183
i 1.1 17 1 1 382.614 0 0.567 ; g- @186
i 1.1 18 1 1 391.266 0 0.567 ; g @189
i 1.1 19 1 1 400.113 0 0.567 ; g+ @191
i 1.1 20 1 1 409.16 0 0.567 ; g# @194
i 1.1 21 1 1 418.412 0 0.567 ; a% @197
i 1.1 22 1 1 427.872 0 0.567 ; a- @200
i 1.1 23 1 1 437.547 0 0.567 ; a @203
i 1.1 24 1 1 447.441 0 0.567 ; a+ @205
i 1.1 25 1 1 457.558 0 0.567 ; a# @208
i 1.1 26 1 1 467.904 0 0.567 ; b% @211
i 1.1 27 1 1 478.484 0 0.567 ; b- @214
i 1.1 28 1 1 489.303 0 0.567 ; b @217
i 1.1 29 1 1 500.367 0 0.567 ; b+ @219
i 1.1 30 1 1 523.251 0 0.567 ; c' @222
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
