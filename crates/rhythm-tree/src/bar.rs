//! A bar: a meter, a beat grouping and one segment per group.

use crate::issue::ParseError;
use crate::node::Node;
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
        let not_a_bar = || ParseError::NotABar {
            spec: s.to_string(),
        };
        let (meter, segs) = s.split_once(':').ok_or_else(not_a_bar)?;
        let (num, den) = meter.split_once('/').ok_or_else(not_a_bar)?;
        let (num, den) = (
            int(num).ok_or_else(not_a_bar)?,
            int(den).ok_or_else(not_a_bar)?,
        );
        if !(1..=64).contains(&num) || !DENOMINATORS.contains(&den) {
            return Err(not_a_bar());
        }
        let groups = default_groups(num, den);
        let mut segs = segs
            .split(',')
            .map(|n| {
                int(n)
                    .filter(|n| (1..=MAX_SLOTS).contains(n))
                    .map(|n| Node { n })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(not_a_bar)?;
        if let [seg] = segs.as_slice() {
            segs = vec![seg.clone(); groups.len()];
        }
        if segs.len() != groups.len()
            || segs.iter().map(|seg| seg.n).sum::<u32>() > MAX_LEAVES_PER_BAR
        {
            return Err(not_a_bar());
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

/// The most slots one node splits into.
pub const MAX_SLOTS: u32 = 64;

/// The most leaves one bar holds.
pub(crate) const MAX_LEAVES_PER_BAR: u32 = 512;

const DENOMINATORS: [u32; 6] = [1, 2, 4, 8, 16, 32];

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

/// A run of digits, saturating at `u32::MAX`.
fn int(s: &str) -> Option<u32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(s.bytes().fold(0u32, |n, b| {
        n.saturating_mul(10).saturating_add(u32::from(b - b'0'))
    }))
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
        for s in [
            "4/4",
            "4/4:",
            "0/4:4",
            "65/4:4",
            "4/5:4",
            "4/4:4,4",
            "4/4:4x",
            "4/4:a",
            "4/4:0",
            "4/4:65",
            "64/4:64",
            "99999999999/4:4",
        ] {
            assert_eq!(
                Bar::parse(s),
                Err(ParseError::NotABar {
                    spec: s.to_string()
                }),
                "{s}"
            );
        }
    }
}
