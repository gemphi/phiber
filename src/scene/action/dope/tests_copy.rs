//! The copy buffer, the column fold, and the Keyframe Type a row carries.
//!
//! A third file of Dope Sheet tests, so each sits under the 240-line read
//! cap (`tests/level_structure.rs`). Same fixture as `tests_more.rs`: the
//! Cube, selected and active, moves by CubeAction's location keys at 1 and 20.

use super::*;
use crate::scene::action::BSceneActions;
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

fn cube_x(s: &BState, model: &SceneModel) -> Vec<f32> {
    let actions = BSceneActions::now(s).keyed_actions(model);
    let action = actions.iter().find(|a| a.name == "CubeAction").unwrap();
    action.curves.iter().find(|c| c.path == "location" && c.index == 0).map_or(Vec::new(), |c| c.keys.iter().map(|k| k.co[0]).collect())
}

/// Key ▸ Copy Keyframes then Paste Keyframes (`set_copy`): the copied
/// column lands on the playhead with the values it held, the spacing of a
/// multi-column copy is kept, and the pasted keys are what ends up selected.
#[test]
fn copy_and_paste_land_the_buffer_on_the_playhead() {
    let model = model();
    crate::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");

    // Nothing picked: copy says so and fills no buffer.
    op(&mut s, "copy");
    assert!(s.get(CtxKey::KeyBuffer).unwrap_or("").is_empty(), "a copy with no selection buffers nothing");
    // Nothing buffered: paste changes no keys.
    let before = cube_x(&s, &model);
    op(&mut s, "paste");
    assert_eq!(cube_x(&s, &model), before, "a paste with an empty buffer is a no-op");

    // Copy the column at frame 1, paste it at 30.
    op(&mut s, "click:1:1");
    assert_eq!(picks(&s), [("Cube".to_string(), 1.0)]);
    op(&mut s, "copy");
    assert!(!s.get(CtxKey::KeyBuffer).unwrap_or("").is_empty(), "the column is in the buffer");
    assert_eq!(picks(&s), [("Cube".to_string(), 1.0)], "a copy leaves the selection alone");
    s.set(CtxKey::Frame, "30");
    op(&mut s, "paste");
    assert_eq!(cube_x(&s, &model), [1.0, 20.0, 30.0], "the key lands on the playhead");
    assert_eq!(picks(&s), [("Cube".to_string(), 30.0)], "the pasted keys are selected");

    // The pasted key carries the value the copied one held.
    let actions = BSceneActions::now(&s).keyed_actions(&model);
    let action = actions.iter().find(|a| a.name == "CubeAction").unwrap();
    let curve = action.curves.iter().find(|c| c.path == "location" && c.index == 0).unwrap();
    let at = |f: f32| curve.keys.iter().find(|k| (k.co[0] - f).abs() < 5e-4).map(|k| k.co[1]).unwrap();
    assert!((at(30.0) - at(1.0)).abs() < 1e-4, "frame 30 holds frame 1's value: {} against {}", at(30.0), at(1.0));
}

/// A two-column copy keeps the gap between the columns: the earliest lands
/// on the playhead, the other that many frames after it (Blender's
/// `KEYFRAME_PASTE_OFFSET_CFRA_START`).
#[test]
fn a_multi_column_paste_keeps_its_spacing() {
    let model = model();
    crate::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    op(&mut s, "all");
    assert_eq!(picks(&s).len(), 2, "both of the Cube's columns");
    op(&mut s, "copy");
    // 1 and 20 are 19 apart: pasted at 50 they land on 50 and 69.
    s.set(CtxKey::Frame, "50");
    op(&mut s, "paste");
    let keys = cube_x(&s, &model);
    assert!(keys.contains(&50.0) && keys.contains(&69.0), "the gap is kept: {keys:?}");
    let mut landed = picks(&s);
    landed.sort_by(|a, b| a.1.total_cmp(&b.1));
    assert_eq!(landed, [("Cube".to_string(), 50.0), ("Cube".to_string(), 69.0)]);
}

