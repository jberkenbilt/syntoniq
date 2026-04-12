+++
title = "Prompt Mode"
weight = 70
sort_by = "weight"
+++

You can run `syntoniq-kbd prompt` to enter the Syntoniq keyboard application's interactive prompt mode. In this mode, you can type the names of notes using the generated note syntax and name pitches using the pitch notation described in the [pitch primer](../../microtonality/pitch-primer/). When you run `syntoniq-kbd prompt`, the tool will give you a summary of available commands that you can type at the `𝄐` prompt. Run `syntoniq-kbd prompt --help` for additional information.

The currently sounding chord consists of notes numbered from 0 to 255. If you type a note name by itself, it will add that note to the chord, assigning it the lowest available note number. You can also explicitly assign a note to a note number. The following commands are available:

* `?` — show help and current state
* `!!!` — reset all state except variables, turning off all notes and resetting base pitch and transposition
* `!!` — silence all notes, preserving base pitch and transposition
* `>>` — reset transposition to `1`
* `= pitch` — set absolute base pitch; this is the pitch of the note `A`
* `* pitch` — apply relative factor to base pitch
* `% a` — set the cycle ratio to `a`; this affects interpretation of `'` and `,` octave markers
* `% a/b` — set the cycle ratio to `a/b`
* `!` — use just intonation; generated notes without `!` overrides are treated as their pure values
* `!n` — align with n divisions of the octave; this is like appending `!n` to the note name
* `!a/n` — align with n divisions of `a`; like appending `!a/n` to the note name
* `!a/b/n` — align with n divisions of `a/b`; like appending `!a/b/n` to the note name
* `note1 > note2` — transpose to give note1's pitch to note2
* `pitch > note2` — transpose to give pitch, relative to base, to note2
* `note > $var` — save note's pitch into variable `$var`
* `pitch > $var` — save pitch into variable `$var`
* `$var > note` — transpose to set note's pitch to the value in `$var`
* `note` — play note, assigning the lowest available note number
* `pitch` — play pitch, relative to base, assigning to the lowest available note number
* `$var` — show the pitch saved into `$var`, if any
* `n < note` — play note as note n, replacing any existing value
* `n < pitch` —  play pitch, relative to base, as note n, replacing any existing value
* `n <` — stop playing note n

To exit, press `CTRL-C` or `CTRL-D`.

# Example Session

Below is a screen capture from a sample session. You may see slightly different output.

