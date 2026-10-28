//! Viewport navigation from a scroll gesture — Blender's trackpad and wheel
//! bindings for the 3D View (`blender_default.py`, 3D View ▸ Navigation, with
//! `use_v3d_mmb_pan` off, the default):
//!
//! - `TRACKPADPAN` (a two-finger swipe) **rotates** the view; with Shift it
//!   **moves** (pans) it; with Ctrl it **zooms**. `TRACKPADZOOM` (a pinch)
//!   zooms too — gpui does not surface a pinch, so Ctrl-swipe and the wheel
//!   are a trackpad's zoom here.
//! - `WHEELINMOUSE` / `WHEELOUTMOUSE` **zoom**. The 3D View binds no modifier
//!   variant of the wheel, so Shift / Ctrl leave it zooming.
//!
//! The canvas (`draw::canvas::view3d`) hands each event's device and modifiers
//! to [`scroll_nav`], accumulates the displacement since its last draw, and
//! fires the sets [`scroll_sets`] returns. So a swipe orbits with the same
//! degrees per pixel as a pointer drag ([`BTurntable`]), a pan follows the
//! fingers one pixel for one, and a zoom steps as the wheel always has — one
//! notch per [`PX_PER_LINE`], smoothly across a swipe.
//!
//! A pointer drag over the canvas follows Blender's middle-mouse scheme
//! ([`drag_nav`]): `MIDDLEMOUSE` **rotates**, Shift+`MIDDLEMOUSE` **moves**,
//! Ctrl+`MIDDLEMOUSE` **zooms** (Shift+Ctrl dollies — a zoom here). The
//! middle button does exactly that, and the *left* button does it too, so a
//! laptop without one can; the maths is the scroll's ([`drag_sets`]).
//!
//! An orbit turns about the selection when Orbit Around Selection has a
//! centre for it ([`BNavBase::around`], `orbit_around`), and a pan slides the
//! orbit centre itself, as Blender's `view3d.move` slides `ofs`, so the view
//! keeps orbiting about the middle of what it shows.
//!
//! Deliberate divergences, recorded: Shift+Ctrl+swipe (bound to nothing in
//! Blender) pans here rather than doing nothing; the drag scheme rides the
//! left button as well as the middle.

use super::orbit::BTurntable;
use super::orbit_around::{BOrbitAround, encode, pan_pivot, turn_pivot};
use super::{pan_x_key, pan_y_key, pitch_key, pivot_key, yaw_key, zoom_key, zoom_step};
use crate::state::{BPath, BState};
use crate::view::prim::BAction;
use glam::Vec3;

/// Where a scroll came from, as gpui reports its delta: a trackpad's pixel
/// deltas (`ScrollDelta::Pixels`) or a wheel's line steps (`ScrollDelta::Lines`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BScrollSource {
    Trackpad,
    Wheel,
}

/// What a scroll does to the view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BScrollNav {
    Orbit,
    Pan,
    Zoom,
}

/// Blender's binding for a scroll of `source` under these modifiers.
pub fn scroll_nav(source: BScrollSource, shift: bool, ctrl: bool) -> BScrollNav {
    match (source, shift, ctrl) {
        (BScrollSource::Trackpad, true, _) => BScrollNav::Pan,
        (BScrollSource::Trackpad, false, true) => BScrollNav::Zoom,
        (BScrollSource::Trackpad, false, false) => BScrollNav::Orbit,
        (BScrollSource::Wheel, _, _) => BScrollNav::Zoom,
    }
}

/// One wheel notch, in the pixels a trackpad would have swiped — the zoom's
/// step per notch and a pan's distance per notch both follow from it.
pub const PX_PER_LINE: f32 = 20.0;
/// The zoom factor per [`PX_PER_LINE`] of scroll — one wheel notch.
pub const ZOOM_PER_LINE: f32 = 1.1;

/// The view a gesture starts from, read when the canvas drew.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BNavBase {
    pub yaw: f32,
    pub pitch: f32,
    pub pan: (f32, f32),
    pub zoom: f32,
    /// The view's centre, Blender units (`View3dKey::Pivot`).
    pub pivot: Vec3,
    /// What an orbit turns about instead of the centre: the selection's
    /// centre under Orbit Around Selection (`BOrbitAround::centre`).
    pub around: Option<Vec3>,
}

impl BNavBase {
    /// The view of the viewport asked about now, and what it orbits about.
    pub fn now(s: &BState) -> Self {
        let pivot = Vec3::from(super::camera::view_pivot(s));
        let (pan, zoom) = (super::view_pan(s), super::view_zoom(s));
        Self { yaw: super::view_yaw(s), pitch: super::view_pitch(s), pan, zoom, pivot, around: BOrbitAround::centre(s) }
    }

