//! The property-split row (`BView::Split`) — Blender's
//! `uiLayout.use_property_split` (`ui_item_prop_split`): the one row
//! (`item`) in its property frame, the label in one column, the value in the
//! other, one row tall. Its ids keep the `section-…` names the old `Section`
//! panel role had.

use super::ctx::BRenderCtx;
use crate::view::BView;
use crate::view::prim::{BItem, BItemValue};
use gpui::AnyElement;

/// Blender's `use_property_split`: the value column takes 60% of the row and
/// the label the other 40% (`UI_ITEM_PROP_SEP_DIVIDE`). Two thirds clipped a
/// checkbox's label in a sidebar ("Outline Sele…") for a field no wider.
pub(super) const VALUE_SHARE: f32 = 0.6;

pub struct BSplit<'a>(pub &'a [BView]);

impl BSplit<'_> {
    /// The split as the one row: its first view, a label, in the label
    /// column; the rest in the value column.
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let (label, values) = match self.0.split_first() {
            Some((BView::Label { text }, rest)) => (text.clone(), rest),
            _ => (String::new(), self.0),
        };
        let id = match label.is_empty() {
            true => "section".to_string(),
            false => format!("section-{label}"),
        };
        let item = BItem::read(id, label, BItemValue::Views(values.to_vec()));
        super::item::view_in(&item, super::item::BItemFrame::Property, ctx)
    }
}
