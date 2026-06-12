//! All gameplay/visual tunables, split by domain. Everything is re-exported
//! flat through `common` (`use crate::common::*`), so call sites never name
//! these modules — change game balance from these files.

pub mod arena;
pub mod units;
pub mod weapons;
pub mod world;

pub use arena::*;
pub use units::*;
pub use weapons::*;
pub use world::*;
