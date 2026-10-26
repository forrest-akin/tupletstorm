//! Rhythm trees for drum notation: exact timing and a compact link format.
//!
//! A [`Score`] is a list of [`Bar`]s and each lane's hits, one letter per leaf. A bar is a meter, a
//! beat grouping and one equal split per segment; [`Bar::leaves`] walks its slots in time order,
//! with onsets as exact [`Rat`]s in quarter notes. [`LANES`] is the kit.
//!
//! [`decode`] reads a v2 link, or a URL holding one. It never panics and never fails: problems
//! come back as [`Issue`]s, and whatever could be read is kept.

mod bar;
mod codec;
mod issue;
mod node;
mod rat;
mod score;

pub use bar::{Bar, Leaf};
pub use codec::{Decoded, decode};
pub use issue::{Fit, Issue, Level, ParseError};
pub use rat::Rat;
pub use score::{LANES, Lane, Score, lane};
