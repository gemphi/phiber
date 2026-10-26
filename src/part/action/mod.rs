//! Animation editor canvas — shared by Action/Nla/Ipo/Drivers hosts.
//! Blender ref: `space_action`, `space_nla`, `space_graph`.

pub mod template;

use crate::host::BHostType;
use crate::state::BState;
use crate::view::BView;
use crate::view::BViewItem;
use crate::view::BViewList;

// `BActionItem` — animation editor canvas. Shared by Action/Nla/Ipo/Drivers.
#[derive(Clone, Copy, Debug)]
pub struct BActionItem {
    pub host: BHostType,
}

impl BActionItem {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BActionItem {
    fn build(&self, _s: &BState) -> BView {
        BView::canvas(self.host)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BActionList {
    pub host: BHostType,
}

impl BActionList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BActionList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BActionItem::new(self.host).build(s)]
    }
}
