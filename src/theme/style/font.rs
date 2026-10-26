//! The interface's typeface — chosen in Preferences (*Typography ▸ Family*,
//! *Weight*) and applied to every letter the window draws.
//!
//! The choices are every family in `assets/phi-fonts` (found at start-up by
//! `bundle.rs`, so a folder dropped in there is a new choice) followed by
//! the system's own face (Segoe UI on Windows, Helvetica Neue on a Mac), the
//! two macOS faces Blender users know from its menus, Inter, and the four
//! monospace faces VS Code ships with or recommends. A face the machine does
//! not have falls back down its own stack, then to the system's
//! (`Font.fallbacks`), so a choice never leaves the window blank.
//!
//! The default is Phi-Soft Condensed, the platform's own face, set Light:
//! its Regular reads heavy at interface sizes.

use super::bundle;

/// The family name Phi-Soft Condensed answers to (its typographic family,
/// name ID 16, shared by every weight).
pub const PHI_SOFT_FAMILY: &str = "Phi-Soft Condensed";

/// Phi-Soft Condensed (1.100) Light, compiled into the binary: the face the
/// browser build draws in (it has no folder to read) and the one a desktop
/// build falls back to when `assets/phi-fonts` is missing.
pub const PHI_SOFT_FACES: [&[u8]; 1] = [include_bytes!("../../../assets/phi-fonts/Phi-Soft/PhiSoftCondensed-Light.otf")];

/// One family Preferences offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BFontFamily {
    /// A family from `assets/phi-fonts`, by its slug (`bundle.rs`).
    Bundled(&'static str),
    /// The platform's interface face: Segoe UI, Helvetica Neue, or whatever
    /// the system names as `sans-serif`.
    System,
    /// macOS's interface face before San Francisco; VS Code's Mac fallback.
    HelveticaNeue,
    /// Windows' interface face.
    SegoeUi,
    /// The open interface face, sized for screens.
    Inter,
    /// macOS's terminal face; VS Code's default on a Mac.
    Menlo,
    /// Windows' code face; VS Code's default there.
    Consolas,
    /// Windows Terminal's face; VS Code's recommended ligature face.
    CascadiaCode,
    /// JetBrains' code face, VS Code's most-installed third-party one.
    JetBrainsMono,
}

impl BFontFamily {
    /// The faces the machine supplies, in the order Preferences lists them
    /// after the bundled ones.
    pub const SYSTEM: [BFontFamily; 8] = [
        BFontFamily::System,
        BFontFamily::HelveticaNeue,
        BFontFamily::SegoeUi,
        BFontFamily::Inter,
        BFontFamily::Menlo,
        BFontFamily::Consolas,
        BFontFamily::CascadiaCode,
        BFontFamily::JetBrainsMono,
    ];
    pub const DEFAULT: BFontFamily = BFontFamily::Bundled("phi_soft_condensed");

    /// Every choice, in the order Preferences lists them: the bundled
    /// families, then the system's.
    pub fn all() -> Vec<BFontFamily> {
        bundle::bundled().iter().map(|family| BFontFamily::Bundled(family.slug)).chain(Self::SYSTEM).collect()
    }

    pub fn label(self) -> &'static str {
        match self {
            BFontFamily::Bundled(slug) => bundle::find(slug).map_or(PHI_SOFT_FAMILY, |family| family.name),
            BFontFamily::System => "System",
            BFontFamily::HelveticaNeue => "Helvetica Neue",
            BFontFamily::SegoeUi => "Segoe UI",
            BFontFamily::Inter => "Inter",
            BFontFamily::Menlo => "Menlo",
            BFontFamily::Consolas => "Consolas",
            BFontFamily::CascadiaCode => "Cascadia Code",
            BFontFamily::JetBrainsMono => "JetBrains Mono",
        }
    }

    /// The value `typography.family` holds.
    pub fn slug(self) -> &'static str {
        match self {
            BFontFamily::Bundled(slug) => slug,
            BFontFamily::System => "system",
            BFontFamily::HelveticaNeue => "helvetica_neue",
            BFontFamily::SegoeUi => "segoe_ui",
            BFontFamily::Inter => "inter",
            BFontFamily::Menlo => "menlo",
            BFontFamily::Consolas => "consolas",
            BFontFamily::CascadiaCode => "cascadia_code",
            BFontFamily::JetBrainsMono => "jetbrains_mono",
        }
    }

    /// A bundled family by its slug, else one of the system's; a family whose
    /// folder has gone is not a choice any more.
    pub fn from_slug(slug: &str) -> Option<Self> {
        if let Some(family) = bundle::find(slug) {
            return Some(BFontFamily::Bundled(family.slug));
        }
        Self::SYSTEM.into_iter().find(|family| family.slug() == slug)
    }

    /// The family name the text system is asked for.
    pub fn family(self) -> &'static str {
        match self {
            BFontFamily::Bundled(_) => self.label(),
            BFontFamily::System => "sans-serif",
            other => other.label(),
        }
    }

    /// The faces tried when `family` is not installed, nearest first, so a
    /// Mac face on Windows lands on its Windows counterpart and a code face
    /// stays a code face.
    pub fn fallbacks(self) -> &'static [&'static str] {
        match self {
            BFontFamily::Bundled(_) => &["Segoe UI", "Helvetica Neue", "Inter", "sans-serif"],
            BFontFamily::System => &["Segoe UI", "Helvetica Neue", "Inter"],
            BFontFamily::HelveticaNeue => &["Helvetica", "Segoe UI", "Inter", "sans-serif"],
            BFontFamily::SegoeUi => &["Helvetica Neue", "Inter", "sans-serif"],
            BFontFamily::Inter => &["Segoe UI", "Helvetica Neue", "sans-serif"],
            BFontFamily::Menlo => &["Consolas", "Cascadia Code", "DejaVu Sans Mono", "monospace"],
            BFontFamily::Consolas => &["Menlo", "Cascadia Code", "DejaVu Sans Mono", "monospace"],
            BFontFamily::CascadiaCode => &["Consolas", "Menlo", "JetBrains Mono", "monospace"],
            BFontFamily::JetBrainsMono => &["Cascadia Code", "Consolas", "Menlo", "monospace"],
        }
    }

    /// Whether the face is a code face — every glyph the same width.
    pub fn monospace(self) -> bool {
        matches!(self, BFontFamily::Menlo | BFontFamily::Consolas | BFontFamily::CascadiaCode | BFontFamily::JetBrainsMono)
    }
}
