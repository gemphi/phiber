//! Folding keys into columns, with the Keyframe Type each column takes.
//!
//! Split from `mod.rs` so each file sits under the 240-line read cap
//! (`tests/level_structure.rs`).

use crate::phiphx::blend::KEYTYPE_KEYFRAME;

/// Keys at the same frame folded into one column, with the Keyframe Type
/// the column takes — Blender's `nupdate_ak_bezt`
/// (`animation/keyframes_keylist.cc`): a **'proper' keyframe has priority
/// over every other type**, so a column holding one is drawn as a keyframe
/// however many breakdowns or jitters share its frame; failing that, the
/// first key's type stands. Blender's own comment says the priority is over
/// "breakdowns (and other types for now)", so there is no ordering between
/// the non-keyframe types and the first one seen wins.
///
/// Frames are rounded to a thousandth, as `BKeyTarget::of` rounds them, so
/// two curves keyed at "the same" frame really do fold.
///
/// Returns `(frames, types)`, the same length and in frame order.
pub fn fold_columns(keys: impl IntoIterator<Item = (f32, u8)>) -> (Vec<f32>, Vec<u8>) {
    let mut list: Vec<(f32, u8)> = keys.into_iter().map(|(f, t)| ((f * 1000.0).round() / 1000.0, t)).collect();
    list.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (mut frames, mut types): (Vec<f32>, Vec<u8>) = (Vec::new(), Vec::new());
    for (frame, keytype) in list {
        match frames.last() {
            // Same column: only a proper keyframe changes what it is.
            Some(&last) if (last - frame).abs() < 5e-4 => {
                if keytype == KEYTYPE_KEYFRAME {
                    if let Some(t) = types.last_mut() {
                        *t = KEYTYPE_KEYFRAME;
                    }
                }
            }
            _ => {
                frames.push(frame);
                types.push(keytype);
            }
        }
    }
    (frames, types)
}
