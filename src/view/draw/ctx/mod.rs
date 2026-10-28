//! `BRenderCtx` — everything walkers need: resolved style, state, handlers,
//! enclosing host identity, and inherited rounded-surface corner ownership.

use crate::host::BHostType;
use crate::region::BRegionType;
use crate::space::BSpaceType;
use crate::state::BState;
use crate::theme::BResolvedUiStyle;
use crate::view::prim::BAction;
use crate::zone::BZoneType;
use gpui::{App, Window};
use std::rc::Rc;

// Preserve the existing context-module path while ownership logic lives in its
// focused, GPUI-independent surface module.
pub use super::surface::BCorners;

pub type BActionHandler = Rc<dyn Fn(&BAction, &mut Window, &mut App) + 'static>;

/// What a text field needs to wire real input: the window entity (which
/// implements `EntityInputHandler`) and the window's focus handle. Threaded
/// from `BPhiberApp::render` so the editing field can register the input
/// handler and draw its caret (`draw::text_input`).
pub type BInputWiring = (gpui::Entity<crate::app::BPhiberApp>, gpui::FocusHandle);

/// Area-resize drag callback: `(space, zone, boundary_index, dpx, zone_extent)`.
/// The app applies `area::size::apply_delta` and re-renders. This is a
/// dedicated seam because one drag writes two fraction keys atomically.
pub type BResizeHandler =
    Rc<dyn Fn(BSpaceType, BZoneType, usize, f32, f32, &mut Window, &mut App) + 'static>;

#[derive(Clone)]
pub struct BRenderCtx<'a> {
    /// The complete immutable style snapshot resolved for this render pass.
    pub style: &'a BResolvedUiStyle,
    pub state: &'a BState,
    pub handler: BActionHandler,
    pub resize: BResizeHandler,
    /// The host being drawn. Descendant walkers consult this for host-scoped
    /// colours and state paths.
    pub host: BHostType,
    /// The enclosing region kind, set by the region walker so descendant
    /// pane / part walkers can tell a header pane (content hugs, laid out
    /// Left / Center / Right) from a content-region pane (Center stretches to
    /// fill). `None` above the region level.
    pub region: Option<BRegionType>,
    /// Corners of the host surface this element is responsible for painting.
    pub corners: BCorners,
    /// True inside chrome that floats over content rather than docking
    /// against it — the viewport's toolbar and sidebar. Floating chrome
    /// connects to nothing, so its controls round on every side and carry a
    /// resting fill to stay legible over the canvas. Docked chrome does
    /// neither: its caps run into the pane they face, and a resting fill
    /// would turn the strip into a row of pills.
    pub floating: bool,
    /// True inside a chrome strip — a header, a tool/path bar or a footer.
    /// A region sets it for a host; a `BPanelRole` strip sets it for a modal,
    /// which composes panels rather than regions. Without it a modal's
    /// controls fell outside the strip treatment and rested on nothing while
    /// the identical control in a host rested on the strip fill.
    pub strip: bool,
    /// True in a slot that hands its whole width to one control — a property
    /// row's value column. A dropdown there is the column: it lines up with
    /// the label opposite it and with every other value in the stack, instead
    /// of hugging its own longest option.
    pub fill: bool,
    /// True in a source list's section (`BCardKind::Section`): a row there
    /// is an entry of a list, which takes a border on hover and when chosen
    /// instead of a fill (`chrome::paint_state`).
    pub listed: bool,
    /// The area being drawn, set by the area walker: what tells two areas
    /// holding the same host apart (a header's pan, `region::pan`).
    pub area: Option<crate::area::BAreaType>,
    /// The window entity + focus for wiring text input, when this pass was
    /// started with them (`BPhiberApp::render`). `None` in tests and anywhere
    /// the renderer is driven without an entity.
    pub input: Option<BInputWiring>,
    /// False inside a menu none of whose rows has a glyph: its rows reserve
    /// no icon slot, so the labels start at the panel's gutter instead of a
    /// blank tile's width in from it.
    pub glyphs: bool,
    /// True in a table's cell: a label there is one line, ending in an
    /// ellipsis where the column does, its whole text on a tooltip — a long
    /// file name does not run on, as Explorer's do not.
    pub clips: bool,
    /// True for the strip right under an area's editor tabs, set by the host
    /// walker (`host::view`): that region paints the active tab's ground
    /// (`BFill::TabGround`). Cleared again for the region's own children.
    pub joined: bool,
    /// True on the Welcome window: its cards are the brand's decked eggs
    /// (`thumb::BDeckPart`), the one place the shared thumbnail wears them.
    pub decked: bool,
}

