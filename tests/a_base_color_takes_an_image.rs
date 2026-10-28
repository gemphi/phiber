//! The Material tab's Base Color takes an image, as Blender's Image
//! Texture ▸ Open does: the folder beside the Surface panel's Image field
//! opens the file browser, and the file committed there is the active
//! object's base texture in the scene every mode draws and F12 renders
//! (`render_scene`). Undo takes it back, Redo brings it again, and a
//! colour typed afterwards unplugs it.

use gpui::{Modifiers, TestAppContext, px, size};
use phiber::app::BPhiberApp;
use phiber::phiphx::Phx;
use phiber::scene::{BObject3D, render_scene};
use phiber::space::BSpaceType;
use phiber::state::{BKey as _, ButsKey, CtxKey, FileKey, ModalKey, OpKey, val::{BValue, V_MODAL_FILE_BROWSER}};
use phiber::view::BAction;

const PLANE: &str = "PackedPlane";
const OPEN: &str = "btn-icon-set-ctx.file.purpose-base_color_image-folder_open";

/// The plane's base image, by size, in the scene F12 renders.
fn base_image(app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext) -> Option<(u32, u32)> {
    app.read_with(cx, |a, _| {
        let loaded = a.scene_cache.as_ref().map(|(_, m)| std::sync::Arc::clone(m));
        render_scene(&a.state, loaded.as_deref()).into_iter().find_map(|o| match o {
            BObject3D::Loaded { name, materials, .. } if name == PLANE => Some(materials.first().and_then(|m| m.texture.as_ref()).map(|t| (t.width, t.height))),
            _ => None,
        })
    })
    .flatten()
}

fn act(app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext, path: phiber::state::BPath, value: String) {
    app.update(cx, |app, cx| app.handle_action(&BAction::Set(path, value), cx));
    cx.run_until_parked();
}

#[gpui::test]
async fn the_base_color_opens_an_image_that_undo_takes_back_and_a_colour_unplugs(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber-base-color-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("bricks.png");
    image::RgbaImage::from_pixel(6, 4, image::Rgba([180, 60, 40, 255])).save(&file).unwrap();
    let image = file.to_string_lossy().to_string();
    let blend = concat!(env!("CARGO_MANIFEST_DIR"), "/../phiphx/tests/fixtures/TEXTURED-PACKED.blend");
    let model = Phx::open(std::path::Path::new(blend)).expect("the fixture loads");

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        app.state.set(&phiber::scene::loaded_path_key(), blend.to_string());
        app.scene_cache = Some((blend.to_string(), std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();
    act(&app, cx, CtxKey::Object.path(), PLANE.into());
    let packed = base_image(&app, cx).expect("the file's own packed image");
    app.update(cx, |app, cx| {
        app.state.set(&ButsKey::NavbarTab.path(), "Material");
        cx.notify();
    });
    cx.run_until_parked();

    // Image Texture ▸ Open: the folder beside the Image field raises the
    // file browser for it.
    let open = cx.debug_bounds(OPEN).expect("the Surface panel's Image row has its folder");
    cx.simulate_click(open.center(), Modifiers::none());
    cx.run_until_parked();
    let state = |app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext, key: phiber::state::BPath| app.read_with(cx, |a, _| a.state.get(&key).map(str::to_string));
    assert_eq!(state(&app, cx, ModalKey::Open.path()).as_deref(), Some(V_MODAL_FILE_BROWSER), "the file browser opens");

    // The browser commits the file: the plane's base image is it.
    act(&app, cx, FileKey::Selected.path(), image.clone());
    assert_eq!(base_image(&app, cx), Some((6, 4)), "the opened image is the base texture F12 renders");
    assert_eq!(state(&app, cx, ButsKey::Value("base_color_image").path()), Some(image.clone()), "the Image field names it");
    assert_eq!(state(&app, cx, FileKey::Purpose.path()), None, "the browser's job is done");

    // Undo takes it back to the file's own; Redo brings it again.
    act(&app, cx, OpKey::Undo.path(), String::new());
    assert_eq!(base_image(&app, cx), Some(packed), "undone: the file's packed image again");
    act(&app, cx, OpKey::Redo.path(), String::new());
    assert_eq!(base_image(&app, cx), Some((6, 4)), "redone: the opened image again");

    // A flat colour unplugs it.
    act(&app, cx, ButsKey::Value("diffuse_color").path(), BValue::Color([0.9, 0.1, 0.1, 1.0]).encode());
    assert_eq!(base_image(&app, cx), None, "a colour typed in: no image under Base Color");
    assert_eq!(state(&app, cx, ButsKey::Value("base_color_image").path()).as_deref(), Some(""));
}
