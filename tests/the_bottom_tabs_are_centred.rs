//! The strip over the bottom area — Timeline, Play, Project Browser, the
//! Dope Sheet and the other animation editors — is centred on the area, as
//! Reality Composer Pro's strip over its bottom panel is. Built as
//! `[spacer, tabs…, spacer]` it sat at the left edge: a row drops its empty
//! groups (`draw::row`), and a strip spreads what is left from its start.

use gpui::{Bounds, Pixels, TestAppContext, px, size};
use phiber::app::BPhiberApp;
use phiber::host::BHostType;
use phiber::space::BSpaceType;
use phiber::state::{CtxKey, ModalKey};

/// The tabs' left and right ends, and the strip's, with `host` showing.
fn spans(cx: &mut TestAppContext, width: f32, host: BHostType) -> ((Pixels, Pixels), Bounds<Pixels>) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(width), px(900.0)));
    let space = BSpaceType::Layout;
    let area = space.spec().areas.iter().find(|a| a.default_host == BHostType::Time).copied().expect("the Timeline area");
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Space, space.slug());
        app.state.set(&phiber::area::area_key(space, &area), host.slug());
        cx.notify();
    });
    cx.run_until_parked();
    let strip = cx.debug_bounds(Box::leak(format!("region-Tabs-{host:?}").into_boxed_str())).expect("the bottom area's tab strip");
    let tabs: Vec<Bounds<Pixels>> = area
        .hosts
        .iter()
        .map(|h| {
            let id: &'static str = Box::leak(format!("tab-{}-{}", h.icon(), h.label()).into_boxed_str());
            cx.debug_bounds(id).unwrap_or_else(|| panic!("a tab for {h:?}"))
        })
        .collect();
    let left = tabs.iter().map(|b| b.origin.x).fold(px(f32::MAX), |a, b| a.min(b));
    let right = tabs.iter().map(|b| b.origin.x + b.size.width).fold(px(f32::MIN), |a, b| a.max(b));
    ((left, right), strip)
}

fn assert_centred(((left, right), strip): ((Pixels, Pixels), Bounds<Pixels>), what: &str) {
    let tabs_mid = (left + right) / 2.0;
    let strip_mid = strip.origin.x + strip.size.width / 2.0;
    assert!(right - left < strip.size.width, "{what}: the tabs fit the strip: {left:?}..{right:?} in {strip:?}");
    assert!((tabs_mid - strip_mid).abs() < px(2.0), "{what}: the tabs are centred on the strip — tabs {left:?}..{right:?}, strip {strip:?}");
    assert!(left - strip.origin.x > px(40.0), "{what}: not at the left edge — tabs from {left:?}, strip from {:?}", strip.origin.x);
}

#[gpui::test]
async fn the_bottom_tabs_are_centred_over_the_timeline(cx: &mut TestAppContext) {
    assert_centred(spans(cx, 1600.0, BHostType::Time), "Timeline at 1600");
}

#[gpui::test]
async fn the_bottom_tabs_stay_centred_whichever_editor_shows(cx: &mut TestAppContext) {
    assert_centred(spans(cx, 1900.0, BHostType::Project), "Project Browser at 1900");
}
