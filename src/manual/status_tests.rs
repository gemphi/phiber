//! `status.rs`'s tests: the classifications, and the doc itself, which
//! fails here when the code moved on without it.

use super::*;
use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        match (path.is_dir(), path.extension().and_then(|e| e.to_str())) {
            (true, _) => rust_files(&path, out),
            (false, Some("rs")) => out.push(path),
            (false, _) => {}
        }
    }
}

/// The files that read a Properties key: they name it, quoted, beside a
/// `ButsKey` or the panel's own specs — `scene::overlay` finds Show Gizmo
/// in `props::tool::OPTIONS` and reads `spec.value_key()`, with no
/// `ButsKey` in sight. The panel specs and their seeding (`part/props`), the
/// file seeding (`app/seed`) and tests write keys rather than read them, so
/// they are left out.
fn readers() -> impl Fn(&str) -> Vec<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    let skipped = [root.join("src/part/props"), root.join("src/app/seed")];
    let sources: Vec<(String, String)> = files
        .into_iter()
        .filter(|path| !skipped.iter().any(|dir| path.starts_with(dir)))
        .filter(|path| path.file_name().and_then(|n| n.to_str()).is_some_and(|n| !n.ends_with("tests.rs")))
        .filter_map(|path| {
            let text = std::fs::read_to_string(&path).ok()?;
            let shown = path.strip_prefix(&root).ok()?.display().to_string().replace('\\', "/");
            ["ButsKey", "value_key(", "part::props::"].iter().any(|mark| text.contains(mark)).then_some((shown, text))
        })
        .collect();
    move |key: &str| -> Vec<String> { sources.iter().filter(|(_, text)| text.contains(&format!("\"{key}\""))).map(|(path, _)| path.clone()).take(2).collect() }
}

static SAMPLE: &[BMenuItem] = &[
    crate::part::menubar::item::set::op("Revert", ""),
    crate::part::menubar::item::set::op_url("Manual", "https://docs.blender.org"),
    BMenuItem { label: "", shortcut: "", kind: BMenuKind::Sep, modal: "", purpose: "", url: "" },
    BMenuItem { label: "Export", shortcut: "", kind: BMenuKind::Sub(SUB), modal: "", purpose: "", url: "" },
];
static SUB: &[BMenuItem] = &[crate::part::menubar::item::set::op_export("glTF 2.0", "gltf")];

#[test]
fn an_op_with_nothing_to_open_is_not_available_and_a_submenu_is_walked() {
    let mut out = Vec::new();
    entries(SAMPLE, "File", &mut out);
    let got: Vec<(&str, BEntryStatus)> = out.iter().map(|e| (e.path.as_str(), e.status)).collect();
    assert_eq!(
        got,
        vec![("File ▸ Revert", BEntryStatus::NotYet), ("File ▸ Manual", BEntryStatus::Wired), ("File ▸ Export ▸ glTF 2.0", BEntryStatus::Wired)],
        "the separator is left out"
    );
}

#[test]
fn a_field_a_rule_takes_applies_and_one_nothing_reads_is_shown_only() {
    let spec = |key: &'static str| BPropSpec { key, label: key, kind: crate::part::props::prop::BPropKind::Bool, default: None };
    let none = |_: &str| -> Vec<String> { Vec::new() };
    // The Shadow switches have their rule (`scene::shadow`).
    assert_eq!(field("Object", &spec("show_shadows"), &none), BFieldStatus::Applies);
    assert_eq!(field("Object", &spec("no_such_field"), &none), BFieldStatus::ShownOnly);
    let found = |_: &str| -> Vec<String> { vec!["src/scene/overlay.rs".to_string()] };
    assert_eq!(field("Object", &spec("no_such_field"), &found), BFieldStatus::Read(vec!["src/scene/overlay.rs".to_string()]));
    assert!(matches!(field("Render", &spec("cycles.samples"), &none), BFieldStatus::Render(_)), "the Render tab is the audit's");
}

/// `docs/status/STATUS.md` is what the code says now.
#[test]
fn the_status_doc_is_current() {
    let doc = markdown(&readers());
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/status/STATUS.md");
    match std::env::var_os("UPDATE_STATUS_DOC") {
        Some(_) => {
            std::fs::create_dir_all(path.parent().expect("docs/status")).expect("docs/status");
            std::fs::write(&path, &doc).expect("write STATUS.md");
        }
        None => {
            let kept = std::fs::read_to_string(&path).unwrap_or_default().replace("\r\n", "\n");
            assert!(kept == doc, "docs/status/STATUS.md is stale: run UPDATE_STATUS_DOC=1 cargo test -p phiber --lib manual::status");
        }
    }
}
