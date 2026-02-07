+++
title = "Pitch Calculator"
weight = 30
sort_by = "weight"
+++

<!-- The `syntoniq --help` text references the absolute URL to this page. -->

The Syntoniq pitch calculator provides some useful information about scales and pitches. This section is very mathematical. If you are not interested in the math behind scales and pitches, feel free to skip.

## Pitch

The `pitch` subcommand computes the product of its arguments as a pitch. The arguments may be either pitches in Syntoniq pitch notation or note names in the "JI" generated scale. For details on both, see [Pitch and Note Primer](../../microtonality/pitch-primer/). This command shows the canonical pitch of each value separately, and then it shows the final product in various ways. If the final pitch is in the range of MIDI notes, information about the MIDI note number and pitch bend is given. Otherwise, the number of octaves and cents are given. This way, you get the most useful information based on whether you're looking at a relative note in a scale or at a final pitch. Here are several examples.

See how the generated note `E` is interpreted:
<!-- generate calc=pitch,E -->
```
syntoniq calc pitch E
---
E            5/4
final pitch  5/4
frequency    1.250
octaves      0.322
cents        386.314¢
```
<!-- generate-end -->

See how the generated note `E` is interpreted in 41-EDO:
<!-- generate calc=pitch,E!41 -->
```
syntoniq calc pitch E!41
---
E!41         ^13|41
final pitch  ^13|41
frequency    1.246
octaves      0.317
cents        380.488¢
```
<!-- generate-end -->

Find the pitch of the 5th step of 19-EDO relative to middle C as defined to be 9 steps below A 440 in 12-EDO:
<!-- generate calc=pitch,440*^-9|12,^5|19 -->
```
syntoniq calc pitch 440*^-9|12 ^5|19
---
440*^-9|12   220*^1|4
^5|19        ^5|19
final pitch  220*^39|76
frequency    313.978
MIDI note    63.158
MPE (hex)    3f, 201b
```
<!-- generate-end -->

Find the value of the generated note `jI` (the syntonic comma!)
<!-- generate calc=pitch,jI -->
```
syntoniq calc pitch jI
---
jI           81/80
final pitch  81/80
frequency    1.012
octaves      0.018
cents        21.506¢
```
<!-- generate-end -->

Follow a chain of transpositions:
<!-- generate calc=pitch,E,A2!17,C,p!31 -->
```
syntoniq calc pitch E A2!17 C p!31
---
E            5/4
A2!17        ^2|17
C            3/2
p!31         1/2*^28|31
final pitch  15/8*^11|527
frequency    1.902
octaves      0.928
cents        1113.316¢
```
<!-- generate-end -->

## Nearest Scale Degree or Ratio

The `near` subcommand can help you find ratios or scale degrees in an equally divided scale close to a given pitch. This command is complex and powerful, and there are some subtle aspects to its behavior.

Arguments:
* `--pitch` — the pitch in Syntoniq pitch notation
* `--interval` — the interval to divide when showing pitches. If specified, the pitches presented will be divisions of this interval. If not specified, the default value depends on the pitch. If the pitch is a ratio, it defaults to 2 (an octave). Otherwise, the output contains ratios. See examples for clarification.
* `--max-denom` — the maximum denominator to use in output. For rational output, this represents how far to go in the harmonic series. For divisions of an interval, this represents the maximum number of divisions to consider. If not specified, the software picks what it considers to be a sensible default.
* `--tolerance` — specified as a Syntoniq pitch; indicates how close a match must be to be shown. The default value is `^1|75`, which is 16¢. This is approximately the amount by which a 12-tone minor third differs from the ratio 6/5.

The output consists of rows sorted by how close they are to the desired pitch. The columns are
* pitch — the pitch in Syntoniq pitch notation
* value — the approximate value as a floating point number with up to three decimal places
* Δ cents — The distance in cents between this pitch and the desired pitch. A positive number indicates that the given pitch is above the desired pitch.

