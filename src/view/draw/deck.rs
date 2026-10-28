//! The brand's decked card: the Welcome window's cards (`ctx.decked`).
//!
//! A softened egg, rounder at its crown than at its foot, standing on the
//! brand's hard shade to its right and below, the way the logo's letters
//! stand on theirs (`BFlex::casts`). Inside it, a deck at each end, as
//! Phiegg's cards carry theirs (`puikit` `Card depth`): three bands in three
//! shades of the card, the one at the edge the widest and strongest, each
//! band inward 75% of the one before it. Three bands, three shades: phi's
//! three by three. The card's picture stands between the two decks.
//!
//! Every number here is a power of the golden ratio of the card's edge, so
//! the shape holds at any size the window gives the cards.

use super::ctx::BRenderCtx;
use super::flex::{BAlign, BEdge, BFill, BFlex, BFlexed, BJustify, BLen, BOverflow, BRadius, BSpace};
use crate::theme::BSurfaceTone;
use gpui::prelude::*;
use gpui::AnyElement;

/// 1/φ, 1/φ², 1/φ⁶, 1/φ⁷.
const PHI_1: f32 = 0.618;
const PHI_2: f32 = 0.382;
const PHI_6: f32 = 0.0557;
const PHI_7: f32 = 0.0344;
/// How much narrower each band is than the one outside it.
const NARROWING: f32 = 0.75;
/// The bands' shades, outermost first: the accent over the card, a step
/// stronger toward the edge, in faint steps, as a Phiegg deck's copies take
/// more tint the further out they stand. The bands touch: their shades are
/// what parts them, not a gap.
const SHADES: [f32; 3] = [0.15, 0.10, 0.06];

/// How far the brand's shade falls right and below a card `size` px across:
/// what the card's name has to clear under it.
pub fn shade(size: f32) -> f32 {
    (size * PHI_6).round().max(3.0)
}

/// A card `size` px across holding `picture`, decked.
pub fn frame(size: f32, picture: AnyElement, ctx: &BRenderCtx) -> AnyElement {
    let deck = |edge: BDeckEdge| {
        let order: [usize; 3] = match edge {
            BDeckEdge::Top => [0, 1, 2],
            BDeckEdge::Bottom => [2, 1, 0],
        };
        order.into_iter().fold(BDeckPart::Deck.flex(ctx).div(ctx), |deck, k| deck.child(BDeckPart::Band { k, size }.flex(ctx).div(ctx)))
    };
    BDeckPart::Egg(size)
        .flex(ctx)
        .div(ctx)
        .child(deck(BDeckEdge::Top))
        .child(BDeckPart::Face.flex(ctx).div(ctx).child(picture))
        .child(deck(BDeckEdge::Bottom))
        .into_any_element()
}

/// The two ends a deck stands at.
#[derive(Clone, Copy)]
enum BDeckEdge {
    Top,
    Bottom,
}

/// The parts of a decked card.
enum BDeckPart {
    /// The card: an egg `size` px across, crown radius 1/φ² of it, foot
    /// radius 1/φ of that, on the card tone, casting the brand's shade.
    Egg(f32),
    /// One end's deck: its bands stacked, centred.
    Deck,
    /// A band, `k` steps in from the edge.
    Band { k: usize, size: f32 },
    /// What stands between the decks: the card's picture, centred.
    Face,
}

impl BFlexed for BDeckPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match *self {
            Self::Egg(size) => {
                let crown = size * PHI_2;
                let foot = crown * PHI_1;
                BFlex::column()
                    .width(BLen::Measured(size))
                    .height(BLen::Measured(size))
                    .keeps_size()
                    .align(BAlign::Center)
                    .justify(BJustify::Between)
                    .pad_y(BSpace::PadSm)
                    .radius(BRadius::Each { tl: crown, tr: crown, br: foot, bl: foot })
                    .fill(BFill::Tone(BSurfaceTone::Card))
                    .border(BEdge::All)
                    .overflow(BOverflow::Clip)
                    .casts(shade(size))
            }
            Self::Deck => BFlex::column().width(BLen::Full).align(BAlign::Center).gap(BSpace::None).keeps_size(),
            Self::Band { k, size } => {
                let accent = ctx.style.colors.accent;
                let width = size * PHI_1 * NARROWING.powi(k as i32);
                BFlex::cell()
                    .width(BLen::Measured(width))
                    .height(BLen::Measured((size * PHI_7).round().max(3.0)))
                    .keeps_size()
                    .radius(BRadius::Pill)
                    .fill(BFill::SwatchHsl((accent.0, accent.1, accent.2, SHADES[k])))
            }
            Self::Face => BFlex::row().fills().width(BLen::Full).min_height(BLen::Zero).align(BAlign::Center).justify(BJustify::Center),
        }
    }
}
