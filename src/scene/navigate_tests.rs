use super::*;
use crate::scene::pick::BPickView;

const BASE: BNavBase = BNavBase { yaw: 30.0, pitch: 30.0, pan: (0.0, 0.0), zoom: 1.0, pivot: Vec3::ZERO, around: None };

fn value(sets: &[(BPath, String)], key: &BPath) -> f32 {
    sets.iter().find(|(k, _)| k == key).map(|(_, v)| v.parse().unwrap()).unwrap_or_else(|| panic!("no set for {key:?} in {sets:?}"))
}

fn pivot(sets: &[(BPath, String)]) -> Option<Vec3> {
    let raw = &sets.iter().find(|(k, _)| *k == BViewSlot::key(View3dKey::Pivot))?.1;
    let p: Vec<f32> = raw.split(',').map(|v| v.parse().unwrap()).collect();
    Some(Vec3::new(p[0], p[1], p[2]))
}

/// Where `p` lands on a square canvas in the view `sets` leave from `base`.
fn on_screen(base: BNavBase, sets: &[(BPath, String)], p: Vec3) -> (f32, f32) {
    let read = |key: BPath, or: f32| sets.iter().find(|(k, _)| *k == key).map_or(or, |(_, v)| v.parse().unwrap());
    let centre = pivot(sets).unwrap_or(base.pivot);
    let ms = crate::scene::MESH_SCALE;
    let setup = crate::scene::camera::BViewSetup { perspective: true, pivot: (centre.x * ms, centre.z * ms, centre.y * ms), ..Default::default() };
    let pan = (read(BViewSlot::key(View3dKey::PanX), base.pan.0), read(BViewSlot::key(View3dKey::PanY), base.pan.1));
    BPickView { yaw: read(BViewSlot::key(View3dKey::Yaw), base.yaw), pitch: read(BViewSlot::key(View3dKey::Pitch), base.pitch), pan, zoom: read(BViewSlot::key(View3dKey::Zoom), base.zoom), setup, aspect: 1.0 }
        .project(p)
        .unwrap()
}

#[test]
fn a_swipe_orbits_shift_pans_ctrl_zooms_and_the_wheel_always_zooms() {
    use BScrollNav::*;
    use BScrollSource::*;
    assert_eq!(scroll_nav(Trackpad, false, false), Orbit, "TRACKPADPAN → view3d.rotate");
    assert_eq!(scroll_nav(Trackpad, true, false), Pan, "Shift TRACKPADPAN → view3d.move");
    assert_eq!(scroll_nav(Trackpad, false, true), Zoom, "Ctrl TRACKPADPAN → view3d.zoom");
    assert_eq!(scroll_nav(Trackpad, true, true), Pan, "Shift wins where Blender binds nothing");
    for (shift, ctrl) in [(false, false), (true, false), (false, true), (true, true)] {
        assert_eq!(scroll_nav(Wheel, shift, ctrl), Zoom, "the wheel zooms, modifiers or not");
    }
}

#[test]
fn an_orbit_turns_with_the_drag_s_degrees_per_pixel() {
    // 40 px right: yaw turns 40 × 0.5°, as a pointer drag would; pitch holds.
    let sets = scroll_sets(BScrollNav::Orbit, BASE, (40.0, 0.0), 1000.0);
    assert!((value(&sets, &BViewSlot::key(View3dKey::Yaw)) - (30.0 + 40.0 * BTurntable::DEG_PER_PX)).abs() < 1e-4);
    assert!((value(&sets, &BViewSlot::key(View3dKey::Pitch)) - 30.0).abs() < 1e-4);
    assert_eq!(pivot(&sets), None, "nothing to orbit about: the centre stays");
    // 20 px down tips the pitch.
    let sets = scroll_sets(BScrollNav::Orbit, BASE, (0.0, 20.0), 1000.0);
    assert!((value(&sets, &BViewSlot::key(View3dKey::Pitch)) - (30.0 + 20.0 * BTurntable::DEG_PER_PX)).abs() < 1e-4);
}

#[test]
fn an_orbit_about_the_selection_keeps_it_where_it_is_on_screen() {
    let shoe = Vec3::new(1.2, -0.4, 0.3);
    let base = BNavBase { around: Some(shoe), ..BASE };
    let before = on_screen(base, &[], shoe);
    for travel in [(120.0, 0.0), (0.0, -90.0), (300.0, 140.0)] {
        let sets = drag_sets(BDragNav::Orbit, base, travel, 1000.0);
        let after = on_screen(base, &sets, shoe);
        assert!((after.0 - before.0).abs() < 1e-4 && (after.1 - before.1).abs() < 1e-4, "{travel:?}: {before:?} → {after:?}");
        assert!(pivot(&sets).is_some_and(|p| p != Vec3::ZERO), "the view centre swings round the shoe");
    }
}

