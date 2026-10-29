//! The canvas elements: the links' `canvas()` under the cards and the
//! sockets' over them.
//!
//! Split out of `set.rs`, which was over the 240-line read budget
//! (`level_structure`). It moves cleanly because it writes no GPUI styling at
//! all — it is closures over the frame's data and calls to `paint`, so the
//! style budget for both files is unchanged by the move. That was the test
//! of whether the split was honest: moving a div to a new file only hides its
//! styling, and the budget refuses it.
//!
//! Two canvases, because the cards are divs between them: a GPUI element
//! paints over the ones before it, so whatever one canvas paints is either all
//! under the cards or all over them. Links belong under (Blender draws them
//! before its nodes); a socket's dot, name and value belong over, as part of
//! the node. One canvas under the cards put every socket name inside a card
//! under its opaque body. The links' canvas plans the cards, the sockets'
//! canvas, painted later in the same frame, draws them from that plan.
//!
//! What it owns is the one place pixels exist. Everything upstream works in view
//! units or fractions of the region, because the element tree is built before
//! the canvas is measured; this closure runs after, so it is where the region's
//! size is first known — which is why it both plans the cards and reports the
//! size the next frame will place against (`view/store`).

use super::super::super::super::ctx::BRenderCtx;
use super::super::plan::{BCardSize, plan};
use super::super::view::{BNodeView, store as view_store};
use super::super::{link, socket};
use super::{BNodeCard, BNodeInk, paint_links, paint_sockets};
use crate::phiphx::blend::shade::graph::{BlendNodeGraph, BlendNodeLink};
use gpui::{Bounds, Pixels, canvas};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// What one frame needs to paint, gathered before the closure takes it.
pub struct BNodePainting {
    pub graph: BlendNodeGraph,
    /// The links as drawn — the file's, with this host's edits applied.
    pub wired: Vec<BlendNodeLink>,
    /// Each node's drawn position, as fractions of the canvas.
    pub placed: Vec<(String, (f32, f32))>,
    /// A link being carried, drawn from its anchor to the cursor.
    pub carrying: Option<link::BLinking>,
    pub view: BNodeView,
    pub size: BCardSize,
    pub ink: BNodeInk,
}

/// The two canvases and the cell the first one's paint pass publishes its
/// bounds into.
pub struct BNodeCanvases {
    /// The links, painted before the cards; it also plans them.
    pub links: gpui::Canvas<()>,
    /// The sockets — dots, names, values — painted after the cards.
    pub sockets: gpui::Canvas<()>,
    /// The bounds reach the gesture handlers through this cell because a mouse
    /// event carries window coordinates and the canvas's own rectangle is not
    /// known until it paints — the same arrangement the animation window uses
    /// (`canvas/time/gesture.rs`).
    pub bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}

/// The links' canvas, which plans the cards, and the sockets' canvas, which
/// paints them over the card divs laid between the two.
pub fn element(painting: BNodePainting, ctx: &BRenderCtx) -> BNodeCanvases {
    let bounds_cell: Rc<Cell<Option<Bounds<Pixels>>>> = Rc::new(Cell::new(None));
    let publish = Rc::clone(&bounds_cell);
    let planned: Rc<RefCell<Vec<BNodeCard>>> = Rc::new(RefCell::new(Vec::new()));
    let (plans, reads) = (Rc::clone(&planned), planned);
    let ink = painting.ink;
    let host = ctx.host;
    let element = canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            publish.set(Some(bounds));
            let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
            // The size the next frame places its cards against — the view has
            // to be shaped like the canvas, and this is where the canvas is
            // finally known (`view/store.rs`).
            if view_store::measured(host, (w, h)) {
                window.request_animation_frame();
            }
            let (cards, mut links) = plan(
                &painting.graph,
                &painting.wired,
                &painting.placed,
                w,
                h,
                painting.size,
                &painting.view,
            );
            // The carried link, from its anchored socket to the cursor.
            if let Some(drag) = &painting.carrying {
                let anchor = cards
                    .iter()
                    .find(|card| card.name == drag.anchor.node)
                    .and_then(|card| {
                        socket::row_of_card(card, &drag.anchor.socket).map(|row| card.socket_at(&row))
                    });
                if let Some(anchor) = anchor {
                    links.push(drag.preview((drag.at.0 * w, drag.at.1 * h), anchor));
                }
            }
            paint_links(&links, &painting.ink, bounds, window);
            *plans.borrow_mut() = cards;
        },
    );
    let sockets = canvas(|_, _, _| (), move |bounds, (), window, cx| paint_sockets(&reads.borrow(), &ink, bounds, window, cx));
    BNodeCanvases { links: element, sockets, bounds: bounds_cell }
}
