//! Topbar zone-visibility controls for zones declared by the active space.

use crate::host::BHostType;
use crate::space::BSpaceList;
use crate::space::BSpaceType;
use crate::state::BState;
use crate::view::BView;
use crate::view::BViewItem;
use crate::view::BZoneToggle;
use crate::zone::BZoneType;
use crate::zone::visibility;

#[derive(Clone, Copy, Debug)]
pub struct BZonebar {
    pub host: BHostType,
}

impl BZonebar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }

    pub fn declared_zones(space: BSpaceType) -> Vec<BZoneType> {
        visibility::TOGGLEABLE
            .into_iter()
            .filter(|zone| !space.spec().areas_in(*zone).is_empty())
            .collect()
    }
}

impl BViewItem for BZonebar {
    fn build(&self, state: &BState) -> BView {
        if self.host != BHostType::Topbar {
            return BView::empty();
        }
        let controls = Self::declared_zones(BSpaceList::active(state))
            .into_iter()
            .map(|zone| BZoneToggle {
                zone,
                // `BZoneType::tooltip` existed all along and no toggle ever
                // passed it on, so the one topbar control with nothing to say
                // on hover was the one whose glyph is a drawing.
                control: crate::view::prim::BControl::new()
                    .with_tooltip(zone.tooltip())
                    .with_active(visibility::is_visible(state, zone))
                    .with_action(visibility::toggle_action(zone).expect("declared toggleable zone")),
            })
            .collect();
        BView::zone_toggle_group(controls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::CtxKey;
    use crate::view::BView;

    fn toggles(state: &BState) -> Vec<BZoneToggle> {
        match BZonebar::new(BHostType::Topbar).build(state) {
            BView::ZoneToggleGroup { controls } => controls,
            other => panic!("the zonebar builds a toggle group, not {other:?}"),
        }
    }

    /// The toggle's on/off is its control's `active`, not a bool beside it —
    /// so it answers in the vocabulary every other control answers in, and
    /// the draw layer reads the state rather than deciding it.
    #[test]
    fn a_zone_toggle_carries_its_state_on_its_control() {
        let mut state = BState::new();
        let shown = toggles(&state);
        assert!(!shown.is_empty(), "the Layout space declares toggleable zones");
        for toggle in &shown {
            assert_eq!(
                toggle.control.active,
                crate::zone::visibility::is_visible(&state, toggle.zone),
                "{:?}: the control says what the zone does",
                toggle.zone,
            );
            assert!(toggle.control.action.is_some(), "{:?} fires something", toggle.zone);
            // The tooltip the zone always had and the toggle never passed on.
            assert_eq!(toggle.control.tooltip.as_deref(), Some(toggle.zone.tooltip()), "{:?}", toggle.zone);
        }
        // Hiding a zone flips its control, and only its own.
        let first = shown[0].zone;
        let before: Vec<bool> = shown.iter().map(|t| t.control.active).collect();
        state.set(&crate::zone::visibility::key(first).expect("a toggleable zone has a key"), crate::state::val::V_FALSE);
        let after = toggles(&state);
        assert!(!after[0].control.active, "{first:?} is off now");
        for (i, t) in after.iter().enumerate().skip(1) {
            assert_eq!(t.control.active, before[i], "{:?} is untouched", t.zone);
        }
    }

    /// Only the Topbar draws them.
    #[test]
    fn no_other_host_builds_zone_toggles() {
        let state = BState::new();
        for host in [BHostType::View3D, BHostType::Outliner, BHostType::Buts] {
            assert!(matches!(BZonebar::new(host).build(&state), BView::Empty), "{host:?}");
        }
        let _ = CtxKey::Space;
    }
}
