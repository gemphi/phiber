//! Focused table family: cells, body rows, and headers, on one column model.
//! `view_row` and `view_header` are the named entry points; `view_row` sits
//! before `BTableRow::render` so the source-text test sees the selection
//! logic between the two wrappers.

use super::atom::icon_path;
use super::ctx::BRenderCtx;
use super::flex::{BAlign, BDim, BFill, BFlex, BFlexed, BJustify, BLen, BOverflow, BRadius, BSpace, BType};
use super::walk;
use crate::theme::BStyleRole;
use crate::view::BView;
use crate::view::prim::{BAction, BColAlign, BColWidth, BRowShade, BTableCell};
use gpui::prelude::*;
use gpui::{AnyElement, svg};

pub struct BTableRow<'a> {
    pub cell_list: &'a [BTableCell],
    pub shade: BRowShade,
    pub action: Option<&'a BAction>,
}

pub struct BTableHeader<'a>(pub &'a [BTableCell]);

struct BTableCellView<'a>(&'a BTableCell);

/// Renders a table body row through `BTableRow::render`, which owns the
/// selection logic, so the dispatcher never owns GPUI composition.
#[allow(dead_code)]
pub fn view_row(
    cell_list: &[BTableCell],
    shade: BRowShade,
    action: Option<&BAction>,
    ctx: &BRenderCtx,
) -> AnyElement {
    BTableRow { cell_list, shade, action }.render(ctx)
}

// ── BTableRow ─────────────────────────────────────────────────────────────

impl BTableRow<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let mut row = BTablePart::Row(self.shade).flex(ctx).div(ctx);
        for cell in self.cell_list {
            row = row.child(BTableCellView(cell).render(ctx));
        }

        let Some(action) = self.action else {
            return row.into_any_element();
        };
        let id = gpui::SharedString::from(table_row_id(self.cell_list, Some(action)));
        let handler = ctx.handler.clone();
        let action = action.clone();
        let dbg = id.clone();
        BTablePart::Clickable
            .flex(ctx)
            .div(ctx)
            .id(id)
            .debug_selector(move || dbg.to_string())
            .cursor_pointer()
            .child(row)
            .on_click(move |_event, window, app| {
                (std::rc::Rc::clone(&handler))(&action, window, app);
            })
            .into_any_element()
    }
}

/// Public compatibility wrapper — renders a table header row.
#[allow(dead_code)]
pub fn view_header(cell_list: &[BTableCell], ctx: &BRenderCtx) -> AnyElement {
    BTableHeader(cell_list).render(ctx)
}

// ── BTableHeader ───────────────────────────────────────────────────────────

impl BTableHeader<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let mut element = BTablePart::Header
            .flex(ctx)
            .div(ctx)
            .debug_selector(|| "table-header".to_string());
        // Each heading cell is its column and the gap before it, with no
        // divider and nothing painted: its label starts (or, for a
        // right-aligned column, ends) where the values do.
        let last = self.0.len().saturating_sub(1);
        for (n, cell) in self.0.iter().enumerate() {
            let head = BTablePart::HeadCell(cell.width, n == 0, n == last).flex(ctx).div(ctx).child(walk(&cell.view, ctx));
            element = element.child(head);
        }
        element.into_any_element()
    }
}

impl BTableCellView<'_> {
    fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let clipping = ctx.clipping();
        let content = match self.0.view.as_ref() {
            BView::Row { list } => match list.split_first() { Some((BView::Icon { name }, rest)) => lead(name, rest, &clipping), _ => walk(&self.0.view, &clipping) },
            view => walk(view, &clipping),
        };
        BTablePart::Cell(self.0.width, self.0.align).flex(ctx).div(ctx).child(content).into_any_element()
    }
}

