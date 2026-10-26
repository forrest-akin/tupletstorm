//! A node of a rhythm tree: an equal split into `n` slots.

use std::fmt;

use crate::issue::ParseError;
use crate::rat::Rat;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Node {
    pub(crate) n: u32,
    /// How many of its parent's slots, or a bar's beat groups for a segment, the node fills.
    pub(crate) w: u32,
    /// The bracket's ratio label, the `m` of `n:m`.
    pub(crate) m: Option<u32>,
    /// The nodes the slots hold, when any slot splits again.
    pub(crate) kids: Option<Vec<Node>>,
}

impl Node {
    /// Reads the node at the cursor, `depth` levels below a segment.
    pub(crate) fn read(c: &mut Cursor, depth: usize) -> Result<Node, ParseError> {
        if depth > MAX_DEPTH {
            return Err(ParseError::TooDeep);
        }
        let n = c.int()?;
        if !(1..=MAX_SLOTS).contains(&n) {
            return Err(ParseError::SplitOutOfRange { n });
        }
        let m = if c.eat(b':') { Some(c.int()?) } else { None };
        if let Some(m) = m.filter(|m| !(1..=MAX_SLOTS).contains(m)) {
            return Err(ParseError::RatioOutOfRange { n, m });
        }
        let kids = if c.eat(b'(') {
            Some(Node::read_kids(c, depth, n)?).filter(|kids| !kids.iter().all(Node::is_plain))
        } else {
            None
        };
        let w = if c.eat(b'@') { c.int()? } else { 1 };
        if w == 0 {
            return Err(ParseError::WeightZero);
        }
        Ok(Node { n, w, m, kids })
    }

    /// Reads a whole node, like `3:2(1,2)`.
    pub(crate) fn parse(s: &str) -> Result<Node, ParseError> {
        let mut c = Cursor::new(s);
        let node = Node::read(&mut c, 0)?;
        match c.rest().chars().next() {
            Some(ch) => Err(ParseError::Unexpected {
                ch,
                at: c.at,
                text: s.to_string(),
            }),
            None => Ok(node),
        }
    }

    /// How many levels the node nests below itself.
    pub(crate) fn height(&self) -> usize {
        self.kids.as_ref().map_or(0, |kids| {
            1 + kids.iter().map(Node::height).max().unwrap_or_default()
        })
    }

    /// How many leaves the node holds.
    pub(crate) fn leaf_count(&self) -> u64 {
        self.kids.as_ref().map_or(u64::from(self.n), |kids| {
            kids.iter().map(Node::leaf_count).sum()
        })
    }

    /// The onset and span of each leaf in time order, for the node at `onset` lasting `span`.
    pub(crate) fn slots(&self, onset: Rat, span: Rat) -> Vec<(Rat, Rat)> {
        let slot = span / i64::from(self.n);
        let Some(kids) = &self.kids else {
            return (0..self.n)
                .map(|i| (onset + slot * i64::from(i), slot))
                .collect();
        };
        let mut at = 0;
        kids.iter()
            .flat_map(|kid| {
                let start = onset + slot * i64::from(at);
                at += kid.w;
                kid.slots(start, slot * i64::from(kid.w))
            })
            .collect()
    }

    /// A single slot, written `1`.
    fn is_plain(&self) -> bool {
        self.n == 1 && self.w == 1 && self.m.is_none() && self.kids.is_none()
    }

    /// After `(`: the children up to `)`, whose weights fill `n` slots.
    fn read_kids(c: &mut Cursor, depth: usize, n: u32) -> Result<Vec<Node>, ParseError> {
        let mut kids = vec![Node::read(c, depth + 1)?];
        while c.eat(b',') {
            kids.push(Node::read(c, depth + 1)?);
        }
        if !c.eat(b')') {
            return Err(ParseError::ExpectedCloseParen {
                at: c.at,
                text: c.s.to_string(),
            });
        }
        let filled = kids.iter().map(|kid| u64::from(kid.w)).sum::<u64>();
        if filled != u64::from(n) {
            return Err(ParseError::KidsDontFill { n, filled });
        }
        Ok(kids)
    }
}

