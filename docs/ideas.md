# Ideas

This is a repository of ideas that may or may not ever be worth building.

# Printed Scores

Muse about printed notation hints. We could potentially generate MusicXML, LilyPond, or add enough metadata to the timeline JSON dump that someone could do their own notation from it.

# Note Name Optimization

Possible note name optimization algorithm. Goal: given a/b, find the "simplest" way to represent a/b as products of n/n-1 and n-1/n. Rough criteria:

* (maybe) normalize to cycle, or maybe to `(1/cycle, cycle)` so simple n-1/n answers can win.
* generate paths
* separate into paths with and without any `n > 25`. If any with `n <= 25`, eliminate the `n > 25` paths.
* among the paths whose length is equal to the length of the shortest path, pick the result with the smallest max(n).
* For winning path, write factors in increasing order of n.  Note that n/n-1 and n-1/n will never both appear.
* XXX complete -- fails for `13/11` or other ratios of non-adjacent primes, among other cases -- see discussion. Perhaps, if we fail to factor anything out, we can split into sequences of `n/n-1*n-1/n-2*...` and then combine aligned items as in the examples below.

To generate paths, use dynamic programming or a recursive function that takes the normalized fraction and factors out choices with some disallowed items. Each time we factor out either n from the numerator and n-1 from the denominator or n-1 from the numerator and n from the denominator, call recursively disallowing the reciprocal of what we just factored out. If we start with simplest form, this filtering may not be required. Never allow the value to fall outside (1/cycle, cycle). Terminate a path if we find n/n-1 or n-1/n with n <= 25.

Examples:
* `16/15` -> normalized, simplified (this is a single n/n-1 with n <= 25 so it automatically wins, but...)
  * `2`, `8/15`
    * [`2`], `2`, `4/15` -> reject (4/15 is too small)
    * [`2`], `2/3`, `4/5` -> terminate
  * `2/3`, `8/5`
    * [`2/3`], `2`, `4/5` -> terminate [could detect duplicate using dynamic programming if we sort by n]
  * `4/3`, `4/5` -> terminate
  * Results: `16/15`, `4/3*4/5`, `2*2/3*4/5`. Winner: `16/15`

* `16/15` is better than `4/3*4/5`
* For `27/16`, `3/2*9/8` best and is found either when factoring `3/2` out or when factoring `9/8` out.
* For `13/8`, there is `3/2*13/12` and also `7/6*13/14`. The winner is `3/2*13/12`.
* For `7/4`, there is `1/2*7/8` and `3/2*7/6`. The first one wins for sum(n) (9 vs 10) and min(n) (2 vs. 3), but the second one wins for smallest max(n) (7 vs. 8) and smallest number if n-1/n (0 vs 1). I think I like the second one better, suggesting max(n) or number if n-1/n. These are probably related but might not always agree.
* For `9/4`, it normalizes to `9/8`, which wins. This keeps `9/8` winning over `3/2*3/2`.
* For `81/80`, there is a single fraction (81/80) but there's also `9/8*9/10`, which is probably better based on the `n <= 25` rule. (`Ij` vs. `Z81`)
* `5/3` -> `1/2*5/6` -- this shows the need to potentially halve or double within the cycle range.
* `13/11` -> fails using the above. We may need to add something that converts this to `12/11*13/12`. Consider `17/13` -> `14/13*15/14*16/15*17/16`. There are opportunities around adjacent steps that align with powers of 2. For example, this can become `8/7*14/13*17/16`.
* `19/11` -> `4/3*9/8*12/11*/19/18`
* For `128/125`, there's `64/63*126/125` (can only be found from `128/127*127/126*126/125`) and also `2*4/5*4/5*4/5`. The second one wins because of the `n <= 25` preference.
* `7/4` -> `7/6*6/5*5/4` = `7/6*3/2`.
* `27/16` -> `27/26*13/12*3/2`

