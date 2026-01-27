;; This file is a copy of csound-template.csd with the instrument name
;; changed to "potato" and the function table changed so the wave form
;; is audibly distinct.

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
instr potato
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
f 1 0 32768 10 1 1 1 1 1 1 1

; i instr start duration [params...]

;; NOTE: for comments that end with @nnn, nnn is the byte offset of
;; the item in the original file.

;; BEGIN SYNTONIQ
; [part] => csound part
; [p1] => 1
; [part.note] => instr.note
; [p1.0] => "potato".3
; [p1.1] => "potato".2
; [p1.2] => "potato".1

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 3
t 0 72
; 6:c,2@376
i "SetPartParam" 0 6 1 "freq_1" 65.406
; 6:c,2 @376
i "potato.1" 0 6 1 1 0.567
; 1:g,@360
i "SetPartParam" 2 1 1 "freq_2" 196.665
; 1:g, @360
i "potato.2" 2 1 1 2 0.567
; 3:g@345
i "SetPartParam" 3 3 1 "freq_3" 393.33
; 3:g @345
i "potato.3" 3 3 1 3 0.567
; 3:c@365
i "SetPartParam" 3 3 1 "freq_2" 261.626
; 3:c @365
i "potato.2" 3 3 1 2 0.567
; 6:c,2@436
i "SetPartParam" 6 6 1 "freq_1" 65.406
; 6:c,2 @436
i "potato.1" 6 6 1 1 0.567
; 1:g,@414
i "SetPartParam" 7 1 1 "freq_2" 196.665
; 1:g, @414
i "potato.2" 7 1 1 2 0.567
; a%,@419
i "SetPartParam" 8 1 1 "freq_2" 213.374
; a%, @419
i "potato.2" 8 1 1 2 0.567
; 3:g@399
i "SetPartParam" 9 3 1 "freq_3" 393.33
; 3:g @399
i "potato.3" 9 3 1 3 0.567
; c@423
i "SetPartParam" 9 1 1 "freq_2" 261.626
; c @423
i "potato.2" 9 1 1 2 0.567
; 2:d@425
i "SetPartParam" 10 2 1 "freq_2" 295.667
; 2:d @425
i "potato.2" 10 2 1 2 0.567
; 6:c,2@476
i "SetPartParam" 12 6 1 "freq_1" 65.406
; 6:c,2 @476
i "potato.1" 12 6 1 1 0.567
; 1:g,@454
i "SetPartParam" 13 1 1 "freq_2" 196.665
; 1:g, @454
i "potato.2" 13 1 1 2 0.567
; a%,@459
i "SetPartParam" 14 1 1 "freq_2" 213.374
; a%, @459
i "potato.2" 14 1 1 2 0.567
; d@463
i "SetPartParam" 15 1 1 "freq_2" 295.667
; d @463
i "potato.2" 15 1 1 2 0.567
; e@465
i "SetPartParam" 16 1 1 "freq_2" 334.138
; e @465
i "potato.2" 16 1 1 2 0.567
; d@467
i "SetPartParam" 17 1 1 "freq_2" 295.667
; d @467
i "potato.2" 17 1 1 2 0.567
; 5:c,2@519
i "SetPartParam" 18 5 1 "freq_2" 65.406
; 5:c,2 @519
i "potato.2" 18 5 1 2 0.567
; 1:g,@494
i "SetPartParam" 19 1 1 "freq_3" 196.665
; 1:g, @494
i "potato.3" 19 1 1 3 0.567
; a%,@499
i "SetPartParam" 20 1 1 "freq_3" 213.374
; a%, @499
i "potato.3" 20 1 1 3 0.567
; d@503
i "SetPartParam" 21 1 1 "freq_3" 295.667
; d @503
i "potato.3" 21 1 1 3 0.567
; e@505
i "SetPartParam" 22 1 1 "freq_3" 334.138
; e @505
i "potato.3" 22 1 1 3 0.567
; 4:c#@507
i "SetPartParam" 23 4 1 "freq_3" 272.513
; 4:c# @507
i "potato.3" 23 4 1 3 0.567
; 4:b%,3@536
i "SetPartParam" 23 4 1 "freq_2" 60.284
; 4:b%,3 @536
i "potato.2" 23 4 1 2 0.567
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
