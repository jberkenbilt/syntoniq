use crate::parsing::score::{NamedPitch, Scale};
use crate::parsing::score_helpers;
use crate::pitch::Pitch;
use num_rational::Ratio;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Serialize, Default)]
pub struct Layouts<'s> {
    pub scales: HashMap<Cow<'s, str>, Arc<Scale<'s>>>,
    pub layouts: HashMap<Cow<'s, str>, Arc<RwLock<Layout<'s>>>>,
}

#[derive(Serialize)]
pub struct Layout<'s> {
    pub keyboard: Cow<'s, str>,
    pub mappings: Vec<LayoutMapping<'s>>,
}
impl<'s> Layout<'s> {
    pub fn note_at_location(&self, location: Coordinate) -> Option<PlacedNote<'s>> {
        // Return information from the first mapping that has the note, if any.
        for m in &self.mappings {
            if let Some(r) = m.note_at_location(location) {
                return r;
            }
        }
        None
    }

    /// Shift the mapping so that the key at `from` moves to `to`. `from` and `to` must belong to
    /// the same mapping, but the keys don't have to be mapped within the mapping. The return value
    /// indicates whether the shift was successful.
    pub fn shift(&self, from: Coordinate, to: Coordinate) -> bool {
        for mapping in &self.mappings {
            if mapping.contains(from) {
                if !mapping.contains(to) {
                    return false;
                }
                // This mapping contains both locations, so we can shift.
                let shift_v = to.row - from.row;
                let shift_h = to.col - from.col;
                let mut offsets = mapping.offsets.write().unwrap();
                offsets.shift_v += shift_v;
                offsets.shift_h += shift_h;
                return true;
            }
        }
        false
    }

    /// Transpose the mapping so that the specified location's pitch is the specified pitch.
    /// The key must be mapped.
    pub fn transpose(&self, pitch: &Pitch, location: Coordinate) -> bool {
        for mapping in &self.mappings {
            if let Some(Some(np)) = mapping.note_at_location(location) {
                let factor = pitch / &np.pitch;
                mapping.offsets.write().unwrap().transpose *= &factor;
                return true;
            }
        }
        false
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
pub struct Coordinate {
    pub row: i32,
    pub col: i32,
}

pub struct PlacedNote<'s> {
    /// Note name, including octave/cycle markers
    pub name: Cow<'s, str>,
    /// Scale the note came from
    pub scale: Arc<Scale<'s>>,
    /// Pitch, including base pitch and cycle offset, but not including any keyboard-controlled
    /// transpositions.
    pub pitch: Pitch,
}

#[derive(Default, Clone)]
pub struct Offsets {
    /// Amount the layout is shifted vertically.
    pub shift_v: i32,
    /// Amount the layout is shifted horizontally.
    pub shift_h: i32,
    /// Transposition factor
    pub transpose: Pitch,
}

pub(crate) mod scale_name {
    use crate::parsing::score::Scale;
    use serde::Serialize;
    use serde::Serializer;
    use std::sync::Arc;

    pub fn serialize<S: Serializer>(v: &Arc<Scale>, s: S) -> Result<S::Ok, S::Error> {
        v.definition.name.serialize(s)
    }
}

#[derive(Serialize)]
pub struct LayoutMapping<'s> {
    pub name: Cow<'s, str>,
    #[serde(with = "scale_name")]
    pub scale: Arc<Scale<'s>>,
    pub base_pitch: Pitch,
    pub anchor_row: i32,
    pub anchor_col: i32,
    pub rows_above: Option<i32>,
    pub rows_below: Option<i32>,
    pub cols_left: Option<i32>,
    pub cols_right: Option<i32>,
    pub details: Arc<MappingDetails<'s>>,
    #[serde(skip)]
    pub offsets: Arc<RwLock<Offsets>>,
}

