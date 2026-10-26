//! "Revert to the Saved File" — `wm.revert_mainfile` asks before it reads
//! the file again and drops the edits. Laid out like `file_close`: the
//! warning, then the answers. Content is `set.rs`.

pub mod set;

use crate::state::BState;
use crate::view::BView;
use crate::view::BViewItem;
pub use set::{CONFIRMED, TITLE};

#[derive(Clone, Copy, Debug, Default)]
pub struct BRevertModal;

impl BViewItem for BRevertModal {
    fn build(&self, _: &BState) -> BView {
        BView::column(vec![
            BView::row(vec![BView::icon("alert_triangle"), BView::label(set::MESSAGE)]),
            BView::row(std::iter::once(BView::spacer()).chain(set::buttons()).collect()),
        ])
    }
}
