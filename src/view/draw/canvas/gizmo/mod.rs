//! View3D navigation gizmo preserving existing yaw/zoom actions.

pub mod compass;

use super::super::color::to_hsla;
use super::super::ctx::BRenderCtx;
use crate::scene::view_slot::BViewSlot;
use crate::scene::{view_zoom, zoom_step};
use crate::state::{BKey as _, View3dKey};
use crate::theme::BStyleRole;
use crate::view::prim::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, MouseButton, SharedString, div, px, svg};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

// Which arrow is being held, if any: the view's, not an element's.
//
// A `Cell` built inside the element lives exactly one frame, and every
// turn of the view repaints the gizmo, so the press armed frame N's cell
// while the release cleared frame N+1's and the spin loop watched a cell
// nobody could clear. The view orbited on after the button came up, until
// the window happened to lose focus. One pointer holds one arrow, so the
// gizmo needs one slot, and it has to outlive the repaint that the
// spinning itself causes. gpui's UI runs on one thread.
thread_local! {
    static SPINNING: Cell<Option<&'static str>> = const { Cell::new(None) };
}

/// How long an orbit arrow is held before it starts turning by itself, and
/// the pace it turns at then (a frame at 60 Hz).
const SPIN_DELAY: Duration = Duration::from_millis(300);
const SPIN_FRAME: Duration = Duration::from_millis(16);

pub struct BGizmo;

impl BGizmo {
    /// A gizmo button: the square with its icon, before what it does.
    fn button(&self, name: &'static str, ctx: &BRenderCtx) -> gpui::Stateful<gpui::Div> {
        let selector = format!("gizmo-{name}");
        div()
            .id(SharedString::from(format!("gizmo-{name}")))
            .debug_selector(move || selector)
            .flex()
            .items_center()
            .justify_center()
            .w(px(ctx.style.dimensions.gizmo_button_size))
            .h(px(ctx.style.dimensions.gizmo_button_size))
            .rounded(px(ctx.style.radius_for(BStyleRole::Button)))
            .cursor_pointer()
            .hover(|style| style.bg(to_hsla(ctx.style.host_item_hover(ctx.host))))
            .child(
                svg()
                    .path(format!("icons/{name}.svg"))
                    .size(px(ctx.style.dimensions.gizmo_icon_size))
                    .flex_shrink_0()
                    .text_color(to_hsla(ctx.style.colors.text)),
            )
    }

    fn control(
        &self,
        name: &'static str,
        tip: &'static str,
        action: BAction,
        ctx: &BRenderCtx,
    ) -> gpui::Stateful<gpui::Div> {
        let handler = ctx.handler.clone();
        self.button(name, ctx).on_click(move |_e: &ClickEvent, w, cx| {
            let _ = tip;
            (Rc::clone(&handler))(&action, w, cx);
        })
    }

