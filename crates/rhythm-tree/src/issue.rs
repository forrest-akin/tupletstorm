//! Problems found while reading link text.

use std::fmt;

use crate::codec::MAX_BARS;
use crate::node::MAX_DEPTH;

/// How serious an issue is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    /// Something was replaced.
    Error,
    /// Something was ignored or adjusted.
    Warn,
}

/// A problem `decode` found and worked around.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Issue {
    /// The input has no v2 groove and no Groove Scribe fields.
    NoGroove,
    /// The input is a Groove Scribe link, which is not imported.
    GrooveScribeNotImported,
    /// The tempo is not a number from 10 to 999; 80 was used.
    TempoOutOfRange {
        /// The field's value, as the link has it.
        raw: String,
    },
    /// The swing is not a number from 0 to 100; 0 was used.
    SwingOutOfRange {
        /// The field's value, as the link has it.
        raw: String,
    },
    /// A bar could not be read and was replaced with 4/4 in 16ths.
    BarReplaced(ParseError),
    /// A bar is written to repeat 0 times; it was kept once.
    RepeatZero {
        /// The bar item, as the link has it.
        spec: String,
    },
    /// Bars past the 512th were dropped.
    TooManyBars,
    /// A lane the kit doesn't have was kept, with any letters.
    CustomLane {
        /// The lane's key.
        lane: String,
    },
    /// A lane has hits for more bars than the groove has; the extra bars were dropped.
    ExtraBars {
        /// The lane's key.
        lane: String,
        /// How many bars of hits the link has.
        parts: usize,
        /// How many bars the groove has.
        bars: usize,
    },
    /// Letters a lane doesn't have were made rests.
    BadLetters {
        /// The lane's key.
        lane: String,
        /// The bar, from 0.
        bar: usize,
        /// The letters, in order.
        letters: String,
    },
    /// A bar of a lane had more or fewer hits than the bar has slots.
    HitCount {
        /// The lane's key.
        lane: String,
        /// The bar, from 0.
        bar: usize,
        /// How many hits the link has.
        hits: usize,
        /// How many slots the bar has.
        slots: usize,
        /// What was done about it.
        fit: Fit,
    },
    /// A field the format doesn't have was ignored.
    UnknownField {
        /// The field's key.
        key: String,
    },
}

impl Issue {
    /// How serious the issue is.
    pub fn level(&self) -> Level {
        match self {
            Issue::BarReplaced(_) => Level::Error,
            _ => Level::Warn,
        }
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Issue::NoGroove => f.write_str("no v=2 and no Groove Scribe fields; starting empty"),
            Issue::GrooveScribeNotImported => {
                f.write_str("Groove Scribe link not imported; starting empty")
            }
            Issue::TempoOutOfRange { raw } => {
                write!(f, "tempo '{raw}' out of range; using 80")
            }
            Issue::SwingOutOfRange { raw } => {
                write!(f, "swing '{raw}' out of range; using 0")
            }
            Issue::BarReplaced(e) => write!(f, "{e}; replaced with 4/4 in 16ths"),
            Issue::RepeatZero { spec } => write!(f, "bar '{spec}' repeats 0 times; kept once"),
            Issue::TooManyBars => {
                write!(f, "more than {MAX_BARS} bars; kept the first {MAX_BARS}")
            }
            Issue::CustomLane { lane } => write!(f, "custom lane '{lane}' kept as-is"),
            Issue::ExtraBars { lane, parts, bars } => {
                write!(f, "{lane}: hits for {parts} bars, groove has {bars}")
            }
            Issue::BadLetters { lane, bar, letters } => {
                write!(f, "{lane} bar {bar}: '{letters}' not valid; made rests")
            }
            Issue::HitCount {
                lane,
                bar,
                hits,
                slots,
                fit,
            } => write!(f, "{lane} bar {bar}: {hits} hits for {slots} slots; {fit}"),
            Issue::UnknownField { key } => write!(f, "unknown field '{key}' ignored"),
        }
    }
}

/// How a bar of hits was fitted to its slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    /// Rests were added at the end.
    Padded,
    /// Hits past the last slot were dropped.
    Cut,
}

impl fmt::Display for Fit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Fit::Padded => "padded",
            Fit::Cut => "cut",
        })
    }
}

