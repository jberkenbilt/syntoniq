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
; [part.note] => instr.note
; [p1.0] => 1.1

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 1
t 0 120 3 120 3 150 9 160 15 160 15 180 21 180 21 180
i 1.1 0 1 1 391.995 0.567 ; 1:g @52
i 1.1 1 1 1 349.228 0.567 ; f @56
i 1.1 2 1 1 293.665 0.567 ; d @58
; mark 'verse-start' @'[80,93)
i 1.1 3 1 1 261.626 0.567 ; 1:c @161
i 1.1 4 1 1 329.628 0.567 ; e @165
i 1.1 5 1 1 391.995 0.567 ; g @167
i 1.1 6 1 1 349.228 0.567 ; f @169
i 1.1 7 1 1 329.628 0.567 ; e @171
i 1.1 8 1 1 293.665 0.567 ; d @173
i 1.1 9 1 1 261.626 0.567 ; c @177
i 1.1 10 1 1 329.628 0.567 ; e @179
i 1.1 11 1 1 391.995 0.567 ; g @181
i 1.1 12 2 1 261.626 0.567 ; 2:c @183
; mark 'chorus-main-start' @'[212,231)
i 1.1 15 1 1 261.626 0.567 ; 1:c @255
i 1.1 16 1 1 329.628 0.567 ; e @259
i 1.1 17 1 1 391.995 0.567 ; g @261
; mark 'chorus-main-end' @'[274,291)
i 1.1 18 2 1 440 0.567 ; 2:a @315
; repeat start 'chorus-main-start' @'[350,369)
i 1.1 21 1 1 261.626 0.567 ; 1:c @255
i 1.1 22 1 1 329.628 0.567 ; e @259
i 1.1 23 1 1 391.995 0.567 ; g @261
; repeat end 'chorus-main-end' @'[374,391)
i 1.1 24 2 1 261.626 0.567 ; 2:c @416
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
