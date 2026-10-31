//! Asset-browser tools region: library selector + "Catalogs" panel
//! (All / Unassigned, then the library's own catalogs), Blender's
//! `asset_catalog_tree_view`. The assets themselves are `asset_grid.rs`.

use super::sidebar::panel;
use crate::state::{AssetKey, BKey as _, BState};
use crate::view::prim::{BAction, BCard, BColumn, BControl, with_peers};
use crate::view::BView;

/// Blender's built-in library choices (`ASSET_LIBRARY_*` in `asset_library.hh`).
pub const LIBRARIES: &[&str] = &["Current File", "Essentials", "All"];

/// The library a new Asset Browser shows: every library at once, as
/// Blender's does (`ASSET_LIBRARY_ALL`).
pub const DEFAULT_LIBRARY: &str = "All";

/// The library chosen.
pub fn library(s: &BState) -> &str {
    s.get(AssetKey::Library).unwrap_or(DEFAULT_LIBRARY)
}

fn catalog_button(s: &BState, icon: &'static str, label: &'static str, value: &'static str) -> BView {
    let active = s.get(AssetKey::Catalog).unwrap_or("all") == value;
    BView::Button(
        BControl::new()
            .with_icon(if active { "check" } else { icon })
            .with_label(label)
            .with_action(BAction::Set(AssetKey::Catalog.path(), value.to_string())),
    )
}

/// All, Unassigned, then each catalog the library's assets are in.
fn catalog_rows(s: &BState, library: &str) -> Vec<BView> {
    let mut rows = vec![catalog_button(s, "asset", "All", "all"), catalog_button(s, "help_circle", "Unassigned", "unassigned")];
    rows.extend(super::asset_grid::catalogs(library).into_iter().map(|c| catalog_button(s, "folder", c, c)));
    rows
}

/// Asset mode — library selector + "Catalogs" panel (All / Unassigned).
pub fn asset_view(s: &BState) -> BView {
    let current = library(s);
    let library_rows: Vec<BView> = LIBRARIES
        .iter()
        .map(|lib| {
            BView::Button(
                BControl::new()
                    .with_icon(if *lib == current { "check" } else { "asset" })
                    .with_label(*lib)
                    .with_action(BAction::Set(AssetKey::Library.path(), lib.to_string())),
            )
        })
        .collect();
    BView::from(BCard::column(BColumn::Sidebar, with_peers(vec![
            panel(s, "Library", library_rows),
            panel(s, "Catalogs", catalog_rows(s, current)),
        ]),))
}
