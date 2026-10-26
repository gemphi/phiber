//! Compositor regions — the same table as `clip`, re-exported rather
//! than repeated: two hosts that lay out identically should not be able
//! to drift apart.

pub use crate::host::clip::set::REGIONS;
