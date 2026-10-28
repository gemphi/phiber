//! Viewport navigation drag: orbit, and with Shift a pan, with Ctrl a zoom —
//! Blender's middle-mouse scheme (`scene::navigate`), on the middle button
//! and on the left, so a laptop without a middle button orbits too.
//!
//! gpui starts a drag on the left button only, so the middle button's press,
//! travel and release are heard here: the press on the canvas, the travel
//! window-wide — an unpainted overlay registers the listener each frame, as
//! `region::pan` does for a strip — so the view keeps turning when the
//! pointer leaves the viewport, as Blender's does.

use super::super::super::ctx::{BActionHandler, BRenderCtx};
use crate::scene::navigate::{BDragNav, BNavBase, drag_nav, drag_sets};
use crate::scene::view_slot::BViewSlot;
use crate::view::draw::flex::{BFlex, BPlace};
use crate::view::prim::BAction;
use gpui::prelude::*;
use gpui::{App, Context, DispatchPhase, Div, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels, Point, Render, Window, canvas, div};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy)]
struct OrbitDrag {
    /// Picked from the modifiers held as the drag began.
    nav: BDragNav,
    /// The view the drag started from.
    base: BNavBase,
    start: (f32, f32),
}

thread_local! {
    /// The middle-button drag under way, and the viewport it began on.
    static MIDDLE: Cell<Option<(Option<BViewSlot>, OrbitDrag)>> = const { Cell::new(None) };
}

impl OrbitDrag {
    /// The view keys for the pointer at `at` (measured as `start` was): the
    /// travel since the press, from the view the drag began on — an orbit
    /// turns on the turntable, a pan follows the pointer a pixel for a pixel,
    /// a zoom goes up-is-in (`drag_sets`). They go to the viewport the drag
    /// began on, wherever the pointer is now.
    fn travel(self, slot: Option<BViewSlot>, at: (f32, f32), handler: &BActionHandler, window: &mut Window, app: &mut App) {
        let total = (at.0 - self.start.0, at.1 - self.start.1);
        let sets = || drag_sets(self.nav, self.base, total, crate::scene::camera::last_size().0);
        let sets = match slot {
            Some(slot) => slot.with(sets),
            None => sets(),
        };
        for (key, value) in sets {
            handler(&BAction::Set(key, value), window, app);
        }
    }
}

struct BGhost;

impl Render for BGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

/// The navigation drag: an orbit, or with Shift held a pan, with Ctrl a zoom
/// (`scene::navigate::drag_nav`). `leave` is set while the canvas looks
/// through the camera: the actions that take the camera's view into the orbit
/// and turn the camera view off (`BCameraView::leave`), applied as the drag
/// starts — Blender's `ED_view3d_persp_switch_from_camera`, which a rotate,
/// a move and a zoom all do.
pub(super) struct BOrbit {
    pub leave: Option<Vec<BAction>>,
    /// The camera frame in canvas pixels, when looking through the camera
    /// (the render-region gesture's reference).
    pub frame: Option<crate::scene::camera_view::BCameraFrame>,
}

