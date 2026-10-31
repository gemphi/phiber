//! What a channel row paints in the window — `draw_backdrops` and
//! `draw_keyframes` in `action_draw.cc` — in the theme's colours
//! (`BAnimColors`). The channels' names are a side column beside it
//! (`part::channels::time`).

use super::super::set::{hsla, rect};
use super::{BRowKind, BTimeRow};
use crate::theme::BAnimColors;
use gpui::{Bounds, PathBuilder, Pixels, Window, fill, point, px};

/// `channel_ui_data_init`: `icon_size = widget_unit / 2`.
const ICON_SIZE: f32 = super::super::WIDGET_UNIT * 0.5;
/// The backdrops, then the diamonds, of every Dope Sheet row, in the
/// window `at` (`x_of` maps a frame to a pixel across it); a picked column
/// (`picked(row, frame)`) in `TH_KEYTYPE_KEYFRAME_SELECT`. The NLA's rows
/// are `strip/set.rs`.
/// `(height, scrub)` is the rows' geometry: their height and the band above
/// them (`BTimeView::row_height`, `scrub_height`).
pub fn paint_rows(rows: &[BTimeRow], colors: &BAnimColors, at: Bounds<Pixels>, x_of: &dyn Fn(f32) -> f32, picked: &dyn Fn(usize, f32) -> bool, (height, scrub): (f32, f32), window: &mut Window) {
    let width = f32::from(at.size.width);
    for (index, row) in rows.iter().enumerate() {
        let (top, center) = (BTimeRow::top(index, height, scrub), BTimeRow::center(index, height, scrub));
        let back = match &row.kind {
            BRowKind::Summary { expanded: true } => Some(colors.summary_back),
            BRowKind::Summary { expanded: false } => None,
            BRowKind::Object { .. } | BRowKind::Slot { .. } => Some(colors.object_back),
            BRowKind::Group { .. } => Some(colors.group_back),
            BRowKind::Curve { .. } => Some(colors.curve_back),
            BRowKind::Action { .. } | BRowKind::Track(_) | BRowKind::Driver { .. } => None,
        };
        if let Some(back) = back {
            window.paint_quad(fill(rect(0.0, top, width, top + height, at), hsla(back)));
        }
        // The row's keys in one stroke per (Keyframe Type, picked) pair —
        // a border path and a fill path each — not two paths a key: a rig's
        // rows hold hundreds of keys and each path is tessellated (Blender
        // batches its diamonds in one instanced draw, `draw_keyframe_shape`).
        // Nearly every row is all plain keyframes, so this is the two
        // strokes it has always been; a row that mixes types pays one pair
        // per type it actually holds.
        let shown: Vec<(f32, u8, bool)> = row
            .keys
            .iter()
            .enumerate()
            .map(|(i, &f)| (x_of(f).round() + 0.5, row.keytype_at(i), picked(index, f)))
            .filter(|(x, _, _)| *x > -ICON_SIZE && *x < width + ICON_SIZE)
            .collect();
        let half = ICON_SIZE / 2.0;
        // The pairs present, in the order they first appear.
        let mut kinds: Vec<(u8, bool)> = Vec::new();
        for &(_, keytype, pick) in &shown {
            if !kinds.contains(&(keytype, pick)) {
                kinds.push((keytype, pick));
            }
        }
        for (keytype, pick) in kinds {
            // `draw_keyframe_shape` scales the diamond by its type: an
            // extreme is larger, a generated key smaller.
            let r = half * BAnimColors::keytype_scale(keytype);
            let xs = || shown.iter().filter(|(_, t, p)| *t == keytype && *p == pick).map(|(x, _, _)| *x);
            diamonds(xs(), center, r, colors.key_border, at, window);
            diamonds(xs(), center, r - 1.0, colors.of_keytype(keytype, pick), at, window);
        }
    }
}

/// One path of diamonds `r` half-wide at each `x` on the line `y`, in one
/// colour.
fn diamonds(xs: impl Iterator<Item = f32>, y: f32, r: f32, color: crate::theme::BColor, at: Bounds<Pixels>, window: &mut Window) {
    let mut path = PathBuilder::fill();
    let mut any = false;
    for x in xs {
        let corner = |dx: f32, dy: f32| point(at.origin.x + px(x + dx), at.origin.y + px(y + dy));
        path.move_to(corner(0.0, -r));
        path.line_to(corner(r, 0.0));
        path.line_to(corner(0.0, r));
        path.line_to(corner(-r, 0.0));
        path.close();
        any = true;
    }
    if let (true, Ok(path)) = (any, path.build()) {
        window.paint_path(path, hsla(color));
    }
}

/// `draw_keyframe_shape`: a diamond `size` across, its border a pixel,
/// filled with `inner` — the key colour, or the band behind it for an
/// outline-only `KEYFRAME_SHAPE_FRAME`.
pub fn diamond(x: f32, y: f32, size: f32, inner: crate::theme::BColor, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window) {
    let half = size / 2.0;
    for (r, color) in [(half, colors.key_border), (half - 1.0, inner)] {
        let corner = |dx: f32, dy: f32| point(at.origin.x + px(x + dx), at.origin.y + px(y + dy));
        let mut path = PathBuilder::fill();
        path.move_to(corner(0.0, -r));
        path.line_to(corner(r, 0.0));
        path.line_to(corner(0.0, r));
        path.line_to(corner(-r, 0.0));
        path.close();
        if let Ok(path) = path.build() {
            window.paint_path(path, hsla(color));
        }
    }
}
