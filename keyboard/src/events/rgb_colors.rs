pub const RGB_COLORS: &[&str] = &[
    // This is a mapping from color codes to the RGB values used in the chart
    // in the programmer's manual. It is not necessarily a perfect
    // representation of the color.
    "",        // 00 -- varies by position; handled by content
    "#b3b3b3", // 01
    "#dddddd", // 02
    "#ffffff", // 03
    "#ffb3b3", // 04
    "#ff6161", // 05
    "#dd6161", // 06
    "#b36161", // 07
    "#fff3d5", // 08
    "#ffb361", // 09
    "#dd8c61", // 0a
    "#b37661", // 0b
    "#ffeea1", // 0c
    "#ffff61", // 0d
    "#dddd61", // 0e
    "#b3b361", // 0f
    "#ddffa1", // 10
    "#c2ff61", // 11
    "#a1dd61", // 12
    "#81b361", // 13
    "#c2ffb3", // 14
    "#61ff61", // 15
    "#61dd61", // 16
    "#61b361", // 17
    "#c2ffc2", // 18
    "#61ff8c", // 19
    "#61dd76", // 1a
    "#61b36b", // 1b
    "#c2ffcc", // 1c
    "#61ffcc", // 1d
    "#61dda1", // 1e
    "#61b381", // 1f
    "#c2fff3", // 20
    "#61ffe9", // 21
    "#61ddc2", // 22
    "#61b396", // 23
    "#c2f3ff", // 24
    "#61eeff", // 25
    "#61c7dd", // 26
    "#61a1b3", // 27
    "#c2ddff", // 28
    "#61c7ff", // 29
    "#61a1dd", // 2a
    "#6181b3", // 2b
    "#a18cff", // 2c
    "#6161ff", // 2d
    "#6161dd", // 2e
    "#6161b3", // 2f
    "#ccb3ff", // 30
    "#a161ff", // 31
    "#8161dd", // 32
    "#7661b3", // 33
    "#ffb3ff", // 34
    "#ff61ff", // 35
    "#dd61dd", // 36
    "#b361b3", // 37
    "#ffb3d5", // 38
    "#ff61c2", // 39
    "#dd61a1", // 3a
    "#b3618c", // 3b
    "#ff7661", // 3c
    "#e9b361", // 3d
    "#ddc261", // 3e
    "#a1a161", // 3f
    "#61b361", // 40
    "#61b38c", // 41
    "#618cd5", // 42
    "#6161ff", // 43
    "#61b3b3", // 44
    "#8c61f3", // 45
    "#ccb3c2", // 46
    "#8c7681", // 47
    "#ff6161", // 48
    "#f3ffa1", // 49
    "#eefc61", // 4a
    "#ccff61", // 4b
    "#76dd61", // 4c
    "#61ffcc", // 4d
    "#61e9ff", // 4e
    "#61a1ff", // 4f
    "#8c61ff", // 50
    "#cc61fc", // 51
    "#ee8cdd", // 52
    "#a17661", // 53
    "#ffa161", // 54
    "#ddf961", // 55
    "#d5ff8c", // 56
    "#61ff61", // 57
    "#b3ffa1", // 58
    "#ccfcd5", // 59
    "#b3fff6", // 5a
    "#cce4ff", // 5b
    "#a1c2f6", // 5c
    "#d5c2f9", // 5d
    "#f98cff", // 5e
    "#ff61cc", // 5f
    "#ffc261", // 60
    "#f3ee61", // 61
    "#e4ff61", // 62
    "#ddcc61", // 63
    "#b3a161", // 64
    "#61ba76", // 65
    "#76c28c", // 66
    "#8181a1", // 67
    "#818ccc", // 68
    "#ccaa81", // 69
    "#dd6161", // 6a
    "#f9b3a1", // 6b
    "#f9ba76", // 6c
    "#fff38c", // 6d
    "#e9f9a1", // 6e
    "#d5ee76", // 6f
    "#8181a1", // 70
    "#f9f9d5", // 71
    "#ddfce4", // 72
    "#e9e9ff", // 73
    "#e4d5ff", // 74
    "#b3b3b3", // 75
    "#d5d5d5", // 76
    "#f9ffff", // 77
    "#e96161", // 78
    "#aa6161", // 79
    "#81f661", // 7a
    "#61b361", // 7b
    "#f3ee61", // 7c
    "#b3a161", // 7d
    "#eec261", // 7e
    "#c27661", // 7f
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rgb_colors() {
        // sanity check that generation of the above list was right
        assert_eq!(RGB_COLORS.len(), 0x80);
        assert_eq!(RGB_COLORS[0x7f], "#c27661");
    }
}
