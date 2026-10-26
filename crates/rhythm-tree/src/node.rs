//! A node of a rhythm tree: an equal split into `n` slots.

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Node {
    pub(crate) n: u32,
}
