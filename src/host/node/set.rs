//! Node Editor regions, panes and parts — the content of the host.
//!
//! Node Editor host (generic). Blender ref: `space_node/space_node.cc:1745-`.
//! Regions: WINDOW, HEADER, ASSET_SHELF, ASSET_SHELF_HEADER, UI, TOOLS.

use crate::host::header;
use crate::pane::{BPaneSpec, BPaneType};
use crate::part::BPartType;
use crate::region::{BRegionSpec, BRegionType};

static CENTER_PANES: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::Toolbar],
    },
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::Custom],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[BPartType::Sidebar],
    },
];
static ASSET_SHELF_HEADER: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::AssetShelfHeader],
}];
static ASSET_SHELF: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::AssetShelf],
}];

pub static REGIONS: &[BRegionSpec] = &[
    BRegionSpec {
        kind: BRegionType::Header,
        panes: header::EDITOR,
    },
    BRegionSpec {
        kind: BRegionType::Center,
        panes: CENTER_PANES,
    },
    BRegionSpec {
        kind: BRegionType::AssetShelfHeader,
        panes: ASSET_SHELF_HEADER,
    },
    BRegionSpec {
        kind: BRegionType::AssetShelf,
        panes: ASSET_SHELF,
    },
];
