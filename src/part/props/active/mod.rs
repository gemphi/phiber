//! The active object's own values in the Properties editor — Blender's
//! `context.object` behind the Object and Material tabs. Structure: the
//! rule that follows the active object, and the entry the loader calls.
//! What is written, and from which field, is `set.rs`.

pub mod set;

use crate::phiphx::scene::SceneModel;
use crate::state::{BKey as _, BPath, BState, CtxKey, rule::BRule};

/// `ctx.object` → the Object and Material tabs, as Blender redraws the
/// Properties editor for whichever object is active.
pub struct BActiveObjectRule;

impl BRule for BActiveObjectRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == CtxKey::Object.path()
    }
    fn after(&self, _: &BPath, name: &str, s: &mut BState) {
        crate::scene::source::with_loaded(|loaded| {
            if let Some(model) = loaded {
                seed(s, model, name);
            }
        });
    }
}

/// Write `name`'s transform and first material into `ctx.buts.*`. Nothing
/// is written for an object the model does not have, so the panels keep
/// showing the last real values rather than zeros. The transform is the
/// object's own (`matrix_local`) — a child's is relative to its parent, as
/// Blender's panel shows it — for a camera, light or empty as for a mesh.
pub fn seed(state: &mut BState, model: &SceneModel, name: &str) {
    use crate::scene::transform::{BTransform, BTransforms};
    let edit = crate::scene::edit::BSceneEdit::now(state);
    match (model.objects.iter().find(|object| object.name == name), edit.added.iter().any(|(added, _)| added == name)) {
        (Some(object), _) => {
            set::seed_transform(state, object);
            set::seed_shadow(state, object.shadow);
            // A mesh with no material — an added primitive, the default
            // scene's (`edit::stage`) — shows the grey a first edit starts
            // from and a recolour gives it (`BRecolors::apply`), not the
            // last object's colour.
            let neutral = crate::phiphx::scene::SceneMaterial::neutral();
            set::seed_material(state, object.materials.first().unwrap_or(&neutral));
        }
        // Made this session (Add ▸ Mesh): no material yet, so the Surface
        // fields show the grey a first edit starts from, not the last
        // object's colour.
        (None, true) => set::seed_material(state, &crate::phiphx::scene::SceneMaterial::neutral()),
        (None, false) => {}
    }
    // What the panel switched for it this session, over what the file says.
    if let Some(shadow) = crate::scene::shadow::BObjectShadowList::now(state).get(name) {
        set::seed_shadow(state, shadow);
    }
    if let Some(recolor) = crate::scene::recolor::BRecolors::now(state).get(name) {
        set::seed_material(state, &recolor.material());
    }
    if let Some(local) = BTransforms::file_local(&edit, model, name) {
        set::seed_edited(state, &BTransform::of_matrix(local));
    }
    // What the panel set for it this session, over what the file says.
    if let Some(t) = crate::scene::transform::BTransforms::now(state).get(name) {
        set::seed_edited(state, &t);
    }
}
