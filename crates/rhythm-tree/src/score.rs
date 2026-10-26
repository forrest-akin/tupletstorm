//! A groove: its title, author, comments, tempo, swing, bars and each lane's hits.

use std::collections::BTreeMap;

use crate::bar::Bar;
use crate::issue::EditError;

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

/// The letters a lane can hold besides `-`: the kit lane's, or `x` for a custom lane.
pub fn letters(id: &str) -> &'static str {
    lane(id).map_or("x", |lane| lane.chars)
}

/// A decoded groove.
#[derive(Clone, Debug, PartialEq)]
pub struct Score {
    title: String,
    author: String,
    comments: String,
    tempo: f64,
    swing: f64,
    bars: Vec<Bar>,
    hits: BTreeMap<String, Vec<String>>,
}

impl Score {
    pub(crate) fn new(
        title: String,
        author: String,
        comments: String,
        tempo: f64,
        swing: f64,
        bars: Vec<Bar>,
        hits: BTreeMap<String, Vec<String>>,
    ) -> Score {
        Score {
            title,
            author,
            comments,
            tempo,
            swing,
            bars,
            hits,
        }
    }

    /// The title; empty when the link has none.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Who wrote the groove; empty when the link doesn't say.
    pub fn author(&self) -> &str {
        &self.author
    }

    /// Notes on the groove; empty when the link has none.
    pub fn comments(&self) -> &str {
        &self.comments
    }

    /// Quarter notes per minute.
    pub fn tempo(&self) -> f64 {
        self.tempo
    }

    /// Swing, from 0 (straight) to 100.
    pub fn swing(&self) -> f64 {
        self.swing
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

    /// Sets one hit: `bar` and `leaf` count from 0. A lane the score lacks starts as rests.
    pub fn set_hit(
        &mut self,
        lane: &str,
        bar: usize,
        leaf: usize,
        letter: char,
    ) -> Result<(), EditError> {
        if !is_lane_key(lane) {
            return Err(EditError::UnknownLane {
                lane: lane.to_string(),
            });
        }
        let chars = letters(lane);
        if letter != '-' && !chars.contains(letter) {
            return Err(EditError::NotALetter {
                lane: lane.to_string(),
                letter,
            });
        }
        if bar >= self.bars.len() {
            return Err(EditError::NoSuchBar { bar });
        }
        if leaf >= self.bars[bar].leaves().len() {
            return Err(EditError::NoSuchSlot {
                lane: lane.to_string(),
                bar,
                leaf,
            });
        }
        let bars = &self.bars;
        let hits = self.hits.entry(lane.to_string()).or_insert_with(|| {
            bars.iter()
                .map(|bar| "-".repeat(bar.leaves().len()))
                .collect()
        });
        hits[bar] = hits[bar]
            .chars()
            .enumerate()
            .map(|(i, c)| if i == leaf { letter } else { c })
            .collect();
        Ok(())
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

pub(crate) const KNOWN_KEYS: [&str; 7] = ["v", "t", "a", "c", "q", "sw", "b"];

/// Format fields that are never lanes. Those not in `KNOWN_KEYS` are reported as unknown.
const RESERVED_KEYS: [&str; 7] = ["t", "a", "c", "q", "sw", "tc", "ov"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode;

    fn score() -> Score {
        decode("#v=2&b=4/4:4&hh=x").score
    }

    fn rests_after(hits: &str) -> String {
        format!("{hits}{}", "-".repeat(16 - hits.len()))
    }

    #[test]
    fn sets_a_hit() {
        let mut score = score();
        score.set_hit("hh", 0, 1, 'X').unwrap();
        score.set_hit("sn", 0, 0, 'o').unwrap();
        assert_eq!(score.hits("hh"), Some(&[rests_after("xX")][..]));
        assert_eq!(score.hits("sn"), Some(&[rests_after("o")][..]));
    }

    #[test]
    fn rejects_bad_edits() {
        let rows = [
            ("hh", 1, 0, 'x', "bar 1 doesn't exist"),
            ("hh", 0, 16, 'x', "hh bar 0 has no slot 16"),
            ("hh", 0, 0, 'q', "'q' is not a hh letter"),
            ("9z", 0, 0, 'x', "no lane '9z'"),
            ("tc", 0, 0, 'x', "no lane 'tc'"),
        ];
        for (lane, bar, leaf, letter, msg) in rows {
            let err = score().set_hit(lane, bar, leaf, letter).unwrap_err();
            assert_eq!(err.to_string(), msg, "{lane} {bar} {leaf} {letter}");
        }
    }
}
