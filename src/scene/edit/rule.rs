//! `ctx.op.scene_edit`: the operators that edit the scene — add a
//! primitive, a lamp, a camera or an empty, delete, duplicate, hide and
//! unhide the selection.

use super::{BAddKind, BSceneEdit, unique_name};
use crate::state::{BKey as _, BPath, BState, CtxKey, OpKey, log, rule::BRule, val::{V_FALSE, V_TRUE}};

/// `ctx.op.scene_edit` → `add:<shape>` / `delete` / `duplicate` / `hide` / `unhide`.
pub struct BSceneEditRule;

impl BRule for BSceneEditRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::SceneEdit.path()
    }
    fn after(&self, _: &BPath, op: &str, state: &mut BState) {
        let selection = crate::scene::select::BSelection::now(state);
        let mut edit = BSceneEdit::now(state);
        // Every name taken: the meshes drawn, the model's lamps, cameras and
        // empties, and what was added since the model in scope was built.
        let names: Vec<String> = crate::scene::source::with_loaded(|model| {
            let meshes: Vec<String> = crate::scene::source::scene_objects(state).iter().map(|o| o.name().to_string()).collect();
            let extras = model.into_iter().flat_map(|m| m.lights.iter().map(|l| &l.name).chain(m.cameras.iter().map(|c| &c.name)).chain(m.empties.iter().map(|e| &e.name)));
            let mut all: Vec<String> = meshes.into_iter().chain(extras.cloned()).chain(edit.added.iter().map(|(n, _)| n.clone())).collect();
            all.dedup();
            all
        });
        match op.split_once(':') {
            Some(("add", shape)) => {
                // A mesh primitive, or a lamp, camera or empty (`kind.rs`).
                let Some((_, label)) = BAddKind::of(shape) else { return };
                let name = unique_name(label, &names);
                edit.added.push((name.clone(), shape.to_string()));
                edit.write(state);
                // At the 3D cursor, aligned to the world (`object.add`'s
                // default `align='WORLD'`).
                let at = crate::scene::cursor::BCursor::now(state).location;
                if at != glam::Vec3::ZERO {
                    let mut transforms = crate::scene::transform::BTransforms::now(state);
                    transforms.set(&name, crate::scene::transform::BTransform { location: at.into(), rotation: [0.0; 3], scale: [1.0; 3] });
                    transforms.write(state);
                }
                state.set(CtxKey::Object, name.clone());
                crate::scene::select::BSelection { list: vec![name.clone()] }.write(state);
                log::push(format!("Added {name}"));
            }
            None if op == "delete" => {
                for name in &selection.list {
                    if let Some(i) = edit.added.iter().position(|(n, _)| n == name) {
                        edit.added.remove(i);
                    } else if let Some(i) = edit.copied.iter().position(|(n, _)| n == name) {
                        edit.copied.remove(i);
                    } else if !edit.deleted.contains(name) {
                        edit.deleted.push(name.clone());
                    }
                }
                edit.write(state);
                let mut links = crate::scene::links::BSceneLinks::now(state);
                selection.list.iter().for_each(|name| links.forget(name));
                links.write(state);
                let mut transforms = crate::scene::transform::BTransforms::now(state);
                selection.list.iter().for_each(|name| transforms.forget(name));
                transforms.write(state);
                let mut shadows = crate::scene::shadow::BObjectShadowList::now(state);
                selection.list.iter().for_each(|name| shadows.forget(name));
                shadows.write(state);
                let mut actions = crate::scene::action::BSceneActions::now(state);
                selection.list.iter().for_each(|name| actions.forget(name));
                actions.write(state);
                state.remove(CtxKey::Object);
                crate::scene::select::BSelection::default().write(state);
                log::push(format!("Deleted {} object(s)", selection.list.len()));
            }
            None if op == "duplicate" => {
                let mut taken = names;
                let mut copies = Vec::new();
                for source in &selection.list {
                    let copy = unique_name(source, &taken);
                    taken.push(copy.clone());
                    edit.copied.push((copy.clone(), source.clone()));
                    copies.push(copy);
                }
                edit.write(state);
                if let Some(first) = copies.first() {
                    state.set(CtxKey::Object, first.clone());
                }
                crate::scene::select::BSelection { list: copies.clone() }.write(state);
                log::push(format!("Duplicated {} object(s)", copies.len()));
            }
            None if op == "hide" => {
                for name in &selection.list {
                    state.set(&crate::part::outliner::key::k_restrict(name, "hide_viewport"), V_TRUE);
                }
                // What is hidden is not selected (`object.hide_view_set`).
                crate::scene::select::BSelection::default().write(state);
                log::push(format!("Hid {} object(s)", selection.list.len()));
            }
            None if op == "unhide" => {
                let hidden: Vec<String> = names
                    .iter()
                    .map(|name| crate::part::outliner::key::k_restrict(name, "hide_viewport").encode())
                    .filter(|key| state.get_encoded(key) == Some(V_TRUE))
                    .collect();
                for key in &hidden {
                    state.set_encoded(key, V_FALSE);
                }
                log::push("Revealed the hidden objects".to_string());
            }
            _ => {}
        }
        state.remove(OpKey::SceneEdit);
    }
}

