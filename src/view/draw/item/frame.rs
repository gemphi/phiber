//! Where a row stands, which is what frames it: its height, and later its
//! columns and fill. A row's data never says; the context does.

use super::super::ctx::BRenderCtx;
use super::super::flex::BDim;
use crate::region::BRegionType;

/// A row's frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BItemFrame {
    /// In a popup: a menu's, a dropdown's options.
    Popup,
    /// In a side column's list: a source list's entry, a navigator's.
    Source,
    /// A property row: its label and its value or control on the split.
    Property,
    /// A tree's row: its depth's guides, its fold, its band, its drag.
    Tree,
}

impl BItemFrame {
    /// The frame `ctx` stands a row in: a source list wherever the rows are
    /// a side column's entries — a section's (`ctx.listed`) or a tools
    /// column that holds panels — else a popup's.
    pub fn of(ctx: &BRenderCtx) -> Self {
        let in_tools = ctx.region == Some(BRegionType::Tools) && ctx.host.tools_hold_panels() && !ctx.strip;
        match ctx.listed || in_tools {
            true => Self::Source,
            false => Self::Popup,
        }
    }

    /// The row's height: one of the two every row is — compact for a list's
    /// rows, the module for a property's.
    pub fn height(self) -> BDim {
        match self {
            Self::Popup | Self::Source | Self::Tree => BDim::CompactRow,
            Self::Property => BDim::CardHead,
        }
    }
}
