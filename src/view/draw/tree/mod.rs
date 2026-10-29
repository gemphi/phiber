//! Focused tree family: indent, guide, disclosure, and row renderers.
//!
//! A tree is a table with indentation: a row's depth is a run of indent
//! units before its arrow slot, which every row keeps (a leaf's is blank,
//! as Blender reserves it), and each unit of that run carries one hierarchy
//! line, in the other zebra band's colour (`guide::guide_fill`) — so a row
//! shows exactly as many lines as it is deep, inside its own background.

use super::atom;
use super::ctx::BRenderCtx;
use super::flex::{BAlign, BDim, BFill, BFlex, BFlexed, BJustify, BLen, BOverflow, BPlace, BRadius, BSpace, BType};
use super::walk;
use crate::theme::BStyleRole;
use crate::view::BView;
use crate::view::prim::{BAction, BItem, BItemTree, BItemValue, BLead, BRowShade};
use gpui::prelude::*;
use gpui::AnyElement;

mod disclosure;
use self::disclosure::BTreeDisclosure;

mod drag;
mod guide;

mod window;
pub use self::window::BRowWindow;

/// The parts of a tree row — one enum for the family, as every other level
/// and family states its parts (`BCardPart`, `BMenuPart`, `BRailPart`).
pub(super) enum BTreePart {
    /// The run of indent units before the arrow slot: a row's depth, drawn,
    /// each unit carrying its hierarchy line (`Guide`) in the row's
    /// opposite band.
    Indent(u16),
    /// One indent unit: a line down its middle, the row's height.
    IndentUnit,
    /// The line itself: a hairline, as tall as the row.
    Guide(BRowShade),
    /// The row itself. `shade` is what it paints at rest and `selected` is
    /// whether the pointer may repaint it — a selected row keeps its fill.
    Row { shade: BRowShade, selected: bool },
    /// A one-unit slot in the row: the type icon's, and the disclosure
    /// triangle's (`disclosure`). One shape, so the arrow column and the
    /// icon column line up down the whole tree by construction rather than
    /// by two places agreeing on a number.
    IconSlot,
    /// The glyph's tile inside its slot: the faint shade every icon stands
    /// on, one indent unit square, so the arrow and icon columns still line
    /// up.
    GlyphTile,
    /// The name and whatever the row lists after it: takes the slack, clips.
    Main,
    /// The controls at the row's end (the restriction toggles).
    Trail,
}

impl BFlexed for BTreePart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        let unit = ctx.style.dimensions.indent_unit;
        match *self {
            Self::Indent(depth) => {
                BFlex::row().width(BLen::Measured(unit * f32::from(depth))).height(BLen::Full).keeps_size()
            }
            Self::IndentUnit => BFlex::row()
                .width(BLen::Measured(unit))
                .height(BLen::Full)
                .keeps_size()
                .justify(BJustify::Center),
            Self::Guide(shade) => BFlex::cell()
                .width(BLen::Measured(ctx.style.strokes.hairline))
                .height(BLen::Full)
                .keeps_size()
                .fill(self::guide::guide_fill(shade, ctx)),
            // A tree row takes the same fills as every other selectable
            // control in the app — the row shade when banded or selected,
            // the shared item hover otherwise. Palette-only `row_alt` /
            // `row_selected` entries let the outliner drift to shades no
            // other region used.
            //
            // At rest it is its editor's body (`host_body_surface`), the
            // surface the list sits on, so the Outliner's content and the
            // region round it are one ground — lifted rows read as strips on
            // a darker parent.
            Self::Row { shade, selected } => {
                let row = BFlex::row()
                    .width(BLen::Full)
                    .height(BLen::Dim(BDim::TreeRow))
                    // A row keeps its height: when the host is short the list
                    // scrolls (`column`) rather than squashing every row.
                    .keeps_size()
                    .align(BAlign::Center)
                    .place(BPlace::Anchor)
                    .radius(BRadius::Role(BStyleRole::MenuItem))
                    .pad_right(BSpace::PadMd)
                    .overflow(BOverflow::Clip)
                    .ink()
                    .type_size(BType::Body)
                    // Asked of the shade, not listed here. The list said
                    // `Selected | Active`, so `Alt` — the band — fell into the
                    // catch-all and painted the body: the Outliner computed a
                    // banded shade for every other row and the renderer threw
                    // it away, which is why its rows had no background at all
                    // but the selected one. The doc two lines up has said
                    // "banded or selected" the whole time.
                    //
                    // `BRowShade::fill` already knows which shades have a
                    // fill; a second list of them here could only drift from
                    // it, and did.
                    .fill(match shade.fill(ctx.style) {
                        Some(_) => BFill::Row(shade),
                        None => BFill::HostBody,
                    });
                if selected { row } else { row.hovered(BFill::RowHover(shade)) }
            }
            Self::IconSlot => BFlex::row()
                .width(BLen::Measured(unit))
                .height(BLen::Full)
                .keeps_size()
                .align(BAlign::Center)
                .justify(BJustify::Center),
            Self::GlyphTile => BFlex::row()
                .width(BLen::Measured(unit))
                .height(BLen::Measured(unit))
                .keeps_size()
                .align(BAlign::Center)
                .justify(BJustify::Center)
                .radius(BRadius::Role(BStyleRole::Button))
                .fill(BFill::IconTileRest),
            Self::Main => BFlex::row()
                .fills()
                .min_width(BLen::Zero)
                .align(BAlign::Center)
                .gap(BSpace::GapXs)
                .overflow(BOverflow::Clip),
            Self::Trail => {
                BFlex::row().keeps_size().align(BAlign::Center).gap(BSpace::GapXs)
            }
        }
    }
}

