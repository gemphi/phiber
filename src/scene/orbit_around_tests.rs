use super::*;
use crate::scene::camera::{BViewSetup, view_pivot};
use crate::scene::select::BSelection;
use crate::scene::view_op::bounds_of;
use crate::scene::{MESH_SCALE, current_scene};

/// The view `(yaw, pitch)` looking at `pivot`, on a canvas twice as wide
/// as it is tall.
fn view(pivot: Vec3, (yaw, pitch): (f32, f32), zoom: f32, perspective: bool) -> BPickView {
    let ms = MESH_SCALE;
    let setup = BViewSetup { perspective, pivot: (pivot.x * ms, pivot.z * ms, pivot.y * ms), aspect: 2.0, ..Default::default() };
    BPickView { yaw, pitch, pan: (0.0, 0.0), zoom, setup, aspect: 2.0 }
}

fn near(a: (f32, f32), b: (f32, f32)) -> bool {
    (a.0 - b.0).abs() < 1e-4 && (a.1 - b.1).abs() < 1e-4
}

/// The default scene, nothing loaded, `names` selected.
fn selecting(names: &[&str]) -> BState {
    crate::scene::source::clear_loaded_scene();
    crate::scene::source::clear_scene_objects();
    let mut state = BState::new();
    BSelection { list: names.iter().map(|n| n.to_string()).collect() }.write(&mut state);
    state
}

fn box_centre_of(name: &str, state: &BState) -> Vec3 {
    let objects = current_scene(state, None);
    let object = objects.iter().find(|o| o.name() == name).unwrap();
    bounds_of(&[object]).unwrap().0
}

#[test]
fn the_preference_is_on_until_turned_off() {
    let mut s = BState::new();
    assert!(BOrbitAround::on(&s), "on by default — Blender's is off, recorded");
    s.set(AppKey::OrbitAroundSelection, V_FALSE);
    assert!(!BOrbitAround::on(&s));
    s.set(AppKey::OrbitAroundSelection, crate::state::val::V_TRUE);
    assert!(BOrbitAround::on(&s));
    let ctrl = BOrbitAround::control(&BState::new());
    assert!(matches!(ctrl.kind, BCtrlKind::Check));
    assert_eq!(ctrl.label.as_deref(), Some("Orbit Around Selection"));
    assert!(matches!(ctrl.value, BValue::Bool(true)));
    assert_eq!(BOrbitAround::key().encode(), "ctx.pref.orbit_around_selection");
}

#[test]
fn an_orbit_about_a_point_keeps_it_where_it_is_on_screen() {
    let (pivot, about) = (Vec3::ZERO, Vec3::new(1.5, -0.5, 0.75));
    let (from, to) = ((30.0, 30.0), (95.0, 62.0));
    for perspective in [false, true] {
        let before = view(pivot, from, 1.3, perspective).project(about).unwrap();
        let turned = turn_pivot(pivot, about, from, to);
        let after = view(turned, to, 1.3, perspective).project(about).unwrap();
        assert!(near(before, after), "perspective {perspective}: {before:?} → {after:?}");
        assert!((turned - about).length() - (pivot - about).length() < 1e-4, "the centre swings round it");
        assert!((turned - pivot).length() > 0.1, "and has moved");
    }
    // Turning about the view's own centre leaves it where it is.
    assert!((turn_pivot(about, about, from, to) - about).length() < 1e-6);
}

#[test]
fn a_pan_slides_the_centre_and_what_is_at_its_depth_follows_the_pointer() {
    let (pivot, angles, zoom, shift) = (Vec3::new(0.2, 0.4, -0.1), (40.0, 25.0), 1.5, (0.1, -0.05));
    let slid = pan_pivot(pivot, angles, zoom, shift);
    let [right, up, _] = axes(angles);
    let at_depth = pivot + right * 0.7 + up * 0.3;
    for perspective in [false, true] {
        let before = view(pivot, angles, zoom, perspective).project(at_depth).unwrap();
        let after = view(slid, angles, zoom, perspective).project(at_depth).unwrap();
        // x in canvas widths; y in canvas heights, so twice the shift on a 2:1 canvas.
        assert!(near((after.0 - before.0, after.1 - before.1), (shift.0, shift.1 * 2.0)), "{before:?} → {after:?}");
        let centre = view(slid, angles, zoom, perspective).project(slid).unwrap();
        assert!(near(centre, (0.0, 0.0)), "the orbit centre is the middle of the view");
    }
    // Orthographic, the slid centre is the old screen-space pan exactly.
    let far = Vec3::new(-2.0, 3.0, 1.0);
    let panned = BPickView { pan: shift, ..view(pivot, angles, zoom, false) }.project(far).unwrap();
    assert!(near(panned, view(slid, angles, zoom, false).project(far).unwrap()));
}