/// A value that leads with its kind's glyph (a file row's folder or file):
/// the glyph a size over a button's, on an oval of the browser's resting
/// tile (`BFill::IconTileRest`), so the list reads as things with names
/// rather than names with marks beside them.
fn lead(name: &str, rest: &[BView], ctx: &BRenderCtx) -> AnyElement {
    let glyph = BTablePart::Glyph.flex(ctx).styled(svg().path(icon_path(name)), ctx);
    let tile = BTablePart::GlyphTile.flex(ctx).div(ctx).child(glyph);
    rest.iter().fold(BTablePart::Lead.flex(ctx).div(ctx).child(tile), |row, view| row.child(walk(view, ctx))).into_any_element()
}

fn table_row_id(cell_list: &[BTableCell], action: Option<&BAction>) -> String {
    if let Some(action) = action {
        let segment = match action {
            BAction::Set(path, value) => format!("set-{}-{value}", path.encode()),
            BAction::Adjust(path, value) => format!("adjust-{}-{value}", path.encode()),
            BAction::Toggle(path) => format!("toggle-{}", path.encode()),
            BAction::Open(path) => format!("open-{}", path.encode()),
            BAction::Reset(path) => format!("reset-{}", path.encode()),
        };
        return format!("table-row-{segment}");
    }
    let first = cell_list
        .first()
        .map(|cell| match cell.view.as_ref() {
            crate::view::BView::Label { text } => text.clone(),
            crate::view::BView::Button(ctl) => ctl.label.clone().unwrap_or_default(),
            _ => String::new(),
        })
        .unwrap_or_default();
    format!("table-row-{first}")
}

/// The parts of a table — one enum for the level, as every other level
/// states its parts (`BCardPart`, `BModalPart`, `BLogPart`).
enum BTablePart {
    /// The band a row is drawn in.
    ///
    /// One geometry, selected or not: a row used to shrink by a border and
    /// gain a margin when it became selected, so selecting or hovering one
    /// nudged it and everything below it — the "dancing" on hover. State may
    /// change what a row paints; it may never change what it measures, which
    /// is why the inset is a `Stroke` margin the row always carries.
    ///
    /// Its text stands a row's own pad (`ROW_PAD`) in from either end of
    /// the band, so a hovered or selected row's fill has room round its
    /// words instead of meeting the first letter.
    Row(BRowShade),
    /// The clickable is the row: its height is the row's, so the hover and
    /// selection fills cover the whole band rather than a strip of it.
    Clickable,
    /// A column heading is a line of words over the rows, laid as they are:
    /// the same inset from the list's edge, nothing painted under it. A
    /// heading fill sat inside the list's gutter and stopped short of its
    /// edges, so it read as a band cut off at both ends.
    Header,
    /// A column's heading cell: the body cell's column, the gap before it
    /// (after the first), and the row's pad at either end (first, last), so
    /// its label sits over its values. It paints nothing hovered; the hand
    /// is what says it sorts.
    HeadCell(BColWidth, bool, bool),
    /// A cell fills the row's height and centres its content in it. Sizing to
    /// content left the columns short of the row, so a value sat at whatever
    /// height its own text happened to be.
    Cell(BColWidth, BColAlign),
    /// A cell's glyph and the words after it, on the cell's centre line.
    Lead,
    /// The oval a leading glyph stands on: a tile's height, wider than it
    /// is tall, rounded to a pill.
    GlyphTile,
    /// The leading glyph: a button's icon and two pixels more.
    Glyph,
}