Show equal divisions of the octave near the ratio 5/4, a major third. The output is shown as equal octave divisions because `--interval` was not specified and the pitch is rational. Here are things to notice:
* Output is in order of closeness. The first two rows show some matches that are less than 1¢ away from 5/4, including `^10|31`. 31-EDO is known to have a very good major third. You can also see that 53-EDO has a very good major third.
* For simplicity of search, pitches are shown in simplest/canonical form but also as a number of divisions of higher ratios. This makes it easier to search. The ratio 5/4 is the degree 4 in 12-EDO (numbering from zero). You can see `^1|3` as well as `^4|12` (and many others) in the output. The ones that are not in simplest form are shown as equal to the simplest version.
* As a validation of Δ cents, you can see that the major third in 12-EDO is 13.686¢ sharp.
<!-- generate calc=near,--pitch,5/4 -->
```
syntoniq calc near --pitch 5/4
---
== 5/4 ≈ 1.250 ==
pitch    value        Δ cents
 ^9|28   1.250    -0.599¢
^10|31   1.251     0.783¢
^17|53   1.249    -1.408¢
^11|34   1.251     1.922¢
 ^8|25   1.248    -2.314¢
^16|50   1.248    -2.314¢ (= ^8|25)
^12|37   1.252     2.875¢
^15|47   1.248    -3.335¢
^13|40   1.253     3.686¢
^14|43   1.253     4.384¢
 ^7|22   1.247    -4.496¢
^14|44   1.247    -4.496¢ (= ^7|22)
^15|46   1.254     4.991¢
^16|49   1.254     5.523¢
^13|41   1.246    -5.826¢
^17|52   1.254     5.994¢
 ^6|19   1.245    -7.366¢
^12|38   1.245    -7.366¢ (= ^6|19)
^11|35   1.243    -9.171¢
^16|51   1.243    -9.843¢
 ^5|16   1.242   -11.314¢
^10|32   1.242   -11.314¢ (= ^5|16)
^15|48   1.242   -11.314¢ (= ^5|16)
^14|45   1.241   -12.980¢
 ^1|3    1.260    13.686¢
 ^2|6    1.260    13.686¢ (= ^1|3)
 ^3|9    1.260    13.686¢ (= ^1|3)
 ^4|12   1.260    13.686¢ (= ^1|3)
 ^5|15   1.260    13.686¢ (= ^1|3)
 ^6|18   1.260    13.686¢ (= ^1|3)
 ^7|21   1.260    13.686¢ (= ^1|3)
 ^8|24   1.260    13.686¢ (= ^1|3)
 ^9|27   1.260    13.686¢ (= ^1|3)
^10|30   1.260    13.686¢ (= ^1|3)
^11|33   1.260    13.686¢ (= ^1|3)
^12|36   1.260    13.686¢ (= ^1|3)
^13|39   1.260    13.686¢ (= ^1|3)
^14|42   1.260    13.686¢ (= ^1|3)
^15|45   1.260    13.686¢ (= ^1|3)
^16|48   1.260    13.686¢ (= ^1|3)
^17|51   1.260    13.686¢ (= ^1|3)
 ^9|29   1.240   -13.900¢
```
<!-- generate-end -->

This output shows some approximations of 5/4 in equal divisions of the tritave (ratio 3). Here, we cap the maximum number of divisions to 27 and specify a tighter tolerance of 12¢ (1/100th of an octave). Notice `3^4|19`: One 1/19 of a tritave is very close to 1/12 of an octave, and the major third can be found at the degree 4 in both cases.
<!-- generate calc=near,--pitch,5/4,--interval,3,--max-denom,27,--tolerance,^1|100 -->
```
syntoniq calc near --pitch 5/4 --interval 3 --max-denom 27 --tolerance ^1|100
---
== 5/4 ≈ 1.250 ==
pitch    value        Δ cents
3^1|5    1.246   -3.737¢
3^2|10   1.246   -3.737¢ (= 3^1|5)
3^3|15   1.246   -3.737¢ (= 3^1|5)
3^4|20   1.246   -3.737¢ (= 3^1|5)
3^5|25   1.246   -3.737¢ (= 3^1|5)
3^5|24   1.257    6.263¢
3^4|19   1.260    8.895¢
```
<!-- generate-end -->

