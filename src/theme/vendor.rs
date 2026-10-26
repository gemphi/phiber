//! The only place phiber touches the vendored `theme` crate (`crates/phiui`):
//! it registers the gpui theme globals and answers gpui's font and density
//! questions. Every colour the app draws comes from `theme`, not from here.

use super::style::bundle;
use super::style::font::BFontFamily;
use super::style::metric::DEFAULT_TYPOGRAPHY;
use gpui::{App, Font, FontFeatures, FontStyle, FontWeight, Pixels, px};
use theme::{
    LoadThemes, ThemeSettingsProvider, UiDensity, init as theme_init, set_theme_settings_provider,
};

/// Fonts and density gpui asks for (tooltips and font queries panic without it).
struct BThemeSettings {
    ui: Font,
    buffer: Font,
}

impl ThemeSettingsProvider for BThemeSettings {
    fn ui_font<'a>(&'a self, _cx: &'a App) -> &'a Font {
        &self.ui
    }
    fn buffer_font<'a>(&'a self, _cx: &'a App) -> &'a Font {
        &self.buffer
    }
    fn ui_font_size(&self, _cx: &App) -> Pixels {
        px(13.0)
    }
    fn buffer_font_size(&self, _cx: &App) -> Pixels {
        px(14.0)
    }
    fn ui_density(&self, _cx: &App) -> UiDensity {
        UiDensity::Compact
    }
}

fn sans() -> Font {
    Font {
        family: BFontFamily::DEFAULT.family().into(),
        features: FontFeatures::default(),
        weight: FontWeight(DEFAULT_TYPOGRAPHY.weight),
        style: FontStyle::Normal,
        fallbacks: None,
    }
}

/// Register the gpui theme globals and our settings provider, once at start-up
/// before any window opens.
pub fn install(cx: &mut App) {
    // Every face in `assets/phi-fonts` (the browser: the one compiled in),
    // registered before the theme so the first frame already draws in it.
    let faces: Vec<_> = bundle::bundled().iter().flat_map(|family| family.face_bytes()).collect();
    if let Err(error) = cx.text_system().add_fonts(faces) {
        log::warn!("the bundled fonts could not be registered: {error}");
    }
    // A family the text system does not know by its name draws every letter
    // in the fallback face instead, so say which ones that would be.
    let known = cx.text_system().all_font_names();
    for family in bundle::bundled() {
        if known.iter().any(|seen| seen == family.name) {
            log::info!("font family {:?} registered, weights {:?}", family.name, family.weights);
        } else {
            log::warn!("font family {:?} is not among the faces this machine knows", family.name);
        }
    }
    theme_init(LoadThemes::JustBase, cx);
    set_theme_settings_provider(Box::new(BThemeSettings { ui: sans(), buffer: sans() }), cx);
}
