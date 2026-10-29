//! A cube dropped on a plane (`tests/fixtures/make_drop.py`): the bodies
//! phiray is handed, the fall against Blender's own, a scrub that reads
//! the bake, and a settings edit that bakes again.

use super::*;
use crate::phiphx::blend::BlendFile;
use crate::state::CtxKey;

fn drop_scene() -> Arc<SceneModel> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/drop.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    Arc::new(SceneModel::from_blend_owned(scene, &path.to_string_lossy()))
}

/// Where the Cube stands at `frame`, as F12 would see it.
fn cube_z(model: &Arc<SceneModel>, s: &mut BState, frame: i32) -> f32 {
    s.set(CtxKey::Frame, frame.to_string());
    crate::scene::source::set_loaded_exact(Some(Arc::clone(model)), s);
    let loaded = crate::scene::source::loaded_shared();
    let scene = crate::scene::current_scene(s, loaded.as_deref());
    let cube = scene.iter().find(|o| o.name() == "Cube").expect("the Cube");
    match cube {
        crate::scene::BObject3D::Loaded { matrix_world, .. } => matrix_world[3][2],
        _ => panic!("the Cube is a loaded object"),
    }
}

fn run() -> u64 {
    bake::with(|b| b.run).unwrap_or_default()
}

#[test]
fn the_solver_gets_object_space_triangles_and_the_raw_world() {
    let model = drop_scene();
    assert!(moves(&model), "an active box falls in the file's world");
    let input = BSimInput::of(&model, &BWorldRows::file(model.physics.world.as_ref().unwrap())).expect("a world with bodies");
    let cube = input.objects.iter().find(|o| o.name == "Cube").expect("the Cube's body");
    // The factory cube is ±1 in object space; its scale of 0.5 stays in
    // the matrix, for `from_blend` to take out once.
    let reach = cube.tris.iter().flatten().fold(0.0f32, |m, v| m.max(v[0].abs()));
    assert!((reach - 1.0).abs() < 1e-5, "object space, not world: {reach}");
    assert!((cube.world[0][0] - 0.5).abs() < 1e-5 && (cube.world[3][2] - 3.0).abs() < 1e-5);
    assert_eq!(cube.body.shape, phiray::sim::BlendShape::Box);
    assert!(input.objects.iter().any(|o| o.name == "Ground" && o.body.passive && o.body.shape == phiray::sim::BlendShape::Mesh));
    assert_eq!((input.start, input.end, input.fps), (1, 60, 24.0));
    assert!((input.world.gravity[2] + 9.81).abs() < 1e-4, "{:?}", input.world.gravity);
}

/// One test, not four: the bake shelf is the process's, and tests run side
/// by side.
#[test]
fn a_dropped_cube_falls_lands_and_scrubs_from_the_bake() {
    let model = drop_scene();
    let mut s = BState::new();
    // Blender 5.2's own solver, frame by frame (`make_drop.py`): 3.0 at
    // the start, 2.3061 at frame 10, landed by 20 and at rest at 0.54 —
    // half the scaled cube over the plane, plus Bullet's 0.04 margin on
    // the plane's mesh shape.
    assert!((cube_z(&model, &mut s, 1) - 3.0).abs() < 1e-4, "the start frame is the file's pose");
    let ten = cube_z(&model, &mut s, 10);
    assert!((ten - 2.3061).abs() < 0.01, "falling at frame 10: {ten}");
    let baked = run();
    let rest = cube_z(&model, &mut s, 60);
    assert!((rest - 0.54).abs() < 0.045, "at rest on the plane, its scale taken once: {rest}");
    assert!((cube_z(&model, &mut s, 30) - rest).abs() < 0.01, "landed and still by frame 30");
    assert!((cube_z(&model, &mut s, 90) - rest).abs() < 1e-6, "past the cache the last frame holds");
    assert!((cube_z(&model, &mut s, -5) - 3.0).abs() < 1e-4, "before it the start pose");

    // Back to frame 10: the same pose, read off the shelf, not baked again.
    assert_eq!(cube_z(&model, &mut s, 10), ten, "a scrub reads the bake");
    assert_eq!(run(), baked, "a scrub does not bake again");

    // A settings edit bakes again: twice the speed falls further by 10.
    let speed = crate::part::props::set::prop_named("rigidbody_world.time_scale").unwrap();
    s.set(crate::state::ButsKey::Value(speed.key), crate::state::val::BValue::Float(2.0).encode());
    let fast = cube_z(&model, &mut s, 10);
    assert_ne!(run(), baked, "the edit filed a new bake");
    assert!(fast < ten - 0.5, "Speed 2 falls further by frame 10: {fast} against {ten}");

    // Bake pins it through an edit; Delete Bake lets the edit in.
    crate::state::rule::after_set(&mut s, &OpKey::RigidBake.path(), "bake");
    let pinned = run();
    s.set(crate::state::ButsKey::Value(speed.key), crate::state::val::BValue::Float(1.0).encode());
    assert_eq!(cube_z(&model, &mut s, 10), fast, "a baked cache holds through the edit");
    assert_eq!(run(), pinned);
    crate::state::rule::after_set(&mut s, &OpKey::RigidBake.path(), "free");
    assert!((cube_z(&model, &mut s, 10) - ten).abs() < 1e-6, "Delete Bake: the edit bakes again");
}
