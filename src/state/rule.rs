//! Rules: what else changes when a key is set.
//!
//! A `Set` writes one key. Some keys mean more than their value — a named
//! viewpoint is two angles, a frame step is a new frame, a path typed into
//! the file browser is a trip through its history. Each of those meanings
//! is a [`BRule`] that lives with the level it belongs to (the viewpoint
//! with the scene, the step with the timeline, the history with the file
//! browser), and this file is only the table that runs them.
//!
//! Rules run once per `Set`, in table order, after the value is stored.
//! What a rule writes does not trigger other rules: a rule that needs a
//! follow-on writes it itself, so the chain is readable in one place.

use super::{BPath, BState};

/// One follow-up of a `Set`.
pub trait BRule: Sync {
    /// Whether this rule runs when `path` is set.
    fn wants(&self, path: &BPath) -> bool;
    /// Runs after `path` was set to `value`.
    fn after(&self, path: &BPath, value: &str, s: &mut BState);
}

/// Every rule in the app, in the order they run.
fn table() -> &'static [&'static dyn BRule] {
    use crate::modal::{reader::back, rename, workspace::pick};
    use crate::part::activity::choose;
    use crate::part::menubar::op as menu;
    use crate::part::outliner::op as outliner;
    use crate::part::{file::op, pref::rule as pref, props::active, props::fold, time::range, time::step};
    use crate::scene::{edit, marker, select, view_op, viewpoint};
    use crate::space::step as space;
    &[
        &viewpoint::BViewpointRule,
        &pref::BThemePickRule,
        &pref::BFactoryRule,
        &crate::part::pref::device::BRenderDeviceRule,
        &choose::BActivityTabRule,
        &choose::BActivityPickRule,
        &step::BFrameStepRule,
        &step::BFrameLockRule,
        &step::BPlayFromRule,
        &range::BFrameRangeRule,
        &active::BActiveObjectRule,
        &select::BPickSelectsRule,
        &crate::part::outliner::reveal::BOutlinerRevealRule,
        &select::BSelectOpRule,
        &edit::BSceneEditRule,
        &crate::scene::cursor::BCursorRule,
        &view_op::BView3dOpRule,
        &marker::BMarkerOpRule,
        &rename::BRenameOpenRule,
        &rename::BRenameRule,
        &outliner::BOutlinerOpRule,
        &outliner::BHideDeselectsRule,
        &crate::part::outliner::drop::BOutlinerDropRule,
        &crate::scene::action::rule::BActionOpRule,
        &crate::scene::action::keying::BKeyframeRule,
        &crate::scene::action::dope::BKeyOpRule,
        &crate::scene::action::nla_editor::BNlaEditRule,
        &crate::part::modebar::workspace::BWorkspaceModeRule,
        &crate::scene::transform::BTransformRule,
        &crate::scene::shadow::BObjectShadowRule,
        &crate::scene::recolor::BRecolorRule,
        &crate::part::sidebar::show::BSidebarOpenRule,
        &crate::scene::xform::BTransformOpRule,
        &crate::scene::transform::BObjectClearRule,
        &crate::scene::parent::BObjectParentRule,
        &crate::area::switch::BEditorTypeRule,
        &crate::scene::action::preview::BPreviewRangeRule,
        &fold::BPropsFoldRule,
        &space::BSpaceStepRule,
        &crate::area::maximize::BAreaMaximizeRule,
        &menu::BReportRule,
        &menu::BFlipRule,
        &crate::modal::command::palette::BPaletteDismissRule,
        &crate::modal::command::palette::BPaletteOpenRule,
        &crate::modal::command::find::BFindGoRule,
        &crate::modal::command::find::BFindFromBarRule,
        &crate::view::draw::canvas::image::zoom::BImageZoomStepRule,
        &crate::scene::render_slot::BRenderSlotRule,
        &crate::scene::render_border::BRenderBorderRule,
        &crate::part::image::shown::BImageOpRule,
        &crate::modal::new_image::BNewImageRule,
        &crate::view::draw::canvas::time::view::op::BTimeViewRule,
        &crate::scene::action::channel::BChannelOpRule,
        &crate::view::draw::canvas::node::view::op::BNodeViewRule,
        &crate::view::draw::canvas::node::select::op::BNodeSelectRule,
        &crate::view::draw::canvas::node::remove::op::BNodeDeleteRule,
        &crate::view::draw::canvas::node::add::duplicate::BNodeDuplicateRule,
        &crate::view::draw::canvas::node::add::rule::BAddLinkClearRule,
        &back::BReaderOpenRule,
        &back::BReaderCloseRule,
        &pick::BSpacePickedRule,
        &op::BSortRule,
        &op::BNavRule,
        &op::BPurposeRule,
        &op::BFileReturnRule,
        &op::BBookmarkAddRule,
        &op::BSelectRule,
        &op::BPickRule,
    ]
}

/// Whether any rule takes a write to `path` — what the generated status doc
/// asks of every Properties field (`manual::status`).
pub fn wanted(path: &BPath) -> bool {
    table().iter().any(|rule| rule.wants(path))
}

/// Run every rule that wants `path`.
pub fn after_set(state: &mut BState, path: &BPath, value: &str) {
    for rule in table() {
        if rule.wants(path) {
            rule.after(path, value, state);
        }
    }
}
