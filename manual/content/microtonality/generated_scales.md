+++
title = "Using Generated Scales"
weight = 40
sort_by = "weight"
+++

In the [Prerequisite Knowledge](../knowledge/) section, we learned about generated note names. We will make use of generated notes in this section, so please review that material if you are not comfortable with it or find things confusing here.

Syntoniq includes a built-in scale called "JI" that uses generated note notation for pure just intonation. In this section, we'll work our way into using generated notation and see what you can do with it.

Let's start with this little chord sequence. Below, you will see it defined twice: once with the default 12-EDO scale using "normal" notes, then with a *generated* 12-EDO scale using generated notes. Listening to the audio, you can hear that the same chord progression is played twice. We have the following sequence:
* D Dominant 7th chord, sustaining the fifth
* G Dominant 7th chord in second inversion after resolving the sustained note
* C Major 7th chord
* C Major 13(#11), stacking a D major triad on top of the major 7th chord.

<!-- generate include=ji-sample-12-edo.stq checksum=822dfc182aa9a11b13665a75dc0fc43164831b7334afaefcb5d09b0f499e6ffd -->
```syntoniq
syntoniq(version=1)
tempo(bpm=60)
[p1.1] 2:d    d     4:c
[p1.2] 2:f#   f     4:e
[p1.3] 3:a      1:g 4:g
[p1.4] 2:c'   b     4:b
[p1.5] 2:~    ~     ~    d'
[p1.6] 2:~    ~     ~    f#'
[p1.7] 2:~    ~     ~    a'

define_generated_scale(scale="gen-12" divisions=12)
use_scale(scale="gen-12")
[p1.1] 2:I    I       4:A
[p1.2] 2:IE   IF      4:E
[p1.3] 3:IC       1:C 4:C
[p1.4] 2:Ih'  If'     4:p'
[p1.5] 2:~    ~       ~    I'
[p1.6] 2:~    ~       ~    Cl'
[p1.7] 2:~    ~       ~    IC'
```
<!-- generate-end -->

{{ audio(src="ji-sample-12-edo-csound.mp3", caption="12-EDO generated scale") }}

Now, here's the same exact notation, this time using the built-in pure just intonation generated scale. The notes below are identical to the second score block above. Listen to how it sounds with just intonation. After the audio, I'll dive into some explanation so you can see what's happening.

<!-- generate include=ji-sample.stq checksum=e4cc1abd8b61d99288335f5ab103f25d3995938872912c0bfb23b338b00d8bb4 -->
```syntoniq
syntoniq(version=1)
tempo(bpm=60)
use_scale(scale="JI")
[p1.1] 2:I    I       4:A
[p1.2] 2:IE   IF      4:E
[p1.3] 3:IC       1:C 4:C
[p1.4] 2:Ih'  If'     4:p'
[p1.5] 2:~    ~       ~    I'
[p1.6] 2:~    ~       ~    Cl'
[p1.7] 2:~    ~       ~    IC'
```
<!-- generate-end -->

{{ audio(src="ji-sample-csound.mp3", caption="Same passage with just intonation") }}

Trying to represent a sequence of chords like that in just intonation is not as simple as you might think. There's no "cheat sheet" to map between a 12-EDO note and a pure interval ratio. Take the `a` from the first note of note 3 (the `[p1.3]` line). Which `a` is this? With the conventional 12-EDO notation, it's just `a`...but that's not good enough for JI (just intonation). The `c'` at the beginning of `[p1.4]` is also interesting. It is the root of the key, but in this case, it's functioning as minor seventh above the root of the chord, which is `d`. Let's take a look at the choices I made for the first chord, which I spelled as `I`, `IE`, `IC`, and `Ih'`.
* `I` — this represents the ratio 9/8
* `IE` — without calculating the ratio, we can recognize `E` as 5/4 and see that the note `IE` is a major third above `I`. We know this interval is a perfect major third. The ratio itself is $\frac{9}{8}\times\frac{5}{4} = \frac{45}{32}$, but you can see that spelling it `IE` conveyed (and implemented!) the intent of creating a major third here without actually having to calculate the ratio.
* `IC` — without doing any calculations, you can see that this is 3/2 (a perfect fifth, denoted by `C`) above `I`. You can look at `I`, `IE`, `IC`, and with a little practice, see this immediately as a pure JI major triad.
* `Ih'` — this one is a little more interesting. `h` represents the ratio 7/8. Before I go further, let's take a quick detour and talk about limits.

*You can skip this paragraph if you want.* If you don't understand this, don't worry; understanding it is not essential. If you are used to working with JI, you might find it interesting. People talk about 5-limit, 7-limit, 11-limit, etc. when they talk about JI. The number here is the largest prime factor of any number that appears in the numerator or the denominator of the fraction. Our Western 12-EDO scale has the property that all the scale degrees are close approximations of a note in 5-limit JI. But what does this really mean? A smaller limit doesn't mean a "simpler" note. Take $\frac{3}{2}^12 = \frac{531441}{4096}$. We saw this earlier as what you get when you wrap around the octave using perfect fifths. That ratio is 3-limit JI, but most people would not call it simple, and the *Pythagorean Comma* that results is not a consonant interval! If you want to confine yourself to 5-limit JI with Syntoniq generated scales, you can do so pretty easily...just stick to letters that are ratios of numbers that have only 2, 3, 5 as factors. That means you can use `I` = 9/8 since $9 = 3^2$ and $8 = 2^3$, but you can't use `H`, which is 8/7, and 7 is a prime greater than 5! (You can use B, C, D, E, F, I, J, P, Y, and their lowercase equivalents.) But the 7th harmonic is a pretty low harmonic, and constructing a seventh chord with it can be quite effective and sound very consonant in the right context. People sometimes refer to the 7/6 ratio as the septimal minor third and the 7/4 ratio as the alp horn Fa. Brass instruments can play that pitch fairly easily. The high "F" fingered with the first and third valves is really 7/4 above "C" (using instrument pitches -- the trumpet is pitched in B♭...but let's not get too far off track here!) One alternative to thinking about prime limits is to think about the simplicity of intervals....but you don't have to think about those things at all if you don't want. You can just listen and see how it sounds! Now, let's get back to the main topic.

Returning to our chord, I have chosen `Ih'` for the seventh. This is a little flat compared to the 7th in 12-EDO, but if you listen to that chord with pure JI, there's more of a buzz than a beat because the harmonics of the notes in the chord line up so well. This creates a slightly different musical color that is not accessible in 12-EDO. Yet when we super-impose these notes onto a 12-EDO scale using this system, Syntoniq picks the closest scale degree. What do we have? Recall that the `'` multiples the frequency by the cycle ratio. Since we didn't specify a cycle ratio, we have the default ratio of 2, which represents the octave. Therefore, `Ih'`, relative to the scale base pitch, is $2\times\frac{9}{8}\times\frac{7}{8} = \frac{63}{32} \approx 2$. That note is very close to the octave, hence its having the same pitch as plain old `c'` in the 12-EDO notation.

Does this seem complex? Well, it is complex, and I don't think I can do anything about that...but it is probably *less complex* than rolling all the ratios by hand. You can play around with different intervals and what they sound like, and then you can build chords, note by note, that contain the exact intervals you want. Let's look at the rest of the sequence. The second chord, after resolution of the sustained note is
* `I` — we saw this before
* `IF` — a minor third (F = 6/5) above the root
* `C` — this is the 3/2 fifth in the scale, which helps us pivot back to the root
* `If'` — a minor third *below* the `I` pitch in the higher octave. This gives us a 5/3 major sixth above the bottom note in our chord. That ratio happens to be $\frac{9}{8}\times\frac{5}{6} = \frac{15}{16}$, which means we could have written it as `p'`, but in this case, it was easier to express the intention of being a minor third below the base of the chord.

The next chord:
* `A` — the root
* `E` — the 5/4 major third
* `C` — the 3/2 perfect fifth
* `p'` — the 15/8 major seventh. It's 15/8 because the `'` doubled the pitch (raising it by an octave). This is the same actual note as our previous `If'`.

Finally, we add the 9th, sharp 11th, and 13th with
* `I'` — 9/4
* `Cl'` — this is 11/12 below the fifth...that puts us at 11/8. This would be 11-limit JI if we were concerned about limits, which we're not. 11/8 is also a fairly low ratio and also reachable on a trumpet if you've got the chops. It's definitely reachable on a French horn. In fact, in his *Serenade for Tenor, Horn and Strings*, Benjamin Britten uses both the 7/4 and 11/8 harmonics in the prologue and epilogue movements.
* `IC'` — keeping things simple, this just adds the fifth as a perfect fifth above the base of the new stacked triad. There were other choices...a fun choice would have been `CM'`, the 13th harmonic above the fifth...but it doesn't have quite the sound I was going for. The effect of two stacked triads with the twist of using the 11th harmonic in place of the major third provided just the right color!

Now, let's try something else. Pure JI can sound a bit dry and buzzy sometimes. It's great in certain spots, but for a whole piece, many (and I count myself here) prefer the slight amount of "out-of-tune-ness" that comes from approximating JI in an EDO scale, plus that brings a whole new set of harmonic devices. So far, I've focused on "normal" chords from 12-tone music theory (plus 7th and 11th harmonics!), but when you start playing with EDOs other than 12, whole new musical landscapes open up.

The 12-EDO scale works very well for simple 5-limit JI intervals, but it can't really approximate 7/4 or 11/8. It turns out that some higher EDOs, particularly 41-EDO, have great approximations for those intervals. What if we wanted to play our chords in 41-EDO? Well, if you wanted to manually define 41-EDO or work with it in those systems that actually support it, you're going to have to decide what to call the notes. You could just call them `n0` through `n40` and do lots of mod 41 arithmetic and memorize how many steps are in each interval...or you could define a generated scale with 41 divisions using Syntoniq. Here's what that looks like. Again, the notes are exactly the same. The only thing we changed was the scale definition.

<!-- generate include=ji-sample-41-edo.stq checksum=e2eb4a2a1fd968ee797380835b64c4e88bc1424638d99d2469424d62244a5480 -->
```syntoniq
syntoniq(version=1)
tempo(bpm=60)
define_generated_scale(scale="gen-41" divisions=41)
use_scale(scale="gen-41")
[p1.1] 2:I    I       4:A
[p1.2] 2:IE   IF      4:E
[p1.3] 3:IC       1:C 4:C
[p1.4] 2:Ih'  If'     4:p'
[p1.5] 2:~    ~       ~    I'
[p1.6] 2:~    ~       ~    Cl'
[p1.7] 2:~    ~       ~    IC'
```
<!-- generate-end -->

{{ audio(src="ji-sample-41-edo-csound.mp3", caption="Same passage in 41-EDO") }}

I don't know about you, but I like this better than either the 12-EDO or the JI version. But personal taste aside, the highlight here is that it was *very easy* to try this experiment. We didn't have to calculate any ratios, and we didn't have to decide what to call any of the notes. We just had to understand how our chords were built based on nature's building blocks: steps in the harmonic series. Here are a few highlights about the 41-EDO version:
* While a whole step in 41-EDO is about 7 steps, the 5/4 ratio is better approximated with 13 steps than 14 steps. You don't have to know or care about that detail. The note `E` lands on the 13th step (numbering from 0). The note `IE` lands on the 20th step (from 0). It works automatically. If you wanted to specify the 13th step explicitly, you could always use the note `A13`.
* The 7th and 11th harmonics both have close approximations in 41-EDO. The 33rd step of 41-EDO is less than 3¢ below 7/4, and the 19th step is less than 5¢ above 11/8.
* These are not *perfect* intervals. They are slightly out of tune, though they are closer than the 12-EDO scale we're used to. This causes a natural "beat" as the harmonics fail to line up. For most people, this makes the music sound more alive. It gives it a bit of a shimmer. I think this is a benefit, not a detriment...but if you want pure intervals, they're there for you to use.

In the next section, I'll introduce Syntoniq's transposition system and present a few more microtonal passages.
