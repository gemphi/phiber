//! The checkbox and the radio option (`BMark`), drawn once.
//!
//! A mark is an icon on a tile, as every icon in a row is: the row's glyph
//! tile (`BDim::GlyphTile`), on the resting tile shade while off, filled with
//! the theme's accent while on (`BFill::IconTileOn`); in the glass mode both
//! are tiles of glass. So a tick in a property panel, a menu, a
//! preference and the outliner is one thing, sits on its row's centre line
//! as any icon does, and says "on" in the one colour the theme says it in.
//! Every renderer that draws a control with a mark comes here (`button`,
//! `menu`, `ctrl`); none draws a box of its own.

use super::atom::icon_in;
use super::ctx::BRenderCtx;
use super::flex::{BAlign, BDim, BFill, BFlex, BFlexed, BJustify, BLen, BRadius};
use crate::theme::BStyleRole;
use crate::view::prim::BMark;
use gpui::prelude::*;
use gpui::AnyElement;

/// The ink on an accent tile: white, as the switch's knob is.
const ON_INK: [f32; 4] = [1.0, 1.0, 1.0, 0.95];
/// The same white as a theme colour, for the icon atom.
const ON_COLOR: crate::theme::BColor = (0.0, 0.0, 100.0, 0.95);

/// One mark, drawn.
pub fn view(mark: BMark, ctx: &BRenderCtx) -> AnyElement {
    let tile = BMarkPart::Tile(mark).flex(ctx).div(ctx);
    match mark {
        BMark::Check(true) => tile.child(icon_in("check", ON_COLOR, ctx)),
        BMark::Check(false) => tile.child(icon_in("square", ctx.style.colors.text_dim, ctx)),
        BMark::Radio(true) => tile.child(BMarkPart::Dot.flex(ctx).div(ctx)),
        BMark::Radio(false) => tile.child(icon_in("radio", ctx.style.colors.text_dim, ctx)),
    }
    .into_any_element()
}

/// The parts of a mark.
enum BMarkPart {
    /// The tile: a row's glyph tile, a box rounded as a button is or a disc
    /// for a radio, on the resting shade or the accent.
    Tile(BMark),
    /// A chosen radio's dot: a third of the tile, in the on ink.
    Dot,
}

impl BFlexed for BMarkPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match *self {
            Self::Tile(mark) => {
                let tile = BFlex::row()
                    .width(BLen::Dim(BDim::GlyphTile))
                    .height(BLen::Dim(BDim::GlyphTile))
                    .keeps_size()
                    .align(BAlign::Center)
                    .justify(BJustify::Center)
                    .fill(match mark.on() {
                        true => BFill::IconTileOn,
                        false => BFill::IconTileRest,
                    });
                match mark {
                    BMark::Check(_) => tile.radius(BRadius::Role(BStyleRole::Button)),
                    BMark::Radio(_) => tile.radius(BRadius::Pill),
                }
            }
            Self::Dot => {
                let dot = BLen::Measured((ctx.style.dimensions.icon_size / 2.5).round());
                BFlex::cell().width(dot).height(dot).keeps_size().radius(BRadius::Pill).fill(BFill::Swatch(ON_INK))
            }
        }
    }
}
