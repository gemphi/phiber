//! The property frame: a row the module tall, its label on the left of the
//! split (Blender's `use_property_split`) and its value on the right, as an
//! inspector's label and its control stand. A read-out's text, a property's
//! controls, a checkbox's label and a switch's track all stand in it; the
//! row has no ground of its own, only what it holds.

use super::super::atom::label as word;
use super::super::ctx::BRenderCtx;
use super::super::flex::{BAlign, BDim, BFlex, BFlexed, BLen, BOverflow, BSpace, BType};
use super::super::split::VALUE_SHARE;
use super::super::walk;
use crate::view::prim::{BItem, BItemValue, BLead};
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString};
use std::rc::Rc;

/// One row in the property frame.
pub(super) fn view(item: &BItem, ctx: &BRenderCtx) -> AnyElement {
    let id = SharedString::from(item.id.clone());
    let mut row = BPropertyPart::Row.flex(ctx).div(ctx).id(id.clone()).debug_selector({
        let id = id.clone();
        move || id.to_string()
    });
    let lead = (item.lead != BLead::None).then(|| super::lead(&item.lead, ctx));
    row = match &item.value {
        // A label and its value: the split, label left, value right.
        BItemValue::Text(_) | BItemValue::Views(_) => {
            let label = lead.into_iter().fold(cell(BPropertyPart::Label, "label", &item.id, ctx), |cell, lead| cell.child(lead));
            let value = cell(BPropertyPart::Value, "value", &item.id, ctx);
            let value = match &item.value {
                BItemValue::Text(text) => value.child(word(text, ctx)),
                BItemValue::Views(views) => {
                    let filling = ctx.filling();
                    views.iter().fold(value, |value, view| value.child(walk(view, &filling)))
                }
                _ => value,
            };
            row.child(label.child(word(&item.label, ctx))).child(value)
        }
        // A checkbox's or a switch's row: the label takes the slack, the
        // switch's track stands at the end.
        _ => {
            let row = lead.into_iter().fold(row, |row, lead| row.child(lead));
            let row = row.child(BPropertyPart::Words.flex(ctx).div(ctx).child(SharedString::from(item.label.clone())));
            match item.value {
                BItemValue::Switch(on) => row.child(super::super::ctrl::switch_track(on, ctx)),
                _ => row,
            }
        }
    };
    let Some(action) = item.action.clone() else {
        return row.into_any_element();
    };
    let handler = ctx.handler.clone();
    row.cursor_pointer()
        .on_click(move |_: &ClickEvent, window: &mut gpui::Window, app: &mut gpui::App| (Rc::clone(&handler))(&action, window, app))
        .into_any_element()
}

/// One column of the split, with its id (`<row>-label`, `<row>-value`).
fn cell(part: BPropertyPart, slot: &str, row: &str, ctx: &BRenderCtx) -> gpui::Stateful<gpui::Div> {
    let id = SharedString::from(format!("{row}-{slot}"));
    part.flex(ctx).div(ctx).id(id.clone()).debug_selector(move || id.to_string())
}

/// The parts of a property row.
enum BPropertyPart {
    /// The row: at least the module tall, growing with stacked controls (a
    /// vector's X, Y, Z), at every row's inset; no ground of its own.
    Row,
    /// The label column, its share of the split; a long label is cut at its
    /// edge rather than running under the value.
    Label,
    /// The value column: the rest of the split, handed to what stands in it.
    Value,
    /// A checkbox's or switch's label, taking the slack.
    Words,
}

impl BFlexed for BPropertyPart {
    fn flex(&self, _ctx: &BRenderCtx) -> BFlex {
        match self {
            Self::Row => BFlex::row()
                .width(BLen::Full)
                .min_height(BLen::Dim(BDim::CardHead))
                .align(BAlign::Center)
                .gap(BSpace::GapXs)
                .pad_x(BSpace::RowInset)
                .ink()
                .type_size(BType::Body),
            Self::Label => BFlex::row().width(BLen::Share(1.0 - VALUE_SHARE)).min_width(BLen::Zero).align(BAlign::Center).gap(BSpace::GapSm).overflow(BOverflow::Clip),
            Self::Value => BFlex::row().width(BLen::Share(VALUE_SHARE)).min_width(BLen::Zero).align(BAlign::Center).gap(BSpace::GapXs),
            Self::Words => BFlex::row().fills().min_width(BLen::Zero),
        }
    }
}
