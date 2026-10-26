//! The typeface is a Preferences choice — *Typography ▸ Family* and *Weight*.
//! The families are every one found in `assets/phi-fonts` (Phi-Soft
//! Condensed, the platform's own, is the default), then the system's, two
//! Mac faces, Inter, and VS Code's four code faces — resolved like every other
//! style value and applied once at the root.

use phiber::{
    state::{BState, key::ui_style as key},
    theme::{BFontFamily, BPaletteName, BPreferenceControl, UI_STYLE_DESCRIPTORS, resolve_ui_style, ui_style_defaults, ui_style_defaults_for},
};

#[test]
fn the_folder_families_then_the_systems_each_with_a_stack_and_a_slug_that_round_trips() {
    let all = BFontFamily::all();
    let bundled: Vec<&str> = all.iter().filter(|f| matches!(f, BFontFamily::Bundled(_))).map(|f| f.label()).collect();
    // Not a count: a folder dropped into `assets/phi-fonts` is a new choice.
    assert!(bundled.contains(&"Phi-Soft Condensed") && bundled.contains(&"Phi-Soft"), "each width is its own family: {bundled:?}");
    assert_eq!(all.len(), bundled.len() + BFontFamily::SYSTEM.len(), "then the eight the machine supplies");
    assert_eq!(all.iter().filter(|family| family.monospace()).count(), 4, "four of them code faces");
    for family in all {
        assert!(!family.family().is_empty() && !family.label().is_empty());
        assert!(!family.fallbacks().is_empty(), "{family:?} has a stack to fall back down");
        assert!(!family.fallbacks().contains(&family.family()), "{family:?} does not fall back to itself");
        assert_eq!(BFontFamily::from_slug(family.slug()), Some(family));
        // A code face falls back to code faces, a text face to text faces.
        let generic = if family.monospace() { "monospace" } else { "sans-serif" };
        assert!(
            family == BFontFamily::System || family.fallbacks().contains(&generic),
            "{family:?} ends on the generic {generic}"
        );
    }
    assert_eq!(BFontFamily::DEFAULT.label(), "Phi-Soft Condensed", "the platform's own face, which ships in the app");
    assert_eq!(BFontFamily::from_slug("comic_sans"), None);
}

#[test]
fn the_family_is_a_typography_choice_that_resolves_from_state() {
    let descriptor = UI_STYLE_DESCRIPTORS
        .iter()
        .find(|descriptor| descriptor.target == "typography.family")
        .expect("the family is a style descriptor");
    assert_eq!((descriptor.preference.section, descriptor.preference.label), ("Typography", "Family"));
    assert_eq!(descriptor.preference.control, BPreferenceControl::Choice);
    let labels: Vec<&str> = descriptor.choices().iter().map(|choice| choice.label).collect();
    assert_eq!(labels, BFontFamily::all().iter().map(|family| family.label()).collect::<Vec<_>>());

    let defaults = ui_style_defaults(BPaletteName::Sphere);
    let mut state = BState::new();
    assert_eq!(resolve_ui_style(defaults, &state).typography.family, BFontFamily::DEFAULT, "Phi-Soft Condensed until another is chosen");
    state.set(&key::TYPOGRAPHY_FAMILY, "menlo");
    assert_eq!(resolve_ui_style(defaults, &state).typography.family, BFontFamily::Menlo);
    state.set(&key::TYPOGRAPHY_FAMILY, "phi_sans");
    assert_eq!(resolve_ui_style(defaults, &state).typography.family.label(), "Phi-Sans", "a folder family is chosen by its slug");
    state.set(&key::TYPOGRAPHY_FAMILY, "papyrus");
    assert_eq!(resolve_ui_style(defaults, &state).typography.family, BFontFamily::DEFAULT, "an unknown face is ignored");
    for palette in BPaletteName::ALL.iter().copied() {
        assert_eq!(ui_style_defaults(palette).typography.family, BFontFamily::DEFAULT, "{palette:?}: every theme starts on Phi-Soft Condensed");
    }
}

#[test]
fn the_weight_is_a_choice_that_starts_light() {
    let descriptor = UI_STYLE_DESCRIPTORS
        .iter()
        .find(|descriptor| descriptor.target == "typography.weight")
        .expect("the weight is a style descriptor");
    assert_eq!((descriptor.preference.section, descriptor.preference.label), ("Typography", "Weight"));
    assert_eq!(descriptor.preference.control, BPreferenceControl::Choice);
    let labels: Vec<&str> = descriptor.choices().iter().map(|choice| choice.label).collect();
    assert_eq!(labels, ["Light", "Regular", "SemiBold", "Bold"]);

    let mut state = BState::new();
    let resolved = |state: &BState| resolve_ui_style(ui_style_defaults_for(state), state).typography.weight;
    assert_eq!(resolved(&state), 300.0, "Light: the Regular reads heavy at interface sizes");
    state.set(&key::TYPOGRAPHY_WEIGHT, "600");
    assert_eq!(resolved(&state), 600.0);
    state.set(&key::TYPOGRAPHY_WEIGHT, "heavy");
    assert_eq!(resolved(&state), 300.0, "a weight that is not a number is ignored");
}

#[test]
fn the_root_sets_the_family_and_weight_once_for_every_letter_under_it() {
    let backend = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/view/draw/backend.rs")).unwrap();
    assert!(backend.contains(".font_family(family.family())"), "the root takes the chosen family");
    assert!(backend.contains(".font_weight("), "and the chosen weight");
    assert!(backend.contains("font_fallbacks"), "and its stack");
    let others = ["view/draw/button.rs", "view/draw/field.rs", "view/draw/card.rs", "view/draw/tab/mod.rs"];
    for file in others {
        let source = std::fs::read_to_string(format!("{}/src/{file}", env!("CARGO_MANIFEST_DIR"))).unwrap();
        assert!(!source.contains("font_family("), "{file} inherits the root's face rather than naming one");
    }
}
