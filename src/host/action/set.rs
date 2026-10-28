//! Dope Sheet regions, panes and parts — the content of the host.
//!
//! Dope Sheet host. Blender ref: `space_action/space_action.cc:987-`.
//! Regions: WINDOW, HEADER, FOOTER, CHANNELS, UI. The footer is
//! `DOPESHEET_HT_playback_controls` (`bl_ui/space_dopesheet.py`): the same
//! `playback_controls` the Timeline draws in its header — the transport in
//! the middle, the frame fields at the right.

use crate::pane::{BPaneSpec, BPaneType};
use crate::part::BPartType;
use crate::region::{BRegionSpec, BRegionType};

/// The channel list at the left, as every side column is
/// (`part::channels::time`), the keys in the middle on the same rows, the
/// sidebar at the right.
static CENTER_PANES: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::Channels],
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

/// `playback_controls` in the footer: the transport, then the frame fields
/// (the Playback popover at its start is the Timeline menubar's, row 2 of
/// `timeline.md`, not yet a part of its own).
static FOOTER_PANES: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::Playback],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[BPartType::FrameRange],
    },
];

/// `DOPESHEET_HT_header`: the editor, its mode and menus on the left; in
/// the Action Editor the action and its slot after the spacer
/// (`_draw_action_selector`).
static HEADER: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::Switcher, BPartType::Modebar, BPartType::Menubar],
    },
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[BPartType::ActionTemplate],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[],
    },
];

pub static REGIONS: &[BRegionSpec] = &[
    BRegionSpec {
        kind: BRegionType::Header,
        panes: HEADER,
    },
    BRegionSpec {
        kind: BRegionType::Center,
        panes: CENTER_PANES,
    },
    BRegionSpec {
        kind: BRegionType::Footer,
        panes: FOOTER_PANES,
    },
];
