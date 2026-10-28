//! Focused renderer for `BView::Row` — flex-row with bounded height.

use super::ctx::BRenderCtx;
use super::{walk, strip};
use super::flex::BFlexed;
use crate::view::BView;
use gpui::prelude::*;
use gpui::AnyElement;

pub struct BRow<'a>(pub &'a [BView]);

impl BRow<'_> {
    pub fn render(&self, ctx: &BRenderCtx<'_>) -> AnyElement {
        // A row is a strip along the horizontal axis. In a header its panes
        // already divide the strip three ways, so its groups hug; elsewhere
        // the groups divide the slack themselves.
        // A row inside any chrome strip — a header region, or a toolbar /
        // footer panel in a modal — holds controls, and controls sit on the
        // strip's line. Everywhere else a row holds columns.
        let in_header = ctx.strip
            || matches!(
                ctx.region,
                Some(crate::region::BRegionType::Header)
                    | Some(crate::region::BRegionType::ToolHeader)
                    | Some(crate::region::BRegionType::Footer)
            );
        // A strip row that holds a field — the file browser's directory and
        // search, the execute bar's file name — gives the field the slack:
        // its group grows and the field with it, the way Blender's path
        // strip stretches the directory across the region.
        // The file browser's execute bar spans its strip the same way: it is
        // three groups across the dialog (`file::execute`) — what is picked,
        // what the dialog does, Cancel and the action — not a run of
        // controls hugging its start.
        let has_field = self.0.iter().any(|v| matches!(v, BView::SearchField { .. }))
            || ctx.region == Some(crate::region::BRegionType::Execute);
        // A content row holds columns (panels, a column, a grid) and stretches
        // them across its height. A row of atoms — an icon beside a name in a
        // file row, a label beside a value — is a line of controls: they sit
        // on its centre line and run from its start, not stretched to its
        // height with the icon pinned mid-cell.
        let holds_columns = self.0.iter().any(|v| {
            matches!(
                v,
                BView::Card(_)
                    | BView::Column { .. }
                    | BView::Row { .. }
                    | BView::Grid { .. }
                    | BView::Canvas { .. }
            )
        });
        let fit = if in_header && has_field {
            strip::BStripFit::Divide
        } else if in_header {
            strip::BStripFit::Hug
        } else if holds_columns {
            strip::BStripFit::Fill
        } else {
            strip::BStripFit::Divide
        };
        // A row of answers — a dialog's Save, Don't Save, Cancel after a
        // spacer — stands at the row's end and takes half of it, its buttons
        // sharing that half equally (`button.rs`, `answer`).
        if let Some(answers) = answers(self.0) {
            let half = answers.iter().fold(BRowPart::Answers.flex(ctx).div(ctx), |half, view| half.child(walk(view, ctx)));
            return BRowPart::AnswerRow.flex(ctx).div(ctx).child(half).into_any_element();
        }
        let groups: Vec<Vec<AnyElement>> = groups(self.0)
            .into_iter()
            .map(|views| views.iter().map(|v| walk(v, ctx)).collect())
            .collect();
        let mut element = strip::place_kept(strip::BStripAxis::Row, fit, groups, ctx);
        element = if in_header {
            // A header row passes the strip's height straight through and
            // centres its controls on the strip's line; a cap's `self_end`
            // still measures against the strip rather than against the row.
            // A row its author split into groups (`Spacer`) spans its pane,
            // so the groups land at its start, middle and end — the file
            // browser's display icons across the details column.
            let spreads = self.0.iter().any(|v| matches!(v, BView::Spacer));
            let element = element.h_full().items_center();
            if has_field || spreads { element.w_full() } else { element }
        } else {
            element.items_stretch().w_full().h_full()
        };
        element.into_any_element()
    }
}

/// A row is a set of groups separated by `BView::Spacer`: whatever the author
/// wanted kept together stays together, and the groups are pushed to the
/// strip's start, centre and end.
fn groups(list: &[BView]) -> Vec<&[BView]> {
    let mut out = Vec::new();
    let mut start = 0usize;
    for (n, child) in list.iter().enumerate() {
        if matches!(child, BView::Spacer) {
            out.push(&list[start..n]);
            start = n + 1;
        }
    }
    out.push(&list[start..]);
    // An empty group is a place: a row that starts with a spacer puts what
    // follows at its end, one that ends with one keeps it at its start
    // (`strip::slot`). Dropping them put `[spacer, buttons]` at the start.
    out
}

/// The answers of a row of answers: the buttons after a leading spacer,
/// every one an action (a word and nothing else), or `None`.
fn answers(list: &[BView]) -> Option<&[BView]> {
    let (BView::Spacer, rest) = list.split_first()? else { return None };
    let action = |view: &BView| matches!(view, BView::Button(c) if c.label.is_some() && c.icon.as_deref().unwrap_or("").is_empty() && c.mark.is_none() && c.swatch.is_none());
    (!rest.is_empty() && rest.iter().all(action)).then_some(rest)
}

/// The parts of a row of answers.
enum BRowPart {
    /// The row: its answers pushed to its end.
    AnswerRow,
    /// The answers: half the row, a gap between them.
    Answers,
}

impl super::flex::BFlexed for BRowPart {
    fn flex(&self, _ctx: &BRenderCtx) -> super::flex::BFlex {
        use super::flex::{BAlign, BFlex, BJustify, BLen, BSpace};
        match self {
            Self::AnswerRow => BFlex::row().width(BLen::Full).align(BAlign::Center).justify(BJustify::End),
            Self::Answers => BFlex::row().width(BLen::Share(0.5)).align(BAlign::Center).gap(BSpace::GapXs),
        }
    }
}
