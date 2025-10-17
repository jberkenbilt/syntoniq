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
; [p1.1] => 1.2
; [p1.2] => 1.3
; [p1.3] => 1.4

i "SetPartParam" 0 0.01 1 "amp" 0.5
t 0 72
i "SetPartParam" 0 0.01 1 "notes" 1
i 1.1 0 1 1 261.626 0.567 ; 1:p @234
i "SetPartParam" 0 0.01 1 "notes" 2
i 1.2 0 1 1 345.217 0.567 ; 1:r @262
i "SetPartParam" 0 0.01 1 "notes" 3
i 1.3 0 1 1 455.517 0.567 ; 1:t @290
i 1.1 1 1 1 300.529 0.567 ; q @238
i 1.2 1 1 1 396.55 0.567 ; s @266
i 1.3 1 1 1 523.251 0.567 ; p' @294
i 1.1 2 1 1 345.217 0.567 ; r @241
i 1.2 2 1 1 455.517 0.567 ; t @269
i 1.3 2 1 1 601.058 0.567 ; q' @297
i 1.1 3 1 1 396.55 0.567 ; s @244
i 1.2 3 1 1 523.251 0.567 ; p' @272
i 1.3 3 1 1 690.434 0.567 ; r' @300
i 1.1 4 1 1 455.517 0.567 ; t @247
i 1.2 4 1 1 601.058 0.567 ; q' @275
i 1.3 4 1 1 793.1 0.567 ; s' @303
i 1.1 5 1 1 523.251 0.567 ; p' @250
i 1.2 5 1 1 690.434 0.567 ; r' @278
i 1.3 5 1 1 911.033 0.567 ; t' @306
i "SetPartParam" 7 0.01 1 "notes" 1
i 1.1 7 1 1 261.626 0.567 ; 1:c @429
i "SetPartParam" 7 0.01 1 "notes" 2
i 1.2 7 1 1 329.628 0.567 ; 1:e @444
i "SetPartParam" 7 0.01 1 "notes" 3
i 1.3 7 1 1 391.995 0.567 ; 1:g @458
i 1.1 8 1 1 523.251 0.567 ; c' @433
i 1.2 8 1 1 329.628 0.567 ; e @448
i 1.3 8 1 1 195.998 0.567 ; g, @462
i "SetPartParam" 8 0.01 1 "notes" 4
i 1.4 8 1 1 130.813 0.567 ; c, @477
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
