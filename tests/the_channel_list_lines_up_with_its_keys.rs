//! The Dope Sheet's channel list is a side column like every other
//! (`part::channels::time`), and the canvas beside it paints only the keys.
//! The two must share one line per channel: the list's row `n` and the
//! canvas's row `n` (`BTimeRow::center`) sit at the same height, or a name
//! stands beside another channel's keys.

use gpui::{TestAppContext, px, size};
use phiber::app::BPhiberApp;
use phiber::space::BSpaceType;
use phiber::state::{CtxKey, ModalKey};
use phiber::theme::METRICS;
use phiber::theme::style::metric::MODULE;
use phiber::view::draw::canvas::time::row::BTimeRow;

fn mid(b: gpui::Bounds<gpui::Pixels>) -> f32 {
    f32::from(b.origin.y) + f32::from(b.size.height) / 2.0
}

#[gpui::test]
async fn the_dope_sheets_summary_row_is_on_its_keys_line(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Space, BSpaceType::Animation.slug());
        cx.notify();
    });
    cx.run_until_parked();

    let canvas = cx.debug_bounds("time-window").expect("the Dope Sheet's keys");
    let list = cx.debug_bounds("panel-sidebar-column").expect("the channel list, a side column");
    let summary = cx.debug_bounds("card-row-channels-0").expect("the Summary channel, the list's first row");
    assert!(list.origin.x + list.size.width <= canvas.origin.x, "the list stands beside the keys, not over them");
    assert_eq!(f32::from(summary.size.height), MODULE, "a channel is a side column's row");
    let keys = f32::from(canvas.origin.y) + BTimeRow::center(0, MODULE, METRICS.h_header);
    assert!((mid(summary) - keys).abs() < 0.51, "the Summary's name is on its keys' line: {} vs {keys}", mid(summary));
}