* `37/19` -> `37/36*36/19` = `37/36*2*18/19` or `37/20*20/19`, `37/20` = `37/36*36/20`, `36/20` = `9/5` = `1/2*9/10`; also `37/20` = `1/2*37/40`....

So we can always try splitting `n/n-k` into `n/n-(k-a) * n-(k-a)/n-k` for each a from 1 to k-1. We could potentially always do that. We could also uses each factor of n as a step.

* `39/28` = `39/36 * 36/28` = `13/12 * 9/7` = `13/12*9/8*8/7` but also `39/38*38/28` = `39/38*19/14` = `39/38*19/18*9/7` = `39/38*19/18*9/8*8/7`. This is probably better than recursively processing `39/37*37/28`.

So, when we have a/b, consider whichever of `(a/b)*cycle` or `(a/b)/cycle` is in the range since one always will be. Then, in addition to factoring out n/n-1 or n-1/n terms, also do the thing where we try adjacent steps. So for 39/28, would would try `39/38*38/28`, `39/36*36/28` and `39/26*26/28`. Oops, we can factor out `3/2`, so `39/28` = `13/14*3/2`...but this would also be found as `39/26*26/28`. Do we *only* need the second way?

* `81/80`: have to check all the multiples of 3 that are in range, so this is `81/80`, `78/80` ... `42/80`. Also, I think we have to go up, e.g. `84/80`...`159/80`...with this, we would find `81/90*80/90` which would give us `9/8*8/9`.

Will this always work? I think it will...but we need some changes. Do we need to look at `81/160` at all? And what about the denominator? Should we check `81/78*78/80`, `81/70*70/80`, etc.? I think the combination of looking at denominator factors and numerator factors going all the way across the range from 1/cycle to cycle would surely do it.

I think limiting the range to neighboring powers of 2 should be okay. So 81/80 would search numerators between 64 and 128. Also denominators. `128/125` would search denominators down to 64. This would find `64/63*126/125`, but also when we got to `128/80`, this would simplify to `8/5` which would find `2*4/5`, so I think it works. This might be too expansive though.

# Piano Keyboard Layout

This idea could be usable for any scale that maps nicely to diatonic (12, 17, 19, 31, 24, 36, 41, 48, 51, 53, 72, a few others) as long as max(scale-degrees-to-semitone, ceil(scale-degrees-to-whole-tone/2)) < number of octaves on the piano.

* note 60 (middle C) is middle base pitch
* half step (white key to black key above it, e to f, b to c) is diatonic semitone
* whole step is whole step
* octave is single step

This gives you something kind of almost like an isomorphic keyboard in feel. For example, in 31-EDO, note 60 would be C, note 61 would be 3 scale degrees up, note 62 would be 5 scale degrees up, ..., note 72 would be one scale degree up, etc. What you'd have with this layout scheme is that the middle C octave would play like a "regular" scale, and the octave to the right would be up one step, etc.

Using my notation of A0 = root, A1 = one step, A2 = 2 steps, etc., 19-EDO would map to the middle C octave and the one above it like this:
```
  A2  A5      A10  A13  A26       A3  A6      A11  A14  A27
A0  A3  A6  A8  A11  A14  A17   A1  A4  A7  A9  A12  A15  A18
```

This would not be the same experience as a hex grid, but it allow you to play 12-tone-like music in different tuning systems. Take the simplest case of 24-EDO: the middle C/note 60 octave would be as it is on a regular keyboard. The octave above would be the same octave shifted up a quarter tone. The octave behind that would be one octave above middle C, etc. For a piano keyboard, this would make those particular EDO flavors manageable.

This would require a different type of layout that wasn't a grid, and we'd have to know about half-step and whole-step intervals, but it would be a relatively light lift.

We'd have to solve function keys (reset, layout selection, octave shift, sustain, shift, transpose), but this could be done in various ways. We have pedals and other things that generate MIDI events that could be mapped.
