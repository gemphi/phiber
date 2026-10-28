//! The one place a `BShadowStyle` meets GPUI's shadow rungs.
//!
//! Split out of `apply.rs` for the file cap. Keeping it alone is the point:
//! a lifted container, a popover and a modal reach their depth through this
//! function, so the elevation scale (`theme::BElevation`) has exactly one
//! place where it becomes pixels.

use crate::theme::BShadowStyle;
use gpui::prelude::*;

/// `element` casting the theme's surface shadow — nothing for `None`. The
/// one place a `BShadowStyle` meets GPUI's shadow rungs, so a lifted
/// container and a popover cast the same shadow in a given theme.
pub(in crate::view::draw) fn shadowed<E: Styled>(element: E, shadow: BShadowStyle) -> E {
    match shadow {
        BShadowStyle::None => element,
        BShadowStyle::Small => element.shadow_sm(),
        BShadowStyle::Medium => element.shadow_md(),
        BShadowStyle::Large => element.shadow_lg(),
        BShadowStyle::ExtraLarge => element.shadow_xl(),
    }
}

/// `element` standing on the brand's hard shade: no blur, `offset` px right
/// and down, in the accent darkened as the logo's letters are over theirs.
/// It follows the element's corners, so an egg casts an egg.
pub(in crate::view::draw) fn cast<E: Styled>(element: E, offset: f32, accent: crate::theme::BColor) -> E {
    let shade = super::super::color::to_hsla((accent.0, accent.1, accent.2 * SHADE_DEPTH, accent.3));
    element.shadow(vec![gpui::BoxShadow {
        color: shade,
        offset: gpui::point(gpui::px(offset), gpui::px(offset)),
        blur_radius: gpui::px(0.0),
        spread_radius: gpui::px(0.0),
        inset: false,
    }])
}

/// How dark the brand's shade is against the accent: the golden ratio's
/// inverse, as the logo's offset is to its face.
const SHADE_DEPTH: f32 = 0.618;

/// `element` as a tile of glass, when the glass mode is in force and
/// `fill` is a tile (`BFill::is_tile`): a hairline rim of white inside the
/// edge, a specular line along the top, a soft glow from within and a faint
/// halo, so its icon glows. These are inset and drop shadows, so the tile
/// keeps its size and corners. The one place a tile becomes glass
/// (`theme/style/glass`).
pub(in crate::view::draw) fn glazed<E: Styled>(element: E, fill: super::fill::BFill, style: &crate::theme::BResolvedUiStyle) -> E {
    let Some(glass) = style.effects.glass.filter(|_| fill.is_tile()) else { return element };
    let white = |alpha: f32| gpui::hsla(0.0, 0.0, 1.0, alpha);
    let px = gpui::px;
    let light = |alpha: f32, y: f32, blur: f32, spread: f32, inset: bool| gpui::BoxShadow {
        color: white(alpha),
        offset: gpui::point(px(0.0), px(y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        inset,
    };
    element.shadow(vec![
        light(glass.halo, 0.0, glass.halo_blur, 0.0, false),
        light(glass.rim, 0.0, 0.0, 1.0, true),
        light(glass.rim, 1.0, 1.0, 0.0, true),
        light(glass.glow, 0.0, glass.glow_blur, 0.0, true),
    ])
}
