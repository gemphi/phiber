//! The stage: in glass, the workspace's 3D view painted behind everything.
//!
//! Glass is only glass with something behind it. Panes sit beside the viewport,
//! so left to themselves they would blur flat colour. Here the centre zone's
//! view hands its paint to a layer at the workspace root, which runs first and
//! at the workspace's full size; the panes then frost the scene as they draw,
//! and the viewport's own element is a hole onto it. Its paint and its picks
//! share the bounds the stage gives it, so a click still lands on what is shown.

use crate::host::BHostType;
use crate::view::draw::ctx::BRenderCtx;
use crate::view::draw::flex::{BFlex, BPlace};
use gpui::prelude::*;
use gpui::{AnyElement, Bounds, Pixels, Window, canvas};
use std::cell::{Cell, RefCell};

/// A view's whole paint, ready to run over the bounds it is given.
pub type BStagePaint = Box<dyn FnOnce(Bounds<Pixels>, &mut Window)>;

thread_local! {
    /// True while the zone whose 3D view may take the stage is being walked.
    static OPEN: Cell<bool> = const { Cell::new(false) };
    static PAINT: RefCell<Option<BStagePaint>> = const { RefCell::new(None) };
}

/// Whether this frame has a stage: glass that lets anything through.
pub fn wanted(ctx: &BRenderCtx) -> bool {
    ctx.style.effects.glass.is_some_and(|glass| glass.blurs())
}

/// Run `walk` with the stage open for the first 3D view inside it.
pub fn open_for<R>(walk: impl FnOnce() -> R) -> R {
    OPEN.set(true);
    let out = walk();
    OPEN.set(false);
    out
}

/// Whether this host's area will be a hole onto the stage: asked by its card,
/// before the view inside has claimed it.
pub fn holds(ctx: &BRenderCtx) -> bool {
    ctx.host == BHostType::View3D && OPEN.get()
}

/// Take the stage, if it is open. Only the first view asking gets it.
pub fn claim() -> bool {
    OPEN.replace(false)
}

/// What the stage paints this frame.
pub fn hand(paint: BStagePaint) {
    PAINT.set(Some(paint));
}

/// The layer itself: the workspace's first child, so it paints under the rest.
pub fn element(ctx: &BRenderCtx) -> AnyElement {
    PAINT.set(None);
    let layer = canvas(
        |_, _, _| {},
        |bounds, _, window, _| {
            if let Some(paint) = PAINT.take() {
                paint(bounds, window);
            }
        },
    );
    BFlex::column().place(BPlace::Overlay).styled(layer, ctx).into_any_element()
}
