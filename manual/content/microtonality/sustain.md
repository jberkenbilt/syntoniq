+++
title = "Tie and Glide"
weight = 70
sort_by = "weight"
+++

Syntoniq supports two ways to sustain a note: tie (`~`) and glide (`&`). In both cases, the note's duration is extended over subsequent holds or subsequent sustained notes. A sustained note ends at the end of the next non-sustained note or when an accented note is encountered. A glide automatically sustains across a hold, but it must be accompanied by a tie to avoid re-articulating the note. By combining tie and glide, you can create long strings of continuous pitch changes.

Here's an example, explained below.

<!-- generate include=tie-glide1.stq checksum=21d05aa539a14da321b70913fc5b9af7bbc434596e299a3032444ab3d9eb627a -->
```syntoniq
syntoniq(version=1)

use_scale(scale="JI")

[p1.1] 2:E:&
[p1.2] 2:A,:~

[p1.1] 2:~  ; glide continues across this line
[p1.2] 2:~  ; tie continues across this line

[p1.1] 2:C          ; note is re-articulated because the glide is not tied
[p1.2] 1:A,:~ A,:^  ; note is re-articulated because of the accent

```
<!-- generate-end -->

{{ audio(src="tie-glide1-csound.mp3", caption="Basic Tie and Glide") }}

In the example above, you can see that `[p1.1]` has `E` gradually changing pitch to `C` (these being equivalent to `e` and `g` in 12-EDO) while `[p1.2]` stays on the same pitch. In both cases, the sustain activity carries across the line with the holds. In the last block, we see that the `C` (corresponding to the 12-EDO note `g`) is re-articulated because the original glide was not tied. `A` (corresponding to the 12-EDO note `c`) is re-articulated because of the explicit accent mark.

In the next example, we mix ties and glides in various ways to morph around between chords. All of the glides are tied in this example for completely continuous pitch in each voice.

<!-- generate include=tie-glide2.stq checksum=ca9283eadcd012fd44fa823c300bc3c6d99c4a77140b615f9b480cb6316b0649 -->
```syntoniq
syntoniq(version=1)

use_scale(scale="JI")

; Start by morphing between a triad in 11-EDO, 12-EDO, 13-EDO, and pure JI
[p1.1] 3:A:~ 3:A:&~
[p1.2] 1:E!11:&~ E!12:&~ E!13:&~ 3:E:&~
[p1.3] 1:C!11:&~ C!12:&~ C!13:&~ 3:C:&~

; Glide to octaves
[p1.1] 3:A,2:~
[p1.2] 3:A:~
[p1.3] 3:A'2:~

; Glide to a chord
[p1.1] 9:A,2
[p1.2] 3:A:&~   2:Bh:~  1:Bh:&~  3:I'
[p1.4] 3:A:&~   6:E
[p1.5] 3:A:&~   6:C,
[p1.3] 3:A'2:&~ 2:Cl':~ 1:Cl':&~ 3:C'
```
<!-- generate-end -->

{{ audio(src="tie-glide2-csound.mp3", caption="Glide Demonstration") }}
