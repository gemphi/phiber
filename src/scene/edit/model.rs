//! The objects the session added, in the model as reading them from a file
//! would have put them: a mesh in `objects`, a lamp in `lights`, a camera in
//! `cameras`, an empty in `empties` — each with a motion of its own
//! (`SceneMotion`), at its rest place (the origin, world-aligned; G and the
//! Transform panel move it from there as they move a file's object).
//!
//! That motion is what lets a key play. I keys an added object as it keys a
//! file's (`action/keying.rs`), and the session's actions drive whoever has
//! a motion of that name (`action/set.rs`): with one, the object stands
//! where its curves put it in the viewport, in playback and in F12 and
//! Ctrl F12, its lamp lights from there and its camera looks from there.
//! Without one, nothing played the keys back.
//!
//! Built once per edit and model (`BEditScene`), like the actions over it.

use super::BSceneEdit;
use super::kind::{self, BAddKind};
use crate::phiphx::blend::{BlendMotion, ObjectTransformRaw};
use crate::phiphx::scene::{IDENTITY_M4, SceneModel, SceneObject, SceneUvMap};
use crate::state::BState;
use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    /// The last model built: from which model, for which edits.
    static BUILT: RefCell<Option<(Arc<SceneModel>, BSceneEdit, Arc<SceneModel>)>> = const { RefCell::new(None) };
}

/// The session's added objects over a loaded model.
pub struct BEditScene;

impl BEditScene {
    /// `model` with the session's added objects in — the same `Arc` when
    /// there are none, the one built last when nothing changed.
    pub fn over(model: Arc<SceneModel>, state: &BState) -> Arc<SceneModel> {
        let edit = BSceneEdit::now(state);
        if edit.made().is_empty() {
            return model;
        }
        if let Some(hit) = BUILT.with(|cell| cell.borrow().as_ref().filter(|(m, e, _)| Arc::ptr_eq(m, &model) && *e == edit).map(|(_, _, built)| Arc::clone(built))) {
            return hit;
        }
        let built = Arc::new(edit.model(&model));
        BUILT.with(|cell| *cell.borrow_mut() = Some((model, edit, Arc::clone(&built))));
        built
    }
}

/// `list[len]` is `at`: an index list one short per entry the file gave no
/// motion is padded with an index nothing has, so the new entry lines up.
fn index(list: &mut Vec<usize>, len: usize, at: usize) {
    list.resize(len, usize::MAX);
    list.push(at);
}

impl BSceneEdit {
    /// Every object the session made, `(name, slug)`: the added ones, and
    /// the duplicates of an added one (which are what their source is).
    pub fn made(&self) -> Vec<(String, String)> {
        let copies = self.copied.iter().filter_map(|(copy, source)| self.added.iter().find(|(name, _)| name == source).map(|(_, slug)| (copy.clone(), slug.clone())));
        self.added.iter().cloned().chain(copies).collect()
    }

    /// `model` with the objects the session made in it.
    pub fn model(&self, model: &SceneModel) -> SceneModel {
        let mut out = model.clone();
        for (name, slug) in self.made() {
            let Some((kind, _)) = BAddKind::of(&slug) else { continue };
            let taken = out.objects.iter().map(|o| &o.name).chain(out.lights.iter().map(|l| &l.name)).chain(out.cameras.iter().map(|c| &c.name)).chain(out.empties.iter().map(|e| &e.name)).any(|n| *n == name);
            let mesh = super::primitive_mesh(&slug);
            if taken || (kind == BAddKind::Mesh && mesh.is_none()) {
                continue;
            }
            let at = out.motion.motion.list.len();
            out.motion.motion.list.push(BlendMotion::still(name.clone(), ObjectTransformRaw { ob_type: kind.ob_type(), ..Default::default() }));
            match (kind, mesh) {
                (BAddKind::Mesh, None) => {}
                (BAddKind::Mesh, Some(mesh)) => {
                    let n = out.objects.len();
                    // The lists that run beside `objects` grow with it when
                    // the file filled them.
                    if out.object_types.len() == n {
                        out.object_types.push(kind.object_type().to_string());
                    }
                    if out.locals.len() == n {
                        out.locals.push(IDENTITY_M4);
                    }
                    if out.uv_maps.len() == n {
                        out.uv_maps.push(Arc::new(SceneUvMap::default()));
                    }
                    index(&mut out.motion.object, n, at);
                    out.objects.push(SceneObject {
                        name,
                        mesh,
                        location: [0.0; 3],
                        scale: [1.0; 3],
                        matrix_world: IDENTITY_M4,
                        materials: Vec::new(),
                        material_indices: Default::default(),
                        uvs: Default::default(),
                        color: [1.0; 4],
                        shadow: Default::default(),
                    });
                }
                (BAddKind::Light(lamp), _) => {
                    index(&mut out.motion.light, out.lights.len(), at);
                    out.extra_locals.push((name.clone(), IDENTITY_M4));
                    out.lights.push(kind::light(&name, lamp));
                }
                (BAddKind::Camera, _) => {
                    index(&mut out.motion.camera, out.cameras.len(), at);
                    out.extra_locals.push((name.clone(), IDENTITY_M4));
                    out.cameras.push(kind::camera(&name));
                }
                (BAddKind::Empty(shape), _) => {
                    index(&mut out.motion.empty, out.empties.len(), at);
                    out.extra_locals.push((name.clone(), IDENTITY_M4));
                    out.empties.push(kind::empty(&name, shape));
                }
            }
        }
        out
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
