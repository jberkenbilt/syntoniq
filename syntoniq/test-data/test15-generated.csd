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
; [p1.1] => 1.4
; [p1.2] => 1.5
; [p1.3] => 1.2
; [p1.4] => 1.3

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 5
t 0 72
; 1:A'@49
i "SetPartParam" 0 1 1 "freq_1" 523.251
; 1:A' @49
i 1.1 0 1 1 1 0.567
; 2:DE,@127
i "SetPartParam" 0 2 1 "freq_2" 218.021
; 2:DE, @127
i 1.2 0 2 1 2 0.567
; 2:D,2@153
i "SetPartParam" 0 2 1 "freq_3" 87.209
; 2:D,2 @153
i 1.3 0 2 1 3 0.567
; f'@54
i "SetPartParam" 1 1 1 "freq_1" 436.043
; f' @54
i 1.1 1 1 1 1 0.567
; A'@57
i "SetPartParam" 2 1 1 "freq_1" 523.251
; A' @57
i 1.1 2 1 1 1 0.567
; p@83
i "SetPartParam" 2 2 1 "freq_4" 245.274
; p @83
i 1.4 2 2 1 4 0.567
; C,@135
i "SetPartParam" 2 2 1 "freq_2" 196.219
; C, @135
i 1.2 2 2 1 2 0.567
; E,2@161
i "SetPartParam" 2 2 1 "freq_3" 81.758
; E,2 @161
i 1.3 2 2 1 3 0.567
; I'@60
i "SetPartParam" 3 1 1 "freq_1" 588.658
; I' @60
i 1.1 3 1 1 1 0.567
; 4:E'@63
i "SetPartParam" 4 4 1 "freq_1" 654.064
; 4:E' @63
i 1.1 4 4 1 1 0.567
; 4:A@89
i "SetPartParam" 4 4 1 "freq_4" 261.626
; 4:A @89
i 1.4 4 4 1 4 0.567
; 4:DE,@114
i "SetPartParam" 4 4 1 "freq_5" 218.021
; 4:DE, @114
i 1.5 4 4 1 5 0.567
; 4:D,@141
i "SetPartParam" 4 4 1 "freq_2" 174.417
; 4:D, @141
i 1.2 4 4 1 2 0.567
; 4:Df,2@167
i "SetPartParam" 4 4 1 "freq_3" 72.674
; 4:Df,2 @167
i 1.3 4 4 1 3 0.567
; 1:A'@252
i "SetPartParam" 8 1 1 "freq_1" 523.251
; 1:A' @252
i 1.1 8 1 1 1 0.567
; 2:DE,@330
i "SetPartParam" 8 2 1 "freq_2" 220
; 2:DE, @330
i 1.2 8 2 1 2 0.567
; 2:D,2@356
i "SetPartParam" 8 2 1 "freq_3" 87.307
; 2:D,2 @356
i 1.3 8 2 1 3 0.567
; f'@257
i "SetPartParam" 9 1 1 "freq_1" 440
; f' @257
i 1.1 9 1 1 1 0.567
; A'@260
i "SetPartParam" 10 1 1 "freq_1" 523.251
; A' @260
i 1.1 10 1 1 1 0.567
; p@286
i "SetPartParam" 10 2 1 "freq_4" 246.942
; p @286
i 1.4 10 2 1 4 0.567
; C,@338
i "SetPartParam" 10 2 1 "freq_2" 195.998
; C, @338
i 1.2 10 2 1 2 0.567
; E,2@364
i "SetPartParam" 10 2 1 "freq_3" 82.407
; E,2 @364
i 1.3 10 2 1 3 0.567
; I'@263
i "SetPartParam" 11 1 1 "freq_1" 587.33
; I' @263
i 1.1 11 1 1 1 0.567
; 4:E'@266
i "SetPartParam" 12 4 1 "freq_1" 659.255
; 4:E' @266
i 1.1 12 4 1 1 0.567
; 4:A@292
i "SetPartParam" 12 4 1 "freq_4" 261.626
; 4:A @292
i 1.4 12 4 1 4 0.567
; 4:DE,@317
i "SetPartParam" 12 4 1 "freq_5" 220
; 4:DE, @317
i 1.5 12 4 1 5 0.567
; 4:D,@344
i "SetPartParam" 12 4 1 "freq_2" 174.614
; 4:D, @344
i 1.2 12 4 1 2 0.567
; 4:Df,2@370
i "SetPartParam" 12 4 1 "freq_3" 73.416
; 4:Df,2 @370
i 1.3 12 4 1 3 0.567
; 1:A'@455
i "SetPartParam" 16 1 1 "freq_1" 523.251
; 1:A' @455
i 1.1 16 1 1 1 0.567
; 2:DE,@533
i "SetPartParam" 16 2 1 "freq_2" 218.003
; 2:DE, @533
i 1.2 16 2 1 2 0.567
; 2:D,2@559
i "SetPartParam" 16 2 1 "freq_3" 87.573
; 2:D,2 @559
i 1.3 16 2 1 3 0.567
; f'@460
i "SetPartParam" 17 1 1 "freq_1" 436.005
; f' @460
i 1.1 17 1 1 1 0.567
; A'@463
i "SetPartParam" 18 1 1 "freq_1" 523.251
; A' @463
i 1.1 18 1 1 1 0.567
; p@489
i "SetPartParam" 18 2 1 "freq_4" 243.216
; p @489
i 1.4 18 2 1 4 0.567
; C,@541
i "SetPartParam" 18 2 1 "freq_2" 195.403
; C, @541
i 1.2 18 2 1 2 0.567
; E,2@567
i "SetPartParam" 18 2 1 "freq_3" 81.411
; E,2 @567
i 1.3 18 2 1 3 0.567
; I'@466
i "SetPartParam" 19 1 1 "freq_1" 583.769
; I' @466
i 1.1 19 1 1 1 0.567
; 4:E'@469
i "SetPartParam" 20 4 1 "freq_1" 651.287
; 4:E' @469
i 1.1 20 4 1 1 0.567
; 4:A@495
i "SetPartParam" 20 4 1 "freq_4" 261.626
; 4:A @495
i 1.4 20 4 1 4 0.567
; 4:DE,@520
i "SetPartParam" 20 4 1 "freq_5" 218.003
; 4:DE, @520
i 1.5 20 4 1 5 0.567
; 4:D,@547
i "SetPartParam" 20 4 1 "freq_2" 175.146
; 4:D, @547
i 1.2 20 4 1 2 0.567
; 4:Df,2@573
i "SetPartParam" 20 4 1 "freq_3" 72.971
; 4:Df,2 @573
i 1.3 20 4 1 3 0.567
; 1:A'@681
i "SetPartParam" 24 1 1 "freq_1" 523.251
; 1:A' @681
i 1.1 24 1 1 1 0.567
; 2:DE,@809
i "SetPartParam" 24 2 1 "freq_2" 220.118
; 2:DE, @809
i 1.2 24 2 1 2 0.567
; 2:D,2@835
i "SetPartParam" 24 2 1 "freq_3" 87.333
; 2:D,2 @835
i 1.3 24 2 1 3 0.567
; f'@686
i "SetPartParam" 25 1 1 "freq_1" 439.922
; f' @686
i 1.1 25 1 1 1 0.567
; B@689
i "SetPartParam" 26 1 1 "freq_1" 523.624
; B @689
i 1.1 26 1 1 1 0.567
; p@765
i "SetPartParam" 26 2 1 "freq_4" 246.927
; p @765
i 1.4 26 2 1 4 0.567
; C,@817
i "SetPartParam" 26 2 1 "freq_2" 196.079
; C, @817
i 1.2 26 2 1 2 0.567
; E,2@843
i "SetPartParam" 26 2 1 "freq_3" 82.426
; E,2 @843
i 1.3 26 2 1 3 0.567
; I'@692
i "SetPartParam" 27 1 1 "freq_1" 587.399
; I' @692
i 1.1 27 1 1 1 0.567
; 4:E'@695
i "SetPartParam" 28 4 1 "freq_1" 659.412
; 4:E' @695
i 1.1 28 4 1 1 0.567
; 4:A@771
i "SetPartParam" 28 4 1 "freq_4" 261.626
; 4:A @771
i 1.4 28 4 1 4 0.567
; 4:DE,@796
i "SetPartParam" 28 4 1 "freq_5" 220.118
; 4:DE, @796
i 1.5 28 4 1 5 0.567
; 4:D,@823
i "SetPartParam" 28 4 1 "freq_2" 174.666
; 4:D, @823
i 1.2 28 4 1 2 0.567
; 4:Df,2@849
i "SetPartParam" 28 4 1 "freq_3" 73.425
; 4:Df,2 @849
i 1.3 28 4 1 3 0.567
; 1:A'@934
i "SetPartParam" 32 1 1 "freq_1" 523.251
; 1:A' @934
i 1.1 32 1 1 1 0.567
; 2:DE,@1012
i "SetPartParam" 32 2 1 "freq_2" 218.774
; 2:DE, @1012
i 1.2 32 2 1 2 0.567
; 2:D,2@1038
i "SetPartParam" 32 2 1 "freq_3" 87.47
; 2:D,2 @1038
i 1.3 32 2 1 3 0.567
; f'@939
i "SetPartParam" 33 1 1 "freq_1" 437.547
; f' @939
i 1.1 33 1 1 1 0.567
; A'@942
i "SetPartParam" 34 1 1 "freq_1" 523.251
; A' @942
i 1.1 34 1 1 1 0.567
; p@968
i "SetPartParam" 34 2 1 "freq_4" 244.652
; p @968
i 1.4 34 2 1 4 0.567
; C,@1020
i "SetPartParam" 34 2 1 "freq_2" 195.633
; C, @1020
i 1.2 34 2 1 2 0.567
; E,2@1046
i "SetPartParam" 34 2 1 "freq_3" 81.795
; E,2 @1046
i 1.3 34 2 1 3 0.567
; I'@945
i "SetPartParam" 35 1 1 "freq_1" 585.145
; I' @945
i 1.1 35 1 1 1 0.567
; 4:E'@948
i "SetPartParam" 36 4 1 "freq_1" 654.36
; 4:E' @948
i 1.1 36 4 1 1 0.567
; 4:A@974
i "SetPartParam" 36 4 1 "freq_4" 261.626
; 4:A @974
i 1.4 36 4 1 4 0.567
; 4:DE,@999
i "SetPartParam" 36 4 1 "freq_5" 218.774
; 4:DE, @999
i 1.5 36 4 1 5 0.567
; 4:D,@1026
i "SetPartParam" 36 4 1 "freq_2" 174.94
; 4:D, @1026
i 1.2 36 4 1 2 0.567
; 4:Df,2@1052
i "SetPartParam" 36 4 1 "freq_3" 73.143
; 4:Df,2 @1052
i 1.3 36 4 1 3 0.567
; 1:A'@1154
i "SetPartParam" 40 1 1 "freq_1" 523.251
; 1:A' @1154
i 1.1 40 1 1 1 0.567
; 2:DE,@1252
i "SetPartParam" 40 2 1 "freq_2" 217.228
; 2:DE, @1252
i 1.2 40 2 1 2 0.567
; 2:D,2@1278
i "SetPartParam" 40 2 1 "freq_3" 87.184
; 2:D,2 @1278
i 1.3 40 2 1 3 0.567
; f'@1159
i "SetPartParam" 41 1 1 "freq_1" 434.456
; f' @1159
i 1.1 41 1 1 1 0.567
; A'@1162
i "SetPartParam" 42 1 1 "freq_1" 523.251
; A' @1162
i 1.1 42 1 1 1 0.567
; p@1208
i "SetPartParam" 42 2 1 "freq_4" 244.518
; p @1208
i 1.4 42 2 1 4 0.567
; C,@1260
i "SetPartParam" 42 2 1 "freq_2" 196.274
; C, @1260
i 1.2 42 2 1 2 0.567
; E,2@1286
i "SetPartParam" 42 2 1 "freq_3" 81.483
; E,2 @1286
i 1.3 42 2 1 3 0.567
; I'@1165
i "SetPartParam" 43 1 1 "freq_1" 588.987
; I' @1165
i 1.1 43 1 1 1 0.567
; 4:E#'@1168
i "SetPartParam" 44 4 1 "freq_1" 651.867
; 4:E#' @1168
i 1.1 44 4 1 1 0.567
; 4:A@1214
i "SetPartParam" 44 4 1 "freq_4" 261.626
; 4:A @1214
i 1.4 44 4 1 4 0.567
; 4:DE,@1239
i "SetPartParam" 44 4 1 "freq_5" 217.228
; 4:DE, @1239
i 1.5 44 4 1 5 0.567
; 4:D,@1266
i "SetPartParam" 44 4 1 "freq_2" 174.368
; 4:D, @1266
i 1.2 44 4 1 2 0.567
; 4:Df,2@1292
i "SetPartParam" 44 4 1 "freq_3" 72.389
; 4:Df,2 @1292
i 1.3 44 4 1 3 0.567
; 1:A'@1394
i "SetPartParam" 48 1 1 "freq_1" 523.251
; 1:A' @1394
i 1.1 48 1 1 1 0.567
; 2:DE,@1492
i "SetPartParam" 48 2 1 "freq_2" 217.853
; 2:DE, @1492
i 1.2 48 2 1 2 0.567
; 2:D,2@1518
i "SetPartParam" 48 2 1 "freq_3" 87.212
; 2:D,2 @1518
i 1.3 48 2 1 3 0.567
; f'@1399
i "SetPartParam" 49 1 1 "freq_1" 435.705
; f' @1399
i 1.1 49 1 1 1 0.567
; A'@1402
i "SetPartParam" 50 1 1 "freq_1" 523.251
; A' @1402
i 1.1 50 1 1 1 0.567
; p@1448
i "SetPartParam" 50 2 1 "freq_4" 245.065
; p @1448
i 1.4 50 2 1 4 0.567
; C,@1500
i "SetPartParam" 50 2 1 "freq_2" 196.211
; C, @1500
i 1.2 50 2 1 2 0.567
; E,2@1526
i "SetPartParam" 50 2 1 "freq_3" 81.692
; E,2 @1526
i 1.3 50 2 1 3 0.567
; I'@1405
i "SetPartParam" 51 1 1 "freq_1" 588.611
; I' @1405
i 1.1 51 1 1 1 0.567
; 4:E#'@1408
i "SetPartParam" 52 4 1 "freq_1" 653.532
; 4:E#' @1408
i 1.1 52 4 1 1 0.567
; 4:A@1454
i "SetPartParam" 52 4 1 "freq_4" 261.626
; 4:A @1454
i 1.4 52 4 1 4 0.567
; 4:DE,@1479
i "SetPartParam" 52 4 1 "freq_5" 217.853
; 4:DE, @1479
i 1.5 52 4 1 5 0.567
; 4:D,@1506
i "SetPartParam" 52 4 1 "freq_2" 174.424
; 4:D, @1506
i 1.2 52 4 1 2 0.567
; 4:Df,2@1532
i "SetPartParam" 52 4 1 "freq_3" 72.62
; 4:Df,2 @1532
i 1.3 52 4 1 3 0.567
; 1:A'@1636
i "SetPartParam" 56 1 1 "freq_1" 523.251
; 1:A' @1636
i 1.1 56 1 1 1 0.567
; 2:DE,@1714
i "SetPartParam" 56 2 1 "freq_2" 217.853
; 2:DE, @1714
i 1.2 56 2 1 2 0.567
; 2:D,2@1740
i "SetPartParam" 56 2 1 "freq_3" 87.212
; 2:D,2 @1740
i 1.3 56 2 1 3 0.567
; f'@1641
i "SetPartParam" 57 1 1 "freq_1" 435.705
; f' @1641
i 1.1 57 1 1 1 0.567
; A'@1644
i "SetPartParam" 58 1 1 "freq_1" 523.251
; A' @1644
i 1.1 58 1 1 1 0.567
; p@1670
i "SetPartParam" 58 2 1 "freq_4" 245.065
; p @1670
i 1.4 58 2 1 4 0.567
; C,@1722
i "SetPartParam" 58 2 1 "freq_2" 196.211
; C, @1722
i 1.2 58 2 1 2 0.567
; E,2@1748
i "SetPartParam" 58 2 1 "freq_3" 81.692
; E,2 @1748
i 1.3 58 2 1 3 0.567
; I'@1647
i "SetPartParam" 59 1 1 "freq_1" 588.611
; I' @1647
i 1.1 59 1 1 1 0.567
; 4:E'@1650
i "SetPartParam" 60 4 1 "freq_1" 653.532
; 4:E' @1650
i 1.1 60 4 1 1 0.567
; 4:A@1676
i "SetPartParam" 60 4 1 "freq_4" 261.626
; 4:A @1676
i 1.4 60 4 1 4 0.567
; 4:DE,@1701
i "SetPartParam" 60 4 1 "freq_5" 217.853
; 4:DE, @1701
i 1.5 60 4 1 5 0.567
; 4:D,@1728
i "SetPartParam" 60 4 1 "freq_2" 174.424
; 4:D, @1728
i 1.2 60 4 1 2 0.567
; 4:Df,2@1754
i "SetPartParam" 60 4 1 "freq_3" 72.62
; 4:Df,2 @1754
i 1.3 60 4 1 3 0.567
; 1:A'@1856
i "SetPartParam" 64 1 1 "freq_1" 523.251
; 1:A' @1856
i 1.1 64 1 1 1 0.567
; 2:DE,@1958
i "SetPartParam" 64 2 1 "freq_2" 222.254
; 2:DE, @1958
i 1.2 64 2 1 2 0.567
; 2:D,2@1984
i "SetPartParam" 64 2 1 "freq_3" 87.011
; 2:D,2 @1984
i 1.3 64 2 1 3 0.567
; f'@1861
i "SetPartParam" 65 1 1 "freq_1" 444.508
; f' @1861
i 1.1 65 1 1 1 0.567
; A'@1864
i "SetPartParam" 66 1 1 "freq_1" 523.251
; A' @1864
i 1.1 66 1 1 1 0.567
; p@1914
i "SetPartParam" 66 2 1 "freq_4" 241.138
; p @1914
i 1.4 66 2 1 4 0.567
; C,@1966
i "SetPartParam" 66 2 1 "freq_2" 196.665
; C, @1966
i 1.2 66 2 1 2 0.567
; E,2@1992
i "SetPartParam" 66 2 1 "freq_3" 80.197
; E,2 @1992
i 1.3 66 2 1 3 0.567
; I'@1867
i "SetPartParam" 67 1 1 "freq_1" 591.334
; I' @1867
i 1.1 67 1 1 1 0.567
; 4:E#'@1870
i "SetPartParam" 68 4 1 "freq_1" 668.276
; 4:E#' @1870
i 1.1 68 4 1 1 0.567
; 4:A@1920
i "SetPartParam" 68 4 1 "freq_4" 261.626
; 4:A @1920
i 1.4 68 4 1 4 0.567
; 4:DE,@1945
i "SetPartParam" 68 4 1 "freq_5" 222.254
; 4:DE, @1945
i 1.5 68 4 1 5 0.567
; 4:D,@1972
i "SetPartParam" 68 4 1 "freq_2" 174.022
; 4:D, @1972
i 1.2 68 4 1 2 0.567
; 4:Df,2@1998
i "SetPartParam" 68 4 1 "freq_3" 73.917
; 4:Df,2 @1998
i 1.3 68 4 1 3 0.567
; 1:A'@2083
i "SetPartParam" 72 1 1 "freq_1" 523.251
; 1:A' @2083
i 1.1 72 1 1 1 0.567
; 2:DE,@2161
i "SetPartParam" 72 2 1 "freq_2" 222.952
; 2:DE, @2161
i 1.2 72 2 1 2 0.567
; 2:D,2@2187
i "SetPartParam" 72 2 1 "freq_3" 85.389
; 2:D,2 @2187
i 1.3 72 2 1 3 0.567
; f'@2088
i "SetPartParam" 73 1 1 "freq_1" 445.904
; f' @2088
i 1.1 73 1 1 1 0.567
; A'@2091
i "SetPartParam" 74 1 1 "freq_1" 523.251
; A' @2091
i 1.1 74 1 1 1 0.567
; p@2117
i "SetPartParam" 74 2 1 "freq_4" 248.041
; p @2117
i 1.4 74 2 1 4 0.567
; C,@2169
i "SetPartParam" 74 2 1 "freq_2" 200.401
; C, @2169
i 1.2 74 2 1 2 0.567
; E,2@2195
i "SetPartParam" 74 2 1 "freq_3" 80.955
; E,2 @2195
i 1.3 74 2 1 3 0.567
; I'@2094
i "SetPartParam" 75 1 1 "freq_1" 582.133
; I' @2094
i 1.1 75 1 1 1 0.567
; 4:E'@2097
i "SetPartParam" 76 4 1 "freq_1" 647.642
; 4:E' @2097
i 1.1 76 4 1 1 0.567
; 4:A@2123
i "SetPartParam" 76 4 1 "freq_4" 261.626
; 4:A @2123
i 1.4 76 4 1 4 0.567
; 4:DE,@2148
i "SetPartParam" 76 4 1 "freq_5" 222.952
; 4:DE, @2148
i 1.5 76 4 1 5 0.567
; 4:D,@2175
i "SetPartParam" 76 4 1 "freq_2" 170.778
; 4:D, @2175
i 1.2 76 4 1 2 0.567
; 4:Df,2@2201
i "SetPartParam" 76 4 1 "freq_3" 72.767
; 4:Df,2 @2201
i 1.3 76 4 1 3 0.567
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
