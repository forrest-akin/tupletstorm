//! Rhythm trees for drum notation: exact timing and a compact link format.
//!
//! A [`Score`] is a title, an author, comments, a tempo, swing, a list of [`Bar`]s and each lane's
//! hits, one letter per leaf.
//! A bar is a meter, a beat grouping and segments, each an equal split over one or more groups;
//! [`Bar::leaves`] walks its slots in time order, with onsets as exact [`Rat`]s in quarter notes.
//! [`LANES`] is the kit.
//!
//! [`decode`] reads a v2 link, or a URL holding one. It never panics and never fails: problems
//! come back as [`Issue`]s, and whatever could be read is kept. [`encode`] writes a score back as
//! link text and is canonical: `encode(decode(encode(s))) == encode(s)`.
//!
//! A slot can split again, with an optional bracket ratio and a weight in slots. Here a 3:2
//! spanning two beats holds a 5:4 in its last slot; the 5:4's second note, the bar's sixth leaf,
//! starts at 22/15 of a quarter:
//!
//! ```
//! use rhythm_tree::{Rat, decode, encode};
//!
//! let link = "v=2&q=90&b=4/4:3:2(2,2,5:4)@2,5:4(1,1,1,3:2@2)@2&sn=ooooooooooooooo";
//! let decoded = decode(link);
//! assert!(decoded.issues.is_empty());
//! assert_eq!(decoded.score.bars()[0].leaves()[5].start, Rat::new(22, 15));
//! assert_eq!(encode(&decoded.score), link);
//! ```

mod bar;
mod codec;
mod issue;
mod node;
mod rat;
mod score;

pub use bar::{Bar, Leaf};
pub use codec::{Decoded, decode, encode};
pub use issue::{EditError, Fit, Issue, Level, ParseError};
pub use rat::Rat;
pub use score::{LANES, Lane, Score, lane, letters};
