//! Channel ▸ Expand / Collapse Channels — `ANIM_OT_channels_expand` and
//! `ANIM_OT_channels_collapse`, the Numpad `+` and `-` of every animation
//! editor's channel list.
//!
//! Its own rule on its own key (`ctx.op.channel_fold`), not another verb
//! under `channel.rs`'s: a fold is UI state kept per channel under
//! `ctx.dopesheet.expanded.<key>` — the same key the disclosure triangle
//! beside the row writes (`part::channels::time`) — where the rest of
//! `channel.rs` writes file overlays that a save puts back. Different thing
//! written, different rule; `BRule::wants` is what decides which runs, so
//! neither has to read a value to find out whether the value was meant for
//! it.
//!
//! One level per press. Blender filters these two with
//! `ANIMFILTER_LIST_VISIBLE` (`setflag_anim_channels`), so from a closed
//! list the objects open and the groups inside them only on the next press.
//! [`BTimeRow::list`] lists exactly the visible rows, so walking it gives
//! that behaviour for free.
//!
//! Blender's `all` property defaults to **true** — "Expand all channels
//! (not just selected ones)" — and the menu entries take the default, so
//! acting on every shown row is what the menu asks for, not the
//! selected-channel divergence `channel.rs` has to record.

use crate::host::BHostType;
use crate::state::val::{V_FALSE, V_TRUE};
use crate::state::{BKey as _, BPath, BState, DopeKey, OpKey, rule::BRule};
use crate::view::draw::canvas::time::curve::BAxisColors;
use crate::view::draw::canvas::time::row::BTimeRow;

/// Only a driver's swatch is coloured from these and no fold is, so any
/// will do (as `part::channels::time` does for its own listing).
const AXIS: BAxisColors = BAxisColors { x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], z: [0.0, 0.0, 1.0], w: [1.0, 1.0, 0.0], other: [0.0, 1.0, 1.0] };

/// `ctx.op.channel_fold` (see the module).
pub struct BChannelFoldRule;

impl BRule for BChannelFoldRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::ChannelFold.path()
    }

    /// `<editor>:expand` / `<editor>:collapse`, as `fire_host` spells it —
    /// the editor showing the menu folds its own channels, so the Dope
    /// Sheet's `+` cannot reach into the Graph editor's list.
    fn after(&self, _: &BPath, op: &str, s: &mut BState) {
        let Some((host, verb)) = op.split_once(':') else { return };
        let Some(host) = BHostType::from_slug(host) else { return };
        let open = match verb {
            "expand" => true,
            "collapse" => false,
            _ => return,
        };
        let rows = BTimeRow::list(s, host, AXIS);
        let keys: Vec<String> = rows.iter().filter_map(|row| row.fold().map(|(key, _)| key)).collect();
        for key in keys {
            s.set(DopeKey::Expanded(crate::state::intern(&key)), if open { V_TRUE } else { V_FALSE });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::blend::{BFCurve, BKey, IPO_BEZ};
    use crate::phiphx::scene::{SceneAction, SceneModel};
    use crate::state::CtxKey;
    use crate::state::action::apply_action;
    use crate::view::prim::BAction;

    /// A rig whose curves sit in two groups, so the list has two levels to
    /// open: the object, then its groups.
    fn model() -> SceneModel {
        let key = |frame: f32| BKey::new([frame, 0.0], [frame, 0.0], [frame, 0.0], IPO_BEZ);
        let curve = |path: &str, group: &str| BFCurve { group: Some(group.to_string()), ..BFCurve::new(path, 0, vec![key(1.0)]) };
        SceneModel {
            actions: vec![SceneAction {
                name: "Act".into(),
                users: vec!["Rig".into()],
                curves: vec![curve("location", "Object Transforms"), curve(r#"pose.bones["chest"].location"#, "chest")],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn rows(s: &BState) -> Vec<String> {
        BTimeRow::list(s, BHostType::Action, AXIS).iter().map(|row| row.name().to_string()).collect()
    }

    /// Through `apply_action`, so the test proves the rule is in the table
    /// and claims the key — not just that its body works.
    fn fold(s: &mut BState, op: &str) {
        apply_action(s, &BAction::Set(OpKey::ChannelFold.path(), op.to_string()));
    }

    #[test]
    fn expand_opens_the_rows_the_editor_shows_and_collapse_shuts_them() {
        crate::scene::source::set_loaded_scene(Some(model()));
        let mut s = BState::new();
        s.set(CtxKey::Object, "Rig");
        assert_eq!(rows(&s), ["Summary", "Rig"], "an object opens closed");

        // One press opens the object. Its groups come with it — closed,
        // because they were not on screen to be opened.
        fold(&mut s, "action:expand");
        assert_eq!(rows(&s), ["Summary", "Rig", "Object Transforms", "chest"]);

        // The next press reaches them.
        fold(&mut s, "action:expand");
        // The bone's curve carries its bone, as `getname_anim_fcurve` names it.
        assert_eq!(rows(&s), ["Summary", "Rig", "Object Transforms", "X Location", "chest", "X Location (chest)"]);

        // Collapse shuts every row in sight, so one press closes the lot.
        fold(&mut s, "action:collapse");
        assert_eq!(rows(&s), ["Summary", "Rig"]);
        crate::scene::source::set_loaded_scene(None);
    }

    #[test]
    fn the_value_names_the_editor_and_nothing_else_is_a_fold() {
        crate::scene::source::set_loaded_scene(Some(model()));
        let mut s = BState::new();
        s.set(CtxKey::Object, "Rig");
        fold(&mut s, "ipo:expand");
        assert!(BTimeRow::is_expanded(&s, "Rig"), "the Graph editor lists the same objects");

        // A value this rule cannot read leaves the folds alone rather than
        // guessing. (It can no longer be a channel property's: those are a
        // different key now, which is the point of the split.)
        for op in ["", "action", "action:wat", "nope:expand", "expand"] {
            let mut s = BState::new();
            s.set(CtxKey::Object, "Rig");
            fold(&mut s, op);
            assert!(!BTimeRow::is_expanded(&s, "Rig"), "{op:?} is not a fold");
        }
        crate::scene::source::set_loaded_scene(None);
    }
}
