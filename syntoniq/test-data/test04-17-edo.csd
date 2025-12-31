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
instr potato
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
; [p1.0] => "potato".3
; [p1.1] => "potato".2
; [p1.2] => "potato".1

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 3
t 0 72
i "potato.1" 0 6 1 65.406 0 0.567 ; 6:c,2 @376
i "potato.2" 2 1 1 196.665 0 0.567 ; 1:g, @360
i "potato.3" 3 3 1 393.33 0 0.567 ; 3:g @345
i "potato.2" 3 3 1 261.626 0 0.567 ; 3:c @365
i "potato.1" 6 6 1 65.406 0 0.567 ; 6:c,2 @436
i "potato.2" 7 1 1 196.665 0 0.567 ; 1:g, @414
i "potato.2" 8 1 1 213.374 0 0.567 ; a%, @419
i "potato.3" 9 3 1 393.33 0 0.567 ; 3:g @399
i "potato.2" 9 1 1 261.626 0 0.567 ; c @423
i "potato.2" 10 2 1 295.667 0 0.567 ; 2:d @425
i "potato.1" 12 6 1 65.406 0 0.567 ; 6:c,2 @476
i "potato.2" 13 1 1 196.665 0 0.567 ; 1:g, @454
i "potato.2" 14 1 1 213.374 0 0.567 ; a%, @459
i "potato.2" 15 1 1 295.667 0 0.567 ; d @463
i "potato.2" 16 1 1 334.138 0 0.567 ; e @465
i "potato.2" 17 1 1 295.667 0 0.567 ; d @467
i "potato.2" 18 5 1 65.406 0 0.567 ; 5:c,2 @519
i "potato.3" 19 1 1 196.665 0 0.567 ; 1:g, @494
i "potato.3" 20 1 1 213.374 0 0.567 ; a%, @499
i "potato.3" 21 1 1 295.667 0 0.567 ; d @503
i "potato.3" 22 1 1 334.138 0 0.567 ; e @505
i "potato.3" 23 4 1 272.513 0 0.567 ; 4:c# @507
i "potato.2" 23 4 1 60.284 0 0.567 ; 4:b%,3 @536
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
