//! The Spreadsheet editor's keys — `ctx.spreadsheet.*`.
//!
//! The domain is which attribute level the table lists (`SpaceSpreadsheet.
//! attribute_domain`: Point, Edge, Face, Face Corner), chosen in the dataset
//! region (`part::spreadsheet::dataset`).
//!
//! The literal `"spreadsheet"` is `host::spreadsheet::SLUG` — host state
//! nests under the host's slug (see the module README). It is written here
//! rather than read from `BHostType` for the same reason `ButsKey` writes
//! `"buts"`: this file is where the domain's spelling lives, and a key enum
//! that reached back into `host::` to spell itself would make the path
//! depend on two places again.

use super::BKey;
use crate::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpreadsheetKey {
    /// The attribute domain the table lists, by slug.
    Domain,
}

impl BKey for SpreadsheetKey {
    fn path(&self) -> BPath {
        let tail: &'static str = match self {
            Self::Domain => "domain",
        };
        BPath::owned(vec!["spreadsheet", tail])
    }
}
