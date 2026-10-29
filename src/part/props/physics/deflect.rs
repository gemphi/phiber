//! Physics ▸ Force Fields and Physics ▸ Collision
//! (`properties_physics_field.py`): the two panel trees Blender draws over
//! `Object.field` and `Object.collision` - one `PartDeflect` in the file,
//! `BForceField` and `BCollider` here - and the active object's values.

use super::{clear, put};
use crate::part::props::prop::*;
use crate::part::props::set::BSectionSpec;
use crate::part::props::set::sec;
use crate::phiphx::blend::physics::{BCollider, BFieldShape, BForceField};
use crate::state::{BState, val::BValue};

static NONE: &[BPropSpec] = &[];
static FIELD_KIND: &[BEnumItem] = &[
    en("NONE", "None"),
    en("FORCE", "Force"),
    en("WIND", "Wind"),
    en("VORTEX", "Vortex"),
    en("MAGNET", "Magnetic"),
    en("HARMONIC", "Harmonic"),
    en("CHARGE", "Charge"),
    en("LENNARDJ", "Lennard-Jones"),
    en("TEXTURE", "Texture"),
    en("GUIDE", "Curve Guide"),
    en("BOID", "Boid"),
    en("TURBULENCE", "Turbulence"),
    en("DRAG", "Drag"),
    en("FLUID_FLOW", "Fluid Flow"),
];
static FIELD_SHAPE: &[BEnumItem] = &[
    en("POINT", "Point"),
    en("LINE", "Line"),
    en("PLANE", "Plane"),
    en("SURFACE", "Surface"),
    en("POINTS", "Every Point"),
];

// Defaults are `BKE_partdeflect_new`'s, which Add ▸ Force Field starts from.
static FIELD: &[BPropSpec] = &[e("field.type", "Type", FIELD_KIND).picked("NONE")];
static FIELD_SETTINGS: &[BPropSpec] = &[
    e("field.shape", "Shape", FIELD_SHAPE).picked("POINT"),
    f("field.strength", "Strength", BUnit::None).about(1.0),
    f("field.flow", "Flow", BUnit::None),
    f("field.noise", "Noise Amount", BUnit::None),
    i("field.seed", "Seed", 0, 128),
];
static FALLOFF: &[BPropSpec] = &[
    f("field.falloff_power", "Power", BUnit::None),
    b("field.use_max_distance", "Max Distance"),
    f("field.distance_max", "Distance", BUnit::Length),
    b("field.use_min_distance", "Min Distance"),
    f("field.distance_min", "Distance", BUnit::Length),
];
static FIELD_SUBS: &[BSectionSpec] = &[sec("Settings", FIELD_SETTINGS), sec("Falloff", FALLOFF)];
pub(super) static FORCE_FIELDS: BSectionSpec = sec("Force Fields", FIELD).subs(FIELD_SUBS);

static PARTICLE: &[BPropSpec] = &[
    f("collision.permeability", "Permeability", BUnit::None),
    f("collision.stickiness", "Stickiness", BUnit::None),
    b("collision.use_particle_kill", "Kill Particles"),
    f("collision.damping_factor", "Damping", BUnit::None),
    f("collision.damping_random", "Randomize", BUnit::None),
    f("collision.friction_factor", "Friction", BUnit::None),
    f("collision.friction_random", "Randomize", BUnit::None),
];
static SOFTBODY: &[BPropSpec] = &[
    f("collision.damping", "Damping", BUnit::None).about(0.1),
    f("collision.thickness_outer", "Thickness Outer", BUnit::None).about(0.02),
    f("collision.thickness_inner", "Inner", BUnit::None).about(0.2),
    f("collision.cloth_friction", "Friction", BUnit::None).about(5.0),
    b("collision.use_culling", "Single Sided").on(),
    b("collision.use_normal", "Override Normals"),
];
static COLLISION_SUBS: &[BSectionSpec] = &[sec("Particle", PARTICLE), sec("Softbody & Cloth", SOFTBODY)];
pub(super) static COLLISION: BSectionSpec = sec("Collision", NONE).subs(COLLISION_SUBS);

/// The active object's force field into its rows; none clears them.
pub(super) fn seed_field(state: &mut BState, field: Option<&BForceField>) {
    let Some(field) = field else {
        clear(state, &FORCE_FIELDS);
        return;
    };
    let shape = match field.shape {
        BFieldShape::Point => "POINT",
        BFieldShape::Line => "LINE",
        BFieldShape::Plane => "PLANE",
        BFieldShape::Surface => "SURFACE",
        BFieldShape::Points => "POINTS",
    };
    for (key, value) in [
        ("field.type", BValue::Enum(field.kind.blender().to_string())),
        ("field.shape", BValue::Enum(shape.to_string())),
        ("field.strength", BValue::Float(field.strength)),
        ("field.flow", BValue::Float(field.flow)),
        ("field.noise", BValue::Float(field.noise)),
        ("field.seed", BValue::Int(i64::from(field.seed))),
        ("field.falloff_power", BValue::Float(field.power)),
        ("field.use_max_distance", BValue::Bool(field.max_distance.is_some())),
        ("field.distance_max", BValue::Float(field.max_distance.unwrap_or(0.0))),
        ("field.use_min_distance", BValue::Bool(field.min_distance.is_some())),
        ("field.distance_min", BValue::Float(field.min_distance.unwrap_or(0.0))),
    ] {
        put(state, key, value);
    }
}

/// The active object's collision settings into their rows; none clears them.
pub(super) fn seed_collider(state: &mut BState, collider: Option<&BCollider>) {
    let Some(collider) = collider else {
        clear(state, &COLLISION);
        return;
    };
    for (key, value) in [
        ("collision.permeability", BValue::Float(collider.permeability)),
        ("collision.stickiness", BValue::Float(collider.stickiness)),
        ("collision.use_particle_kill", BValue::Bool(collider.kill_particles)),
        ("collision.damping_factor", BValue::Float(collider.damping_factor)),
        ("collision.damping_random", BValue::Float(collider.damping_random)),
        ("collision.friction_factor", BValue::Float(collider.friction_factor)),
        ("collision.friction_random", BValue::Float(collider.friction_random)),
        ("collision.damping", BValue::Float(collider.damping)),
        ("collision.thickness_outer", BValue::Float(collider.thickness_outer)),
        ("collision.thickness_inner", BValue::Float(collider.thickness_inner)),
        ("collision.cloth_friction", BValue::Float(collider.cloth_friction)),
        ("collision.use_culling", BValue::Bool(collider.single_sided)),
        ("collision.use_normal", BValue::Bool(collider.override_normals)),
    ] {
        put(state, key, value);
    }
}