/// Why an edit to a score was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditError {
    /// The key can't name a lane.
    UnknownLane {
        /// The key.
        lane: String,
    },
    /// The score has no bar at this index.
    NoSuchBar {
        /// The bar, from 0.
        bar: usize,
    },
    /// The bar has no leaf at this index.
    NoSuchSlot {
        /// The lane's key.
        lane: String,
        /// The bar, from 0.
        bar: usize,
        /// The leaf, from 0.
        leaf: usize,
    },
    /// The lane has no such letter.
    NotALetter {
        /// The lane's key.
        lane: String,
        /// The letter.
        letter: char,
    },
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditError::UnknownLane { lane } => write!(f, "no lane '{lane}'"),
            EditError::NoSuchBar { bar } => write!(f, "bar {bar} doesn't exist"),
            EditError::NoSuchSlot { lane, bar, leaf } => {
                write!(f, "{lane} bar {bar} has no slot {leaf}")
            }
            EditError::NotALetter { lane, letter } => {
                write!(f, "'{letter}' is not a {lane} letter")
            }
        }
    }
}

/// Why a piece of link text could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// The text does not have the shape of a bar.
    NotABar {
        /// The text.
        spec: String,
    },
    /// The meter's numerator is outside 1–64.
    NumeratorOutOfRange {
        /// The numerator.
        num: u32,
        /// The denominator.
        den: u32,
    },
    /// The meter's denominator is not 1, 2, 4, 8, 16 or 32.
    DenominatorOutOfRange {
        /// The numerator.
        num: u32,
        /// The denominator.
        den: u32,
    },
    /// The segments don't cover the bar's beat groups.
    SegmentsCover {
        /// How many beat groups the segments cover.
        used: u64,
        /// The meter's numerator.
        num: u32,
        /// The meter's denominator.
        den: u32,
        /// How many beat groups the bar has.
        groups: usize,
    },
    /// A bar's beat grouping doesn't sum to its numerator.
    GroupingDoesntAddUp {
        /// The grouping's parts.
        groups: Vec<u32>,
        /// The meter's numerator.
        num: u32,
    },
    /// A bar's segments are followed by text that isn't a segment.
    UnexpectedInBar {
        /// The first character that couldn't be read.
        ch: char,
        /// The bar.
        bar: String,
    },
    /// A number was expected and something else was found.
    ExpectedNumber {
        /// Byte offset into `text`.
        at: usize,
        /// The text being read.
        text: String,
    },
    /// A split is outside 1–64.
    SplitOutOfRange {
        /// The split.
        n: u32,
    },
    /// A bracket ratio's second number is outside 1–64.
    RatioOutOfRange {
        /// The split.
        n: u32,
        /// The ratio's second number.
        m: u32,
    },
    /// Nodes nest more than 6 levels below a segment.
    TooDeep,
    /// A node's weight is 0.
    WeightZero,
    /// A node's children are not followed by `)`.
    ExpectedCloseParen {
        /// Byte offset into `text`.
        at: usize,
        /// The text being read.
        text: String,
    },
    /// A node's children fill more or fewer slots than it splits into.
    KidsDontFill {
        /// The split.
        n: u32,
        /// How many slots the children's weights fill.
        filled: u64,
    },
    /// A bar has more than 512 leaves.
    TooManySlots {
        /// How many leaves the bar has.
        slots: u64,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::NotABar { spec } => {
                write!(f, "'{spec}' is not a bar (expected like 4/4:4)")
            }
            ParseError::NumeratorOutOfRange { num, den } => {
                write!(f, "meter {num}/{den}: numerator out of range")
            }
            ParseError::DenominatorOutOfRange { num, den } => {
                write!(f, "meter {num}/{den}: denominator out of range")
            }
            ParseError::SegmentsCover {
                used,
                num,
                den,
                groups,
            } => write!(
                f,
                "segments cover {used} beat groups; {num}/{den} has {groups}"
            ),
            ParseError::GroupingDoesntAddUp { groups, num } => {
                let parts: Vec<String> = groups.iter().map(u32::to_string).collect();
                write!(f, "grouping {} doesn't add up to {num}", parts.join("+"))
            }
            ParseError::UnexpectedInBar { ch, bar } => {
                write!(f, "unexpected '{ch}' in bar '{bar}'")
            }
            ParseError::ExpectedNumber { at, text } => {
                write!(f, "expected a number at {at} in '{text}'")
            }
            ParseError::SplitOutOfRange { n } => write!(f, "a split of {n} is out of range"),
            ParseError::RatioOutOfRange { n, m } => write!(f, "ratio {n}:{m}"),
            ParseError::TooDeep => write!(f, "nested deeper than {MAX_DEPTH} levels"),
            ParseError::WeightZero => f.write_str("weight must be at least 1"),
            ParseError::ExpectedCloseParen { at, text } => {
                write!(f, "expected ')' at {at} in '{text}'")
            }
            ParseError::KidsDontFill { n, filled } => {
                write!(f, "children of {n}(…) fill {filled} slots, not {n}")
            }
            ParseError::TooManySlots { slots } => {
                write!(f, "{slots} slots in one bar is too many")
            }
        }
    }
}