This example shows some ratios close to degree 7 in 17-EDO. That scale degree is very close to a perfect fourth, ratio 4/3. We see ratios in the output because no interval was given and the pitch is not rational.
<!-- generate calc=near,--pitch,^7|17 -->
```
syntoniq calc near --pitch ^7|17
---
== ^7|17 ≈ 1.330 ==
pitch   value   Δ cents
 4/3    1.333     3.927¢
41/31   1.323   -10.091¢
37/28   1.321   -11.600¢
33/25   1.320   -13.472¢
29/22   1.318   -15.858¢
```
<!-- generate-end -->

This example combines `--interval` with a non-rational pitch. Here, we are finding pitches close to the third degree in 12-EDO (approximately a minor third) in divisions of the tritave. Notice that `3^2|13` is less than 5¢ below the 12-EDO minor third. The scale made up of 13 equal divisions of the tritave is the Bohlen-Pierce scale.
<!-- generate calc=near,--pitch,^3|12,--interval,3,--max-denom,27 -->
```
syntoniq calc near --pitch ^3|12 --interval 3 --max-denom 27
---
== ^3|12 ≈ 1.189 ==
pitch    value         Δ cents
3^3|19   1.189     0.195¢
3^4|25   1.192     2.721¢
3^2|13   1.184    -4.664¢
3^4|26   1.184    -4.664¢ (= 3^2|13)
3^3|20   1.179    -9.279¢
3^1|6    1.201    10.721¢
3^2|12   1.201    10.721¢ (= 3^1|6)
3^3|18   1.201    10.721¢ (= 3^1|6)
3^4|24   1.201    10.721¢ (= 3^1|6)
3^4|27   1.177   -11.501¢
```
<!-- generate-end -->

## Equal Scales

The `equal-scale` subcommand shows you information about scales made up of equal divisions of an interval. The output columns are as follows. The examples should provide additional clarification.

* `pitch` — the scale degree shown with divisions as the exponent denominator even if this is not the simplest form
* `simplified` — the canonical representation of the scale degree's relative pitch in Syntoniq pitch notation
* `value` — the scale degree as a floating point number to three decimal places
* `cents` —  the scale degree shown in cents
* `note` — a note that would give this pitch in a generated scale with this many divisions
* `Δ scale degree` — how far off the generated note's pure ratio is from the desired pitch in scale degrees. This is shown as the note ± a fraction of scale degrees
* `Δ cents` — how far off the generated note's pure ratio is from the desired pitch in cents

This table shows 12-EDO. Observe the following:
* Consider the row that starts with `2^4|12`.
  * `2^4|12` means the 4th degree of a scale that divides ratio 2 (the octave) into 12 equal pieces
  * `^1|3` is the simplified version of the pitch. We usually omit 2 when it's the base, and 4/12 simplifies to 1/3.
  * The value of 1.260 is slightly above 1.250, which is 5/4. This is expected since the major third is sharp in 12-EDO.
  * We can see that this note is exactly 400.000¢. Cents are defined based on 12-EDO, so we expect a round number here.
  * The note generated note `E` represents the ratio 5/4. That is the closest "simple" ratio to this scale degree.
  * The value `E! + 0.137°` means that this scale degree is 0.137 scale degrees higher than `E!`. The `!` "coerces" the pitch into a pure ratio. Since one scale degree is a half step in 12-EDO, this is telling us that this scale degree is 0.137 half steps sharper than the pure generated note `E`. This information can be useful if you are using generated notes to create intervals that are "portable" across different equally divided scales.
  * The value `E! + 13.686¢` shows us the same information in a different way: this scale degree is 13.686¢ sharper than a perfect major third. Because this is 12-EDO and scale degrees are 100¢, we see this is about 100× the value in scale degrees.