impl fmt::Display for Node {
    /// Writes the node as [`Node::read`] reads it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.n)?;
        if let Some(m) = self.m {
            write!(f, ":{m}")?;
        }
        if let Some(kids) = &self.kids {
            let kids: Vec<String> = kids.iter().map(Node::to_string).collect();
            write!(f, "({})", kids.join(","))?;
        }
        if self.w != 1 {
            write!(f, "@{}", self.w)?;
        }
        Ok(())
    }
}

/// Replaces the node at `path` in `nodes` with `node`, keeping the replaced node's weight. An
/// unsplit node on the way splits into `n` leaves first.
pub(crate) fn graft(nodes: &mut Vec<Node>, path: &[u32], node: &Node) -> Result<(), ParseError> {
    if path.len() > MAX_DEPTH + 1 {
        return Err(ParseError::TooDeep);
    }
    let missing = || ParseError::NoSuchPath {
        path: path.to_vec(),
    };
    let (&last, above) = path.split_last().ok_or_else(missing)?;
    let mut list = nodes;
    for &i in above {
        let target = list.get_mut(i as usize).ok_or_else(missing)?;
        let n = target.n as usize;
        list = target.kids.get_or_insert_with(|| vec![LEAF; n]);
    }
    let target = list.get_mut(last as usize).ok_or_else(missing)?;
    *target = Node {
        w: target.w,
        ..node.clone()
    };
    Ok(())
}

const LEAF: Node = Node {
    n: 1,
    w: 1,
    m: None,
    kids: None,
};

/// The most slots one node splits into.
pub(crate) const MAX_SLOTS: u32 = 64;

/// The most levels a node nests below a segment.
pub(crate) const MAX_DEPTH: usize = 6;

/// A read position in link text.
pub(crate) struct Cursor<'a> {
    s: &'a str,
    at: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(s: &'a str) -> Cursor<'a> {
        Cursor { s, at: 0 }
    }

    /// The digit run at the cursor, moving past it.
    pub(crate) fn int(&mut self) -> Result<u32, ParseError> {
        let digits = self.rest().bytes().take_while(u8::is_ascii_digit).count();
        let n = int(&self.rest()[..digits]).ok_or_else(|| ParseError::ExpectedNumber {
            at: self.at,
            text: self.s.to_string(),
        })?;
        self.at += digits;
        Ok(n)
    }

    /// Moves past `b` when it is next.
    pub(crate) fn eat(&mut self, b: u8) -> bool {
        let next = self.rest().as_bytes().first() == Some(&b);
        if next {
            self.at += 1;
        }
        next
    }

    pub(crate) fn rest(&self) -> &str {
        &self.s[self.at..]
    }
}

/// A run of digits, saturating at `u32::MAX`.
pub(crate) fn int(s: &str) -> Option<u32> {
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

    fn parse(s: &str) -> Result<Node, ParseError> {
        Node::read(&mut Cursor::new(s), 0)
    }

    #[test]
    fn round_trips_nodes() {
        for s in ["4", "6:4", "3:2(2,2,5:4)", "5:4(1,1,1,3:2@2)", "3(1@2,1)"] {
            assert_eq!(parse(s).map(|node| node.to_string()), Ok(s.to_string()));
        }
    }

    #[test]
    fn canonicalises_plain_children() {
        assert_eq!(
            parse("4(1,1,1,1)").map(|node| node.to_string()),
            Ok("4".to_string())
        );
    }

    #[test]
    fn rejects_bad_nodes() {
        for (s, msg) in [
            ("5(1,1,1)", "children of 5(…) fill 3 slots, not 5"),
            ("3(1,1", "expected ')' at 5 in '3(1,1'"),
            ("4:0", "ratio 4:0"),
            ("4:65", "ratio 4:65"),
            ("4@0", "weight must be at least 1"),
            (
                "2(2(2(2(2(2(2(1,1),1),1),1),1),1),1)",
                "nested deeper than 6 levels",
            ),
        ] {
            assert_eq!(
                parse(s).map_err(|e| e.to_string()),
                Err(msg.to_string()),
                "{s}"
            );
        }
    }
}
