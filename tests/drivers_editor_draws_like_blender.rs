//! End to end: the Drivers editor is the Graph editor in `SIPO_MODE_DRIVERS`
//! — the animation window on `ED_drivers_editor_init`'s `−2 .. 2` view,
//! the shown object's drivers listed under it in the channel region with
//! their colour swatches (`ANIM_channel_draw`), the cursor's value and
//! input lines crossing at the origin (`graph_main_region_draw`), and none
//! of the time editors' furniture: no frame range, no playhead.

use gpui::{TestAppContext, px, size};
use phiber::phiphx::blend::BFCurve;
use phiber::phiphx::scene::{SceneDrivers, SceneModel};
use phiber::app::BPhiberApp;
use phiber::host::BHostType;
use phiber::space::BSpaceType;

/// The channel list is a side column beside the window
/// (`part::channels::time`): the view starts at the window's left edge.
const CHANNEL_WIDTH: f32 = 0.0;
/// The frame-number strip's height: the theme's `header_height`
/// (`the_scrub_strip_is_as_tall_as_a_header`).
const SCRUB: f32 = 36.0;
/// `GPU_line_width(2.0)` for the cursor.
const CURSOR_WIDTH: f32 = 2.0;

/// The quads of the last frame as `(x, y, w, h, alpha, has_border)` in logical pixels.
fn quads(cx: &mut gpui::VisualTestContext) -> Vec<(f32, f32, f32, f32, f32, bool)> {
    let scale = cx.update(|window, _| window.scale_factor());
    cx.scene_fingerprint()
        .into_iter()
        .filter(|line| line.starts_with("quad "))
        .map(|line| {
            let mut parts = line.split(' ');
            let origin = parts.nth(1).unwrap();
            let size = parts.next().unwrap();
            let (x, y) = origin.split_once(',').unwrap();
            let (w, h) = size.split_once('x').unwrap();
            let alpha = line.split("a: ").nth(1).and_then(|rest| rest.split(' ').next()).and_then(|a| a.trim_end_matches('}').trim().parse::<f32>().ok()).unwrap_or(-1.0);
            let border = !line.contains(" w0.00/0.00/0.00/0.00 ");
            (x.parse::<f32>().unwrap() / scale, y.parse::<f32>().unwrap() / scale, w.parse::<f32>().unwrap() / scale, h.parse::<f32>().unwrap() / scale, alpha, border)
        })
        .collect()
}

#[gpui::test]
async fn the_drivers_editor_lists_the_drivers_and_crosses_its_cursor_at_the_origin(cx: &mut TestAppContext) {
    let mut inverted = BFCurve::new("pose.bones[\"MCH-forearm_ik.L\"].constraints[\"IK.001\"].mute", 0, Vec::new());
    inverted.generator = Some(vec![1.0, -1.0]);
    let plain = BFCurve::new("pose.bones[\"MCH-thumb.02_drv.L\"].rotation_euler", 0, Vec::new());
    let mut model = SceneModel { drivers: vec![SceneDrivers { object: "rig".into(), curves: vec![inverted, plain] }], ..Default::default() };
    model.selection.selected = vec!["rig".into()];

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    // The Animation workspace's Dope Sheet band offers the Drivers editor.
    let space = BSpaceType::Animation;
    let area = space.spec().areas.iter().find(|a| a.default_host == BHostType::Action).copied().expect("the Dope Sheet area");
    assert!(area.hosts.contains(&BHostType::Drivers));
    app.update(cx, |app, cx| {
        app.scene_cache = Some(("drivers-test".into(), std::sync::Arc::new(model)));
        app.state.remove(phiber::state::ModalKey::Open);
        app.state.set(phiber::state::CtxKey::Space, space.slug());
        app.state.set(&phiber::area::area_key(space, &area), BHostType::Drivers.slug());
        cx.notify();
    });
    cx.run_until_parked();

    let window = cx.debug_bounds("time-window").expect("the Drivers editor window is painted");
    assert!(cx.debug_bounds("region-Header-Drivers").is_some());
    let (canvas_left, top) = (f32::from(window.origin.x), f32::from(window.origin.y));
    let left = canvas_left + CHANNEL_WIDTH;
    let (width, height) = (f32::from(window.size.width) - CHANNEL_WIDTH, f32::from(window.size.height));
    let painted = quads(cx);
    let near = |a: f32, b: f32| (a - b).abs() <= 1.5;
    // `ED_drivers_editor_init`: −2 .. 2 both ways, so the origin is the
    // window's centre. The value line runs the width of the view …
    let (mid_x, mid_y) = (left + width / 2.0, top + height / 2.0);
    assert!(
        painted.iter().any(|(x, y, w, h, alpha, _)| near(*x, left) && near(*w, width) && near(*h, CURSOR_WIDTH) && (y + h / 2.0 - mid_y).abs() <= 2.0 && (alpha - 0.8).abs() < 0.05),
        "the cursor's value line crosses the view at 0: {painted:?}"
    );
    // … and the input line runs from under the strip to the foot.
    assert!(
        painted.iter().any(|(x, y, w, h, alpha, _)| near(*w, CURSOR_WIDTH) && (x + w / 2.0 - mid_x).abs() <= 2.0 && near(*y, top + SCRUB) && near(*h, height - SCRUB) && (alpha - 0.8).abs() < 0.05),
        "the cursor's input line crosses the view at 0: {painted:?}"
    );
    // No playhead: nothing bordered stands the height of the view.
    assert!(!painted.iter().any(|(_, y, _, h, _, border)| *border && near(*y, top + SCRUB) && near(*h, height - SCRUB)), "the Drivers editor has no playhead");
    // No frame range: no curtain at its alpha.
    let in_view = |x: f32, y: f32| x >= left - 1.0 && x < left + width && y >= top && y < top + height;
    assert!(!painted.iter().any(|(x, y, _, _, alpha, _)| in_view(*x, *y) && (alpha - 0.6).abs() < 0.01), "no `ANIM_draw_framerange` in the Drivers editor");
    // The channel list beside the window: the object, then each driver with
    // its colour swatch (`part::channels::time`), as a side column's rows.
    let list = cx.debug_bounds("panel-sidebar-column").expect("the drivers' channel list");
    assert!(f32::from(list.origin.x + list.size.width) <= canvas_left, "the list stands beside the view");
    for row in [0, 1, 2] {
        assert!(cx.debug_bounds(Box::leak(format!("card-row-drivers-{row}").into_boxed_str())).is_some(), "channel row {row} is listed");
    }
}
