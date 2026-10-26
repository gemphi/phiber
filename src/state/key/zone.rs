//! The zones' keys — whether a side zone is shown, and the size a zone was
//! dragged to, under `ctx.zone.*` and `ctx.zone_size.*`.

use crate::space::BSpaceType;
use crate::state::BPath;
use crate::zone::BZoneType;
use super::BKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ZoneKey {
    /// `"false"` when a toggleable zone (Left, Right, Bottom) is hidden.
    Shown(BZoneType),
    /// The pixel size a zone's seam was dragged to, per space.
    Size(BSpaceType, BZoneType),
}

impl BKey for ZoneKey {
    fn path(&self) -> BPath {
        match self {
            Self::Shown(zone) => BPath::owned(vec!["zone", zone.slug()]),
            Self::Size(space, zone) => BPath::owned(vec!["zone_size", space.slug(), zone.slug()]),
        }
    }
}
