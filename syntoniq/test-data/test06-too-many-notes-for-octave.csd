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
; syntoniq notes. By design, the instrument's parameters only include
; required parameters (instrument, start time, duration) and
; identification of part and note numbers. This allows arbitrary new
; parameters or changes in behavior, such as ramping previously
; constant values, without breaking backward compatibility.
instr 1
  ; p1..p3 are always instrument, start time, duration
  iPartNum = p4
  iNoteNum = p5
  iVelocity = p6 // 0 to 1

  SFreqChan sprintf "p%d_freq_%d", iPartNum, iNoteNum
  SAmpChan sprintf "p%d_amp", iPartNum
  SNotesChan sprintf "p%d_notes", iPartNum
  kBaseVol chnget SAmpChan
  kNoteCount chnget SNotesChan
  kFreq chnget SFreqChan

  kNoteCount = (kNoteCount == 0 ? 1 : kNoteCount)
  kAmp = kBaseVol * iVelocity
  ; Attenuate based on polyphony
  kFinalAmp = kAmp / sqrt(kNoteCount)
  aEnv madsr 0.05, 0.05, 0.9, 0.15

  ; For most of the frequency range, we use a custom sound mixed with
  ; specific harmonics. At higher frequency ranges, we fall back to a
  ; sine/triangle mix for fewer artifacts.
  aMain poscil3 1, kFreq, 1

  ; blend sine and triangle
  aSine poscil3 0.9, kFreq
  aTriangle vco2 0.9, kFreq, 12
  aHigh = (aSine * 0.5) + (aTriangle * 0.5)

  ; For frequencies in the range of iLowThresh to iHighThresh,
  ; interpolate how much of the main mix we want. It drops to 0
  ; through that range.
  iLowThresh = 2000
  iHighThresh = 4000
  ; map iLowThresh, iHighThresh -> 1, 0 and clamp
  kInterp linlin kFreq, 1, 0, iLowThresh, iHighThresh
  kMainMix limit kInterp, 0, 1

  ; blend
  kHighMix = 1 - kMainMix
  aSignal = (aHigh * kHighMix) + (aMain * kMainMix) * aEnv * kFinalAmp
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
; 1/2:c,3@211
i "SetPartParam" 0 0.5 1 "freq_1" 32.703
; 1/2:c,3 @211
i 1.1 0 0.5 1 1 0.567
; e-,3@219
i "SetPartParam" 0.5 0.5 1 "freq_1" 39.993
; e-,3 @219
i 1.1 0.5 0.5 1 1 0.567
; g,3@224
i "SetPartParam" 1 0.5 1 "freq_1" 48.908
; g,3 @224
i 1.1 1 0.5 1 1 0.567
; b-,3@228
i "SetPartParam" 1.5 0.5 1 "freq_1" 59.811
; b-,3 @228
i 1.1 1.5 0.5 1 1 0.567
; 1/2:c,2@241
i "SetPartParam" 2 0.5 1 "freq_1" 65.406
; 1/2:c,2 @241
i 1.1 2 0.5 1 1 0.567
; e-,2@249
i "SetPartParam" 2.5 0.5 1 "freq_1" 79.986
; e-,2 @249
i 1.1 2.5 0.5 1 1 0.567
; g,2@254
i "SetPartParam" 3 0.5 1 "freq_1" 97.816
; g,2 @254
i 1.1 3 0.5 1 1 0.567
; b-,2@258
i "SetPartParam" 3.5 0.5 1 "freq_1" 119.621
; b-,2 @258
i 1.1 3.5 0.5 1 1 0.567
; 1/2:c,1@271
i "SetPartParam" 4 0.5 1 "freq_1" 130.813
; 1/2:c,1 @271
i 1.1 4 0.5 1 1 0.567
; e-,1@279
i "SetPartParam" 4.5 0.5 1 "freq_1" 159.973
; e-,1 @279
i 1.1 4.5 0.5 1 1 0.567
; g,1@284
i "SetPartParam" 5 0.5 1 "freq_1" 195.633
; g,1 @284
i 1.1 5 0.5 1 1 0.567
; b-,1@288
i "SetPartParam" 5.5 0.5 1 "freq_1" 239.242
; b-,1 @288
i 1.1 5.5 0.5 1 1 0.567
; 1/2:c@301
i "SetPartParam" 6 0.5 1 "freq_1" 261.626
; 1/2:c @301
i 1.1 6 0.5 1 1 0.567
; e-@307
i "SetPartParam" 6.5 0.5 1 "freq_1" 319.945
; e- @307
i 1.1 6.5 0.5 1 1 0.567
; g@310
i "SetPartParam" 7 0.5 1 "freq_1" 391.266
; g @310
i 1.1 7 0.5 1 1 0.567
; b-@312
i "SetPartParam" 7.5 0.5 1 "freq_1" 478.484
; b- @312
i 1.1 7.5 0.5 1 1 0.567
; 1/2:c'1@323
i "SetPartParam" 8 0.5 1 "freq_1" 523.251
; 1/2:c'1 @323
i 1.1 8 0.5 1 1 0.567
; e-'1@331
i "SetPartParam" 8.5 0.5 1 "freq_1" 639.891
; e-'1 @331
i 1.1 8.5 0.5 1 1 0.567
; g'1@336
i "SetPartParam" 9 0.5 1 "freq_1" 782.531
; g'1 @336
i 1.1 9 0.5 1 1 0.567
; b-'1@340
i "SetPartParam" 9.5 0.5 1 "freq_1" 956.968
; b-'1 @340
i 1.1 9.5 0.5 1 1 0.567
; 1/2:c'2@353
i "SetPartParam" 10 0.5 1 "freq_1" 1046.502
; 1/2:c'2 @353
i 1.1 10 0.5 1 1 0.567
; e-'2@361
i "SetPartParam" 10.5 0.5 1 "freq_1" 1279.782
; e-'2 @361
i 1.1 10.5 0.5 1 1 0.567
; g'2@366
i "SetPartParam" 11 0.5 1 "freq_1" 1565.063
; g'2 @366
i 1.1 11 0.5 1 1 0.567
; b-'2@370
i "SetPartParam" 11.5 0.5 1 "freq_1" 1913.937
; b-'2 @370
i 1.1 11.5 0.5 1 1 0.567
; 1/2:c'3@383
i "SetPartParam" 12 0.5 1 "freq_1" 2093.005
; 1/2:c'3 @383
i 1.1 12 0.5 1 1 0.567
; e-'3@391
i "SetPartParam" 12.5 0.5 1 "freq_1" 2559.564
; e-'3 @391
i 1.1 12.5 0.5 1 1 0.567
; g'3@396
i "SetPartParam" 13 0.5 1 "freq_1" 3130.126
; g'3 @396
i 1.1 13 0.5 1 1 0.567
; b-'3@400
i "SetPartParam" 13.5 0.5 1 "freq_1" 3827.874
; b-'3 @400
i 1.1 13.5 0.5 1 1 0.567
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
