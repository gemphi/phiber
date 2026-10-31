//! The Dope Sheet's Select menu beyond all/none/invert: the column selects
//! and Before / After Current Frame (`ACTION_OT_select_column`,
//! `ACTION_OT_select_leftright`).
//!
//! A fourth file of Dope Sheet tests, so each sits under the 240-line read
//! cap (`tests/level_structure.rs`). Same fixture as `tests_more.rs`: the
//! Cube, selected and active, moves by CubeAction's location keys at 1 and 20.

use super::*;
use crate::state::action::apply_action;
use crate::state::{BKey as _, OpKey};
use crate::view::prim::BAction;

fn model() -> SceneModel {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/actions.blend");
    let scene = crate::phiphx::blend::BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    SceneModel::from_blend_owned(scene, &path.to_string_lossy())
}

fn op(s: &mut BState, value: &str) {
    apply_action(s, &BAction::Set(OpKey::Keys.path(), value.to_string()));
}

fn picks(s: &BState) -> Vec<(String, f32)> {
    BKeySelection::now(s).0.into_iter().map(|p| (p.object, p.frame)).collect()
}
/// Select ▸ Column on Current Frame and Columns on Selected Keys
/// (`ACTION_OT_select_column`): both ADD to the selection — Blender's column
/// select never clears what is already picked.
#[test]
fn column_select_adds_the_columns_it_names() {
    let model = model();
    crate::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");

    // The playhead on 20: that column joins an empty selection.
    s.set(CtxKey::Frame, "20");
    op(&mut s, "column:cfra");
    assert_eq!(picks(&s), [("Cube".to_string(), 20.0)]);
    // And on 1 it ADDS rather than replacing.
    s.set(CtxKey::Frame, "1");
    op(&mut s, "column:cfra");
    let mut now = picks(&s);
    now.sort_by(|a, b| a.1.total_cmp(&b.1));
    assert_eq!(now, [("Cube".to_string(), 1.0), ("Cube".to_string(), 20.0)]);

    // A frame with nothing keyed on it picks nothing up.
    op(&mut s, "none");
    s.set(CtxKey::Frame, "7");
    op(&mut s, "column:cfra");
    assert!(picks(&s).is_empty(), "no key on frame 7");

    // Columns on Selected Keys: the frames already picked, re-selected
    // across every shown row (one object here, so it is a no-op that must
    // not lose the selection).
    op(&mut s, "click:1:20");
    op(&mut s, "column:keys");
    assert_eq!(picks(&s), [("Cube".to_string(), 20.0)]);
}

/// Select ▸ Before / After Current Frame (`ACTION_OT_select_leftright`):
/// this one REPLACES, and a key standing ON the playhead belongs to both
/// sides — Blender runs the range to `cfra ± 0.1`.
#[test]
fn left_and_right_replace_the_selection_and_share_the_playhead() {
    let model = model();
    crate::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    // Keys at 1 and 20; playhead between them.
    s.set(CtxKey::Frame, "10");
    op(&mut s, "leftright:left");
    assert_eq!(picks(&s), [("Cube".to_string(), 1.0)], "before 10");
    op(&mut s, "leftright:right");
    assert_eq!(picks(&s), [("Cube".to_string(), 20.0)], "after 10, and it replaced");

    // On a keyed frame the key is in both halves.
    s.set(CtxKey::Frame, "20");
    op(&mut s, "leftright:right");
    assert_eq!(picks(&s), [("Cube".to_string(), 20.0)]);
    op(&mut s, "leftright:left");
    let mut both = picks(&s);
    both.sort_by(|a, b| a.1.total_cmp(&b.1));
    assert_eq!(both, [("Cube".to_string(), 1.0), ("Cube".to_string(), 20.0)], "the playhead's own key counts as before it too");
    crate::scene::source::set_loaded_scene(None);
}

/// Between Selected Markers with no marker selected says so and changes
/// nothing (Blender's `ED_markers_get_minmax` would have no extremes).
#[test]
fn between_selected_markers_needs_a_marker() {
    let model = model();
    crate::scene::source::set_loaded_scene(Some(model));
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    op(&mut s, "click:1:1");
    op(&mut s, "column:between");
    assert_eq!(picks(&s), [("Cube".to_string(), 1.0)], "nothing added, nothing lost");
    crate::scene::source::set_loaded_scene(None);
}
