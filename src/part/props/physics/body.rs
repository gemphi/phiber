//! Physics ▸ Rigid Body (`properties_physics_rigidbody.py`): the panel
//! tree Blender draws for `Object.rigid_body`, and the active object's
//! values from `BRigidBody`.

use super::{clear, put};
use crate::part::props::prop::*;
use crate::part::props::set::BSectionSpec;
use crate::part::props::set::sec;
use crate::phiphx::blend::physics::{BMeshSource, BRigidBody};
use crate::state::{BState, val::BValue};

static KIND: &[BEnumItem] = &[en("ACTIVE", "Active"), en("PASSIVE", "Passive")];
static SHAPE: &[BEnumItem] = &[
    en("BOX", "Box"),
    en("SPHERE", "Sphere"),
    en("CAPSULE", "Capsule"),
    en("CYLINDER", "Cylinder"),
    en("CONE", "Cone"),
    en("CONVEX_HULL", "Convex Hull"),
    en("MESH", "Mesh"),
    en("COMPOUND", "Compound Parent"),
];
static SOURCE: &[BEnumItem] = &[en("BASE", "Base"), en("DEFORM", "Deform"), en("FINAL", "Final")];

// Defaults are `rna_rigidbody.cc`'s (`BKE_rigidbody_create_object`), so
// an object without a rigid body reads as Add ▸ Rigid Body would make it.
static RIGID_BODY: &[BPropSpec] = &[e("rigid_body.type", "Type", KIND).picked("ACTIVE")];
static SETTINGS: &[BPropSpec] = &[
    f("rigid_body.mass", "Mass", BUnit::Mass).about(1.0),
    b("rigid_body.enabled", "Dynamic").on(),
    b("rigid_body.kinematic", "Animated"),
];
static COLLISIONS: &[BPropSpec] = &[
    e("rigid_body.collision_shape", "Shape", SHAPE).picked("CONVEX_HULL"),
    e("rigid_body.mesh_source", "Source", SOURCE).picked("DEFORM"),
    b("rigid_body.use_deform", "Deforming"),
];
static SURFACE: &[BPropSpec] = &[
    f("rigid_body.friction", "Friction", BUnit::None).about(0.5),
    f("rigid_body.restitution", "Bounciness", BUnit::None),
];
static SENSITIVITY: &[BPropSpec] = &[
    b("rigid_body.use_margin", "Collision Margin"),
    f("rigid_body.collision_margin", "Margin", BUnit::Length).about(0.04),
];
// Blender's twenty-cell grid, as the numbers of the cells that are on.
static COLLECTIONS: &[BPropSpec] = &[s("rigid_body.collision_collections", "Collections")];
static DYNAMICS: &[BPropSpec] = &[
    f("rigid_body.linear_damping", "Damping Translation", BUnit::None).about(0.04),
    f("rigid_body.angular_damping", "Rotation", BUnit::None).about(0.1),
];
static DEACTIVATION_HEADER: &[BPropSpec] = &[b("rigid_body.use_deactivation", "Deactivation")];
static DEACTIVATION: &[BPropSpec] = &[
    b("rigid_body.use_start_deactivated", "Start Deactivated"),
    f("rigid_body.deactivate_linear_velocity", "Velocity Linear", BUnit::Velocity).about(0.4),
    f("rigid_body.deactivate_angular_velocity", "Angular", BUnit::Velocity).about(0.5),
];
static COLLISIONS_SUBS: &[BSectionSpec] = &[
    sec("Surface Response", SURFACE).closed(),
    sec("Sensitivity", SENSITIVITY).closed(),
    sec("Collections", COLLECTIONS).closed(),
];
static DYNAMICS_SUBS: &[BSectionSpec] = &[sec("Deactivation", DEACTIVATION).closed().header(DEACTIVATION_HEADER)];
static SUBS: &[BSectionSpec] = &[
    sec("Settings", SETTINGS),
    sec("Collisions", COLLISIONS).subs(COLLISIONS_SUBS),
    sec("Dynamics", DYNAMICS).closed().subs(DYNAMICS_SUBS),
];
pub(super) static SECTION: BSectionSpec = sec("Rigid Body", RIGID_BODY).subs(SUBS);

/// The active object's rigid body into its rows; none clears them.
pub(super) fn seed(state: &mut BState, body: Option<&BRigidBody>) {
    let Some(body) = body else {
        clear(state, &SECTION);
        return;
    };
    let source = match body.mesh_source {
        BMeshSource::Base => "BASE",
        BMeshSource::Deform => "DEFORM",
        BMeshSource::Final => "FINAL",
    };
    let collections: Vec<String> = (0..20).filter(|k| body.collections & (1 << k) != 0).map(|k| k.to_string()).collect();
    for (key, value) in [
        ("rigid_body.type", BValue::Enum(body.kind.blender().to_string())),
        ("rigid_body.mass", BValue::Float(body.mass)),
        ("rigid_body.enabled", BValue::Bool(body.enabled)),
        ("rigid_body.kinematic", BValue::Bool(body.kinematic)),
        ("rigid_body.collision_shape", BValue::Enum(body.shape.blender().to_string())),
        ("rigid_body.mesh_source", BValue::Enum(source.to_string())),
        ("rigid_body.use_deform", BValue::Bool(body.deforming)),
        ("rigid_body.friction", BValue::Float(body.friction)),
        ("rigid_body.restitution", BValue::Float(body.restitution)),
        ("rigid_body.use_margin", BValue::Bool(body.use_margin)),
        ("rigid_body.collision_margin", BValue::Float(body.margin)),
        ("rigid_body.collision_collections", BValue::Text(collections.join(", "))),
        ("rigid_body.linear_damping", BValue::Float(body.linear_damping)),
        ("rigid_body.angular_damping", BValue::Float(body.angular_damping)),
        ("rigid_body.use_deactivation", BValue::Bool(body.use_deactivation)),
        ("rigid_body.use_start_deactivated", BValue::Bool(body.start_deactivated)),
        ("rigid_body.deactivate_linear_velocity", BValue::Float(body.linear_sleep)),
        ("rigid_body.deactivate_angular_velocity", BValue::Float(body.angular_sleep)),
    ] {
        put(state, key, value);
    }
}
