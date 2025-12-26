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
; [p1.1] => 1.4
; [p1.2] => 1.5
; [p1.3] => 1.2
; [p1.4] => 1.3

i "SetPartParam" 0 0.01 1 "amp" 0.5
t 0 72
i "SetPartParam" 0 0.01 1 "notes" 1
i 1.1 0 1 1 523.251 0.567 ; 1:A' @49
i "SetPartParam" 0 0.01 1 "notes" 2
i 1.2 0 2 1 218.021 0.567 ; 2:DE, @127
i "SetPartParam" 0 0.01 1 "notes" 3
i 1.3 0 2 1 87.209 0.567 ; 2:D,2 @153
i 1.1 1 1 1 436.043 0.567 ; f' @54
i 1.1 2 1 1 523.251 0.567 ; A' @57
i "SetPartParam" 2 0.01 1 "notes" 4
i 1.4 2 2 1 245.274 0.567 ; p @83
i 1.2 2 2 1 196.219 0.567 ; C, @135
i 1.3 2 2 1 81.758 0.567 ; E,2 @161
i 1.1 3 1 1 588.658 0.567 ; I' @60
i 1.1 4 4 1 654.064 0.567 ; 4:E' @63
i 1.4 4 4 1 261.626 0.567 ; 4:A @89
i "SetPartParam" 4 0.01 1 "notes" 5
i 1.5 4 4 1 218.021 0.567 ; 4:DE, @114
i 1.2 4 4 1 174.417 0.567 ; 4:D, @141
i 1.3 4 4 1 72.674 0.567 ; 4:Df,2 @167
i "SetPartParam" 8 0.01 1 "notes" 1
i 1.1 8 1 1 523.251 0.567 ; 1:A' @252
i "SetPartParam" 8 0.01 1 "notes" 2
i 1.2 8 2 1 220 0.567 ; 2:DE, @330
i "SetPartParam" 8 0.01 1 "notes" 3
i 1.3 8 2 1 87.307 0.567 ; 2:D,2 @356
i 1.1 9 1 1 440 0.567 ; f' @257
i 1.1 10 1 1 523.251 0.567 ; A' @260
i "SetPartParam" 10 0.01 1 "notes" 4
i 1.4 10 2 1 246.942 0.567 ; p @286
i 1.2 10 2 1 195.998 0.567 ; C, @338
i 1.3 10 2 1 82.407 0.567 ; E,2 @364
i 1.1 11 1 1 587.33 0.567 ; I' @263
i 1.1 12 4 1 659.255 0.567 ; 4:E' @266
i 1.4 12 4 1 261.626 0.567 ; 4:A @292
i "SetPartParam" 12 0.01 1 "notes" 5
i 1.5 12 4 1 220 0.567 ; 4:DE, @317
i 1.2 12 4 1 174.614 0.567 ; 4:D, @344
i 1.3 12 4 1 73.416 0.567 ; 4:Df,2 @370
i "SetPartParam" 16 0.01 1 "notes" 1
i 1.1 16 1 1 523.251 0.567 ; 1:A' @455
i "SetPartParam" 16 0.01 1 "notes" 2
i 1.2 16 2 1 218.003 0.567 ; 2:DE, @533
i "SetPartParam" 16 0.01 1 "notes" 3
i 1.3 16 2 1 87.573 0.567 ; 2:D,2 @559
i 1.1 17 1 1 436.005 0.567 ; f' @460
i 1.1 18 1 1 523.251 0.567 ; A' @463
i "SetPartParam" 18 0.01 1 "notes" 4
i 1.4 18 2 1 243.216 0.567 ; p @489
i 1.2 18 2 1 195.403 0.567 ; C, @541
i 1.3 18 2 1 81.411 0.567 ; E,2 @567
i 1.1 19 1 1 583.769 0.567 ; I' @466
i 1.1 20 4 1 651.287 0.567 ; 4:E' @469
i 1.4 20 4 1 261.626 0.567 ; 4:A @495
i "SetPartParam" 20 0.01 1 "notes" 5
i 1.5 20 4 1 218.003 0.567 ; 4:DE, @520
i 1.2 20 4 1 175.146 0.567 ; 4:D, @547
i 1.3 20 4 1 72.971 0.567 ; 4:Df,2 @573
i "SetPartParam" 24 0.01 1 "notes" 1
i 1.1 24 1 1 523.251 0.567 ; 1:A' @658
i "SetPartParam" 24 0.01 1 "notes" 2
i 1.2 24 2 1 218.774 0.567 ; 2:DE, @736
i "SetPartParam" 24 0.01 1 "notes" 3
i 1.3 24 2 1 87.47 0.567 ; 2:D,2 @762
i 1.1 25 1 1 437.547 0.567 ; f' @663
i 1.1 26 1 1 523.251 0.567 ; A' @666
i "SetPartParam" 26 0.01 1 "notes" 4
i 1.4 26 2 1 244.652 0.567 ; p @692
i 1.2 26 2 1 195.633 0.567 ; C, @744
i 1.3 26 2 1 81.795 0.567 ; E,2 @770
i 1.1 27 1 1 585.145 0.567 ; I' @669
i 1.1 28 4 1 654.36 0.567 ; 4:E' @672
i 1.4 28 4 1 261.626 0.567 ; 4:A @698
i "SetPartParam" 28 0.01 1 "notes" 5
i 1.5 28 4 1 218.774 0.567 ; 4:DE, @723
i 1.2 28 4 1 174.94 0.567 ; 4:D, @750
i 1.3 28 4 1 73.143 0.567 ; 4:Df,2 @776
i "SetPartParam" 32 0.01 1 "notes" 1
i 1.1 32 1 1 523.251 0.567 ; 1:A' @878
i "SetPartParam" 32 0.01 1 "notes" 2
i 1.2 32 2 1 217.228 0.567 ; 2:DE, @976
i "SetPartParam" 32 0.01 1 "notes" 3
i 1.3 32 2 1 87.184 0.567 ; 2:D,2 @1002
i 1.1 33 1 1 434.456 0.567 ; f' @883
i 1.1 34 1 1 523.251 0.567 ; A' @886
i "SetPartParam" 34 0.01 1 "notes" 4
i 1.4 34 2 1 244.518 0.567 ; p @932
i 1.2 34 2 1 196.274 0.567 ; C, @984
i 1.3 34 2 1 81.483 0.567 ; E,2 @1010
i 1.1 35 1 1 588.987 0.567 ; I' @889
i 1.1 36 4 1 651.867 0.567 ; 4:E#' @892
i 1.4 36 4 1 261.626 0.567 ; 4:A @938
i "SetPartParam" 36 0.01 1 "notes" 5
i 1.5 36 4 1 217.228 0.567 ; 4:DE, @963
i 1.2 36 4 1 174.368 0.567 ; 4:D, @990
i 1.3 36 4 1 72.389 0.567 ; 4:Df,2 @1016
i "SetPartParam" 40 0.01 1 "notes" 1
i 1.1 40 1 1 523.251 0.567 ; 1:A' @1118
i "SetPartParam" 40 0.01 1 "notes" 2
i 1.2 40 2 1 217.853 0.567 ; 2:DE, @1216
i "SetPartParam" 40 0.01 1 "notes" 3
i 1.3 40 2 1 87.212 0.567 ; 2:D,2 @1242
i 1.1 41 1 1 435.705 0.567 ; f' @1123
i 1.1 42 1 1 523.251 0.567 ; A' @1126
i "SetPartParam" 42 0.01 1 "notes" 4
i 1.4 42 2 1 245.065 0.567 ; p @1172
i 1.2 42 2 1 196.211 0.567 ; C, @1224
i 1.3 42 2 1 81.692 0.567 ; E,2 @1250
i 1.1 43 1 1 588.611 0.567 ; I' @1129
i 1.1 44 4 1 653.532 0.567 ; 4:E#' @1132
i 1.4 44 4 1 261.626 0.567 ; 4:A @1178
i "SetPartParam" 44 0.01 1 "notes" 5
i 1.5 44 4 1 217.853 0.567 ; 4:DE, @1203
i 1.2 44 4 1 174.424 0.567 ; 4:D, @1230
i 1.3 44 4 1 72.62 0.567 ; 4:Df,2 @1256
i "SetPartParam" 48 0.01 1 "notes" 1
i 1.1 48 1 1 523.251 0.567 ; 1:A' @1358
i "SetPartParam" 48 0.01 1 "notes" 2
i 1.2 48 2 1 222.254 0.567 ; 2:DE, @1460
i "SetPartParam" 48 0.01 1 "notes" 3
i 1.3 48 2 1 87.011 0.567 ; 2:D,2 @1486
i 1.1 49 1 1 444.508 0.567 ; f' @1363
i 1.1 50 1 1 523.251 0.567 ; A' @1366
i "SetPartParam" 50 0.01 1 "notes" 4
i 1.4 50 2 1 241.138 0.567 ; p @1416
i 1.2 50 2 1 196.665 0.567 ; C, @1468
i 1.3 50 2 1 80.197 0.567 ; E,2 @1494
i 1.1 51 1 1 591.334 0.567 ; I' @1369
i 1.1 52 4 1 668.276 0.567 ; 4:E#' @1372
i 1.4 52 4 1 261.626 0.567 ; 4:A @1422
i "SetPartParam" 52 0.01 1 "notes" 5
i 1.5 52 4 1 222.254 0.567 ; 4:DE, @1447
i 1.2 52 4 1 174.022 0.567 ; 4:D, @1474
i 1.3 52 4 1 73.917 0.567 ; 4:Df,2 @1500
i "SetPartParam" 56 0.01 1 "notes" 1
i 1.1 56 1 1 523.251 0.567 ; 1:A' @1585
i "SetPartParam" 56 0.01 1 "notes" 2
i 1.2 56 2 1 222.952 0.567 ; 2:DE, @1663
i "SetPartParam" 56 0.01 1 "notes" 3
i 1.3 56 2 1 85.389 0.567 ; 2:D,2 @1689
i 1.1 57 1 1 445.904 0.567 ; f' @1590
i 1.1 58 1 1 523.251 0.567 ; A' @1593
i "SetPartParam" 58 0.01 1 "notes" 4
i 1.4 58 2 1 248.041 0.567 ; p @1619
i 1.2 58 2 1 200.401 0.567 ; C, @1671
i 1.3 58 2 1 80.955 0.567 ; E,2 @1697
i 1.1 59 1 1 582.133 0.567 ; I' @1596
i 1.1 60 4 1 647.642 0.567 ; 4:E' @1599
i 1.4 60 4 1 261.626 0.567 ; 4:A @1625
i "SetPartParam" 60 0.01 1 "notes" 5
i 1.5 60 4 1 222.952 0.567 ; 4:DE, @1650
i 1.2 60 4 1 170.778 0.567 ; 4:D, @1677
i 1.3 60 4 1 72.767 0.567 ; 4:Df,2 @1703
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
