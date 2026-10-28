//! Phiber's status against Blender, written from Phiber's own sets so it
//! cannot fall behind the code: every editor's menus as `part::menubar`
//! declares them, every Properties field as `part::props` declares it, and
//! the Render tab as `scene::render_audit` judges it. Grouped by component:
//! each editor once, with the spaces that show it.
//!
//! The status docs written by hand went stale within a week of each other
//! (`docs/space-status.md` still called the Graph Editor unstarted after it
//! was editing keys). This one is `docs/status/STATUS.md`; `status_tests.rs`
//! fails when it is stale and writes it again under `UPDATE_STATUS_DOC=1`.

use crate::host::BHostType;
use crate::part::menubar::item::{BMenuItem, BMenuKind, BMenuSpec};
use crate::part::props::prop::BPropSpec;
use crate::scene::render_audit::BAudited;
use crate::space::BSpaceType;
use crate::state::{BKey as _, ButsKey};
use std::fmt::Write as _;

/// What a menu entry does when it is picked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BEntryStatus {
    /// It runs something: an operator, a toggle, a modal, a live list.
    Wired,
    /// It reports "not available yet" (`menubar/op.rs`).
    NotYet,
}

/// What a Properties field does when it is edited.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BFieldStatus {
    /// A rule takes the edit (`state::rule`).
    Applies,
    /// Code beside the panel reads it: the files that do.
    Read(Vec<String>),
    /// The Render tab's own verdict (`render_audit::BAudited`).
    Render(BAudited),
    /// Nothing reads it: the field shows a value and an edit changes nothing.
    ShownOnly,
}

/// One menu entry: where it sits, its shortcut, what it does.
pub struct BEntry {
    pub path: String,
    pub shortcut: &'static str,
    pub status: BEntryStatus,
}

/// Every entry of `items`, a submenu's under its own name.
pub fn entries(items: &'static [BMenuItem], trail: &str, out: &mut Vec<BEntry>) {
    for item in items {
        let path = format!("{trail} ▸ {}", item.label);
        let status = match item.kind {
            BMenuKind::Sep => continue,
            BMenuKind::Sub(list) => {
                entries(list, &path, out);
                continue;
            }
            // An `Op` opens what its modal, purpose or link say; with none it
            // only says it is not available yet.
            BMenuKind::Op => match (item.modal, item.purpose, item.url) {
                ("", "", "") => BEntryStatus::NotYet,
                _ => BEntryStatus::Wired,
            },
            BMenuKind::Toggle(..)
            | BMenuKind::HostToggle(..)
            | BMenuKind::Act(_)
            | BMenuKind::Fire(..)
            | BMenuKind::HostFire(..)
            | BMenuKind::Recent
            | BMenuKind::History
            | BMenuKind::Live(_) => BEntryStatus::Wired,
        };
        out.push(BEntry { path, shortcut: item.shortcut, status });
    }
}

/// What editing `spec` on `tab` does. `readers` names the files beside the
/// panel that read a key (the doc's caller scans the sources).
pub fn field(tab: &str, spec: &BPropSpec, readers: &dyn Fn(&str) -> Vec<String>) -> BFieldStatus {
    let ruled = [ButsKey::Value(spec.key).path(), ButsKey::Axis(spec.key, "x").path()].iter().any(crate::state::rule::wanted);
    match (tab == "Render", ruled) {
        (true, _) => BFieldStatus::Render(BAudited::of(spec.key)),
        (false, true) => BFieldStatus::Applies,
        (false, false) => match readers(spec.key) {
            files if files.is_empty() => BFieldStatus::ShownOnly,
            files => BFieldStatus::Read(files),
        },
    }
}

/// The tab-strip spaces that show `host` in one of their areas. An area
/// that can switch to any editor counts only for the one it opens on.
pub fn spaces_showing(host: BHostType) -> Vec<&'static str> {
    BSpaceType::ALL
        .iter()
        .filter(|space| {
            space.spec().areas.iter().any(|area| area.default_host == host || (area.hosts != BHostType::SWITCHABLE && area.hosts.contains(&host)))
        })
        .map(|space| space.label())
        .collect()
}