impl BOrbit {
    /// The gesture surface. A press that moves is the orbit; one that does
    /// not is a click, which `hit` turns into a pick (`hit.rs`) — gpui's
    /// drag threshold is what tells the two apart.
    pub fn render(&self, ctx: &BRenderCtx, hit: super::hit::BHit) -> gpui::Stateful<Div> {
        // G / R / S under way: the pointer drives the transform (`xform`).
        if let Some(modal) = crate::scene::xform::BTransformModal::now(ctx.state) {
            return super::xform::surface(ctx, hit, modal);
        }
        // B armed the box: the drag is the box, not an orbit (`boxsel`).
        if crate::scene::boxsel::armed(ctx.state) {
            return super::boxsel::surface(ctx, hit);
        }
        // Ctrl B armed the render region (`render_border`).
        if super::render_border::armed(ctx.state) {
            return super::render_border::surface(ctx, hit, self.frame);
        }
        if crate::scene::circlesel::BCircle::armed(ctx.state) {
            return super::circlesel::surface(ctx, hit);
        }
        // Leaving the camera view, the drag starts from the camera's view —
        // what `leave` sets — not the orbit's old one; and it turns about the
        // selection when Orbit Around Selection has one (`BNavBase::around`).
        let base = BNavBase::now(ctx.state).after(self.leave.as_deref().unwrap_or_default());
        let drag = Arc::new(Mutex::new(OrbitDrag { nav: BDragNav::Orbit, base, start: (0.0, 0.0) }));
        let slot = BViewSlot::current();
        let handler = ctx.handler.clone();
        let start_handler = ctx.handler.clone();
        let leave = self.leave.clone();
        // gpui hands the drag's start as an offset into the surface — the
        // canvas — so the pointer is measured from the canvas too.
        let canvas_bounds = hit.bounds.clone();
        let layer = super::hit::layer("view3d-orbit", gpui::CursorStyle::OpenHand, ctx);
        let layer = super::lasso::wire(layer, ctx.state, ctx.handler.clone(), hit.clone())
            .on_mouse_down(MouseButton::Right, hit.clone().on_cursor(ctx))
            .on_click(hit.on_click(ctx))
            .on_drag(
                drag.clone(),
                move |state, position: Point<Pixels>, window, cx| {
                    if let Ok(mut state) = state.lock() {
                        state.start = (f32::from(position.x), f32::from(position.y));
                        // The modifiers held as the drag begins pick the
                        // gesture, as Blender's keymap reads them on the press.
                        let m = window.modifiers();
                        state.nav = drag_nav(m.shift, m.control);
                    }
                    for action in leave.iter().flatten() {
                        (Rc::clone(&start_handler))(action, window, cx);
                    }
                    cx.new(|_| BGhost)
                },
            )
            .on_drag_move::<Arc<Mutex<OrbitDrag>>>(move |event, window, app| {
                let drag = *event.drag(app).lock().unwrap();
                let origin = canvas_bounds.get().map(|b| b.origin).unwrap_or_default();
                let at = window.mouse_position() - origin;
                drag.travel(slot, (f32::from(at.x), f32::from(at.y)), &handler, window, app);
            });
        middle(layer, slot, base, self.leave.clone(), ctx)
    }
}

/// The middle button's drag (`MIDDLEMOUSE` → `view3d.rotate`, with Shift
/// `view3d.move`, with Ctrl `view3d.zoom`): picked by the modifiers at the
/// press, and leaving the camera view at the press, as Blender's does
/// (`ED_view3d_persp_ensure`) — no click to tell apart from it.
fn middle(layer: gpui::Stateful<Div>, slot: Option<BViewSlot>, base: BNavBase, leave: Option<Vec<BAction>>, ctx: &BRenderCtx) -> gpui::Stateful<Div> {
    let (press, moved) = (ctx.handler.clone(), ctx.handler.clone());
    let listener = canvas(
        |_, _, _| (),
        move |_, (), window, _| {
            window.on_mouse_event(move |e: &MouseMoveEvent, phase, window, app| {
                let Some((at, drag)) = MIDDLE.get() else { return };
                if phase != DispatchPhase::Bubble || at != slot {
                    return;
                }
                // A release away from the canvas is seen as the next move
                // without the button.
                match e.pressed_button {
                    Some(MouseButton::Middle) => drag.travel(slot, (f32::from(e.position.x), f32::from(e.position.y)), &moved, window, app),
                    _ => MIDDLE.set(None),
                }
            });
        },
    );
    layer
        .on_mouse_down(MouseButton::Middle, move |e: &MouseDownEvent, window, app| {
            let start = (f32::from(e.position.x), f32::from(e.position.y));
            MIDDLE.set(Some((slot, OrbitDrag { nav: drag_nav(e.modifiers.shift, e.modifiers.control), base, start })));
            for action in leave.iter().flatten() {
                press(action, window, app);
            }
        })
        .on_mouse_up(MouseButton::Middle, |_, _, _| MIDDLE.set(None))
        .child(BFlex::column().place(BPlace::Overlay).styled(listener, ctx))
}
