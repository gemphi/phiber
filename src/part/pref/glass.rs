//! Preferences ▸ Themes ▸ Glass: Reduce Transparency, shown while the glass
//! mode is in force (`theme/style/glass`). Apple's switch of the same name
//! turns its glass frosted; this one makes every surface solid and takes the
//! window off the blurred backdrop, so text reads on any desktop.

use crate::host::BHostType;
use crate::part::props::BPanelItem;
use crate::state::val::{BValue, V_FALSE};
use crate::state::{AppKey, BKey as _, BState};
use crate::theme::BThemeMode;
use crate::view::BView;
use crate::view::prim::{BCard, BCtrl, BCtrlKind};

/// The Glass panel.
pub struct BGlassPanel;

impl BGlassPanel {
    pub const TITLE: &'static str = "Glass";

    /// The panel, or nothing outside the glass mode.
    pub fn view(state: &BState) -> Option<BView> {
        let (_, mode) = crate::theme::theme_in_force(state);
        if mode != BThemeMode::Glass {
            return None;
        }
        let key = BPanelItem::expand_key(BHostType::Prefs, "Appearance", Self::TITLE);
        let open = state.get(&key) != Some(V_FALSE);
        if !open {
            return Some(BCard::panel(Self::TITLE).fold(key, false).into());
        }
        let reduce = BCtrl::read(BCtrlKind::Check, AppKey::ReduceTransparency.path(), BValue::Bool(false), state)
            .with_label("Reduce Transparency")
            .with_tooltip("Solid surfaces instead of glass, for text that reads over any backdrop");
        Some(BCard::panel(Self::TITLE).fold(key, true).body(vec![BView::ctrl(reduce)]).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_shows_in_glass_alone_and_holds_reduce_transparency() {
        let mut s = BState::new();
        assert!(BGlassPanel::view(&s).is_none(), "no glass, no panel");
        s.set(AppKey::ThemeMode, "glass");
        let Some(BView::Card(card)) = BGlassPanel::view(&s) else { panic!() };
        let BView::Ctrl(ctrl) = &card.body[0] else { panic!("{:?}", card.body) };
        assert_eq!(ctrl.key, AppKey::ReduceTransparency.path());
        assert!(matches!(ctrl.value, BValue::Bool(false)), "off until set");
    }
}