const LEGEND: &str = "\
| Word | Means |
|---|---|
| Wired | The menu entry runs something. Whether that operation is complete is not checked here. |
| Not available yet | The entry only reports \"not available yet\". |
| Applies | A rule takes the edited value into the scene or the app. |
| Read by | No rule takes it, but the file named reads the field. |
| Shown only | Nothing reads the field: it shows a value and an edit changes nothing. |
| Rendered / Ignored / n/a | The Render tab's own audit: the renderer uses it, does not, or has nothing to use it on. |
";

fn field_words(status: &BFieldStatus) -> (String, String) {
    match status {
        BFieldStatus::Applies => ("Applies".into(), String::new()),
        BFieldStatus::Read(files) => ("Read by".into(), files.iter().map(|f| format!("`{f}`")).collect::<Vec<_>>().join(", ")),
        BFieldStatus::Render(BAudited::Applied) => ("Rendered".into(), String::new()),
        BFieldStatus::Render(BAudited::NotApplicable(why)) => ("n/a".into(), (*why).to_string()),
        BFieldStatus::Render(BAudited::Ignored) => ("Ignored".into(), "the renderer does not read it".into()),
        BFieldStatus::ShownOnly => ("Shown only".into(), String::new()),
    }
}

/// The whole doc, in Markdown.
pub fn markdown(readers: &dyn Fn(&str) -> Vec<String>) -> String {
    let mut doc = String::from(
        "# Phiber status against Blender\n\n\
         Generated from the code by `manual::status`. Do not edit it by hand: after a change, run\n\
         `UPDATE_STATUS_DOC=1 cargo test -p phiber --lib manual::status` to write it again.\n\n",
    );
    doc.push_str(LEGEND);
    let editors: Vec<(BHostType, Vec<&str>, Vec<BEntry>)> = BHostType::ALL
        .iter()
        .map(|&host| {
            let mut list = Vec::new();
            for menu in BMenuSpec::list_for(host) {
                entries(menu.items, menu.label, &mut list);
            }
            (host, spaces_showing(host), list)
        })
        .filter(|(_, _, list)| !list.is_empty())
        .collect();
    let tabs: Vec<(&str, Vec<(&BPropSpec, BFieldStatus)>)> = crate::part::props::set::TABS
        .iter()
        .map(|&tab| {
            let fields = crate::part::props::set::sections_for(BHostType::Buts, tab).iter().flat_map(|s| s.all_props()).map(|p| (p, field(tab, p, readers))).collect();
            (tab, fields)
        })
        .collect();

    doc.push_str("\n## Summary\n\n| Component | Shown in | Entries or fields | Working | Not yet |\n|---|---|---:|---:|---:|\n");
    for (host, spaces, list) in &editors {
        let wired = list.iter().filter(|e| e.status == BEntryStatus::Wired).count();
        let _ = writeln!(doc, "| {} menus | {} | {} | {wired} | {} |", host.label(), spaces.join(", "), list.len(), list.len() - wired);
    }
    for (tab, fields) in &tabs {
        let idle = fields
            .iter()
            .filter(|(_, s)| matches!(s, BFieldStatus::ShownOnly | BFieldStatus::Render(BAudited::Ignored | BAudited::NotApplicable(_))))
            .count();
        let _ = writeln!(doc, "| Properties ▸ {tab} | | {} | {} | {idle} |", fields.len(), fields.len() - idle);
    }

    doc.push_str("\n## Editors\n");
    for (host, spaces, list) in &editors {
        let _ = writeln!(doc, "\n### {}\n\nShown in: {}.\n\n| Menu entry | Shortcut | Status |\n|---|---|---|", host.label(), match spaces.is_empty() {
            true => "a space's switchable area only".to_string(),
            false => spaces.join(", "),
        });
        for e in list {
            let word = match e.status {
                BEntryStatus::Wired => "Wired",
                BEntryStatus::NotYet => "Not available yet",
            };
            let _ = writeln!(doc, "| {} | {} | {word} |", e.path, e.shortcut);
        }
    }

    doc.push_str("\n## Properties\n");
    for (tab, fields) in &tabs {
        let _ = writeln!(doc, "\n### {tab}\n\n| Field | Key | Status | Evidence |\n|---|---|---|---|");
        for (spec, status) in fields {
            let (word, evidence) = field_words(status);
            let _ = writeln!(doc, "| {} | `{}` | {word} | {evidence} |", spec.label, spec.key);
        }
    }
    doc
}

#[cfg(test)]
#[path = "status_tests.rs"]
mod tests;
