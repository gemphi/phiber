//! The Asset Browser's main region: the chosen library's assets, under the
//! chosen catalog, as a grid of tiles (`FILE_IMGDISPLAY`). A tile clicked
//! adds its asset to the scene at the 3D cursor, as Add would
//! (`ctx.op.scene_edit`, `scene::edit`) — Blender drags an asset into the
//! viewport; a click is the one gesture a tile here answers.
//!
//! Blender's Essentials library is the assets it ships (`datafiles/assets`);
//! phiber's carries built-ins it makes itself, so a scene can be built
//! from the browser with no file: a Cube and a UV Sphere under Primitives,
//! a Sun under Lights. Current File lists what the open file marked as an
//! asset, which phiber does not read yet, so it lists nothing.

use super::asset_sidebar::library;
use crate::state::{AssetKey, BKey as _, BState, OpKey};
use crate::view::BView;
use crate::view::prim::{BAction, BCard, BColumn, BThumb};

/// One built-in asset: its name, tile icon, catalog and what Add makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BAsset {
    pub name: &'static str,
    pub icon: &'static str,
    pub catalog: &'static str,
    /// The `add:<slug>` it fires (`scene::edit::BAddKind::of`).
    pub slug: &'static str,
}

/// The Essentials library's built-ins.
pub const ESSENTIALS: &[BAsset] = &[
    BAsset { name: "Cube", icon: "cube", catalog: "Primitives", slug: "cube" },
    BAsset { name: "UV Sphere", icon: "sphere", catalog: "Primitives", slug: "uv_sphere" },
    BAsset { name: "Sun", icon: "sun", catalog: "Lights", slug: "light_sun" },
];

/// What a library holds: Essentials' built-ins, the same under All (every
/// library together), nothing in Current File.
pub fn of_library(library: &str) -> &'static [BAsset] {
    match library {
        "Essentials" | "All" => ESSENTIALS,
        _ => &[],
    }
}

/// The catalogs a library's assets are in, each once, in order.
pub fn catalogs(library: &str) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for asset in of_library(library) {
        if !out.contains(&asset.catalog) {
            out.push(asset.catalog);
        }
    }
    out
}

/// The assets shown: the library's, narrowed to the catalog chosen — All,
/// Unassigned (none of the built-ins), or one catalog.
pub fn shown(s: &BState) -> Vec<BAsset> {
    let catalog = s.get(AssetKey::Catalog).unwrap_or("all");
    of_library(library(s))
        .iter()
        .filter(|asset| match catalog {
            "all" => true,
            "unassigned" => false,
            path => asset.catalog == path,
        })
        .copied()
        .collect()
}

/// Add `asset` to the scene.
pub fn add(asset: &BAsset) -> BAction {
    BAction::Set(OpKey::SceneEdit.path(), format!("add:{}", asset.slug))
}

fn tile(asset: &BAsset, size: f32) -> BView {
    BView::thumbnail(BThumb {
        preview_path: String::new(),
        icon: asset.icon.to_string(),
        label: asset.name.to_string(),
        subtitle: asset.catalog.to_string(),
        size,
        selected: false,
        action: Some(add(asset)),
    })
}

pub fn view(s: &BState) -> BView {
    let assets = shown(s);
    let body = match assets.is_empty() {
        true => BView::label(match library(s) {
            "Current File" => "No assets marked in the current file",
            _ => "No assets in this catalog",
        }),
        false => BView::grid(assets.iter().map(|asset| tile(asset, super::thumb::size(s))).collect()),
    };
    BView::from(BCard::column(BColumn::Body, vec![body]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::action::apply_action;

    #[test]
    fn essentials_offers_a_cube_a_sphere_and_a_sun_by_catalog() {
        let mut s = BState::new();
        s.set(AssetKey::Library, "Essentials");
        let names: Vec<&str> = shown(&s).iter().map(|a| a.name).collect();
        assert_eq!(names, ["Cube", "UV Sphere", "Sun"]);
        assert_eq!(catalogs("Essentials"), ["Primitives", "Lights"]);
        s.set(AssetKey::Catalog, "Lights");
        assert_eq!(shown(&s).iter().map(|a| a.name).collect::<Vec<_>>(), ["Sun"]);
        s.set(AssetKey::Library, "Current File");
        assert!(shown(&s).is_empty(), "the open file marks no assets");
    }

    #[test]
    fn a_tile_clicked_adds_its_asset_to_the_scene() {
        crate::scene::source::clear_loaded_scene();
        crate::scene::source::clear_scene_objects();
        let mut s = BState::new();
        for asset in ESSENTIALS {
            assert!(crate::scene::edit::BAddKind::of(asset.slug).is_some(), "{} adds something", asset.name);
            apply_action(&mut s, &add(asset));
        }
        let added: Vec<String> = crate::scene::edit::BSceneEdit::now(&s).added.into_iter().map(|(name, _)| name).collect();
        // The default scene has a Cube and a Sphere already.
        assert_eq!(added, ["Cube.001", "Sphere.001", "Sun"]);
        assert_eq!(s.get(crate::state::CtxKey::Object), Some("Sun"), "the last one added is active");
    }
}
