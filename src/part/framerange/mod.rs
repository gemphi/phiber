//! Frame range — the right group of the Timeline header: the current
//! frame, the preview-range toggle, and Start / End (of the preview range
//! while it is on). Blender ref: `bl_ui/space_time.py::playback_controls`,
//! after its second `separator_spacer()`.

pub mod set;

use crate::host::BHostType;
use crate::state::BState;
use crate::view::BView;
use crate::view::BViewItem;
use crate::view::BViewList;

#[derive(Clone, Copy, Debug)]
pub struct BFrameRange {
    pub host: BHostType,
}

impl BFrameRange {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BFrameRange {
    fn build(&self, s: &BState) -> BView {
        BView::row(set::BRangeField::ALL.iter().map(|field| field.view(s)).collect())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BFrameRangeList {
    pub host: BHostType,
}

impl BFrameRangeList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BFrameRangeList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BFrameRange::new(self.host).build(s)]
    }
}
