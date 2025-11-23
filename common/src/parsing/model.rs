use crate::pitch::Pitch;
use num_rational::Ratio;
use serde::{Serialize, Serializer};
use std::borrow::Cow;
use std::env;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Index;
use std::ops::Range;
use std::sync::LazyLock;
use to_static_derive::ToStatic;

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy, ToStatic)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
impl Serialize for Span {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        [self.start, self.end].serialize(serializer)
    }
}
impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{})", self.start, self.end)
    }
}
impl Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.start..index.end]
    }
}
impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}
impl From<Span> for Range<usize> {
    fn from(value: Span) -> Self {
        value.start..value.end
    }
}
impl Span {
    pub fn relative_to(&self, other: Span) -> Span {
        Span {
            start: self.start - other.start,
            end: self.end - other.start,
        }
    }
}

macro_rules! color {
    ($f:expr, $color:literal, $( $rest:tt )* ) => {
        {
            if *crate::USE_COLOR {
                write!($f, "\x1b[38;5;{}m", $color)?;
            }
            write!($f, $($rest)*)?;
            if *crate::USE_COLOR {
                write!($f, "\x1b[0m")?;
            }
            Ok(())
        }
    };
}

#[derive(Serialize, Debug, Clone)]
pub struct Token<'s, T: Debug + Serialize> {
    pub(crate) raw: &'s str,
    pub(crate) t: T,
}
impl<'s, T: Debug + Display + Serialize> Display for Token<'s, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let raw: String = self
            .raw
            .chars()
            .map(|c| {
                match c {
                    '\n' => '⏎', // also ␊
                    '\r' => '␍',
                    _ => c,
                }
            })
            .collect();
        write!(f, "{} ", self.t)?;
        color!(f, 248, "raw=⟨{raw}⟩")
    }
}
impl<'s, T: Debug + Serialize> Token<'s, T> {
    pub fn new_spanned(raw: &'s str, span: impl Into<Span>, t: T) -> Spanned<Self> {
        Spanned::new(span, Self { raw, t })
    }
}
impl<'s, T: Debug + Serialize + Copy> Copy for Token<'s, T> {}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spanned<T: Debug + Serialize> {
    pub span: Span,
    pub value: T,
}
impl<T: Debug + Display + Serialize> Display for Spanned<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        color!(f, 5, "{}:", self.span)?;
        write!(f, "{}", self.value)
    }
}
impl<T: Debug + Copy + Serialize> Copy for Spanned<T> {}

impl<T: Debug + Serialize> Spanned<T> {
    pub fn new(span: impl Into<Span>, value: impl Into<T>) -> Self {
        Self {
            span: span.into(),
            value: value.into(),
        }
    }
    pub fn value(self) -> T {
        self.value
    }

    pub fn as_ref<U: Debug + Serialize + ?Sized>(&self) -> Spanned<&U>
    where
        T: AsRef<U>,
    {
        Spanned::<&U> {
            span: self.span,
            value: self.value.as_ref(),
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
/// Represents a pitch. All ratios also parse into pitches. If something that parsed into
/// a pitch was originally specified as a ratio, you can get the value as a ratio. During the
/// lexical phase, we never know whether a ratio is supposed to be a ratio or a pitch. During the
/// semantic phase, if a pitch was provided when a ratio was wanted, we can give an error at that
/// time.
pub enum PitchOrNumber {
    Integer((u32, Pitch)),
    Ratio((Ratio<u32>, Pitch)),
    Pitch(Pitch),
}
impl Display for PitchOrNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        color!(f, 6, "{}", self.as_pitch())
    }
}

impl PitchOrNumber {
    pub fn as_pitch(&self) -> &Pitch {
        match self {
            PitchOrNumber::Integer((_, p)) => p,
            PitchOrNumber::Ratio((_, p)) => p,
            PitchOrNumber::Pitch(p) => p,
        }
    }

    pub fn into_pitch(self) -> Pitch {
        match self {
            PitchOrNumber::Integer((_, p)) => p,
            PitchOrNumber::Ratio((_, p)) => p,
            PitchOrNumber::Pitch(p) => p,
        }
    }