impl<'s> LayoutMapping<'s> {
    pub fn contains(&self, location: Coordinate) -> bool {
        let min_row = self.rows_below.map(|x| self.anchor_row - x);
        let max_row = self.rows_above.map(|x| self.anchor_row + x);
        let min_col = self.cols_left.map(|x| self.anchor_col - x);
        let max_col = self.cols_right.map(|x| self.anchor_col + x);
        // If there is no bound specified, a value is considered in bounds.
        let ge_min_row = min_row.map(|x| location.row >= x).unwrap_or(true);
        let le_max_row = max_row.map(|x| location.row <= x).unwrap_or(true);
        let ge_min_col = min_col.map(|x| location.col >= x).unwrap_or(true);
        let le_max_col = max_col.map(|x| location.col <= x).unwrap_or(true);
        ge_min_row && le_max_row && ge_min_col && le_max_col
    }

    /// If result is None, the mapping does not include the row and column. If it is Some(None),
    /// it includes the row and column, but the spot is unmapped. If Some(Some(_)), it is the note
    /// at that position with its untransposed pitch, covering base pitch, scale degree, and cycle
    /// count.
    pub fn note_at_location(&self, location: Coordinate) -> Option<Option<PlacedNote<'s>>> {
        if self.contains(location) {
            // Shift amounts apply to the layout, effectively moving the anchor by that amount.
            // loc - (anchor + shift) = loc - anchor - shift.
            let offsets = self.offsets.read().unwrap();
            let row_delta: i32 = location.row - self.anchor_row - offsets.shift_v;
            let col_delta: i32 = location.col - self.anchor_col - offsets.shift_h;
            Some(
                self.details
                    .note_at_anchor_delta(&self.scale, row_delta, col_delta)
                    .map(|mut x| {
                        x.base_factor *= &self.base_pitch;
                        x.base_factor *= &offsets.transpose;
                        PlacedNote {
                            name: x.name,
                            scale: self.scale.clone(),
                            pitch: x.base_factor,
                        }
                    }),
            )
        } else {
            // The mapping doesn't cover this coordinate.
            None
        }
    }
}

#[derive(Serialize)]
pub enum MappingDetails<'s> {
    Isomorphic(IsomorphicMapping<'s>),
    Manual(ManualMapping<'s>),
}
impl<'s> MappingDetails<'s> {
    fn note_at_anchor_delta(
        &self,
        scale: &Scale<'s>,
        row_delta: i32,
        col_delta: i32,
    ) -> Option<NamedPitch<'s>> {
        match self {
            MappingDetails::Isomorphic(x) => x.note_at_anchor_delta(scale, row_delta, col_delta),
            MappingDetails::Manual(x) => x.note_at_anchor_delta(row_delta, col_delta),
        }
    }
    pub fn name(&self) -> &Cow<'s, str> {
        match self {
            MappingDetails::Isomorphic(x) => &x.name,
            MappingDetails::Manual(x) => &x.name,
        }
    }
    #[cfg(test)]
    fn as_isomorphic(&self) -> Option<&IsomorphicMapping<'s>> {
        match self {
            MappingDetails::Isomorphic(m) => Some(m),
            MappingDetails::Manual(_) => None,
        }
    }
    #[cfg(test)]
    fn as_manual(&self) -> Option<&ManualMapping<'s>> {
        match self {
            MappingDetails::Isomorphic(_) => None,
            MappingDetails::Manual(m) => Some(m),
        }
    }
}

