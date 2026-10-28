//! The Project Browser's two side columns as one kind of thing: sections of
//! an accordion, each with its icon on its heading, folding open or shut
//! (`ProjectKey::Open`). The navigator at the left and the inspector at the
//! right are built from the one [`section`], as a sidebar's panels are, so
//! neither column is a layout of its own.
//!
//! The details are the host's sidebar (`sidebar::set`, the Item tab): the
//! card picked as Info (what it is), Stats (its counts), Owner and Pricing,
//! then what it uses and what uses it, each a label/value row on the
//! property split (`BView::value_row`); else the prompt.

use super::{db, library};
use crate::state::val::V_FALSE;
use crate::state::{BKey as _, BState, ProjectKey};
use crate::view::prim::{BCard, BColumn};
use crate::view::{BView, BViewItem};
use phiecs::prelude::Word;

/// One section of a side column: its heading's icon and word, folding as
/// `ProjectKey::Open(slug)` says, open unless shut.
pub fn section(s: &BState, label: impl Into<String>, icon: &str, slug: &'static str, rows: Vec<BView>) -> BView {
    let open = s.get(ProjectKey::Open(slug)) != Some(V_FALSE);
    BCard::side(label, icon).fold(ProjectKey::Open(slug).path(), open).body(rows).into()
}

/// A side column of sections.
pub fn column(sections: Vec<BView>) -> BView {
    BView::from(BCard::column(BColumn::Sidebar, sections))
}

/// Who holds the asset and on what terms. A scene's assets are the open
/// file's, held here and not offered anywhere, until the project carries an
/// owner and a price of its own.
fn owner() -> Vec<BView> {
    vec![BView::value_row("Owner", "You"), BView::value_row("Access", "Private")]
}

fn pricing() -> Vec<BView> {
    vec![BView::value_row("Price", "Not listed"), BView::value_row("License", "All rights reserved")]
}

/// The sidebar's sections for the picked asset (`sidebar::set`, the
/// Project Browser's Item tab): a design from the Library, or a scene asset;
/// else the prompt.
pub fn sections(s: &BState) -> Vec<BView> {
    if let Some(sections) = library::sections(s) {
        return sections;
    }
    let found = db::current().and_then(|db| {
        let spelling = s.get(ProjectKey::Selected)?;
        let w = db.world.words().iter().copied().find(|&w| db.world.spelling(w) == spelling)?;
        Some((db, w))
    });
    let Some((db, w)) = found else {
        return vec![BView::label("Select a file to view its properties.")];
    };
    let Some(a) = db.asset(w) else { return Vec::new() };
    let info = vec![BView::value_row("Name", a.name.clone()), BView::value_row("Kind", a.kind.label().trim_end_matches('s'))];
    let stats: Vec<BView> = a.info.iter().map(|(k, v)| BView::value_row(k.clone(), v.clone())).collect();
    let names = |ws: Vec<Word>| ws.iter().filter_map(|&u| db.asset(u)).map(|u| u.name.clone()).collect::<Vec<_>>();
    let mut sections = vec![section(s, "Info", a.kind.icon(), "info", info)];
    if !stats.is_empty() {
        sections.push(section(s, "Stats", "bar_chart", "stats", stats));
    }
    sections.push(section(s, "Owner", "user", "owner", owner()));
    sections.push(section(s, "Pricing", "tag", "pricing", pricing()));
    let uses = names(db.uses_of(w));
    if !uses.is_empty() {
        sections.push(section(s, format!("Uses ({})", uses.len()), "link", "uses", uses.into_iter().take(24).map(BView::label).collect()));
    }
    let users = names(db.used_by(w));
    if !users.is_empty() {
        sections.push(section(s, format!("Used by ({})", users.len()), "link", "used_by", users.into_iter().take(24).map(BView::label).collect()));
    }
    sections
}

/// The details as a column of their own, where a host lays them out as one
/// rather than in its sidebar.
#[derive(Clone, Copy, Debug)]
pub struct BProjectInfo;

impl BViewItem for BProjectInfo {
    fn build(&self, s: &BState) -> BView {
        column(sections(s))
    }
}
