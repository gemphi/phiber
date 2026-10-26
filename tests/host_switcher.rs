//! Switching an area's editor type from its header, driven through the app.

use phiber::state::{CtxKey};
use gpui::{TestAppContext, px, size};
use phiber::app::BPhiberApp;
use phiber::host::BHostType;
use phiber::space::{BSpaceType, BSpaceList};
use phiber::state::{BState};
use phiber::view::{BView, BViewList};

/// Every popover that is open in the current tree, as (open, body-size).
fn open_popovers(v: &BView, out: &mut Vec<usize>) {
    if let BView::Popover { open, body, .. } = v {
        if *open {
            out.push(body.len());
        }
    }
    for child in children(v) {
        open_popovers(child, out);
    }
}

fn children(v: &BView) -> Vec<&BView> {
    match v {
        BView::Space { list, .. }
        | BView::Zone { list, .. }
        | BView::Host { list, .. }
        | BView::Region { list, .. }
        | BView::Pane { list, .. }
        | BView::Row { list }
        | BView::Column { list } => list.iter().collect(),
        BView::Card(card) if matches!(card.kind, phiber::view::prim::BCardKind::Column(_)) => card.body.iter().collect(),
        BView::Area { child, .. } | BView::Part { body: child, .. } => vec![child.as_ref()],
        BView::Popover { anchor, body, .. } => {
            let mut out = vec![anchor.as_ref()];
            out.extend(body.iter());
            out
        }
        _ => Vec::new(),
    }
}

/// Which host the Layout Center area is showing.
fn center_host(s: &BState) -> Option<BHostType> {
    fn find(v: &BView, out: &mut Option<BHostType>) {
        if let BView::Host { kind, .. } = v {
            if kind.fills_content() || *kind == BHostType::Buts {
                out.get_or_insert(*kind);
            }
        }
        for child in children(v) {
            find(child, out);
        }
    }
    let mut out = None;
    for view in BSpaceList::new().items(s) {
        find(&view, &mut out);
    }
    out
}

#[gpui::test]
async fn choosing_an_editor_type_switches_the_area_and_closes_the_popover(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let before = center_host(&app.state);
        assert_eq!(
            before,
            Some(BHostType::View3D),
            "Layout's Center area starts on the 3D viewport"
        );
    });

    // Open the switcher, then choose a different editor type the way the
    // popover's own menu item does.
    app.update(cx, |app, cx| {
        let space = BSpaceType::Layout;
        let area = phiber::space::BSpaceType::Layout
            .spec()
            .areas_in(phiber::zone::BZoneType::Center)[0];
        let open = phiber::area::current::switcher_key(space, &area);
        app.state.set(&open, phiber::state::val::V_TRUE);
        let slot = phiber::area::current::area_key(space, &area);
        app.handle_action(
            &phiber::view::prim::BAction::Set(
                slot,
                BHostType::Image.slug().to_string(),
            ),
            cx,
        );
        cx.notify();
    });
    cx.run_until_parked();

    app.update(cx, |app, _| {
        assert_eq!(
            center_host(&app.state),
            Some(BHostType::Image),
            "choosing an editor type must switch the area"
        );
        let mut open = Vec::new();
        for view in BSpaceList::new().items(&app.state) {
            open_popovers(&view, &mut open);
        }
        assert!(
            open.is_empty(),
            "choosing an editor type must close the switcher: {open:?} still open"
        );
    });
}