impl<'a> BRenderCtx<'a> {
    pub fn new(
        style: &'a BResolvedUiStyle,
        state: &'a BState,
        handler: BActionHandler,
        resize: BResizeHandler,
    ) -> Self {
        Self {
            style,
            state,
            handler,
            resize,
            host: BHostType::Empty,
            region: None,
            corners: BCorners::NONE,
            strip: false,
            floating: false,
            fill: false,
            listed: false,
            area: None,
            input: None,
            glyphs: true,
            clips: false,
            joined: false,
            decked: false,
        }
    }

    /// Carry the text-input wiring for this pass (set once, at the root).
    pub fn with_input(mut self, wiring: Option<BInputWiring>) -> Self {
        self.input = wiring;
        self
    }

    /// Derive a context inside area `area`.
    pub fn in_area(&self, area: crate::area::BAreaType) -> Self {
        Self { area: Some(area), ..self.clone() }
    }

    /// The Welcome window's content: its cards are decked (`decked`).
    pub fn decked(&self) -> Self {
        Self { decked: true, ..self.clone() }
    }

    /// The strip under an area's editor tabs (`joined`).
    pub fn joined(&self) -> Self {
        Self { joined: true, ..self.clone() }
    }

    /// A table cell's context: its labels clip (`clips`).
    pub fn clipping(&self) -> Self {
        Self { clips: true, ..self.clone() }
    }

    /// Derive a context for a slot whose single control takes the full width.
    pub fn filling(&self) -> Self {
        Self { fill: true, ..self.clone() }
    }

    pub fn with_host(&self, host: BHostType) -> Self {
        Self { host, corners: BCorners::NONE, joined: false, ..self.clone() }
    }

    /// Derive a context tagged with the enclosing region kind. Set by the
    /// region walker before it walks its pane children.
    pub fn with_region(&self, region: BRegionType) -> Self {
        Self { region: Some(region), strip: self.strip || strip_region(region, self.host), joined: false, ..self.clone() }
    }

    /// A popover's body: it floats over the region its anchor stands in, so
    /// it belongs to none — a dropdown in a header's menu is a free control,
    /// not one of the header's caps (`BCapSide::of`).
    pub fn in_popover(&self) -> Self {
        let mut ctx = self.clone();
        ctx.region = None;
        ctx
    }

    /// Derive a context inside a `BPanelRole` chrome strip, for modal bodies
    /// that compose panels instead of regions.
    pub fn in_strip(&self) -> Self {
        Self { strip: true, ..self.clone() }
    }

    /// Derive a context for chrome that floats over content instead of
    /// docking against it.
    pub fn floating(&self) -> Self {
        Self { strip: true, floating: true, ..self.clone() }
    }

    /// A region's panels drawn inside floating chrome — the N panel beside
    /// the viewport's floating tab strip: docked panels, as Properties'
    /// are, not strip caps.
    pub fn content_of(&self, region: BRegionType) -> Self {
        Self { region: Some(region), corners: BCorners::NONE, strip: false, floating: false, ..self.clone() }
    }

    pub fn with_corners(&self, corners: BCorners) -> Self {
        Self { corners, joined: false, ..self.clone() }
    }
}

/// Regions whose contents follow the chrome-strip treatment.
fn strip_region(region: BRegionType, host: crate::host::BHostType) -> bool {
    match region {
        BRegionType::Tabs | BRegionType::Header | BRegionType::ToolHeader | BRegionType::Footer | BRegionType::Execute => true,
        // A tool rail is a strip; a tools column of panels (the file
        // browser's source list) is content.
        BRegionType::Tools => !host.tools_hold_panels(),
        _ => false,
    }
}

#[cfg(test)]
mod tests;
