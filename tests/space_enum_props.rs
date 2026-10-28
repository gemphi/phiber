//! Property 1 — Space enum round-trip.
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirements 1.4, 1.5, 52.11
//!
//! For every `s` in `BSpaceType::EVERY`, `BSpaceType::from_slug(s.slug())`
//! returns `Some(s)`. For any string not in the slug set, `from_slug`
//! returns `None`.

use phiber::space::BSpaceType;
use proptest::prelude::*;

fn arb_space() -> impl Strategy<Value = BSpaceType> {
    prop_oneof![
        Just(BSpaceType::Layout),
        Just(BSpaceType::Modeling),
        Just(BSpaceType::Sculpting),
        Just(BSpaceType::UvEditing),
        Just(BSpaceType::TexturePaint),
        Just(BSpaceType::Shading),
        Just(BSpaceType::Animation),
        Just(BSpaceType::Rendering),
        Just(BSpaceType::Compositing),
        Just(BSpaceType::GeometryNodes),
        Just(BSpaceType::Scripting),
        Just(BSpaceType::Simulation),
    ]
}

proptest! {
    #[test]
    fn round_trip_slug_from_slug(s in arb_space()) {
        prop_assert_eq!(BSpaceType::from_slug(s.slug()), Some(s));
    }

    #[test]
    fn unknown_slug_is_none(garbage in "[a-z0-9]{1,20}") {
        // Reject any accidental match against a real slug.
        if BSpaceType::EVERY.iter().any(|t| t.slug() == garbage) {
            return Ok(());
        }
        prop_assert_eq!(BSpaceType::from_slug(&garbage), None);
    }
}

#[test]
fn every_has_twelve_variants() {
    assert_eq!(BSpaceType::EVERY.len(), 12);
}

/// The offered spaces are a subset of the spaces that exist: Sculpting
/// still resolves by slug but is not offered. Modelling and Simulation are,
/// in the Welcome window's New File order (Modelling before Shading).
#[test]
fn offered_spaces_exclude_sculpting_only() {
    assert_eq!(BSpaceType::ALL.len(), 11);
    assert!(BSpaceType::ALL.contains(&BSpaceType::Modeling));
    assert!(BSpaceType::ALL.contains(&BSpaceType::Simulation));
    assert!(!BSpaceType::ALL.contains(&BSpaceType::Sculpting));
    assert_eq!(BSpaceType::from_slug("Sculpting"), Some(BSpaceType::Sculpting));
    assert!(!BSpaceType::Sculpting.is_tab());
    let at = |t| BSpaceType::ALL.iter().position(|s| *s == t);
    assert!(at(BSpaceType::Modeling) < at(BSpaceType::Shading), "Modelling before Shading");
}
