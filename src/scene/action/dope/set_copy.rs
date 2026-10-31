//! Key ▸ Copy Keyframes and Paste Keyframes (`ACTION_OT_copy`,
//! `ACTION_OT_paste`; `space_action/action_edit.cc`'s `actkeys_copy_exec`
//! and `actkeys_paste_exec` over `animation/keyframes_general.cc`'s
//! `copy_animedit_keys` / `paste_animedit_keys`).
//!
//! Blender keeps the copied keys in a runtime buffer, not in the file, so
//! this one lives in `ctx.keys.buffer` (`CtxKey::KeyBuffer`) rather than in
//! the session's edits: it never reaches a save and never travels to a
//! collaborator. Each entry is an object, a frame **relative to the
//! earliest copied frame**, and the channel values read off the curves
//! there — so a paste lands the first copied column on the playhead and
//! keeps the spacing of the rest, which is Blender's default offset mode
//! (`KEYFRAME_PASTE_OFFSET_CFRA_START`).
//!
//! Divergences, both deliberate:
//! - Blender's buffer can be pasted onto *other* selected channels (it
//!   matches by channel path). This pastes back onto the objects it was
//!   copied from, which is the same result in the common case and never
//!   silently writes to a channel the user did not copy.
//! - Merge mode is Blender's default `MIX`: a pasted frame overwrites a key
//!   already on that frame (one key per frame per curve is phiber's model,
//!   as `keys::BKeyInsert::apply` has it) and leaves the others alone.

use super::super::BSceneActions;
use super::super::keys::{BKeyInsert, BKeyed};
use super::super::retime::BKeyStep;
use super::{BKeyPick, BKeySelection, BKeyTarget};
use crate::phiphx::scene::SceneModel;
use crate::state::{BState, CtxKey, log};
use std::sync::Arc;

const SEP: char = '|';
const AND: char = '#';
const IS: char = '=';
const EACH: char = ';';
const PART: char = ':';

/// One copied column: an object, its frame relative to the earliest copied
/// one, and the values the curves held there.
#[derive(Clone, Debug, PartialEq)]
pub struct BKeyCopy {
    pub object: String,
    /// Frames after the earliest column in the buffer; the first is 0.
    pub at: f32,
    pub channels: Vec<BKeyed>,
}

impl BKeyCopy {
    /// The buffer as `ctx.keys.buffer` spells it.
    pub fn parse_all(raw: &str) -> Vec<Self> {
        raw.split(SEP).filter_map(Self::parse).collect()
    }

    pub fn encode_all(list: &[Self]) -> String {
        list.iter().map(Self::encode).collect::<Vec<_>>().join(&SEP.to_string())
    }

    fn parse(entry: &str) -> Option<Self> {
        let (head, body) = entry.split_once(IS)?;
        // From the right: an object's name may hold the separator.
        let (object, at) = head.rsplit_once(AND)?;
        let channels = body
            .split(EACH)
            .filter_map(|c| {
                // From the right as well: a path may hold `:`.
                let mut part = c.rsplitn(3, PART);
                let (value, index, path) = (part.next()?.parse().ok()?, part.next()?.parse().ok()?, part.next()?);
                Some(BKeyed { path: path.to_string(), index, value })
            })
            .collect();
        Some(Self { object: object.to_string(), at: at.parse().ok()?, channels })
    }

    fn encode(&self) -> String {
        let channels: Vec<String> = self.channels.iter().map(|c| format!("{}{PART}{}{PART}{}", c.path, c.index, c.value)).collect();
        format!("{}{AND}{}{IS}{}", self.object, self.at, channels.join(&EACH.to_string()))
    }
}

