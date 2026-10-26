//! `BSpec` — the single trait every store spec at every level implements.

use crate::area::BAreaType;
use crate::host::BHostType;
use crate::modal::BModalType;
use crate::pane::BPaneType;
use crate::part::BPartType;
use crate::region::BRegionType;
use crate::space::BSpaceType;
use crate::zone::BZoneType;

/// Unified level tag — one arm per hierarchy tier (plus Modal peer).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BSpecKind {
    Space(BSpaceType),
    Zone(BZoneType),
    Area(BAreaType),
    Host(BHostType),
    Region(BRegionType),
    Pane(BPaneType),
    Part(BPartType),
    Modal(BModalType),
}

/// Every static spec exposes identity, label, icon. Children live on the
/// spec via inherent typed accessors.
pub trait BSpec: Copy + std::fmt::Debug + 'static {
    fn kind(&self) -> BSpecKind;
    fn label(&self) -> &'static str;
    fn icon(&self) -> &'static str {
        ""
    }
}