#[test]
fn a_pan_slides_the_orbit_centre_one_pixel_for_one() {
    // 100 px across a 1000 px canvas: what is at the centre's depth moves 0.1
    // canvas widths, and the centre stays the middle of the view.
    let sets = scroll_sets(BScrollNav::Pan, BASE, (100.0, 50.0), 1000.0);
    let (before, after) = (on_screen(BASE, &[], Vec3::ZERO), on_screen(BASE, &sets, Vec3::ZERO));
    assert!((after.0 - before.0 - 0.1).abs() < 1e-5 && (after.1 - before.1 - 0.05).abs() < 1e-5, "{before:?} → {after:?}");
    assert_eq!((value(&sets, &BViewSlot::key(View3dKey::PanX)), value(&sets, &BViewSlot::key(View3dKey::PanY))), (0.0, 0.0), "no screen-space offset");
    let centre = pivot(&sets).unwrap();
    let middle = on_screen(BASE, &sets, centre);
    assert!(middle.0.abs() < 1e-5 && middle.1.abs() < 1e-5, "the orbit centre is the middle of the view: {middle:?}");
    // From a view still carrying an old screen-space pan, the pan folds in.
    let panned = BNavBase { pan: (0.2, -0.1), ..BASE };
    let sets = scroll_sets(BScrollNav::Pan, panned, (100.0, 0.0), 1000.0);
    let moved = on_screen(BASE, &sets, Vec3::ZERO);
    assert!((moved.0 - 0.3).abs() < 1e-4 && (moved.1 + 0.1).abs() < 1e-4, "{moved:?}");
}

#[test]
fn a_drag_orbits_shift_drag_pans_ctrl_drag_zooms() {
    use BDragNav::*;
    assert_eq!(drag_nav(false, false), Orbit, "MIDDLEMOUSE → view3d.rotate");
    assert_eq!(drag_nav(true, false), Pan, "Shift MIDDLEMOUSE → view3d.move");
    assert_eq!(drag_nav(false, true), Zoom, "Ctrl MIDDLEMOUSE → view3d.zoom");
    assert_eq!(drag_nav(true, true), Zoom, "Shift Ctrl MIDDLEMOUSE → view3d.dolly, a zoom here");
    // The drag's orbit and pan are the scroll's, pixel for pixel.
    assert_eq!(drag_sets(Orbit, BASE, (40.0, 20.0), 1000.0), scroll_sets(BScrollNav::Orbit, BASE, (40.0, 20.0), 1000.0));
    assert_eq!(drag_sets(Pan, BASE, (100.0, 50.0), 1000.0), scroll_sets(BScrollNav::Pan, BASE, (100.0, 50.0), 1000.0));
    // A zoom drag: the pointer moving UP (dy < 0) zooms IN, as Blender's does.
    let up = drag_sets(Zoom, BASE, (0.0, -PX_PER_LINE), 1000.0);
    assert!((value(&up, &BViewSlot::key(View3dKey::Zoom)) - zoom_step(1.0, ZOOM_PER_LINE)).abs() < 1e-5, "up = in");
    let down = drag_sets(Zoom, BASE, (0.0, PX_PER_LINE), 1000.0);
    assert!((value(&down, &BViewSlot::key(View3dKey::Zoom)) - zoom_step(1.0, 1.0 / ZOOM_PER_LINE)).abs() < 1e-5, "down = out");
    // Sideways travel does not zoom.
    let side = drag_sets(Zoom, BASE, (300.0, 0.0), 1000.0);
    assert!((value(&side, &BViewSlot::key(View3dKey::Zoom)) - 1.0).abs() < 1e-6);
}

#[test]
fn a_zoom_steps_one_factor_per_wheel_notch_and_smoothly_between() {
    // One notch (PX_PER_LINE) is the wheel's factor, either way.
    let in_ = scroll_sets(BScrollNav::Zoom, BASE, (0.0, PX_PER_LINE), 1000.0);
    assert!((value(&in_, &BViewSlot::key(View3dKey::Zoom)) - zoom_step(1.0, ZOOM_PER_LINE)).abs() < 1e-5);
    let out = scroll_sets(BScrollNav::Zoom, BASE, (0.0, -PX_PER_LINE), 1000.0);
    assert!((value(&out, &BViewSlot::key(View3dKey::Zoom)) - zoom_step(1.0, 1.0 / ZOOM_PER_LINE)).abs() < 1e-5);
    // Half a notch is the square root of the factor — a swipe is continuous.
    let half = scroll_sets(BScrollNav::Zoom, BASE, (0.0, PX_PER_LINE / 2.0), 1000.0);
    assert!((value(&half, &BViewSlot::key(View3dKey::Zoom)) - zoom_step(1.0, ZOOM_PER_LINE.sqrt())).abs() < 1e-5);
    // Zoom only ever sets the zoom key.
    assert_eq!(in_.len(), 1);
}

#[test]
fn leaving_the_camera_view_hands_its_view_to_the_gesture() {
    let leave = vec![
        BAction::Set(BViewSlot::key(View3dKey::Yaw), "12.5".into()),
        BAction::Set(BViewSlot::key(View3dKey::Pitch), "-4".into()),
        BAction::Set(BViewSlot::key(View3dKey::Zoom), "2.5".into()),
        BAction::Set(BViewSlot::key(View3dKey::Pivot), "1,2,3".into()),
        BAction::Set(BViewSlot::key(View3dKey::PanX), "0".into()),
        BAction::Set(BViewSlot::key(View3dKey::PanY), "0".into()),
    ];
    let base = BNavBase { pan: (0.4, 0.1), around: Some(Vec3::X), ..BASE }.after(&leave);
    assert_eq!((base.yaw, base.pitch, base.zoom, base.pan), (12.5, -4.0, 2.5, (0.0, 0.0)));
    assert_eq!(base.pivot, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(base.around, Some(Vec3::X), "what it orbits about is the selection's, not the camera's");
    assert_eq!(BASE.after(&[]), BASE, "no camera view: the view as it was");
}