#[test]
fn the_centre_is_the_mean_of_the_selected_boxes_and_needs_the_preference() {
    let state = selecting(&["Gem"]);
    let gem = box_centre_of("Gem", &state);
    assert!((BOrbitAround::centre(&state).unwrap() - gem).length() < 1e-5);
    let state = selecting(&["Gem", "Sphere"]);
    let both = (gem + box_centre_of("Sphere", &state)) * 0.5;
    assert!((BOrbitAround::centre(&state).unwrap() - both).length() < 1e-5);
    // Hidden, the sphere does not count.
    let mut hidden = selecting(&["Gem", "Sphere"]);
    hidden.set(&crate::part::outliner::key::k_restrict("Sphere", "hide_viewport"), crate::state::val::V_TRUE);
    assert!((BOrbitAround::centre(&hidden).unwrap() - gem).length() < 1e-5);
    let mut off = selecting(&["Gem"]);
    off.set(AppKey::OrbitAroundSelection, V_FALSE);
    assert_eq!(BOrbitAround::centre(&off), None, "off: the view turns about its own centre");
    assert_eq!(BOrbitAround::centre(&selecting(&[])), None, "nothing selected");
}

#[test]
fn a_mesh_s_centre_is_its_own_box_through_its_matrix() {
    use crate::phiphx::scene::SceneNormals;
    let vertices = vec![[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 2.0, 0.0], [0.0, 2.0, 1.0]];
    let mesh = Arc::new(SceneMesh { vertices, indices: vec![0, 1, 2, 0, 2, 3], normals: SceneNormals::PerFace(Vec::new()), corners: Default::default() });
    // A quarter turn about Z, then moved to (10, 0, 5): (2, 1, 0.5) lands at (9, 2, 5.5).
    let m = [[0.0, 1.0, 0.0, 0.0], [-1.0, 0.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [10.0, 0.0, 5.0, 1.0]];
    let local = (*BOXES.of(&mesh, local_box)).unwrap();
    let world = Mat4::from_cols_array_2d(&m).transform_point3((local.0 + local.1) * 0.5);
    assert!((world - Vec3::new(9.0, 2.0, 5.5)).length() < 1e-5, "{world}");
    let empty = Arc::new(SceneMesh { vertices: Vec::new(), indices: Vec::new(), normals: SceneNormals::PerFace(Vec::new()), corners: Default::default() });
    assert_eq!(*BOXES.of(&empty, local_box), None, "no vertices: the origin stands in");
}

#[test]
fn numpad_orbits_turn_about_the_selection_and_keep_it_on_screen() {
    let mut state = selecting(&["Sphere"]);
    let sphere = box_centre_of("Sphere", &state);
    let on_screen = |s: &BState| BPickView::now(s).project(sphere).unwrap();
    let before = on_screen(&state);
    step(&mut state, View3dKey::Yaw, 15.0);
    step(&mut state, View3dKey::Pitch, -15.0);
    assert!(near(on_screen(&state), before), "{before:?} → {:?}", on_screen(&state));
    assert_ne!(view_pivot(&state), [0.0; 3], "the view centre moved round the sphere");
    // Off, Numpad turns about the view's centre and leaves it where it is.
    let mut off = selecting(&["Sphere"]);
    off.set(AppKey::OrbitAroundSelection, V_FALSE);
    step(&mut off, View3dKey::Yaw, 15.0);
    assert_eq!(off.get(View3dKey::Pivot), None);
    assert_eq!(off.get(View3dKey::Yaw).and_then(|v| v.parse::<f32>().ok()), Some(45.0));
}
