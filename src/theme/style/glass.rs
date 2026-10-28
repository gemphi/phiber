//! The glass mode: Apple's Liquid Glass, read for a desktop window.
//!
//! Apple's material is a layer for controls and navigation that floats over
//! the content. It blurs and bends what is behind it, catches a specular
//! highlight along its rim, glows from inside when touched, and turns
//! frosted when Reduce Transparency is on. Glass here is a whole theme mode
//! (`BThemeMode::Glass`): every palette read from its dark ladder, with the
//! surfaces glazed by the one material below. Nothing is glazed per host.
//!
//! What is real and what is not:
//!
//! * **Blur** is the desktop behind the window, blurred by the system's
//!   compositor (`WindowBackgroundAppearance::Blurred`: acrylic on Windows,
//!   a visual-effect view on macOS, KWin blur on Wayland). The window's
//!   ground is translucent so that blur shows through every surface.
//! * **Inside the window nothing is blurred.** gpui has no backdrop filter
//!   and no way to sample what it drew underneath, so chrome floating over
//!   the canvas is translucent without being frosted. Apple's Clear variant
//!   (see-through, dimmed underneath) is the closest match. The canvas is
//!   content and stays opaque (`viewport_surface`).
//! * **Rim, glow and halo** are inset and drop shadows on a tile
//!   (`view/draw/flex/paint::glazed`). Light can't bend at the edges
//!   without a backdrop to bend, so the rim highlight stands in for the
//!   lensing.
//!
//! Text stays solid. The ground dims what is behind it before any pane does,
//! as Apple dims behind Clear glass, so text keeps its contrast over a white
//! desktop (`text_reads_over_a_white_desktop`). Reduce Transparency
//! (`AppKey::ReduceTransparency`) makes every surface solid, takes the window
//! off the blur and keeps the rims.

use super::{BEffects, BResolvedColors, BResolvedUiStyle, BShadowStyle};
use crate::state::{AppKey, BState, val::V_TRUE};
use crate::theme::BColor;

/// The material: how much of each layer shows what is behind it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BGlass {
    /// The window's ground under every area. It also dims the desktop.
    pub ground: f32,
    /// The top bar, the status bar and the activity rail.
    pub chrome: f32,
    /// An editor's panes, headers and cards.
    pub pane: f32,
    /// A control's fill on a pane: a fill on glass, never glass on glass.
    pub control: f32,
    /// Chrome floating over the canvas: the toolbar, the sidebar, gizmos.
    pub floating: f32,
    /// An icon tile at rest: white over the pane at this alpha.
    pub tile: f32,
    /// A lit tile (a mark that is on): the accent at this alpha. Glass
    /// tinted for the one thing that is chosen.
    pub tint: f32,
    /// The rim's highlight: a hairline of white inside the edge.
    pub rim: f32,
    /// The glow from inside the tile, and how far it spreads in px.
    pub glow: f32,
    pub glow_blur: f32,
    /// The halo the tile casts: what makes its icon glow faintly.
    pub halo: f32,
    pub halo_blur: f32,
}

/// Liquid Glass as phiber wears it.
pub const LIQUID_GLASS: BGlass = BGlass {
    ground: 0.72,
    chrome: 0.80,
    pane: 0.86,
    control: 0.72,
    floating: 0.80,
    tile: 0.14,
    tint: 0.78,
    rim: 0.42,
    glow: 0.20,
    glow_blur: 6.0,
    halo: 0.12,
    halo_blur: 8.0,
};

impl BGlass {
    /// This glass with Reduce Transparency on: every surface solid and the
    /// floating chrome back to the theme's own weight. The rims stay, since
    /// they add an edge rather than take contrast away (Increase Contrast
    /// adds a border for the same reason).
    pub const fn solid(self, floating: f32) -> Self {
        Self { ground: 1.0, chrome: 1.0, pane: 1.0, control: 1.0, floating, tile: 0.2, tint: 1.0, ..self }
    }

    /// Whether the window should be blurred behind: it is when the ground
    /// lets anything through.
    pub fn blurs(self) -> bool {
        self.ground < 1.0
    }

    /// A theme's effects in glass. A surface that floats has depth, so it
    /// casts at least Large.
    pub const fn wear(self, base: BEffects) -> BEffects {
        BEffects { glass: Some(self), surface_shadow: base.surface_shadow.at_least(BShadowStyle::Large), ..base }
    }

    /// The colours glazed. Each surface slot keeps its shade and takes its
    /// layer's alpha. The canvas, text, the accent and the meanings are
    /// untouched.
    pub fn glaze(self, c: &mut BResolvedColors) {
        for (slot, alpha) in [
            (&mut c.bg_app, self.ground),
            (&mut c.sep, self.ground),
            (&mut c.bg_activity, self.chrome),
            (&mut c.bg_top, self.chrome),
            (&mut c.bg_bottom, self.chrome),
            (&mut c.bg_left, self.pane),
            (&mut c.bg_right, self.pane),
            (&mut c.bg_header, self.pane),
            (&mut c.bg_panel, self.pane),
            (&mut c.bg_control, self.control),
            (&mut c.bg_hover, self.control),
            (&mut c.bg_active, self.control),
            (&mut c.row_alt, self.control),
        ] {
            slot.3 = alpha;
        }
    }

    /// An icon tile at rest on `ground`: frosted white, its hue the
    /// ground's.
    pub fn tile_on(self, ground: BColor) -> BColor {
        (ground.0, ground.1, 96.0, self.tile)
    }

    /// The lit tile: the accent, tinted glass.
    pub fn tinted(self, accent: BColor) -> BColor {
        (accent.0, accent.1, accent.2, accent.3 * self.tint)
    }
}

/// Whether Reduce Transparency is on (Preferences ▸ Themes ▸ Glass).
pub fn reduced(state: &BState) -> bool {
    state.get(AppKey::ReduceTransparency) == Some(V_TRUE)
}

/// Whether the window wears the blurred backdrop: Glass is in force and
/// Reduce Transparency is off.
pub fn backdrop(state: &BState) -> bool {
    let (_, mode) = crate::theme::theme_in_force(state);
    mode == crate::theme::BThemeMode::Glass && !reduced(state)
}

impl BResolvedUiStyle {
    /// This style glazed, when its mode is glass, and solid again when
    /// `reduced`. The one place a glass theme becomes translucent.
    pub fn glazed(mut self, reduced: bool) -> Self {
        let Some(glass) = self.effects.glass else { return self };
        let glass = match reduced {
            true => glass.solid(self.effects.floating_alpha),
            false => glass,
        };
        glass.glaze(&mut self.colors);
        self.effects.glass = Some(glass);
        self.effects.floating_alpha = glass.floating;
        self
    }
}

#[cfg(test)]
#[path = "glass_tests.rs"]
mod tests;
