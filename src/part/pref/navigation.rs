//! Preferences ▸ Navigation ▸ Orbit & Pan (`USERPREF_PT_navigation_orbit`):
//! Orbit Around Selection (`scene::orbit_around`). Blender's panel also
//! holds the Orbit Method and its sensitivity, and Auto Perspective and
//! Depth, which Phiber has no switch for yet — its orbit is the turntable.

use crate::host::BHostType;
use crate::part::props::BPanelItem;
use crate::scene::orbit_around::BOrbitAround;
use crate::state::val::V_FALSE;
use crate::state::BState;
use crate::view::BView;
use crate::view::prim::BCard;

/// The Orbit & Pan panel.
pub struct BOrbitPan;

impl BOrbitPan {
    pub const TITLE: &'static str = "Orbit & Pan";

    pub fn view(state: &BState) -> BView {
        let key = BPanelItem::expand_key(BHostType::Prefs, "Navigation", Self::TITLE);
        let open = state.get(&key) != Some(V_FALSE);
        if !open {
            return BCard::panel(Self::TITLE).fold(key, false).into();
        }
        BCard::panel(Self::TITLE).fold(key, true).body(vec![BView::ctrl(BOrbitAround::control(state))]).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppKey;
    use crate::state::val::BValue;

    #[test]
    fn the_card_holds_orbit_around_selection_on_until_turned_off() {
        let mut s = BState::new();
        let BView::Card(card) = BOrbitPan::view(&s) else { panic!() };
        let BView::Ctrl(ctrl) = &card.body[0] else { panic!("{:?}", card.body) };
        assert_eq!(ctrl.key, BOrbitAround::key());
        assert_eq!(ctrl.label.as_deref(), Some("Orbit Around Selection"));
        assert!(matches!(ctrl.value, BValue::Bool(true)), "on until set");
        s.set(AppKey::OrbitAroundSelection, V_FALSE);
        let BView::Card(card) = BOrbitPan::view(&s) else { panic!() };
        let BView::Ctrl(ctrl) = &card.body[0] else { panic!() };
        assert!(matches!(ctrl.value, BValue::Bool(false)));
    }
}
