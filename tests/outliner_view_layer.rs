//! Opening a real `.blend` shows Blender's View Layer tree: the file's
//! collections (not a flat list), Blender object-type icons, the file's
//! active object, and hidden collections with their eye closed.

use phiber::state::{BKey as _, CtxKey};
use phiber::state::OutlinerKey;
use phiber::phiphx::Phx;
use phiber::app::{import::{ImportOutcome, settle_import}};
use phiber::part::outliner::tree::{self, BRowKind};
use phiber::scene::source::{clear_loaded_scene, set_loaded_scene};
use phiber::state::{BState};
use std::path::PathBuf;

fn fixture() -> Option<PathBuf> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../phiphx/tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend");
    p.is_file().then_some(p)
}

#[test]
fn loaded_blend_outliner_mirrors_the_collection_tree() {
    let Some(path) = fixture() else { return };
    let model = Phx::open(&path).expect("load fixture");

    // Scene properties come from the file, not factory defaults.
    assert_eq!(model.timeline.frame_end, 140);
    assert_eq!(model.render.resolution_x, 1440);
    assert_eq!(model.render.engine, "CYCLES");

    let mut s = BState::new();
    let settled = settle_import(&mut s, ImportOutcome::Loaded(model)).expect("model");

    set_loaded_scene(Some(settled.clone()));
    let rows = tree::flatten(&s);
    clear_loaded_scene();

    assert_eq!(rows[0].kind, BRowKind::SceneCollection);
    assert_eq!(rows[0].label, "Scene Collection");
    for name in ["Cam+Lights", "THYMUS-GRP", "08_biped_geo_grp"] {
        assert!(
            rows.iter().any(|r| r.kind == BRowKind::Collection && r.label == name),
            "collection {name} missing from the outliner"
        );
    }
    // Nested collections are deeper than their parents.
    let depth = |label: &str| rows.iter().find(|r| r.label == label).unwrap().depth;
    assert!(depth("08_biped_geo_grp") > depth("THYMUS-GRP"));
    // Cameras and lights appear under their collections with their own icons.
    let cam = rows.iter().find(|r| r.label == "front_cam").expect("camera row");
    assert_eq!(cam.icon, "camera");
    let light = rows.iter().find(|r| r.label == "spotlight_01").expect("light row");
    assert_eq!(light.icon, "light");

    // "Cam+Lights" is hidden in the file's view layer → the COLLECTION's eye
    // is closed and everything under it inherits that, as
    // `LAYER_COLLECTION_HIDE` does. Marking each object instead would have
    // left them hidden after the collection's eye was opened again.
    let hidden = phiber::part::outliner::layer::hidden_by_collections(&s, &settled, phiber::part::outliner::key::is_hidden);
    assert!(hidden.contains("front_cam"), "front_cam hides with its collection");
    assert!(!tree::is_hidden(&s, "front_cam"), "…not through a flag of its own");
    assert!(!hidden.contains("head_base_geo"));
    assert!(!tree::is_hidden(&s, "head_base_geo"));
    // A collection row leads with its checkbox; unchecking it excludes the
    // collection from the view layer, so everything under it goes too.
    set_loaded_scene(Some(settled.clone()));
    let views = phiber::view::BViewItem::build(&phiber::part::outliner::BOutlinerItem::new(), &s);
    let phiber::view::BView::Column { list } = views.shown() else { panic!("a column") };
    let grp = list
        .iter()
        .find(|v| matches!(v, phiber::view::BView::TreeRow { list, .. } if list.iter().any(|c| matches!(c, phiber::view::BView::Label { text } | phiber::view::BView::Button(phiber::view::prim::BControl { label: Some(text), .. }) if text == "THYMUS-GRP"))))
        .expect("the THYMUS-GRP row");
    let phiber::view::BView::TreeRow { lead, .. } = grp else { panic!() };
    let phiber::view::BView::Button(check) = &lead[0] else { panic!("a checkbox leads the row: {lead:?}") };
    assert_eq!(check.icon.as_deref(), Some("check_square"), "included: checked");
    let Some(phiber::view::BAction::Set(key, value)) = &check.action else { panic!() };
    s.set(key, value.clone());
    let hidden = phiber::part::outliner::layer::hidden_by_collections(&s, &settled, phiber::part::outliner::key::is_hidden);
    assert!(hidden.contains("head_base_geo"), "excluded with its collection");
    let views = phiber::view::BViewItem::build(&phiber::part::outliner::BOutlinerItem::new(), &s);
    let phiber::view::BView::Column { list } = views.shown() else { panic!() };
    let unchecked = list.iter().any(|v| matches!(v, phiber::view::BView::TreeRow { lead, .. } if lead.iter().any(|c| matches!(c, phiber::view::BView::Button(b) if b.icon.as_deref() == Some("square")))));
    assert!(unchecked, "the box is drawn unchecked");
    clear_loaded_scene();
    // The file's active object (if any) became Phiber's active object.
    if let Some(active) = &settled.selection.active {
        assert_eq!(s.get(CtxKey::Object), Some(active.as_str()));
    }
}

#[test]
fn outliner_search_keeps_ancestor_collections() {
    let Some(path) = fixture() else { return };
    let model = Phx::open(&path).expect("load fixture");
    let mut s = BState::new();
    s.set(&OutlinerKey::Search.path(), "neck_geo");
    set_loaded_scene(Some(model));
    let rows = tree::flatten(&s);
    clear_loaded_scene();
    let labels: Vec<_> = rows.iter().map(|r| r.label.as_str()).collect();
    assert!(labels.contains(&"neck_geo"), "{labels:?}");
    assert!(labels.contains(&"08_biped_geo_grp"), "{labels:?}");
    assert!(!labels.contains(&"front_cam"), "non-matching rows are filtered: {labels:?}");
}
