//! Top Bar regions, panes and parts — the content of the host.
//!
//! Topbar host — menu, where the window stands, zone visibility, and host
//! switcher.
//! Blender ref: DNA `SPACE_TOPBAR` (single HEADER region, global screen area).

use crate::pane::{BPaneSpec, BPaneType};
use crate::part::BPartType;
use crate::region::{BRegionSpec, BRegionType};

static HEADER_PANES: &[BPaneSpec] = &[
    // Left: what the window is — the space switcher, the brand, the menus.
    // Centre: the search that finds anything in it (`find`), reading the
    // space you are in while idle. Right: what can be hidden, then the
    // window controls. The spaces are no row of tabs here: nine of them do
    // not fit a third of the bar, and the search finds every one.
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::CaptionLead, BPartType::Menubar],
    },
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::Search],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[BPartType::Zonebar, BPartType::CaptionTrail],
    },
];

pub static REGIONS: &[BRegionSpec] = &[BRegionSpec {
    kind: BRegionType::Header,
    panes: HEADER_PANES,
}];
