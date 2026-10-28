//! The palette's results and where a pick goes.

use super::*;
use crate::state::rule::BRule as _;

#[test]
fn every_kind_finds_something_and_all_finds_them_all() {
    let all = hits("", BFindKind::All, None).len();
    let mut sum = 0;
    for kind in [BFindKind::Space, BFindKind::Editor, BFindKind::Panel, BFindKind::Menu] {
        let n = hits("", kind, None).len();
        assert!(n > 0, "{kind:?} finds nothing");
        sum += n;
    }
    assert_eq!(all, sum);
}

#[test]
fn a_panel_pick_opens_its_space_its_tab_and_the_panel() {
    let hit = hits("properties render sampling", BFindKind::Panel, None).into_iter().next().expect("Render ▸ Sampling");
    let BAction::Set(path, value) = &hit.action else { panic!("a set") };
    let mut s = BState::new();
    s.set(path, value.clone());
    BFindGoRule.after(path, value, &mut s);
    assert_eq!(s.get(crate::state::ButsKey::NavbarTab), Some("Render"));
    assert_eq!(s.get(&BPanelItem::expand_key(BHostType::Buts, "Render", "Sampling")), Some(V_TRUE));
    assert!(s.get(CtxKey::Space).is_some());
}

#[test]
fn a_sidebar_pick_opens_the_sidebar_on_its_tab() {
    let hit = hits("3d viewport sidebar item", BFindKind::Panel, None).into_iter().next().expect("Sidebar ▸ Item");
    let BAction::Set(path, value) = &hit.action else { panic!("a set") };
    let mut s = BState::new();
    BFindGoRule.after(path, value, &mut s);
    assert_eq!(s.get(&BPath::owned(vec![BHostType::View3D.slug(), "show", "sidebar"])), Some(V_TRUE));
    assert_eq!(s.get(&crate::part::sidebar::show::BSidebarShown(BHostType::View3D).tab_key()), Some("Item"));
}

#[test]
fn a_space_pick_switches_the_space_and_closes_the_palette() {
    // The spaces are found here, not in a row of tabs: picking one has to
    // land in it with the palette gone and its caret out of the query.
    use crate::state::action::apply_action;
    let hit = hits("shading", BFindKind::Space, None).into_iter().next().expect("the Shading space");
    let mut s = BState::new();
    apply_action(&mut s, &BAction::Set(ModalKey::Open.path(), V_MODAL_COMMAND.to_string()));
    assert!(crate::state::edit::is_editing(&s, AppKey::CommandQuery), "the palette takes the caret");
    apply_action(&mut s, &hit.action);
    assert_eq!(s.get(CtxKey::Space), Some(BSpaceType::Shading.slug()));
    assert_eq!(s.get(ModalKey::Open), Some(V_EMPTY), "the palette closes on the pick");
    assert!(!crate::state::edit::is_editing(&s, AppKey::CommandQuery), "and its caret goes with it");
}
