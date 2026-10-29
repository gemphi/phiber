//! The status bar shows the scene, or shows nothing — never its own labels.
//!
//! The foot of the window read "Scene 1 Stats Memory Version". Three of the
//! five segments had no value, and `unwrap_or(self.spec.label)` printed the
//! segment's NAME in its place, so the app announced the words "Stats",
//! "Memory" and "Version" where Blender shows counts.
//!
//! Two of the three can be answered from what the app already knows: the
//! visible counts, and the build. The third — memory — nothing measures, so
//! it shows nothing, which is what Blender's status bar does with a figure
//! it does not have.
//!
//! The counts come from `scene::stats_pairs`, the same function the
//! viewport's read-out draws, so the corner of the canvas and the foot of
//! the window cannot disagree about how big the scene is. They are three
//! read-outs, each its own `label: value`, not one line of text — and
//! plain text, with no chip under them (`view/draw/pair.rs`).

use phiber::part::statusbar::{BStatusSegment, BStatusbarItem};
use phiber::part::statusbar::set::SEGMENTS;
use phiber::state::{BPath, BState};
use phiber::view::{BView, BViewItem};

fn spec(kind: BStatusSegment) -> &'static phiber::part::statusbar::BStatusSegmentSpec {
    SEGMENTS.iter().find(|s| s.kind == kind).expect("the segment")
}

/// A segment as it reads: a label's text, a pair's `label: value`, a row's
/// read-outs with `|` between them; nothing when it shows nothing.
fn read(view: &BView) -> Option<String> {
    match view {
        BView::Label { text } => Some(text.clone()),
        BView::Pair { label, value } => Some(format!("{label}: {value}")),
        BView::Row { list } => Some(list.iter().filter_map(read).collect::<Vec<_>>().join(" | ")),
        _ => None,
    }
}

fn text(kind: BStatusSegment, state: &BState) -> Option<String> {
    read(&BStatusbarItem::new(*spec(kind)).build(state))
}

#[test]
fn no_segment_ever_prints_its_own_name() {
    // The exact bug: a segment with nothing to say saying what it is called.
    let state = BState::new();
    for segment in SEGMENTS {
        if let Some(shown) = text(segment.kind, &state) {
            assert_ne!(shown, segment.label, "{:?} printed its own label", segment.kind);
        }
    }
}

#[test]
fn a_segment_with_nothing_to_say_says_nothing() {
    // Memory: nothing measures it, so the foot of the window is quiet about
    // it rather than reading "Memory".
    assert_eq!(text(BStatusSegment::Memory, &BState::new()), None);
}

#[test]
fn the_stats_are_the_viewports_own_count() {
    // One function, two read-outs. A second copy here is how the corner of
    // the canvas and the foot of the window come to disagree.
    let state = BState::new();
    let BView::Row { list } = BStatusbarItem::new(*spec(BStatusSegment::Stats)).build(&state) else {
        panic!("the stats are a row of read-outs");
    };
    let shown: Vec<(String, String)> = list
        .iter()
        .map(|view| match view {
            BView::Pair { label, value } => (label.clone(), value.clone()),
            other => panic!("each stat is a label and a value: {other:?}"),
        })
        .collect();
    assert_eq!(shown, phiber::scene::stats_pairs(&state).to_vec());
    let labels: Vec<&str> = shown.iter().map(|(label, _)| label.as_str()).collect();
    assert_eq!(labels, ["Objects", "Verts", "Faces"], "Blender's wording");
    assert_eq!(text(BStatusSegment::Stats, &state), Some(phiber::scene::stats_line(&state)));
}

#[test]
fn the_version_is_the_build() {
    let shown = text(BStatusSegment::Version, &BState::new()).expect("the version segment");
    assert_eq!(shown, format!("v{}", env!("CARGO_PKG_VERSION")));
    assert!(shown.starts_with('v'), "Blender ends its status bar with vX.Y.Z");
}

#[test]
fn the_stats_are_counted_live_not_written() {
    // A file's load writes its stats line (`app::seed`), but the bar counts
    // what the view layer shows now — the same as the viewport's corner —
    // so a written line, or an empty one, does not stand in for the count.
    for written in ["Objects: 9 | Verts: 9 | Faces: 9", ""] {
        let mut state = BState::new();
        state.set(&BPath::owned(vec!["stats"]), written);
        assert_eq!(text(BStatusSegment::Stats, &state), Some(phiber::scene::stats_line(&state)), "written {written:?}");
    }
}
