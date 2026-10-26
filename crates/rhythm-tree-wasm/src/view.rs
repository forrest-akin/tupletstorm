//! Plain data the browser receives, built by functions that run natively.

use rhythm_tree::{Bar, Issue, Leaf, Level, Rat, decode};
use serde::Serialize;

/// An issue found while opening a link.
#[derive(Debug, Serialize, PartialEq)]
pub struct IssueView {
    /// `"error"` or `"warn"`.
    pub level: &'static str,
    pub msg: String,
}

/// What opening a link gives the client.
#[derive(Debug, Serialize)]
pub struct Opened {
    pub issues: Vec<IssueView>,
}

/// A groove as the client draws it.
#[derive(Debug, Serialize)]
pub struct GrooveView {
    pub bars: Vec<BarView>,
}

/// One bar.
#[derive(Debug, Serialize)]
pub struct BarView {
    /// Like `7/8`.
    pub meter: String,
    /// Length in quarters.
    pub length: RatView,
    pub leaves: Vec<LeafView>,
}

/// A leaf slot of a bar.
#[derive(Debug, Serialize)]
pub struct LeafView {
    pub segment: usize,
    /// Onset in quarters from the top of the bar.
    pub start: RatView,
    /// Length in quarters.
    pub span: RatView,
}

/// An exact number of quarters.
#[derive(Debug, Serialize, PartialEq)]
pub struct RatView {
    pub num: i64,
    pub den: i64,
}

/// Opens a link or URL.
pub fn open_of(input: &str) -> Opened {
    Opened {
        issues: decode(input).issues.iter().map(issue_view).collect(),
    }
}

fn issue_view(issue: &Issue) -> IssueView {
    let level = match issue.level() {
        Level::Error => "error",
        Level::Warn => "warn",
    };
    IssueView {
        level,
        msg: issue.to_string(),
    }
}

/// The groove a link holds.
pub fn view_of(link: &str) -> GrooveView {
    GrooveView {
        bars: decode(link).score.bars().iter().map(bar_view).collect(),
    }
}

fn bar_view(bar: &Bar) -> BarView {
    let (num, den) = bar.meter();
    BarView {
        meter: format!("{num}/{den}"),
        length: rat_view(bar.length()),
        leaves: bar.leaves().iter().map(leaf_view).collect(),
    }
}

fn leaf_view(leaf: &Leaf) -> LeafView {
    LeafView {
        segment: leaf.segment,
        start: rat_view(leaf.start),
        span: rat_view(leaf.span),
    }
}

fn rat_view(r: Rat) -> RatView {
    RatView {
        num: r.num(),
        den: r.den(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn views_bars() {
        let view = view_of("#v=2&b=4/4:4;7/8:4,4,6");
        let meters: Vec<_> = view.bars.iter().map(|b| b.meter.as_str()).collect();
        let leaves: Vec<_> = view.bars.iter().map(|b| b.leaves.len()).collect();
        assert_eq!(meters, ["4/4", "7/8"]);
        assert_eq!(leaves, [16, 14]);
    }

    #[test]
    fn opens_with_issues() {
        let opened = open_of("?TimeSig=4/4&H=|x-x-|");
        let issues: Vec<_> = opened
            .issues
            .iter()
            .map(|i| (i.level, i.msg.as_str()))
            .collect();
        assert_eq!(
            issues,
            [("warn", "Groove Scribe link not imported; starting empty")]
        );
    }
}
