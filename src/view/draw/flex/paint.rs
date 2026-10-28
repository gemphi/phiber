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
