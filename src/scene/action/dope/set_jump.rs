//! Key ▸ Jump to Keyframes — the one operator that moves the PLAYHEAD rather
//! than editing keys (`screen.keyframe_jump` acts on every key; over a
//! selection phiber jumps within it). No key changes, so the picks are left
//! as they are and `picks.write` never runs.

use super::BKeySelection;
use crate::state::{BState, CtxKey, log};

/// The playhead, as the Dope Sheet's operators read it (`ctx.frame`).
pub(super) fn frame_now(s: &BState) -> f32 {
    s.get(CtxKey::Frame).and_then(|v| v.parse::<f32>().ok()).unwrap_or(1.0)
}

pub(super) fn jump(s: &mut BState, picks: &BKeySelection) {
    let current = frame_now(s);
    let Some(nearest) = picks
        .0
        .iter()
        .map(|p| p.frame)
        .min_by(|a, b| (a - current).abs().total_cmp(&(b - current).abs()))
    else {
        log::push("No keyframes selected to jump to".to_string());
        return;
    };
    s.set(CtxKey::Frame, (nearest.round() as i32).to_string());
}
