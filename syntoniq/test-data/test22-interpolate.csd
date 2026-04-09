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
; [p1.1] => 1.1
; [p1.2] => 1.2
; [p1.3] => 1.3

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 3
t 0 60 0.5 60 0.5 90 3.5 120 3.5 120 3.5 120 6.5 90 6.5 90 6.5 60 8 60 8 72
; 1/2:c,:~@578
i "SetPartParam" 0 0.5 1 "freq_1" 132
; 3:c,:&~@690
i "SetPartParamRamp" 0.5 3 1 "freq_1" 132 264
; 3:c:&~@803
i "SetPartParamRamp" 3.5 3 1 "freq_1" 264 528
; 1/2:c'@896
i "SetPartParam" 6.5 0.5 1 "freq_1" 528
; 1/2:c,:~ @578
i 1.1 0 7 1 1 0.567
; 1/2:c:~@594
i "SetPartParam" 0 0.5 1 "freq_2" 264
; 1/2:c@705
i "SetPartParam" 0.5 0.5 1 "freq_2" 264
; 1/2:c:~ @594
i 1.2 0 1 1 2 0.567
; 1/2:c,2:~@609
i "SetPartParam" 0 0.5 1 "freq_3" 66
; 3:c,2:~@734
i "SetPartParam" 0.5 3 1 "freq_3" 66
; 3:c,2:~@850
i "SetPartParam" 3.5 3 1 "freq_3" 66
; 1/2:c,2@925
i "SetPartParam" 6.5 0.5 1 "freq_3" 66
; 1/2:c,2:~ @609
i 1.3 0 7 1 3 0.567
i "SetPartParamRamp" 0 7 1 "amp" 0.063 1 ; 8@0< @624
; mark 'a' @'[678,681)
; 1:e@711
i "SetPartParam" 1 1 1 "freq_2" 332.619
; 1:e @711
i 1.2 1 1 1 2 0.567
; g#@715
i "SetPartParam" 2 1 1 "freq_2" 419.074
; g# @715
i 1.2 2 1 1 2 0.567
; 1/2:c':~@718
i "SetPartParam" 3 0.5 1 "freq_2" 528
; 1/2:c'@817
i "SetPartParam" 3.5 0.5 1 "freq_2" 528
; 1/2:c':~ @718
i 1.2 3 1 1 2 0.567
; mark 'b' @'[791,794)
; 1:e'@824
i "SetPartParam" 4 1 1 "freq_2" 665.238
; 1:e' @824
i 1.2 4 1 1 2 0.567
; g#'@829
i "SetPartParam" 5 1 1 "freq_2" 838.148
; g#' @829
i 1.2 5 1 1 2 0.567
; 1/2:c'2:~@833
i "SetPartParam" 6 0.5 1 "freq_2" 1056
; 1/2:c'2@910
i "SetPartParam" 6.5 0.5 1 "freq_2" 1056
; 1/2:c'2:~ @833
i 1.2 6 1 1 2 0.567
; mark 'c' @'[884,887)
i "SetPartParam" 7 0.01 1 "amp" 1 ; 127@0 @950
; mark 'd' @'[982,985)
; 1/2:c':~@994
i "SetPartParam" 8 0.5 1 "freq_1" 528
; 3:c':&~@1073
i "SetPartParamRamp" 8.5 3 1 "freq_1" 528 264
; 3:c:&~@1153
i "SetPartParamRamp" 11.5 3 1 "freq_1" 264 132
; 1/2:c,@1228
i "SetPartParam" 14.5 0.5 1 "freq_1" 132
; 1/2:c':~ @994
i 1.1 8 7 1 1 0.567
; 1/2:c'2:~@1010
i "SetPartParam" 8 0.5 1 "freq_2" 1056
; 1/2:c'2@1088
i "SetPartParam" 8.5 0.5 1 "freq_2" 1056
; 1/2:c'2:~ @1010
i 1.2 8 1 1 2 0.567
; 1/2:c,2:~@1027
i "SetPartParam" 8 0.5 1 "freq_3" 66
; 3:c,2:~@1121
i "SetPartParam" 8.5 3 1 "freq_3" 66
; 3:c,2:~@1196
i "SetPartParam" 11.5 3 1 "freq_3" 66
; 1/2:c,2@1255
i "SetPartParam" 14.5 0.5 1 "freq_3" 66
; 1/2:c,2:~ @1027
i 1.3 8 7 1 3 0.567
i "SetPartParamRamp" 8 6.5 1 "amp" 1 0.063 ; 127@0> @1042
; mark 'e' @'[1061,1064)
; 1:g#'@1096
i "SetPartParam" 9 1 1 "freq_2" 838.148
; 1:g#' @1096
i 1.2 9 1 1 2 0.567
; e'@1102
i "SetPartParam" 10 1 1 "freq_2" 665.238
; e' @1102
i 1.2 10 1 1 2 0.567
; 1/2:c':~@1105
i "SetPartParam" 11 0.5 1 "freq_2" 528
; 1/2:c'@1167
i "SetPartParam" 11.5 0.5 1 "freq_2" 528
; 1/2:c':~ @1105
i 1.2 11 1 1 2 0.567
; mark 'f' @'[1141,1144)
; 1:g#@1174
i "SetPartParam" 12 1 1 "freq_2" 419.074
; 1:g# @1174
i 1.2 12 1 1 2 0.567
; e@1179
i "SetPartParam" 13 1 1 "freq_2" 332.619
; e @1179
i 1.2 13 1 1 2 0.567
; 1/2:c:~@1181
i "SetPartParam" 14 0.5 1 "freq_2" 264
; 1/2:c@1242
i "SetPartParam" 14.5 0.5 1 "freq_2" 264
; 1/2:c:~ @1181
i 1.2 14 1 1 2 0.567
; mark 'g' @'[1216,1219)
i "SetPartParam" 14.5 0.01 1 "amp" 0.063 ; 8@0 @1268
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
