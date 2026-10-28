//! The Project Browser's state, under `project.*`: the folder picked in the
//! navigator, the asset picked in the grid, and how the grid is shown —
//! cards or a list, their size, their order, the filter.

use super::BKey;
use crate::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProjectKey {
    /// The navigator's folder: a kind's slug, or `all` (All Files).
    Folder,
    /// The asset picked, by its spelling in the project database.
    Selected,
    /// `true` shows a list rather than cards.
    List,
    /// The cards' edge, px.
    Size,
    /// `true` orders Z to A.
    Descending,
    /// The filter field's text.
    Search,
    /// A footwear design, by slug, confirmed against its sketch.
    Confirmed(&'static str),
    /// `false` folds a navigator section shut (`collections`, `project`,
    /// `library`); open unless said otherwise, as the file browser's are.
    Open(&'static str),
}

impl BKey for ProjectKey {
    fn path(&self) -> BPath {
        let tail: &'static str = match self {
            Self::Confirmed(slug) => return BPath::owned(vec!["project", "confirmed", slug]),
            Self::Open(section) => return BPath::owned(vec!["project", "open", section]),
            Self::Folder => "folder",
            Self::Selected => "selected",
            Self::List => "list",
            Self::Size => "size",
            Self::Descending => "descending",
            Self::Search => "search",
        };
        BPath::owned(vec!["project", tail])
    }
}
