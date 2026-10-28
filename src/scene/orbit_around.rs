//! Orbit Around Selection (Preferences ▸ Navigation ▸ Orbit & Pan;
//! `PreferencesInput.use_rotate_around_active`, `USER_ORBIT_SELECTION`):
//! an orbit turns the view about the centre of what is selected rather than
//! about the view's own centre.
//!
//! Blender moves the view centre round the selection as the view turns
//! (`view3d_orbit_apply_dyn_ofs`): the selection keeps its place on screen
//! and everything else swings round it, so a model off to one side stays
//! there while the view walks round it. The centre is
//! `view3d_orbit_calc_center`'s in Object Mode — the mean of each selected
//! object's bounding-box centre, in the world; a camera, light or empty,
//! which has no box, counts with its origin.
//!
//! Panning moves the view centre too (`view3d.move` slides
//! `RegionView3D.ofs`): [`pan_pivot`] turns a slide on screen into the
//! centre that shows it, so the orbit centre stays in the middle of the view.
//!
//! Divergence, recorded: Blender ships with the preference off. Phiber's is
//! on until it is turned off — asked for, so that turning the view with a
//! model selected turns it round the model.

use super::BObject3D;
use super::camera::SCREEN;
use super::keep::BMeshKeep;
use super::orbit::BTurntable;
use super::pick::BPickView;
use super::view_slot::BViewSlot;
use crate::phiphx::scene::SceneMesh;
use crate::state::val::{BValue, V_FALSE};
use crate::state::{AppKey, BKey as _, BPath, BState, View3dKey};
use crate::view::prim::{BCtrl, BCtrlKind};
use glam::{Mat4, Vec3};
use std::sync::Arc;

/// The preference.
pub struct BOrbitAround;

impl BOrbitAround {
    /// On until set — Blender's is off (the module note says why).
    pub const DEFAULT: bool = true;

    pub fn key() -> BPath {
        AppKey::OrbitAroundSelection.path()
    }

    pub fn on(s: &BState) -> bool {
        s.get(AppKey::OrbitAroundSelection).map_or(Self::DEFAULT, |v| v != V_FALSE)
    }

    /// The Orbit & Pan card's checkbox, with Blender's label and tooltip.
    pub fn control(s: &BState) -> BCtrl {
        BCtrl::read(BCtrlKind::Check, Self::key(), BValue::Bool(Self::DEFAULT), s)
            .with_label("Orbit Around Selection")
            .with_tooltip("Use selection as the pivot point")
    }

    /// The point an orbit turns about now: the centre of the shown objects
    /// selected, when the preference is on and there are any.
    pub fn centre(s: &BState) -> Option<Vec3> {
        let selection = super::select::BSelection::now(s);
        if !Self::on(s) || selection.list.is_empty() {
            return None;
        }
        let hidden = crate::part::outliner::key::BHidden::now(s);
        let objects = super::source::scene_objects(s);
        let shown = |name: &str| selection.contains(name) && !hidden.hides(s, name);
        let mut points: Vec<Vec3> = objects.iter().filter(|o| shown(o.name())).map(centre_of).collect();
        if points.len() < selection.list.len() {
            let extras = super::source::with_loaded(|m| m.map(|m| super::pick::extra::BPickExtra::list(s, m)).unwrap_or_default());
            points.extend(extras.iter().filter(|e| selection.contains(&e.name)).map(|e| e.origin));
        }
        (!points.is_empty()).then(|| points.iter().sum::<Vec3>() / points.len() as f32)
    }
}

/// Each mesh's own box, made once and kept while the mesh lives; `None`
/// for a mesh with no vertices.
static BOXES: BMeshKeep<Option<(Vec3, Vec3)>> = BMeshKeep::new();

fn local_box(mesh: &Arc<SceneMesh>) -> Option<(Vec3, Vec3)> {
    mesh.vertices.iter().map(|v| Vec3::from(*v)).fold(None, |b, p| Some(b.map_or((p, p), |(lo, hi): (Vec3, Vec3)| (lo.min(p), hi.max(p)))))
}

/// One object's bounding-box centre in the world: its own box's centre
/// through its matrix, or its origin when it has no vertices.
pub fn centre_of(object: &BObject3D) -> Vec3 {
    match object {
        BObject3D::Primitive { x, y, .. } => {
            let (lo, hi) = super::view_op::extent_of(object);
            match lo.is_finite() && hi.is_finite() {
                true => (lo + hi) * 0.5,
                false => Vec3::new(x / super::MESH_SCALE, y / super::MESH_SCALE, 0.0),
            }
        }
        BObject3D::Loaded { mesh, matrix_world, .. } => {
            let m = Mat4::from_cols_array_2d(matrix_world);
            (*BOXES.of(mesh, local_box)).map_or(m.w_axis.truncate(), |(lo, hi)| m.transform_point3((lo + hi) * 0.5))
        }
    }
}

/// The view's axes for `(yaw, pitch)`: right, up and back.
fn axes((yaw, pitch): (f32, f32)) -> [Vec3; 3] {
    let (right, up, back) = BPickView { yaw, pitch, pan: (0.0, 0.0), zoom: 1.0, setup: Default::default(), aspect: 1.0 }.axes();
    [right, up, back]
}

/// `view3d_orbit_apply_dyn_ofs`: the view centre after the view turned
/// from `from` to `to` (yaw, pitch) about `about`. The point keeps its
/// place in the view — so on screen, in perspective as in orthographic.
pub fn turn_pivot(pivot: Vec3, about: Vec3, from: (f32, f32), to: (f32, f32)) -> Vec3 {
    let ([r0, u0, b0], [r1, u1, b1]) = (axes(from), axes(to));
    let d = pivot - about;
    about + r1 * d.dot(r0) + u1 * d.dot(u0) + b1 * d.dot(b0)
}

/// `view3d.move` (`viewmove_apply`): the view centre that shows the view
/// slid `shift` canvas widths (x right, y down — what the pan keys
/// measured). What lies at the centre's depth follows the pointer exactly,
/// as Blender's `ED_view3d_win_to_delta` at the centre's depth has it.
pub fn pan_pivot(pivot: Vec3, (yaw, pitch): (f32, f32), zoom: f32, shift: (f32, f32)) -> Vec3 {
    let [right, up, _] = axes((yaw, pitch));
    pivot + (up * shift.1 - right * shift.0) / (SCREEN * zoom * super::MESH_SCALE)
}

/// A pivot as `View3dKey::Pivot` holds it.
pub fn encode(p: Vec3) -> String {
    format!("{},{},{}", p.x, p.y, p.z)
}

/// `view3d.view_orbit` (Numpad 4/6/8/2, the gizmo's arrows): turn the
/// view's yaw or pitch `by` degrees — about the selection when
/// [`BOrbitAround::centre`] has one, else about the view's centre.
pub fn step(state: &mut BState, key: View3dKey, by: f32) {
    let from = (super::view_yaw(state), super::view_pitch(state));
    let to = match key {
        View3dKey::Yaw => ((from.0 + by).rem_euclid(360.0), from.1),
        _ => (from.0, BTurntable::wrap(from.1 + by)),
    };
    state.set(&BViewSlot::key(key), match key { View3dKey::Yaw => to.0, _ => to.1 }.to_string());
    if let Some(about) = BOrbitAround::centre(state) {
        let pivot = Vec3::from(super::camera::view_pivot(state));
        state.set(&super::pivot_key(), encode(turn_pivot(pivot, about, from, to)));
    }
}

#[cfg(test)]
#[path = "orbit_around_tests.rs"]
mod tests;
