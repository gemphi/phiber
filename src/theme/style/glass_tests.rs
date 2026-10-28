use super::*;
use crate::theme::{BPaletteName, BThemeMode, ui_style_defaults_in};

/// A theme's style as the window draws it in glass: resolved, then glazed.
fn glass(name: BPaletteName, reduced: bool) -> BResolvedUiStyle {
    crate::theme::resolve_ui_style(ui_style_defaults_in(name, BThemeMode::Glass), &BState::new()).glazed(reduced)
}

/// `[r, g, b]` of a theme colour, ignoring its alpha.
fn rgb(color: BColor) -> [f32; 3] {
    let c = gpui::hsla(color.0 / 360.0, color.1 / 100.0, color.2 / 100.0, 1.0).to_rgb();
    [c.r, c.g, c.b]
}

/// `top` laid over `under` at its own alpha.
fn over(top: BColor, under: [f32; 3]) -> [f32; 3] {
    let [r, g, b] = rgb(top);
    let a = top.3;
    [r * a + under[0] * (1.0 - a), g * a + under[1] * (1.0 - a), b * a + under[2] * (1.0 - a)]
}

/// WCAG relative luminance and contrast.
fn luminance([r, g, b]: [f32; 3]) -> f32 {
    let linear = |c: f32| if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
    0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
}

fn contrast(a: [f32; 3], b: [f32; 3]) -> f32 {
    let (hi, lo) = (luminance(a).max(luminance(b)), luminance(a).min(luminance(b)));
    (hi + 0.05) / (lo + 0.05)
}

#[test]
fn every_theme_has_a_glass_mode_on_its_dark_ladder() {
    for name in BPaletteName::ALL.iter().copied() {
        let glass = ui_style_defaults_in(name, BThemeMode::Glass);
        let dark = ui_style_defaults_in(name, BThemeMode::Dark);
        assert_eq!(glass.colors, dark.colors, "{name:?}: glass reads the dark palette and glazes it when drawn");
        assert_eq!(glass.effects.glass, Some(LIQUID_GLASS), "{name:?}");
        assert_eq!(dark.effects.glass, None, "{name:?}: dark is not glass");
        assert_eq!(ui_style_defaults_in(name, BThemeMode::Light).effects.glass, None, "{name:?}");
        assert!(glass.effects.surface_shadow.rung() >= BShadowStyle::Large.rung(), "{name:?}: glass floats, so it casts");
    }
}

#[test]
fn glazing_touches_surfaces_and_leaves_canvas_text_and_meanings() {
    for name in BPaletteName::ALL.iter().copied() {
        let glazed = glass(name, false);
        let dark = crate::theme::resolve_ui_style(ui_style_defaults_in(name, BThemeMode::Dark), &BState::new());
        for (slot, alpha) in [
            (glazed.colors.bg_app, LIQUID_GLASS.ground),
            (glazed.colors.bg_top, LIQUID_GLASS.chrome),
            (glazed.colors.bg_panel, LIQUID_GLASS.pane),
            (glazed.colors.bg_control, LIQUID_GLASS.control),
        ] {
            assert_eq!(slot.3, alpha, "{name:?}");
        }
        assert_eq!(glazed.colors.bg_panel.2, dark.colors.bg_panel.2, "{name:?}: a pane keeps its shade");
        assert_eq!(glazed.viewport_surface().3, 1.0, "{name:?}: the canvas is content");
        assert_eq!((glazed.colors.text, glazed.colors.accent), (dark.colors.text, dark.colors.accent), "{name:?}");
        assert_eq!(glazed.colors.bg_modal_dim, dark.colors.bg_modal_dim, "{name:?}");
    }
}

#[test]
fn text_reads_over_a_white_desktop() {
    // The worst backdrop the blur can bring: pure white. The ground dims it
    // first, as Apple dims behind Clear glass, then the pane; body text on
    // the result must hold WCAG AA (4.5 : 1), in every theme.
    let white = [1.0; 3];
    for name in BPaletteName::ALL.iter().copied() {
        let style = glass(name, false);
        let ground = over(style.colors.bg_app, white);
        for (what, surface) in [("pane", style.colors.bg_panel), ("top bar", style.colors.bg_top), ("header", style.surface(crate::theme::BSurfaceTone::Header).bottom)] {
            let ratio = contrast(rgb(style.colors.text), over(surface, ground));
            assert!(ratio >= 4.5, "{name:?} {what}: text at {ratio:.2} : 1 over a white desktop");
        }
        // Floating chrome over the canvas: no blur there, so it must read
        // over the canvas's own shade.
        let floating = style.step(style.colors.bg_panel, 1);
        let floating = (floating.0, floating.1, floating.2, style.effects.floating_alpha);
        let ratio = contrast(rgb(style.colors.text), over(floating, rgb(style.viewport_surface())));
        assert!(ratio >= 4.5, "{name:?}: floating text at {ratio:.2} : 1 over the canvas");
    }
}

#[test]
fn reduce_transparency_is_solid_and_keeps_the_rims() {
    for name in BPaletteName::ALL.iter().copied() {
        let style = glass(name, true);
        let c = style.colors;
        for slot in [c.bg_app, c.sep, c.bg_top, c.bg_activity, c.bg_panel, c.bg_header, c.bg_control, c.row_alt] {
            assert_eq!(slot.3, 1.0, "{name:?}");
        }
        let glass = style.effects.glass.expect("still glass");
        assert!(!glass.blurs(), "{name:?}: no backdrop");
        assert_eq!(glass.rim, LIQUID_GLASS.rim, "{name:?}: the rims stay");
        assert_eq!(style.effects.floating_alpha, ui_style_defaults_in(name, BThemeMode::Dark).effects.floating_alpha);
    }
}

#[test]
fn tiles_are_frosted_and_the_lit_one_is_tinted() {
    let style = glass(BPaletteName::DEFAULT, false);
    let rest = style.icon_tile_rest(crate::host::BHostType::File);
    assert_eq!(rest.3, LIQUID_GLASS.tile, "a resting tile is mostly what is behind it");
    assert!(rest.2 > 90.0, "and frosted white");
    let on = style.icon_tile_on();
    assert_eq!((on.0, on.1, on.2), (style.colors.accent.0, style.colors.accent.1, style.colors.accent.2));
    assert!(on.3 < 1.0, "the accent as tinted glass");
    let dark = crate::theme::resolve_ui_style(ui_style_defaults_in(BPaletteName::DEFAULT, BThemeMode::Dark), &BState::new());
    assert_eq!(dark.icon_tile_on(), dark.colors.accent, "outside glass the lit tile is the accent");
    assert!(style.icon_ink(false).2 > dark.icon_ink(false).2, "a glass icon is a step brighter");
}

#[test]
fn the_backdrop_follows_the_mode_and_the_switch() {
    let mut state = BState::new();
    assert!(!backdrop(&state), "light by default: opaque");
    state.set(AppKey::ThemeMode, "glass");
    assert!(backdrop(&state));
    state.set(AppKey::ReduceTransparency, V_TRUE);
    assert!(!backdrop(&state));
}