<!-- generate calc=equal-scale,--divisions,12 -->
```
syntoniq calc equal-scale --divisions 12
---
 pitch    simplified   value     cents      note   Δ scale degree      Δ cents
 2^0|12       1        1.000      0.000¢     A      A! + 0.000°     A! +  0.000¢
 2^1|12      ^1|12     1.059    100.000¢     R      R! + 0.010°     R! +  1.045¢
 2^2|12      ^1|6      1.122    200.000¢     I      I! - 0.039°     I! -  3.910¢
 2^3|12      ^1|4      1.189    300.000¢     F      F! - 0.156°     F! - 15.641¢
 2^4|12      ^1|3      1.260    400.000¢     E      E! + 0.137°     E! + 13.686¢
 2^5|12      ^5|12     1.335    500.000¢     D      D! + 0.020°     D! +  1.955¢
 2^6|12      ^1|2      1.414    600.000¢    Cq     Cq! + 0.030°    Cq! +  3.000¢
 2^7|12      ^7|12     1.498    700.000¢     C      C! - 0.020°     C! -  1.955¢
 2^8|12      ^2|3      1.587    800.000¢    Be     Be! - 0.137°    Be! - 13.686¢
 2^9|12      ^3|4      1.682    900.000¢    Bf     Bf! + 0.156°    Bf! + 15.641¢
2^10|12      ^5|6      1.782   1000.000¢    Bi     Bi! + 0.039°    Bi! +  3.910¢
2^11|12     ^11|12     1.888   1100.000¢    Br     Br! - 0.010°    Br! -  1.045¢
2^12|12       2        2.000   1200.000¢    A'     A'! + 0.000°    A'! +  0.000¢
```
<!-- generate-end -->

Let's take a look at 19-EDO. If you're used to looking at scales defined in terms of cents (as in Scala files), the value 63.158¢ will likely look familiar to you: it is the step size in cents of a 19-EDO scale. You can see it in a lot of other ways as well. Here are some things to notice:
* The numbers don't match as obviously between the Δ scale degree and Δ cents columns. Cents are very useful for getting a sense of perceptual closeness, but fractions of a scale degree give you a more precise way of knowing "how good" a ratio is, especially if you need to decide whether to use a `#` or `%` accidental to force the pitch to go one way or another.
* Scanning down the Δ scale degree and Δ cents columns, you can see which ratios are closer to scale degrees. The row starting with `2^5|19` shows that `F` is less than 0.2¢ away from the scale degree. Since `F` is 6/5, which is a minor third, this confirms that 19-EDO has an extremely precise minor third.
* The table shows `G%` for `2^4|19`. The `%` here is unnecessary since `G` is closer to `^4|19` than to `^5|19`, but Syntoniq suggests a `%` or `#` if the note is more than 0.2 scale degrees away. This just gives you an extra visual indicator that the ratio is a slightly worse approximation of the pitch.

<!-- generate calc=equal-scale,--divisions,19 -->
```
syntoniq calc equal-scale --divisions 19
---
 pitch    simplified   value     cents      note   Δ scale degree      Δ cents
 2^0|19       1        1.000      0.000¢     A      A! + 0.000°      A! +  0.000¢
 2^1|19      ^1|19     1.037     63.158¢     Y      Y! - 0.119°      Y! -  7.515¢
 2^2|19      ^2|19     1.076    126.316¢     N      N! - 0.031°      N! -  1.982¢
 2^3|19      ^3|19     1.116    189.474¢     J      J! + 0.112°      J! +  7.070¢
 2^4|19      ^4|19     1.157    252.632¢    G%      G! - 0.225°      G! - 14.239¢
 2^5|19      ^5|19     1.200    315.789¢     F      F! + 0.002°      F! +  0.148¢
 2^6|19      ^6|19     1.245    378.947¢     E      E! - 0.117°      E! -  7.366¢
 2^7|19      ^7|19     1.291    442.105¢    FN     FN! - 0.029°     FN! -  1.834¢
 2^8|19      ^8|19     1.339    505.263¢     D      D! + 0.114°      D! +  7.218¢
 2^9|19      ^9|19     1.389    568.421¢    DY     DY! - 0.005°     DY! -  0.296¢
2^10|19     ^10|19     1.440    631.579¢    Cy     Cy! + 0.005°     Cy! +  0.296¢
2^11|19     ^11|19     1.494    694.737¢     C      C! - 0.114°      C! -  7.218¢
2^12|19     ^12|19     1.549    757.895¢   Bfn    Bfn! + 0.029°    Bfn! +  1.834¢
2^13|19     ^13|19     1.607    821.053¢    Be     Be! + 0.117°     Be! +  7.366¢
2^14|19     ^14|19     1.667    884.211¢    Bf     Bf! - 0.002°     Bf! -  0.148¢
2^15|19     ^15|19     1.728    947.368¢   Bg#     Bg! + 0.225°     Bg! + 14.239¢
2^16|19     ^16|19     1.793   1010.526¢    Bj     Bj! - 0.112°     Bj! -  7.070¢
2^17|19     ^17|19     1.859   1073.684¢    Bn     Bn! + 0.031°     Bn! +  1.982¢
2^18|19     ^18|19     1.928   1136.842¢    By     By! + 0.119°     By! +  7.515¢
2^19|19       2        2.000   1200.000¢    A'     A'! + 0.000°     A'! +  0.000¢
```
<!-- generate-end -->

