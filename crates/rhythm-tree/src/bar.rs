//! A bar: a meter, a beat grouping and one segment per group.

use std::fmt;

use crate::issue::ParseError;
use crate::node::{Cursor, MAX_SLOTS, Node, int};
use crate::rat::Rat;

/// A bar of a groove.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bar {
    num: u32,
    den: u32,
    groups: Vec<u32>,
    segs: Vec<Node>,
}

/// A slot of a bar that holds one hit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Leaf {
    /// The segment (beat group) the leaf is in.
    pub segment: usize,
    /// Onset in quarters from the top of the bar.
    pub start: Rat,
    /// Length in quarters.
    pub span: Rat,
}

impl Bar {
    /// Reads a bar written like `7/8:4,4,6`.
    pub fn parse(s: &str) -> Result<Bar, ParseError> {
        Bar::parse_in(s, s)
    }

    /// Reads a bar item like `6/8:6*2`: the bar and how many times it repeats. `*N` is a repeat
    /// only when the text after `:` stays non-empty without it.
    pub(crate) fn parse_item(item: &str) -> Result<(Bar, u32), ParseError> {
        let has_segments = |spec: &str| {
            spec.split_once(':')
                .is_some_and(|(_, segs)| !segs.is_empty())
        };
        let (spec, count) = match item.rsplit_once('*') {
            Some((spec, count)) if has_segments(spec) => {
                int(count).map_or((item, 1), |k| (spec, k))
            }
            _ => (item, 1),
        };
        Ok((Bar::parse_in(spec, item)?, count))
    }

    /// Reads the bar `s`, naming `item`, the text it was written in, in errors.
    fn parse_in(s: &str, item: &str) -> Result<Bar, ParseError> {
        let not_a_bar = || ParseError::NotABar {
            spec: item.to_string(),
        };
        let (meter, segs) = s
            .split_once(':')
            .filter(|(_, segs)| !segs.is_empty() && !segs.contains(LINE_TERMINATORS))
            .ok_or_else(not_a_bar)?;
        let (num, den) = meter.split_once('/').ok_or_else(not_a_bar)?;
        let (num, den) = (
            int(num).ok_or_else(not_a_bar)?,
            int(den).ok_or_else(not_a_bar)?,
        );
        if !(1..=64).contains(&num) {
            return Err(ParseError::NumeratorOutOfRange { num, den });
        }
        if !DENOMINATORS.contains(&den) {
            return Err(ParseError::DenominatorOutOfRange { num, den });
        }
        let groups = default_groups(num, den);
        let mut cursor = Cursor::new(segs);
        let mut segs = vec![];
        loop {
            let n = cursor.int()?;
            if !(1..=MAX_SLOTS).contains(&n) {
                return Err(ParseError::SplitOutOfRange { n });
            }
            segs.push(Node { n });
            if !cursor.eat(b',') {
                break;
            }
        }
        if let Some(ch) = cursor.rest().chars().next() {
            return Err(ParseError::UnexpectedInBar {
                ch,
                bar: item.to_string(),
            });
        }
        if let [seg] = segs.as_slice() {
            segs = vec![seg.clone(); groups.len()];
        }
        if segs.len() != groups.len() {
            return Err(ParseError::SegmentsCover {
                used: segs.len(),
                num,
                den,
                groups: groups.len(),
            });
        }
        let slots = segs.iter().map(|seg| u64::from(seg.n)).sum::<u64>();
        if slots > u64::from(MAX_LEAVES_PER_BAR) {
            return Err(ParseError::TooManySlots { slots });
        }
        Ok(Bar {
            num,
            den,
            groups,
            segs,
        })
    }

    /// 4/4 in 16ths: what stands in for a bar that can't be read.
    pub(crate) fn sixteenths() -> Bar {
        let groups = default_groups(4, 4);
        Bar {
            num: 4,
            den: 4,
            segs: vec![Node { n: 4 }; groups.len()],
            groups,
        }
    }

    /// The meter as `(numerator, denominator)`.
    pub fn meter(&self) -> (u32, u32) {
        (self.num, self.den)
    }

    /// Length in quarters.
    pub fn length(&self) -> Rat {
        Rat::new(i64::from(self.num) * 4, i64::from(self.den))
    }

    /// The bar's leaves in time order.
    pub fn leaves(&self) -> Vec<Leaf> {
        let mut leaves = vec![];
        let mut onset = Rat::new(0, 1);
        for (segment, (seg, &group)) in self.segs.iter().zip(&self.groups).enumerate() {
            let span = Rat::new(i64::from(group) * 4, i64::from(self.den));
            let slot = span / i64::from(seg.n);
            for i in 0..seg.n {
                leaves.push(Leaf {
                    segment,
                    start: onset + slot * i64::from(i),
                    span: slot,
                });
            }
            onset = onset + span;
        }
        leaves
    }
}

