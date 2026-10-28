//! The default scene, made a scene. With no file open the viewport draws
//! phiber's Default_Scene — `OBJECTS`, a Cube, a Sphere and a Gem with no
//! model behind them — so nothing in it could be keyed or recoloured, and
//! nothing added to it had anywhere to live: a lamp, a camera or a key is
//! only ever in a model.
//!
//! Once the session builds on it — adds an object, or keys one — the stage
//! stands in for the file: the three primitives as meshes where they are
//! drawn, in the Scene Collection's Collection, each with a still motion,
//! and Blender's factory timeline, world and render settings. The draw
//! pass scopes it in place of nothing (`source::set_loaded_at`), and from
//! there the session's objects and keys go over it as over a file's.
//! Undone back to nothing, the Default_Scene is drawn as before.
//!
//! It is not a file: nothing is read, and Save still asks for one.

use super::BSceneEdit;
use crate::phiphx::blend::{BlendMotion, ObjectTransformRaw};
use crate::phiphx::scene::{IDENTITY_M4, SceneCollection, SceneModel, SceneObject, SceneUvMap};
use crate::scene::{MESH_SCALE, OBJECTS};
use crate::state::BState;
use std::cell::OnceCell;
use std::sync::Arc;

thread_local! {
    /// Built once: the object list and the edits over it are memoised on
    /// the model's identity, so the stage hands out the same `Arc`.
    static STAGE: OnceCell<Arc<SceneModel>> = const { OnceCell::new() };
}

/// The Default_Scene as a model.
pub struct BStage;

impl BStage {
    /// The Default_Scene's primitives as meshes where the viewport draws
    /// them (`(x, y)` / `MESH_SCALE` on the ground), in a Collection under
    /// the Scene Collection, as Blender's startup file keeps its objects.
    pub fn model() -> Arc<SceneModel> {
        STAGE.with(|cell| Arc::clone(cell.get_or_init(|| Arc::new(Self::build()))))
    }

    fn build() -> SceneModel {
        let mut model = SceneModel::default();
        for (i, o) in OBJECTS.iter().enumerate() {
            let location = [o.x / MESH_SCALE, o.y / MESH_SCALE, 0.0];
            let mut matrix_world = IDENTITY_M4;
            matrix_world[3] = [location[0], location[1], location[2], 1.0];
            model.motion.motion.list.push(BlendMotion::still(o.name, ObjectTransformRaw { loc: location, ..Default::default() }));
            model.motion.object.push(i);
            model.object_types.push("MESH".to_string());
            model.locals.push(matrix_world);
            model.uv_maps.push(Arc::new(SceneUvMap::default()));
            model.objects.push(SceneObject {
                name: o.name.to_string(),
                mesh: Arc::new(super::shape::mesh_of(o.shape.mesh_type())),
                location,
                scale: [1.0; 3],
                matrix_world,
                materials: Vec::new(),
                material_indices: Default::default(),
                uvs: Default::default(),
                color: [1.0; 4],
                shadow: Default::default(),
            });
        }
        let collection = |name: &str, children: Vec<SceneCollection>, object_indices: Vec<usize>| SceneCollection {
            name: name.to_string(),
            children,
            object_indices,
            extra_names: Vec::new(),
            exclude: false,
            hide_viewport: false,
            visibility: Default::default(),
        };
        model.collections = vec![collection("Scene Collection", vec![collection("Collection", Vec::new(), (0..OBJECTS.len()).collect())], Vec::new())];
        model
    }

    /// Has the session built on the default scene — added an object, or
    /// keyed one?
    pub fn wanted(state: &BState) -> bool {
        !BSceneEdit::now(state).made().is_empty() || !crate::scene::action::BSceneActions::now(state).is_empty()
    }

    /// The model the session's edits go over: the file's, else the stage
    /// once the session built on the default scene, else none.
    pub fn under(model: Option<Arc<SceneModel>>, state: &BState) -> Option<Arc<SceneModel>> {
        model.or_else(|| Self::wanted(state).then(Self::model))
    }
}

#[cfg(test)]
#[path = "stage_tests.rs"]
mod tests;
