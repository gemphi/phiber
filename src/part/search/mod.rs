//! Header search — the field in the middle of a host header.
//!
//! Every host presents itself the same way: what it is on the left, what you
//! are looking for in the middle, how it is filtered on the right. A host that
//! has nothing to search leaves the middle empty.
//!
//! Blender ref: the outliner's `filter_text` and the file browser's
//! `filter_search`, both drawn in their header.

use crate::state::BKey as _;
use crate::state::AppKey;
use crate::state::CtxKey;
use crate::state::OutlinerKey;
use crate::space::BSpaceType;
use crate::state::FileKey;
use crate::host::BHostType;
use crate::state::BPath;
use crate::state::BState;
use crate::state::edit;
use crate::view::prim::BAction;
use crate::view::BView;
use crate::view::BViewItem;

#[derive(Clone, Copy, Debug)]
pub struct BSearch {
    pub host: BHostType,
}

impl BSearch {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

/// The key a host filters by, when it has one.
pub fn search_key(host: BHostType) -> Option<BPath> {
    match host {
        BHostType::Outliner => Some(OutlinerKey::Search.path()),
        // `SpaceProperties.search_filter` — Blender has searched the
        // Properties editor from its header since 2.90.
        BHostType::Buts => Some(crate::state::ButsKey::Search.path()),
        BHostType::File | BHostType::Asset => {
            Some(FileKey::Search.path())
        }
        // Reality Composer Pro's browser filters its cards by name.
        BHostType::Project => Some(crate::state::ProjectKey::Search.path()),
        // `DopeSheet.filter_text` — the animation editors narrow their
        // channels by name, which is how you find one in a rig. One key for
        // the four of them, as Blender has one `DopeSheet` per space showing
        // the same panel (`part::filter::dope`).
        host if crate::part::filter::filters_channels(host) => {
            Some(crate::state::DopeKey::Search.path())
        }
        // The top bar's search is the palette's: it finds every place the
        // window can take you (`modal::command::find`).
        BHostType::Topbar => Some(crate::state::AppKey::CommandQuery.path()),
        // The rest of the window's own chrome — the status bar, the activity
        // rail — is not an editor and has no search of its own; nor has an
        // empty area, which holds nothing to find.
        BHostType::Statusbar | BHostType::Activity | BHostType::Empty => None,
        // Every other editor searches from the centre of its header, the same
        // field everywhere: `<host>.search`.
        host => Some(BPath::owned(vec![host.slug(), "search"])),
    }
}

impl BViewItem for BSearch {
    fn build(&self, s: &BState) -> BView {
        let Some(key) = search_key(self.host) else {
            return BView::empty();
        };
        if self.host == BHostType::Topbar {
            return whereabouts(s, &key);
        }
        let value = s.get(&key).unwrap_or("").to_string();
        let editing = edit::is_editing(s, &key);
        BView::filter_field("Search", value, editing, BAction::Set(key, String::new()))
    }
}

/// The top bar's search at rest: where the window stands — `Spaces ›
/// Layout` — as crumbs in the field's box. The spaces are found here, not
/// in a row of tabs that nine of them would overflow, so the space you are
/// in reads here too.
///
/// A click on the box opens the palette (`find::BFindFromBarRule`), which
/// takes the caret; the bar's box never types. A crumb opens it on its own
/// tab: Spaces on the spaces, the space itself on everything. It shows at
/// every width — with the tabs gone, a wrapped bar's second line has room.
fn whereabouts(s: &BState, query: &BPath) -> BView {
    use crate::modal::command::find::BFindKind;
    let space = s.get(CtxKey::Space).and_then(BSpaceType::from_slug).unwrap_or(BSpaceType::Layout);
    let tab = |kind: BFindKind| BAction::Set(AppKey::CommandTab.path(), kind.slug().to_string());
    let trail = vec![
        (BFindKind::Space.label().to_string(), tab(BFindKind::Space)),
        (space.label().to_string(), tab(BFindKind::All)),
    ];
    BView::Crumbs { trail, edit: vec![BAction::Set(AppKey::Edit.path(), query.encode())] }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::action::apply_action;
    use crate::state::val::V_MODAL_COMMAND;
    use crate::state::ModalKey;

    #[test]
    fn the_top_bar_search_reads_the_space_you_are_in_and_opens_the_palette() {
        let mut s = BState::new();
        s.set(CtxKey::Space, BSpaceType::Shading.slug());
        let BView::Crumbs { trail, edit } = BSearch::new(BHostType::Topbar).build(&s) else { panic!("the top bar's search is crumbs") };
        let names: Vec<&str> = trail.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, ["Spaces", BSpaceType::Shading.label()]);
        for action in &edit {
            apply_action(&mut s, action);
        }
        assert_eq!(s.get(ModalKey::Open), Some(V_MODAL_COMMAND), "a click opens the palette");
        assert!(edit::is_editing(&s, AppKey::CommandQuery), "with the caret in its query");
    }

    #[test]
    fn the_spaces_crumb_opens_the_palette_on_the_spaces() {
        use crate::modal::command::find::BFindKind;
        let mut s = BState::new();
        let BView::Crumbs { trail, edit } = BSearch::new(BHostType::Topbar).build(&s) else { panic!("crumbs") };
        apply_action(&mut s, &trail[0].1);
        for action in &edit {
            apply_action(&mut s, action);
        }
        assert_eq!(BFindKind::of(&s), BFindKind::Space);
        assert_eq!(s.get(ModalKey::Open), Some(V_MODAL_COMMAND));
    }
}
