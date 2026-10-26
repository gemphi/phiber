//! Settings modal — Preferences dialog opened via `ctx.modal = "settings"`.

pub mod set;

use crate::host::BHostItem;
use crate::host::BHostType;
use crate::state::BState;
use crate::view::BView;
use crate::view::BViewItem;
pub use set::BSettingsCategory;
pub use set::BSettingsCategorySpec;
pub use set::CATEGORIES;
pub use set::NAV_TABS;
pub use set::category_for;

#[derive(Clone, Copy, Debug, Default)]
pub struct BSettingsModal;

impl BViewItem for BSettingsModal {
    /// The Preferences editor itself, elevated — one implementation for the
    /// workspace host and the dialog (Blender opens `SPACE_USERPREF` in a
    /// new window the same way).
    fn build(&self, s: &BState) -> BView {
        BHostItem::new(BHostType::Prefs).build(s)
    }
}
