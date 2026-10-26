//! Part walker — a leaf composer. Its body is the only place primitives
//! may appear.

use gpui::AnyElement;
use crate::view::BView;
use super::ctx::BRenderCtx;
use super::walk;

pub fn view(body: &BView, ctx: &BRenderCtx) -> AnyElement {
    walk(body, ctx)
}