/// The buffer round-trips through the key it is kept in.
#[test]
fn the_copy_buffer_round_trips() {
    use super::set_copy::BKeyCopy;
    use crate::scene::action::keys::BKeyed;
    let list = vec![
        BKeyCopy { object: "Cube".into(), at: 0.0, channels: vec![BKeyed { path: "location".into(), index: 0, value: 1.5 }] },
        // An object name and a path that both hold the separators.
        BKeyCopy {
            object: "Odd#Name".into(),
            at: 19.0,
            channels: vec![BKeyed { path: "pose.bones[\"a:b\"].location".into(), index: 2, value: -1.0 }],
        },
    ];
    assert_eq!(BKeyCopy::parse_all(&BKeyCopy::encode_all(&list)), list);
}

/// `fold_columns` is Blender's `nupdate_ak_bezt`: keys on the same frame
/// become one column, and a 'proper' keyframe has priority over every other
/// type — so a column holding one is a keyframe however many breakdowns or
/// jitters share its frame. Failing that, the first key's type stands.
#[test]
fn a_column_takes_a_proper_keyframe_over_any_other_type() {
    use crate::phiphx::blend::{KEYTYPE_BREAKDOWN, KEYTYPE_GENERATED, KEYTYPE_JITTER, KEYTYPE_KEYFRAME};
    use crate::scene::action::dope::fold_columns;

    // Out of order, with two curves keyed on frame 5 — one a breakdown, one
    // a proper keyframe: the column is a keyframe.
    let (frames, types) = fold_columns(vec![
        (9.0, KEYTYPE_JITTER),
        (5.0, KEYTYPE_BREAKDOWN),
        (1.0, KEYTYPE_BREAKDOWN),
        (5.0, KEYTYPE_KEYFRAME),
    ]);
    assert_eq!(frames, [1.0, 5.0, 9.0], "sorted and folded");
    assert_eq!(types, [KEYTYPE_BREAKDOWN, KEYTYPE_KEYFRAME, KEYTYPE_JITTER]);

    // The priority does not run the other way: a keyframe already in the
    // column is not displaced by a later breakdown.
    let (_, types) = fold_columns(vec![(5.0, KEYTYPE_KEYFRAME), (5.0, KEYTYPE_BREAKDOWN)]);
    assert_eq!(types, [KEYTYPE_KEYFRAME]);

    // Two non-keyframe types on one frame: the first stands (Blender has no
    // priority between them — "and other types for now").
    let (_, types) = fold_columns(vec![(5.0, KEYTYPE_GENERATED), (5.0, KEYTYPE_JITTER)]);
    assert_eq!(types, [KEYTYPE_GENERATED]);

    // Frames are rounded to a thousandth before folding, so keys at "the
    // same" frame really do fold.
    let (frames, _) = fold_columns(vec![(5.0, KEYTYPE_KEYFRAME), (5.00004, KEYTYPE_KEYFRAME)]);
    assert_eq!(frames, [5.0]);

    assert_eq!(fold_columns(Vec::new()), (Vec::new(), Vec::new()));
}

/// The type a Keyframe Type edit sets reaches the row the Dope Sheet draws:
/// set the column at frame 1 to a breakdown and the object's row says so,
/// index for index with its frames.
#[test]
fn the_row_carries_the_keyframe_type_it_was_given() {
    use crate::phiphx::blend::{KEYTYPE_BREAKDOWN, KEYTYPE_KEYFRAME};
    use crate::view::draw::canvas::time::row::BTimeRow;
    let model = model();
    crate::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    op(&mut s, "click:1:1");
    op(&mut s, "keytype:breakdown");

    let actions = BSceneActions::now(&s).keyed_actions(&model);
    let row = BTimeRow::object("Cube", &actions);
    assert_eq!(row.keys, [1.0, 20.0]);
    assert_eq!(row.keytype_at(0), KEYTYPE_BREAKDOWN, "the edited column is a breakdown");
    assert_eq!(row.keytype_at(1), KEYTYPE_KEYFRAME, "the other is untouched");
    // Past the end reads as a plain keyframe rather than panicking.
    assert_eq!(row.keytype_at(99), KEYTYPE_KEYFRAME);
}