    pub fn try_as_ratio(&self) -> Option<Ratio<u32>> {
        match self {
            PitchOrNumber::Integer((i, _)) => Some(Ratio::from_integer(*i)),
            PitchOrNumber::Ratio((r, _)) => Some(*r),
            PitchOrNumber::Pitch(_) => None,
        }
    }

    pub fn try_as_int(&self) -> Option<u32> {
        match self {
            PitchOrNumber::Integer((i, _)) => Some(*i),
            _ => None,
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum ParamValue<'s> {
    Zero,
    PitchOrNumber(PitchOrNumber),
    String(Cow<'s, str>),
}
impl<'s> Display for ParamValue<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParamValue::Zero => color!(f, 6, "0"),
            ParamValue::PitchOrNumber(pr) => Display::fmt(pr, f),
            ParamValue::String(s) => color!(f, 166, "\"{s}\""),
        }
    }
}

impl<'s> ParamValue<'s> {
    pub fn try_as_pitch(&self) -> Option<&Pitch> {
        match self {
            ParamValue::Zero => None,
            ParamValue::PitchOrNumber(pr) => Some(pr.as_pitch()),
            ParamValue::String(_) => None,
        }
    }

    pub fn try_as_ratio(&self) -> Option<Ratio<u32>> {
        match self {
            ParamValue::Zero => None, // 0 can be represented as a ratio, but we don't allow it
            ParamValue::PitchOrNumber(pr) => pr.try_as_ratio(),
            ParamValue::String(_) => None,
        }
    }

    pub fn try_as_int(&self) -> Option<u32> {
        match self {
            ParamValue::Zero => Some(0),
            ParamValue::PitchOrNumber(pr) => pr.try_as_int(),
            ParamValue::String(_) => None,
        }
    }

