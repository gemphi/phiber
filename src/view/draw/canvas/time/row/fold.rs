//! A channel row's fold — `ACHANNEL_SETTING_EXPAND`.
//!
//! Which rows fold, what each one's fold is called, and whether it stands
//! open. Split from `row/mod.rs` so that file sits under the 240-line read
//! cap (`tests/level_structure.rs`); the fold is also the one thing three
//! separate callers have to agree on, so it reads better gathered:
//!
//! - `BTimeRow::list` asks [`BTimeRow::is_expanded`] to decide whether to
//!   list a row's children at all,
//! - the channel list draws its disclosure triangle from [`BTimeRow::fold`]
//!   (`part::channels::time`),
//! - and Channel ▸ Expand / Collapse Channels walks the same `fold` over the
//!   shown rows (`scene::action::channel_fold`).
//!
//! All three therefore name a fold the same way, which is the point: the key
//! is built here and nowhere else.

use super::{BRowKind, BTimeRow};
use crate::state::BState;

impl BTimeRow {
    /// Whether `name`'s channel is unfolded.
    ///
    /// Closed by default, as Blender leaves an object's channels closed until
    /// asked: a rig with three hundred bone curves would otherwise bury every
    /// other object in the list on first paint.
    pub fn is_expanded(s: &BState, name: &str) -> bool {
        s.get(crate::state::DopeKey::Expanded(crate::state::intern(name)))
            == Some(crate::state::val::V_TRUE)
    }

    /// The key this row's fold is kept under (`DopeKey::Expanded`) and
    /// whether it stands open — `None` for a row that does not fold
    /// (`ACHANNEL_SETTING_EXPAND` is unavailable on it).
    pub fn fold(&self) -> Option<(String, bool)> {
        match &self.kind {
            BRowKind::Object { name, expanded } => Some((name.clone(), *expanded)),
            BRowKind::Slot { action, identifier, expanded, .. } => Some((Self::slot_key(action, identifier), *expanded)),
            BRowKind::Group { object, name, expanded } => Some((Self::group_key(object, name), *expanded)),
            // The summary's fold is not kept in the state (the Dope Sheet
            // always opens it, the Timeline never does), and a curve, a
            // driver, an NLA track and an action line are leaves.
            _ => None,
        }
    }

    /// The fold key of `action`'s slot `identifier`.
    pub fn slot_key(action: &str, identifier: &str) -> String {
        format!("{action}\u{1f}slot\u{1f}{identifier}")
    }

    /// The fold key of `object`'s group `group`: one object's `chest` is not
    /// another's.
    pub fn group_key(object: &str, group: &str) -> String {
        format!("{object}\u{1f}{group}")
    }
}
