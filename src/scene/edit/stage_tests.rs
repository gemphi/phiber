//! `stage.rs`'s tests: the stage is the Default_Scene where it is drawn,
//! it stands in only once the session builds on it, and with no file open
//! an added Sun lights the scene, keys play and the default Cube can be
//! keyed and recoloured.

use super::*;
use crate::scene::motion::lights_at;
use crate::scene::source::{clear_scene_objects, loaded_shared, scene_objects, set_loaded_at};
use crate::state::action::apply_action;
use crate::state::val::BValue;
use crate::state::{BKey as _, BPath, ButsKey, CtxKey, OpKey};
use crate::view::prim::BAction;

/// A draw pass with no file open.
fn pass(state: &BState) -> Option<Arc<SceneModel>> {
    set_loaded_at(None, state);
    loaded_shared()
}

fn act(state: &mut BState, path: BPath, value: &str) {
    apply_action(state, &BAction::Set(path, value.to_string()));
    pass(state);
}

fn translation(state: &BState, name: &str) -> Option<[f32; 3]> {
    scene_objects(state).iter().find(|o| o.name() == name).map(|o| match o {
        crate::scene::BObject3D::Loaded { matrix_world, .. } => [matrix_world[3][0], matrix_world[3][1], matrix_world[3][2]],
        crate::scene::BObject3D::Primitive { x, y, .. } => [x / MESH_SCALE, y / MESH_SCALE, 0.0],
    })
}

#[test]
fn the_stage_is_the_default_scene_where_it_is_drawn() {
    let stage = BStage::model();
    assert!(Arc::ptr_eq(&stage, &BStage::model()), "built once");
    assert_eq!(stage.objects.iter().map(|o| o.name.as_str()).collect::<Vec<_>>(), OBJECTS.iter().map(|o| o.name).collect::<Vec<_>>());
    for (object, drawn) in stage.objects.iter().zip(OBJECTS) {
        assert_eq!(object.matrix_world[3], [drawn.x / MESH_SCALE, drawn.y / MESH_SCALE, 0.0, 1.0], "{}", drawn.name);
    }
    assert_eq!(stage.collections[0].children[0].object_indices, [0, 1, 2], "in the Scene Collection's Collection");
    assert!(stage.source_path.is_none() && !stage.motion.animated());
}

#[test]
fn the_stage_stands_in_only_once_the_session_builds_on_the_default_scene() {
    clear_scene_objects();
    let mut state = BState::new();
    assert!(pass(&state).is_none(), "nothing built: the Default_Scene, no model");
    assert!(scene_objects(&state).iter().all(|o| matches!(o, crate::scene::BObject3D::Primitive { .. })));
    act(&mut state, OpKey::SceneEdit.path(), "add:light_sun");
    let scene = pass(&state).expect("a Sun added: the stage stands in");
    assert_eq!(scene.lights.iter().map(|l| l.name.as_str()).collect::<Vec<_>>(), ["Sun"]);
    let names: Vec<String> = scene_objects(&state).iter().map(|o| o.name().to_string()).collect();
    assert_eq!(names, ["Cube", "Sphere", "Gem"], "the default objects stay where they were");
    assert_eq!(translation(&state, "Cube"), Some([OBJECTS[0].x / MESH_SCALE, OBJECTS[0].y / MESH_SCALE, 0.0]));
    // Taken back, the Default_Scene is drawn as it was.
    BSceneEdit::default().write(&mut state);
    assert!(pass(&state).is_none());
}

/// No file open: Add ▸ Light ▸ Sun, its rotation keyed at 1 and 60; the
/// default Cube keyed too (I with nothing added first would do it alone),
/// then turned red in the Material tab.
#[test]
fn with_no_file_open_a_sun_keys_play_and_the_default_cube_is_keyed_and_red() {
    clear_scene_objects();
    let mut state = BState::new();
    pass(&state);
    // I on the default Cube with nothing else done: the stage is keyed.
    act(&mut state, CtxKey::Frame.path(), "1");
    crate::scene::select::BSelection { list: vec!["Cube".into()] }.write(&mut state);
    act(&mut state, CtxKey::Object.path(), "Cube");
    act(&mut state, OpKey::Keyframe.path(), "insert");
    assert!(pass(&state).is_some(), "a key on the default Cube makes the default scene a scene");
    let start = translation(&state, "Cube").unwrap();
    act(&mut state, CtxKey::Frame.path(), "60");
    act(&mut state, CtxKey::Object.path(), "Cube");
    act(&mut state, ButsKey::Axis("location", "z").path(), "3");
    act(&mut state, OpKey::Keyframe.path(), "insert");
    // The Sun, keyed turning.
    act(&mut state, OpKey::SceneEdit.path(), "add:light_sun");
    for (frame, angle) in [("1", "10"), ("60", "70")] {
        act(&mut state, CtxKey::Frame.path(), frame);
        act(&mut state, CtxKey::Object.path(), "Sun");
        crate::scene::select::BSelection { list: vec!["Sun".into()] }.write(&mut state);
        act(&mut state, ButsKey::Axis("rotation_euler", "x").path(), angle);
        act(&mut state, OpKey::Keyframe.path(), "insert");
    }
    // The Cube's Base Color.
    act(&mut state, CtxKey::Object.path(), "Cube");
    act(&mut state, ButsKey::Value("diffuse_color").path(), &BValue::Color([0.8, 0.05, 0.05, 1.0]).encode());

    let scene = pass(&state).unwrap();
    assert!(scene.motion.animated());
    let sun_y = |frame: f32| lights_at(&scene, Some(frame))[0].matrix_world[2][1];
    assert!((sun_y(1.0) + 10f32.to_radians().sin()).abs() < 1e-4 && (sun_y(60.0) + 70f32.to_radians().sin()).abs() < 1e-4, "{} {}", sun_y(1.0), sun_y(60.0));
    act(&mut state, CtxKey::Frame.path(), "1");
    assert_eq!(translation(&state, "Cube"), Some(start));
    act(&mut state, CtxKey::Frame.path(), "60");
    assert_eq!(translation(&state, "Cube").map(|t| t[2]), Some(3.0), "the default Cube's key plays");
    let red = scene_objects(&state).iter().find(|o| o.name() == "Cube").and_then(|o| match o {
        crate::scene::BObject3D::Loaded { materials, .. } => materials.first().map(|m| m.base_color),
        crate::scene::BObject3D::Primitive { .. } => None,
    });
    assert_eq!(red, Some([0.8, 0.05, 0.05, 1.0]));
}
