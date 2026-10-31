//! The Physics tab's Rigid Body rows, over the file's body.
//!
//! The Scene tab's Rigid Body World rows reach the solver through
//! [`BPhysicsRows`](super::BPhysicsRows); these are the other half — what
//! Properties ▸ Physics ▸ Rigid Body shows for ONE object. Blender's
//! Properties editor edits the active object, so that is the one body a
//! panel edit can move; every other body is the file's as read.
//!
//! Each row is read over the file's value, so a row the user never touched
//! reads back as the file had it and the bake is unchanged. The keys are
//! `part::props::physics::body`'s, and the seed that puts the file into them
//! is that module's `seed`.

use crate::phiphx::blend::physics::{BRigidBody, BRigidKind, BRigidShape};
use crate::state::{BState, ButsKey, val::{BValue, BValueKind}};

/// What the state **holds** for `key`, or `None`.
///
/// Deliberately not `prop_view::shown`, which falls back to the property's
/// RNA default when the state has nothing: that default is Blender's for a
/// *new* body (mass 1, Convex Hull), not this file's, so a panel the user
/// has not opened would have overridden the file with it. Reading the raw
/// value means an untouched row leaves the file's value alone, which is the
/// whole contract of this module.
fn held(s: &BState, key: &'static str, kind: BValueKind) -> Option<BValue> {
    BValue::parse(kind, s.get(ButsKey::Value(key))?)
}

fn flag(s: &BState, key: &'static str, file: bool) -> bool {
    match held(s, key, BValueKind::Bool) {
        Some(BValue::Bool(on)) => on,
        _ => file,
    }
}

fn number(s: &BState, key: &'static str, file: f32) -> f32 {
    match held(s, key, BValueKind::Float) {
        Some(BValue::Float(v)) => v,
        _ => file,
    }
}

fn chosen(s: &BState, key: &'static str) -> Option<String> {
    match held(s, key, BValueKind::Enum) {
        Some(BValue::Enum(name)) if !name.is_empty() => Some(name),
        _ => None,
    }
}

/// `eRigidBodyOb_Type` as the Type row spells it.
fn kind_of(name: &str) -> Option<BRigidKind> {
    match name {
        "ACTIVE" => Some(BRigidKind::Active),
        "PASSIVE" => Some(BRigidKind::Passive),
        _ => None,
    }
}

/// `eRigidBody_Shape` as the Shape row spells it (`BRigidShape::blender`).
fn shape_of(name: &str) -> Option<BRigidShape> {
    [
        BRigidShape::Box,
        BRigidShape::Sphere,
        BRigidShape::Capsule,
        BRigidShape::Cylinder,
        BRigidShape::Cone,
        BRigidShape::ConvexHull,
        BRigidShape::Mesh,
        BRigidShape::Compound,
    ]
    .into_iter()
    .find(|s| s.blender() == name)
}

/// Blender's twenty-cell Collision Collections grid, which the row holds as
/// the numbers of the cells that are on (`body::seed` writes it that way):
/// `"0, 3"` is bits 0 and 3. An unreadable list leaves the file's mask.
fn collections_of(text: &str, file: u32) -> u32 {
    let mut mask = 0u32;
    let mut any = false;
    for part in text.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match part.parse::<u32>() {
            Ok(bit) if bit < 20 => {
                mask |= 1 << bit;
                any = true;
            }
            // A cell outside the grid, or not a number: the row is not
            // one phiber wrote, so the file's mask stands.
            _ => return file,
        }
    }
    // An empty list is a real answer — every collection off — and that is
    // not the same as "nothing to say".
    match any || text.trim().is_empty() {
        true => mask,
        false => file,
    }
}

/// `file` with every Physics ▸ Rigid Body row the panel shows over it.
///
/// Only the fields the solver reads are taken (`sim::input::body`'s list):
/// Deforming and Mesh Source are drawn and saved but nothing simulates
/// them, so overriding them here would claim an effect that does not exist.
pub(super) fn shown_over(s: &BState, file: &BRigidBody) -> BRigidBody {
    BRigidBody {
        object: file.object.clone(),
        kind: chosen(s, "rigid_body.type").and_then(|n| kind_of(&n)).unwrap_or(file.kind),
        shape: chosen(s, "rigid_body.collision_shape").and_then(|n| shape_of(&n)).unwrap_or(file.shape),
        enabled: flag(s, "rigid_body.enabled", file.enabled),
        kinematic: flag(s, "rigid_body.kinematic", file.kinematic),
        mass: number(s, "rigid_body.mass", file.mass),
        friction: number(s, "rigid_body.friction", file.friction),
        restitution: number(s, "rigid_body.restitution", file.restitution),
        use_margin: flag(s, "rigid_body.use_margin", file.use_margin),
        margin: number(s, "rigid_body.collision_margin", file.margin),
        linear_damping: number(s, "rigid_body.linear_damping", file.linear_damping),
        angular_damping: number(s, "rigid_body.angular_damping", file.angular_damping),
        use_deactivation: flag(s, "rigid_body.use_deactivation", file.use_deactivation),
        start_deactivated: flag(s, "rigid_body.use_start_deactivated", file.start_deactivated),
        linear_sleep: number(s, "rigid_body.deactivate_linear_velocity", file.linear_sleep),
        angular_sleep: number(s, "rigid_body.deactivate_angular_velocity", file.angular_sleep),
        collections: match held(s, "rigid_body.collision_collections", BValueKind::Text) {
            Some(BValue::Text(list)) => collections_of(&list, file.collections),
            _ => file.collections,
        },
        // Read and saved, but nothing simulates them: left as the file has
        // them so the bake key does not move for a row with no effect.
        mesh_source: file.mesh_source,
        deforming: file.deforming,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_collections_grid_reads_its_cells_and_keeps_the_file_when_it_cannot() {
        assert_eq!(collections_of("0, 3", 0b1), 0b1001, "cells 0 and 3");
        assert_eq!(collections_of("19", 0), 1 << 19, "the last cell of the grid");
        assert_eq!(collections_of("", 0b101), 0, "an empty list is every collection off");
        assert_eq!(collections_of("   ", 0b101), 0, "and so is whitespace");
        // Outside the grid or not a number: the file's mask stands rather
        // than a mask phiber invented.
        assert_eq!(collections_of("20", 0b110), 0b110, "cell 20 is off the grid");
        assert_eq!(collections_of("first", 0b110), 0b110);
    }

    #[test]
    fn the_rows_name_the_enums_blender_does() {
        assert_eq!(kind_of("PASSIVE"), Some(BRigidKind::Passive));
        assert_eq!(kind_of("ACTIVE"), Some(BRigidKind::Active));
        assert_eq!(kind_of("nonsense"), None, "an unknown Type leaves the file's");
        assert_eq!(shape_of("CONVEX_HULL"), Some(BRigidShape::ConvexHull));
        assert_eq!(shape_of("MESH"), Some(BRigidShape::Mesh));
        assert_eq!(shape_of("COMPOUND"), Some(BRigidShape::Compound));
        assert_eq!(shape_of("nonsense"), None);
        // Every shape the panel offers resolves, so no row is unreachable.
        for shape in [BRigidShape::Box, BRigidShape::Sphere, BRigidShape::Capsule, BRigidShape::Cylinder, BRigidShape::Cone, BRigidShape::ConvexHull, BRigidShape::Mesh, BRigidShape::Compound] {
            assert_eq!(shape_of(shape.blender()), Some(shape), "{shape:?}");
        }
    }
}
