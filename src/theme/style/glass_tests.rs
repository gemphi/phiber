use super::*;
use crate::theme::{BPaletteName, BThemeMode, ui_style_defaults_in};

/// Each glass mode beside the plain mode whose ladder it reads.
const MODES: [(BThemeMode, BThemeMode); 2] = [(BThemeMode::Glass, BThemeMode::Dark), (BThemeMode::GlassLight, BThemeMode::Light)];

/// A theme's style as the window draws it in a glass mode: resolved, then glazed.
fn glass_in(name: BPaletteName, mode: BThemeMode, reduced: bool) -> BResolvedUiStyle {
    crate::theme::resolve_ui_style(ui_style_defaults_in(name, mode), &BState::new()).glazed(reduced)
}

/// A theme's style resolved in a plain mode.
fn plain(name: BPaletteName, mode: BThemeMode) -> BResolvedUiStyle {
    crate::theme::resolve_ui_style(ui_style_defaults_in(name, mode), &BState::new())
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
fn every_theme_has_a_glass_mode_on_each_ladder() {
    for name in BPaletteName::ALL.iter().copied() {
        for (mode, base) in MODES {
            let glass = ui_style_defaults_in(name, mode);
            let base = ui_style_defaults_in(name, base);
            assert_eq!(glass.colors, base.colors, "{name:?} {mode:?}: glass reads its ladder and glazes it when drawn");
            assert_eq!(glass.radii, base.radii, "{name:?} {mode:?}");
            assert_eq!(glass.effects.glass, Some(LIQUID_GLASS), "{name:?} {mode:?}");
            assert_eq!(base.effects.glass, None, "{name:?}: a plain mode is not glass");
            assert!(glass.effects.surface_shadow.rung() >= BShadowStyle::Large.rung(), "{name:?} {mode:?}: glass floats, so it casts");
        }
    }
}

#[test]
fn glazing_touches_surfaces_and_leaves_canvas_text_and_meanings() {
    for name in BPaletteName::ALL.iter().copied() {
        for (mode, base) in MODES {
            let glazed = glass_in(name, mode, false);
            let base = plain(name, base);
            for (slot, alpha) in [
                (glazed.colors.bg_app, LIQUID_GLASS.ground),
                (glazed.colors.bg_top, LIQUID_GLASS.chrome),
                (glazed.colors.bg_panel, LIQUID_GLASS.pane),
                (glazed.colors.bg_control, LIQUID_GLASS.control),
            ] {
                assert_eq!(slot.3, alpha, "{name:?} {mode:?}");
            }
            assert!((glazed.colors.bg_panel.2 - base.colors.bg_panel.2).abs() < 0.5, "{name:?} {mode:?}: a pane keeps its shade");
            assert!(glazed.colors.bg_panel.1 < 1.0, "{name:?} {mode:?}: glass has no colour of its own");
            assert_eq!(glazed.viewport_surface().3, 1.0, "{name:?} {mode:?}: the canvas is content");
            assert_eq!((glazed.colors.text, glazed.colors.accent), (base.colors.text, base.colors.accent), "{name:?} {mode:?}");
            assert_eq!(glazed.colors.bg_modal_dim, base.colors.bg_modal_dim, "{name:?} {mode:?}");
        }
    }
}

/// `[r, g, b]`'s lightness as the shader reads it (`glass.hlsl`).
fn luma([r, g, b]: [f32; 3]) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Every window as it is drawn: what is behind it, frosted (thicker the further
/// it stands from the tint, `glass.hlsl`), then its fill. Body text must hold
/// WCAG AA (4.5 : 1) in every theme and both modes, over the canvas, anything
/// from black to white, and a white or black desktop seen through the window's
/// ground.
#[test]
fn text_reads_over_a_white_desktop() {
    for name in BPaletteName::ALL.iter().copied() {
        for (mode, _) in MODES {
            let style = glass_in(name, mode, false);
            let ground = style.colors.bg_app;
            let tint = rgb(ground);
            let thick = |under: [f32; 3]| {
                let t = (((luma(under) - luma(tint)).abs() - 0.2) / 0.4).clamp(0.0, 1.0);
                (super::THICKEST * t * t * (3.0 - 2.0 * t)).max(LIQUID_GLASS.frost.tint)
            };
            let floating = style.step(style.colors.bg_panel, 1);
            let floating = (floating.0, floating.1, floating.2, style.effects.floating_alpha);
            let windows = [
                ("pane", style.colors.bg_panel),
                ("top bar", style.colors.bg_top),
                ("header", style.surface(crate::theme::BSurfaceTone::Header).bottom),
                ("floating chrome", floating),
            ];
            // Every grey from black to white, and the canvas and the desktops.
            let greys = (0..=20).map(|v| [v as f32 / 20.0; 3]);
            let behind: Vec<[f32; 3]> =
                greys.chain([rgb(style.viewport_surface()), over(ground, [1.0; 3]), over(ground, [0.0; 3])]).collect();
            for (what, fill) in windows {
                for canvas in behind.iter().copied() {
                    let frost = (ground.0, ground.1, ground.2, thick(canvas));
                    let ratio = contrast(rgb(style.colors.text), over(fill, over(frost, canvas)));
                    assert!(ratio >= 4.5, "{name:?} {mode:?} {what}: text at {ratio:.2} : 1 over {canvas:?}");
                }
            }
        }
    }
}

#[test]
fn reduce_transparency_is_solid() {
    for name in BPaletteName::ALL.iter().copied() {
        for (mode, base) in MODES {
            let style = glass_in(name, mode, true);
            let c = style.colors;
            for slot in [c.bg_app, c.sep, c.bg_top, c.bg_activity, c.bg_panel, c.bg_header, c.bg_control, c.row_alt] {
                assert_eq!(slot.3, 1.0, "{name:?} {mode:?}");
            }
            let glass = style.effects.glass.expect("still glass");
            assert!(!glass.blurs(), "{name:?} {mode:?}: no backdrop");
            assert_eq!(style.effects.floating_alpha, ui_style_defaults_in(name, base).effects.floating_alpha);
        }
    }
}

/// Glass is the normal look with its windows made of glass: nothing a person
/// presses is glass, and the theme's radii stand.
#[test]
fn only_the_windows_are_glass() {
    for name in BPaletteName::ALL.iter().copied() {
        for (mode, base) in MODES {
            let glazed = glass_in(name, mode, false);
            let base = plain(name, base);
            assert_eq!(glazed.icon_tile_on(), base.icon_tile_on(), "{name:?} {mode:?}: the lit tile");
            assert_eq!(glazed.colors.bg_control.3, 1.0, "{name:?} {mode:?}: the control is not glass");
            assert_eq!(glazed.radii, base.radii, "{name:?} {mode:?}: the theme's radii");
            assert!(glazed.colors.bg_panel.3 < 1.0, "{name:?} {mode:?}: the window is");
        }
    }
}

#[test]
fn the_backdrop_follows_the_mode_and_the_switch() {
    let mut state = BState::new();
    assert!(!backdrop(&state), "light by default: opaque");
    state.set(AppKey::ThemeMode, "glass");
    assert!(backdrop(&state));
    state.set(AppKey::ThemeMode, "glass_light");
    assert!(backdrop(&state));
    state.set(AppKey::ReduceTransparency, V_TRUE);
    assert!(!backdrop(&state));
}
