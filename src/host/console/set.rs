//! Python Console regions, panes and parts — the content of the host.
//!
//! Python Console host. Blender ref: `space_console/space_console.cc:370-`.
//! Regions: WINDOW, HEADER.

use crate::host::header;
use crate::pane::{BPaneSpec, BPaneType};
use crate::part::BPartType;
use crate::region::{BRegionSpec, BRegionType};

static CENTER_PANES: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::Custom],
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
];
