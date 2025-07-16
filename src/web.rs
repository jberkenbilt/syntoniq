use crate::events::Color;
use std::collections::HashMap;
use std::sync::LazyLock;

// See color.py for iterating on color choices.
static _RGB_COLORS: LazyLock<HashMap<Color, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (Color::Blue, "#6161ff"),
        (Color::Green, "#61ff61"),
        (Color::Purple, "#a161ff"),
        (Color::Pink, "#f98cff"),
        (Color::Red, "#dd6161"),
        (Color::Orange, "#ffb361"),
        (Color::Cyan, "#61eeff"),
        (Color::Yellow, "#ffff61"),
        (Color::DullGray, "#b3b3b3"),
        (Color::White, "#ffffff"),
    ])
});
