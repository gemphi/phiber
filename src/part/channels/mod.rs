//! Channels — vertical list of animation channels.

pub mod set;
pub mod time;

use crate::host::BHostType;
use crate::state::BState;
use crate::view::BView;
use crate::view::BViewItem;
use crate::view::BViewList;
use self::set::channels_for;
pub use set::BChannelSpec;

// `BChannelsItem` — one channel row; carries a `BChannelSpec`.
#[derive(Clone, Copy, Debug)]
pub struct BChannelsItem {
    pub spec: BChannelSpec,
}

impl BChannelsItem {
    pub fn new(spec: BChannelSpec) -> Self {
        Self { spec }
    }
}

impl BViewItem for BChannelsItem {
    fn build(&self, _s: &BState) -> BView {
        BView::label(self.spec.label)
    }
}

// `BChannelsList` — reads `channels_for(host)`.

#[derive(Clone, Copy, Debug)]
pub struct BChannelsList {
    pub host: BHostType,
}

impl BChannelsList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BChannelsList {
    fn items(&self, s: &BState) -> Vec<BView> {
        channels_for(self.host)
            .iter()
            .map(|c| BChannelsItem::new(*c).build(s))
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BChannels {
    pub host: BHostType,
}

impl BChannels {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BChannels {
    fn build(&self, s: &BState) -> BView {
        // The animation editors list their channels as a side column
        // (`time`); the rest list theirs from `set`.
        time::list(s, self.host).unwrap_or_else(|| BView::column(BChannelsList::new(self.host).items(s)))
    }
}
