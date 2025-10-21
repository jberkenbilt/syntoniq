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
t 0 90 2 90 2 60 4 60 4 90 6 90 6 180 6 180 6 90 7 90 7 60
i "SetPartParam" 0 0.01 1 "notes" 1
i 1.1 0 0.5 1 349.228 0.567 ; 1/2:f @693
i 1.1 0.5 0.5 1 349.228 0.567 ; f @699
i 1.1 1 0.5 1 349.228 0.567 ; f @701
i 1.1 1.5 0.5 1 349.228 0.567 ; f @703
i 1.1 2 0.5 1 391.995 0.567 ; 1/2:g @857
i 1.1 2.5 0.5 1 391.995 0.567 ; g @863
i 1.1 3 0.5 1 391.995 0.567 ; g @865
i 1.1 3.5 0.5 1 391.995 0.567 ; g @867
; mark 'c' @'[910,913)
i 1.1 4 0.5 1 440 0.567 ; 1/2:a @940
i 1.1 4.5 0.5 1 440 0.567 ; a @946
i 1.1 5 0.5 1 440 0.567 ; a @948
i 1.1 5.5 0.5 1 440 0.567 ; a @950
; mark 'd' @'[1008,1011)
i 1.1 7 0.5 1 493.883 0.567 ; 1/2:b @1270
i 1.1 7.5 0.5 1 493.883 0.567 ; b @1276
i 1.1 8 0.5 1 493.883 0.567 ; b @1278
i 1.1 8.5 0.5 1 493.883 0.567 ; b @1280
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