pub struct BTreeIndent {
    pub depth: u16,
    pub shade: BRowShade,
}

pub struct BTreeRow<'a> {
    pub depth: u16,
    pub expanded: Option<bool>,
    pub toggle: Option<&'a BAction>,
    /// Between the disclosure and the icon (a collection's checkbox).
    pub lead: &'a [BView],
    pub icon: &'a str,
    pub list: &'a [BView],
    pub trail: &'a [BView],
    pub shade: BRowShade,
    /// Dragged and dropped on (`drag.rs`).
    pub drag: Option<&'a crate::view::prim::BTreeDrag>,
}

impl BTreeIndent {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let mut run = BTreePart::Indent(self.depth).flex(ctx).div(ctx);
        for _ in 0..self.depth {
            run = run.child(BTreePart::IndentUnit.flex(ctx).div(ctx).child(BTreePart::Guide(self.shade).flex(ctx).div(ctx)));
        }
        run.into_any_element()
    }
}

impl BTreeRow<'_> {
    /// The row as the one row (`item`) in its tree frame: the name's action
    /// lifted to the row (Blender selects an outliner item by clicking
    /// anywhere on its row, and a labelled button as the name cell gave the
    /// name its own hover inside the row's), the rest of its cells after the
    /// name, its toggles at its end.
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let (select, label, rest) = match self.list.split_first() {
            Some((BView::Button(ctl), rest)) if ctl.icon.is_none() && ctl.action.is_some() => (ctl.action.clone(), ctl.label.clone().unwrap_or_default(), rest.to_vec()),
            _ => (None, String::new(), self.list.to_vec()),
        };
        let id = match (&select, self.drag) {
            (_, Some(drag)) => format!("outliner-row-{}", drag.payload),
            (Some(action), None) => super::button::element_id("tree-row", None, action),
            (None, None) => String::new(),
        };
        let tree = BItemTree {
            depth: self.depth,
            expanded: self.expanded,
            toggle: self.toggle.cloned(),
            leads: self.lead.to_vec(),
            shade: self.shade,
            drag: self.drag.cloned(),
        };
        let item = BItem {
            lead: BLead::Glyph(self.icon.to_string()),
            action: select,
            trail: self.trail.to_vec(),
            tree: Some(tree),
            ..BItem::read(id, label, BItemValue::Views(rest))
        };
        super::item::view_in(&item, super::item::BItemFrame::Tree, ctx)
    }
}

/// A row in the tree frame (`item::BItemFrame::Tree`): its depth's guides,
/// its fold, what leads it (a collection's checkbox), its glyph in the
/// indent-unit slot (so a child's arrow stands under its parent's icon), its
/// name and cells, its toggles; its band at rest and its drag.
pub(in crate::view::draw) fn frame(item: &BItem, tree: &BItemTree, ctx: &BRenderCtx) -> AnyElement {
    let selected = matches!(tree.shade, BRowShade::Selected | BRowShade::Active);
    let mut element = BTreePart::Row { shade: tree.shade, selected }.flex(ctx).div(ctx);
    element = element.child(BTreeIndent { depth: tree.depth, shade: tree.shade }.render(ctx));
    element = element.child(BTreeDisclosure { expanded: tree.expanded, toggle: tree.toggle.as_ref() }.render(ctx));
    for child in &tree.leads {
        element = element.child(walk(child, ctx));
    }
    let icon = match &item.lead {
        BLead::Glyph(icon) => icon.as_str(),
        _ => "",
    };
    let glyph = match icon.is_empty() {
        true => BTreePart::IconSlot.flex(ctx).div(ctx),
        false => BTreePart::IconSlot.flex(ctx).div(ctx).child(BTreePart::GlyphTile.flex(ctx).div(ctx).child(atom::icon(icon, ctx))),
    };
    element = element.child(glyph);
    let mut main = BTreePart::Main.flex(ctx).div(ctx);
    if !item.label.is_empty() {
        main = main.child(walk(&BView::label(item.label.clone()), ctx));
    }
    if let BItemValue::Views(cells) = &item.value {
        for child in cells {
            main = main.child(walk(child, ctx));
        }
    }
    element = element.child(main);
    if !item.trail.is_empty() {
        let trail = item.trail.iter().fold(BTreePart::Trail.flex(ctx).div(ctx), |trail, child| trail.child(walk(child, ctx)));
        element = element.child(trail);
    }
    if item.id.is_empty() {
        return element.into_any_element();
    }
    let mut element = element.id(gpui::SharedString::from(item.id.clone()));
    if let Some(action) = item.action.clone() {
        let handler = ctx.handler.clone();
        element = element.cursor_pointer().on_click(move |_event, window, app| {
            (std::rc::Rc::clone(&handler))(&action, window, app);
        });
    }
    match &tree.drag {
        Some(drag) => self::drag::BRowDrag::wire(element, drag, icon, ctx).into_any_element(),
        None => element.into_any_element(),
    }
}
