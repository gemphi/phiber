//! What a Keyframe Type looks like — `draw_keyframe_shape`'s colour and
//! size switches (`editors/animation/keyframes_draw.cc`).
//!
//! Split from `mod.rs` so each file sits under the 240-line read cap
//! (`tests/level_structure.rs`). The colours are fields on [`BAnimColors`]
//! there, because that is the struct a painter is handed; what each one is
//! chosen to be is here.

use super::{BAnimColors, BResolvedUiStyle};
use crate::phiphx::blend::{KEYTYPE_BREAKDOWN, KEYTYPE_EXTREME, KEYTYPE_GENERATED, KEYTYPE_JITTER, KEYTYPE_MOVEHOLD};
use crate::theme::palette::BColor;
use crate::theme::style::color::BResolvedColors;

/// A Keyframe Type's selected colour is its own lifted: Blender's selected
/// variants are the brighter of each pair (`b3dbe8` → `54bfed`).
pub(super) const KEYTYPE_SELECT_LIFT: f32 = 0.12;
/// Moving Hold and Generated are Blender's two greys, the second the darker
/// (`808080` and `585858`): the palette's dim text, and a step under it.
const GENERATED_UNDER_MOVEHOLD: f32 = -0.14;

/// The unselected fill of each Keyframe Type, indexed by
/// `eBezTriple_KeyframeType`.
///
/// Blender gives each a fixed hex (`userdef_default_theme.c`): a plain
/// keyframe `bfbfbf`, an extreme pink `e8b3cc`, a breakdown light blue
/// `b3dbe8`, a jitter green `94e575`, a moving hold mid grey `808080` and a
/// generated key dark grey `585858`. Phiber maps a Blender theme colour onto
/// its OWN palette rather than copying the hex — `key` is already the
/// palette's text and not `bfbfbf` — so each type takes the token nearest
/// Blender's intent: the cool brand for a breakdown, the online green for a
/// jitter, the soft accent for an extreme, and the dim text (and a step
/// under it) for the two greys.
pub(super) fn fills(style: &BResolvedUiStyle, colors: &BResolvedColors) -> [BColor; 6] {
    let mut out = [colors.text; 6];
    out[KEYTYPE_EXTREME as usize] = colors.accent_soft;
    out[KEYTYPE_BREAKDOWN as usize] = colors.brand_cool;
    out[KEYTYPE_JITTER as usize] = colors.online;
    out[KEYTYPE_MOVEHOLD as usize] = colors.text_dim;
    out[KEYTYPE_GENERATED as usize] = style.shade(colors.text_dim, GENERATED_UNDER_MOVEHOLD);
    out
}

impl BAnimColors {
    /// The fill a key of `keytype` takes, selected or not — the colour
    /// switch in `draw_keyframe_shape`. A plain keyframe keeps the
    /// palette's own `key` / `key_select`, so a theme that overrides those
    /// still rules the common case; an unknown type reads as one.
    pub fn of_keytype(&self, keytype: u8, selected: bool) -> BColor {
        let plain = match selected {
            true => self.key_select,
            false => self.key,
        };
        let table = match selected {
            true => &self.keytype_select,
            false => &self.keytype,
        };
        match keytype {
            0 => plain,
            t => table.get(t as usize).copied().unwrap_or(plain),
        }
    }

    /// How much bigger or smaller `keytype` draws its diamond — Blender's
    /// own factors (the size switch in `draw_keyframe_shape`): an extreme
    /// is 1.2× a plain keyframe, a moving hold 0.925, a breakdown 0.85, a
    /// jitter 0.8 and a generated key 0.75.
    pub fn keytype_scale(keytype: u8) -> f32 {
        match keytype {
            KEYTYPE_BREAKDOWN => 0.85,
            KEYTYPE_MOVEHOLD => 0.925,
            KEYTYPE_EXTREME => 1.2,
            KEYTYPE_JITTER => 0.8,
            KEYTYPE_GENERATED => 0.75,
            _ => 1.0,
        }
    }
}