    /// An orbit arrow: the press turns the view a step (`view3d.view_orbit`,
    /// as Numpad 4/6/8/2 do), and holding it keeps the view turning a small
    /// step a frame (`view_op::ORBIT_SPIN`) until the button comes up, so a
    /// model can be walked round without a click every 15°.
    fn held(&self, name: &'static str, way: &'static str, ctx: &BRenderCtx) -> gpui::Stateful<gpui::Div> {
        let op = move |what: &str| BAction::Set(crate::state::OpKey::View3d.path(), format!("{what}_{way}"));
        let handler = ctx.handler.clone();
        self.button(name, ctx)
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                SPINNING.with(|s| s.set(Some(way)));
                handler(&op("orbit"), window, cx);
                let handler = Rc::clone(&handler);
                window
                    .spawn(cx, async move |cx| {
                        cx.background_executor().timer(SPIN_DELAY).await;
                        // The window captures the pointer on a press, so the
                        // release reaches `on_mouse_up_out` wherever it lands;
                        // a switch to another window stops the turn as well.
                        while SPINNING.with(|s| s.get()) == Some(way) {
                            let turned = cx.update(|window, cx| {
                                let on = window.is_window_active();
                                on.then(|| handler(&op("spin"), window, cx)).is_some()
                            });
                            match turned.unwrap_or(false) {
                                true => {}
                                false => SPINNING.with(|s| s.set(None)),
                            }
                            cx.background_executor().timer(SPIN_FRAME).await;
                        }
                    })
                    .detach();
            })
            .on_mouse_up(MouseButton::Left, move |_, _, _| SPINNING.with(|s| s.set(None)))
            .on_mouse_up_out(MouseButton::Left, move |_, _, _| SPINNING.with(|s| s.set(None)))
    }

    fn zoom_action(&self, ctx: &BRenderCtx, factor: f32) -> BAction {
        BAction::Set(BViewSlot::key(View3dKey::Zoom), format!("{}", zoom_step(view_zoom(ctx.state), factor)))
    }

    fn surface(&self, ctx: &BRenderCtx) -> gpui::Div {
        let mut border = ctx.style.colors.text;
        border.3 = ctx.style.effects.viewport_grid_alpha;
        let surface = div()
            .border(px(ctx.style.strokes.border))
            .border_color(to_hsla(border))
            // The gizmos are floating chrome, so they take the same fill as
            // the toolbar and sidebar beside them. The modal dim they used
            // was far darker than anything else over the canvas.
            .bg(crate::view::draw::pane::floating_bg(ctx))
            // The canvas under a gizmo takes a click as a pick, and a pick
            // of nothing deselects: an arrow or a zoom press also emptied
            // the selection ("Deselected all") until the gizmo kept its
            // presses to itself.
            .occlude();
        // A gizmo sits on the scene, so it casts on it — a rung under a
        // dropdown, as `BElevation::Floating` says.
        crate::view::draw::flex::shadowed(
            surface.rounded(px(ctx.style.radius_for(BStyleRole::Card))),
            ctx.style.shadow_for(crate::theme::BElevation::Floating),
        )
    }

    fn pill(&self, ctx: &BRenderCtx, list: Vec<gpui::Stateful<gpui::Div>>) -> gpui::Div {
        let mut column = self
            .surface(ctx)
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(ctx.style.spacing.gap_xs / 2.0))
            .px(px(ctx.style.spacing.pad_xs / 2.0))
            .py(px(ctx.style.spacing.pad_xs * 0.75));
        for child in list {
            column = column.child(child);
        }
        column
    }

    fn orbit(&self, ctx: &BRenderCtx) -> gpui::Div {
        // Blender's navigate gizmo has four orbit arrows around the compass
        // (up / left / right / down). Lay them as a 2-column grid so the
        // vertical pair flanks the compass and the horizontal pair is below.
        self.surface(ctx)
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap(px(ctx.style.spacing.gap_xs / 2.0))
            .px(px(ctx.style.spacing.pad_xs * 0.75))
            .py(px(ctx.style.spacing.pad_xs / 2.0))
            .child(self.held("arrow_left", "left", ctx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(ctx.style.spacing.gap_xs / 2.0))
                    .child(self.held("arrow_up", "up", ctx))
                    .child(self.held("arrow_down", "down", ctx)),
            )
            .child(self.held("arrow_right", "right", ctx))
    }

    /// The navigate sphere. It lives in the corner the user chose, because it
    /// is the thing they aim at.
    pub fn compass(&self, ctx: &BRenderCtx) -> AnyElement {
        self::compass::BCompass.render(ctx)
    }

    /// The square clusters — orbit arrows and the zoom / view pill. They sit
    /// at the other end of the same side, so the sphere is not buried under a
    /// tall column of boxes.
    pub fn controls(&self, ctx: &BRenderCtx, camera: &crate::scene::camera_view::BCameraSlot) -> AnyElement {
        let persp = crate::scene::is_perspective(ctx.state);
        let mut pill = Vec::with_capacity(6);
        pill.push(self.control("zoom_in", "Zoom In", self.zoom_action(ctx, 1.2), ctx));
        pill.push(self.control("zoom_out", "Zoom Out", self.zoom_action(ctx, 1.0 / 1.2), ctx));
        // Recenter pan: click "grab" resets horizontal pan to 0. Vertical
        // follows Frame All. Data-driven single-click semantics.
        pill.push(self.control("grab", "Recenter View", BAction::Reset(BViewSlot::key(View3dKey::PanX)), ctx));
        // Frame All: reset zoom to 1.0. Kept as a single-key reset so the
        // button dispatches a single BAction.
        pill.push(self.control("home", "Frame All", BAction::Set(BViewSlot::key(View3dKey::Zoom), "1".to_string()), ctx));
        // The camera view, in and out (`VIEW3D_OT_view_camera`, Numpad 0),
        // for the area this gizmo sits in — only where there is one.
        if let Some(toggle) = camera.toggle() {
            pill.push(self.control(
                "camera",
                if camera.through { "Leave Camera View" } else { "View Camera" },
                toggle,
                ctx,
            ));
        }
        // Perspective / Orthographic toggle — matches Blender's numpad-5 shortcut.
        pill.push(self.control(
            if persp { "perspective" } else { "orthographic" },
            if persp { "Switch to Orthographic" } else { "Switch to Perspective" },
            BAction::Set(BViewSlot::key(View3dKey::Perspective), if persp { "false".to_string() } else { "true".to_string() }),
            ctx,
        ));
        div()
            .flex()
            .flex_col()
            // The groups share a centre line, so each keeps the same margin
            // from the viewport edge. Aligning their right edges made a round
            // group look inset next to a square one.
            .items_center()
            .gap(px(ctx.style.spacing.gap_sm))
            .child(self.orbit(ctx))
            .child(self.pill(ctx, pill))
            .into_any_element()
    }
}