/// `copy` and `paste`. Returns the picks to write, or `None` when the arm
/// has already said its piece and nothing is to be written.
pub(super) fn copy_paste(op: &str, s: &mut BState, model: &Arc<SceneModel>, picks: BKeySelection) -> Option<BKeySelection> {
    match op {
        "copy" => {
            if picks.0.is_empty() {
                log::push("No keyframes selected to copy".to_string());
                return None;
            }
            // The curves as the session has them, so a copy takes the keys
            // the user can see rather than the ones the file was saved with.
            let before = BSceneActions::now(s).keyed_actions(&super::super::set::BActionScene::raw(model.clone()));
            let first = picks.0.iter().map(|p| p.frame).fold(f32::INFINITY, f32::min);
            let mut buffer: Vec<BKeyCopy> = Vec::new();
            for pick in &picks.0 {
                let mut channels: Vec<BKeyed> = Vec::new();
                for target in BKeyTarget::of(&before, &pick.object) {
                    if target.slot.is_empty() || !target.columns.iter().any(|c| (c - pick.frame).abs() < 5e-4) {
                        continue;
                    }
                    let Some(action) = before.iter().find(|a| a.name == target.action) else { continue };
                    let Some(slot) = action.slots.iter().find(|sl| sl.identifier == target.slot) else { continue };
                    channels.extend(
                        action.curves[slot.curves.clone()]
                            .iter()
                            .filter(|c| c.keys.iter().any(|k| (k.co[0] - pick.frame).abs() < 5e-4))
                            .map(|c| BKeyed { path: c.path.clone(), index: c.index, value: c.evaluate(pick.frame) }),
                    );
                }
                if !channels.is_empty() {
                    buffer.push(BKeyCopy { object: pick.object.clone(), at: pick.frame - first, channels });
                }
            }
            if buffer.is_empty() {
                log::push("No keyframes to copy".to_string());
                return None;
            }
            log::push(format!("Copied {} keyframe column(s)", buffer.len()));
            s.set(CtxKey::KeyBuffer, BKeyCopy::encode_all(&buffer));
            // A copy changes nothing and leaves the selection as it was.
            None
        }
        "paste" => {
            let buffer = BKeyCopy::parse_all(s.get(CtxKey::KeyBuffer).unwrap_or(""));
            if buffer.is_empty() {
                log::push("No keyframes in the copy buffer".to_string());
                return None;
            }
            // Blender's default offset: the earliest copied column lands on
            // the playhead.
            let current = s.get(CtxKey::Frame).and_then(|v| v.parse::<f32>().ok()).unwrap_or(1.0);
            let mut edits = BSceneActions::now(s);
            let before = edits.keyed_actions(&super::super::set::BActionScene::raw(model.clone()));
            let mut landed: Vec<BKeyPick> = Vec::new();
            for entry in &buffer {
                let to = current + entry.at;
                let mut any = false;
                for target in BKeyTarget::of(&before, &entry.object) {
                    if target.slot.is_empty() {
                        continue;
                    }
                    let Some(action) = before.iter().find(|a| a.name == target.action) else { continue };
                    let Some(slot) = action.slots.iter().find(|sl| sl.identifier == target.slot) else { continue };
                    // Only the channels this slot actually carries: a buffer
                    // copied from one object must not invent curves on
                    // another's slot.
                    let run = &action.curves[slot.curves.clone()];
                    let channels: Vec<BKeyed> = entry
                        .channels
                        .iter()
                        .filter(|c| run.iter().any(|curve| curve.path == c.path && curve.index == c.index))
                        .cloned()
                        .collect();
                    if channels.is_empty() {
                        continue;
                    }
                    edits.keyed.push(BKeyStep::Insert(BKeyInsert { action: target.action.clone(), slot: target.slot.clone(), frame: to, channels }));
                    any = true;
                }
                if any {
                    landed.push(BKeyPick { object: entry.object.clone(), frame: to });
                }
            }
            if landed.is_empty() {
                log::push("Nothing in the buffer matched a channel here".to_string());
                return None;
            }
            edits.write(s);
            log::push(format!("Pasted {} keyframe column(s)", landed.len()));
            // The pasted keys are what is selected, as Blender leaves them.
            Some(BKeySelection(landed))
        }
        _ => None,
    }
}
