//! A path as breadcrumbs — Explorer's address bar: This PC › Local Disk
//! (C:) › Users › …, each folder a link to itself, in the field's own box.
//! A click on the box between the crumbs puts the caret in the path, which
//! is then typed as the directory field is (`file::path_bar`).
//!
//! The top bar's search reads the same way while idle — `Spaces › Layout`,
//! where the window stands (`part::search`) — led by the magnifier, since
//! the box is a search: a click anywhere on it, a crumb included, opens the
//! palette.

use super::ctx::BRenderCtx;
use super::field;
use super::flex::{BAlign, BFill, BFlex, BFlexed, BLen, BOverflow, BRadius, BSpace, BType};
use crate::host::BHostType;
use crate::theme::BStyleRole;
use crate::view::prim::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString};
use std::rc::Rc;

/// The bar's own containers, named once.
enum BCrumbPart {
    /// The magnifier and the trail after it, in the search's box.
    Find,
    /// The crumbs in a line, clipped from the start's side when the path is
    /// longer than the box.
    Trail,
    /// One folder: its name on a hover ground, the field's corner.
    Crumb,
}

impl BFlexed for BCrumbPart {
    fn flex(&self, _ctx: &BRenderCtx) -> BFlex {
        match self {
            Self::Find => BFlex::row().align(BAlign::Center).gap(BSpace::GapSm).grows().min_width(BLen::Zero),
            Self::Trail => BFlex::row().align(BAlign::Center).gap(BSpace::GapXs).grows().min_width(BLen::Zero).overflow(BOverflow::Clip),
            Self::Crumb => BFlex::row()
                .align(BAlign::Center)
                .pad_x(BSpace::PadXs)
                .radius(BRadius::Role(BStyleRole::Field))
                .keeps_size()
                .hovered(BFill::ItemHover)
                .type_size(BType::Body)
                .ink(),
        }
    }
}

/// What a trail is for, by the header it stands in.
#[derive(Clone, Copy)]
enum BCrumbRole {
    /// A folder path: a crumb walks to its folder, and the box types.
    Path,
    /// Where the window stands, in the top bar's search: a crumb does its
    /// own part (which palette tab) and then what the box does (open it).
    Find,
}

impl BCrumbRole {
    fn of(ctx: &BRenderCtx) -> Self {
        match ctx.host {
            BHostType::Topbar => Self::Find,
            _ => Self::Path,
        }
    }

    /// The box's selector — `search…` for the top bar's, so it is measured
    /// as the search it is (`tests/header_controls_never_overlap.rs`).
    fn id(self) -> &'static str {
        match self {
            Self::Path => "crumbs",
            Self::Find => "search-crumbs",
        }
    }

    /// A crumb's selector, kept apart from a folder of the same name.
    fn crumb_id(self, name: &str) -> String {
        match self {
            Self::Path => format!("crumb-{name}"),
            Self::Find => format!("crumb-find-{name}"),
        }
    }

    /// What a crumb runs after its own action: nothing for a folder; for a
    /// search crumb, the box's own click.
    fn then(self, edit: &[BAction]) -> Vec<BAction> {
        match self {
            Self::Path => Vec::new(),
            Self::Find => edit.to_vec(),
        }
    }
}

pub fn view(trail: &[(String, BAction)], edit: &[BAction], ctx: &BRenderCtx) -> AnyElement {
    let role = BCrumbRole::of(ctx);
    let mut line = BCrumbPart::Trail.flex(ctx).div(ctx);
    for (n, (name, action)) in trail.iter().enumerate() {
        if n > 0 {
            line = line.child(super::atom::icon_in("chevron_right", ctx.style.colors.text_dim, ctx));
        }
        let handler = ctx.handler.clone();
        let actions: Vec<BAction> = std::iter::once(action.clone()).chain(role.then(edit)).collect();
        let id = role.crumb_id(name);
        line = line.child(
            BCrumbPart::Crumb
                .flex(ctx)
                .div(ctx)
                .id(SharedString::from(id.clone()))
                .debug_selector(move || id)
                .cursor_pointer()
                .child(SharedString::from(name.clone()))
                .on_click(move |_: &ClickEvent, window, app| {
                    // The crumb's own, not the box's under it as well.
                    app.stop_propagation();
                    for action in &actions {
                        (Rc::clone(&handler))(action, window, app);
                    }
                }),
        );
    }
    let body = match role {
        BCrumbRole::Path => line.into_any_element(),
        BCrumbRole::Find => BCrumbPart::Find
            .flex(ctx)
            .div(ctx)
            .child(super::search::lead_icon(ctx.style.colors.text_dim, ctx))
            .child(line)
            .into_any_element(),
    };
    let handler = ctx.handler.clone();
    let edit = edit.to_vec();
    let id = role.id();
    field::shell(false, ctx)
        .id(SharedString::from(id))
        .debug_selector(move || id.to_string())
        .cursor_text()
        .child(body)
        .on_click(move |_: &ClickEvent, window, app| {
            for action in &edit {
                (Rc::clone(&handler))(action, window, app);
            }
        })
        .into_any_element()
}
