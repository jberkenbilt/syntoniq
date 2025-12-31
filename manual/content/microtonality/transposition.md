+++
title = "Transposition"
weight = 50
sort_by = "weight"
+++

Now that you've learned about defining scales and using generated scales, it's time to talk about transposition. Please see [Language Reference](../../syntoniq_language/language-reference/) for a list of all the directives and their parameters, or run `syntoniq doc` from the command line. This section will cover the two directives you will use for transposition.

A side note on syntax: the transposition directives all take optional `part` parameters that specifies part names. We've been using `p1` as the part name, but you can call it anything that uses alphanumeric characters or the underscore, like `trumpet_2` or `Alto` or `potato`. Syntoniq directives can have repeatable options, so if we wanted to change the base pitch of the tuning for parts `p1` and `p2`, we could call `set_base_pitch(... part="p1" part="p2")`. If no `part` parameters are specified, the directives apply to all parts.

The two directives we will cover here are
* `set_base_pitch` — changes the base pitch of a scale to either an absolute frequency or to a multiple of the current base pitch
* `transpose` — transpose the whole scale so that the note name given as the argument to `written` takes its pitch from the note name given as `pitch_from`

Let's dive into some examples. The first example takes us on a bit of a wild ride through some strange pivots and modulations. I'm not going to claim that it is a great work of art, but it should demonstrate the basics of Syntoniq transposition. Editorial note: the comments in the score below refer to material in the text after the audio. This is in the hope of making this easier to consume on various screen sizes!

<!-- generate include=transposition1.stq checksum=b36219371ef7d887f0d3042b1b99ca97e45d0aaa887d221f9ff0c05a8fdaaf41 -->
```syntoniq
; See Transposition section of manual for the "note x" parts.
syntoniq(version=1)
define_generated_scale(scale="gen-53" divisions=53)
tempo(bpm=40)
use_scale(scale="gen-53")
set_base_pitch(absolute=220*6/5) ; note 1

; Play some chords with some pivots: note 2
[p1.0] 1:A  A
[p1.1] 1:E  E
[p1.2] 1:C  C
[p1.3] 1:A' h':~ ; sustain pivot

; After transpose, `A` will sound like `h` did before: note 3
transpose(written="A" pitch_from="h")

; Do it again
[p1.0] 1:A  A
[p1.1] 1:E  E
[p1.2] 1:C  C
[p1.3] 1:A' h':~

; Pivot 7/8 to 11/8: note 4
transpose(written="Cl" pitch_from="h")

; Pivot, then move to a major triad
[p1.0] 1:A   A
[p1.1] 1:C   C
[p1.2] 1:I'  E'
[p1.3] 1:Cl' C'

; Change keys: note 5
transpose(written="A" pitch_from="D++")

[p1.0] 2:A
[p1.1] 2:C
[p1.2] 2:A'
[p1.3] 2:E'

; Transpose up by two more steps: note 6, then override: note 7
set_base_pitch(relative=^2|53)
[p1.0] 1:A  A   A     2:A,
[p1.1] 1:C  C!  C!19  2:C
[p1.2] 1:A' A'  A'    2:E'
[p1.3] 1:E' E!' E!19' 2:A'2
```
<!-- generate-end -->

{{ audio(src="transposition1-csound.mp3", caption="Transposition Example 1") }}

Notes from above:
1. When setting the base pitch, we chose `220*6/5` to clearly indicate a 6/5 minor third above 220 Hz. We could have written 264. This is just to show we can set the base pitch to any frequency.
2. We start by playing some chords and pivoting on the `h'` note, which is the septimal minor seventh. The `:~` after the note indicates a sustain. The sustain goes to the next note with the same *note number*, which comes from the line prefix. In this case, this is `[p1.3]` (note 3 of part `p1`), so the note is tied to the next note in `[p1.3]`.
3. This is the first example of the transpose syntax. When we say `transpose(written="A" pitch_from="h")`, we are saying that, after transposition, the "written note `A`" will get its pitch from the pitch currently belonging to note `h`. This shifts the pitch down by a ratio of 7/8. It's tricky to indicate the transposition direction clearly, so think of this as describing the "state change". When we transpose, we are saying that a given written note gets its pitch from something else, and in our case, the something else can only be the tuning before transposition. Clear? Hopefully it will become clear!
4. This time, we take the pitch of `h` and give it to `Cl`. Since `h` is below the root by a little more than a whole step (7/8) and `Cl` is above the root by a little more than a fourth (11/8), this will move the pitch down by more than a fifth. But you don't really have to worry about that too much. We're saying the new `Cl` sounds like the old `h`. That means the new `Cl'` sounds like the old `h'`. As we sustain the note in `[p1.3]` again, the transposition clearly tells us what the new note has to be to keep the same sound.
5. At this point in the music, it sounds like we are setting up for a key change: it feels like a dominant wanting to shift up a fourth to a new tonic. To do this, we say the note `A` (the root) should get its pitch from what is currently a fourth up, which would be `D`. But as a little microtonal twist, let's worm-hole to a new key two 53-EDO steps above the fourth by taking the pitch from `D++`. We could have also written this as `DA2`, since `A2` means two steps of the current interval division.
6. This time, we transpose using `set_base_pitch(relative=^2|53)`, meaning to multiply `^2|53` to the pitch. This is the same as going up two steps. We could have used `A` and `A2` in a transpose statement, but the intent is easier to read here. The transposition amount doesn't have to be related to the scale in any way.
7. After the transposition, play the chord a few more times. The second time, use `!` to override the divisions and play the pure JI intervals. 53-EDO is tight, so there's not much difference, but there's a little. Then use `!19` to find the closest note in 19-EDO. Here, the major third is a bit flat, so this sounds noticeably different. Finally, change the chord's voicing and return to 53-EDO. We could have done all this by defining more scales repeatedly calling `use_scale`, but to just "borrow" a note from another scale, using the overrides is easier.

