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
; [p2] => 2
; [part.note] => instr.note
; [p1.0] => 1.1
; [p1.1] => 1.2
; [p1.2] => 1.3
; [p1.3] => 1.4
; [p2.0] => 1.5
; [p2.1] => 1.6

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 2 "amp" 0.5
t 0 72
i "SetPartParam" 0 0.01 1 "notes" 1
i 1.1 0 4 1 261.626 0.567 ; 4:c @598
i "SetPartParam" 0 0.01 1 "notes" 2
i 1.2 0 4 1 327.032 0.567 ; 4:e @609
i "SetPartParam" 0 0.01 1 "notes" 3
i 1.3 0 4 1 392.438 0.567 ; 4:g @620
i "SetPartParam" 0 0.01 1 "notes" 4
i 1.4 0 2 1 457.845 0.567 ; 2:h7 @631
i "SetPartParamRamp" 0 1 1 "amp" 0.252 0.504 ; 32@0< @646
i "SetPartParam" 0 0.01 2 "notes" 1
i 1.5 0 4 2 783.991 0.567 ; 4:g' @677
i "SetPartParam" 0 0.01 2 "notes" 2
i 1.6 0 4 2 1046.502 0.567 ; 4:c'2 @689
i "SetPartParam" 0 0.01 2 "amp" 0.756 ; 96@0 @700
i "SetPartParamRamp" 1 1 1 "amp" 0.504 0.756 ; 64@1< @652
i 1.4 2 2 1 465.112 0.567 ; 2:b% @636
i "SetPartParamRamp" 2 1 1 "amp" 0.756 1 ; 96@2< @658
i "SetPartParam" 3 0.01 1 "amp" 1 ; 127@3 @664
i "SetPartParam" 4 0.01 1 "notes" 1
i 1.1 4 4 1 294.329 0.567 ; 4:d @713
i "SetPartParam" 4 0.01 1 "notes" 2
i 1.2 4 2 1 367.911 0.567 ; 2:f# @724
i "SetPartParam" 4 0.01 1 "notes" 3
i 1.3 4 4 1 436.043 0.567 ; 4:a @742
i "SetPartParamRamp" 4 1 1 "amp" 1 0.756 ; 127@0> @751
i "SetPartParam" 4 0.01 2 "notes" 1
i 1.5 4 4 2 1174.659 0.567 ; 4:d'2 @782
i "SetPartParam" 4 0.01 2 "notes" 2
i 1.6 4 4 2 1479.978 0.567 ; 4:f#'2 @795
i "SetPartParam" 4 0.01 2 "amp" 0.252 ; 32@0 @807
i "SetPartParamRamp" 5 1 1 "amp" 0.756 0.504 ; 96@1> @758
i 1.2 6 2 1 359.735 0.567 ; 2:h11 @729
i "SetPartParamRamp" 6 1 1 "amp" 0.504 0.252 ; 64@2> @764
i "SetPartParam" 7 0.01 1 "amp" 0.252 ; 32@3 @770
i "SetPartParam" 8 0.01 1 "notes" 1
i 1.1 8 4 1 327.032 0.567 ; 4:c @895
i "SetPartParam" 8 0.01 1 "notes" 2
i 1.2 8 4 1 408.79 0.567 ; 4:e @906
i "SetPartParam" 8 0.01 1 "notes" 3
i 1.3 8 4 1 490.548 0.567 ; 4:g @917
i "SetPartParam" 8 0.01 1 "notes" 4
i 1.4 8 2 1 572.306 0.567 ; 2:h7 @928
i "SetPartParamRamp" 8 1 1 "amp" 0.252 0.504 ; 32@0< @943
i "SetPartParamRamp" 9 1 1 "amp" 0.504 0.756 ; 64@1< @949
i 1.4 10 2 1 581.39 0.567 ; 2:b% @933
i "SetPartParamRamp" 10 1 1 "amp" 0.756 1 ; 96@2< @955
i "SetPartParam" 11 0.01 1 "amp" 1 ; 127@3 @961
i "SetPartParam" 12 0.01 1 "notes" 1
i 1.1 12 4 1 261.626 0.567 ; 4:c @1051
i "SetPartParam" 12 0.01 1 "notes" 2
i 1.2 12 4 1 327.032 0.567 ; 4:e @1062
i "SetPartParam" 12 0.01 1 "notes" 3
i 1.3 12 4 1 392.438 0.567 ; 4:g @1073
i "SetPartParam" 12 0.01 1 "notes" 4
i 1.4 12 2 1 457.845 0.567 ; 2:h7 @1084
i "SetPartParamRamp" 12 1 1 "amp" 0.252 0.504 ; 32@0< @1099
i "SetPartParamRamp" 13 1 1 "amp" 0.504 0.756 ; 64@1< @1105
i 1.4 14 2 1 465.112 0.567 ; 2:b% @1089
i "SetPartParamRamp" 14 1 1 "amp" 0.756 1 ; 96@2< @1111
i "SetPartParam" 15 0.01 1 "amp" 1 ; 127@3 @1117
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
