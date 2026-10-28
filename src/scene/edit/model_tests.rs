//! `model.rs`'s tests: what Add makes stands in the model as a file's
//! object would — mesh, lamp, camera, empty, each with a motion — and a key
//! on it plays, with no `.blend` anywhere.

use super::*;
use crate::scene::motion::{active_camera_at, lights_at};
use crate::scene::source::{clear_scene_objects, loaded_shared, set_loaded_at};
use crate::state::action::apply_action;
use crate::state::{BKey as _, ButsKey, CtxKey, OpKey};
use crate::view::prim::BAction;

/// A draw pass: the model the state's edits make, scoped at its frame.
fn pass(state: &BState, file: &Arc<SceneModel>) -> Arc<SceneModel> {
    set_loaded_at(Some(Arc::clone(file)), state);
    loaded_shared().expect("a model in scope")
}

fn op(state: &mut BState, file: &Arc<SceneModel>, path: crate::state::BPath, value: &str) {
    apply_action(state, &BAction::Set(path, value.to_string()));
    pass(state, file);
}

#[test]
fn what_add_makes_stands_in_the_model_with_a_motion_each() {
    let mut edit = BSceneEdit::default();
    for (name, slug) in [("Cube", "cube"), ("Sun", "light_sun"), ("Camera", "camera"), ("Empty", "empty_cube"), ("Nothing", "teapot")] {
        edit.added.push((name.into(), slug.into()));
    }
    edit.copied.push(("Sun.001".into(), "Sun".into()));
    let model = edit.model(&SceneModel::default());
    assert_eq!(model.objects.iter().map(|o| o.name.as_str()).collect::<Vec<_>>(), ["Cube"]);
    assert_eq!(model.lights.iter().map(|l| (l.name.as_str(), l.kind.as_str())).collect::<Vec<_>>(), [("Sun", "SUN"), ("Sun.001", "SUN")], "a duplicate of an added lamp is a lamp");
    assert_eq!((model.cameras[0].name.as_str(), model.empties[0].display_type.as_str()), ("Camera", "CUBE"));
    let motion = |i: usize| model.motion.motion.list[i].name.as_str();
    assert_eq!(motion(model.motion.object[0]), "Cube");
    assert_eq!((motion(model.motion.light[0]), motion(model.motion.light[1])), ("Sun", "Sun.001"));
    assert_eq!((motion(model.motion.camera[0]), motion(model.motion.empty[0])), ("Camera", "Empty"));
    assert_eq!(model.motion.motion.list[model.motion.light[0]].transform.rotmode, 1, "XYZ Euler, as Blender adds an object");
    // Built over it again, nothing is added twice.
    assert_eq!(edit.model(&model).motion.motion.list.len(), model.motion.motion.list.len());
}

#[test]
fn the_same_edits_over_the_same_model_build_once() {
    let mut state = BState::new();
    let file = Arc::new(SceneModel::default());
    assert!(Arc::ptr_eq(&BEditScene::over(Arc::clone(&file), &state), &file), "nothing added: the file's own model");
    BSceneEdit { added: vec![("Sun".into(), "light_sun".into())], ..Default::default() }.write(&mut state);
    let a = BEditScene::over(Arc::clone(&file), &state);
    assert!(Arc::ptr_eq(&a, &BEditScene::over(Arc::clone(&file), &state)));
    assert_eq!(a.lights.len(), 1);
}

/// Add ▸ Light ▸ Sun, its rotation typed in and keyed at 1 and at 60 — the
/// Sun turns between, and the cube keyed the same way moves; the camera
/// added looks from where it was put.
#[test]
fn keys_on_added_objects_play() {
    clear_scene_objects();
    let file = Arc::new(SceneModel::default());
    let mut state = BState::new();
    pass(&state, &file);
    let key_at = |state: &mut BState, frame: &str, object: &str, field: &'static str, axis: &'static str, value: &str| {
        state.set(CtxKey::Frame, frame);
        pass(state, &file);
        op(state, &file, CtxKey::Object.path(), object);
        crate::scene::select::BSelection { list: vec![object.to_string()] }.write(state);
        op(state, &file, ButsKey::Axis(field, axis).path(), value);
        op(state, &file, OpKey::Keyframe.path(), "insert");
    };
    for slug in ["light_sun", "cube", "camera"] {
        op(&mut state, &file, OpKey::SceneEdit.path(), &format!("add:{slug}"));
    }
    key_at(&mut state, "1", "Sun", "rotation_euler", "x", "10");
    key_at(&mut state, "60", "Sun", "rotation_euler", "x", "70");
    key_at(&mut state, "1", "Cube", "location", "x", "-2");
    key_at(&mut state, "60", "Cube", "location", "x", "2");
    op(&mut state, &file, CtxKey::Object.path(), "Camera");
    op(&mut state, &file, ButsKey::Axis("location", "z").path(), "12");

    let scene = pass(&state, &file);
    assert!(scene.motion.animated(), "the keys move the scene");
    let sun_z = |frame: f32| lights_at(&scene, Some(frame))[0].matrix_world[2];
    let (first, last) = (sun_z(1.0), sun_z(60.0));
    assert!((first[1] - (-(10f32.to_radians().sin()))).abs() < 1e-4, "frame 1: turned 10° about X: {first:?}");
    assert!((last[1] - (-(70f32.to_radians().sin()))).abs() < 1e-4, "frame 60: turned 70°: {last:?}");
    let mut cube_x = |frame: &str| {
        state.set(CtxKey::Frame, frame);
        pass(&state, &file);
        crate::scene::scene_objects(&state).iter().find(|o| o.name() == "Cube").map(|o| match o {
            crate::scene::BObject3D::Loaded { matrix_world, .. } => matrix_world[3][0],
            crate::scene::BObject3D::Primitive { x, .. } => *x,
        })
    };
    assert_eq!(cube_x("1"), Some(-2.0));
    assert_eq!(cube_x("60"), Some(2.0));
    let between = cube_x("30").unwrap();
    assert!(between > -2.0 && between < 2.0, "played between the keys: {between}");
    assert_eq!(active_camera_at(&loaded_shared().unwrap(), Some(30.0)).map(|c| c.matrix_world[3][2]), Some(12.0), "the camera stands where it was put");
}
