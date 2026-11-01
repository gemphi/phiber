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
//! * **Inside the window it is real too.** The windows are backdrops
//!   (`gpui::BackdropStyle`, appkit's glass stages on the GPU): every pane's
//!   card and the chrome floating over the canvas blur, tint and light what is
//!   under them (`view/draw/flex/paint::frosted`). The 3D view runs behind them
//!   (`view3d::stage`), so there is something to see.
//! * **Only the windows are glass.** A button, a tile and an icon are what
//!   they are in the dark mode, in the theme's own radii and colours, so glass
//!   is the normal look with its windows made of glass. No surface gains
//!   colour, either: what shows through brings its own.
//!
//! Text stays solid. The ground dims what is behind it before any pane does,
//! as Apple dims behind Clear glass, so text keeps its contrast over a white
//! desktop (`text_reads_over_a_white_desktop`). Reduce Transparency
//! (`AppKey::ReduceTransparency`) makes every surface solid, takes the window
//! off the blur and keeps the rims.

use super::{BEffects, BResolvedColors, BResolvedUiStyle, BShadowStyle};
use crate::state::{AppKey, BState, val::V_TRUE};
use crate::theme::BColor;

/// The most the frost thickens over what is brightest (`glass.hlsl`).
pub const THICKEST: f32 = 0.88;

/// How one kind of glass treats what is behind it: appkit's
/// `NSGlassEffectView.Style`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BFrost {
    /// How far the blur reaches, in px.
    pub blur: f32,
    /// The lift on what shows through.
    pub saturation: f32,
    /// How far what shows through moves toward the ground's colour at the
    /// least. Over what is bright the shader thickens it (`glass.hlsl`), up to
    /// `THICKEST`, so text holds without dimming a dark scene.
    pub tint: f32,
    /// How far the rim pulls its sample inward, in px.
    pub lens: f32,
    /// How much light the rim catches.
    pub edge: f32,
}

/// The material: how much of each layer shows what is behind it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BGlass {
    /// The window's ground under every area. It also dims the desktop.
    pub ground: f32,
    /// The top bar, the status bar and the activity rail.
    pub chrome: f32,
    /// An editor's panes, headers and cards.
    pub pane: f32,
    /// A control's fill: opaque, since controls are not glass.
    pub control: f32,
    /// Chrome floating over the canvas: the toolbar, the sidebar, gizmos.
    pub floating: f32,
    /// What a window does to what is behind it.
    pub frost: BFrost,
}

/// Liquid Glass as phiber wears it.
pub const LIQUID_GLASS: BGlass = BGlass {
    ground: 0.72,
    chrome: 0.30,
    pane: 0.30,
    control: 1.0,
    // The frost carries the rest of the contrast (`frost.tint`).
    floating: 0.30,
    frost: BFrost { blur: 24.0, saturation: 1.2, tint: 0.35, lens: 26.0, edge: 0.35 },
};

impl BGlass {
    /// This glass with Reduce Transparency on: every surface solid and the
    /// floating chrome back to the theme's own weight.
    pub const fn solid(self, floating: f32) -> Self {
        Self { ground: 1.0, chrome: 1.0, pane: 1.0, control: 1.0, floating, ..self }
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

    /// The colours glazed: Apple's glass has no colour of its own, so the
    /// ladder goes grey, and what shows through brings the colour. Each surface
    /// keeps its step and takes its layer's alpha. Text, the accent and the
    /// meanings are untouched.
    pub fn glaze(self, c: &mut BResolvedColors) {
        let hued = c.ladder;
        c.ladder = hued.neutral();
        let grey = c.ladder;
        let viewport = c.bg_viewport.3;
        for (slot, alpha) in [
            (&mut c.bg_viewport, viewport),
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
            (&mut c.row_alt, self.pane),
        ] {
            *slot = match hued.step_of(*slot) {
                Some(step) => grey.at(step, alpha),
                None => (slot.0, 0.0, slot.2, alpha),
            };
        }
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
    /// The light a window's rim catches: the ladder's lightest shade, so it is
    /// the theme's own white.
    pub fn glass_light(&self) -> BColor {
        self.colors.ladder.color(crate::theme::palette::set::SHADE_TEXT_HI)
    }

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
        // On glass, half a control's shade would fade into the window.
        self.effects.control_rest_alpha = 1.0;
        self
    }
}

#[cfg(test)]
#[path = "glass_tests.rs"]
mod tests;
