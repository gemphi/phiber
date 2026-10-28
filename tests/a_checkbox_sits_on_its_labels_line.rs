//! A checkbox in a property row sits on the row's centre line, level with
//! the label opposite it, as any icon in a row does. Inspector checkboxes
//! sat up and away from their words ("up in the air").

use gpui::{TestAppContext, px, size};
use phiber::app::BPhiberApp;
use phiber::state::{BKey as _, CtxKey, ModalKey, View3dKey};
use phiber::view::BAction;

#[gpui::test]
async fn a_property_checkbox_is_level_with_its_label(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.handle_action(&BAction::Set(CtxKey::Object.path(), "Cube".into()), cx);
    });
    cx.run_until_parked();
    let view = cx.debug_bounds("tab-view_layer-view_layer").expect("the View tab").center();
    cx.simulate_click(view, gpui::Modifiers::none());
    cx.run_until_parked();

    let id: &'static str = Box::leak(format!("check-{}", View3dKey::BorderUse.path().encode()).into_boxed_str());
    let check = cx.debug_bounds(id).expect("View ▸ Render Region's checkbox");
    let row = cx.debug_bounds("section-Render Region").expect("its property row");
    let label = cx.debug_bounds("section-Render Region-label").expect("its label column");
    let mid = |b: gpui::Bounds<gpui::Pixels>| f32::from(b.origin.y) + f32::from(b.size.height) / 2.0;
    assert!((mid(check) - mid(row)).abs() <= 0.5, "the box on the row's centre line: box {check:?}, row {row:?}");
    assert!((mid(check) - mid(label)).abs() <= 0.5, "level with its label: box {check:?}, label {label:?}");
}
