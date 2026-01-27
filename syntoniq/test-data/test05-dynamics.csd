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
; [p2] => 2
; [part.note] => instr.note
; [p1.0] => 1.1
; [p1.1] => 1.2
; [p1.2] => 1.3
; [p1.3] => 1.4
; [p2.0] => 1.5
; [p2.1] => 1.6

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 4
i "SetPartParam" 0 0.01 2 "amp" 0.5
i "SetPartParam" 0 0.01 2 "notes" 4
t 0 72
; 4:c@598
i "SetPartParam" 0 4 1 "freq_1" 261.626
; 4:c @598
i 1.1 0 4 1 1 0.567
; 4:e@609
i "SetPartParam" 0 4 1 "freq_2" 327.032
; 4:e @609
i 1.2 0 4 1 2 0.567
; 4:g@620
i "SetPartParam" 0 4 1 "freq_3" 392.438
; 4:g @620
i 1.3 0 4 1 3 0.567
; 2:h7@631
i "SetPartParam" 0 2 1 "freq_4" 457.845
; 2:h7 @631
i 1.4 0 2 1 4 0.567
i "SetPartParamRamp" 0 1 1 "amp" 0.252 0.504 ; 32@0< @646
; 4:g'@677
i "SetPartParam" 0 4 2 "freq_5" 783.991
; 4:g' @677
i 1.5 0 4 2 5 0.567
; 4:c'2@689
i "SetPartParam" 0 4 2 "freq_6" 1046.502
; 4:c'2 @689
i 1.6 0 4 2 6 0.567
i "SetPartParam" 0 0.01 2 "amp" 0.756 ; 96@0 @700
i "SetPartParamRamp" 1 1 1 "amp" 0.504 0.756 ; 64@1< @652
; 2:b%@636
i "SetPartParam" 2 2 1 "freq_4" 465.112
; 2:b% @636
i 1.4 2 2 1 4 0.567
i "SetPartParamRamp" 2 1 1 "amp" 0.756 1 ; 96@2< @658
i "SetPartParam" 3 0.01 1 "amp" 1 ; 127@3 @664
; 4:d@713
i "SetPartParam" 4 4 1 "freq_1" 294.329
; 4:d @713
i 1.1 4 4 1 1 0.567
; 2:f#@724
i "SetPartParam" 4 2 1 "freq_2" 367.911
; 2:f# @724
i 1.2 4 2 1 2 0.567
; 4:a@742
i "SetPartParam" 4 4 1 "freq_3" 436.043
; 4:a @742
i 1.3 4 4 1 3 0.567
i "SetPartParamRamp" 4 1 1 "amp" 1 0.756 ; 127@0> @751
; 4:d'2@782
i "SetPartParam" 4 4 2 "freq_5" 1174.659
; 4:d'2 @782
i 1.5 4 4 2 5 0.567
; 4:f#'2@795
i "SetPartParam" 4 4 2 "freq_6" 1479.978
; 4:f#'2 @795
i 1.6 4 4 2 6 0.567
i "SetPartParam" 4 0.01 2 "amp" 0.252 ; 32@0 @807
i "SetPartParamRamp" 5 1 1 "amp" 0.756 0.504 ; 96@1> @758
; 2:h11@729
i "SetPartParam" 6 2 1 "freq_2" 359.735
; 2:h11 @729
i 1.2 6 2 1 2 0.567
i "SetPartParamRamp" 6 1 1 "amp" 0.504 0.252 ; 64@2> @764
i "SetPartParam" 7 0.01 1 "amp" 0.252 ; 32@3 @770
; 4:c@895
i "SetPartParam" 8 4 1 "freq_1" 327.032
; 4:c @895
i 1.1 8 4 1 1 0.567
; 4:e@906
i "SetPartParam" 8 4 1 "freq_2" 408.79
; 4:e @906
i 1.2 8 4 1 2 0.567
; 4:g@917
i "SetPartParam" 8 4 1 "freq_3" 490.548
; 4:g @917
i 1.3 8 4 1 3 0.567
; 2:h7@928
i "SetPartParam" 8 2 1 "freq_4" 572.306
; 2:h7 @928
i 1.4 8 2 1 4 0.567
i "SetPartParamRamp" 8 1 1 "amp" 0.252 0.504 ; 32@0< @943
i "SetPartParamRamp" 9 1 1 "amp" 0.504 0.756 ; 64@1< @949
; 2:b%@933
i "SetPartParam" 10 2 1 "freq_4" 581.39
; 2:b% @933
i 1.4 10 2 1 4 0.567
i "SetPartParamRamp" 10 1 1 "amp" 0.756 1 ; 96@2< @955
i "SetPartParam" 11 0.01 1 "amp" 1 ; 127@3 @961
; 4:c@1051
i "SetPartParam" 12 4 1 "freq_1" 261.626
; 4:c @1051
i 1.1 12 4 1 1 0.567
; 4:e@1062
i "SetPartParam" 12 4 1 "freq_2" 327.032
; 4:e @1062
i 1.2 12 4 1 2 0.567
; 4:g@1073
i "SetPartParam" 12 4 1 "freq_3" 392.438
; 4:g @1073
i 1.3 12 4 1 3 0.567
; 2:h7@1084
i "SetPartParam" 12 2 1 "freq_4" 457.845
; 2:h7 @1084
i 1.4 12 2 1 4 0.567
i "SetPartParamRamp" 12 1 1 "amp" 0.252 0.504 ; 32@0< @1099
i "SetPartParamRamp" 13 1 1 "amp" 0.504 0.756 ; 64@1< @1105
; 2:b%@1089
i "SetPartParam" 14 2 1 "freq_4" 465.112
; 2:b% @1089
i 1.4 14 2 1 4 0.567
i "SetPartParamRamp" 14 1 1 "amp" 0.756 1 ; 96@2< @1111
i "SetPartParam" 15 0.01 1 "amp" 1 ; 127@3 @1117
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
