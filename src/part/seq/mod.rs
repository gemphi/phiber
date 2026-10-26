//! Sequencer canvas. Blender ref: `space_sequencer/space_sequencer.cc`
//! WINDOW (timeline) + PREVIEW regions.

use crate::host::BHostType;
use crate::state::BState;
use crate::view::BView;
use crate::view::BViewItem;
use crate::view::BViewList;

// `BSeqItem` — sequencer timeline canvas (Blender WINDOW + PREVIEW regions).
#[derive(Clone, Copy, Debug, Default)]
pub struct BSeqItem;

impl BSeqItem {
    pub fn new() -> Self {
        Self
    }
}

impl BViewItem for BSeqItem {
    fn build(&self, _s: &BState) -> BView {
        BView::canvas(BHostType::Seq)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BSeqList;

impl BSeqList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BSeqList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BSeqItem::new().build(s)]
    }
}
