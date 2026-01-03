+++
title = "Layout Engine"
weight = 60
sort_by = "weight"
+++

* This section probably doesn't get an accompanying video as it is very technically dense and more coding oriented.
* Go over syntoniq language features for creating custom isomorphic and manual mappings and combining mappings to create layouts, including row and column layout for rectangular and hexagonal grids (with rationale for hexagonal)
* Mention that, while the software doesn't prevent you from using isomorphic mappings with uneven tunings, it might create a confusing situation, but there are use cases if you "know what you're doing", such as dealing with regular but uneven tunings or intentionally experimenting with out-of-tune keys in JI
* Work in the 17-EDO layout somewhere and point out the lack of third/sixth colors

<!-- generate include=keyboard.stq checksum=e82304e9ba1a938b08606997373534c7b2ef4c8f49b3fb2f35b03d4d10999d21 -->
```syntoniq
syntoniq(version=1)

define_isomorphic_mapping(mapping="12-EDO-h2v5" steps_h=2 steps_v=5)
place_mapping(layout="12-EDO-h2v5" mapping="12-EDO-h2v5" keyboard="launchpad" anchor_row=4 anchor_col=3)
place_mapping(layout="12-EDO-h2v5" mapping="12-EDO-h2v5" keyboard="hexboard" anchor_row=8 anchor_col=8)
place_mapping(layout="12-EDO-h2v5" mapping="12-EDO-h2v5" keyboard="hexboard-60" anchor_row=9 anchor_col=8)

define_isomorphic_mapping(mapping="19-EDO-h3v2" scale="19-EDO" steps_h=3 steps_v=2)
place_mapping(layout="19-EDO-h3v2" mapping="19-EDO-h3v2" keyboard="launchpad" anchor_row=4 anchor_col=3)
place_mapping(layout="19-EDO-h3v2" mapping="19-EDO-h3v2" keyboard="hexboard" anchor_row=8 anchor_col=7)
place_mapping(layout="19-EDO-h3v2" mapping="19-EDO-h3v2" keyboard="hexboard-60" anchor_row=9 anchor_col=8)

define_isomorphic_mapping(mapping="31-EDO-h5v3" scale="31-EDO" steps_h=5 steps_v=3)
place_mapping(layout="31-EDO-h5v3" mapping="31-EDO-h5v3" keyboard="launchpad" anchor_row=4 anchor_col=3)
place_mapping(layout="31-EDO-h5v3" mapping="31-EDO-h5v3" keyboard="hexboard" anchor_row=7 anchor_col=8)
place_mapping(layout="31-EDO-h5v3" mapping="31-EDO-h5v3" keyboard="hexboard-60" anchor_row=8 anchor_col=7)

define_generated_scale(scale="17-EDO" divisions=17)
define_isomorphic_mapping(mapping="17-EDO-h3v2" scale="17-EDO" steps_h=3 steps_v=2)
place_mapping(layout="17-EDO-h3v2" mapping="17-EDO-h3v2" keyboard="launchpad" anchor_row=4 anchor_col=3)
place_mapping(layout="17-EDO-h3v2" mapping="17-EDO-h3v2" keyboard="hexboard" anchor_row=8 anchor_col=7)
place_mapping(layout="17-EDO-h3v2" mapping="17-EDO-h3v2" keyboard="hexboard-60" anchor_row=9 anchor_col=8)

define_generated_scale(scale="41-EDO" divisions=41)
define_isomorphic_mapping(mapping="41-EDO-h7v3" scale="41-EDO" steps_h=7 steps_v=3)
place_mapping(layout="41-EDO-h7v3" mapping="41-EDO-h7v3" keyboard="launchpad" anchor_row=4 anchor_col=3)
place_mapping(layout="41-EDO-h7v3" mapping="41-EDO-h7v3" keyboard="hexboard" anchor_row=7 anchor_col=8)
place_mapping(layout="41-EDO-h7v3" mapping="41-EDO-h7v3" keyboard="hexboard-60" anchor_row=8 anchor_col=7)

define_generated_scale(scale="53-EDO" divisions=53)
define_isomorphic_mapping(mapping="53-EDO-h9v4" scale="53-EDO" steps_h=9 steps_v=4)
place_mapping(layout="53-EDO-h9v4" mapping="53-EDO-h9v4" keyboard="launchpad" anchor_row=4 anchor_col=3)
place_mapping(layout="53-EDO-h9v4" mapping="53-EDO-h9v4" keyboard="hexboard" anchor_row=7 anchor_col=8)
place_mapping(layout="53-EDO-h9v4" mapping="53-EDO-h9v4" keyboard="hexboard-60" anchor_row=9 anchor_col=7)

define_manual_mapping(
    mapping="JI"
    scale="JI"
    v_factor = 3/2
    h_factor = 2
) <<
p    F   EK    e'   Bh
  @A   I    E    D    C
>>
place_mapping(
    layout="JI"
    base_pitch=264
    keyboard="hexboard"
    mapping="JI"
    anchor_row=7
    anchor_col=5
)
place_mapping(
    layout="JI"
    base_pitch=264
    keyboard="launchpad"
    mapping="JI"
    anchor_row=3
    anchor_col=2
)
place_mapping(
    layout="JI-19-EDO"
    keyboard="launchpad"
    mapping="JI"
    anchor_row=1
    anchor_col=1
    rows_above=3
    rows_below=0
)
place_mapping(
    layout="JI-19-EDO"
    keyboard="launchpad"
    mapping="19-EDO-h3v2"
    anchor_row=6
    anchor_col=3
)
place_mapping(
    layout="JI-19-EDO"
    keyboard="hexboard"
    mapping="JI"
    anchor_row=1
    anchor_col=2
    rows_above=3
    rows_below=0
)
place_mapping(
    layout="JI-19-EDO"
    keyboard="hexboard"
    mapping="19-EDO-h3v2"
    anchor_row=9
    anchor_col=9
)

define_scale(scale="harmonics") <<
1 h1   2 h2   3 h3   4 h4   5 h5   6 h6   7 h7   8 h8
9 h9   10 h10   11 h11   12 h12   13 h13   14 h14   15 h15   16 h16
17 h17   18 h18   19 h19   20 h20   21 h21   22 h22   23 h23   24 h24
25 h25   26 h26   27 h27   28 h28   29 h29   30 h30   31 h31   32 h32
33 h33   34 h34   35 h35   36 h36   37 h37   38 h38   39 h39   40 h40
41 h41   42 h42   43 h43   44 h44   45 h45   46 h46   47 h47   48 h48
49 h49   50 h50   51 h51   52 h52   53 h53   54 h54   55 h55   56 h56
57 h57   58 h58   59 h59   60 h60   61 h61   62 h62   63 h63   64 h64
>>

define_manual_mapping(mapping="harmonics" scale="harmonics") <<
h57 h58 h59 h60 h61 h62 h63 h64
h49 h50 h51 h52 h53 h54 h55 h56
h41 h42 h43 h44 h45 h46 h47 h48
h33 h34 h35 h36 h37 h38 h39 h40
h25 h26 h27 h28 h29 h30 h31 h32
h17 h18 h19 h20 h21 h22 h23 h24
h9 h10 h11 h12 h13 h14 h15 h16
@h1 h2 h3 h4 h5 h6 h7 h8
>>
place_mapping(
    layout="harmonics"
    mapping="harmonics"
    base_pitch=50
    keyboard="launchpad"
    anchor_row=1
    anchor_col=1
)
place_mapping(
    layout="harmonics"
    mapping="harmonics"
    base_pitch=50
    keyboard="hexboard"
    anchor_row=4
    anchor_col=4
    rows_above=7
    rows_below=0
    cols_right=7
    cols_left=0
)

define_generated_scale(scale="13-ED3" cycle_ratio=3 divisions=13)
define_isomorphic_mapping(mapping="13-ED3-h2v3" scale="13-ED3" steps_h=2 steps_v=3)
place_mapping(layout="13-ED3-h2v3" mapping="13-ED3-h2v3" keyboard="hexboard" anchor_row=7 anchor_col=8)

define_generated_scale(scale="27-ED3" cycle_ratio=3 divisions=27)
define_isomorphic_mapping(mapping="27-ED3-h3v5" scale="27-ED3" steps_h=3 steps_v=5)
place_mapping(layout="27-ED3-h3v5" mapping="27-ED3-h3v5" keyboard="hexboard" anchor_row=7 anchor_col=8)
```
<!-- generate-end -->
