//! Focused renderer for `BView::Column` — flex-col filling its slot.

use super::ctx::BRenderCtx;
use super::flex::{BFlex, BLen, BSpace};
use super::walk;
use crate::view::BView;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BColumn<'a>(pub &'a [BView]);

/// A view that is only words — a label, a `name: value`, a paragraph. Words
/// take a row's inset (`BSpace::RowInset`) wherever they stand, in a card's
/// body or a column of rows, so a heading over a group of switches starts
/// where the switches' own labels do; a control carries its own.
pub fn is_words(view: &BView) -> bool {
    matches!(view, BView::Label { .. } | BView::Pair { .. } | BView::Prose { .. })
}

impl BColumn<'_> {
    pub fn render(&self, ctx: &BRenderCtx<'_>) -> AnyElement {
        // A column of panels is a band stack — a modal composing its own
        // header / tool bar / content / footer, the way a host composes
        // regions. Bands stack flush: a gap between them showed the surface
        // behind as a stripe down each side of the dialog. Loose content
        // keeps its gap.
        let bands = !self.0.is_empty()
            && self
                .0
                .iter()
                .all(|child| matches!(child, BView::Card(_)));
        let gap = if bands {
            ctx.style.spacing.gap_none
        } else {
            ctx.style.spacing.gap_xs
        };
        // A tree is a list: it scrolls when the host is short, and its
        // hierarchy lines run the whole height rather than stopping at the
        // last row.
        let deepest = self
            .0
            .iter()
            .filter_map(|child| match child {
                BView::TreeRow { depth, .. } => Some(*depth),
                _ => None,
            })
            .max();
        let mut element = div()
            .id(gpui::SharedString::from("column"))
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_h_0()
            .min_w_0()
            .overflow_y_scroll()
            .gap(px(gap));
        let Some(_depth) = deepest else {
            for child in self.0 {
                element = element.child(match is_words(child) {
                    true => BFlex::row().pad_x(BSpace::RowInset).div(ctx).child(walk(child, ctx)).into_any_element(),
                    false => walk(child, ctx),
                });
            }
            return element.into_any_element();
        };
        // A tree lays out only the rows in view (`tree::BRowWindow`).
        let key = gpui::SharedString::from(format!("tree-{:?}-{}", ctx.area, ctx.host.slug()));
        let window = super::tree::BRowWindow::new(key, self.0, ctx);
        let hovered = std::rc::Rc::clone(&window.hovered);
        element = element
            .track_scroll(&window.handle)
            .on_hover(move |over, _, _| hovered.set(*over))
            .relative();
        let spacer = |height: f32| BFlex::cell().width(BLen::Full).height(BLen::Measured(height)).keeps_size().div(ctx).into_any_element();
        element = element.children(window.above().map(spacer));
        for child in &self.0[window.first..window.last] {
            element = element.child(walk(child, ctx));
        }
        element.children(window.below().map(spacer)).into_any_element()
    }
}
