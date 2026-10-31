//! The Action Editor's slot rows (`animfilter_action_slot`).
//!
//! Split from `mod.rs` so each file sits under the 240-line read cap
//! (`tests/level_structure.rs`).

use super::{BRowKind, BTimeRow};
use crate::state::BState;

impl BTimeRow {
    /// The Action Editor's channels under its summary: each slot of the
    /// active object's action with its keys, and when open its groups and
    /// curves.
    pub(super) fn action_editor(s: &BState, model: &crate::phiphx::scene::SceneModel) -> Vec<Self> {
        let Some(object) = s.get(crate::state::CtxKey::Object).filter(|o| !o.is_empty()) else { return Vec::new() };
        let edits = crate::scene::action::BSceneActions::now(s);
        let Some(action) = edits.of(object, model).and_then(|(name, _)| edits.action(&name, model)) else { return Vec::new() };
        let mut out = Vec::new();
        for slot in &action.slots {
            let curves: Vec<&crate::phiphx::blend::BFCurve> = action.curves.get(slot.curves.clone()).unwrap_or(&[]).iter().collect();
            let (keys, keytypes) = crate::scene::action::dope::fold_columns(
                curves.iter().flat_map(|c| c.keys.iter().map(|k| (k.co[0], k.keytype))).collect::<Vec<_>>(),
            );
            let user = action.bindings.iter().find(|(_, h)| *h == slot.handle).map(|(o, _)| o.clone());
            let owner = user.unwrap_or_else(|| if slot.name().is_empty() { object.to_string() } else { slot.name().to_string() });
            let fold = Self::slot_key(&action.name, &slot.identifier);
            let expanded = s.get(crate::state::DopeKey::Expanded(crate::state::intern(&fold))) != Some(crate::state::val::V_FALSE);
            out.push(Self { kind: BRowKind::Slot { object: owner.clone(), action: action.name.clone(), identifier: slot.identifier.clone(), expanded }, keys, keytypes });
            if expanded {
                out.extend(Self::grouped(s, &owner, curves));
            }
        }
        out
    }
}
