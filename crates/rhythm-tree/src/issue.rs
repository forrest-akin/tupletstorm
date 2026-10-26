//! Problems found while reading link text.

use std::fmt;

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
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::NotABar { spec } => {
                write!(f, "'{spec}' is not a bar (expected like 4/4:4)")
            }
        }
    }
}
