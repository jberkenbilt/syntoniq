+++
title = "Tie and Glide"
weight = 60
sort_by = "weight"
+++

Syntoniq supports two ways to sustain a note: tie (`~`) and glide (`&`). In both cases, the note's duration is extended over subsequent holds or subsequent sustained notes. A sustained note ends at the end of the next non-sustained note or when an accented note is encountered.

Here's a example, explained below.

<!-- generate include=tie-glide1.stq checksum=66286ef6b073a19b0bc3cc4560629ac35e518c05a15a71a6f32acb6fb923f950 -->
```syntoniq
syntoniq(version=1)

use_scale(scale="JI")

[p1.1] 2:E:&
[p1.2] 2:A,:~

[p1.1] 2:~  ; glide continues across this line
[p1.2] 2:~  ; tie continues across this line

[p1.1] 2:C
[p1.2] 1:A,:~ A,:^  ; note is re-articulated because of the accent

```
<!-- generate-end -->

{{ audio(src="tie-glide1-csound.mp3", caption="Basic Tie and Glide") }}

In this example, you can see that `[p1.1]` has `E` gradually changing pitch to `C` (these being equivalent to `e` and `g` in 12-EDO) while `[p1.2]` stays on the same pitch. In both cases, the sustain activity carries across the line with the holds. In the final line, we see that the `A` (corresponding to the 12-EDO note `c`) is re-articulated because of the explicit accent mark.

In this example, we mix ties and glides in various ways to morph around between chords.

<!-- generate include=tie-glide2.stq checksum=8d93da00300fdba4bb8b7e02ce33665b38a0ab5d5cc93df183494fa0b329ff1e -->
```syntoniq
syntoniq(version=1)

use_scale(scale="JI")

; Start by morphing between a triad in 11-EDO, 12-EDO, 13-EDO, and pure JI
[p1.1] 3:A:~ 3:A:&
[p1.2] 1:E!11:& E!12:& E!13:& 3:E:&
[p1.3] 1:C!11:& C!12:& C!13:& 3:C:&

; Glide to octaves
[p1.1] 3:A,2:~
[p1.2] 3:A:~
[p1.3] 3:A'2:~

; Glide to a chord
[p1.1] 9:A,2
[p1.2] 3:A:&   2:Bh:~  1:Bh:&  3:I'
[p1.4] 3:A:&   6:E
[p1.5] 3:A:&   6:C,
[p1.3] 3:A'2:& 2:Cl':~ 1:Cl':& 3:C'
```
<!-- generate-end -->

{{ audio(src="tie-glide2-csound.mp3", caption="Glide Demonstration") }}
