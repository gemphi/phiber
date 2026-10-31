//! The sidebar's own geometry — `ctx.sidebar.*`.
//!
//! One width and one hidden flag, shared by every sidebar: Blender's
//! `SCREEN_OT_region_scale` sizes the region the drag is on, and phiber keeps
//! a single size so the file browser's source list and the strip above it
//! land on the same edge (`view::size`).
//!
//! Not per host, deliberately. A per-host width would let the browser's list
//! and its strip disagree, which is the bug the shared key prevents.

use super::BKey;
use crate::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SidebarKey {
    /// The width a sidebar column was dragged to, in pixels.
    Width,
    /// Dragged shut (`RGN_FLAG_HIDDEN`): `"true"` while hidden.
    Hidden,
}

impl BKey for SidebarKey {
    fn path(&self) -> BPath {
        let tail: &'static str = match self {
            Self::Width => "width",
            Self::Hidden => "hidden",
        };
        BPath::owned(vec!["sidebar", tail])
    }
}
