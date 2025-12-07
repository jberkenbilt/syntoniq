use crate::parsing::score::{NamedPitch, Scale};
use crate::parsing::score_helpers;
use crate::pitch::Pitch;
use num_rational::Ratio;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, RwLock};
use to_static_derive::ToStatic;

#[derive(Serialize, Default, ToStatic)]
pub struct Layouts<'s> {
    pub scales: HashMap<Cow<'s, str>, Arc<Scale<'s>>>,
    pub layouts: Vec<Arc<Layout<'s>>>,
}

#[derive(Serialize, Default, ToStatic)]
pub struct Layout<'s> {
    pub name: Cow<'s, str>,
    pub keyboard: Cow<'s, str>,
    pub mappings: Vec<LayoutMapping<'s>>,
    /// Amount of stagger. This is set by the keyboard. Every `stagger` rows, columns are shifted
    /// to the right for manual mappings and region boundaries. This does not affect isomorphic
    /// layout, which must be uniform. Rectangular keyboards should keep this at 0. Hexagonal
    /// keyboards would typically set it to 2.
    #[serde(skip)]
    pub stagger: AtomicI32,
}
impl<'s> Layout<'s> {
    pub fn note_at_location(self: &Arc<Self>, location: Coordinate) -> Option<PlacedNote<'s>> {
        // Return information from the first mapping that has the note, if any.
        let stagger = self.stagger.load(Ordering::Relaxed);
        for m in &self.mappings {
            if let Some(r) = m.note_at_location(location, stagger) {
                return r;
            }
        }
        None
    }

    /// Shift the mapping so that the key at `from` moves to `to`. `from` and `to` must belong to
    /// the same mapping, but the keys don't have to be mapped within the mapping. The return value
    /// indicates whether the shift was successful.
    pub fn shift(&self, from: Coordinate, to: Coordinate) -> bool {
        let stagger = self.stagger.load(Ordering::Relaxed);
        for mapping in &self.mappings {
            if mapping.contains(from, stagger) {
                if !mapping.contains(to, stagger) {
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
    pub fn transpose(self: &Arc<Self>, pitch: &Pitch, location: Coordinate) -> bool {
        let stagger = self.stagger.load(Ordering::Relaxed);
        for mapping in &self.mappings {
            if let Some(Some(np)) = mapping.note_at_location(location, stagger) {
                let factor = pitch / &np.pitch;
                mapping.offsets.write().unwrap().transpose *= &factor;
                return true;
            }
        }
        false
    }

    /// Transpose all mappings up or down an octave. We use octave here rather than cycle because
    /// this applies to all mappings uniformly.
    pub fn octave_shift(&self, up: bool) {
        let transposition = if up {
            Pitch::from(Ratio::new(2, 1))
        } else {
            Pitch::from(Ratio::new(1, 2))
        };
        for m in &self.mappings {
            m.offsets.write().unwrap().transpose *= &transposition;
        }
    }

    /// Set the stagger. See field-level documentation. This is intended to be called by a keyboard
    /// when it accepts a layout.
    pub fn stagger(&self, stagger: i32) {
        self.stagger.store(stagger, Ordering::Relaxed);
    }
}

#[derive(Serialize, Debug, Clone, Copy, PartialOrd, PartialEq, Eq, Hash)]
pub struct Coordinate {
    pub row: i32,
    pub col: i32,
}

pub struct PlacedNote<'s> {
    /// Note name, including octave/cycle markers
    pub name: Cow<'s, str>,
    /// Scale the note came from
    pub scale: Arc<Scale<'s>>,
    /// Configured base pitch of the mapping
    pub scale_base: Pitch,
    /// Current transposition
    pub transposition: Pitch,
    /// Contribution of base factor from tiling of manual mapping
    pub tile_factor: Pitch,
    /// Final pitch
    pub pitch: Pitch,
    /// Normalized interval over the base pitch
    pub base_interval: Pitch,
    /// Normalized scale degree (from 0 to degrees-1)
    pub degree: u32,
    /// Whether this comes from an isomorphic layout
    pub isomorphic: bool,
}

#[derive(Default, Clone, ToStatic)]
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

#[derive(Serialize, ToStatic)]
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
    pub fn contains(&self, location: Coordinate, stagger: i32) -> bool {
        let min_row = self.rows_below.map(|x| self.anchor_row - x);
        let max_row = self.rows_above.map(|x| self.anchor_row + x);
        let stagger_offset = if stagger == 0 {
            0
        } else {
            (location.row - self.anchor_row).div_euclid(stagger)
        };
        // Min/max column in the row, adjusted for stagger.
        let min_col = self.cols_left.map(|x| self.anchor_col - x + stagger_offset);
        let max_col = self
            .cols_right
            .map(|x| self.anchor_col + x + stagger_offset);
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
    pub fn note_at_location(
        &self,
        location: Coordinate,
        stagger: i32,
    ) -> Option<Option<PlacedNote<'s>>> {
        if self.contains(location, stagger) {
            // Shift amounts apply to the layout, effectively moving the anchor by that amount.
            // loc - (anchor + shift) = loc - anchor - shift.
            let offsets = self.offsets.read().unwrap();
            let row_delta: i32 = location.row - self.anchor_row - offsets.shift_v;
            let col_delta: i32 = location.col - self.anchor_col - offsets.shift_h;
            Some(
                self.details
                    .note_at_anchor_delta(&self.scale, row_delta, col_delta, stagger)
                    .map(|x| {
                        let mut pitch = x.base_factor;
                        pitch *= &self.base_pitch;
                        pitch *= &offsets.transpose;
                        pitch *= &x.tile_factor;
                        PlacedNote {
                            name: x.name,
                            scale: self.scale.clone(),
                            scale_base: self.base_pitch.clone(),
                            transposition: offsets.transpose.clone(),
                            tile_factor: x.tile_factor,
                            pitch,
                            base_interval: x.base_interval,
                            degree: x.degree,
                            isomorphic: x.isomorphic,
                        }
                    }),
            )
        } else {
            // The mapping doesn't cover this coordinate.
            None
        }
    }
}

#[derive(Serialize, ToStatic)]
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
        stagger: i32,
    ) -> Option<NamedPitch<'s>> {
        match self {
            MappingDetails::Isomorphic(x) => x.note_at_anchor_delta(scale, row_delta, col_delta),
            MappingDetails::Manual(x) => x.note_at_anchor_delta(row_delta, col_delta, stagger),
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

#[derive(Serialize, ToStatic)]
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
        let base_interval = scale.pitches[pitch_idx as usize].clone();
        let base_factor = &base_interval * &Pitch::from(scale.definition.cycle.pow(cycle));
        let given_name = scale.primary_names[pitch_idx as usize].clone();
        let name = score_helpers::format_note_cycle(given_name, cycle);
        Some(NamedPitch {
            name,
            base_factor,
            base_interval,
            degree: pitch_idx as u32,
            tile_factor: Default::default(),
            isomorphic: true,
        })
    }
}
#[derive(Serialize, ToStatic)]
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
    fn note_at_anchor_delta(
        &self,
        row_delta: i32,
        col_delta: i32,
        stagger: i32,
    ) -> Option<NamedPitch<'s>> {
        let row = self.anchor_row + row_delta;
        let num_rows = self.notes.len() as i32;
        let num_cols = self.notes.first().expect("notes is empty").len() as i32;
        let row_idx = row.rem_euclid(num_rows);
        let v_repetitions = row.div_euclid(num_rows);
        let stagger_offset = if stagger == 0 {
            0
        } else {
            row_delta.div_euclid(stagger)
        };
        let col = self.anchor_col + col_delta - stagger_offset;
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
        named_pitch.tile_factor = factor;
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
        let layout = &LAYOUTS.layouts[1];
        assert_eq!(layout.name, "l2");
        let mapping = &layout.mappings[1];
        assert_eq!(mapping.name, "m2");
        assert!(mapping.details.as_manual().is_none());
        let im = mapping.details.as_isomorphic().unwrap();
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, 0, 0).unwrap(),
            NamedPitch {
                name: "c".into(),
                base_factor: Pitch::unit(),
                base_interval: Pitch::unit(),
                degree: 0,
                tile_factor: Pitch::unit(),
                isomorphic: true,
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, 0, 1).unwrap(),
            NamedPitch {
                name: "d".into(),
                base_factor: Pitch::must_parse("^1|6"),
                base_interval: Pitch::must_parse("^1|6"),
                degree: 2,
                tile_factor: Pitch::unit(),
                isomorphic: true,
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, 0, -1).unwrap(),
            NamedPitch {
                name: "b%,".into(),
                base_factor: Pitch::must_parse("1/2*^5|6"),
                base_interval: Pitch::must_parse("^5|6"),
                degree: 10,
                tile_factor: Pitch::unit(),
                isomorphic: true,
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, 1, -1).unwrap(),
            NamedPitch {
                name: "e%".into(),
                base_factor: Pitch::must_parse("^1|4"),
                base_interval: Pitch::must_parse("^1|4"),
                degree: 3,
                tile_factor: Pitch::unit(),
                isomorphic: true,
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, -1, 1).unwrap(),
            NamedPitch {
                name: "a,".into(),
                base_factor: Pitch::must_parse("^-1|4"),
                base_interval: Pitch::must_parse("^3|4"),
                degree: 9,
                tile_factor: Pitch::unit(),
                isomorphic: true,
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, 3, 1).unwrap(),
            NamedPitch {
                name: "f'".into(),
                base_factor: Pitch::must_parse("2*^5|12"),
                base_interval: Pitch::must_parse("^5|12"),
                degree: 5,
                tile_factor: Pitch::unit(),
                isomorphic: true,
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, 5, 1).unwrap(),
            NamedPitch {
                name: "e%'2".into(),
                base_factor: Pitch::must_parse("4*^1|4"),
                base_interval: Pitch::must_parse("^1|4"),
                degree: 3,
                tile_factor: Pitch::unit(),
                isomorphic: true,
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, -2, -1).unwrap(),
            NamedPitch {
                name: "c,".into(),
                base_factor: Pitch::must_parse("1/2"),
                base_interval: Pitch::must_parse("1"),
                degree: 0,
                tile_factor: Pitch::unit(),
                isomorphic: true,
            }
        );
        assert_eq!(
            im.note_at_anchor_delta(&mapping.scale, -3, 1).unwrap(),
            NamedPitch {
                name: "b,2".into(),
                base_factor: Pitch::must_parse("1/2*^-1|12"),
                base_interval: Pitch::must_parse("^11|12"),
                degree: 11,
                tile_factor: Pitch::unit(),
                isomorphic: true,
            }
        );
    }

    #[test]
    fn test_manual() {
        let layout = &LAYOUTS.layouts[0];
        assert_eq!(layout.name, "l1");
        let mapping = &layout.mappings[0];
        assert_eq!(mapping.name, "m1");
        assert!(mapping.details.as_isomorphic().is_none());
        let mm = mapping.details.as_manual().unwrap();
        assert_eq!(
            mm.note_at_anchor_delta(0, 0, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("e#"),
                base_factor: Pitch::must_parse("^7|19"),
                base_interval: Pitch::must_parse("^7|19"),
                degree: 8,
                tile_factor: Pitch::unit(),
                isomorphic: false,
            }
        );
        assert!(mm.note_at_anchor_delta(0, -2, 0).is_none());
        assert!(mm.note_at_anchor_delta(1, 2, 0).is_none());
        assert!(mm.note_at_anchor_delta(4, 7, 0).is_none());
        assert!(mm.note_at_anchor_delta(-2, -3, 0).is_none());
        // With staggered, every n rows up effectively shifts the column to the left because
        // things on the keyboard are shifted right. The only sensible values for stagger are 0
        // for rectangular keyboards and 2 for hexagonal keyboards, though one could conceive of
        // a hexagonal keyboard situated at 30 degrees and arranged so that every third row is
        // vertically aligned one column off. The arithmetic may feel inverted in this test.
        // Remember that the arguments here are anchor deltas. If the anchor is at 5, 6 and you
        // ask for 9, 6, the unstaggered column delta would be 0 (6 - 6). If there is a stagger of
        // 2, then row 7, which is 4 rows up, would have its columns 2 spaces to the right (4 / 2).
        // That means we need to *add* 2 to the requested delta so that we go two columns further
        // to the *left* when we ask the mapping what character would be that delta. In other words,
        // if we are working in a layout with a stagger value of 2, we have to go 2 steps farther
        // to the left, which *increases* our delta, to find the note in a lookup table that is
        // not staggered.
        assert!(mm.note_at_anchor_delta(0, -2, 2).is_none());
        assert!(mm.note_at_anchor_delta(1, 2, 2).is_none());
        assert!(mm.note_at_anchor_delta(4, 9, 2).is_none());
        assert!(mm.note_at_anchor_delta(-2, -4, 2).is_none());
        assert_eq!(
            mm.note_at_anchor_delta(0, 1, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("f"),
                base_factor: Pitch::must_parse("^8|19"),
                base_interval: Pitch::must_parse("^8|19"),
                degree: 9,
                tile_factor: Pitch::unit(),
                isomorphic: false,
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(1, 0, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("g#"),
                base_factor: Pitch::must_parse("^12|19"),
                base_interval: Pitch::must_parse("^12|19"),
                degree: 13,
                tile_factor: Pitch::unit(),
                isomorphic: false,
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(1, 1, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("c'"),
                base_factor: Pitch::must_parse("2"),
                base_interval: Pitch::must_parse("1"),
                degree: 0,
                tile_factor: Pitch::unit(),
                isomorphic: false,
            }
        );
        // Go up one vertical repetition
        assert_eq!(
            mm.note_at_anchor_delta(4, 1, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("c'"),
                base_factor: Pitch::must_parse("2"),
                base_interval: Pitch::must_parse("1"),
                degree: 0,
                tile_factor: Pitch::must_parse("^1|2"),
                isomorphic: false,
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(4, 3, 2).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("c'"),
                base_factor: Pitch::must_parse("2"),
                base_interval: Pitch::must_parse("1"),
                degree: 0,
                tile_factor: Pitch::must_parse("^1|2"),
                isomorphic: false,
            }
        );
        // Go right one horizontal repetition
        assert_eq!(
            mm.note_at_anchor_delta(4, 6, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("c'"),
                base_factor: Pitch::must_parse("2"),
                base_interval: Pitch::must_parse("1"),
                degree: 0,
                tile_factor: Pitch::must_parse("1.5*^1|2"),
                isomorphic: false,
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(-1, 0, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("d%"),
                base_factor: Pitch::must_parse("^2|19"),
                base_interval: Pitch::must_parse("^2|19"),
                degree: 3,
                tile_factor: Pitch::unit(),
                isomorphic: false,
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(-1, -1, 2).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("d%"),
                base_factor: Pitch::must_parse("^2|19"),
                base_interval: Pitch::must_parse("^2|19"),
                degree: 3,
                tile_factor: Pitch::unit(),
                isomorphic: false,
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(-1, 1, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("c2"),
                base_factor: Pitch::must_parse("2"),
                base_interval: Pitch::must_parse("1"),
                degree: 0,
                tile_factor: Pitch::unit(),
                isomorphic: false,
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(-1, 0, 2).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("c2"),
                base_factor: Pitch::must_parse("2"),
                base_interval: Pitch::must_parse("1"),
                degree: 0,
                tile_factor: Pitch::unit(),
                isomorphic: false,
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(-1, 2, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("q"),
                base_factor: Pitch::must_parse("^21|20"),
                base_interval: Pitch::must_parse("^1|20"),
                degree: 1,
                tile_factor: Pitch::unit(),
                isomorphic: false,
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(-2, 0, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("g#"),
                base_factor: Pitch::must_parse("^12|19"),
                base_interval: Pitch::must_parse("^12|19"),
                degree: 13,
                tile_factor: Pitch::must_parse("0.5*^1|2"),
                isomorphic: false,
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(-2, -1, 2).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("g#"),
                base_factor: Pitch::must_parse("^12|19"),
                base_interval: Pitch::must_parse("^12|19"),
                degree: 13,
                tile_factor: Pitch::must_parse("0.5*^1|2"),
                isomorphic: false,
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(-5, 0, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("g#"),
                base_factor: Pitch::must_parse("^12|19"),
                base_interval: Pitch::must_parse("^12|19"),
                degree: 13,
                tile_factor: Pitch::must_parse("0.5"),
                isomorphic: false,
            }
        );
        assert_eq!(
            mm.note_at_anchor_delta(-2, -6, 0).unwrap(),
            NamedPitch {
                name: Cow::Borrowed("g,"),
                base_factor: Pitch::must_parse("1/2*^11|19"),
                base_interval: Pitch::must_parse("^11|19"),
                degree: 12,
                tile_factor: Pitch::must_parse("1/3*^1|2"),
                isomorphic: false,
            }
        );
    }

    #[test]
    fn test_layout() {
        // The l1 layout places the m1 manual mapping so that it occupies columns 2..=11 and rows
        // 4..=9. The rest of the board is unmapped. We can't tell the difference between an
        // unmapped key within the mapped region and something that falls outside the map. That
        // comes later.
        let layout = &LAYOUTS.layouts[0];
        assert_eq!(layout.name, "l1");
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
        // Shift one row left. This means we should see the note to the right of that.
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
        // With a stagger of 2, we need to three rows and column to the right.
        layout.stagger(2);
        let r = layout
            .note_at_location(Coordinate { row: 8, col: 5 })
            .unwrap();
        assert_eq!(r.name, "c2");
        assert_eq!(r.pitch, Pitch::must_parse("800*^1|2"));
        layout.stagger(0);
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
        let layout = &LAYOUTS.layouts[1];
        assert_eq!(layout.name, "l2");

        // Before we re-establish the old offsets for replication, exercise region boundaries with
        // and without stagger. Column 11 is the right-most column in rows 4 to 9 inclusive. Check
        // the anchor first.
        let r = layout
            .note_at_location(Coordinate { row: 5, col: 4 })
            .unwrap();
        assert_eq!(r.name, "e#");
        assert_eq!(r.pitch, Pitch::must_parse("400*^7|19"));
        // The anchor is at 5, 4 so 4, 11 should be one row down and two columns plus one horizontal
        // repetition over.
        let r = layout
            .note_at_location(Coordinate { row: 4, col: 11 })
            .unwrap();
        assert_eq!(r.name, "q");
        assert_eq!(r.pitch, Pitch::must_parse("600*^21|20"));
        // Three rows above should be the same note one vertical repetition later.
        let r = layout
            .note_at_location(Coordinate { row: 7, col: 11 })
            .unwrap();
        assert_eq!(r.name, "q");
        assert_eq!(r.pitch, Pitch::must_parse("600*^21|20*^1|2"));
        // Four rows above is one row higher.
        let r = layout
            .note_at_location(Coordinate { row: 8, col: 11 })
            .unwrap();
        assert_eq!(r.name, "f#");
        assert_eq!(r.pitch, Pitch::must_parse("600*^9|19*^1|2"));
        // One column to the right should put us in the new mapping.
        let r = layout
            .note_at_location(Coordinate { row: 4, col: 12 })
            .unwrap();
        assert_eq!(r.name, "e,2");
        assert_eq!(r.pitch, Pitch::must_parse("75*^1|3"));
        // Three rows above should be 15 steps above in the new mapping.
        let r = layout
            .note_at_location(Coordinate { row: 7, col: 12 })
            .unwrap();
        assert_eq!(r.name, "g,");
        assert_eq!(r.pitch, Pitch::must_parse("150*^7|12"));
        // Four rows above is 5 more steps.
        let r = layout
            .note_at_location(Coordinate { row: 8, col: 12 })
            .unwrap();
        assert_eq!(r.name, "c");
        assert_eq!(r.pitch, Pitch::must_parse("300"));

        // Now exercise with stagger. We intentionally use even and odd numbers of rows to make sure
        // the boundary condition is correctly tested with even and odd offsets. For every two rows
        // we go above the anchor row (5), we need to shift the column to the right to get to the
        // same note. The anchor should not move.
        layout.stagger(2);
        let r = layout
            .note_at_location(Coordinate { row: 5, col: 4 })
            .unwrap();
        assert_eq!(r.name, "e#");
        assert_eq!(r.pitch, Pitch::must_parse("400*^7|19"));
        // What was at 4, 11 should now be at 4, 10.
        let r = layout
            .note_at_location(Coordinate { row: 4, col: 10 })
            .unwrap();
        assert_eq!(r.name, "q");
        assert_eq!(r.pitch, Pitch::must_parse("600*^21|20"));
        // What was at 7, 11 should now be at 7, 12.
        let r = layout
            .note_at_location(Coordinate { row: 7, col: 12 })
            .unwrap();
        assert_eq!(r.name, "q");
        assert_eq!(r.pitch, Pitch::must_parse("600*^21|20*^1|2"));
        // Going up one more row doesn't further shift the column since we are in the 2-row group
        // above the anchor row.
        let r = layout
            .note_at_location(Coordinate { row: 8, col: 12 })
            .unwrap();
        assert_eq!(r.name, "f#");
        assert_eq!(r.pitch, Pitch::must_parse("600*^9|19*^1|2"));
        // 4, 11 is now outside the region.
        let r = layout
            .note_at_location(Coordinate { row: 4, col: 11 })
            .unwrap();
        assert_eq!(r.name, "d,2");
        assert_eq!(r.pitch, Pitch::must_parse("75*^1|6"));
        // We need to go to 7, 13 to get to the new mapping now.
        let r = layout
            .note_at_location(Coordinate { row: 7, col: 13 })
            .unwrap();
        assert_eq!(r.name, "a,");
        assert_eq!(r.pitch, Pitch::must_parse("150*^3|4"));
        // This is in the new region now.
        let r = layout
            .note_at_location(Coordinate { row: 8, col: 13 })
            .unwrap();
        assert_eq!(r.name, "d");
        assert_eq!(r.pitch, Pitch::must_parse("300*^1|6"));
        // Reset the stagger for remaining tests.
        layout.stagger(0);

        // Re-establish the shift for the first mapping to match above. This intentionally repeats
        // some tests from before the stagger tests to ensure consistency.
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
        // Exercise octave transpositions. Check notes from both mappings.
        layout.octave_shift(true);
        let r = layout
            .note_at_location(Coordinate { row: 9, col: 9 })
            .unwrap();
        assert_eq!(r.name, "f");
        assert_eq!(r.pitch, Pitch::must_parse("1200*^8|19*^1|2"));
        let r = layout
            .note_at_location(Coordinate { row: 10, col: 9 })
            .unwrap();
        assert_eq!(r.name, "e");
        assert_eq!(r.pitch, Pitch::must_parse("900*^1|3"));
        layout.octave_shift(false);
        let r = layout
            .note_at_location(Coordinate { row: 9, col: 9 })
            .unwrap();
        assert_eq!(r.name, "f");
        assert_eq!(r.pitch, Pitch::must_parse("600*^8|19*^1|2"));
        let r = layout
            .note_at_location(Coordinate { row: 10, col: 9 })
            .unwrap();
        assert_eq!(r.name, "e");
        assert_eq!(r.pitch, Pitch::must_parse("450*^1|3"));
    }
}