    pub fn try_as_string(&self) -> Option<&Cow<'s, str>> {
        match self {
            ParamValue::Zero => None,
            ParamValue::PitchOrNumber(_r) => None,
            ParamValue::String(s) => Some(s),
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Param<'s> {
    pub key: Spanned<&'s str>,
    pub value: Spanned<ParamValue<'s>>,
}
impl<'s> Display for Param<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        color!(f, 88, "{}", self.key.value,)?;
        color!(f, 55, "=")?;
        write!(f, "{}", self.value.value)
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum DataBlock<'s> {
    Scale(ScaleBlock<'s>),
    Layout(LayoutBlock<'s>),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct RawDirective<'s> {
    pub name: Spanned<&'s str>,
    pub params: Vec<Param<'s>>,
    #[serde(skip_serializing_if = "Option::is_none")] // omit if None
    pub block: Option<Spanned<DataBlock<'s>>>,
}
impl<'s> Display for RawDirective<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        color!(f, 39, "{}(", self.name.value)?;
        let mut first = true;
        for p in &self.params {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(f, "{p}")?;
        }
        color!(f, 39, ")")?;
        if let Some(block) = &self.block {
            match &block.value {
                DataBlock::Scale(x) => write!(f, "{x}")?,
                DataBlock::Layout(x) => write!(f, "{x}")?,
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct DynamicLeader<'s> {
    pub name: Spanned<&'s str>,
}
impl<'s> Display for DynamicLeader<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        color!(f, 98, "{}", self.name.value)?;
        write!(f, "]")
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct NoteLeader<'s> {
    pub name: Spanned<&'s str>,
    pub note: Spanned<u32>,
}
impl<'s> Display for NoteLeader<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        color!(f, 98, "{}.{}", self.name.value, self.note.value)?;
        write!(f, "]")
    }
}

#[derive(Serialize, Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum Articulation {
    Accent,
    Marcato,
    Shorten,
}
impl Display for Articulation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Articulation::Accent => write!(f, ">"),
            Articulation::Marcato => write!(f, "^"),
            Articulation::Shorten => write!(f, "."),
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct RegularNote<'s> {
    pub duration: Option<Spanned<Ratio<u32>>>,
    #[serde(flatten)]
    pub note: NoteOctave<'s>,
    pub articulation: Vec<Spanned<Articulation>>,
    pub sustained: Option<Span>,
}
impl<'s> Display for RegularNote<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(x) = self.duration {
            color!(f, 3, "{}", x.value)?;
            color!(f, 55, ":")?;
        }
        write!(f, "{}", self.note)?;
        if !self.articulation.is_empty() {
            color!(f, 55, "(")?;
            for i in &self.articulation {
                color!(f, 4, "{}", i.value)?;
            }
            color!(f, 55, ")")?;
        }
        if self.sustained.is_some() {
            color!(f, 4, "~")?;
        }
        Ok(())
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Hold {
    pub duration: Option<Spanned<Ratio<u32>>>,
    pub ch: Spanned<char>,
}
impl Display for Hold {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "~{}",
            match self.duration {
                None => "".to_string(),
                Some(d) => d.value.to_string(),
            }
        )
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum Note<'s> {
    Regular(RegularNote<'s>),
    Hold(Hold),
    BarCheck(Span),
}
impl<'s> Display for Note<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Note::Regular(x) => write!(f, "{x}"),
            Note::Hold(x) => write!(f, "{x}"),
            Note::BarCheck(_) => write!(f, "|"),
        }
    }
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub enum DynamicChange {
    Crescendo,
    Diminuendo,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct RegularDynamic {
    pub level: Spanned<u8>,
    pub change: Option<Spanned<DynamicChange>>,
    pub position: Spanned<Ratio<u32>>,
}
impl Display for RegularDynamic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        color!(f, 2, "{}", self.level.value)?;
        color!(f, 55, "@")?;
        color!(f, 3, "{}", self.position.value)?;
        color!(
            f,
            4,
            "{}",
            match self.change {
                None => "",
                Some(d) => match d.value {
                    DynamicChange::Crescendo => "<",
                    DynamicChange::Diminuendo => ">",
                },
            }
        )
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum Dynamic {
    Regular(RegularDynamic),
    BarCheck(Span),
}
impl Display for Dynamic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Dynamic::Regular(d) => write!(f, "{d}"),
            Dynamic::BarCheck(_) => write!(f, "|"),
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct DynamicLine<'s> {
    pub leader: Spanned<DynamicLeader<'s>>,
    pub dynamics: Vec<Spanned<Dynamic>>,
}
impl<'s> Display for DynamicLine<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.leader.value)?;
        for i in &self.dynamics {
            write!(f, " {}", i.value)?;
        }
        Ok(())
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct NoteLine<'s> {
    pub leader: Spanned<NoteLeader<'s>>,
    pub notes: Vec<Spanned<Note<'s>>>,
}
impl<'s> Display for NoteLine<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.leader.value)?;
        for i in &self.notes {
            write!(f, " {}", i.value)?;
        }
        Ok(())
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ScaleNote<'s> {
    pub pitch: Spanned<PitchOrNumber>,
    pub note_names: Vec<Spanned<&'s str>>,
}
impl<'s> Display for ScaleNote<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.pitch.value, f)?;
        for name in &self.note_names {
            color!(f, 2, " {}", name.value)?;
        }
        Ok(())
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ScaleBlock<'s> {
    pub notes: Spanned<Vec<Spanned<ScaleNote<'s>>>>,
}
impl<'s> Display for ScaleBlock<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<<(scale)|")?;
        for n in &self.notes.value {
            write!(f, "{}|", &n.value)?;
        }
        write!(f, ">>")
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct NoteOctave<'s> {
    pub name: Spanned<&'s str>,
    pub octave: Option<Spanned<i8>>,
}
impl<'s> Display for NoteOctave<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        color!(f, 2, "{}", self.name.value)?;
        if let Some(x) = self.octave {
            match x.value {
                x if x < 0 => color!(f, 4, ",{}", -x)?,
                x => color!(f, 4, "'{x}")?,
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum LayoutItemType<'s> {
    Note(Spanned<NoteOctave<'s>>),
    Empty(Span),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct LayoutItem<'s> {
    pub item: LayoutItemType<'s>,
    pub is_anchor: Option<Span>,
}
impl<'s> Display for LayoutItem<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_anchor.is_some() {
            write!(f, "@")?;
        }
        match &self.item {
            LayoutItemType::Note(n) => write!(f, "{n}"),
            LayoutItemType::Empty(_) => write!(f, "~"),
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct LayoutBlock<'s> {
    pub rows: Spanned<Vec<Spanned<Vec<Spanned<LayoutItem<'s>>>>>>,
}
impl<'s> Display for LayoutBlock<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<<(layout)|")?;
        for row in &self.rows.value {
            let mut first = true;
            for n in &row.value {
                if first {
                    first = false;
                } else {
                    write!(f, " ")?;
                }
                write!(f, "{}", &n.value)?;
            }
            write!(f, "|")?;
        }
        write!(f, ">>")
    }
}

