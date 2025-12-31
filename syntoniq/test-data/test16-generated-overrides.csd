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
t 0 72
i 1.1 0 1 1 360 0.567 ; 1:C @133
i 1.1 1 1 1 1080 0.567 ; C' @137
i 1.1 2 1 1 360 0.567 ; C! @140
i 1.1 3 1 1 1080 0.567 ; C!' @143
i 1.1 4 1 1 367.924 0.567 ; C!18 @147
i 1.1 5 1 1 122.641 0.567 ; C!18, @152
i 1.1 6 1 1 359.594 0.567 ; C!2/12 @158
i 1.1 7 1 1 119.865 0.567 ; C!2/12, @165
i 1.1 8 1 1 256.022 0.567 ; A1!17 @173
i 1.1 9 1 1 360 0.567 ; 1:C @267
i 1.1 10 1 1 720 0.567 ; C' @271
i 1.1 11 1 1 360 0.567 ; C! @274
i 1.1 12 1 1 720 0.567 ; C!' @277
i 1.1 13 1 1 367.924 0.567 ; C!18 @281
i 1.1 14 1 1 183.962 0.567 ; C!18, @286
i 1.1 15 1 1 359.594 0.567 ; C!2/12 @292
i 1.1 16 1 1 179.797 0.567 ; C!2/12, @299
i 1.1 17 1 1 256.022 0.567 ; A1!17 @307
i 1.1 18 1 1 355.135 0.567 ; 1:C @430
i 1.1 19 1 1 1065.404 0.567 ; C' @434
i 1.1 20 1 1 360 0.567 ; C! @437
i 1.1 21 1 1 1080 0.567 ; C!' @440
i 1.1 22 1 1 355.135 0.567 ; C!18 @444
i 1.1 23 1 1 118.378 0.567 ; C!18, @449
i 1.1 24 1 1 359.594 0.567 ; C!2/12 @455
i 1.1 25 1 1 119.865 0.567 ; C!2/12, @462
i 1.1 26 1 1 248.443 0.567 ; A1!17 @470
i 1.1 27 1 1 252.048 0.567 ; A1 @476
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
