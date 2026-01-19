+++
title = "Pitch and Note Primer"
weight = 20
sort_by = "weight"
+++

Before we continue with using Syntoniq for creating microtonal music, let's take a quick diversion into Syntoniq's *lossless pitch notation* and the related method for creating note names on the fly using Syntoniq's generated scales.

# Pitch Notation

Traditional systems for creating arbitrary scales, like [Scala](https://huygens-fokker.org/scala/), are tied to the baggage of 12-tone systems. Scala files can contain ratios, such as the ones we discussed in [Prerequisite Knowledge](../knowledge/), and they can express intervals in terms of *cents*. In traditional notation, one *cent* is 1/1200th of an octave, or 1/100th of an equal temperament semitone. Cents are very useful when talking about perception or tuning instruments that are playing 12-tone music, but it doesn't make a lot of cents [sic] to rely on that method when defining scales. If you see a number like 70.588¢, you can't tell (unless you're really good at mental math) that this is very nearly 1/17th of an octave. There are actually two problems with this. One is that the number 70.588 doesn't have any immediately obvious *meaning*. The other is hiding in the phrase "very nearly". Since a well-trained ear can discriminate pitches in context with a resolution of only about 3¢, a number like 70.588, accurate to 0.001¢, is good enough for all practical purposes...but it is still an approximation, rounded to a few decimal places. These errors add up. Let's take a simple example. Suppose you approximated the square root of 10 ($\sqrt{10}$) as 3.16. You want $(\sqrt{10})^2 = 10$, but $3.16^2$ = 9.9856. When you do arithmetic on decimal approximations of irrational numbers, error adds up. It may not be significant, but it can matter, especially since the amount of error increases as you do more and more steps. Take 11.429¢. This is about 1/105th of an octave. 11.321¢ is about 1/106th of an octave. If you see 11.37¢, is this supposed to be 2/211ths of an octave, or is it accumulated rounding errors? There's really no way to know.

Let's say you want to express the pitch of middle C as exactly 9 steps of size $\sqrt[12]{2}$ below 440 Hz. You could represent this as 261.626 Hz, but that's not *exactly* right. What if we could directly represent $440\times 2^\frac{-9}{12}$? This is what the Syntoniq pitch notation allows you to do.

There are various ways people have tried to do this. Syntoniq has its own, distinct way. It is designed to be easy to type on a regular keyboard, and it avoids the `\` character, which gets confusing in formats like TOML or JSON or in programming languages since it is usually a string quoting character. Here's how it works.

* A *pitch* is the product of *factors*.
* A *factor* is either a *ratio* or a *ratio raised to a rational power*.
* A *ratio* is written as `a` or `a/b`, which just represent the values $a$ or $\frac{a}{b}$. You can also write a ratio as a decimal with a maximum of three decimal places. (This works everywhere rationals are allowed, including note durations.)
* A *ratio raised to a rational power* is written as `a/b^c|d`. This represents $\frac{a}{b}^{\(\frac{c}{d}\)}$.
  * If you omit both `a` and `b` and write `^c|d`, it means $2^{\(\frac{c}{d}\)}$.
  * If you omit `b` and write `a^c|d`, it means $a^{\(\frac{c}{d}\)}$.
* The *product of factors* is represented by concatenating factors with `*`. You can optionally prepend a leading `*`, which makes it easier to just concatenate strings together to form pitches.

Examples:
* If you want middle C as $440 \times \sqrt[12]{2}$, you can write it as `440*^-9|12`. This is also the same as `220*1|4`. Syntoniq *canonicalizes pitches* so that it recognizes these as *the same pitch*. There is no question about accumulating error.
* It's common to standardize on that pitch for middle C. What if you want one step of 17-EDO (an octave divided into 17 equal parts) above middle C? Yes, this is something people often want. (In the US, people also buy "12-inch metric rulers. Think about it.) Rather than multiplying 261.626 by 70.588¢, you can write `440*^-9|12*^1|17`.

The Syntoniq pitch notation has two major advantages:
* It is *lossless*&mdash;there is no accumulation of rounding errors
* It is *semantically meaningful*&mdash;you can see that `440*^-9|12*^1|17` is 440 lowered by 9 equal 12th-octave steps and then raised by 1 equal 17th-octave step.

Syntoniq allows you to create frequencies and perform transpositions using this pitch notation. That will be discussed in later sections.

# Notes in Generated Scales

In 12-tone notation, notes get letter names from `A` to `G` with accidentals like ♯ and ♭ to change their pitches. There are several problems with this system that affect microtonal musicians:
* This system is inconsistent across keys. A major third from `C` is `E`, but a major third from `D` is `F♯`. (Actually, this is a headache for 12-tone musicians too!)
* There aren't enough notes for scales that have lots of divisions. People solve this by inventing all sorts of different accidentals. This manual is not going to discuss those systems other than to acknowledge their existence.
* This system ties you to 12-tone thinking. If you are trying to superimpose 12-tone music on a scale that also has good approximations for the notes in a diatonic scale, like 19-EDO or 31-EDO (EDO = Equal Divisions of the Octave), these note names are okay, but what if you are playing around in 23-EDO or 7-EDO? These scales don't map to our regular note names very well.
* If you are working in just intonation, when you say `D`, do you mean a ratio of 9/8 over C? A ratio of 10/9 over C, which is 9/8 *below* 5/4 *above* C and is also perceived as a whole tone? If you want a B♭, do you want the one that is a 6/5 minor third above G (ratio 9/5), or do you want the one that's a fourth above F (ratio 16/9)? Or maybe something entirely different? It's very hard to specify which one you want.

Syntoniq introduces the concept of [generated scales](../generated-scales/), discussed later, but for now, here's what you need to know:
* Generated scales contain notes that are *constructed from pure ratios*.
* A generate note can either exist as a pure interval or be *superimposed* on an equal-step tuning system, which can divide an octave or *any other interval*.

Here's how the system works with just intonation.

* Note names are always *relative to the base pitch* of the tuning. By default, the base pitch is `220*^1|4`, which is the 12-tone middle C based on A440.
* `A` and `a` represent the base pitch of the scale.
* Uppercase letters `B` through `Y` represent the ratio $\frac{n}{n-1}$, where $n$ is the *ordinal position of the letter in the alphabet*. For example `B` is 2/1, `C` is 3/2, `D` is 4/3, etc. These letters represent *moving up one step in the harmonic sequence*.
* Lowercase letters `b` through `y` are the reciprocals of their uppercase counterparts. For example `b` is 1/2, `c` is 2/3, `d` is 3/4, etc. These letters represent *moving down one step in the harmonic sequence*.
* The letter `Z` must be followed by a number. `Zn` represents the ratio $\frac{n}{n-1}$. For example, `Z33` is 33/32. This allows you to access higher harmonics, though these are seldom needed for reasons that will become clear momentarily.
* The letter `z`, followed by a number, is the reciprocal of its uppercase counterpart, so `z33` is 32/33.
* Concatenating notes *multiples their ratios*. So `CE` is the same as $\frac{3}{2} \times \frac{5}{4} = \frac{15}{8}$. This is a major seventh written as a major third above a perfect fifth. Likewise, you could write `Bf`, meaning $2 \times \frac{5}{6} = \frac{5}{3}$, a minor third below the octave, or a major sixth. What about 33/32? You can write this is `Ik`, which is $\frac{9}{8} \times \frac{11}{12} = \frac{33}{32}$. Semantically, this is saying "go up 9/8, then go down 11/12". This represents moving up by one step in the harmonic sequence and then moving down by a slightly later step, and when working in just intonation, that's often how you land on these later, smaller step sizes. You might even write `Ik` without even realizing that it's the same as `Z33`.

Syntoniq includes a built-in scale called "JI" that uses this system. (If you want to see the built-in scales in syntoniq format, you can just run `syntoniq built-in-scales`.) Additionally, when generated scales are superimposed on a divided interval, the following additional rules apply:

* Rather than a note specifying a pure ratio, it represents *the closest step in the scale* to that ratio. For example, in a 12-tone equal tempered "conventional" C major scale, the Syntoniq generated note `C` would correspond to conventional note `G` because, that scale, the note `G` is the closest note to the ratio 3/2. To be precise, `G` is the seventh step in 12-EDO, so its relative pitch is `^7|12` $= 2^{\frac{7}{12}} \approx 1.4983$. You can see that this is a tiny bit less than $\frac{3}{2} = 1.5$. In fact, the fifth in 12-EDO is just under 2¢ flat compared to a perfect fifth, and that's below most people's threshold to hear...though we can detect even smaller differences from pure intervals if we are trained to listen. That's why a fifth sounds very pure in 12-EDO.
* You can add `#` or `%` to a note, causing it to *round up* or *round down* instead of moving to the closest note. For example, `L` would represent 12/11, which is roughly between a half step and a whole step. In a Syntoniq generated scale with 12 divisions of the octave, you could get a half step with `L%` or a whole step with `L#`. Since `12/11` is very slightly closer to a whole step than a half step (it's about 150.637¢), plain `L` would map to a whole step with 12 divisions. `#` and `%` are valid in pure just intonation, but they never change the pitch of a note. When defining a generated scale, you can also supply a *tolerance*&mdash;pitches that are within the tolerance are considered exact matches and are not affected by `#` and `%`.
* You can add `+` or `-` to move up or down one step. For example, in 31-EDO, our note `C` would correspond to the 18th step, which is the closest step to a perfect fifth in that scale. You could use `C-` to refer to the 17th step and `C+` to refer to the 19th step. You could use `C++` to refer to the 20th step, but why would use choose C++ if you could choose rust? (A little programming language joke...Syntoniq is implemented in rust!)
* The letters `A` and `a` can be followed by a number. `An` means to go *up* $n$ steps. `an` means to go *down* $n$ steps. That means that, if your scale has $d$ equal divisions of the ratio $\frac{a}{b}$, `An` is equivalent to `a/b^n|d`, and `an` is equivalent to `a/b^-n|d`. The notes `A0` and `a0` are always the same as the root pitch and are also accepted in just intonation.

Here are the last few rules of generated note names:

* If you end a note name with `!`, it will always use the exact ratio regardless of the number of divisions in the scale. This allows you to insert a pure just intonation ratio in a equal-step tuning.
* You can add numbers after `!` representing the number of divisions and the interval being divided. The syntax is different, but the defaults are the same as with pitch notation. In all cases, this changes the interpretation of  `+`, `-`, `An`, `an`, `#`, and `%` as if the scale had been defined to divide the given interval into the given number of divisions. Specifically:
  * `!z`&mdash;divide the octave (ratio 2) into $z$ equal steps
  * `!y/z`&mdash;divide the ratio $y$ into $z$ equal steps
  * `!x/y/z`&mdash;divide the ratio $x/y$ into $z$ equal steps

Examples of using `!`:
* If you want a quarter tone below a 12-EDO fifth, you could write `C-!24`, which means "go one 24th of an octave below the note closest to a fifth in 24-EDO."
* For example, `C!3/19` would be the closest pitch to a degree in a scale made up of 19 divisions of the ratio 3, which is an octave and a fifth. Since there are 19 regular 12-tone half steps in an octave and a fifth, a step in 19 divisions of 3 is about the same size as a half step, and this would correspond to the pitch factor `3^7|19`.
* The note `C!9/4/14` would treat the scale as having 14 divisions of the ratio $\frac{9}{4}$, which also has a step size of about a half step. This corresponds to `^9/4^7|14` = `^9/4^1|2`, or exactly half of the 9/4 ratio.

# Defining Generated Scales

You can define a generated scale using the `define_generated_scale` directive. You can always run `syntoniq doc` to get a summary of the syntax of any directive. Examples of `define_generated_scale` will appear in later sections. When you define a generated scale, you can specify the cycle size (the ratio that `'` and `,` move the pitch by), the divided interval, which defaults to the cycle size but doesn't have to match it, the number of divisions, and the tolerance for deciding whether a pitch is a perfect match for purposes of `#` and `%`. This provides a great deal of flexibility, allowing you to work in different tuning systems and to define pitches functionally.

# Enharmonics

A property of generated notes is that there are many (infinitely many, in fact) ways to "spell" the name of any note. The generated note syntax is actually equivalent to the lossless pitch notation. (Proving this mathematically is out of scope, but the gist is that you can always write the product of rationals and rationals raised to rational powers as a single rational raised to a single rational power, and `Ay!w/x/z` is the same as `w/x^y|z`.) This means you can create virtually any pitch using this notation.

In regular 12-tone notation, notes have *enharmonic spellings*. For example, you could write either `F♯` or `G♭`, and those would have the same pitch in 12-EDO. You would use `G♭` if you wanted the third in an `E♭` minor scale, and you would use `F♯` if you wanted the third in a `D` major scale. This generated note system also has enharmonics, and you can also use them to spell a note functionally. For example, consider the note whose pitch as a major sixth above the root. This is usually represented by the ratio 5/3, which is a 6/5 minor third below the octave ($2 \div \frac{6}{5} = \frac{5}{3}$). But perhaps you wanted a perfect fifth above the whole tone. This would be $\frac{9}{8} \times \frac{3}{2} = \frac{27}{16}$. Dealing with these kinds of ratio calculations are the bane of existence for anyone working in just intonation. In this system, the first note could be spelled `Bf` (a minor sixth below the octave), and the second one would be spelled `IC`, a fifth above the whole tone. The order doesn't matter, so you could write these is `fB` or `CI` if you wanted. Why would you want to? Perhaps you have a chord, and you want to shift the whole chord by the same amount. You can do this by appending or prepending the same letter to it. For example, a perfect major triad is `A E C`. If you want that exact same chord with the same ratios up a whole step, you could write `IA IE IC`. Since `A` is multiplying by 1, you could also just write `I IE IC`. In this way, you can "spell" a note based on its harmonic function just like with regular notation, only with a lot more flexibility. It might sound complex, and compared to working in 12-EDO as a trained musician, it is...but when you start trying to use pure just intonation or exploit features of higher EDO scales, once you get used to it, you might find this is actually a lot simpler than other systems.

Before leaving this topic, I'll point out one other interesting thing that comes into play with higher EDO scales. In 31-EDO, you can approximate a major scale by realizing that there are 5 steps to a whole tone and 3 steps to a semitone. (I'm hand-waving here and glossing over the 2-step chromatic semitone, but this is the gist. $31 = 5+5+3+5+5+5+3$, just like $12 = 2+2+1+2+2+2+1$.) If you want the major third in that scale, you can find it at step 10, numbered from 0. This is just where you'd expect it to land: two 5-step "whole steps" above the root. That means that, for a 31-EDO generated scale, `A10` is enharmonic with `E`. What happens if you take this in 41-EDO? 41-EDO also works for diatonic music because we have $41 = 7+7+3+7+7+7+3$. Following the above logic, we'd expect to find the major third at step 14 (two 7-step "whole tones" above the root)...and step 14 is an okay major third. But remember that the major third in 12-EDO is a little sharp. It turns out that the 13th step of 41-EDO is actually a better major third than the 14th step. In our notation `E` in 41-EDO is enharmonic with `A13`, not `A14`! But it's a little flat. If you wanted the sharper major third, you could write `E#`, and that would land on `A14`. It may take some getting used to, but I hope you can see how powerful and versatile this system is.

# 12-EDO Cheat Sheet

Here are some ways to spell pitches using generated notes that map to 12-EDO. This chart uses all 5-limit ratios, meaning ratios whose numerators and denominators contain no prime factors above 5. (This is a topic in just intonation. If you don't know about it, don't worry; you don't need to know for Syntoniq.)

| 12-EDO | Generated | Ratio |
|--------|-----------|-------|
| C      | A         | 1     |
| C♯/D♭  | P         | 16/15 |
| D      | I         | 9/8   |
| D♯/E♭  | F         | 6/5   |
| E      | E         | 5/4   |
| F      | D         | 4/3   |
| F♯/G♭  | Cp        | 45/32 |
| G      | C         | 3/2   |
| G♯/A♭  | Be        | 8/5   |
| A      | Bf        | 5/3   |
| A♯/B♭  | Bi        | 16/9  |
| B      | Bp        | 15/8  |
