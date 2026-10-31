//! Thumbnails display: `.blend` files show their embedded preview and the
//! `REND` scene / frame range, like Blender's file browser.

use phiber::state::BKey as _;
use phiber::state::FileKey;
use phiber::part::file::table;
use phiber::scene::view_slot::BViewSlot;
use phiber::state::BState;
use phiber::state::View3dKey;
use phiber::view::BView;
use std::path::PathBuf;

fn find<'a>(v: &'a BView, out: &mut Vec<&'a BView>) {
    match v {
        BView::Thumbnail(_) => out.push(v),
        BView::Grid { list } | BView::Row { list } | BView::Column { list } => {
            list.iter().for_each(|c| find(c, out))
        }
        BView::Card(card) if matches!(card.kind, phiber::view::prim::BCardKind::Column(_)) => card.body.iter().for_each(|c| find(c, out)),
        _ => {}
    }
}

#[test]
fn thumbnails_mode_shows_blend_previews() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures");
    if !dir.join("THYMUS-BOLDROID-WAVE-003.blend").is_file() {
        return;
    }
    let mut s = BState::new();
    s.set(&FileKey::Path.path(), dir.to_string_lossy().to_string());
    s.set(&FileKey::Display.path(), "Thumbnails");
    let view = table::view(&s);
    let mut tiles = Vec::new();
    find(&view, &mut tiles);
    let thymus = tiles
        .iter()
        .find_map(|t| match t {
            BView::Thumbnail(t) if t.label.starts_with("THYMUS") => Some(t),
            _ => None,
        })
        .expect("THYMUS tile");
    assert!(thymus.preview_path.ends_with(".blend"));
    assert_eq!(thymus.subtitle, "Scene · 1–140 · 3.6");
    let info = phiber::part::file::preview::quick_info(&thymus.preview_path).unwrap();
    let t = info.thumbnail.as_ref().expect("embedded preview");
    assert_eq!((t.width, t.height), (128, 128));
}

#[test]
fn list_mode_is_unchanged() {
    let s = BState::new();
    let view = table::view(&s);
    let mut tiles = Vec::new();
    find(&view, &mut tiles);
    assert!(tiles.is_empty());
}

/// "Load UI" (#28): the Open dialog offers the option, and opening THYMUS
/// restores its Layout workspace and camera view.
#[test]
fn open_restores_saved_workspace_and_view() {
    use phiber::phiphx::Phx;
    use phiber::app::load_ui::restore_ui;
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../phiphx/tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend");
    let model = Phx::open(std::path::Path::new(path)).expect("THYMUS loads");
    let v = model.ui.view.as_ref().expect("saved 3D view");
    assert!(v.camera_view, "saved looking through the scene camera");
    let mut s = phiber::state::BState::default();
    s.set(phiber::state::CtxKey::Space, "Shading");
    restore_ui(&mut s, &model);
    assert_eq!(s.get(phiber::state::CtxKey::Space), Some("Layout"));
    assert!(s.get(&BViewSlot::key(View3dKey::Yaw)).is_some());
    let opt = phiber::part::file::option::panel(&phiber::state::BState::default());
    assert!(opt.is_some(), "Open shows the Load UI option");
}