#[derive(Serialize)]
pub struct IsomorphicMapping<'s> {
    pub name: Cow<'s, str>,
    pub steps_h: i32,
    pub steps_v: i32,
}
impl<'s> IsomorphicMapping<'s> {
    fn note_at_anchor_delta(
        &self,
        scale: &Scale<'s>,
        row_delta: i32,
        col_delta: i32,
    ) -> Option<NamedPitch<'s>> {
        let full_degree = (row_delta * self.steps_v) + (col_delta * self.steps_h);
        let num_degrees = scale.pitches.len() as i32;
        let pitch_idx = full_degree.rem_euclid(num_degrees);
        let cycle = full_degree.div_euclid(num_degrees);
        let base_factor =
            &scale.pitches[pitch_idx as usize] * &Pitch::from(scale.definition.cycle.pow(cycle));
        let given_name = scale.primary_names[pitch_idx as usize];
        let name = score_helpers::format_note_cycle(given_name, cycle);
        Some(NamedPitch { name, base_factor })
    }
}
#[derive(Serialize)]
pub struct ManualMapping<'s> {
    pub name: Cow<'s, str>,
    pub h_factor: Pitch,
    pub v_factor: Pitch,
    /// Valid row index
    pub anchor_row: i32,
    /// Valid column index
    pub anchor_col: i32,
    /// Outer vec is rows, inner vec is columns; all rows have the same number of columns.
    pub notes: Vec<Vec<Option<NamedPitch<'s>>>>,
}
impl<'s> ManualMapping<'s> {
    /// It is required that all elements of `notes` are the same length and that
    /// `anchor_row` and `anchor_column` are valid indices into notes.
    fn note_at_anchor_delta(&self, row_delta: i32, col_delta: i32) -> Option<NamedPitch<'s>> {
        let row = self.anchor_row + row_delta;
        let col = self.anchor_col + col_delta;
        let num_rows = self.notes.len() as i32;
        let num_cols = self.notes.first().expect("notes is empty").len() as i32;
        let row_idx = row.rem_euclid(num_rows);
        let v_repetitions = row.div_euclid(num_rows);
        let col_idx = col.rem_euclid(num_cols);
        let h_repetitions = col.div_euclid(num_cols);
        let note_row = self
            .notes
            .get(row_idx as usize)
            .expect("row_idx out of range");
        let note_col = note_row
            .get(col_idx as usize)
            .expect("col_idx out of range")
            .as_ref();
        let mut named_pitch = note_col?.clone();

        // For manual layout, we don't know the relationship between cycles and factors, and there
        // may not even be one. Don't try to modify the note names. Just adjust the pitches.
        fn adjust(factor: &mut Pitch, mut repetitions: i32, tile_factor: &Pitch) {
            while repetitions > 0 {
                *factor *= tile_factor;
                repetitions -= 1;
            }
            if repetitions < 0 {
                let f = tile_factor.invert();
                while repetitions < 0 {
                    *factor *= &f;
                    repetitions += 1;
                }
            }
        }

