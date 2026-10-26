//! A groove: its bars.

use crate::bar::Bar;

/// A decoded groove.
#[derive(Clone, Debug, PartialEq)]
pub struct Score {
    bars: Vec<Bar>,
}

impl Score {
    pub(crate) fn new(bars: Vec<Bar>) -> Score {
        Score { bars }
    }

    /// The bars, in order.
    pub fn bars(&self) -> &[Bar] {
        &self.bars
    }
}
