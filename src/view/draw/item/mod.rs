//! The one row renderer (`BItem`): every row the app draws is a lead, a
//! label, a value and a trail, framed by where it stands (`frame.rs`). A
//! menu's row and a source list's are this row; so, in turn, are a tree's,
//! a table's and a property's. What differs is data on the row, or the
//! frame — never a renderer of its own.

mod frame;
mod property;

pub use frame::BItemFrame;

use super::atom::icon_in;
use super::chrome;
use super::ctx::BRenderCtx;
use super::flex::{BAlign, BDim, BFill, BFlex, BFlexed, BJustify, BLen, BOverflow, BRadius, BSpace, BType};
use crate::theme::BStyleRole;
use crate::view::prim::{BItem, BItemValue, BLead};
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString};
use std::rc::Rc;

/// One row, in the frame where it stands.
pub fn view(item: &BItem, ctx: &BRenderCtx) -> AnyElement {
    view_in(item, BItemFrame::of(ctx), ctx)
}

/// One row in `frame`.
pub fn view_in(item: &BItem, frame: BItemFrame, ctx: &BRenderCtx) -> AnyElement {
    match frame {
        BItemFrame::Property => property::view(item, ctx),
        BItemFrame::Popup | BItemFrame::Source => list(item, frame, ctx),
    }
}

/// A read-out (`BView::Pair`): in the status bar a line of words after
/// one another (`pair::pair`); in a side column the property frame, the
/// label left and the value right.
pub fn pair(label: &str, value: &str, ctx: &BRenderCtx) -> AnyElement {
    match ctx.host == crate::host::BHostType::Statusbar {
        true => super::pair::pair(label, value, ctx),
        false => property::view(&BItem::read(format!("section-{label}"), label, BItemValue::Text(value.to_string())), ctx),
    }
}

/// A list's row, in a popup or a source list.
fn list(item: &BItem, frame: BItemFrame, ctx: &BRenderCtx) -> AnyElement {
    let mut row = BItemPart::Row(frame).flex(ctx).div(ctx);
    // Every row reserves its lead's slot, drawn or not, so a list's labels
    // share one left edge; a list with no lead anywhere keeps none
    // (`ctx.glyphs`).
    if ctx.glyphs || item.lead != BLead::None {
        row = row.child(lead(&item.lead, ctx));
    }
    row = row.child(BItemPart::Label.flex(ctx).div(ctx).child(SharedString::from(item.label.clone())));
    if let BItemValue::Shortcut(shortcut) = &item.value {
        row = row.child(BItemPart::Shortcut.flex(ctx).div(ctx).child(SharedString::from(shortcut.clone())));
    }
    if item.submenu {
        row = row.child(BItemPart::Submenu.flex(ctx).div(ctx).child(icon_in("chevron_right", ctx.style.colors.text_dim, ctx)));
    }
    let Some(action) = item.action.clone() else {
        return row.into_any_element();
    };
    let id = SharedString::from(item.id.clone());
    let element = BItemPart::Cap.flex(ctx).div(ctx).id(id.clone()).debug_selector(move || id.to_string()).cursor_pointer().child(row);
    // Flat at rest, lit under the pointer and when on, the one control
    // treatment (`chrome::paint`); a source list's entry takes its border on
    // hover and when chosen (`ctx.listed`). Muted, the same at its weight.
    let handler = ctx.handler.clone();
    let mut element = chrome::paint_state(element, item.active, item.muted, ctx)
        .on_click(move |_: &ClickEvent, window: &mut gpui::Window, app: &mut gpui::App| (Rc::clone(&handler))(&action, window, app));
    if let Some(hover) = item.hover.clone() {
        let handler = ctx.handler.clone();
        element = element.on_hover(move |over: &bool, window: &mut gpui::Window, app: &mut gpui::App| {
            if *over {
                (Rc::clone(&handler))(&hover, window, app);
            }
        });
    }
    element.into_any_element()
}

/// The lead in its slot: a glyph on the row's tile, a checkbox or radio
/// mark, a swatch; an empty slot keeps the tile's size and paints nothing.
fn lead(lead: &BLead, ctx: &BRenderCtx) -> AnyElement {
    match lead {
        BLead::Mark(mark) => super::mark::view(*mark, ctx),
        BLead::None => BItemPart::Slot { tile: false }.flex(ctx).div(ctx).into_any_element(),
        BLead::Glyph(name) => BItemPart::Slot { tile: true }.flex(ctx).div(ctx).child(icon_in(name, ctx.style.colors.text_dim, ctx)).into_any_element(),
        BLead::Swatch(color) => BItemPart::Slot { tile: false }.flex(ctx).div(ctx).child(super::ctrl::swatch_face(*color, ctx).div(ctx)).into_any_element(),
    }
}

/// A divider between a list's groups: one hairline rule, in the border
/// shade, a step lighter than the surface, so a group boundary is a line
/// you can see rather than a change of shade found only by hovering.
pub fn sep(ctx: &BRenderCtx) -> AnyElement {
    BItemPart::Separator.flex(ctx).div(ctx).into_any_element()
}

/// The parts of a row, in the order they are laid out.
enum BItemPart {
    /// The row: its frame's height and inset, its label's ink and type.
    Row(BItemFrame),
    /// The lead's slot: a glyph tile (`BDim::GlyphTile`), on the resting
    /// tile shade when it holds a glyph (`BFill::IconTileRest`).
    Slot { tile: bool },
    /// The label: it takes the slack, so the value and trail sit at the end.
    Label,
    /// A key binding at the row's end, a step quieter than the label.
    Shortcut,
    /// The arrow of a row that opens a menu of its own.
    Submenu,
    /// The clickable row: rounded on every corner and clipped, so its hover
    /// fill never bleeds past them, and keeping its size when its list is
    /// taller than the space (the list scrolls, its rows do not squash).
    Cap,
    Separator,
}

impl BFlexed for BItemPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match self {
            Self::Row(frame) => BFlex::row()
                .width(BLen::Full)
                .height(BLen::Dim(frame.height()))
                .keeps_size()
                .align(BAlign::Center)
                .gap(BSpace::GapSm)
                .pad_x(BSpace::RowInset)
                .ink()
                .type_size(BType::Body),
            Self::Slot { tile } => {
                let size = BLen::Dim(BDim::GlyphTile);
                let cell = BFlex::row().width(size).height(size).keeps_size().align(BAlign::Center).justify(BJustify::Center);
                match tile {
                    true => cell.fill(BFill::IconTileRest).radius(BRadius::Role(BStyleRole::Button)),
                    false => cell,
                }
            }
            Self::Label => BFlex::row().fills().min_width(BLen::Zero).overflow(BOverflow::Clip),
            Self::Shortcut => BFlex::row().dim_ink().type_size(BType::Caption),
            Self::Submenu => {
                let slot = BLen::Measured(ctx.style.dimensions.icon_size);
                BFlex::row().width(slot).height(slot).keeps_size().align(BAlign::Center).justify(BJustify::Center)
            }
            Self::Cap => BFlex::row()
                .width(BLen::Full)
                .keeps_size()
                .radius(BRadius::Role(BStyleRole::MenuItem))
                .overflow(BOverflow::Clip),
            Self::Separator => BFlex::row()
                .width(BLen::Full)
                .height(BLen::Measured(ctx.style.strokes.hairline))
                .fill(BFill::MenuDivider)
                .margin_y(BSpace::GapXs),
        }
    }
}
