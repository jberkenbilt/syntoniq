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
; [p2] => 2
; [part.note] => instr.note
; [p1.0] => 1.1
; [p1.1] => 1.3
; [p1.2] => 1.4
; [p2.0] => 1.2

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 3
i "SetPartParam" 0 0.01 2 "amp" 0.5
i "SetPartParam" 0 0.01 2 "notes" 3
t 0 72
; 1:c:~@58
i "SetPartParam" 0 1 1 "freq_1" 261.626
; c:&~@67
i "SetPartParamRamp" 1 1 1 "freq_1" 261.626 293.665
; d:~@72
i "SetPartParam" 2 1 1 "freq_1" 293.665
; d:&~@76
i "SetPartParamRamp" 3 1 1 "freq_1" 293.665 329.628
; e:&~@81
i "SetPartParamRamp" 4 1 1 "freq_1" 329.628 293.665
; d:&~@87
i "SetPartParamRamp" 5 1 1 "freq_1" 293.665 261.626
; c:~@94
i "SetPartParam" 6 1 1 "freq_1" 261.626
; c@98
i "SetPartParam" 7 1 1 "freq_1" 261.626
; 1:c:~ @58
i 1.1 0 8 1 1 0.567
; 1:c,3:&~@107
i "SetPartParamRamp" 0 4 2 "freq_2" 32.703 2093.005
; c'3:~@130
i "SetPartParam" 4 1 2 "freq_2" 2093.005
; g'2:&~@136
i "SetPartParamRamp" 5 1 2 "freq_2" 1567.982 261.626
; c:&~@143
i "SetPartParamRamp" 6 1 2 "freq_2" 261.626 130.813
; c,@148
i "SetPartParam" 7 1 2 "freq_2" 130.813
; 1:c,3:&~ @107
i 1.2 0 8 2 2 0.567
; 3:c:&~@159
i "SetPartParamRamp" 8 3 1 "freq_1" 261.626 261.626
; 3:A:~@225
i "SetPartParam" 11 3 1 "freq_1" 261.626
; 3:A:&~@265
i "SetPartParamRamp" 14 3 1 "freq_1" 261.626 65.406
; 3:A,2@308
i "SetPartParam" 17 3 1 "freq_1" 65.406
; 3:c:&~ @159
i 1.1 8 12 1 1 0.567
; 3:e:&~@173
i "SetPartParamRamp" 8 3 1 "freq_3" 329.628 327.032
; 3:E:~@238
i "SetPartParam" 11 3 1 "freq_3" 327.032
; 3:E:&~@279
i "SetPartParamRamp" 14 3 1 "freq_3" 327.032 163.516
; 3:E,@321
i "SetPartParam" 17 3 1 "freq_3" 163.516
; 3:e:&~ @173
i 1.3 8 12 1 3 0.567
; 3:g:&~@187
i "SetPartParamRamp" 8 3 1 "freq_4" 391.995 392.438
; 3:C:~@251
i "SetPartParam" 11 3 1 "freq_4" 392.438
; 3:C:&~@293
i "SetPartParamRamp" 14 3 1 "freq_4" 392.438 784.877
; 3:C'@333
i "SetPartParam" 17 3 1 "freq_4" 784.877
; 3:g:&~ @187
i 1.4 8 12 1 4 0.567
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
