//! Spaces modal — every workspace as a tile; picking one switches `space`
//! and closes the dialog. Driven by `BSpaceType::ALL` (the same set as the
//! Topbar tabs), so the two can never disagree.

pub mod pick;

use crate::state::BKey as _;
use crate::state::CtxKey;
use crate::space::BSpaceType;
use crate::state::BState;
use crate::view::prim::BAction;
use crate::view::prim::BControl;
use crate::view::BView;
use crate::view::BViewItem;

#[derive(Clone, Copy, Debug, Default)]
pub struct BSpacesModal;

impl BViewItem for BSpacesModal {
    fn build(&self, s: &BState) -> BView {
        let active = s.get(CtxKey::Space).and_then(BSpaceType::from_slug).unwrap_or(BSpaceType::Layout);
        let tiles = BSpaceType::ALL
            .iter()
            .filter(|sp| sp.is_tab())
            .map(|&sp| {
                BView::Button(
                    BControl::new()
                        .with_icon(sp.icon())
                        .with_label(sp.label())
                        .with_active(sp == active)
                        // The reducer closes this picker once a space is chosen.
                        .with_action(BAction::Set(CtxKey::Space.path(), sp.slug().to_string())),
                )
            })
            .collect();
        BView::grid(tiles)
    }
}