    /// The view once `leave` has run — the camera view handing its view to
    /// the orbit as a gesture starts (`BCameraView::leave`): its yaw, pitch,
    /// zoom, pivot and pan, where it sets them.
    pub fn after(mut self, leave: &[BAction]) -> Self {
        let set = |key: BPath| leave.iter().find_map(|a| match a {
            BAction::Set(at, value) if *at == key => value.parse::<f32>().ok(),
            _ => None,
        });
        self.yaw = set(yaw_key()).unwrap_or(self.yaw);
        self.pitch = set(pitch_key()).unwrap_or(self.pitch);
        self.zoom = set(zoom_key()).unwrap_or(self.zoom);
        self.pan = (set(pan_x_key()).unwrap_or(self.pan.0), set(pan_y_key()).unwrap_or(self.pan.1));
        let pivot = leave.iter().find_map(|a| match a {
            BAction::Set(at, value) if *at == pivot_key() => Some(value.split(',').filter_map(|p| p.trim().parse::<f32>().ok()).collect::<Vec<_>>()),
            _ => None,
        });
        self.pivot = pivot.filter(|p| p.len() == 3).map_or(self.pivot, |p| Vec3::new(p[0], p[1], p[2]));
        self
    }
}

/// The view keys to set for `nav` after `acc` pixels of scroll `(dx, dy)`
/// (`dy` down) since the base, over a canvas `width` pixels wide.
pub fn scroll_sets(nav: BScrollNav, base: BNavBase, acc: (f32, f32), width: f32) -> Vec<(BPath, String)> {
    match nav {
        // The same turntable as a pointer drag: the swipe is the drag. About
        // the selection, the centre swings round it (`turn_pivot`).
        BScrollNav::Orbit => {
            let (yaw, pitch) = BTurntable::start(base.yaw, base.pitch).at(acc.0, acc.1);
            let mut sets = vec![(yaw_key(), yaw.to_string()), (pitch_key(), pitch.to_string())];
            if let Some(about) = base.around {
                sets.push((pivot_key(), encode(turn_pivot(base.pivot, about, (base.yaw, base.pitch), (yaw, pitch)))));
            }
            sets
        }
        // The slide is in canvas widths (`camera::view_project`), so a pixel
        // of scroll moves the view one pixel — it follows the fingers. It
        // slides the orbit centre, not the picture, and takes any old
        // screen-space pan into it, so the centre is the middle of the view.
        BScrollNav::Pan => {
            let w = width.max(1.0);
            let shift = (base.pan.0 + acc.0 / w, base.pan.1 + acc.1 / w);
            let pivot = pan_pivot(base.pivot, (base.yaw, base.pitch), base.zoom, shift);
            vec![(pivot_key(), encode(pivot)), (pan_x_key(), "0".to_string()), (pan_y_key(), "0".to_string())]
        }
        // A notch is one factor; a swipe steps fractionally, so it is smooth.
        BScrollNav::Zoom => {
            let factor = ZOOM_PER_LINE.powf(acc.1 / PX_PER_LINE);
            vec![(zoom_key(), zoom_step(base.zoom, factor).to_string())]
        }
    }
}

/// What a pointer drag does to the view — Blender's `MIDDLEMOUSE` bindings,
/// on the middle button and on the left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BDragNav {
    Orbit,
    Pan,
    Zoom,
}

/// Blender's binding for a drag under these modifiers: `view3d.rotate`,
/// Shift → `view3d.move`, Ctrl → `view3d.zoom`, Shift+Ctrl → `view3d.dolly`
/// (a zoom here, the view having no separate dolly).
pub fn drag_nav(shift: bool, ctrl: bool) -> BDragNav {
    match (shift, ctrl) {
        (true, false) => BDragNav::Pan,
        (_, true) => BDragNav::Zoom,
        (false, false) => BDragNav::Orbit,
    }
}

/// The view keys to set for a drag of `nav` that has travelled `total`
/// pixels `(dx, dy)` (`dy` down) from where it began, from the base view,
/// over a canvas `width` pixels wide. Orbit and pan are the scroll's; a
/// zoom drag goes Blender's way — the pointer up zooms in (`viewzoom_apply`,
/// the default vertical style), so the sign is turned.
pub fn drag_sets(nav: BDragNav, base: BNavBase, total: (f32, f32), width: f32) -> Vec<(BPath, String)> {
    match nav {
        BDragNav::Orbit => scroll_sets(BScrollNav::Orbit, base, total, width),
        BDragNav::Pan => scroll_sets(BScrollNav::Pan, base, total, width),
        BDragNav::Zoom => scroll_sets(BScrollNav::Zoom, base, (0.0, -total.1), width),
    }
}

#[cfg(test)]
#[path = "navigate_tests.rs"]
mod tests;
