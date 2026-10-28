//! The right pane of a tool strip over a details column, halved: the view
//! buttons (the row's first group, up to its first spacer) take the first
//! half of the column, each a wide tile in an equal share of it; the rest of
//! the row (the filters, the gear) stands in the other half as the row it
//! was. The File Browser's view modes are its one user.

use super::super::ctx::BRenderCtx;
use super::super::flex::{BAlign, BFlex, BFlexed, BLen, BSpace};
use super::super::walk;
use crate::pane::BPaneType;
use crate::region::BRegionType;
use crate::view::BView;
use crate::view::layout::BPaneLayout;
use gpui::prelude::*;
use gpui::{Div, Stateful};

/// Whether this pane is halved: the right pane of a tool strip laid out as
/// the columns under it, over a host's details column.
pub(super) fn applies(kind: BPaneType, columns: bool, ctx: &BRenderCtx) -> bool {
    columns && kind.is_right() && ctx.host.has_details_column() && ctx.region == Some(BRegionType::ToolHeader)
}

/// `view` into the halved pane `element`: a part whose body is a row is
/// split at its first spacer; anything else, a row with no spacer among
/// them, is walked as it is and so ends at the pane's far edge as every
/// right-hand group does.
pub(super) fn halved(element: Stateful<Div>, view: &BView, ctx: &BRenderCtx) -> Stateful<Div> {
    let list = match view {
        BView::Part { body, .. } => match body.as_ref() {
            BView::Row { list } => list,
            _ => return element.child(walk(view, ctx)),
        },
        _ => return element.child(walk(view, ctx)),
    };
    let Some(at) = list.iter().position(|view| matches!(view, BView::Spacer)) else {
        return element.child(walk(view, ctx));
    };
    // Each control is handed its share's width (`filling`), so an icon
    // button runs it as a wide tile (`button.rs`).
    let filling = ctx.filling();
    let half = list[..at].iter().fold(BHalfPart::Half.flex(ctx).div(ctx), |half, view| half.child(BHalfPart::Share.flex(ctx).div(ctx).child(walk(view, &filling))));
    let rest = BView::row(list[at..].iter().skip(1).cloned().collect());
    element.child(half).child(BHalfPart::Rest.flex(ctx).div(ctx).child(walk(&rest, ctx)))
}

/// The parts of a halved pane.
enum BHalfPart {
    /// The first half of the column, from its start.
    Half,
    /// One control's equal share of the half.
    Share,
    /// The rest of the row, after the half.
    Rest,
}

impl BFlexed for BHalfPart {
    fn flex(&self, _ctx: &BRenderCtx) -> BFlex {
        match self {
            Self::Half => BFlex::row().width(BLen::Share(0.5)).keeps_size().align(BAlign::Center).gap(BSpace::GapXs),
            Self::Share => BFlex::row().fills().min_width(BLen::Zero).align(BAlign::Center),
            Self::Rest => BFlex::row().fills().min_width(BLen::Zero).height(BLen::Full).align(BAlign::Center),
        }
    }
}
