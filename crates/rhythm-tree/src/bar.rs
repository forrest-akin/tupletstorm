//! A bar: a meter, a beat grouping and segments, each covering one or more groups.

use std::fmt;

use crate::issue::ParseError;
use crate::node::{Cursor, MAX_DEPTH, Node, graft, int};
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
    /// The segment the leaf is in.
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
        let (meter, grouping) = match meter.split_once('~') {
            Some((meter, grouping)) => {
                let parts = grouping.split('-').map(int).collect::<Option<Vec<_>>>();
                (meter, Some(parts.ok_or_else(not_a_bar)?))
            }
            None => (meter, None),
        };
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
        let groups = grouping.unwrap_or_else(|| default_groups(num, den));
        let sum = groups.iter().map(|&g| u64::from(g)).sum::<u64>();
        if groups.contains(&0) || sum != u64::from(num) {
            return Err(ParseError::GroupingDoesntAddUp { groups, num });
        }
        let mut cursor = Cursor::new(segs);
        let mut segs = vec![];
        loop {
            segs.push(Node::read(&mut cursor, 0)?);
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
        if let [seg @ Node { w: 1, .. }] = segs.as_slice() {
            segs = vec![seg.clone(); groups.len()];
        }
        let used = segs.iter().map(|seg| u64::from(seg.w)).sum::<u64>();
        if used != groups.len() as u64 {
            return Err(ParseError::SegmentsCover {
                used,
                num,
                den,
                groups: groups.len(),
            });
        }
        let slots = segs.iter().map(Node::leaf_count).sum::<u64>();
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
            segs: vec![
                Node {
                    n: 4,
                    w: 1,
                    m: None,
                    kids: None,
                };
                groups.len()
            ],
            groups,
        }
    }

    /// The meter as `(numerator, denominator)`.
    pub fn meter(&self) -> (u32, u32) {
        (self.num, self.den)
    }

    /// The beat grouping in 1/den units, summing to the numerator (7/8 → `[2, 2, 3]`).
    pub fn groups(&self) -> &[u32] {
        &self.groups
    }

    /// Whether the grouping is the meter's default, which links leave unwritten.
    pub fn has_default_grouping(&self) -> bool {
        self.groups == default_groups(self.num, self.den)
    }

    /// Length in quarters.
    pub fn length(&self) -> Rat {
        Rat::new(i64::from(self.num) * 4, i64::from(self.den))
    }

    /// The bar with `node` grafted in at `path`, within the limits a bar has.
    pub(crate) fn grafted(&self, path: &[u32], node: &Node) -> Result<Bar, ParseError> {
        let mut bar = self.clone();
        graft(&mut bar.segs, path, node)?;
        if bar.segs.iter().any(|seg| seg.height() > MAX_DEPTH) {
            return Err(ParseError::TooDeep);
        }
        let slots = bar.segs.iter().map(Node::leaf_count).sum::<u64>();
        if slots > u64::from(MAX_LEAVES_PER_BAR) {
            return Err(ParseError::TooManySlots { slots });
        }
        Ok(bar)
    }

    /// The bar's leaves in time order.
    pub fn leaves(&self) -> Vec<Leaf> {
        let mut leaves = vec![];
        let mut onset = Rat::new(0, 1);
        let mut groups = self.groups.iter();
        for (segment, seg) in self.segs.iter().enumerate() {
            let units: u32 = groups.by_ref().take(seg.w as usize).sum();
            let span = Rat::new(i64::from(units) * 4, i64::from(self.den));
            leaves.extend(
                seg.slots(onset, span)
                    .into_iter()
                    .map(|(start, span)| Leaf {
                        segment,
                        start,
                        span,
                    }),
            );
            onset = onset + span;
        }
        leaves
    }
}

