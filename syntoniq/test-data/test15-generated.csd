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
; [p1.1] => 1.4
; [p1.2] => 1.5
; [p1.3] => 1.2
; [p1.4] => 1.3

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 5
t 0 72
i 1.1 0 1 1 523.251 0 0.567 ; 1:A' @49
i 1.2 0 2 1 218.021 0 0.567 ; 2:DE, @127
i 1.3 0 2 1 87.209 0 0.567 ; 2:D,2 @153
i 1.1 1 1 1 436.043 0 0.567 ; f' @54
i 1.1 2 1 1 523.251 0 0.567 ; A' @57
i 1.4 2 2 1 245.274 0 0.567 ; p @83
i 1.2 2 2 1 196.219 0 0.567 ; C, @135
i 1.3 2 2 1 81.758 0 0.567 ; E,2 @161
i 1.1 3 1 1 588.658 0 0.567 ; I' @60
i 1.1 4 4 1 654.064 0 0.567 ; 4:E' @63
i 1.4 4 4 1 261.626 0 0.567 ; 4:A @89
i 1.5 4 4 1 218.021 0 0.567 ; 4:DE, @114
i 1.2 4 4 1 174.417 0 0.567 ; 4:D, @141
i 1.3 4 4 1 72.674 0 0.567 ; 4:Df,2 @167
i 1.1 8 1 1 523.251 0 0.567 ; 1:A' @252
i 1.2 8 2 1 220 0 0.567 ; 2:DE, @330
i 1.3 8 2 1 87.307 0 0.567 ; 2:D,2 @356
i 1.1 9 1 1 440 0 0.567 ; f' @257
i 1.1 10 1 1 523.251 0 0.567 ; A' @260
i 1.4 10 2 1 246.942 0 0.567 ; p @286
i 1.2 10 2 1 195.998 0 0.567 ; C, @338
i 1.3 10 2 1 82.407 0 0.567 ; E,2 @364
i 1.1 11 1 1 587.33 0 0.567 ; I' @263
i 1.1 12 4 1 659.255 0 0.567 ; 4:E' @266
i 1.4 12 4 1 261.626 0 0.567 ; 4:A @292
i 1.5 12 4 1 220 0 0.567 ; 4:DE, @317
i 1.2 12 4 1 174.614 0 0.567 ; 4:D, @344
i 1.3 12 4 1 73.416 0 0.567 ; 4:Df,2 @370
i 1.1 16 1 1 523.251 0 0.567 ; 1:A' @455
i 1.2 16 2 1 218.003 0 0.567 ; 2:DE, @533
i 1.3 16 2 1 87.573 0 0.567 ; 2:D,2 @559
i 1.1 17 1 1 436.005 0 0.567 ; f' @460
i 1.1 18 1 1 523.251 0 0.567 ; A' @463
i 1.4 18 2 1 243.216 0 0.567 ; p @489
i 1.2 18 2 1 195.403 0 0.567 ; C, @541
i 1.3 18 2 1 81.411 0 0.567 ; E,2 @567
i 1.1 19 1 1 583.769 0 0.567 ; I' @466
i 1.1 20 4 1 651.287 0 0.567 ; 4:E' @469
i 1.4 20 4 1 261.626 0 0.567 ; 4:A @495
i 1.5 20 4 1 218.003 0 0.567 ; 4:DE, @520
i 1.2 20 4 1 175.146 0 0.567 ; 4:D, @547
i 1.3 20 4 1 72.971 0 0.567 ; 4:Df,2 @573
i 1.1 24 1 1 523.251 0 0.567 ; 1:A' @681
i 1.2 24 2 1 220.118 0 0.567 ; 2:DE, @809
i 1.3 24 2 1 87.333 0 0.567 ; 2:D,2 @835
i 1.1 25 1 1 439.922 0 0.567 ; f' @686
i 1.1 26 1 1 523.624 0 0.567 ; B @689
i 1.4 26 2 1 246.927 0 0.567 ; p @765
i 1.2 26 2 1 196.079 0 0.567 ; C, @817
i 1.3 26 2 1 82.426 0 0.567 ; E,2 @843
i 1.1 27 1 1 587.399 0 0.567 ; I' @692
i 1.1 28 4 1 659.412 0 0.567 ; 4:E' @695
i 1.4 28 4 1 261.626 0 0.567 ; 4:A @771
i 1.5 28 4 1 220.118 0 0.567 ; 4:DE, @796
i 1.2 28 4 1 174.666 0 0.567 ; 4:D, @823
i 1.3 28 4 1 73.425 0 0.567 ; 4:Df,2 @849
i 1.1 32 1 1 523.251 0 0.567 ; 1:A' @934
i 1.2 32 2 1 218.774 0 0.567 ; 2:DE, @1012
i 1.3 32 2 1 87.47 0 0.567 ; 2:D,2 @1038
i 1.1 33 1 1 437.547 0 0.567 ; f' @939
i 1.1 34 1 1 523.251 0 0.567 ; A' @942
i 1.4 34 2 1 244.652 0 0.567 ; p @968
i 1.2 34 2 1 195.633 0 0.567 ; C, @1020
i 1.3 34 2 1 81.795 0 0.567 ; E,2 @1046
i 1.1 35 1 1 585.145 0 0.567 ; I' @945
i 1.1 36 4 1 654.36 0 0.567 ; 4:E' @948
i 1.4 36 4 1 261.626 0 0.567 ; 4:A @974
i 1.5 36 4 1 218.774 0 0.567 ; 4:DE, @999
i 1.2 36 4 1 174.94 0 0.567 ; 4:D, @1026
i 1.3 36 4 1 73.143 0 0.567 ; 4:Df,2 @1052
i 1.1 40 1 1 523.251 0 0.567 ; 1:A' @1154
i 1.2 40 2 1 217.228 0 0.567 ; 2:DE, @1252
i 1.3 40 2 1 87.184 0 0.567 ; 2:D,2 @1278
i 1.1 41 1 1 434.456 0 0.567 ; f' @1159
i 1.1 42 1 1 523.251 0 0.567 ; A' @1162
i 1.4 42 2 1 244.518 0 0.567 ; p @1208
i 1.2 42 2 1 196.274 0 0.567 ; C, @1260
i 1.3 42 2 1 81.483 0 0.567 ; E,2 @1286
i 1.1 43 1 1 588.987 0 0.567 ; I' @1165
i 1.1 44 4 1 651.867 0 0.567 ; 4:E#' @1168
i 1.4 44 4 1 261.626 0 0.567 ; 4:A @1214
i 1.5 44 4 1 217.228 0 0.567 ; 4:DE, @1239
i 1.2 44 4 1 174.368 0 0.567 ; 4:D, @1266
i 1.3 44 4 1 72.389 0 0.567 ; 4:Df,2 @1292
i 1.1 48 1 1 523.251 0 0.567 ; 1:A' @1394
i 1.2 48 2 1 217.853 0 0.567 ; 2:DE, @1492
i 1.3 48 2 1 87.212 0 0.567 ; 2:D,2 @1518
i 1.1 49 1 1 435.705 0 0.567 ; f' @1399
i 1.1 50 1 1 523.251 0 0.567 ; A' @1402
i 1.4 50 2 1 245.065 0 0.567 ; p @1448
i 1.2 50 2 1 196.211 0 0.567 ; C, @1500
i 1.3 50 2 1 81.692 0 0.567 ; E,2 @1526
i 1.1 51 1 1 588.611 0 0.567 ; I' @1405
i 1.1 52 4 1 653.532 0 0.567 ; 4:E#' @1408
i 1.4 52 4 1 261.626 0 0.567 ; 4:A @1454
i 1.5 52 4 1 217.853 0 0.567 ; 4:DE, @1479
i 1.2 52 4 1 174.424 0 0.567 ; 4:D, @1506
i 1.3 52 4 1 72.62 0 0.567 ; 4:Df,2 @1532
i 1.1 56 1 1 523.251 0 0.567 ; 1:A' @1636
i 1.2 56 2 1 217.853 0 0.567 ; 2:DE, @1714
i 1.3 56 2 1 87.212 0 0.567 ; 2:D,2 @1740
i 1.1 57 1 1 435.705 0 0.567 ; f' @1641
i 1.1 58 1 1 523.251 0 0.567 ; A' @1644
i 1.4 58 2 1 245.065 0 0.567 ; p @1670
i 1.2 58 2 1 196.211 0 0.567 ; C, @1722
i 1.3 58 2 1 81.692 0 0.567 ; E,2 @1748
i 1.1 59 1 1 588.611 0 0.567 ; I' @1647
i 1.1 60 4 1 653.532 0 0.567 ; 4:E' @1650
i 1.4 60 4 1 261.626 0 0.567 ; 4:A @1676
i 1.5 60 4 1 217.853 0 0.567 ; 4:DE, @1701
i 1.2 60 4 1 174.424 0 0.567 ; 4:D, @1728
i 1.3 60 4 1 72.62 0 0.567 ; 4:Df,2 @1754
i 1.1 64 1 1 523.251 0 0.567 ; 1:A' @1856
i 1.2 64 2 1 222.254 0 0.567 ; 2:DE, @1958
i 1.3 64 2 1 87.011 0 0.567 ; 2:D,2 @1984
i 1.1 65 1 1 444.508 0 0.567 ; f' @1861
i 1.1 66 1 1 523.251 0 0.567 ; A' @1864
i 1.4 66 2 1 241.138 0 0.567 ; p @1914
i 1.2 66 2 1 196.665 0 0.567 ; C, @1966
i 1.3 66 2 1 80.197 0 0.567 ; E,2 @1992
i 1.1 67 1 1 591.334 0 0.567 ; I' @1867
i 1.1 68 4 1 668.276 0 0.567 ; 4:E#' @1870
i 1.4 68 4 1 261.626 0 0.567 ; 4:A @1920
i 1.5 68 4 1 222.254 0 0.567 ; 4:DE, @1945
i 1.2 68 4 1 174.022 0 0.567 ; 4:D, @1972
i 1.3 68 4 1 73.917 0 0.567 ; 4:Df,2 @1998
i 1.1 72 1 1 523.251 0 0.567 ; 1:A' @2083
i 1.2 72 2 1 222.952 0 0.567 ; 2:DE, @2161
i 1.3 72 2 1 85.389 0 0.567 ; 2:D,2 @2187
i 1.1 73 1 1 445.904 0 0.567 ; f' @2088
i 1.1 74 1 1 523.251 0 0.567 ; A' @2091
i 1.4 74 2 1 248.041 0 0.567 ; p @2117
i 1.2 74 2 1 200.401 0 0.567 ; C, @2169
i 1.3 74 2 1 80.955 0 0.567 ; E,2 @2195
i 1.1 75 1 1 582.133 0 0.567 ; I' @2094
i 1.1 76 4 1 647.642 0 0.567 ; 4:E' @2097
i 1.4 76 4 1 261.626 0 0.567 ; 4:A @2123
i 1.5 76 4 1 222.952 0 0.567 ; 4:DE, @2148
i 1.2 76 4 1 170.778 0 0.567 ; 4:D, @2175
i 1.3 76 4 1 72.767 0 0.567 ; 4:Df,2 @2201
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