```
** Commands **
?               -- show this help and current state
!!!             -- reset all state except variables
!!              -- silence all notes
>>              -- reset transposition to 1
= pitch         -- set absolute base pitch
* pitch         -- apply relative factor to base pitch
% a             -- set the cycle ratio to `a`
% a/b           -- set the cycle ratio to `a/b`
!               -- use just intonation
!n              -- align with n divisions of the octave
!a/n            -- align with n divisions of `a`
!a/b/n          -- align with n divisions of `a/b`
note1 > note2   -- transpose to give note1's pitch to note2
pitch > note2   -- transpose to give pitch, relative to base, to note2
note > $var     -- save note's pitch into variable $var
pitch > $var    -- save pitch into variable $var
$var > note     -- transpose to set note's pitch to the value in $var
note            -- play note, assigning to the lowest available note number
pitch           -- play pitch, relative to base, assigning to the lowest available note number
$var            -- show the pitch saved into $var, if any
n < note        -- play note as note n, replacing any existing value
n < pitch       -- play pitch, relative to base, as note n, replacing any existing value
n <             -- stop playing note n
** All notes use generated note syntax. **
Exit with CTRL-C or CTRL-D.
𝄐 A
*  0: A = 220*^1|4 (220*^1|4 × 1 × 1) ≈ 261.626
𝄐 E
   0: A = 220*^1|4 (220*^1|4 × 1 × 1) ≈ 261.626
*  1: E = 275*^1|4 (220*^1|4 × 1 × 5/4) ≈ 327.032
𝄐 C
   0: A = 220*^1|4 (220*^1|4 × 1 × 1) ≈ 261.626
   1: E = 275*^1|4 (220*^1|4 × 1 × 5/4) ≈ 327.032
*  2: C = 330*^1|4 (220*^1|4 × 1 × 3/2) ≈ 392.438
𝄐 1 < F
-     E = 275*^1|4 (220*^1|4 × 1 × 5/4) ≈ 327.032
   0: A = 220*^1|4 (220*^1|4 × 1 × 1) ≈ 261.626
*  1: F = 264*^1|4 (220*^1|4 × 1 × 6/5) ≈ 313.951
   2: C = 330*^1|4 (220*^1|4 × 1 × 3/2) ≈ 392.438
𝄐 1 < E
-     F = 264*^1|4 (220*^1|4 × 1 × 6/5) ≈ 313.951
   0: A = 220*^1|4 (220*^1|4 × 1 × 1) ≈ 261.626
*  1: E = 275*^1|4 (220*^1|4 × 1 × 5/4) ≈ 327.032
   2: C = 330*^1|4 (220*^1|4 × 1 × 3/2) ≈ 392.438
𝄐 Bp
   0: A = 220*^1|4 (220*^1|4 × 1 × 1) ≈ 261.626
   1: E = 275*^1|4 (220*^1|4 × 1 × 5/4) ≈ 327.032
   2: C = 330*^1|4 (220*^1|4 × 1 × 3/2) ≈ 392.438
*  3: Bp = 412.5*^1|4 (220*^1|4 × 1 × 15/8) ≈ 490.548
𝄐 I
   0: A = 220*^1|4 (220*^1|4 × 1 × 1) ≈ 261.626
   1: E = 275*^1|4 (220*^1|4 × 1 × 5/4) ≈ 327.032
   2: C = 330*^1|4 (220*^1|4 × 1 × 3/2) ≈ 392.438
   3: Bp = 412.5*^1|4 (220*^1|4 × 1 × 15/8) ≈ 490.548
*  4: I = 247.5*^1|4 (220*^1|4 × 1 × 9/8) ≈ 294.329
𝄐 4 < I'
-     I = 247.5*^1|4 (220*^1|4 × 1 × 9/8) ≈ 294.329
   0: A = 220*^1|4 (220*^1|4 × 1 × 1) ≈ 261.626
   1: E = 275*^1|4 (220*^1|4 × 1 × 5/4) ≈ 327.032
   2: C = 330*^1|4 (220*^1|4 × 1 × 3/2) ≈ 392.438
   3: Bp = 412.5*^1|4 (220*^1|4 × 1 × 15/8) ≈ 490.548
*  4: I' = 495*^1|4 (220*^1|4 × 1 × 9/4) ≈ 588.658
𝄐 !!
turning off all notes
𝄐 E > $orig_E
𝄐 $orig_E
$orig_E = 5/4
𝄐 $orig_E > A
$orig_E = 5/4
transposition = 5/4
𝄐 A
*  0: A = 275*^1|4 (220*^1|4 × 5/4 × 1) ≈ 327.032
𝄐 E
   0: A = 275*^1|4 (220*^1|4 × 5/4 × 1) ≈ 327.032
*  1: E = 343.75*^1|4 (220*^1|4 × 5/4 × 5/4) ≈ 408.790
𝄐 C
   0: A = 275*^1|4 (220*^1|4 × 5/4 × 1) ≈ 327.032
   1: E = 343.75*^1|4 (220*^1|4 × 5/4 × 5/4) ≈ 408.790
*  2: C = 412.5*^1|4 (220*^1|4 × 5/4 × 3/2) ≈ 490.548
𝄐 !!
turning off all notes
𝄐 $orig_E > E
$orig_E = 5/4
transposition = 1
𝄐 A
*  0: A = 220*^1|4 (220*^1|4 × 1 × 1) ≈ 261.626
𝄐 E
   0: A = 220*^1|4 (220*^1|4 × 1 × 1) ≈ 261.626
*  1: E = 275*^1|4 (220*^1|4 × 1 × 5/4) ≈ 327.032
𝄐 C
   0: A = 220*^1|4 (220*^1|4 × 1 × 1) ≈ 261.626
   1: E = 275*^1|4 (220*^1|4 × 1 × 5/4) ≈ 327.032
*  2: C = 330*^1|4 (220*^1|4 × 1 × 3/2) ≈ 392.438
𝄐 !!
turning off all notes
𝄐 !17
divisions = interval: 2, divisions: 17
𝄐 A
*  0: A = 220*^1|4 (220*^1|4 × 1 × 1) ≈ 261.626
𝄐 JK
   0: A = 220*^1|4 (220*^1|4 × 1 × 1) ≈ 261.626
*  1: JK = 220*^37|68 (220*^1|4 × 1 × ^5|17) ≈ 320.788
𝄐 C
   0: A = 220*^1|4 (220*^1|4 × 1 × 1) ≈ 261.626
   1: JK = 220*^37|68 (220*^1|4 × 1 × ^5|17) ≈ 320.788
*  2: C = 220*^57|68 (220*^1|4 × 1 × ^10|17) ≈ 393.330
𝄐 CJK
   0: A = 220*^1|4 (220*^1|4 × 1 × 1) ≈ 261.626
   1: JK = 220*^37|68 (220*^1|4 × 1 × ^5|17) ≈ 320.788
   2: C = 220*^57|68 (220*^1|4 × 1 × ^10|17) ≈ 393.330
*  3: CJK = 440*^9|68 (220*^1|4 × 1 × ^15|17) ≈ 482.275
𝄐 !!!
turning off all notes
resetting state
𝄐 5/4
*  0: 5/4 = 275*^1|4 (220*^1|4 × 1 × 5/4) ≈ 327.032
𝄐 9/8
   0: 5/4 = 275*^1|4 (220*^1|4 × 1 × 5/4) ≈ 327.032
*  1: 9/8 = 247.5*^1|4 (220*^1|4 × 1 × 9/8) ≈ 294.329
𝄐 !!!
turning off all notes
resetting state
𝄐 <CTRL-D>
turning off all notes
```
