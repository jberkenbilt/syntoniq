+++
title = "Save and Restore Pitch"
weight = 60
sort_by = "weight"
+++

Now that you've learned how to do transposition in Syntoniq, we'll introduce three new directives. Sometimes transposition can be disorienting in Syntoniq as it lends itself to lots of use of relative pitches. These directives provide an alternative to the `transpose` directive and can help you stay oriented.

We will cover three directives:
* `save_pitch` — saves the pitch of a specified note in a *variable*
* `restore_pitch` — transposes a part so that the pitch stored in a variable is assigned to a particular note
* `check_pitch` — checks that all its parameters, which can be notes, variables, or pitches, have the same pitch value

You can implement `transpose` using `save_pitch` and `restore_pitch`. `check_pitch` doesn't produce or change any musical output. Its sole purpose is to generate an error if things aren't how you think they are.

<!-- generate include=save-restore1.stq checksum=9f03080b80e13a50c94ec4c511917893a6ec30f8767966afd58886bab8f857e9 -->
```syntoniq
; See Save and Restore Pitch section of manual for the "note x" parts.
syntoniq(version=1)
tempo(bpm=40)
use_scale(scale="JI")

; Set the base pitch to 264 for our part p1. That way, a major sixth
; above the root will have the frequency 440, aligning with
; traditional A440. A major sixth above is a minor third below. That
; puts us off by an octave, so we check the note `f'`.
set_base_pitch(absolute=264)
check_pitch(note=f' pitch=440)

; Major triad, JI
[p1.0] 1:A
[p1.1] 1:E
[p1.2] 1:C

save_pitch(note=E var="orig_E")
save_pitch(note=p var="orig_p")
restore_pitch(note=A var="orig_E")
; Right now `d` is a perfect fourth below the original E. That's
; 3/4*5/4 = 15/16.
check_pitch(note=d var="orig_p" pitch=264*15/16)
[p1.0] 1:d
[p1.1] 1:A
[p1.2] 1:E

restore_pitch(note=C var="orig_E")
[p1.0] 1:A
[p1.1] 1:C
[p1.2] 1:A'

restore_pitch(note=h var="orig_E")
[p1.0] 1:h f
[p1.1] 1:A A
[p1.2] 1:E D

; Lost? Let's check. The original E = 5/4, so A should be 8/7*5/4 =
; 10/7 above the base pitch.
check_pitch(note=A pitch=264*10/7)

; Unrelated but to demonstrate, you can also sanity check generated
; notes. This verifies that a note lands where you think it does. The
; closest scale degree to a major third in 41-EDO is the 13th step.
check_pitch(note=E!41 note=A13!41)
; You can also see how the octave marks work.
check_pitch(note=B note=A')
check_pitch(note=b note=A,)
```
<!-- generate-end -->

{{ audio(src="save-restore1-csound.mp3", caption="Save/Restore Example 1") }}

As shown in the inline comments, we've used variables to save pitches to variables, restore pitches, and check the values of notes and pitches.

You have now seen how to use transposition in Syntoniq, and you've seen most of the important features. The rest is simple in comparison. The remaining features of the language will be covered in the next part.