If you see a generated note in the table and you're not sure what it means, you can always use the `pitch` subcommand. For example, `2^12|19` is `Bfn`. What's `Bfn`?
<!-- generate calc=pitch,Bfn -->
```
syntoniq calc pitch Bfn
---
Bfn          65/42
final pitch  65/42
frequency    1.548
octaves      0.630
cents        756.060¢
```
<!-- generate-end -->

Let's take a fresh look at Bohlen-Pierce. Since this divides the tritave, we specify `--iterval 3`. Notice that the top note, `A'`, is 1901.955¢. This is slightly above 1900¢ because the ratio 3 is just slightly sharp relative to an octave and a fifth in 12-EDO. Also notice that, as we climb past the octave, we start seeing notes that start with `B`. Since `B` is the ratio of 2/1, a `B` adds an octave to the pitch. This is an absolute ratio; it always means an octave regardless of the divided interval. In this case, `A'` is a tritave above `A` because the divided interval is 3.
<!-- generate calc=equal-scale,--divisions,13,--interval,3 -->
```
syntoniq calc equal-scale --divisions 13 --interval 3
---
 pitch    simplified   value     cents      note   Δ scale degree      Δ cents
 3^0|13       1        1.000      0.000¢     A      A! + 0.000°     A! +  0.000¢
 3^1|13     3^1|13     1.088    146.304¢     L      L! - 0.030°     L! -  4.333¢
 3^2|13     3^2|13     1.184    292.608¢     F      F! - 0.157°     F! - 23.033¢
 3^3|13     3^3|13     1.289    438.913¢    E#      E! + 0.360°     E! + 52.599¢
 3^4|13     3^4|13     1.402    585.217¢    DT     DT! - 0.011°    DT! -  1.629¢
 3^5|13     3^5|13     1.526    731.521¢    C#      C! + 0.202°     C! + 29.566¢
 3^6|13     3^6|13     1.660    877.825¢    Bf     Bf! - 0.045°    Bf! -  6.533¢
 3^7|13     3^7|13     1.807   1024.130¢    Bj     Bj! + 0.045°    Bj! +  6.533¢
 3^8|13     3^8|13     1.966   1170.434¢    B%      B! - 0.202°     B! - 29.566¢
 3^9|13     3^9|13     2.140   1316.738¢    BO     BO! - 0.018°    BO! -  2.705¢
3^10|13    3^10|13     2.328   1463.042¢    BG     BG! - 0.026°    BG! -  3.829¢
3^11|13    3^11|13     2.533   1609.347¢    BE     BE! + 0.157°    BE! + 23.033¢
3^12|13    3^12|13     2.757   1755.651¢   BD#     BD! + 0.394°    BD! + 57.606¢
3^13|13       3        3.000   1901.955¢    A'     A'! + 0.000°    A'! +  0.000¢
```
<!-- generate-end -->
