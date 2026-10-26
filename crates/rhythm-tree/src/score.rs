//! A groove: its bars and each lane's hits.

use std::collections::BTreeMap;

use crate::bar::Bar;

/// A kit piece: one row of hits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lane {
    /// The link key, like `hh`.
    pub id: &'static str,
    /// Like `Hi-hat`.
    pub name: &'static str,
    /// The letters a hit can be; `-` is a rest.
    pub chars: &'static str,
}

/// The kit, in the order the editor shows it. Letters are Groove Scribe's tab letters.
pub const LANES: [Lane; 14] = [
    lane_of("cr", "Crash", "xX"),
    lane_of("rd", "Ride", "xXb"),
    lane_of("hh", "Hi-hat", "xXo+"),
    lane_of("sk", "Stacker", "x"),
    lane_of("cb", "Cowbell", "x"),
    lane_of("sn", "Snare", "oOgxfdb"),
    lane_of("t1", "High tom", "oO"),
    lane_of("t2", "Mid tom", "oO"),
    lane_of("t3", "Low tom", "oO"),
    lane_of("t4", "Floor tom", "oO"),
    lane_of("kk", "Kick", "oO"),
    lane_of("hf", "Hi-hat foot", "x"),
    lane_of("mt", "Metronome", "nN"),
    lane_of("st", "Sticking", "RLBc"),
];

const fn lane_of(id: &'static str, name: &'static str, chars: &'static str) -> Lane {
    Lane { id, name, chars }
}

/// The kit lane with this id.
pub fn lane(id: &str) -> Option<&'static Lane> {
    LANES.iter().find(|lane| lane.id == id)
}

/// A decoded groove.
#[derive(Clone, Debug, PartialEq)]
pub struct Score {
    bars: Vec<Bar>,
    hits: BTreeMap<String, Vec<String>>,
}

impl Score {
    pub(crate) fn new(bars: Vec<Bar>, hits: BTreeMap<String, Vec<String>>) -> Score {
        Score { bars, hits }
    }

    /// The bars, in order.
    pub fn bars(&self) -> &[Bar] {
        &self.bars
    }

    /// A lane's hits, one string per bar with one letter per leaf, or `None` when the link has no such lane.
    pub fn hits(&self, lane: &str) -> Option<&[String]> {
        self.hits.get(lane).map(Vec::as_slice)
    }

    /// The lanes with hits: kit lanes in [`LANES`] order, then custom lanes sorted.
    pub fn lanes(&self) -> Vec<&str> {
        let kit = LANES.iter().map(|lane| lane.id);
        let custom = self
            .hits
            .keys()
            .map(String::as_str)
            .filter(|id| lane(id).is_none());
        kit.filter(|id| self.hits.contains_key(*id))
            .chain(custom)
            .collect()
    }
}

/// Whether a key names a lane: not a format field, and a lowercase letter, then one to five
/// lowercase letters or digits.
pub(crate) fn is_lane_key(key: &str) -> bool {
    if RESERVED_KEYS.contains(&key) {
        return false;
    }
    let mut chars = key.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && (2..=6).contains(&key.len())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

pub(crate) const KNOWN_KEYS: [&str; 2] = ["v", "b"];

/// Format fields that are never lanes. Those not in `KNOWN_KEYS` are reported as unknown.
const RESERVED_KEYS: [&str; 7] = ["t", "a", "c", "q", "sw", "tc", "ov"];
