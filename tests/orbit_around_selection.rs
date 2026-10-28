//! With something selected, the view turns about it — Preferences ▸
//! Navigation ▸ Orbit Around Selection (`use_rotate_around_active`), on
//! here — so the selection keeps its place on screen while the scene swings
//! round it. The middle button navigates as Blender's does: a drag orbits,
//! with Shift it pans, with Ctrl it zooms; and a pan slides the orbit centre,
//! so the view keeps orbiting about the middle of what it shows.

use glam::Vec3;
use gpui::{Modifiers, MouseButton, Pixels, Point, TestAppContext, VisualTestContext, px, size};
use phiber::app::BPhiberApp;
use phiber::scene::orbit_around::centre_of;
use phiber::scene::pick::BPickView;
use phiber::scene::select::BSelection;
use phiber::scene::view_slot::BViewSlot;
use phiber::state::{AppKey, ModalKey};

type App = gpui::Entity<BPhiberApp>;

/// The Layout viewport with the Sphere selected and the pointer over it:
/// the app, the canvas's middle, and the Sphere's centre.
fn layout(cx: &mut TestAppContext, orbit_around: bool) -> (App, &mut VisualTestContext, Point<Pixels>, Vec3) {
    BViewSlot::forget_hover();
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        BSelection { list: vec!["Sphere".into()] }.write(&mut app.state);
        app.state.set(AppKey::OrbitAroundSelection, if orbit_around { "true" } else { "false" });
        cx.notify();
    });
    cx.run_until_parked();
    let middle = cx.debug_bounds("view3d-orbit").expect("the Layout viewport's canvas").center();
    cx.simulate_mouse_move(middle, None, Modifiers::none());
    cx.run_until_parked();
    let sphere = app.read_with(cx, |a, _| {
        phiber::scene::source::scene_objects(&a.state).iter().find(|o| o.name() == "Sphere").map(centre_of)
    });
    (app, cx, middle, sphere.expect("the Sphere is in the scene"))
}

/// A middle-button drag from `from` by `by` pixels, with `modifiers` held.
fn middle_drag(cx: &mut VisualTestContext, from: Point<Pixels>, by: (f32, f32), modifiers: Modifiers) {
    let to = Point::new(from.x + px(by.0), from.y + px(by.1));
    cx.simulate_mouse_down(from, MouseButton::Middle, modifiers);
    cx.simulate_mouse_move(Point::new(from.x + px(by.0 / 2.0), from.y + px(by.1 / 2.0)), Some(MouseButton::Middle), modifiers);
    cx.simulate_mouse_move(to, Some(MouseButton::Middle), modifiers);
    cx.simulate_mouse_up(to, MouseButton::Middle, modifiers);
    cx.run_until_parked();
}

/// Where `p` is on the hovered viewport's canvas (canvas widths across from
/// its middle), and the view's yaw, zoom and pivot.
fn seen(app: &App, cx: &mut VisualTestContext, p: Vec3) -> ((f32, f32), f32, f32, [f32; 3]) {
    app.read_with(cx, |a, _| {
        let s = &a.state;
        let at = BPickView::now(s).project(p).expect("in front of the eye");
        (at, phiber::scene::view_yaw(s), phiber::scene::view_zoom(s), phiber::scene::camera::view_pivot(s))
    })
}

#[gpui::test]
async fn a_middle_drag_orbits_about_the_selection_and_keeps_it_in_place(cx: &mut TestAppContext) {
    let (app, cx, middle, sphere) = layout(cx, true);
    let (before, yaw, _, pivot) = seen(&app, cx, sphere);
    middle_drag(cx, middle, (160.0, 60.0), Modifiers::none());
    let (after, turned, _, moved) = seen(&app, cx, sphere);
    assert!((turned - yaw).abs() > 10.0, "the view turned: {yaw} → {turned}");
    assert!((after.0 - before.0).abs() < 1e-3 && (after.1 - before.1).abs() < 1e-3, "the sphere stays put: {before:?} → {after:?}");
    assert_ne!(moved, pivot, "the view centre swung round the sphere");
    BViewSlot::forget_hover();
}

#[gpui::test]
async fn off_a_middle_drag_orbits_about_the_views_centre(cx: &mut TestAppContext) {
    let (app, cx, middle, sphere) = layout(cx, false);
    let (before, yaw, _, pivot) = seen(&app, cx, sphere);
    middle_drag(cx, middle, (160.0, 0.0), Modifiers::none());
    let (after, turned, _, kept) = seen(&app, cx, sphere);
    assert!((turned - yaw).abs() > 10.0);
    assert_eq!(kept, pivot, "the view centre stays");
    assert!((after.0 - before.0).abs() > 1e-2, "so the sphere swings across the view: {before:?} → {after:?}");
    BViewSlot::forget_hover();
}

#[gpui::test]
async fn shift_middle_drag_slides_the_orbit_centre_and_ctrl_middle_drag_zooms(cx: &mut TestAppContext) {
    let (app, cx, middle, _) = layout(cx, true);
    let (_, _, zoom, pivot) = seen(&app, cx, Vec3::ZERO);
    let width = f32::from(cx.debug_bounds("view3d-orbit").unwrap().size.width);
    middle_drag(cx, middle, (120.0, -40.0), Modifiers::shift());
    let (old_centre, _, _, slid) = seen(&app, cx, Vec3::from(pivot));
    assert_ne!(slid, pivot, "the pan slid the orbit centre");
    assert!((old_centre.0 - 120.0 / width).abs() < 2e-3, "what was the middle follows the pointer: {old_centre:?}");
    let (new_centre, ..) = seen(&app, cx, Vec3::from(slid));
    assert!(new_centre.0.abs() < 1e-4 && new_centre.1.abs() < 1e-4, "the orbit centre is the middle of the view: {new_centre:?}");
    assert_eq!(app.read_with(cx, |a, _| phiber::scene::view_pan(&a.state)), (0.0, 0.0), "no screen-space offset");
    middle_drag(cx, middle, (0.0, -60.0), Modifiers::control());
    let (_, _, zoomed, _) = seen(&app, cx, Vec3::from(slid));
    assert!(zoomed > zoom, "Ctrl, the pointer up, zooms in: {zoom} → {zoomed}");
    BViewSlot::forget_hover();
}
