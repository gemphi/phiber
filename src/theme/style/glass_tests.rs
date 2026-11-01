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
        assert!((glazed.colors.bg_panel.2 - dark.colors.bg_panel.2).abs() < 0.5, "{name:?}: a pane keeps its shade");
        assert!(glazed.colors.bg_panel.1 < 1.0, "{name:?}: glass has no colour of its own");
        assert_eq!(glazed.viewport_surface().3, 1.0, "{name:?}: the canvas is content");
        assert_eq!((glazed.colors.text, glazed.colors.accent), (dark.colors.text, dark.colors.accent), "{name:?}");
        assert_eq!(glazed.colors.bg_modal_dim, dark.colors.bg_modal_dim, "{name:?}");
    }
}

/// Every window as it is drawn: what is behind it, frosted (thicker the
/// brighter it is, `glass.hlsl`), then its fill. Body text must hold WCAG AA
/// (4.5 : 1) in every theme, over the canvas, the brightest render it can show,
/// and a white desktop seen through the window's ground.
#[test]
fn text_reads_over_a_white_desktop() {
    let thick = |luma: f32| {
        let t = ((luma - 0.35) / 0.6).clamp(0.0, 1.0);
        (super::THICKEST * t * t * (3.0 - 2.0 * t)).max(LIQUID_GLASS.frost.tint)
    };
    for name in BPaletteName::ALL.iter().copied() {
        let style = glass(name, false);
        let ground = style.colors.bg_app;
        let floating = style.step(style.colors.bg_panel, 1);
        let floating = (floating.0, floating.1, floating.2, style.effects.floating_alpha);
        let windows = [
            ("pane", style.colors.bg_panel),
            ("top bar", style.colors.bg_top),
            ("header", style.surface(crate::theme::BSurfaceTone::Header).bottom),
            ("floating chrome", floating),
        ];
        let behind = [
            ("canvas", rgb(style.viewport_surface())),
            ("bright render", [0.75; 3]),
            ("white render", [1.0; 3]),
            ("white desktop", over(ground, [1.0; 3])),
        ];
        for (what, fill) in windows {
            for (under, canvas) in behind {
                let luma = 0.2126 * canvas[0] + 0.7152 * canvas[1] + 0.0722 * canvas[2];
                let frost = (ground.0, ground.1, ground.2, thick(luma));
                let ratio = contrast(rgb(style.colors.text), over(fill, over(frost, canvas)));
                assert!(ratio >= 4.5, "{name:?} {what}: text at {ratio:.2} : 1 over the {under}");
            }
        }
    }
}

#[test]
fn reduce_transparency_is_solid() {
    for name in BPaletteName::ALL.iter().copied() {
        let style = glass(name, true);
        let c = style.colors;
        for slot in [c.bg_app, c.sep, c.bg_top, c.bg_activity, c.bg_panel, c.bg_header, c.bg_control, c.row_alt] {
            assert_eq!(slot.3, 1.0, "{name:?}");
        }
        let glass = style.effects.glass.expect("still glass");
        assert!(!glass.blurs(), "{name:?}: no backdrop");
        assert_eq!(style.effects.floating_alpha, ui_style_defaults_in(name, BThemeMode::Dark).effects.floating_alpha);
    }
}

/// Glass is the normal look with its windows made of glass: nothing a person
/// presses is glass, and the theme's radii stand.
#[test]
fn only_the_windows_are_glass() {
    for name in BPaletteName::ALL.iter().copied() {
        let glazed = glass(name, false);
        let dark = crate::theme::resolve_ui_style(ui_style_defaults_in(name, BThemeMode::Dark), &BState::new());
        assert_eq!(glazed.icon_tile_on(), dark.icon_tile_on(), "{name:?}: the lit tile");
        assert_eq!(glazed.colors.bg_control.3, 1.0, "{name:?}: the control is not glass");
        assert_eq!(glazed.radii, dark.radii, "{name:?}: the theme's radii");
        assert!(glazed.colors.bg_panel.3 < 1.0, "{name:?}: the window is");
    }
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
