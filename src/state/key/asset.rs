//! The asset browser's filters — `ctx.asset.*`.
//!
//! Blender keeps these on the space (`SpaceFile.asset_params`:
//! `AssetLibraryReference` and the active catalog); the browser's sidebar
//! writes them and its grid reads them (`part::file::asset_sidebar`).

use super::BKey;
use crate::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AssetKey {
    /// The active catalog filter: `all`, `unassigned`, or a catalog path.
    Catalog,
    /// The active asset library's name (`AssetLibraryReference`).
    Library,
}

impl BKey for AssetKey {
    fn path(&self) -> BPath {
        let tail: &'static str = match self {
            Self::Catalog => "catalog",
            Self::Library => "library",
        };
        BPath::owned(vec!["asset", tail])
    }
}
