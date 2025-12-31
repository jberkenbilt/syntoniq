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
  aEnv madsr 0.05, 0.05, 0.9, 0.15

  ; For most of the frequency range, we use a custom sound mixed with
  ; specific harmonics. At higher frequency ranges, we fall back to a
  ; sine/triangle mix for fewer artifacts.
  aMain poscil3 1, iFreq, 1

  ; blend sine and triangle
  aSine poscil3 0.9, iFreq
  aTriangle vco2 0.9, iFreq, 12
  aHigh = (aSine * 0.5) + (aTriangle * 0.5)

  ; For frequencies in the range of iLowThresh to iHighThresh,
  ; interpolate how much of the main mix we want. It drops to 0
  ; through that range.
  iLowThresh = 2000
  iHighThresh = 4000
  ; map iLowThresh, iHighThresh -> 1, 0 and clamp
  iInterp linlin iFreq, 1, 0, iLowThresh, iHighThresh
  iMainMix limit iInterp, 0, 1

  ; blend
  iHighMix = 1 - iMainMix
  aSignal = (aHigh * iHighMix) + (aMain * iMainMix) * aEnv * kFinalAmp
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
; [part.note] => instr.note
; [p1.0] => 1.1

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 1
t 0 180 3 180 3 225 9 240 15 240 15 270 21 270 21 270 24 270 24 270
i 1.1 0 1 1 391.995 0 0.567 ; 1:g @52
i 1.1 1 1 1 349.228 0 0.567 ; f @56
i 1.1 2 1 1 293.665 0 0.567 ; d @58
; mark 'verse-start' @'[80,93)
i 1.1 3 1 1 261.626 0 0.567 ; 1:c @161
i 1.1 4 1 1 329.628 0 0.567 ; e @165
i 1.1 5 1 1 391.995 0 0.567 ; g @167
i 1.1 6 1 1 349.228 0 0.567 ; f @169
i 1.1 7 1 1 329.628 0 0.567 ; e @171
i 1.1 8 1 1 293.665 0 0.567 ; d @173
i 1.1 9 1 1 261.626 0 0.567 ; c @177
i 1.1 10 1 1 329.628 0 0.567 ; e @179
i 1.1 11 1 1 391.995 0 0.567 ; g @181
i 1.1 12 2 1 261.626 0 0.567 ; 2:c @183
; mark 'chorus-main-start' @'[212,231)
i 1.1 15 1 1 261.626 0 0.567 ; 1:c @255
i 1.1 16 1 1 329.628 0 0.567 ; e @259
i 1.1 17 1 1 391.995 0 0.567 ; g @261
; mark 'chorus-main-end' @'[274,291)
i 1.1 18 2 1 440 0 0.567 ; 2:a @315
i 1.1 21 2 1 261.626 0 0.567 ; 2:c @416
; mark 'verse-end' @'[435,446)
; mark 'ending' @'[533,541)
i 1.1 24 2 1 391.995 0 0.567 ; 2:g @550
i 1.1 26 1 1 391.995 0 0.567 ; 1:g @554
i 1.1 27 1 1 349.228 0 0.567 ; f @558
i 1.1 28 1 1 329.628 0 0.567 ; e @560
i 1.1 29 1 1 293.665 0 0.567 ; d @562
i 1.1 30 4 1 261.626 0 0.567 ; 4:c @566
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
