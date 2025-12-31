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
t 0 72
i 1.1 0 0.5 1 32.703 0 0.567 ; 1/2:c,3 @211
i 1.1 0.5 0.5 1 39.993 0 0.567 ; e-,3 @219
i 1.1 1 0.5 1 48.908 0 0.567 ; g,3 @224
i 1.1 1.5 0.5 1 59.811 0 0.567 ; b-,3 @228
i 1.1 2 0.5 1 65.406 0 0.567 ; 1/2:c,2 @241
i 1.1 2.5 0.5 1 79.986 0 0.567 ; e-,2 @249
i 1.1 3 0.5 1 97.816 0 0.567 ; g,2 @254
i 1.1 3.5 0.5 1 119.621 0 0.567 ; b-,2 @258
i 1.1 4 0.5 1 130.813 0 0.567 ; 1/2:c,1 @271
i 1.1 4.5 0.5 1 159.973 0 0.567 ; e-,1 @279
i 1.1 5 0.5 1 195.633 0 0.567 ; g,1 @284
i 1.1 5.5 0.5 1 239.242 0 0.567 ; b-,1 @288
i 1.1 6 0.5 1 261.626 0 0.567 ; 1/2:c @301
i 1.1 6.5 0.5 1 319.945 0 0.567 ; e- @307
i 1.1 7 0.5 1 391.266 0 0.567 ; g @310
i 1.1 7.5 0.5 1 478.484 0 0.567 ; b- @312
i 1.1 8 0.5 1 523.251 0 0.567 ; 1/2:c'1 @323
i 1.1 8.5 0.5 1 639.891 0 0.567 ; e-'1 @331
i 1.1 9 0.5 1 782.531 0 0.567 ; g'1 @336
i 1.1 9.5 0.5 1 956.968 0 0.567 ; b-'1 @340
i 1.1 10 0.5 1 1046.502 0 0.567 ; 1/2:c'2 @353
i 1.1 10.5 0.5 1 1279.782 0 0.567 ; e-'2 @361
i 1.1 11 0.5 1 1565.063 0 0.567 ; g'2 @366
i 1.1 11.5 0.5 1 1913.937 0 0.567 ; b-'2 @370
i 1.1 12 0.5 1 2093.004 0 0.567 ; 1/2:c'3 @383
i 1.1 12.5 0.5 1 2559.564 0 0.567 ; e-'3 @391
i 1.1 13 0.5 1 3130.126 0 0.567 ; g'3 @396
i 1.1 13.5 0.5 1 3827.874 0 0.567 ; b-'3 @400
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