impl fmt::Display for Bar {
    /// Writes the bar as [`Bar::parse`] reads it, with identical one-group segments written once.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let splits: Vec<String> = self.segs.iter().map(Node::to_string).collect();
        let repeated = splits.windows(2).all(|pair| pair[0] == pair[1])
            && self.segs.iter().all(|seg| seg.w == 1);
        let written = if repeated { &splits[..1] } else { &splits[..] };
        write!(f, "{}/{}", self.num, self.den)?;
        if !self.has_default_grouping() {
            let parts: Vec<String> = self.groups.iter().map(u32::to_string).collect();
            write!(f, "~{}", parts.join("-"))?;
        }
        write!(f, ":{}", written.join(","))
    }
}

/// The most leaves one bar holds.
pub(crate) const MAX_LEAVES_PER_BAR: u32 = 512;

const DENOMINATORS: [u32; 6] = [1, 2, 4, 8, 16, 32];

/// The characters the reference's `.` doesn't match, so its bar and override patterns reject them.
pub(crate) const LINE_TERMINATORS: [char; 4] = ['\n', '\r', '\u{2028}', '\u{2029}'];

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
    fn reads_an_explicit_grouping() {
        assert_eq!(
            Bar::parse("5/8~3-2:6,4").map(|bar| bar.groups().to_vec()),
            Ok(vec![3, 2])
        );
    }

    #[test]
    fn elides_the_default_grouping() {
        for (s, written) in [
            ("7/8~3-2-2:6,4,4", "7/8~3-2-2:6,4,4"),
            ("7/8~2-2-3:4,4,6", "7/8:4,4,6"),
        ] {
            assert_eq!(
                Bar::parse(s).map(|bar| bar.to_string()),
                Ok(written.to_string()),
                "{s}"
            );
        }
    }

    #[test]
    fn rejects_groupings_that_dont_add_up() {
        for (s, msg) in [
            ("4/4~1-1:4", "grouping 1+1 doesn't add up to 4"),
            ("4/4~0-4:4", "grouping 0+4 doesn't add up to 4"),
            (
                "4/4~18446744073709551615-18446744073709551615:4",
                "grouping 4294967295+4294967295 doesn't add up to 4",
            ),
        ] {
            assert_eq!(
                Bar::parse(s).map_err(|e| e.to_string()),
                Err(msg.to_string()),
                "{s}"
            );
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
    fn places_nested_tuplet_leaves_exactly() {
        let starts = Bar::parse("4/4:3:2(2,2,5:4)@2,5:4(1,1,1,3:2@2)@2").map(|bar| {
            bar.leaves()
                .iter()
                .map(|leaf| leaf.start * 15)
                .collect::<Vec<_>>()
        });
        let expected = [0, 5, 10, 15, 20, 22, 24, 26, 28, 30, 36, 42, 48, 52, 56];
        assert_eq!(
            starts,
            Ok(expected.map(|start| Rat::new(start, 1)).to_vec())
        );
    }

    #[test]
    fn weights_count_beat_groups() {
        let spans = Bar::parse("5/4:4@3,4@2").map(|bar| {
            bar.leaves()
                .iter()
                .filter(|leaf| leaf.segment == 0)
                .map(|leaf| leaf.span)
                .collect::<Vec<_>>()
        });
        assert_eq!(spans, Ok(vec![Rat::new(3, 4); 4]));
    }

    #[test]
    fn huge_weights_saturate() {
        assert_eq!(
            Bar::parse("4/4:4@99999999999").map_err(|e| e.to_string()),
            Err("segments cover 4294967295 beat groups; 4/4 has 4".to_string())
        );
    }

    #[test]
    fn writes_weighted_segments_apart() {
        assert_eq!(
            Bar::parse("4/4:4@2,4@2").map(|bar| bar.to_string()),
            Ok("4/4:4@2,4@2".to_string())
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
            (
                "4/5~2-x:4",
                "'4/5~2-x:4' is not a bar (expected like 4/4:4)",
            ),
            ("99/4~x:4", "'99/4~x:4' is not a bar (expected like 4/4:4)"),
            (
                "2/11~2-2-x3:4,4:3",
                "'2/11~2-2-x3:4,4:3' is not a bar (expected like 4/4:4)",
            ),
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
            ("4/4:4(64,64,64,64)", "1024 slots in one bar is too many"),
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
