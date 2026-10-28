//! A menu's row (`BView::MenuItem`): the one row (`item`), from its control
//! — its glyph or mark, its label, its shortcut, the arrow of a submenu —
//! with the id a menu's rows answer to, `menu-<label>`. A popup's rows, a
//! source list's and a navigator's are all this row; the frame they stand in
//! is the item's to decide.

use super::ctx::BRenderCtx;
use crate::view::prim::{BControl, BItem};
use gpui::AnyElement;

pub fn view(ctl: &BControl, ctx: &BRenderCtx) -> AnyElement {
    let id = format!("menu-{}", ctl.label.as_deref().unwrap_or(""));
    super::item::view(&BItem::of(ctl, id), ctx)
}

pub fn sep(ctx: &BRenderCtx) -> AnyElement {
    super::item::sep(ctx)
}
