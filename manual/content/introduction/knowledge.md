+++
title = "Prerequisite Knowledge"
weight = 30
sort_by = "weight"
+++

* Microtonality Knowledge Check -- a high-level fly-over of basic tuning theory with external links; flavor is "Explaining this is out of scope, but here are some things you should learn about to get the full benefit". I will introduce concepts that I will refer to in the docs. I will try to make the docs stand-alone to the extent possible, but there is an assumption that someone using this tool is not a total microtonality newbie.

----------



You don't need to be a tuning expert to use Syntoniq...but it helps! To go beyond using Syntoniq with the built-in scales, you should understand a little about and pure "just intonation" interval ratios and about equal divisions of an octave or other interval. There is plenty of good information about these topics. This section covers the basics. If you can't follow the material here, you may need to do a little more work before you are ready to define your own scales with Syntoniq.

# Interval Ratios and Interval Divisions

This section is technical. You don't have to understand it deeply. If you do, great -- you have all the background you need to understand how to create pitches and custom scales in Syntoniq! If not, just know that some things might seem confusing. If you want to master Syntoniq with custom scales, you may need to find some reference materials to learn this. The goal here is not to teach you everything there is to know about tuning theory. It just to set an expectation of what knowledge is assumed. The target audience here is people who are designing their own scales.

TODO: flesh this out and provide some links, but don't try to teach it.

A pitch is defined by a *frequency*, e.g. 440 Hz for the "A" above "middle C".

If you double a frequency, the pitch sounds on octave higher, so 220 Hz and 880 Hz are also the "A" note, an octave below and an octave above the 440 Hz A.

This is often described in terms of dividing a vibrating string. If you create a "node" by putting your finger on a spot in the exact center of the string, each half will vibrate at 880 Hz and sound on octave higher. A string 1/3 as long would vibrate at three times the frequency, or 1320 Hz. It's pitch would be an octave and a fifth higher. The ratio of the frequencies between the octave and the octave + fifth is 3/2, so we call 3/2 the ratio of a perfect fifth. These ratios correspond to the harmonic sequence and relate to overtones of musical tones other than a pure sine wave. Here are some common ratios:

| ratio | interval |
| --- | --- |
| 2/1 | octave |
| 3/2 | perfect fifth |
| 4/3 | perfect fourth |
| 5/4 | major third |
| 6/5 | minor third |
| 9/8 | whole tone |

If you know about the circle of fifths, you know you can cycle around all the notes of the scale by fifths. On a piano, you can play C → G → D → A → E → B → F♯ → C♯ → G♯ = A♭ → E♭ → B♭ → F → C. This is 12 steps of a perfect fifth, and it covers 7 octaves. (Notice the symmetry: a fifth is 7 chromatic steps in the 12-tone scale.) What would be the frequency multiplier if you took 12 fifths? It would be $(\frac{3}{2})^{12}$. How close is this to what you would expect with seven octaves? Seven octaves would be $2^7$. Do these two numbers match? Not quite: $\frac{3}{2}^{12} \approx 129.74633$, but $2^7 = 128$, so it doesn't quite line up.

Historically, there have been many ways to compensate, but the one that is prevalent in virtually all western music is 12-tone equal temperament (12-TET), also called 12-EDO (equal divisions of the octave). In this system, a half step is $\sqrt[12]{2}$. If you take 12 of those and multiplying them together, you get 2, a perfect octave.

When working with tuning systems, we are usually working with one of two things: a pure ratio or the $n$th root of some small number, usually 2.

Later, we'll come back to Syntoniq's lossless pitch notation and generated scales.
