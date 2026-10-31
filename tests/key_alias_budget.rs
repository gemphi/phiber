//! A key is named, not aliased.
//!
//! `tests/key_budget.rs` keeps a moved domain's path from being hand-spelt
//! outside its `state/key/` file. It greps for the path, so it cannot see a
//! path wrapped in a function — and that is exactly how the rule got dodged:
//! `scene/key.rs` grew fifteen `pub fn <name>_key() -> BPath` functions that
//! returned what their domain enum already returned, so `loading_key()` and
//! `SceneKey::Loading.path()` were two spellings of one key and the grep was
//! blind to the first. Six of them were being called 80+ times.
//!
//! This test closes that hole. It finds every **zero-argument** function
//! returning a `BPath` outside `state/key/` and asserts the set has not
//! grown. Zero-argument is the whole tell: a function that takes the slot,
//! the host or the row it is keying (`BViewSlot::path(self, field)`,
//! `area::area_key(space, area)`) is composing a path from data the enum
//! cannot know, which is real work. One that takes nothing can only be
//! returning a constant — the enum variant, said twice.
//!
//! The list below is the set as found, each entry marked with why it is
//! there. It may shrink; an addition fails the test. To remove an entry,
//! delete the function and let its callers name the key: `BState::get / set /
//! has / remove` take `impl BKey`, so `s.get(&loading_key())` becomes
//! `s.get(SceneKey::Loading)` and gets shorter doing it.

use std::path::{Path, PathBuf};

/// `ADAPTER` — structurally required, not debt.
///
/// The menus hold a *function pointer*: `BMenuKind::Toggle(fn() -> BPath,
/// bool)` and `Act(fn() -> BAction)` (`part/menubar/item/mod.rs`). A
/// `fn() -> BPath` is the only thing that can be stored there, so an entry
/// that flips a fixed key needs a named function to point at. These cannot
/// be inlined and should not be.
///
/// `ALIAS` — debt. Called directly, returns `SomeKey::Variant.path()` and
/// nothing else. Removable the way the `scene/key.rs` ones were.
///
/// `NO_ENUM` — worse debt. Spells a path with `BPath::owned(vec![…])`
/// because its domain has **no key enum at all**. `key_budget.rs`'s `MOVED`
/// table cannot cover these domains until one exists, so they are invisible
/// to both rulers. These want a `state/key/` file, not just inlining.
const ALLOWED: &[&str] = &[
    // ── ADAPTER: menu `fn` pointers ──
    "part/menubar/editor/anim/set.rs::dope_sheet_sidebar",
    "part/menubar/editor/set.rs::k_auto_save_prefs",
    "part/menubar/editor/set.rs::k_markers_locked",
    "part/menubar/topbar/set.rs::k_lock_interface",
    "part/menubar/topbar/set.rs::k_status_bar",
    "part/sidebar/show.rs::view3d_key",
    "part/toolbar/mod.rs::view3d_key",
    // ── ALIAS: one variant each, called directly ──
    "area/maximize/mod.rs::key",
    "part/statuslog/mod.rs::ascending_key",
    "part/statuslog/mod.rs::order_key",
    "scene/orbit_around.rs::key",
    "scene/shading.rs::shading_key",
    "scene/viewport_aa.rs::key",
    "view/draw/canvas/node/add/rule.rs::added",
    // `panel` and `panel_path` are the SAME path twice in one file
    // (`MenuKey::Panel(shader::SLUG, "Add")`) — an alias that already
    // duplicated itself, which is the failure mode this test is about.
    "view/draw/canvas/node/add/rule.rs::panel",
    "view/draw/canvas/node/add/rule.rs::panel_path",
    "view/draw/canvas/view3d/corner.rs::k_corner",
];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("dir").flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// `fn name() -> BPath` — the name, for each zero-argument path function the
/// line declares. Deliberately textual: the point is to catch the shape as
/// it is written, the same way `key_budget.rs` catches a spelt path.
fn zero_arg_path_fn(line: &str) -> Option<&str> {
    let line = line.trim_start();
    let rest = line.strip_prefix("pub ").unwrap_or(line);
    // `pub(super) fn`, `pub(in crate::x) fn`
    let rest = match rest.starts_with("pub(") {
        true => rest.split_once(") ").map(|(_, tail)| tail)?,
        false => rest,
    };
    let rest = rest.strip_prefix("fn ")?;
    let (name, tail) = rest.split_once('(')?;
    let tail = tail.strip_prefix(')')?;
    let tail = tail.trim_start().strip_prefix("->")?.trim_start();
    let returns_path = tail.starts_with("BPath") || tail.starts_with("crate::state::BPath");
    returns_path.then_some(name.trim()).filter(|n| !n.is_empty())
}

#[test]
fn no_new_function_stands_in_for_a_key() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    files.sort();

    let mut found: Vec<String> = Vec::new();
    for file in files {
        let rel = file.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
        // The key enums' own `impl BKey { fn path(&self) }` is the one place
        // a path is meant to be spelt.
        if rel.starts_with("state/key/") {
            continue;
        }
        let src = std::fs::read_to_string(&file).unwrap();
        for line in src.lines() {
            if let Some(name) = zero_arg_path_fn(line) {
                found.push(format!("{rel}::{name}"));
            }
        }
    }
    found.sort();

    let added: Vec<&String> = found.iter().filter(|f| !ALLOWED.contains(&f.as_str())).collect();
    assert!(
        added.is_empty(),
        "a function is standing in for a key — name the key instead (see this file's \
         header; `BState::get/set/has/remove` take `impl BKey`, so no `.path()` is \
         needed at the call site):\n  {}",
        added.iter().map(|f| f.as_str()).collect::<Vec<_>>().join("\n  ")
    );

    // The list may only shrink. A stale entry means the cleanup happened and
    // the budget was not tightened after it, which is how a budget rots.
    let gone: Vec<&&str> = ALLOWED.iter().filter(|a| !found.iter().any(|f| f == *a)).collect();
    assert!(
        gone.is_empty(),
        "these are gone — drop them from ALLOWED so the budget keeps its grip:\n  {}",
        gone.iter().map(|a| **a).collect::<Vec<_>>().join("\n  ")
    );
}

/// The shape-reader itself, since the whole test rests on it.
#[test]
fn the_reader_knows_a_zero_argument_path_function_from_a_builder() {
    assert_eq!(zero_arg_path_fn("pub fn zoom_key() -> BPath {"), Some("zoom_key"));
    assert_eq!(zero_arg_path_fn("    fn key() -> BPath {"), Some("key"));
    assert_eq!(zero_arg_path_fn("pub fn k_corner() -> crate::state::BPath {"), Some("k_corner"));
    assert_eq!(zero_arg_path_fn("    pub(super) fn loaded_path() -> BPath {"), Some("loaded_path"));
    // Takes something: it is composing, not aliasing.
    assert_eq!(zero_arg_path_fn("pub fn path(self, field: View3dKey) -> BPath {"), None);
    assert_eq!(zero_arg_path_fn("pub fn key(field: View3dKey) -> BPath {"), None);
    assert_eq!(zero_arg_path_fn("pub fn area_key(space: BSpaceType, area: &str) -> BPath {"), None);
    // Returns something else.
    assert_eq!(zero_arg_path_fn("pub fn label() -> String {"), None);
    assert_eq!(zero_arg_path_fn("pub fn key() -> Option<BPath> {"), None);
    assert_eq!(zero_arg_path_fn("// pub fn zoom_key() -> BPath {"), None);
}
