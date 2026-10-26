//! Problems found while reading link text.

use std::fmt;

use crate::codec::MAX_BARS;

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
    /// A bar could not be read and was replaced with 4/4 in 16ths.
    BarReplaced(ParseError),
    /// Bars past the 512th were dropped.
    TooManyBars,
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
            Issue::BarReplaced(e) => write!(f, "{e}; replaced with 4/4 in 16ths"),
            Issue::TooManyBars => {
                write!(f, "more than {MAX_BARS} bars; kept the first {MAX_BARS}")
            }
            Issue::UnknownField { key } => write!(f, "unknown field '{key}' ignored"),
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
        used: usize,
        /// The meter's numerator.
        num: u32,
        /// The meter's denominator.
        den: u32,
        /// How many beat groups the bar has.
        groups: usize,
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
            ParseError::UnexpectedInBar { ch, bar } => {
                write!(f, "unexpected '{ch}' in bar '{bar}'")
            }
            ParseError::ExpectedNumber { at, text } => {
                write!(f, "expected a number at {at} in '{text}'")
            }
            ParseError::SplitOutOfRange { n } => write!(f, "a split of {n} is out of range"),
            ParseError::TooManySlots { slots } => {
                write!(f, "{slots} slots in one bar is too many")
            }
        }
    }
}
