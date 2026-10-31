//! The Preferences editor's keys — `ctx.prefs.*`.
//!
//! Only the nav-bar tab, and it earns a file because it was spelt **twice**:
//! `part::activity::option::prefs_tab_key()` and
//! `modal::command::find` each wrote
//! `BPath::owned(vec![BHostType::Prefs.slug(), "navbar", "tab"])`. Neither
//! ruler could see it — `key_budget.rs` only checks the domains in its
//! `MOVED` table, and `prefs` was not one.
//!
//! Shaped after [`ButsKey::NavbarTab`](super::ButsKey): each host's keys live
//! in that host's enum. There is no `NavbarExpanded` here on purpose —
//! `part::strip::expanded_key(host, strip)` already builds
//! `<host>.<strip>.expanded` for every rail, and a variant for it would be a
//! second spelling of that path.

use super::BKey;
use crate::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrefsKey {
    /// The active category of the Preferences nav bar, by label. The
    /// Activity bar writes it so a category picked there selects the same
    /// tab the Preferences screen reads.
    NavbarTab,
}

impl BKey for PrefsKey {
    fn path(&self) -> BPath {
        match self {
            Self::NavbarTab => BPath::owned(vec!["prefs", "navbar", "tab"]),
        }
    }
}
