//! File Browser modal — the Open / Save As / Import / Export overlay.
//! Content is shared with the `file` host: sidebar categories, columns,
//! filters, and nav actions all live in `part::file`.
//!
//! The dialog raises the `File` host (`host::file::REGIONS`) in a card, so
//! its header, path strip, source list, file list and execute bar are the
//! regions the editor has in a workspace — one declaration, one layout.

use crate::host::BHostItem;
use crate::host::BHostType;
use crate::state::BState;
use crate::view::BView;
use crate::view::BViewItem;
pub use crate::part::file::set::BFileCategory;
pub use crate::part::file::set::BFileColumn;
pub use crate::part::file::set::BFileDisplayType;
pub use crate::part::file::set::BFileEntry;
pub use crate::part::file::set::CATEGORIES;
pub use crate::part::file::set::COLUMNS;
pub use crate::part::file::set::DISPLAY_MODES;
pub use crate::part::file::set::FILTERS;
pub use crate::part::file::set::NAV;
pub use crate::part::file::set::SYSTEM;

#[derive(Clone, Copy, Debug, Default)]
pub struct BFileBrowserModal;

impl BViewItem for BFileBrowserModal {
    /// The dialog is the File host raised in a card: the same regions, in
    /// the same order, laid out by the same walkers as the editor in a
    /// workspace. Nothing here is built a second time.
    fn build(&self, s: &BState) -> BView {
        BHostItem::new(BHostType::File).build(s)
    }
}