For a second example, let's take a little melody in one EDO, and then use some steps in a second EDO to change to a new, unrelated key. This melody will start in 17-EDO, one of my personal favorites. The single step in 17-EDO is almost exactly the ratio 25/24 (off by less than 0.1¢), which is what you get if you go up a major third and down a minor third in just intonation ($\frac{5}{4}\times\frac{5}{6} = \frac{25}{24}$). Two steps is almost exactly 13/12, flat by less than 3¢. That makes the notes `Y` and `M` particularly useful in 17-EDO. 17-EDO also has a very good fourth and fifth: 10 steps of 17-EDO is less than 4¢ sharp for 3/2. It lacks an interval close to the major third, but 5 steps is a close neutral third, quite close to 11/9. 11/9 can be written as `JK` in our system. This is a feature that falls out of normal arithmetic. Each single letter represents a single harmonic sequence step by design. That means each pair of adjacent letters represents two steps: `JK` = $\frac{10}{9}\times\frac{11}{10} = \frac{11}{9}$.

<!-- generate include=transposition2.stq checksum=a0e19521eaed67bbf24e9d1afce1e91ea5dc52d57940aee17a098d32fcea31fe -->
```syntoniq
; See Transposition section of manual for the "note x" parts.
syntoniq(version=1)
define_generated_scale(scale="gen-17" divisions=17)
define_generated_scale(scale="gen-13" divisions=13)
tempo(bpm=40)
use_scale(scale="gen-17")

; Use a bar check: note 1
[p1.0] 1:A    A   2:A    | 1:A    MA   2:A
[p1.1] 1:JK   I   2:JK   | 1:JK   MJK  2:JK
[p1.2] 1:C    D   2:C    | 1:C    MC   2:C
[p1.3] 1:CJK  DJK 2:CJK  | 1:CJK  MCJK 2:CJK
[p1.4] 1:I'   A'  2:I'   | 1:I'   MI'  2:I'

; Repeat up a step: note 2
transpose(written="A" pitch_from="Y")
[p1.0] 1:A    A   2:A    | 1:A    MA   2:A
[p1.1] 1:JK   I   2:JK   | 1:JK   MJK  2:JK
[p1.2] 1:C    D   2:C    | 1:C    MC   2:C
[p1.3] 1:CJK  DJK 2:CJK  | 1:CJK  MCJK 2:CJK:~ ; sustain
[p1.4] 1:I'   A'  2:I'   | 1:I'   MI'  2:I'

; Sustain and pivot: note 3
use_scale(scale="gen-13")
transpose(written="A" pitch_from="CJK!17")
[p1.3] 1:a a1 a2 a3

use_scale(scale="gen-17")
transpose(written="CJK" pitch_from="a4!13")
; Repeat after stepping in 13-EDO: note 4
[p1.0] 1:A    A   2:A    | 1:A    MA   2:A
[p1.1] 1:JK   I   2:JK   | 1:JK   MJK  2:JK
[p1.2] 1:C    D   2:C    | 1:C    MC   2:C
[p1.3] 1:CJK  DJK 2:CJK  | 1:CJK  MCJK 2:CJK
[p1.4] 1:I'   A'  2:I'   | 1:I'   MI'  2:I'

mark(label="a")
; Closing sequence: note 5
[p1.0] 6:A,
[p1.1] 6:C,
[p1.2] 1:I IM I Im 2:I
[p1.3] 1:C CM C Cm 2:C
[p1.4] 4:~         2:CE#
```
<!-- generate-end -->

{{ audio(src="transposition2-csound.mp3", caption="Transposition Example 2") }}

Notes:
1. This example introduces the `|` character as a "bar check". Syntoniq makes sure that each line in a score block has the same number of bar checks and that each bar check happens at the same beat offset. While Syntoniq doesn't have the concept of time signatures, these can be useful checks. Syntoniq also ensures beats are consistent at the end of each line. This first passage is some native 17-EDO harmonies involving use of the neutral third and the two-step 13/12 interval.
2. Here we repeat the same passage up one step, but we go up a step using the note `Y`, which corresponds closely to a single 17-EDO step...but this would be portable to other scales and would sound similar but with the flavor of that tuning system.
3. Here we switch to a new scale (13-EDO) and just use step sizes (`a` notes). 13-EDO doesn't map very well to the diatonic scale, and the intention here is to just demonstrate stepping. Notice that our transposition assigns the pitch to the note `A` from `CJK!17`. This prevents us from having to do the transposition in multiple steps. We can be in 13-EDO and still take a pitch from a note in 17-EDO. Then we just step a single note along in 13-EDO.
4. Now we're back to 17-EDO and repeat the same chord sequence in the new key, defined by stepping through 13-EDO. You can't really express this cleanly in another way. 221-EDO would exactly contain 13-EDO and 17-EDO (since $13\times 17=221$), but that's a bit silly. Maybe you would never want to do this...but Syntoniq gives you the ability to travel through alien landscapes like this if you feel like it.
5. This wraps up with a chord sequence. The last note in `[p1.4]` is `CE#`. 17-EDO doesn't have a major third (`E`), and the closest note to 5/4 is the 5-step neutral third. By including the `#` character, we are telling Syntoniq to go to the *next higher step* instead of the *closest step*. This gives us a very sharp major third (almost a flat fourth)—an intentional musical choice in this case.

You have now seen how to use transposition in Syntoniq, and you've seen most of the important features. The rest is simple in comparison. The remaining features of the language will be covered in the next part.
