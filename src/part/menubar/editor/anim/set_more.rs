//! `anim/set.rs`'s second half — the Select / Channel / Transform / Snap /
//! Mirror / Keyframe / Action menus, split so each file sits under the
//! 240-line read cap (`tests/level_structure.rs`).

use super::super::super::super::item::BMenuItem;
use super::*;
use crate::state::OpKey;

/// `DOPESHEET_MT_select`.
pub static SELECT: &[BMenuItem] = &[
    fire("All", "A", OpKey::Keys, "all"),
    fire("None", "Alt A", OpKey::Keys, "none"),
    fire("Invert", "Ctrl I", OpKey::Keys, "invert"),
    SEP,
    op("Box Select", "B"),
    op("Box Select (Axis Range)", "Alt B"),
    op("Circle Select", "C"),
    sub("Lasso Select", LASSO),
    SEP,
    op("More", "Ctrl Numpad +"),
    op("Less", "Ctrl Numpad -"),
    SEP,
    op("Select Linked", "L"),
    SEP,
    fire("Columns on Selected Keys", "K", OpKey::Keys, "column:keys"),
    fire("Column on Current Frame", "Ctrl K", OpKey::Keys, "column:cfra"),
    fire("Columns on Selected Markers", "Shift K", OpKey::Keys, "column:markers"),
    fire("Between Selected Markers", "Alt K", OpKey::Keys, "column:between"),
    SEP,
    fire("Before Current Frame", "[", OpKey::Keys, "leftright:left"),
    fire("After Current Frame", "]", OpKey::Keys, "leftright:right"),
];
/// `DOPESHEET_MT_channel`.
pub static CHANNEL: &[BMenuItem] = &[
    op("Delete Channels", "X"),
    fire("Clean Channels", "", OpKey::Keys, "clean:channels"),
    SEP,
    op("Group Channels", "Ctrl G"),
    op("Ungroup Channels", "Ctrl Alt G"),
    SEP,
    sub("Toggle Channel Setting", CHANNEL_SETTING_TOGGLE),
    sub("Enable Channel Setting", CHANNEL_SETTING_ENABLE),
    sub("Disable Channel Setting", CHANNEL_SETTING_DISABLE),
    SEP,
    op("Toggle Channel Editability", "Tab"),
    sub("Extrapolation Mode", EXTRAPOLATION),
    SEP,
    fire_host("Expand Channels", "Numpad +", OpKey::ChannelFold, ":expand"),
    fire_host("Collapse Channels", "Numpad -", OpKey::ChannelFold, ":collapse"),
    SEP,
    sub("Move...", MOVE),
    SEP,
    op("Revive Disabled F-Curves", ""),
    SEP,
    op("Bake Channels", ""),
    SEP,
    op("Frame Selected Channels", ""),
];
pub static TRANSFORM: &[BMenuItem] = &[fire("Move", "G", OpKey::Keys, "grab:start"), op("Extend", "E"), op("Slide", "Shift T"), op("Scale", "S")];
pub static SNAP: &[BMenuItem] = &[
    fire("Selection to Current Frame", "", OpKey::Keys, "snap:current"),
    fire("Selection to Nearest Frame", "", OpKey::Keys, "snap:nearest_frame"),
    fire("Selection to Nearest Second", "", OpKey::Keys, "snap:nearest_second"),
    fire("Selection to Nearest Marker", "", OpKey::Keys, "snap:nearest_marker"),
];
pub static MIRROR: &[BMenuItem] = &[
    fire("By Times Over Current Frame", "", OpKey::Keys, "mirror:current"),
    op("By Values Over Zero Value", ""),
    fire("By Times Over First Selected Marker", "", OpKey::Keys, "mirror:marker"),
];
/// `DOPESHEET_MT_key_type` (`ACTION_OT_keyframe_type`): what the editors
/// draw a key as. Order and labels are Blender's `rna_enum_beztriple_keyframe_type_items`.
pub static KEYFRAME_TYPE: &[BMenuItem] = &[
    fire("Keyframe", "", OpKey::Keys, "keytype:keyframe"),
    fire("Breakdown", "", OpKey::Keys, "keytype:breakdown"),
    fire("Moving Hold", "", OpKey::Keys, "keytype:movehold"),
    fire("Extreme", "", OpKey::Keys, "keytype:extreme"),
    fire("Jitter", "", OpKey::Keys, "keytype:jitter"),
    fire("Generated", "", OpKey::Keys, "keytype:generated"),
];
/// `DOPESHEET_MT_key`.
pub static KEY: &[BMenuItem] = &[
    sub("Transform", TRANSFORM),
    sub("Snap", SNAP),
    sub("Mirror", MIRROR),
    SEP,
    // `action.keyframe_insert` asks which channels; I keys them all.
    fire("Insert Keyframes", "I", OpKey::Keyframe, "channels"),
    SEP,
    fire("Jump to Keyframes", "Ctrl G", OpKey::Keys, "jump"),
    SEP,
    fire("Copy Keyframes", "Ctrl C", OpKey::Keys, "copy"),
    fire("Paste Keyframes", "Ctrl V", OpKey::Keys, "paste"),
    op("Paste Flipped", "Shift Ctrl V"),
    fire("Duplicate", "Shift D", OpKey::Keys, "duplicate"),
    fire("Delete Keyframes", "X", OpKey::Keys, "delete"),
    SEP,
    sub("Keyframe Type", KEYFRAME_TYPE),
    sub("Handle Type", HANDLE_TYPE),
    sub("Interpolation Mode", INTERPOLATION),
    sub("Easing Mode", EASING),
    SEP,
    fire("Clean Keyframes", "", OpKey::Keys, "clean:keys"),
    op("Bake Keyframes", "Shift Alt O"),
    SEP,
    op("Discontinuity (Euler) Filter", ""),
];
/// `DOPESHEET_MT_action`.
pub static ACTION: &[BMenuItem] = &[
    op("Merge Animation", ""),
    op("Separate Slots", ""),
    SEP,
    op("Move Slots to new Action", ""),
    SEP,
    fire("Push Down Action", "", OpKey::Action, "push_down"),
    fire("Stash Action", "", OpKey::Action, "stash"),
];