impl BFlexed for BTablePart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match *self {
            Self::Row(shade) => BFlex::row()
                .width(BLen::Full)
                .height(BLen::Measured(row_band(ctx)))
                .align(BAlign::Center)
                .pad_x(ROW_PAD)
                .gap(BSpace::GapMd)
                // The fill, the hover and the shade mapping are the tree's
                // and the list's too (`tone::row_fill` / `tone::row_hover`).
                .fill(BFill::Row(shade))
                .hovered(BFill::RowHover(shade))
                .radius(BRadius::Role(BStyleRole::MenuItem))
                .margin_x(BSpace::Stroke)
                .margin_y(BSpace::Stroke)
                .ink()
                .type_size(BType::Body),
            Self::Clickable => BFlex::row().width(BLen::Full).height(BLen::Measured(row_band(ctx))),
            Self::Header => BFlex::row()
                .width(BLen::Full)
                .height(BLen::Dim(BDim::CardHead))
                .align(BAlign::Center)
                .margin_x(BSpace::Stroke)
                .gap(BSpace::None)
                .ink()
                .type_size(BType::Body),
            Self::HeadCell(width, lead, last) => {
                let cell = BFlex::row()
                    .align(BAlign::Center)
                    .height(BLen::Full)
                    .min_height(BLen::Zero)
                    .overflow(BOverflow::Clip);
                let w = match width {
                    BColWidth::Fixed(w) => Some(w),
                    BColWidth::FileDate => Some(ctx.style.dimensions.file_date_column_width),
                    BColWidth::FileSize => Some(ctx.style.dimensions.file_size_column_width),
                    BColWidth::Flex => None,
                };
                let inset = ROW_PAD.px(ctx);
                let gap = ctx.style.spacing.gap_md;
                let cell = match lead {
                    true => cell.pad_left(ROW_PAD),
                    false => cell.pad_left(BSpace::GapMd),
                };
                let cell = if last { cell.pad_right(ROW_PAD) } else { cell };
                // The cell's span beyond its column: gap before, row's pad.
                let extra = [(!lead, gap), (lead, inset), (last, inset)].iter().filter(|(on, _)| *on).map(|(_, px)| px).sum::<f32>();
                match w {
                    None => cell.fills().min_width(BLen::Zero),
                    Some(w) => cell.width(BLen::Measured(w + extra)).keeps_size(),
                }
            }
            Self::Cell(width, align) => {
                let cell = BFlex::row()
                    .align(BAlign::Center)
                    .height(BLen::Full)
                    .min_height(BLen::Zero)
                    .overflow(BOverflow::Clip)
                    .justify(match align {
                        BColAlign::Start => BJustify::Start,
                        BColAlign::End => BJustify::End,
                    });
                // A right-aligned column ends where its heading's word does,
                // short of the heading's sort arrow.
                let cell = if align == BColAlign::End { cell.pad_right(BSpace::ArrowSlot) } else { cell };
                let fixed = |w: f32| cell.width(BLen::Measured(w)).keeps_size();
                match width {
                    BColWidth::Fixed(w) => fixed(w),
                    BColWidth::FileDate => fixed(ctx.style.dimensions.file_date_column_width),
                    BColWidth::FileSize => fixed(ctx.style.dimensions.file_size_column_width),
                    BColWidth::Flex => cell.fills().min_width(BLen::Zero),
                }
            }
            Self::Lead => BFlex::row().fills().min_width(BLen::Zero).height(BLen::Full).align(BAlign::Center).gap(BSpace::GapSm),
            Self::GlyphTile => BFlex::row().width(BLen::Measured(glyph_size(ctx) + 2.0 * BSpace::PadMd.px(ctx))).height(BLen::Dim(BDim::Tile)).keeps_size().align(BAlign::Center).justify(BJustify::Center).radius(BRadius::Pill).fill(BFill::IconTileRest),
            Self::Glyph => BFlex::cell().width(BLen::Measured(glyph_size(ctx))).height(BLen::Measured(glyph_size(ctx))).keeps_size().ink(),
        }
    }
}

/// A row's leading glyph: a button's icon and two pixels more.
fn glyph_size(ctx: &BRenderCtx) -> f32 {
    ctx.style.dimensions.icon_size + 2.0
}

/// A row's text from either end of its band, and a heading's over it: a
/// row's own pad, as a menu row's and a list entry's are.
const ROW_PAD: BSpace = BSpace::PadSm;

/// A row's own height: the row rung less the border it insets itself by at
/// top and bottom, so the band it occupies is exactly one row.
fn row_band(ctx: &BRenderCtx) -> f32 {
    ctx.style.dimensions.row_height - 2.0 * ctx.style.strokes.border
}