        let mut factor = Pitch::from(Ratio::from_integer(1));
        adjust(&mut factor, v_repetitions, &self.v_factor);
        adjust(&mut factor, h_repetitions, &self.h_factor);
        named_pitch.base_factor *= &factor;
        Some(named_pitch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing;
    use crate::parsing::Options;
    use std::sync::LazyLock;

    static LAYOUTS: LazyLock<Layouts<'static>> = LazyLock::new(|| {
        let input = include_str!("test-data/layouts.stq");
        parsing::layouts("", input, &Options::default()).unwrap()
    });

    #[test]
    fn test_isomorphic() {
        let layout = LAYOUTS.layouts.get("l2").unwrap().read().unwrap();
        let mapping = &layout.mappings[1];
        assert_eq!(mapping.name, "m2");
        assert!(mapping.details.as_manual().is_none());
        let im = mapping.details.as_isomorphic().unwrap();
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, 0, 0).unwrap(),
            NamedPitch {
                name: "c".into(),
                base_factor: Pitch::unit(),
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, 0, 1).unwrap(),
            NamedPitch {
                name: "d".into(),
                base_factor: Pitch::must_parse("^1|6"),
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, 0, -1).unwrap(),
            NamedPitch {
                name: "b%,".into(),
                base_factor: Pitch::must_parse("1/2*^5|6"),
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, 1, -1).unwrap(),
            NamedPitch {
                name: "e%".into(),
                base_factor: Pitch::must_parse("^1|4"),
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, -1, 1).unwrap(),
            NamedPitch {
                name: "a,".into(),
                base_factor: Pitch::must_parse("^-1|4"),
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, 3, 1).unwrap(),
            NamedPitch {
                name: "f'".into(),
                base_factor: Pitch::must_parse("2*^5|12"),
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, 5, 1).unwrap(),
            NamedPitch {
                name: "e%'2".into(),
                base_factor: Pitch::must_parse("4*^1|4"),
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, -2, -1).unwrap(),
            NamedPitch {
                name: "c,".into(),
                base_factor: Pitch::must_parse("1/2"),
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, -3, 1).unwrap(),
            NamedPitch {
                name: "b,2".into(),
                base_factor: Pitch::must_parse("1/2*^-1|12"),
            }
        );
    }

    #[test]
    fn test_manual() {
        let layout = LAYOUTS.layouts.get("l1").unwrap().read().unwrap();
        let mapping = &layout.mappings[0];
        assert_eq!(mapping.name, "m1");
        assert!(mapping.details.as_isomorphic().is_none());
        let mm = mapping.details.as_manual().unwrap();
        assert_eq!(
            mm.note_at_anchor_delta(0, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("e#"),
                base_factor: Pitch::must_parse("^7|19"),
            }
        );
        assert!(mm.note_at_anchor_delta(0, -2).is_none());
        assert!(mm.note_at_anchor_delta(1, 2).is_none());
        assert!(mm.note_at_anchor_delta(4, 7).is_none());
        assert!(mm.note_at_anchor_delta(-2, -3).is_none());
        assert_eq!(
            mm.note_at_anchor_delta(0, 1).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("f"),
                base_factor: Pitch::must_parse("^8|19"),
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(1, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("g#"),
                base_factor: Pitch::must_parse("^12|19"),
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(1, 1).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("c'"),
                base_factor: Pitch::must_parse("2"),
            }
        );
        // Go up one vertical repetition
        assert_eq!(
            mm.note_at_anchor_delta(4, 1).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("c'"),
                base_factor: Pitch::must_parse("2*^1|2"),
            }
        );
        // Go right one horizontal repetition
        assert_eq!(
            mm.note_at_anchor_delta(4, 6).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("c'"),
                base_factor: Pitch::must_parse("2*^1|2*1.5"),
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(-1, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("d%"),
                base_factor: Pitch::must_parse("^2|19"),
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(-1, 1).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("c2"),
                base_factor: Pitch::must_parse("2"),
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(-1, 2).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("q"),
                base_factor: Pitch::must_parse("^21|20"),
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(-2, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("g#"),
                base_factor: Pitch::must_parse("0.5*^12|19*^1|2"),
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(-5, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("g#"),
                base_factor: Pitch::must_parse("0.5*^12|19"),
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(-2, -6).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("g,"),
                base_factor: Pitch::must_parse("1/6*^11|19*^1|2"),
            }
        );
    }

    #[test]
    fn test_layout() {
        // The l1 layout places the m1 manual mapping so that it occupies columns 2..=11 and rows
        // 4..=9. The rest of the board is unmapped. We can't tell the difference between an
        // unmapped key within the mapped region and something that falls outside the map. That
        // comes later.
        let layout = LAYOUTS.layouts.get("l1").unwrap().read().unwrap();
        // Start with the anchor. The answer takes shifting into consideration. The anchor in the
        // mapping is "7".
        let r = layout
            .note_at_location(Coordinate { row: 5, col: 4 })
            .unwrap();
        assert_eq!(r.name, "e#");
        assert_eq!(r.pitch, Pitch::must_parse("400*^7|19"));
        // Shift one row up. This means we should see the note below this.
        assert!(layout.shift(Coordinate { row: 4, col: 4 }, Coordinate { row: 5, col: 4 }));
        let r = layout
            .note_at_location(Coordinate { row: 5, col: 4 })
            .unwrap();
        assert_eq!(r.name, "d%");
        assert_eq!(r.pitch, Pitch::must_parse("400*^2|19"));
        // // Shift one row left. This means we should see the note to the right of that.
        assert!(layout.shift(Coordinate { row: 5, col: 5 }, Coordinate { row: 5, col: 4 }));
        let r = layout
            .note_at_location(Coordinate { row: 5, col: 4 })
            .unwrap();
        assert_eq!(r.name, "c2");
        assert_eq!(r.pitch, Pitch::must_parse("800"));
        // Three rows above this should be the same note, but the pitch is up by v_factor.
        let r = layout
            .note_at_location(Coordinate { row: 8, col: 4 })
            .unwrap();
        assert_eq!(r.name, "c2");
        assert_eq!(r.pitch, Pitch::must_parse("800*^1|2"));
        // Five rows to the right should be the same note, but the pitch is up by h_factor.
        let r = layout
            .note_at_location(Coordinate { row: 8, col: 9 })
            .unwrap();
        assert_eq!(r.name, "c2");
        assert_eq!(r.pitch, Pitch::must_parse("1200*^1|2"));
        // Up one row brings us to the top row in the region.
        let r = layout
            .note_at_location(Coordinate { row: 9, col: 9 })
            .unwrap();
        assert_eq!(r.name, "f");
        assert_eq!(r.pitch, Pitch::must_parse("600*^8|19*^1|2"));
        // Three rows to the left is unmapped explicitly.
        let r = layout.note_at_location(Coordinate { row: 9, col: 6 });
        assert!(r.is_none());
        // One row from the previous mapped note puts us outside the region.
        let r = layout.note_at_location(Coordinate { row: 10, col: 9 });
        assert!(r.is_none());
        // Can't shift from location not in a mapping
        assert!(!layout.shift(
            Coordinate { row: 12, col: 12 },
            Coordinate { row: 4, col: 4 }
        ));
        // Can't transpose an unmapped key.
        assert!(!layout.transpose(&Pitch::must_parse("500"), Coordinate { row: 9, col: 6 }));
        assert!(!layout.transpose(&Pitch::must_parse("500"), Coordinate { row: 10, col: 9 }));

        // Now switch to layout l2, which adds another mapping that fills the whole area.
        // Save the offsets we applied to the mapping so we can replicate.
        let offsets = layout.mappings[0].offsets.write().unwrap().clone();
        let layout = LAYOUTS.layouts.get("l2").unwrap().read().unwrap();
        // Re-establish the shift for the first mapping to match above.
        *layout.mappings[0].offsets.write().unwrap() = offsets;
        // Apply a transposition the second mapping. Effective base pitch is 450.
        assert!(layout.transpose(&Pitch::must_parse("450"), Coordinate { row: 10, col: 7 }));
        // Same as before: this is at the top of the first region.
        let r = layout
            .note_at_location(Coordinate { row: 9, col: 9 })
            .unwrap();
        assert_eq!(r.name, "f");
        assert_eq!(r.pitch, Pitch::must_parse("600*^8|19*^1|2"));
        // Same as before: this is explicitly unmapped.
        let r = layout.note_at_location(Coordinate { row: 9, col: 6 });
        assert!(r.is_none());
        // This time, we fall into the second mapping. This is two characters to the right of
        // the anchor, which is four scale degrees. We are also transposing by 1.5.
        let r = layout
            .note_at_location(Coordinate { row: 10, col: 9 })
            .unwrap();
        assert_eq!(r.name, "e");
        assert_eq!(r.pitch, Pitch::must_parse("450*^1|3"));
        // Can't shift across layouts
        assert!(!layout.shift(
            Coordinate { row: 4, col: 4 },
            Coordinate { row: 12, col: 12 }
        ));
    }
}
