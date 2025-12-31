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
; [p1.2] => 1.3

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 3
t 0 72
i 1.1 0 1 1 261.626 0 0.567 ; 1:c @325
i 1.2 0 1 1 327.032 0 0.567 ; 1:e @336
i 1.3 0 1 1 392.438 0 0.567 ; 1:g @347
i 1.1 1 1 1 327.032 0 0.567 ; 1:c @427
i 1.2 1 1 1 408.79 0 0.567 ; 1:e @438
i 1.3 1 1 1 490.548 0 0.567 ; 1:g @449
i 1.1 2 1 1 261.626 0 0.567 ; 1:c @527
i 1.2 2 1 1 327.032 0 0.567 ; 1:e @538
i 1.3 2 1 1 392.438 0 0.567 ; 1:g @549
i 1.1 3 1 1 264 0 0.567 ; 1:c @637
i 1.2 3 1 1 330 0 0.567 ; 1:e @648
i 1.3 3 1 1 396 0 0.567 ; 1:g @659
i 1.1 4 1 1 297 0 0.567 ; 1:c @727
i 1.2 4 1 1 371.25 0 0.567 ; 1:e @738
i 1.3 4 1 1 445.5 0 0.567 ; 1:g @749
i 1.1 5 1 1 264 0 0.567 ; 1:c @825
i 1.2 5 1 1 330 0 0.567 ; 1:e @836
i 1.3 5 1 1 396 0 0.567 ; 1:g @847
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
