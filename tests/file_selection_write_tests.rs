//! Task 11.2 — File Browser selection write tests.
//!
//! Feature: blend-file-mesh-loading
//! Validates: Requirement 1.1
//!
//! Selecting a `.blend` row in the File Browser writes the resolvable path to
//! the `file.selected` state key, replacing any prior value.
//! Selecting a non-`.blend` row also writes the key, but `should_import`
//! returns false for it.

use phiber::{
    app::import::should_import,
    state::action::apply_action,
    state::{BKey as _, BState},
    view::prim::BAction,
};

// --------------------------------------------------------------------------
// Helpers
// --------------------------------------------------------------------------

/// Dispatch a `Set("file", "selected", path)` action — mirrors what the File
/// Browser row's on-click handler sends.
fn select_file(state: &mut BState, path: &str) {
    apply_action(state, &BAction::Set(phiber::state::FileKey::Selected.path(), path.to_string()));
}

// --------------------------------------------------------------------------
// .blend selection writes the path to file.selected
// --------------------------------------------------------------------------

#[test]
fn blend_selection_writes_to_the_file_selected_key() {
    let mut state = BState::new();
    select_file(&mut state, "/path/to/model.blend");
    assert_eq!(
        state.get(phiber::state::FileKey::Selected),
        Some("/path/to/model.blend"),
        "file.selected must hold the selected .blend path"
    );
}

#[test]
fn blend_selection_uppercase_extension_writes_correctly() {
    let mut state = BState::new();
    select_file(&mut state, "/assets/Mesh.BLEND");
    assert_eq!(state.get(phiber::state::FileKey::Selected), Some("/assets/Mesh.BLEND"));
}

#[test]
fn blend_selection_triggers_should_import() {
    let mut state = BState::new();
    select_file(&mut state, "/dir/model.blend");
    let selected = state.get(phiber::state::FileKey::Selected);
    assert!(
        should_import(None, selected),
        "should_import must be true for a new .blend selection"
    );
}

// --------------------------------------------------------------------------
// Non-.blend selection writes the key but does NOT trigger import
// --------------------------------------------------------------------------

#[test]
fn non_blend_selection_writes_key() {
    let mut state = BState::new();
    select_file(&mut state, "/notes/readme.txt");
    assert_eq!(
        state.get(phiber::state::FileKey::Selected),
        Some("/notes/readme.txt"),
        "file.selected is written even for non-.blend paths"
    );
}

#[test]
fn non_blend_selection_does_not_trigger_should_import() {
    let mut state = BState::new();
    select_file(&mut state, "notes.txt");
    let selected = state.get(phiber::state::FileKey::Selected);
    assert!(
        !should_import(None, selected),
        "should_import must be false for a non-.blend path"
    );
}

#[test]
fn empty_selection_does_not_trigger_should_import() {
    let mut state = BState::new();
    select_file(&mut state, "");
    let selected = state.get(phiber::state::FileKey::Selected);
    // An empty selection: the state key may be absent or empty; either way
    // should_import must be false.
    assert!(!should_import(None, selected));
}

// --------------------------------------------------------------------------
// Second selection replaces the first
// --------------------------------------------------------------------------

#[test]
fn second_blend_replaces_first() {
    let mut state = BState::new();
    select_file(&mut state, "/first.blend");
    assert_eq!(state.get(phiber::state::FileKey::Selected), Some("/first.blend"));

    select_file(&mut state, "/second.blend");
    assert_eq!(
        state.get(phiber::state::FileKey::Selected),
        Some("/second.blend"),
        "second selection must overwrite the first"
    );
}

#[test]
fn non_blend_replaces_prior_blend() {
    let mut state = BState::new();
    select_file(&mut state, "/first.blend");
    select_file(&mut state, "image.png");
    assert_eq!(state.get(phiber::state::FileKey::Selected), Some("image.png"));
    // should_import is false now.
    assert!(!should_import(
        Some("/first.blend"),
        state.get(phiber::state::FileKey::Selected)
    ));
}

#[test]
fn blend_after_non_blend_triggers_import() {
    let mut state = BState::new();
    select_file(&mut state, "readme.txt");
    select_file(&mut state, "/new/model.blend");
    let selected = state.get(phiber::state::FileKey::Selected);
    assert!(
        should_import(None, selected),
        "blend after non-blend must trigger import"
    );
}

// --------------------------------------------------------------------------
// Selecting the same .blend again does NOT trigger re-import
// --------------------------------------------------------------------------

#[test]
fn same_blend_selection_does_not_retrigger_import() {
    let mut state = BState::new();
    select_file(&mut state, "/scene.blend");
    // Simulate that this blend is already loaded.
    let loaded_path = Some("/scene.blend");
    let selected = state.get(phiber::state::FileKey::Selected);
    assert!(
        !should_import(loaded_path, selected),
        "selecting the already-loaded .blend must not trigger re-import"
    );
}
