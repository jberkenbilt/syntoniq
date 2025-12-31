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

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 1
t 0 72 2 72 2 20 8 300 12 300 12 60 13 60 13 90 16 90 16 60 18 60 18 60 19 60 19 90 22 90 22 180 24 180 24 90 25 90 25 60
i 1.1 0 0.5 1 261.626 0.567 ; 1/2:c @311
i 1.1 0.5 0.5 1 261.626 0.567 ; c @317
i 1.1 1 0.5 1 261.626 0.567 ; c @319
i 1.1 1.5 0.5 1 261.626 0.567 ; c @321
i 1.1 2 0.5 1 293.665 0.567 ; d @323
i 1.1 2.5 0.5 1 293.665 0.567 ; d @325
i 1.1 3 0.5 1 293.665 0.567 ; d @327
i 1.1 3.5 0.5 1 293.665 0.567 ; d @329
; mark 'a' @'[343,346)
i 1.1 4 0.5 1 293.665 0.567 ; 1/2:d @355
i 1.1 4.5 0.5 1 293.665 0.567 ; d @361
i 1.1 5 0.5 1 293.665 0.567 ; d @363
i 1.1 5.5 0.5 1 293.665 0.567 ; d @365
i 1.1 6 0.5 1 293.665 0.567 ; d @367
i 1.1 6.5 0.5 1 293.665 0.567 ; d @369
i 1.1 7 0.5 1 293.665 0.567 ; d @371
i 1.1 7.5 0.5 1 293.665 0.567 ; d @373
i 1.1 8 0.5 1 293.665 0.567 ; d @377
i 1.1 8.5 0.5 1 293.665 0.567 ; d @379
i 1.1 9 0.5 1 293.665 0.567 ; d @381
i 1.1 9.5 0.5 1 293.665 0.567 ; d @383
i 1.1 10 0.5 1 293.665 0.567 ; d @385
i 1.1 10.5 0.5 1 293.665 0.567 ; d @387
i 1.1 11 0.5 1 293.665 0.567 ; d @389
i 1.1 11.5 0.5 1 293.665 0.567 ; d @391
; mark 'a1' @'[447,451)
i 1.1 12 0.5 1 329.628 0.567 ; 1/2:e @580
i 1.1 12.5 0.5 1 329.628 0.567 ; e @586
i 1.1 13 0.5 1 349.228 0.567 ; f @588
i 1.1 13.5 0.5 1 349.228 0.567 ; f @590
; mark 'a2' @'[603,607)
; mark 'b' @'[681,684)
i 1.1 14 0.5 1 349.228 0.567 ; 1/2:f @693
i 1.1 14.5 0.5 1 349.228 0.567 ; f @699
i 1.1 15 0.5 1 349.228 0.567 ; f @701
i 1.1 15.5 0.5 1 349.228 0.567 ; f @703
i 1.1 16 0.5 1 391.995 0.567 ; 1/2:g @857
i 1.1 16.5 0.5 1 391.995 0.567 ; g @863
i 1.1 17 0.5 1 391.995 0.567 ; g @865
i 1.1 17.5 0.5 1 391.995 0.567 ; g @867
; repeat start 'a1' @'[883,887)
i 1.1 18 0.5 1 329.628 0.567 ; 1/2:e @580
i 1.1 18.5 0.5 1 329.628 0.567 ; e @586
i 1.1 19 0.5 1 349.228 0.567 ; f @588
i 1.1 19.5 0.5 1 349.228 0.567 ; f @590
; repeat end 'a2' @'[892,896)
; mark 'c' @'[910,913)
i 1.1 20 0.5 1 440 0.567 ; 1/2:a @940
i 1.1 20.5 0.5 1 440 0.567 ; a @946
i 1.1 21 0.5 1 440 0.567 ; a @948
i 1.1 21.5 0.5 1 440 0.567 ; a @950
; mark 'd' @'[1008,1011)
; repeat start 'c' @'[1068,1071)
i 1.1 22 0.5 1 440 0.567 ; 1/2:a @940
i 1.1 22.5 0.5 1 440 0.567 ; a @946
i 1.1 23 0.5 1 440 0.567 ; a @948
i 1.1 23.5 0.5 1 440 0.567 ; a @950
; repeat end 'd' @'[1076,1079)
i 1.1 25 0.5 1 493.883 0.567 ; 1/2:b @1270
i 1.1 25.5 0.5 1 493.883 0.567 ; b @1276
i 1.1 26 0.5 1 493.883 0.567 ; b @1278
i 1.1 26.5 0.5 1 493.883 0.567 ; b @1280
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