impl fmt::Display for Bar {
    /// Writes the bar as [`Bar::parse`] reads it, with identical segments written once.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let splits: Vec<String> = self.segs.iter().map(|seg| seg.n.to_string()).collect();
        let written = if splits.windows(2).all(|w| w[0] == w[1]) {
            &splits[..1]
        } else {
            &splits[..]
        };
        write!(f, "{}/{}:{}", self.num, self.den, written.join(","))
    }
}

/// The most leaves one bar holds.
pub(crate) const MAX_LEAVES_PER_BAR: u32 = 512;

const DENOMINATORS: [u32; 6] = [1, 2, 4, 8, 16, 32];

/// The characters the reference's `.` doesn't match, so its bar pattern rejects them.
const LINE_TERMINATORS: [char; 4] = ['\n', '\r', '\u{2028}', '\u{2029}'];

/// Quarter-note meters beat by beat; others in 3s when they divide by 3, else in 2s ending in a 3
/// (7/8 is 2+2+3).
fn default_groups(num: u32, den: u32) -> Vec<u32> {
    if den <= 4 {
        return vec![1; num as usize];
    }
    if num % 3 == 0 {
        return vec![3; num as usize / 3];
    }
    let mut groups = vec![];
    let mut rest = num;
    while rest > 3 {
        groups.push(2);
        rest -= 2;
    }
    groups.push(rest);
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf_counts(bar: &Bar) -> Vec<usize> {
        let mut counts = vec![];
        for leaf in bar.leaves() {
            if leaf.segment == counts.len() {
                counts.push(0);
            }
            counts[leaf.segment] += 1;
        }
        counts
    }

    #[test]
    fn parses_meter_grouping_and_segments() {
        let bar = Bar::parse("7/8:4,4,6").unwrap();
        assert_eq!(bar.meter(), (7, 8));
        assert_eq!(leaf_counts(&bar), [4, 4, 6]);
    }

    #[test]
    fn repeats_a_single_segment_across_groups() {
        assert_eq!(leaf_counts(&Bar::parse("4/4:4").unwrap()), [4, 4, 4, 4]);
    }

    #[test]
    fn groups_by_default() {
        let rows: &[(&str, &[u32])] = &[
            ("4/4", &[1, 1, 1, 1]),
            ("3/4", &[1, 1, 1]),
            ("7/8", &[2, 2, 3]),
            ("5/8", &[2, 3]),
            ("6/8", &[3, 3]),
            ("9/8", &[3, 3, 3]),
            ("11/16", &[2, 2, 2, 2, 3]),
            ("2/8", &[2]),
        ];
        for &(meter, groups) in rows {
            let bar = Bar::parse(&format!("{meter}:1")).unwrap();
            let den = i64::from(bar.meter().1);
            let spans: Vec<_> = bar
                .leaves()
                .iter()
                .map(|leaf| leaf.span * den / 4)
                .collect();
            let groups: Vec<_> = groups.iter().map(|&g| Rat::new(i64::from(g), 1)).collect();
            assert_eq!(spans, groups, "{meter}");
        }
    }

    #[test]
    fn walks_leaves_exactly() {
        let leaves = Bar::parse("7/8:4,4,6").unwrap().leaves();
        assert_eq!(leaves.len(), 14);
        assert_eq!((leaves[8].segment, leaves[8].start), (2, Rat::new(2, 1)));
        assert_eq!(leaves[13].start, Rat::new(13, 4));
        assert_eq!(
            leaves
                .iter()
                .fold(Rat::new(0, 1), |sum, leaf| sum + leaf.span),
            Rat::new(7, 2)
        );
    }

    #[test]
    fn rejects_malformed_bars() {
        for (s, msg) in [
            ("4/4", "'4/4' is not a bar (expected like 4/4:4)"),
            ("4/4:", "'4/4:' is not a bar (expected like 4/4:4)"),
            ("0/4:4", "meter 0/4: numerator out of range"),
            ("65/4:4", "meter 65/4: numerator out of range"),
            ("4/5:4", "meter 4/5: denominator out of range"),
            ("4/4:4,4", "segments cover 2 beat groups; 4/4 has 4"),
            ("4/4:4x", "unexpected 'x' in bar '4/4:4x'"),
            ("4/4:4\n", "'4/4:4\n' is not a bar (expected like 4/4:4)"),
            ("4/4:4\r", "'4/4:4\r' is not a bar (expected like 4/4:4)"),
            (
                "4/4:4\u{2028}",
                "'4/4:4\u{2028}' is not a bar (expected like 4/4:4)",
            ),
            (
                "4/4:4\u{2029}",
                "'4/4:4\u{2029}' is not a bar (expected like 4/4:4)",
            ),
            ("4/4:a", "expected a number at 0 in 'a'"),
            ("4/4:0", "a split of 0 is out of range"),
            ("4/4:65", "a split of 65 is out of range"),
            ("64/4:64", "4096 slots in one bar is too many"),
            (
                "99999999999/4:4",
                "meter 4294967295/4: numerator out of range",
            ),
        ] {
            assert_eq!(
                Bar::parse(s).map_err(|e| e.to_string()),
                Err(msg.to_string()),
                "{s}"
            );
        }
    }
}
