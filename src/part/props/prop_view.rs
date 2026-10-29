//! Blender ref: `uiItemR` — every RNA prop renders as label + kind-specific
//! control. A property's kind (`BPropKind`) picks the control (`BCtrlKind`);
//! the control reads its value from `ctx.buts.<key>` as the value category
//! it edits and writes back through `BAction::Set`. So the render engine, a
//! sample count, a colour and a name are one mechanism: a control over a
//! typed key. Choosing Phiray is `Set(ctx.buts.engine, "PHIRAY")`.

use super::prop::BEnumItem;
use super::prop::BDefault;
use super::prop::BPropKind;
use super::prop::BPropSpec;
use crate::state::{BKey as _, BPath, BState, ButsKey, val::BValue};
use crate::view::BView;
use crate::view::prim::{BCtrl, BCtrlKind, BOption};





fn options(items: &'static [BEnumItem]) -> impl Iterator<Item = BOption> {
    items.iter().map(|i| BOption { key: i.key.to_string(), label: i.label.to_string(), icon: String::new() })
}

/// The control a property kind is edited with, and its factory value.
fn ctrl_for(kind: BPropKind) -> (BCtrlKind, BValue) {
    match kind {
        BPropKind::Bool => (BCtrlKind::Check, BValue::Bool(false)),
        BPropKind::Int { min, max } => (BCtrlKind::Int { min, max }, BValue::Int(min.max(0).min(max))),
        BPropKind::Float { .. } => (
            BCtrlKind::Number { min: f32::MIN, max: f32::MAX, step: 0.1 },
            BValue::Float(0.0),
        ),
        BPropKind::Enum(items) => (
            BCtrlKind::Select,
            BValue::Enum(items.first().map(|i| i.key).unwrap_or("").to_string()),
        ),
        BPropKind::Str | BPropKind::Pointer { .. } => (BCtrlKind::Text, BValue::Text(String::new())),
        BPropKind::Color { .. } => (BCtrlKind::Color, BValue::Color([0.8, 0.8, 0.8, 1.0])),
        // Vectors are one control per axis (below); this is the axis's kind.
        BPropKind::Vec3 { .. } | BPropKind::Vec4 { .. } => (
            BCtrlKind::Number { min: f32::MIN, max: f32::MAX, step: 0.1 },
            BValue::Float(0.0),
        ),
    }
}

/// The property's own RNA default, where it has one.
fn default_of(spec: &BPropSpec, kind: BPropKind) -> Option<BValue> {
    Some(match (spec.default?, kind) {
        (BDefault::Bool(v), _) => BValue::Bool(v),
        (BDefault::Int(v), BPropKind::Int { min, max }) => BValue::Int(v.clamp(min, max)),
        (BDefault::Int(v), _) => BValue::Int(v),
        (BDefault::Float(v), _) => BValue::Float(v),
        (BDefault::Enum(v), _) => BValue::Enum(v.to_string()),
    })
}

/// The components a vector property is drawn as, one control each; none
/// for a scalar.
fn axes(kind: BPropKind) -> &'static [&'static str] {
    match kind {
        BPropKind::Vec3 { .. } => &["x", "y", "z"],
        BPropKind::Vec4 { .. } => &["r", "g", "b", "a"],
        _ => &[],
    }
}

/// What a control shows while the state holds nothing for its key: the
/// open file's value for a Scene-tab row the file carries
/// (`scene::file_value` — the Rigid Body World, the gravity), else the
/// property's RNA default, else the kind's factory value.
fn fallback(spec: &BPropSpec, axis: Option<usize>, factory: BValue) -> BValue {
    super::scene::file_value(spec.key, axis).or_else(|| default_of(spec, spec.kind)).unwrap_or(factory)
}

/// One control over `key`, read from the state; `axis` is the component
/// of a vector property it edits.
fn control(spec: &BPropSpec, key: BPath, axis: Option<usize>, s: &BState) -> BCtrl {
    // The Render tab's Device: the card phiray draws on, not Cycles' CPU /
    // GPU switch read from the file (`pref::device`).
    if spec.key == "cycles.device" {
        return crate::part::pref::device::BRenderDevices::control(spec.open_key(), s);
    }
    let (kind, factory) = ctrl_for(spec.kind);
    let mut ctrl = BCtrl::read(kind, key, fallback(spec, axis, factory), s);
    match spec.kind {
        BPropKind::Enum(items) => {
            ctrl = ctrl.with_options(options(items)).with_menu(spec.open_key(), s);
        }
        BPropKind::Pointer { target } => ctrl = ctrl.with_label(target),
        _ => {}
    }
    ctrl
}

fn value(spec: &BPropSpec, s: &BState) -> BView {
    let axes = match axes(spec.kind) {
        // A path with the folder that browses for it: the Base Color's image.
        [] if spec.key == crate::scene::recolor::IMAGE_FIELD => return super::material::image_row(BView::ctrl(control(spec, spec.value_key(), None, s))),
        [] => return BView::ctrl(control(spec, spec.value_key(), None, s)),
        axes => axes,
    };
    // Blender's `uiItemR` on a vector: one field per axis, stacked, each
    // naming its axis inside the field ("X  0 m"). Side by side they were
    // three fields in half a panel's width, and a narrow panel (the N panel)
    // clipped them to the first — Location showed one number.
    BView::column(
        axes.iter()
            .enumerate()
            .map(|(k, axis)| BView::ctrl(control(spec, ButsKey::Axis(spec.key, axis).path(), Some(k), s).with_label(axis.to_uppercase())))
            .collect(),
    )
}

pub fn prop_row(spec: &BPropSpec, s: &BState) -> BView {
    // A property row is a Panel with role `Section` so the renderer splits it
    // 50/50 (label left, control right) — Blender's `uiItemR` split factor.
    BView::split(vec![BView::label(spec.label), value(spec, s)])
}

/// A property drawn in a panel's header (`draw_header`, `text=""`): its
/// control alone, no label — Motion Blur's checkbox before the title.
pub fn header_ctrl(spec: &BPropSpec, s: &BState) -> BView {
    value(spec, s)
}

/// The value a property's row shows, read the way the row reads it: the
/// state's, else `fallback`. `axis` picks a vector's component. What a
/// test asks when it wants to know what the panel says rather than what
/// the state holds.
pub fn shown(spec: &BPropSpec, axis: Option<usize>, s: &BState) -> BValue {
    let key = match axis.and_then(|k| axes(spec.kind).get(k).copied()) {
        Some(name) => ButsKey::Axis(spec.key, name).path(),
        None => spec.value_key(),
    };
    control(spec, key, axis, s).value
}

impl BPropSpec {
    /// State key that stores the value of `self`. Segments live under `buts` so
    /// all Property-panel values share one root and can be introspected
    /// uniformly.
    ///
    /// Frame Start and End are the exceptions: they are the scene's frame
    /// range, which the Timeline's header and Render Animation already keep
    /// at `ctx.frame_start` / `ctx.frame_end`, and a second copy under
    /// `buts` would be a panel that disagrees with the header beside it.
    pub fn value_key(&self) -> BPath {
        use crate::state::{BKey as _, CtxKey};
        match self.key {
            "frame_start" => CtxKey::FrameStart.path(),
            "frame_end" => CtxKey::FrameEnd.path(),
            key => ButsKey::Value(key).path(),
        }
    }

    /// State key gating whether the dropdown for `self` is open.
    fn open_key(&self) -> BPath {
        ButsKey::Dropdown(self.key).path()
    }
}
