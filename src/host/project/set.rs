//! Project Browser regions, as Reality Composer Pro lays its browser out:
//! one toolbar under the area's editor tabs, then the navigator at the
//! left, the cards in the middle, the inspector at the right.
//!
//! No header of its own: the Project Browser always sits under the bottom
//! area's tabs, which already name it and switch it, so a header would only
//! repeat the tab. The toolbar's three groups stand over the columns they
//! act on: import and new over the navigator, the search centred over the
//! cards, the view controls over the inspector.

use crate::pane::{BPaneSpec, BPaneType};
use crate::part::BPartType;
use crate::region::{BRegionSpec, BRegionType};

static BAR_PANES: &[BPaneSpec] = &[
    BPaneSpec { kind: BPaneType::Left, parts: &[BPartType::ProjectBar] },
    BPaneSpec { kind: BPaneType::Center, parts: &[BPartType::Search] },
    BPaneSpec { kind: BPaneType::Right, parts: &[BPartType::ProjectView] },
];
static NAV_PANES: &[BPaneSpec] = &[BPaneSpec { kind: BPaneType::Center, parts: &[BPartType::ProjectNav] }];
static CENTER_PANES: &[BPaneSpec] = &[BPaneSpec { kind: BPaneType::Center, parts: &[BPartType::Custom] }];
static INFO_PANES: &[BPaneSpec] = &[BPaneSpec { kind: BPaneType::Center, parts: &[BPartType::ProjectInfo] }];

pub static REGIONS: &[BRegionSpec] = &[
    BRegionSpec { kind: BRegionType::ToolHeader, panes: BAR_PANES },
    BRegionSpec { kind: BRegionType::Tools, panes: NAV_PANES },
    BRegionSpec { kind: BRegionType::Center, panes: CENTER_PANES },
    BRegionSpec { kind: BRegionType::UI, panes: INFO_PANES },
];