pub fn trace(msg: impl Display) {
    static TRACING: LazyLock<bool> = LazyLock::new(|| env::var("SYNTONIQ_TRACE_LEXER").is_ok());
    if *TRACING {
        eprintln!("{msg}");
    }
}

/// Helper for merge_option_spans
pub trait GetSpan {
    fn get_span(&self) -> Option<Span>;
}

impl<T: Debug + Serialize> GetSpan for Spanned<T> {
    fn get_span(&self) -> Option<Span> {
        Some(self.span)
    }
}

impl<T: Debug + Serialize> GetSpan for Option<Spanned<T>> {
    fn get_span(&self) -> Option<Span> {
        self.as_ref().map(|x| x.span)
    }
}

impl<T: Debug + Serialize> GetSpan for &[Spanned<T>] {
    fn get_span(&self) -> Option<Span> {
        if self.is_empty() {
            return None;
        }
        Some((self[0].span.start..self[self.len() - 1].span.end).into())
    }
}

impl<T: Debug + Serialize> GetSpan for Option<Vec<Spanned<T>>> {
    fn get_span(&self) -> Option<Span> {
        let s = self.as_ref()?.as_slice();
        s.get_span()
    }
}

/// Return a span that covers the range of all these spans assuming they are sorted.
pub(crate) fn merge_spans(spans: &[Option<Span>]) -> Option<Span> {
    let first = spans.iter().find(|x| x.is_some())?.as_ref()?;
    let last = spans.iter().rfind(|x| x.is_some())?.as_ref()?;
    Some((first.start..last.end).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_range() {
        let r = 1..5;
        assert_eq!(Range::from(Span::from(r.clone())), r);
        let s: Span = r.into();
        assert_eq!(Span::from(Range::from(s)), s);
    }

    #[test]
    fn test_merge_spans() {
        assert_eq!(
            GetSpan::get_span(&Some(vec![
                Spanned::<i32>::new(0..4, 3),
                Spanned::new(5..8, 6),
            ]))
            .unwrap(),
            (0..8).into()
        );
        assert!(merge_spans(&[]).is_none());
        assert!(merge_spans(&[None]).is_none());
        assert_eq!(
            merge_spans(&[None, Some((1..2).into()), None, Some((3..4).into()), None]).unwrap(),
            (1..4).into()
        );
    }

    #[test]
    fn test_param_value() {
        let pv = ParamValue::String("a".into());
        assert!(pv.try_as_int().is_none());
        assert!(pv.try_as_ratio().is_none());
        assert!(pv.try_as_pitch().is_none());
        assert_eq!(pv.try_as_string().unwrap(), "a");
        let pv = ParamValue::PitchOrNumber(PitchOrNumber::Integer((12, Pitch::must_parse("12"))));
        assert_eq!(pv.try_as_int().unwrap(), 12);
        assert_eq!(pv.try_as_ratio().unwrap(), Ratio::from_integer(12));
        assert_eq!(*pv.try_as_pitch().unwrap(), Pitch::must_parse("12"));
        assert!(pv.try_as_string().is_none());
        let pv = ParamValue::Zero;
        assert_eq!(pv.try_as_int().unwrap(), 0);
        assert!(pv.try_as_ratio().is_none());
        assert!(pv.try_as_pitch().is_none());
        assert!(pv.try_as_string().is_none());
    }
}
