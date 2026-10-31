//! Keys — every path the state is addressed by, spelt once, as an enum.
//!
//! A key is `ctx.{domain}.{owner}.{field}`. The domain is a file here; each
//! variant of its enum is one field, and `BKey::path` is the only place the
//! segments are written. A caller writes `s.get(FileKey::Path)`, never a
//! string with a dot in it, so a reader looking for what the file browser
//! stores opens `keys/file.rs` and sees all of it; a search for a key's uses
//! is a search for a variant.
//!
//! The older `K_…` constants in `state/key.rs` and the `k_…()` functions
//! scattered through the parts are the domains not yet moved here; they
//! implement `BKey` through `BPath`, so both spellings meet at the store.

pub mod activity;
pub mod app;
pub mod area;
pub mod asset;
pub mod buts;
pub mod dope;
pub mod ctx;
pub mod file;
pub mod prefs;
pub mod sidebar;
pub mod spreadsheet;
pub mod image;
pub mod uv;
pub mod menu;
pub mod modal;
pub mod node;
pub mod op;
pub mod outliner;
pub mod play;
pub mod project;
pub mod screen;
pub mod scene;
pub mod time_view;
/// The theme tokens (`ctx.ui_style.*`) — a domain spelt as consts, see the file.
pub mod ui_style;
pub mod view3d;
pub mod zone;

pub use activity::ActivityKey;
pub use app::AppKey;
pub use area::AreaKey;
pub use asset::AssetKey;
pub use buts::ButsKey;
pub use prefs::PrefsKey;
pub use sidebar::SidebarKey;
pub use spreadsheet::SpreadsheetKey;
pub use dope::DopeKey;
pub use ctx::CtxKey;
pub use file::FileKey;
pub use image::ImageKey;
pub use uv::UvKey;
pub use menu::MenuKey;
pub use modal::ModalKey;
pub use node::NodeKey;
pub use op::OpKey;
pub use outliner::OutlinerKey;
pub use play::PlayKey;
pub use project::ProjectKey;
pub use screen::ScreenKey;
pub use scene::SceneKey;
pub use time_view::TimeViewKey;
pub use view3d::View3dKey;
pub use zone::ZoneKey;

use super::BPath;

/// Anything that names a state path.
pub trait BKey {
    fn path(&self) -> BPath;

    /// The family this key belongs to: everything up to and including the dot
    /// after the domain, so `ctx.file.path` gives `ctx.file.` and one answer
    /// covers every key under it. A key only one segment deep (`ctx.modal`)
    /// has no family below it and stands for itself, which is why the result
    /// is matched by `ends_with('.')` at the call site rather than always
    /// being a prefix.
    ///
    /// This is here so that no one has to WRITE a domain prefix: the domain
    /// name lives in the key enum and nowhere else (`tests/key_budget.rs`),
    /// and a list of families is a list of keys.
    fn family(&self) -> String {
        let encoded = self.path().encode();
        match encoded.match_indices('.').nth(1) {
            Some((at, _)) => encoded[..=at].to_string(),
            None => encoded,
        }
    }
}

impl BKey for BPath {
    fn path(&self) -> BPath {
        self.clone()
    }
}

impl<K: BKey> BKey for &K {
    fn path(&self) -> BPath {
        (**self).path()
    }
}

// ── The helpers that are not a key enum (was `state/key.rs`) ──


/// Integer / float state reads with a fallback.
pub fn int_or(s: &super::BState, key: impl super::BKey, dflt: i32) -> i32 {
    s.get(key).and_then(|v| v.parse().ok()).unwrap_or(dflt)
}
pub fn float_or(s: &super::BState, key: impl super::BKey, dflt: f32) -> f32 {
    s.get(key).and_then(|v| v.parse().ok()).unwrap_or(dflt)
}


