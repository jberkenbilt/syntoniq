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
; [part.note] => instr.note
; [p1.0] => 1.1

i "SetPartParam" 0 0.01 1 "amp" 0.5
t 0 120
i "SetPartParam" 0 0.01 1 "notes" 1
i 1.1 0 1 1 391.995 0.567 ; 1:g @52
i 1.1 1 1 1 349.228 0.567 ; f @56
i 1.1 2 1 1 293.665 0.567 ; d @58
; mark 'verse-start' @'[80,93)
i 1.1 3 1 1 261.626 0.567 ; 1:c @123
i 1.1 4 1 1 329.628 0.567 ; e @127
i 1.1 5 1 1 391.995 0.567 ; g @129
i 1.1 6 1 1 349.228 0.567 ; f @131
i 1.1 7 1 1 329.628 0.567 ; e @133
i 1.1 8 1 1 293.665 0.567 ; d @135
i 1.1 9 1 1 261.626 0.567 ; c @139
i 1.1 10 1 1 329.628 0.567 ; e @141
i 1.1 11 1 1 391.995 0.567 ; g @143
i 1.1 12 2 1 261.626 0.567 ; 2:c @145
; mark 'chorus-main-start' @'[174,193)
i 1.1 15 1 1 261.626 0.567 ; 1:c @202
i 1.1 16 1 1 329.628 0.567 ; e @206
i 1.1 17 1 1 391.995 0.567 ; g @208
; mark 'chorus-main-end' @'[221,238)
i 1.1 18 2 1 440 0.567 ; 2:a @262
; repeat start 'chorus-main-start' @'[297,316)
i 1.1 21 1 1 261.626 0.567 ; 1:c @202
i 1.1 22 1 1 329.628 0.567 ; e @206
i 1.1 23 1 1 391.995 0.567 ; g @208
; repeat end 'chorus-main-end' @'[321,338)
i 1.1 24 2 1 261.626 0.567 ; 2:c @363
; mark 'verse-end' @'[382,393)
; repeat start 'verse-start' @'[431,444)
i 1.1 27 1 1 261.626 0.567 ; 1:c @123
i 1.1 28 1 1 329.628 0.567 ; e @127
i 1.1 29 1 1 391.995 0.567 ; g @129
i 1.1 30 1 1 349.228 0.567 ; f @131
i 1.1 31 1 1 329.628 0.567 ; e @133
i 1.1 32 1 1 293.665 0.567 ; d @135
i 1.1 33 1 1 261.626 0.567 ; c @139
i 1.1 34 1 1 329.628 0.567 ; e @141
i 1.1 35 1 1 391.995 0.567 ; g @143
i 1.1 36 2 1 261.626 0.567 ; 2:c @145
; mark 'chorus-main-start' @'[174,193)
i 1.1 39 1 1 261.626 0.567 ; 1:c @202
i 1.1 40 1 1 329.628 0.567 ; e @206
i 1.1 41 1 1 391.995 0.567 ; g @208
; mark 'chorus-main-end' @'[221,238)
i 1.1 42 2 1 440 0.567 ; 2:a @262
; repeat start 'chorus-main-start' @'[297,316)
i 1.1 45 1 1 261.626 0.567 ; 1:c @202
i 1.1 46 1 1 329.628 0.567 ; e @206
i 1.1 47 1 1 391.995 0.567 ; g @208
; repeat end 'chorus-main-end' @'[321,338)
i 1.1 48 2 1 261.626 0.567 ; 2:c @363
; repeat end 'verse-end' @'[449,460)
i 1.1 51 2 1 391.995 0.567 ; 2:g @476
i 1.1 53 1 1 391.995 0.567 ; 1:g @480
i 1.1 54 1 1 349.228 0.567 ; f @484
i 1.1 55 1 1 329.628 0.567 ; e @486
i 1.1 56 1 1 293.665 0.567 ; d @488
i 1.1 57 4 1 261.626 0.567 ; 4:c @492
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
