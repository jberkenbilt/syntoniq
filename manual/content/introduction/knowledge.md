+++
title = "Prerequisite Knowledge"
weight = 30
sort_by = "weight"
+++

You don't need to be a tuning expert to use Syntoniq...but it helps! To go beyond using Syntoniq with the built-in scales, you should understand a little about pure "just intonation" interval ratios and about equal divisions of an octave or other interval. This section will go over the basics, but there is plenty of good information about these topics in other places. Here are a few references:

* [Just intonation](https://en.xen.wiki/w/Just_intonation)
* [Equal-step tuning](https://en.xen.wiki/w/Equal-step_tuning)

In the material that follows, we provide simplified, "notional" explanations. These are not intended to be complete, scholarly, or formal. They are designed for you to check your knowledge. If this all makes sense to you, you have the knowledge required to use Syntoniq to its maximum potential. This section becomes increasingly technical as it proceeds, but you don't need any math more advanced than raising rational numbers (fractions) to rational powers (e.g. $2^\frac{1}{12}$).

# Frequency and Pitch

Sound is created by vibration. When a physical object, such as a metal string or column of air, vibrates back and forth at a fixed frequency within a certain range, we perceive that as a pitch. The frequency of the vibration, measured in Hertz or cycles per second, abbreviated Hz, is the number of times per second that the vibrating object returns to its original position. The human ear perceives this vibration as a recognizable pitch when it falls in the range of about 16 Hz to about 16,000 Hz. (In reality, all sounds except the *pure tone* consist of mixtures of frequencies, but we perceive musical pitches as having a *fundamental frequency*, which is the dominant frequency. Other frequencies combine to produce either multiple pitches or *timbre*, but a discussion of timbre is out of scope here.)

Our perception of frequency as pitch is *logarithmic*. That means that, to increase a pitch by a fixed amount, you have to *multiply* something by the frequency rather than adding it. (If it were adding, we'd say our perception of pitch were linear.) For example, when you double the frequency of a sound, the pitch goes up by one octave. As a concrete example, consider the sound 440 Hz. That is the pitch of the A above middle C on a piano.

```syntoniq
syntoniq(version=1)
[p1.0] 4:a
```

{{ audio(src="introduction/440hz.mp3", caption="440 Hz") }}

If we play 110, 220, 440, and 880 Hz tones, we hear the A in four separate octaves.

```syntoniq
syntoniq(version=1)
[p1.0] 1:a,2 a, a a'
```

{{ audio(src="introduction/110-to-880hz.mp3", caption="110, 220, 440, 880 Hz") }}

# Pure Intervals

Since doubling the frequency of a sound increases the pitch by an octave, we say the octave has a ratio of 2:1, or 2/1 or just 2. Other ratios represent other intervals. The ratio of a perfect fifth is 3:2, or 3/2. We will write ratios as fractions, like 3/2. Here's 440 Hz followed by 660 Hz, where $660 = 440 \times \frac{3}{2}$.

```syntoniq
syntoniq(version=1)
[p1.0] 1:a e'
```

{{ audio(src="introduction/440-to-660hz.mp3", caption="440, 660 Hz") }}

# The Harmonic Series

In nature, when something vibrates, there are *overtones*, which are integer multiples of the *fundamental frequency*. These overtones combine with a fundamental frequency to create timbre, but when taken as fundamental frequencies themselves, they create a series of pitches known as the harmonic series. If you play a brass instrument, these will be familiar to you: all the notes in the harmonic series (that you can reach!) are played with the same fingering.

The sample below holds a fundamental frequency of 100 Hz and plays, on top of it, each successive note in the harmonic series up to the 12th harmonic. This is our first example of a custom scale, but don't worry about the syntax yet...that will be explained later.

```syntoniq
syntoniq(version=1)
define_scale(scale="harmonics") <<
1 h1 | 2 h2 | 3 h3 | 4 h4 | 5 h5 | 6 h6
7 h7 | 8 h8 | 9 h9 | 10 h10 | 11 h11 | 12 h12
>>
use_scale(scale="harmonics")
set_base_pitch(absolute=100)
[p1.0] 12:h1
[p1.1] 1:h1 h2 h3 h4 h5 h6 h7 h8 h9 h10 h11 h12
```

{{ audio(src="introduction/harmonic-1-to-12.mp3", caption="First 12 notes in harmonic series") }}

# Just Intonation and Equal Tuning

You can create musical scales based on ratios. The first musical scales were likely created in this way since these intervals arise in nature.

We've already seen that the octave is 2/1 and the perfect fifth is 3/2. Here are some more:

| ratio | interval |
| --- | --- |
| 2/1 | octave |
| 3/2 | perfect fifth |
| 4/3 | perfect fourth |
| 5/4 | major third |
| 6/5 | minor third |
| 9/8 | whole tone |

If you know about the circle of fifths, you know you can cycle around all the notes of the scale by fifths. On a piano, you can play C → G → D → A → E → B → F♯ → C♯ → G♯ = A♭ → E♭ → B♭ → F → C. This is 12 steps of a perfect fifth, and it covers 7 octaves. (Notice the symmetry: a fifth is 7 chromatic steps in the 12-tone scale.) What would be the frequency multiplier if you took 12 fifths? It would be $(\frac{3}{2})^{12}$. How close is this to what you would expect with seven octaves? Seven octaves would be $2^7$. Do these two numbers match? Not quite: $\frac{3}{2}^{12} \approx 129.74633$, but $2^7 = 128$, so it doesn't quite line up.

Historically, there have been many ways to compensate, but the one that is prevalent in virtually all western music is 12-tone equal temperament (12-TET), also called 12-EDO (equal divisions of the octave). In this system, a half step is $\sqrt[12]{2}$, or $2^\frac{1}{12}$. If you take 12 of those and multiply them together, you get 2, a perfect octave.

Let's make this more concrete with a different discrepancy. If a perfect fifth is $\frac{3}{2}$, then two of them together are $\frac{3}{2}\times\frac{3}{2} = \frac{9}{4}$. This happens to be double $\frac{9}{8}$. Does this check out? Yes: two perfect fifths above C is D ($\frac{9}{8}$) on octave up ($\times 2$). D is a whole tone above C, and our table above shows a whole tone as $\frac{9}{8}$. What happens if we take two whole steps? On a piano, this would bring you to the note E. What do we get if we just repeat the whole step? $\frac{9}{8}\times\frac{9}{8} = \frac{81}{64}$. That should be a major third, right? But wait...a major third is $\frac{5}{4} = \frac{80}{64}$, and we just got $\frac{81}{64}$. There's a difference of $\frac{81}{80}$. (If you're paying really close attention, you may notice that I called this a "difference" but didn't subtract...I divided: $\frac{81}{64} \div \frac{80}{64} = \frac{81}{80}$. We have to divide and multiply because of the logarithmic relationship between frequency and pitch, but I called it a difference because that's how we perceive it! While less formally correct, I think this terminology is more intuitive.) This ratio, $\frac{81}{80}$ is known as the *syntonic comma* and is the tiny interval from which Syntoniq takes its name!

Let's hear what this sounds like. In this example, you'll see the use of the built-in "JI" scale with some unfamiliar note names like `I` and `II`. We will come back to that later! You'll also get a sneak preview of a few of Syntoniq's other features.

```syntoniq
syntoniq(version=1)
; Part p1 uses the default scale. Part p2 uses the built-in "JI"
; scale, for just intonation.
use_scale(scale="JI" part="p2")

; Play two whole steps and a major third using 12-tone equal
; temperament
[p1.0] 1:c d e ~ 3:e 1:~
[p1.1] 4:~       3:c 1:~

; Play two whole steps and the 81/64 interval using just intonation
[p2.0] 1:A I II ~ 3:II 1:~
[p2.1] 4:~        3:A  1:~

; Switch back and forth between a perfect 5/4 major third, a 12-tone
; equal temperament major third (2^(1/3)), and the 81/64 ratio from
; stacking two whole tones.
tempo(bpm=30)
mark(label="a")
[p1.0] 1:~ c ~  ~
[p1.1] 1:~ e ~  ~
[p2.1] 1:A ~ A  ~
[p2.2] 1:E ~ II ~
mark(label="b")

; Repeat three more times
repeat(start="a" end="b")
repeat(start="a" end="b")
repeat(start="a" end="b")
```

{{ audio(src="introduction/major-thirds.mp3", caption="5/4, $\sqrt[3]{2}$, 81/64") }}

Syntoniq has its own notation for [representing pitches](TODO). We'll discuss that later in the manual. For now, this has been a review of the basics of just intonation and equal-step tuning. We'll continue to build upon that in the remainder of the manual.
