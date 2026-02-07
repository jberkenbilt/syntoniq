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
; [part_a] => 1
; [part_b] => 2
; [part_c] => 3
; [part.note] => instr.note
; [part_a.1] => 1.1
; [part_a.2] => 1.2
; [part_a.3] => 1.3
; [part_a.4] => 1.4
; [part_a.5] => 1.5
; [part_a.6] => 1.6
; [part_a.7] => 1.7
; [part_a.8] => 1.8
; [part_a.9] => 1.9
; [part_a.10] => 1.10
; [part_b.1] => 1.11
; [part_b.2] => 1.12
; [part_b.3] => 1.13
; [part_b.4] => 1.14
; [part_b.5] => 1.15
; [part_b.6] => 1.16
; [part_b.7] => 1.17
; [part_b.8] => 1.18
; [part_b.9] => 1.19
; [part_b.10] => 1.20
; [part_c.1] => 1.21
; [part_c.2] => 1.22
; [part_c.3] => 1.23
; [part_c.4] => 1.24
; [part_c.5] => 1.25
; [part_c.6] => 1.26
; [part_c.7] => 1.27
; [part_c.8] => 1.28
; [part_c.9] => 1.29
; [part_c.10] => 1.30

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 10
i "SetPartParam" 0 0.01 2 "amp" 0.5
i "SetPartParam" 0 0.01 2 "notes" 10
i "SetPartParam" 0 0.01 3 "amp" 0.5
i "SetPartParam" 0 0.01 3 "notes" 10
t 0 200
; 1:A01!11,@191
i "SetPartParam" 0 1 1 "freq_1" 139.321
; 1:A01!11, @191
i 1.01 0 1 1 1 0.567
; 1:A02!11,@233
i "SetPartParam" 1 1 1 "freq_2" 148.383
; 1:A02!11, @233
i 1.02 1 1 1 2 0.567
; 1:A03!11,@275
i "SetPartParam" 2 1 1 "freq_3" 158.034
; 1:A03!11, @275
i 1.03 2 1 1 3 0.567
; 1:A04!11,@317
i "SetPartParam" 3 1 1 "freq_4" 168.312
; 1:A04!11, @317
i 1.04 3 1 1 4 0.567
; 1:A05!11,@359
i "SetPartParam" 4 1 1 "freq_5" 179.259
; 1:A05!11, @359
i 1.05 4 1 1 5 0.567
; 1:A06!11,@401
i "SetPartParam" 5 1 1 "freq_6" 190.919
; 1:A06!11, @401
i 1.06 5 1 1 6 0.567
; 1:A07!11,@443
i "SetPartParam" 6 1 1 "freq_7" 203.336
; 1:A07!11, @443
i 1.07 6 1 1 7 0.567
; 1:A08!11,@485
i "SetPartParam" 7 1 1 "freq_8" 216.561
; 1:A08!11, @485
i 1.08 7 1 1 8 0.567
; 1:A09!11,@527
i "SetPartParam" 8 1 1 "freq_9" 230.647
; 1:A09!11, @527
i 1.09 8 1 1 9 0.567
; 1:A10!11,@569
i "SetPartParam" 9 1 1 "freq_10" 245.648
; 1:A10!11, @569
i 1.10 9 1 1 10 0.567
; 1:A01!11@611
i "SetPartParam" 10 1 2 "freq_11" 278.642
; 1:A01!11 @611
i 1.11 10 1 2 11 0.567
; 1:A02!11@652
i "SetPartParam" 11 1 2 "freq_12" 296.765
; 1:A02!11 @652
i 1.12 11 1 2 12 0.567
; 1:A03!11@693
i "SetPartParam" 12 1 2 "freq_13" 316.067
; 1:A03!11 @693
i 1.13 12 1 2 13 0.567
; 1:A04!11@734
i "SetPartParam" 13 1 2 "freq_14" 336.624
; 1:A04!11 @734
i 1.14 13 1 2 14 0.567
; 1:A05!11@775
i "SetPartParam" 14 1 2 "freq_15" 358.519
; 1:A05!11 @775
i 1.15 14 1 2 15 0.567
; 1:A06!11@816
i "SetPartParam" 15 1 2 "freq_16" 381.837
; 1:A06!11 @816
i 1.16 15 1 2 16 0.567
; 1:A07!11@857
i "SetPartParam" 16 1 2 "freq_17" 406.672
; 1:A07!11 @857
i 1.17 16 1 2 17 0.567
; 1:A08!11@898
i "SetPartParam" 17 1 2 "freq_18" 433.123
; 1:A08!11 @898
i 1.18 17 1 2 18 0.567
; 1:A09!11@939
i "SetPartParam" 18 1 2 "freq_19" 461.294
; 1:A09!11 @939
i 1.19 18 1 2 19 0.567
; 1:A10!11@980
i "SetPartParam" 19 1 2 "freq_20" 491.297
; 1:A10!11 @980
i 1.20 19 1 2 20 0.567
; 1:A01!11'@1021
i "SetPartParam" 20 1 3 "freq_21" 557.284
; 1:A01!11' @1021
i 1.21 20 1 3 21 0.567
; 1:A02!11'@1063
i "SetPartParam" 21 1 3 "freq_22" 593.53
; 1:A02!11' @1063
i 1.22 21 1 3 22 0.567
; 1:A03!11'@1105
i "SetPartParam" 22 1 3 "freq_23" 632.134
; 1:A03!11' @1105
i 1.23 22 1 3 23 0.567
; 1:A04!11'@1147
i "SetPartParam" 23 1 3 "freq_24" 673.249
; 1:A04!11' @1147
i 1.24 23 1 3 24 0.567
; 1:A05!11'@1189
i "SetPartParam" 24 1 3 "freq_25" 717.038
; 1:A05!11' @1189
i 1.25 24 1 3 25 0.567
; 1:A06!11'@1231
i "SetPartParam" 25 1 3 "freq_26" 763.675
; 1:A06!11' @1231
i 1.26 25 1 3 26 0.567
; 1:A07!11'@1273
i "SetPartParam" 26 1 3 "freq_27" 813.345
; 1:A07!11' @1273
i 1.27 26 1 3 27 0.567
; 1:A08!11'@1315
i "SetPartParam" 27 1 3 "freq_28" 866.246
; 1:A08!11' @1315
i 1.28 27 1 3 28 0.567
; 1:A09!11'@1357
i "SetPartParam" 28 1 3 "freq_29" 922.587
; 1:A09!11' @1357
i 1.29 28 1 3 29 0.567
; 1:A10!11'@1399
i "SetPartParam" 29 1 3 "freq_30" 982.593
; 1:A10!11' @1399
i 1.30 29 1 3 30 0.567
; 1:A01!11,@206
i "SetPartParam" 31 1 1 "freq_1" 139.321
; 1:A01!11, @206
i 1.01 31 1 1 1 0.567
; 1:A02!11,@248
i "SetPartParam" 31 1 1 "freq_2" 148.383
; 1:A02!11, @248
i 1.02 31 1 1 2 0.567
; 1:A03!11,@290
i "SetPartParam" 31 1 1 "freq_3" 158.034
; 1:A03!11, @290
i 1.03 31 1 1 3 0.567
; 1:A04!11,@332
i "SetPartParam" 31 1 1 "freq_4" 168.312
; 1:A04!11, @332
i 1.04 31 1 1 4 0.567
; 1:A05!11,@374
i "SetPartParam" 31 1 1 "freq_5" 179.259
; 1:A05!11, @374
i 1.05 31 1 1 5 0.567
; 1:A06!11,@416
i "SetPartParam" 31 1 1 "freq_6" 190.919
; 1:A06!11, @416
i 1.06 31 1 1 6 0.567
; 1:A07!11,@458
i "SetPartParam" 31 1 1 "freq_7" 203.336
; 1:A07!11, @458
i 1.07 31 1 1 7 0.567
; 1:A08!11,@500
i "SetPartParam" 31 1 1 "freq_8" 216.561
; 1:A08!11, @500
i 1.08 31 1 1 8 0.567
; 1:A09!11,@542
i "SetPartParam" 31 1 1 "freq_9" 230.647
; 1:A09!11, @542
i 1.09 31 1 1 9 0.567
; 1:A10!11,@584
i "SetPartParam" 31 1 1 "freq_10" 245.648
; 1:A10!11, @584
i 1.10 31 1 1 10 0.567
; 1:A01!11@626
i "SetPartParam" 31 1 2 "freq_11" 278.642
; 1:A01!11 @626
i 1.11 31 1 2 11 0.567
; 1:A02!11@667
i "SetPartParam" 31 1 2 "freq_12" 296.765
; 1:A02!11 @667
i 1.12 31 1 2 12 0.567
; 1:A03!11@708
i "SetPartParam" 31 1 2 "freq_13" 316.067
; 1:A03!11 @708
i 1.13 31 1 2 13 0.567
; 1:A04!11@749
i "SetPartParam" 31 1 2 "freq_14" 336.624
; 1:A04!11 @749
i 1.14 31 1 2 14 0.567
; 1:A05!11@790
i "SetPartParam" 31 1 2 "freq_15" 358.519
; 1:A05!11 @790
i 1.15 31 1 2 15 0.567
; 1:A06!11@831
i "SetPartParam" 31 1 2 "freq_16" 381.837
; 1:A06!11 @831
i 1.16 31 1 2 16 0.567
; 1:A07!11@872
i "SetPartParam" 31 1 2 "freq_17" 406.672
; 1:A07!11 @872
i 1.17 31 1 2 17 0.567
; 1:A08!11@913
i "SetPartParam" 31 1 2 "freq_18" 433.123
; 1:A08!11 @913
i 1.18 31 1 2 18 0.567
; 1:A09!11@954
i "SetPartParam" 31 1 2 "freq_19" 461.294
; 1:A09!11 @954
i 1.19 31 1 2 19 0.567
; 1:A10!11@995
i "SetPartParam" 31 1 2 "freq_20" 491.297
; 1:A10!11 @995
i 1.20 31 1 2 20 0.567
; 1:A01!11'@1036
i "SetPartParam" 31 1 3 "freq_21" 557.284
; 1:A01!11' @1036
i 1.21 31 1 3 21 0.567
; 1:A02!11'@1078
i "SetPartParam" 31 1 3 "freq_22" 593.53
; 1:A02!11' @1078
i 1.22 31 1 3 22 0.567
; 1:A03!11'@1120
i "SetPartParam" 31 1 3 "freq_23" 632.134
; 1:A03!11' @1120
i 1.23 31 1 3 23 0.567
; 1:A04!11'@1162
i "SetPartParam" 31 1 3 "freq_24" 673.249
; 1:A04!11' @1162
i 1.24 31 1 3 24 0.567
; 1:A05!11'@1204
i "SetPartParam" 31 1 3 "freq_25" 717.038
; 1:A05!11' @1204
i 1.25 31 1 3 25 0.567
; 1:A06!11'@1246
i "SetPartParam" 31 1 3 "freq_26" 763.675
; 1:A06!11' @1246
i 1.26 31 1 3 26 0.567
; 1:A07!11'@1288
i "SetPartParam" 31 1 3 "freq_27" 813.345
; 1:A07!11' @1288
i 1.27 31 1 3 27 0.567
; 1:A08!11'@1330
i "SetPartParam" 31 1 3 "freq_28" 866.246
; 1:A08!11' @1330
i 1.28 31 1 3 28 0.567
; 1:A09!11'@1372
i "SetPartParam" 31 1 3 "freq_29" 922.587
; 1:A09!11' @1372
i 1.29 31 1 3 29 0.567
; 1:A10!11'@1414
i "SetPartParam" 31 1 3 "freq_30" 982.593
; 1:A10!11' @1414
i 1.30 31 1 3 30 0.567
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
