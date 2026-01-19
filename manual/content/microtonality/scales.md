+++
title = "Defining Scales"
weight = 30
sort_by = "weight"
+++

Syntoniq comes with a few built-in scales and allows you to define your own scales. It is not a goal of Syntoniq to support a wide range of built-in scales or to load scales from Scala files. Instead, you can define your own scales that map from a relative Syntoniq pitch to one or more note names. This is done using the `define_scale` directive. For authoritative documentation on Syntoniq directives, see [Language Reference](../../reference/language-reference/) or run `syntoniq doc`. The `define_scale` directive has one required parameter, `scale`, indicating the scale name, and an optional `cycle_ratio` parameter, indicating what the "octave" markers do. The default for `cycle_ratio` is 2, indicating that `'` and `,` change pitches by an octave, but you can set this to a different value for tritave-based scales or scales that cycle differently from an octave. The `define_scale` directive must be followed by a data block, which is delimited by `<<` and `>>`. The data block contains pitches mapped to one or more note names. Each note is defined as a pitch followed by one or more note names.

As a reminder, the pitch `^1|5` would represent exactly a fifth of an octave because this notation means $2^{\frac{1}{5}}$. Using `define_scale`, you could define a 5-EDO scale with the notes `p`, `q`, `r`, `s`, and `t` like this example, which defines the scale and then plays one full cycle of the scale, repeating the first note an octave up.

<!-- generate include=5-edo-pqrst.stq checksum=d36b2afd757139f7ad938c4693a0b4fdc054c93dfdc092d6edf5e91fa9ad22f5 -->
```syntoniq
syntoniq(version=1)
define_scale(scale="5-EDO") <<
^0|5 p
^1|5 q
^2|5 r
^3|5 s
^4|5 t
>>
use_scale(scale="5-EDO")

[p1.0] 1:p q r s t p'
```
<!-- generate-end -->

{{ audio(src="5-edo-pqrst-csound.mp3", caption="5-EDO scale") }}

Here's an example of a scale with a cycle size other than an octave: the [Bohlen-Pierce Scale](https://en.wikipedia.org/wiki/Bohlen%E2%80%93Pierce_scale). This scale divides the *tritave*, a ratio of 3/1, into 13 equal steps. This example defines the scale using the notes `j` through `v` for the steps. We call this "13-ED3", indicating 13 equal divisions of the ratio 3/1 (or just 3). This example shows how you can use a cycle size of other than an octave. We play a series of chords, followed by a pause, followed by the notes `j` and `j'`, so you can hear that there is an octave and a fifth, not an octave because of the cycle mark. Here are some things to notice:
* We put more than one note definition per line. You can do this to save space or to organize pitches.
* We defined more than one note name for each note. In this case, in addition to `j` through `v`, we defined `xn` where `n` is the step number.
* We had to give `3^n|13` as the pitch for each $n$. The cycle ratio doesn't change the meaning of any individual pitch. It only changes what the cycle marks (`'` and `,`) do to the pitches.
* The second line uses the `xn` versions of the note names. This is just to show that you can do it. Those notes could have been written starting with `1:m n o p q r s t` and would have sounded the same.

<!-- generate include=13-ed3.stq checksum=bcf8771a20812cac2c3ce520e1a4c74ee37d45d55f83e38f55847d1bf46caf00 -->
```syntoniq
syntoniq(version=1)
define_scale(scale="13-ED3" cycle_ratio=3) <<
 3^0|13 j x0    3^1|13 k x1    3^2|13 l x2
 3^3|13 m x3    3^4|13 n x4    3^5|13 o x5
 3^6|13 p x6    3^7|13 q x7    3^8|13 r x8
 3^9|13 s x9   3^10|13 t x10  3^11|13 u x11
3^12|13 v x12
>>
use_scale(scale="13-ED3")

[p1.0] 1:j  k  l  m  n  o  p  q   4:~
[p1.1] 1:x3 x4 x5 x6 x7 x8 x9 x10   ~ 3:j
[p1.3] 1:p  q  r  s  t  u  v  j'    ~ 3:j'
```
<!-- generate-end -->

{{ audio(src="13-ed3-csound.mp3", caption="Bohlen-Pierce scale: 13-ED3") }}

# Built-in Scales

Syntoniq comes with a few built-in scales to get you started. You can see them defined below for reference. The real power of Syntoniq comes not from using the built-in EDO scales but from using generated scales. There is a built-in generated scale called "JI", which we saw briefly in a previous section. Note that the *note names* in these scales contain the `#` and `%` symbols. In explicitly defined scales, `#`, `%`, `+`, `-`, and the other characters with special meanings in generated scales are just note characters. They are intended to carry semantic meaning, but the software doesn't treat them differently from anything other character. We could have replaced `c#` with `c*` or `potato`, and it would have meant the same thing. Also, notice that it is a bit painful to manually define high-EDO scales and decide what to call the notes. The next section discusses generated scales.

<!-- generate include=built-in-scales.stq checksum=ba801915c90d45b07687330b5142afa618ff7d975f10bf67e4c585dfc0ee0456 -->
```syntoniq
define_scale(scale="12-EDO") <<
^-1|12 c%
 ^0|12 c       ^1|12 c# d%
 ^2|12 d       ^3|12 e% d#
 ^4|12 e f%
 ^5|12 f e#    ^6|12 f# g%
 ^7|12 g       ^8|12 a% g#
 ^9|12 a      ^10|12 b% a#
^11|12 b      ^12|12 b#
>>
define_scale(scale="19-EDO") <<
^-1|19 c%
 ^0|19 c     ^1|19 c#      ^2|19 d%
 ^3|19 d     ^4|19 d#      ^5|19 e%
 ^6|19 e     ^7|19 e# f%
 ^8|19 f     ^9|19 f#      ^10|19 g%
^11|19 g    ^12|19 g#      ^13|19 a%
^14|19 a    ^15|19 a#      ^16|19 b%
^17|19 b    ^18|19 b#
>>
define_scale(scale="31-EDO") <<
^-2|31 c%   ^-1|31 c-
 ^0|31 c     ^1|31 c+ d%%    ^2|31 c#      ^3|31 d%   ^4|31 d- c##
 ^5|31 d     ^6|31 d+ e%%    ^7|31 d#      ^8|31 e%   ^9|31 e- d##
^10|31 e    ^11|31 f% e+    ^12|31 e# f-
^13|31 f    ^14|31 f+ g%%   ^15|31 f#      ^16|31 g%   ^17|31 g- f##
^18|31 g    ^19|31 g+ a%%   ^20|31 g#      ^21|31 a%   ^22|31 a- g##
^23|31 a    ^24|31 a+ b%%   ^25|31 a#      ^26|31 b%   ^27|31 b- a##
^28|31 b    ^29|31 b+       ^30|31 b#
>>
define_generated_scale(scale="JI")
```
<!-- generate-end -->
