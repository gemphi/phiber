//! The 3D view's readers. Split out of `scene/mod.rs` so the parent stays
//! under the 200-line cap.
//!
//! This file holds no keys. Its `*_key()` functions each returned
//! `BViewSlot::key(View3dKey::<Field>)` and nothing else, so a view field
//! had two spellings and the call site read as though the key were global —
//! which these are not: every 3D Viewport keeps its own view
//! (`view_slot::BViewSlot`). Call sites say `BViewSlot::key(View3dKey::Zoom)`
//! now, which names the slot resolution it goes through.
//!
//! What is left is the `view_*` readers, which earn their place: each applies
//! Blender's factory default and, where it matters, a documented clamp.

use super::view_slot::BViewSlot;
use crate::state::{BState, View3dKey};

// ── Yaw / zoom / perspective ────────────────────────────────────────────
//
// Every reader goes through the area being drawn or acted on
// (`view_slot::BViewSlot`): each 3D Viewport has a view of its own.

/// Is the viewport in perspective mode? Defaults to `true` (Blender default).
pub fn is_perspective(s: &BState) -> bool {
    BViewSlot::get(s, View3dKey::Perspective) != Some("false")
}

// ── Pitch (elevation) state ─────────────────────────────────────────────
//
// The pitch axis tilts the camera up/down about the horizontal plane.
// 0° = looking level with the ground plane; +30° = looking down onto it
// (Blender's default user-perspective elevation).

/// Read the current view pitch in degrees, in `(-180, 180]` — past ±90 the
/// view has gone over a pole (`orbit::BTurntable`). Defaults to 30° —
/// Blender's startup elevation angle.
pub fn view_pitch(s: &BState) -> f32 {
    let pitch = BViewSlot::get(s, View3dKey::Pitch).and_then(|v| v.parse::<f32>().ok());
    super::orbit::BTurntable::wrap(pitch.unwrap_or(30.0))
}

/// Add `delta_deg` to the current pitch, over a pole if it goes that far.
pub fn pitch_step(pitch: f32, delta_deg: f32) -> f32 {
    super::orbit::BTurntable::wrap(pitch + delta_deg)
}

// ── Pan (view translation) state ────────────────────────────────────────
//
// The pan axis translates the view laterally in screen space without
// rotating the camera. Values are in the same normalized coord space as
// `project3` (roughly `[-1, 1]` covers the visible area at zoom = 1).

/// Read the current view pan as `(x, y)` in screen-normalized units.
/// Defaults to `(0.0, 0.0)`.
pub fn view_pan(s: &BState) -> (f32, f32) {
    let read = |field| BViewSlot::get(s, field).and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
    let (x, y) = (read(View3dKey::PanX), read(View3dKey::PanY));
    (x, y)
}

// ── File / import state keys ────────────────────────────────────────────
//
// There are no `*_key()` functions here any more. They returned exactly
// what their domain enum already returns — `loading_key()` was
// `SceneKey::Loading.path()` and nothing else — so they were an alias
// layer, and an alias layer is how the one-spelling-per-key rule
// (`tests/key_budget.rs`) got dodged: that test greps for hand-written
// paths, and a path wrapped in a function is invisible to it.
//
// Call sites now name the key. Most got shorter with it, because
// `BState::get / set / has / remove` take `impl BKey`: `s.get(&loading_key())`
// became `s.get(SceneKey::Loading)`. Only the places that genuinely need a
// `BPath` — `BAction::Set`, a `==` against a path, `.encode()` — still say
// `.path()`.

pub use super::load_stage::LoadStage;

/// Read the current import progress fraction (0..=1), or `None` when no import
/// is running. Clamped so a corrupt value can never overflow the bar.
pub fn view_load_progress(s: &BState) -> Option<f32> {
    s.get(crate::state::SceneKey::LoadProgress)
        .and_then(|v| v.parse::<f32>().ok())
        .map(|v| v.clamp(0.0, 1.0))
}

/// Read the current import stage label, or `None` when no import is running.
pub fn view_load_stage(s: &BState) -> Option<&str> {
    s.get(crate::state::SceneKey::LoadStage)
}

/// True iff `name` ends (case-insensitive) with the `.blend` suffix. Shared
/// predicate used by both the File_Browser gate and the import trigger.
pub fn is_blend_path(name: &str) -> bool {
    let suffix = ".blend";
    name.len() >= suffix.len() && name[name.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
}

// ── Zoom clamp + view yaw/zoom accessors ────────────────────────────────

/// Zoom clamp range — wide enough for the views Blender files save (a
/// close-up camera view is ~4×, a whole-city view ~0.05×). 1.0 is the default
/// framing; below 1 pulls back, above 1 pushes in.
pub const ZOOM_MIN: f32 = 0.02;
pub const ZOOM_MAX: f32 = 50.0;

/// Read the current view yaw in degrees (default 30° — Blender's startup
/// user-perspective angle).
pub fn view_yaw(s: &BState) -> f32 {
    BViewSlot::get(s, View3dKey::Yaw)
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(30.0)
}

/// Read the current view zoom factor (default 1.0), clamped to
/// `[ZOOM_MIN, ZOOM_MAX]` so a corrupt state value can never explode the
/// projection.
pub fn view_zoom(s: &BState) -> f32 {
    BViewSlot::get(s, View3dKey::Zoom)
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(1.0)
        .clamp(ZOOM_MIN, ZOOM_MAX)
}

/// Multiply `zoom` by `factor` and clamp to the allowed range. Scroll up
/// zooms in (factor > 1), scroll down zooms out (factor < 1).
pub fn zoom_step(zoom: f32, factor: f32) -> f32 {
    (zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX)
}
