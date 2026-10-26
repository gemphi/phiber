//! Tests for `op::render` — split out so the module itself sits under the
//! 240-line read cap (`tests/level_structure.rs`).

use super::show_render_result;
use crate::area::current::{active_host, area_key};
use crate::area::BAreaType;
use crate::host::BHostType;
use crate::space::BSpaceType;
use crate::state::BState;

fn center() -> &'static crate::area::BAreaSpec {
    BSpaceType::Rendering.spec().areas.iter().find(|a| a.kind == BAreaType::Center).unwrap()
}

#[test]
fn a_sequencer_left_in_the_rendering_center_gives_way_to_the_image_editor() {
    let mut s = BState::new();
    s.set(&area_key(BSpaceType::Rendering, center()), BHostType::Seq.slug());
    assert_eq!(active_host(BSpaceType::Rendering, center(), |k| s.get(k).map(str::to_string)), BHostType::Seq);
    s.set(crate::state::ImageKey::Shown, "Untitled.001"); // a leftover New image, resumed
    assert!(show_render_result(&mut s), "the area was converted");
    assert_eq!(active_host(BSpaceType::Rendering, center(), |k| s.get(k).map(str::to_string)), BHostType::Image);
    assert_eq!(s.get(crate::state::ImageKey::Shown), Some(crate::part::image::shown::RENDER_RESULT), "and it shows the render, not the leftover");
    // Already an Image editor on the Render Result: nothing to do.
    assert!(!show_render_result(&mut s));
}

#[test]
fn another_workspace_s_center_is_not_touched() {
    // The key is namespaced per space: Layout's Center keeps its Sequencer.
    let mut s = BState::new();
    let layout_center = BSpaceType::Layout.spec().areas.iter().find(|a| a.kind == BAreaType::Center).unwrap();
    s.set(&area_key(BSpaceType::Layout, layout_center), BHostType::Seq.slug());
    show_render_result(&mut s);
    assert_eq!(active_host(BSpaceType::Layout, layout_center, |k| s.get(k).map(str::to_string)), BHostType::Seq);
}
