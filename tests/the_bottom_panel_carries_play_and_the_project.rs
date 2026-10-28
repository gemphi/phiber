//! End to end: the Layout's bottom panel opens with Reality Composer Pro's
//! editor tabs over its editor; switched to Play it shows the picker — the
//! match, the sets, the chosen set's cards — and a card picked is the
//! match changed; switched to the Project Browser it shows its navigator,
//! its toolbar and its inspector.

use gpui::{Modifiers, TestAppContext, px, size};
use phiber::app::BPhiberApp;
use phiber::host::BHostType;
use phiber::space::BSpaceType;
use phiber::state::{CtxKey, PlayKey};

fn selectors(cx: &mut gpui::VisualTestContext) -> Vec<String> {
    cx.all_debug_bounds().into_iter().map(|(selector, _)| selector).collect()
}

fn show(app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext, host: BHostType) {
    let space = BSpaceType::Layout;
    let area = space.spec().areas.iter().find(|a| a.default_host == BHostType::Time).copied().expect("the Timeline area");
    assert!(area.hosts.contains(&host), "{host:?} is offered beside the Timeline");
    app.update(cx, |app, cx| {
        app.state.remove(phiber::state::ModalKey::Open);
        app.state.set(CtxKey::Space, space.slug());
        app.state.set(&phiber::area::area_key(space, &area), host.slug());
        cx.notify();
    });
    cx.run_until_parked();
}

#[gpui::test]
async fn play_is_a_tab_of_the_bottom_panel_and_a_card_picks_the_match(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(900.0)));
    show(&app, cx, BHostType::Time);
    let tabs = cx.debug_bounds("region-Tabs-Time").expect("the bottom panel's editor tabs");
    let header = cx.debug_bounds("region-Header-Time").expect("the Timeline's header");
    assert!(tabs.origin.y < header.origin.y, "the tabs sit over the editor's header");

    show(&app, cx, BHostType::Play);
    assert!(cx.debug_bounds("region-Tabs-Play").is_some(), "the tabs stay over Play");
    let names = selectors(cx);
    for sport in ["Basketball", "Netball", "Volleyball", "Beach volleyball", "Soccer"] {
        assert!(names.iter().any(|n| n == &format!("thumb-{sport}")), "a card for {sport}: {names:?}");
    }
    let soccer = cx.debug_bounds("thumb-Soccer").expect("the Soccer card");
    cx.simulate_click(soccer.center(), Modifiers::none());
    cx.run_until_parked();
    let picked = app.read_with(cx, |a, _| a.state.get(PlayKey::Choice).map(phiplay::pick::decode)).expect("a match picked");
    assert_eq!(picked.sport, phiplay::sport::BSport::Soccer);

    // Another set: the home side's sixteen teams.
    app.update(cx, |app, cx| {
        app.state.set(PlayKey::Row, "home");
        cx.notify();
    });
    cx.run_until_parked();
    let names = selectors(cx);
    assert_eq!(names.iter().filter(|n| n.starts_with("thumb-")).count(), 16, "sixteen teams");
}

#[gpui::test]
async fn the_project_browser_is_a_tab_with_its_three_columns(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(900.0)));
    show(&app, cx, BHostType::Project);
    for region in ["region-Tabs-Project", "region-ToolHeader-Project", "region-Tools-Project", "region-Center-Project", "region-UI-Project"] {
        assert!(cx.debug_bounds(region).is_some(), "{region}: {:?}", selectors(cx));
    }
    // Two rows, not three: the tab names and switches the editor, so there is
    // no header repeating it; the toolbar sits straight under the tabs.
    assert!(cx.debug_bounds("region-Header-Project").is_none(), "no header under the tabs");
    let tabs = cx.debug_bounds("region-Tabs-Project").unwrap();
    let bar = cx.debug_bounds("region-ToolHeader-Project").unwrap();
    assert!(bar.origin.y >= tabs.origin.y + tabs.size.height - px(1.0), "the toolbar is the row under the tabs: {tabs:?} {bar:?}");
    let nav = cx.debug_bounds("region-Tools-Project").unwrap();
    let cards = cx.debug_bounds("region-Center-Project").unwrap();
    let info = cx.debug_bounds("region-UI-Project").unwrap();
    assert!(nav.origin.x < cards.origin.x && cards.origin.x < info.origin.x, "navigator, cards, inspector, left to right: {nav:?} {cards:?} {info:?}");
    // Its three groups stand over those columns: the search centred over the
    // cards and never wider than a panel.
    let search = cx
        .all_debug_bounds()
        .into_iter()
        .find_map(|(name, bounds)| name.contains("ctx.project.search").then_some(bounds))
        .unwrap_or_else(|| {
            let near: Vec<String> = selectors(cx).into_iter().filter(|n| n.contains("search") || n.contains("ToolHeader-Project") || n.contains("pane-") && n.ends_with("-Project")).collect();
            panic!("the Project Browser has its search; near it: {near:?}")
        });
    let mid = |b: gpui::Bounds<gpui::Pixels>| b.origin.x + b.size.width / 2.0;
    assert!((mid(search) - mid(cards)).abs() < px(24.0), "the search is centred over the cards: {search:?} {cards:?}");
    assert!(search.size.width <= px(241.0), "and no wider than a panel: {search:?}");
}
