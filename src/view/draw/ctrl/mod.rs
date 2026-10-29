//! The one renderer for controls (`BView::Ctrl`): a widget per
//! `BCtrlKind`, every edit a `Set` of the control's key.
//!
//! A control composes the atoms that already exist — a button for a
//! checkbox or a step, a tab for a radio option, the field shell for typed
//! text, a popover for a dropdown — so a checkbox in a property panel is
//! the same checkbox as in a filter menu, and a number in the render panel
//! steps the way a frame number does. The number field is `number.rs`.

mod complex;
mod number;
mod picker;
mod swatch;

use self::swatch::BCtrlSwatch;
use super::flex::{BAlign, BDim, BEdge, BFill, BFlex, BFlexed, BJustify, BLen, BRadius, BSpace};
use super::ctx::BRenderCtx;
use super::{walk, button, search, track};
use super::track::BTrack;
use crate::state::val::BValue;
use crate::theme::BStyleRole;
use crate::view::BView;
use crate::view::prim::{BAction, BControl, BCtrl, BCtrlKind, BItem, BItemValue, BLead, BMark, BPopoverSide};
use gpui::prelude::*;
use gpui::AnyElement;

enum BCtrlPart {
    /// A row of the control's pieces: steps around a number, options of a radio.
    Row,
    /// A colour's swatch, filled with the value — in whichever spelling the
    /// control carries it (`BCtrlSwatch`).
    Swatch(BCtrlSwatch),
    /// The pill track, `true` while on: accent-filled with the knob at the
    /// end, else the card tone with the knob at the start.
    Track(bool),
    /// The knob: a white disc with a hairline, so it reads on either fill.
    Knob,
    /// A checkbox alone in a property row's value column: the mark on the
    /// row's centre line, a control's height.
    CheckBox,
}

/// The switch's track and knob, in pixels — `UISwitch`'s proportions at the
/// row's scale: a track a little wider than two knobs, the knob inset by the
/// track's padding either side.
const SWITCH_TRACK: (f32, f32) = (34.0, 20.0);
const SWITCH_KNOB: f32 = 16.0;

/// An off track: `ink` at the strength that reads on any surface.
const fn switch_off(ink: crate::theme::BColor) -> crate::theme::BColor {
    (ink.0, ink.1, ink.2, 0.35)
}

impl BFlexed for BCtrlPart {
    fn flex(&self, _c: &BRenderCtx) -> BFlex {
        match self {
            Self::Row => BFlex::row().align(BAlign::Center).gap(BSpace::GapXs).min_width(BLen::Zero),
            Self::CheckBox => BFlex::row().align(BAlign::Center).self_align(super::flex::BSelf::Center).height(BLen::Dim(BDim::ButtonHeight)),
            Self::Track(on) => BFlex::row()
                .width(BLen::Measured(SWITCH_TRACK.0))
                .height(BLen::Measured(SWITCH_TRACK.1))
                .keeps_size()
                .radius(BRadius::All(SWITCH_TRACK.1 / 2.0))
                .align(BAlign::Center)
                .justify(if *on { BJustify::End } else { BJustify::Start })
                .pad_x(BSpace::GapXs)
                // Off, the track is still drawn — the dimmed ink at a third —
                // so a switch keeps its shape either way and only the knob
                // moves; on the card tone it vanished and the knob sat alone.
                .fill(if *on { BFill::SwatchHsl(_c.style.colors.accent) } else { BFill::SwatchHsl(switch_off(_c.style.colors.text_dim)) }),
            Self::Knob => BFlex::cell()
                .width(BLen::Measured(SWITCH_KNOB))
                .height(BLen::Measured(SWITCH_KNOB))
                .keeps_size()
                .radius(BRadius::All(SWITCH_KNOB / 2.0))
                .fill(BFill::Swatch([1.0, 1.0, 1.0, 0.95]))
                .border(BEdge::All),
            Self::Swatch(value) => BFlex::cell()
                .width(BLen::Dim(super::flex::BDim::ButtonHeight))
                .height(BLen::Dim(super::flex::BDim::ButtonHeight))
                .keeps_size()
                .radius(BRadius::Role(BStyleRole::Field))
                .border(BEdge::All)
                .fill(match value {
                    BCtrlSwatch::Rgba(rgba) => BFill::Swatch(*rgba),
                    BCtrlSwatch::Hsl(color) => BFill::SwatchHsl(*color),
                }),
        }
    }
}

