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
; [p1.1] => 1.2
; [p1.2] => 1.3
; [p2.0] => 1.4

i "SetPartParam" 0 0.01 1 "amp" 0.5
i "SetPartParam" 0 0.01 1 "notes" 3
i "SetPartParam" 0 0.01 2 "amp" 0.5
i "SetPartParam" 0 0.01 2 "notes" 3
t 0 45
; 1:d@106
i "SetPartParam" 0 1 1 "freq_1" 293.665
; 1:d @106
i 1.1 0 1 1 1 0.567
; 1:f#@120
i "SetPartParam" 0 1 1 "freq_2" 369.994
; 1:f# @120
i 1.2 0 1 1 2 0.567
; 1:a@134
i "SetPartParam" 0 1 1 "freq_3" 440
; 1:a @134
i 1.3 0 1 1 3 0.567
; mark 'a' @'[219,222)
; 1:p,@282
i "SetPartParam" 2 1 1 "freq_1" 130.813
; 1:p, @282
i 1.1 2 1 1 1 0.567
; 1:c@294
i "SetPartParam" 2 1 2 "freq_4" 261.626
; 1:c @294
i 1.4 2 1 2 4 0.567
; 1:e@331
i "SetPartParam" 3 1 1 "freq_1" 329.628
; 1:e @331
i 1.1 3 1 1 1 0.567
; 1:g@342
i "SetPartParam" 3 1 1 "freq_2" 391.995
; 1:g @342
i 1.2 3 1 1 2 0.567
; 1:b@353
i "SetPartParam" 3 1 1 "freq_3" 493.883
; 1:b @353
i 1.3 3 1 1 3 0.567
; 1:q,@406
i "SetPartParam" 4 1 1 "freq_1" 195.998
; 1:q, @406
i 1.1 4 1 1 1 0.567
; 1:q@418
i "SetPartParam" 4 1 2 "freq_4" 391.995
; 1:q @418
i 1.4 4 1 2 4 0.567
; mark 'b' @'[433,436)
; q@505
i "SetPartParam" 6 1 1 "freq_1" 391.995
; q @505
i 1.1 6 1 1 1 0.567
; g'@521
i "SetPartParam" 6 1 2 "freq_4" 783.991
; g' @521
i 1.4 6 1 2 4 0.567
; mark 'c' @'[619,622)
; repeat start 'a' @'[637,640)
; 1:p,@282
i "SetPartParam" 8 1 1 "freq_1" 130.813
; 1:p, @282
i 1.1 8 1 1 1 0.567
; 1:c@294
i "SetPartParam" 8 1 2 "freq_4" 261.626
; 1:c @294
i 1.4 8 1 2 4 0.567
; 1:e@331
i "SetPartParam" 9 1 1 "freq_1" 329.628
; 1:e @331
i 1.1 9 1 1 1 0.567
; 1:g@342
i "SetPartParam" 9 1 1 "freq_2" 391.995
; 1:g @342
i 1.2 9 1 1 2 0.567
; 1:b@353
i "SetPartParam" 9 1 1 "freq_3" 493.883
; 1:b @353
i 1.3 9 1 1 3 0.567
; 1:q,@406
i "SetPartParam" 10 1 1 "freq_1" 195.998
; 1:q, @406
i 1.1 10 1 1 1 0.567
; 1:q@418
i "SetPartParam" 10 1 2 "freq_4" 391.995
; 1:q @418
i 1.4 10 1 2 4 0.567
; repeat end 'b' @'[645,648)
; p@681
i "SetPartParam" 12 1 1 "freq_1" 261.626
; p @681
i 1.1 12 1 1 1 0.567
; c'@697
i "SetPartParam" 12 1 2 "freq_4" 523.251
; c' @697
i 1.4 12 1 2 4 0.567
; mark 'd' @'[713,716)
; repeat start 'c' @'[793,796)
; repeat start 'a' @'[637,640)
; 1:p,@282
i "SetPartParam" 14 1 1 "freq_1" 130.813
; 1:p, @282
i 1.1 14 1 1 1 0.567
; 1:c@294
i "SetPartParam" 14 1 2 "freq_4" 261.626
; 1:c @294
i 1.4 14 1 2 4 0.567
; 1:e@331
i "SetPartParam" 15 1 1 "freq_1" 329.628
; 1:e @331
i 1.1 15 1 1 1 0.567
; 1:g@342
i "SetPartParam" 15 1 1 "freq_2" 391.995
; 1:g @342
i 1.2 15 1 1 2 0.567
; 1:b@353
i "SetPartParam" 15 1 1 "freq_3" 493.883
; 1:b @353
i 1.3 15 1 1 3 0.567
; 1:q,@406
i "SetPartParam" 16 1 1 "freq_1" 195.998
; 1:q, @406
i 1.1 16 1 1 1 0.567
; 1:q@418
i "SetPartParam" 16 1 2 "freq_4" 391.995
; 1:q @418
i 1.4 16 1 2 4 0.567
; repeat end 'b' @'[645,648)
; p@681
i "SetPartParam" 18 1 1 "freq_1" 261.626
; p @681
i 1.1 18 1 1 1 0.567
; c'@697
i "SetPartParam" 18 1 2 "freq_4" 523.251
; c' @697
i 1.4 18 1 2 4 0.567
; repeat end 'd' @'[801,804)
; 1:p,@876
i "SetPartParam" 20 1 1 "freq_1" 130.813
; 1:p, @876
i 1.1 20 1 1 1 0.567
; 1:c@888
i "SetPartParam" 20 1 2 "freq_4" 261.626
; 1:c @888
i 1.4 20 1 2 4 0.567
;; END SYNTONIQ

e

</CsScore>
</CsoundSynthesizer>
