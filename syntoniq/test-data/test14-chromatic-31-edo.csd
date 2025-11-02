<CsoundSynthesizer>

<CsOptions>
-odac
</CsOptions>

<CsInstruments>

sr = 44100
ksmps = 32
nchnls = 2
0dbfs = 1

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

; Each part has associated channels:
; - p<n>_amp -- a volume level from 0 to 1 inclusive
; - p<n>_notes -- the number of notes currently "on" for the part
; These are set using the "SetPartParam" and "SetPartParamRamp" control
; instruments.

; A single instrument may be used for multiple parts. Any additional
; instrument must accept the same parameters to be a target for
; syntoniq notes.
instr 1
  ; p1..p3 are always instrument, start time, duration
  iPartNum = p4
  iFreq = p5
  iVelocity = p6 // 0 to 1

  SAmpChan sprintf "p%d_amp", iPartNum
  SNotesChan sprintf "p%d_notes", iPartNum
  kBaseVol chnget SAmpChan
  kNoteCount chnget SNotesChan

  kNoteCount = (kNoteCount == 0 ? 1 : kNoteCount)
  kAmp = kBaseVol * iVelocity
  ; Attenuate based on polyphony
  kFinalAmp = kAmp / sqrt(kNoteCount)
  kEnv madsr 0.05, 0, 0.8, 0.2

  aTone oscil3 kFinalAmp * kEnv, iFreq, 1
  aFilt moogladder aTone, 2000 + (kEnv * 3000), 0.2

  outs aFilt, aFilt
endin

</CsInstruments>
<CsScore>

; function table for oscilator
f 1 0 32768 10 1 .6 .6 .4 .2 .2 .1

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
t 0 90
i "SetPartParam" 0 0.01 1 "notes" 1
i 1.1 0 1 1 261.626 0.567 ; 1:c @526
i "SetPartParam" 0 0.01 1 "notes" 2
i 1.2 0 31 1 130.813 0.567 ; 31:c, @621
i 1.1 1 1 1 267.541 0.567 ; c+ @530
i 1.1 2 1 1 273.591 0.567 ; c# @533
i 1.1 3 1 1 279.777 0.567 ; d% @536
i 1.1 4 1 1 286.103 0.567 ; d- @539
i 1.1 5 1 1 292.572 0.567 ; d @542
i 1.1 6 1 1 299.188 0.567 ; d+ @544
i 1.1 7 1 1 305.953 0.567 ; d# @547
i 1.1 8 1 1 312.871 0.567 ; e% @550
i 1.1 9 1 1 319.945 0.567 ; e- @553
i 1.1 10 1 1 327.18 0.567 ; e @556
i 1.1 11 1 1 334.578 0.567 ; e+ @558
i 1.1 12 1 1 342.143 0.567 ; f- @561
i 1.1 13 1 1 349.88 0.567 ; f @564
i 1.1 14 1 1 357.791 0.567 ; f+ @566
i 1.1 15 1 1 365.881 0.567 ; f# @569
i 1.1 16 1 1 374.154 0.567 ; g% @572
i 1.1 17 1 1 382.614 0.567 ; g- @575
i 1.1 18 1 1 391.266 0.567 ; g @578
i 1.1 19 1 1 400.113 0.567 ; g+ @580
i 1.1 20 1 1 409.16 0.567 ; g# @583
i 1.1 21 1 1 418.412 0.567 ; a% @586
i 1.1 22 1 1 427.872 0.567 ; a- @589
i 1.1 23 1 1 437.547 0.567 ; a @592
i 1.1 24 1 1 447.441 0.567 ; a+ @594
i 1.1 25 1 1 457.558 0.567 ; a# @597
i 1.1 26 1 1 467.904 0.567 ; b% @600
i 1.1 27 1 1 478.484 0.567 ; b- @603
i 1.1 28 1 1 489.303 0.567 ; b @606
i 1.1 29 1 1 500.367 0.567 ; b+ @608
i 1.1 30 1 1 523.251 0.567 ; c' @611
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