/// One control, drawn.
///
/// A control composes atoms that already exist — a button for a checkbox or
/// a step, a tab for a radio option, the field shell for typed text, a
/// popover for a dropdown — so a checkbox in a property panel is the same
/// checkbox as in a filter menu. Each kind is a method of this type rather
/// than a free function beside it, so the widget a kind draws as is reached
/// through the control, not from anywhere.
pub struct BCtrlView<'a>(pub &'a BCtrl);

impl BCtrlView<'_> {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        match self.0.kind {
            BCtrlKind::Check => self.flag(ctx),
            BCtrlKind::Switch => self.switch(ctx),
            BCtrlKind::Select => self.select(ctx),
            BCtrlKind::Int { .. } | BCtrlKind::Number { .. } => self.number(ctx),
            BCtrlKind::Slider { .. } => self.slider(ctx),
            BCtrlKind::Text | BCtrlKind::Search | BCtrlKind::Path => self.text(ctx),
            BCtrlKind::Color => self.color(ctx),
        }
    }

    /// A boolean: one control that answers to its own state, drawn as a
    /// button carrying the glyph for the state it is in.
    ///
    /// It took the glyphs as `Option`s and fell back to a tab — fill says the
    /// state, no glyph — for `BCtrlKind::Toggle`, which no caller ever
    /// produced. The glyphs are glyphs again now that the fallback has gone
    /// with that variant.
    /// A checkbox: the one row (`item`) leading with the checkbox mark, its
    /// label after it, the whole row flipping the box. A box with no label
    /// stands alone in a property row's value column, the mark and nothing
    /// else.
    fn flag(&self, ctx: &BRenderCtx) -> AnyElement {
        let ctrl = self.0;
        let on = matches!(ctrl.value, BValue::Bool(true));
        let id = format!("check-{}", ctrl.key.encode());
        let Some(label) = &ctrl.label else {
            let (handler, action, id) = (ctx.handler.clone(), ctrl.flip(), gpui::SharedString::from(id));
            return BCtrlPart::CheckBox
                .flex(ctx)
                .div(ctx)
                .id(id.clone())
                .debug_selector(move || id.to_string())
                .cursor_pointer()
                .on_click(move |_e: &gpui::ClickEvent, w: &mut gpui::Window, cx: &mut gpui::App| (std::rc::Rc::clone(&handler))(&action, w, cx))
                .child(super::mark::view(BMark::Check(on), ctx))
                .into_any_element();
        };
        let item = BItem::read(id, label.clone(), BItemValue::None).led(BLead::Mark(BMark::Check(on))).acting(ctrl.flip());
        super::item::view_in(&item, super::item::BItemFrame::Property, ctx)
    }

    /// A switch (`UISwitch`): the one row, its label taking the slack and the
    /// track at its end (`switch_track`), the whole row flipping it, so a
    /// column of these lines up.
    fn switch(&self, ctx: &BRenderCtx) -> AnyElement {
        let ctrl = self.0;
        let on = matches!(ctrl.value, BValue::Bool(true));
        let item = BItem::read(format!("switch-{}", ctrl.key.encode()), ctrl.label.clone().unwrap_or_default(), BItemValue::Switch(on)).acting(ctrl.flip());
        super::item::view_in(&item, super::item::BItemFrame::Property, ctx)
    }
}

/// A switch's track, on or off: the pill with its knob at the end it is on.
pub fn switch_track(on: bool, ctx: &BRenderCtx) -> AnyElement {
    BCtrlPart::Track(on).flex(ctx).div(ctx).child(BCtrlPart::Knob.flex(ctx).div(ctx)).into_any_element()
}


pub fn view(ctrl: &BCtrl, ctx: &BRenderCtx) -> AnyElement {
    BCtrlView(ctrl).render(ctx)
}

/// A swatch button's face: a square of the colour, bordered so a colour close to
/// the panel's own shade still reads as a swatch rather than as a hole.
pub fn swatch_face(color: crate::theme::BColor, ctx: &BRenderCtx) -> BFlex {
    // An icon's size, the way the zone glyph states it: there is no `BDim` for it.
    let size = BLen::Measured(ctx.style.dimensions.icon_size);
    BFlex::cell()
        .width(size)
        .height(size)
        .keeps_size()
        .radius(BRadius::Role(BStyleRole::Field))
        .border(BEdge::All)
        .fill(BFill::SwatchHsl(color))
}
