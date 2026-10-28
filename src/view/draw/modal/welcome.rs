//! Focused centered body layout for the viewport-filling Phiber Welcome form.

use super::super::flex::{BAlign, BFlex, BFlexed, BJustify, BLen, BSpace};
use super::super::ctx::BRenderCtx;
use super::super::walk;
use crate::view::BView;
use gpui::prelude::*;
use gpui::AnyElement;

pub struct BWelcome<'a>(pub &'a BView);

impl BWelcome<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let list = match self.0 {
            BView::Column { list } => list.as_slice(),
            _ => std::slice::from_ref(self.0),
        };
        let mut column = BWelcomePart::WelcomeColumn
            .flex(ctx)
            .div(ctx)
            .id("welcome-content-column")
            .debug_selector(|| "welcome-content-column".to_string());
        // Its cards are the brand's decked eggs (`deck.rs`).
        let decked = ctx.decked();
        for child in list {
            column = column.child(BWelcomePart::WelcomeLine.flex(ctx).div(ctx).child(walk(child, &decked)));
        }
        BWelcomePart::WelcomeLayout
            .flex(ctx)
            .div(ctx)
            .id("welcome-centered-layout")
            .debug_selector(|| "welcome-centered-layout".to_string())
            .child(column)
            .into_any_element()
    }
}

/// The Welcome form's own layout — its shape, not the modal vocabulary's.
///
/// These lived in `BModalPart` beside `Layer`, `Backdrop` and
/// `TitleBar`, which every modal uses. Nothing but this screen ever asked
/// for them, and a shared vocabulary carrying one screen's furniture is how
/// that vocabulary grows to sixteen variants.
enum BWelcomePart {
    /// The Welcome form: its content column, centred in the modal.
    WelcomeLayout,
    /// The column runs the window's width: every line in it centres itself,
    /// and the cards grow with the room (`splash::card_size`). It used to
    /// stop at the Splash spec's width, so a larger window only added
    /// backdrop round the same small grid.
    WelcomeColumn,
    /// One line of the column — the logo, its tagline, the tabs, the cards
    /// — as tall as what it holds, so the lines stack rather than share the
    /// window's height out between them.
    WelcomeLine,
}

impl BFlexed for BWelcomePart {
    fn flex(&self, _ctx: &BRenderCtx) -> BFlex {
        match *self {
            Self::WelcomeLayout => BFlex::row()
                .fills()
                .full()
                .min_height(BLen::Zero)
                .align(BAlign::Center)
                .justify(BJustify::Center)
                .pad(BSpace::PadLg),

            Self::WelcomeColumn => BFlex::column().width(BLen::Full).gap(BSpace::GapLg),

            Self::WelcomeLine => BFlex::column().width(BLen::Full).keeps_size(),
        }
    }
}
