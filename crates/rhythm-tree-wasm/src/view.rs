//! Plain data the browser receives, built by functions that run natively.

use rhythm_tree::{
    Bar, EditError, Issue, LANES, Leaf, Level, Rat, Score, decode, encode, lane, letters,
};
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
    /// The canonical link: what the client keeps and passes back.
    pub link: String,
    /// The lanes the editor shows: the whole kit, then the link's custom lanes.
    pub lanes: Vec<String>,
    pub issues: Vec<IssueView>,
}

/// A groove as the client draws it.
#[derive(Debug, Serialize)]
pub struct GrooveView {
    /// The title; empty when the link has none.
    pub title: String,
    /// Quarter notes per minute.
    pub tempo: f64,
    pub bars: Vec<BarView>,
    /// The requested lanes, in the order asked for.
    pub lanes: Vec<LaneView>,
}

/// One lane's hits.
#[derive(Debug, Serialize, PartialEq)]
pub struct LaneView {
    pub id: String,
    /// What the editor calls it.
    pub name: String,
    /// The letters a cell cycles through, rest first.
    pub letters: String,
    pub bars: Vec<LaneBarView>,
}

/// One bar of one lane.
#[derive(Debug, Serialize, PartialEq)]
pub struct LaneBarView {
    /// One letter per leaf; `-` is a rest.
    pub hits: String,
}

/// One bar.
#[derive(Debug, Serialize)]
pub struct BarView {
    /// Like `7/8`.
    pub meter: String,
    /// Like `3+2`; `None` for the meter's default grouping.
    pub grouping: Option<String>,
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
    let decoded = decode(input);
    let custom = decoded
        .score
        .lanes()
        .into_iter()
        .filter(|id| lane(id).is_none());
    Opened {
        link: encode(&decoded.score),
        lanes: LANES
            .iter()
            .map(|lane| lane.id)
            .chain(custom)
            .map(String::from)
            .collect(),
        issues: decoded.issues.iter().map(issue_view).collect(),
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
pub fn view_of(link: &str, lanes: &[&str]) -> GrooveView {
    let score = score_of(link);
    GrooveView {
        title: score.title().to_string(),
        tempo: score.tempo(),
        bars: score.bars().iter().map(bar_view).collect(),
        lanes: lanes.iter().map(|id| lane_view(&score, id)).collect(),
    }
}

/// The link with one hit set; `bar` and `leaf` count from 0.
pub fn set_hit_of(
    link: &str,
    lane: &str,
    bar: usize,
    leaf: usize,
    letter: char,
) -> Result<String, EditError> {
    let mut score = score_of(link);
    score.set_hit(lane, bar, leaf, letter)?;
    Ok(encode(&score))
}

fn score_of(link: &str) -> Score {
    // The link is canonical, so its issues were already listed when it was opened.
    decode(link).score
}

fn lane_view(score: &Score, id: &str) -> LaneView {
    let (name, chars) = (lane(id).map_or(id, |lane| lane.name), letters(id));
    let bars = match score.hits(id) {
        Some(hits) => hits
            .iter()
            .map(|hits| LaneBarView { hits: hits.clone() })
            .collect(),
        None => score
            .bars()
            .iter()
            .map(|bar| LaneBarView {
                hits: "-".repeat(bar.leaves().len()),
            })
            .collect(),
    };
    LaneView {
        id: id.to_string(),
        name: name.to_string(),
        letters: format!("-{chars}"),
        bars,
    }
}

fn bar_view(bar: &Bar) -> BarView {
    let (num, den) = bar.meter();
    BarView {
        meter: format!("{num}/{den}"),
        grouping: (!bar.has_default_grouping()).then(|| {
            let parts: Vec<String> = bar.groups().iter().map(u32::to_string).collect();
            parts.join("+")
        }),
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
    fn views_the_title_and_tempo() {
        let view = view_of("#v=2&t=A%20%26%20B&q=92.5", &[]);
        assert_eq!((view.title.as_str(), view.tempo), ("A & B", 92.5));
    }

    #[test]
    fn views_bars() {
        let view = view_of("#v=2&b=4/4:4;7/8:4,4,6", &[]);
        let meters: Vec<_> = view.bars.iter().map(|b| b.meter.as_str()).collect();
        let leaves: Vec<_> = view.bars.iter().map(|b| b.leaves.len()).collect();
        assert_eq!(meters, ["4/4", "7/8"]);
        assert_eq!(leaves, [16, 14]);
    }

    fn lane(id: &str, name: &str, letters: &str, hits: &str) -> LaneView {
        LaneView {
            id: id.to_string(),
            name: name.to_string(),
            letters: letters.to_string(),
            bars: vec![LaneBarView {
                hits: hits.to_string(),
            }],
        }
    }

    #[test]
    fn views_a_custom_lane() {
        let view = view_of("#v=2&b=4/4:4&zz9=x", &["zz9"]);
        assert_eq!(
            view.lanes,
            [lane("zz9", "zz9", "-x", &format!("x{}", "-".repeat(15)))]
        );
    }

    #[test]
    fn views_a_lane_the_link_lacks_as_rests() {
        let view = view_of("#v=2&b=4/4:4", &["hh"]);
        assert_eq!(view.lanes, [lane("hh", "Hi-hat", "-xXo+", &"-".repeat(16))]);
    }

    #[test]
    fn refuses_a_slot_the_bar_lacks() {
        assert!(matches!(
            set_hit_of("v=2&q=80&b=4/4:4", "hh", 0, 16, 'x'),
            Err(EditError::NoSuchSlot { .. })
        ));
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

    #[test]
    fn opens_to_the_canonical_link() {
        assert_eq!(open_of("https://example.com/").link, "v=2&q=80&b=4/4:4");
    }
}
