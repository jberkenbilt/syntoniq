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
; [p2] => 2
; [part.note] => instr.note
; [p1.0] => 1.1
; [p1.1] => 1.2
; [p1.2] => 1.3
; [p2.0] => 1.4

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 3
i "SetPartParam" 0 0.01 2 "amp" 0.5
i "SetPartParam" 0 0.01 2 "notes" 3
t 0 45
i 1.1 0 1 1 293.665 0.567 ; 1:d @106
i 1.2 0 1 1 369.994 0.567 ; 1:f# @120
i 1.3 0 1 1 440 0.567 ; 1:a @134
; mark 'a' @'[219,222)
i 1.1 2 1 1 130.813 0.567 ; 1:p, @282
i 1.4 2 1 2 261.626 0.567 ; 1:c @294
i 1.1 3 1 1 329.628 0.567 ; 1:e @331
i 1.2 3 1 1 391.995 0.567 ; 1:g @342
i 1.3 3 1 1 493.883 0.567 ; 1:b @353
i 1.1 4 1 1 195.998 0.567 ; 1:q, @406
i 1.4 4 1 2 391.995 0.567 ; 1:q @418
; mark 'b' @'[433,436)
i 1.1 6 1 1 391.995 0.567 ; q @505
i 1.4 6 1 2 783.991 0.567 ; g' @521
; mark 'c' @'[619,622)
; repeat start 'a' @'[637,640)
i 1.1 8 1 1 130.813 0.567 ; 1:p, @282
i 1.4 8 1 2 261.626 0.567 ; 1:c @294
i 1.1 9 1 1 329.628 0.567 ; 1:e @331
i 1.2 9 1 1 391.995 0.567 ; 1:g @342
i 1.3 9 1 1 493.883 0.567 ; 1:b @353
i 1.1 10 1 1 195.998 0.567 ; 1:q, @406
i 1.4 10 1 2 391.995 0.567 ; 1:q @418
; repeat end 'b' @'[645,648)
i 1.1 12 1 1 261.626 0.567 ; p @681
i 1.4 12 1 2 523.251 0.567 ; c' @697
; mark 'd' @'[713,716)
; repeat start 'c' @'[793,796)
; repeat start 'a' @'[637,640)
i 1.1 14 1 1 130.813 0.567 ; 1:p, @282
i 1.4 14 1 2 261.626 0.567 ; 1:c @294
i 1.1 15 1 1 329.628 0.567 ; 1:e @331
i 1.2 15 1 1 391.995 0.567 ; 1:g @342
i 1.3 15 1 1 493.883 0.567 ; 1:b @353
i 1.1 16 1 1 195.998 0.567 ; 1:q, @406
i 1.4 16 1 2 391.995 0.567 ; 1:q @418
; repeat end 'b' @'[645,648)
i 1.1 18 1 1 261.626 0.567 ; p @681
i 1.4 18 1 2 523.251 0.567 ; c' @697
; repeat end 'd' @'[801,804)
i 1.1 20 1 1 130.813 0.567 ; 1:p, @876
i 1.4 20 1 2 261.626 0.567 ; 1:c @888
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
