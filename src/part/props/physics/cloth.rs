//! Physics ▸ Cloth (`properties_physics_cloth.py`): the panel tree Blender
//! draws for a Cloth modifier's `settings` and `collision_settings`, and
//! the active object's values from `BCloth`.

use super::{clear, put};
use crate::part::props::prop::*;
use crate::part::props::set::BSectionSpec;
use crate::part::props::set::sec;
use crate::phiphx::blend::physics::BCloth;
use crate::state::{BState, val::BValue};

static BENDING: &[BEnumItem] = &[en("ANGULAR", "Angular"), en("LINEAR", "Linear")];

// Defaults are `rna_cloth.cc`'s (`cloth_init`), what Add ▸ Cloth starts from.
static CLOTH: &[BPropSpec] = &[i("cloth.settings.quality", "Quality Steps", 1, 80).at(5)];
static PHYSICAL: &[BPropSpec] = &[
    f("cloth.settings.mass", "Vertex Mass", BUnit::Mass).about(0.3),
    f("cloth.settings.air_damping", "Air Viscosity", BUnit::None).about(1.0),
    e("cloth.settings.bending_model", "Bending Model", BENDING).picked("ANGULAR"),
];
static STIFFNESS: &[BPropSpec] = &[
    f("cloth.settings.tension_stiffness", "Tension", BUnit::None).about(15.0),
    f("cloth.settings.compression_stiffness", "Compression", BUnit::None).about(15.0),
    f("cloth.settings.shear_stiffness", "Shear", BUnit::None).about(5.0),
    f("cloth.settings.bending_stiffness", "Bending", BUnit::None).about(0.5),
];
static DAMPING: &[BPropSpec] = &[
    f("cloth.settings.tension_damping", "Tension", BUnit::None).about(5.0),
    f("cloth.settings.compression_damping", "Compression", BUnit::None).about(5.0),
    f("cloth.settings.shear_damping", "Shear", BUnit::None).about(5.0),
    f("cloth.settings.bending_damping", "Bending", BUnit::None).about(0.5),
];
static PRESSURE_HEADER: &[BPropSpec] = &[b("cloth.settings.use_pressure", "Pressure")];
static PRESSURE: &[BPropSpec] = &[
    f("cloth.settings.uniform_pressure_force", "Pressure", BUnit::None),
    b("cloth.settings.use_pressure_volume", "Custom Volume"),
    f("cloth.settings.target_volume", "Target Volume", BUnit::None),
    f("cloth.settings.pressure_factor", "Pressure Scale", BUnit::None).about(1.0),
    f("cloth.settings.fluid_density", "Fluid Density", BUnit::None),
];
static SHAPE: &[BPropSpec] = &[
    s("cloth.settings.vertex_group_mass", "Pin Group"),
    f("cloth.settings.pin_stiffness", "Stiffness", BUnit::None).about(1.0),
    b("cloth.settings.use_sewing_springs", "Sewing"),
    f("cloth.settings.shrink_min", "Shrinking Factor", BUnit::None),
    b("cloth.settings.use_dynamic_mesh", "Dynamic Mesh"),
];
static COLLISIONS: &[BPropSpec] = &[i("cloth.collision_settings.collision_quality", "Quality", 1, 20).at(2)];
static OBJECT_HEADER: &[BPropSpec] = &[b("cloth.collision_settings.use_collision", "Object Collisions").on()];
static OBJECT: &[BPropSpec] = &[
    f("cloth.collision_settings.distance_min", "Distance", BUnit::Length).about(0.015),
    f("cloth.collision_settings.impulse_clamp", "Impulse Clamping", BUnit::None),
    f("cloth.collision_settings.friction", "Friction", BUnit::None).about(5.0),
    p("cloth.collision_settings.collection", "Collection", "Collection"),
];
static SELF_HEADER: &[BPropSpec] = &[b("cloth.collision_settings.use_self_collision", "Self Collisions")];
static SELF: &[BPropSpec] = &[
    f("cloth.collision_settings.self_friction", "Friction", BUnit::None).about(5.0),
    f("cloth.collision_settings.self_distance_min", "Distance", BUnit::Length).about(0.015),
    f("cloth.collision_settings.self_impulse_clamp", "Impulse Clamping", BUnit::None),
    s("cloth.collision_settings.vertex_group_self_collisions", "Vertex Group"),
];
static WEIGHTS: &[BPropSpec] = &[
    s("cloth.settings.vertex_group_structural_stiffness", "Structural Group"),
    s("cloth.settings.vertex_group_shear_stiffness", "Shear Group"),
    s("cloth.settings.vertex_group_bending", "Bending Group"),
];
static PHYSICAL_SUBS: &[BSectionSpec] = &[
    sec("Stiffness", STIFFNESS),
    sec("Damping", DAMPING),
    sec("Pressure", PRESSURE).header(PRESSURE_HEADER),
];
static COLLISIONS_SUBS: &[BSectionSpec] = &[
    sec("Object Collisions", OBJECT).header(OBJECT_HEADER),
    sec("Self Collisions", SELF).header(SELF_HEADER),
];
static SUBS: &[BSectionSpec] = &[
    sec("Physical Properties", PHYSICAL).subs(PHYSICAL_SUBS),
    sec("Shape", SHAPE).closed(),
    sec("Collisions", COLLISIONS).closed().subs(COLLISIONS_SUBS),
    sec("Property Weights", WEIGHTS).closed(),
];
pub(super) static SECTION: BSectionSpec = sec("Cloth", CLOTH).subs(SUBS);

/// The active object's cloth into its rows; none clears them.
pub(super) fn seed(state: &mut BState, cloth: Option<&BCloth>) {
    let Some(cloth) = cloth else {
        clear(state, &SECTION);
        return;
    };
    let bending = match cloth.angular_bending {
        true => "ANGULAR",
        false => "LINEAR",
    };
    let hit = &cloth.collision;
    for (key, value) in [
        ("cloth.settings.quality", BValue::Int(i64::from(cloth.quality))),
        ("cloth.settings.mass", BValue::Float(cloth.mass)),
        ("cloth.settings.air_damping", BValue::Float(cloth.air_damping)),
        ("cloth.settings.bending_model", BValue::Enum(bending.to_string())),
        ("cloth.settings.tension_stiffness", BValue::Float(cloth.tension)),
        ("cloth.settings.compression_stiffness", BValue::Float(cloth.compression)),
        ("cloth.settings.shear_stiffness", BValue::Float(cloth.shear)),
        ("cloth.settings.bending_stiffness", BValue::Float(cloth.bending)),
        ("cloth.settings.tension_damping", BValue::Float(cloth.tension_damping)),
        ("cloth.settings.compression_damping", BValue::Float(cloth.compression_damping)),
        ("cloth.settings.shear_damping", BValue::Float(cloth.shear_damping)),
        ("cloth.settings.bending_damping", BValue::Float(cloth.bending_damping)),
        ("cloth.settings.use_pressure", BValue::Bool(cloth.pressure)),
        ("cloth.settings.uniform_pressure_force", BValue::Float(cloth.pressure_force)),
        ("cloth.settings.use_pressure_volume", BValue::Bool(cloth.target_volume.is_some())),
        ("cloth.settings.target_volume", BValue::Float(cloth.target_volume.unwrap_or(0.0))),
        ("cloth.settings.pressure_factor", BValue::Float(cloth.pressure_factor)),
        ("cloth.settings.fluid_density", BValue::Float(cloth.fluid_density)),
        ("cloth.settings.vertex_group_mass", BValue::Text(cloth.pin_group.clone())),
        ("cloth.settings.pin_stiffness", BValue::Float(cloth.pin_stiffness)),
        ("cloth.settings.use_sewing_springs", BValue::Bool(cloth.sewing)),
        ("cloth.settings.shrink_min", BValue::Float(cloth.shrink[0])),
        ("cloth.settings.use_dynamic_mesh", BValue::Bool(cloth.dynamic_mesh)),
        ("cloth.settings.vertex_group_structural_stiffness", BValue::Text(cloth.stiffness_groups[0].clone())),
        ("cloth.settings.vertex_group_shear_stiffness", BValue::Text(cloth.stiffness_groups[1].clone())),
        ("cloth.settings.vertex_group_bending", BValue::Text(cloth.stiffness_groups[2].clone())),
        ("cloth.collision_settings.collision_quality", BValue::Int(i64::from(hit.quality))),
        ("cloth.collision_settings.use_collision", BValue::Bool(hit.enabled)),
        ("cloth.collision_settings.distance_min", BValue::Float(hit.distance)),
        ("cloth.collision_settings.impulse_clamp", BValue::Float(hit.impulse_clamp)),
        ("cloth.collision_settings.friction", BValue::Float(hit.friction)),
        ("cloth.collision_settings.collection", BValue::Text(hit.collection.clone())),
        ("cloth.collision_settings.use_self_collision", BValue::Bool(hit.self_enabled)),
        ("cloth.collision_settings.self_friction", BValue::Float(hit.self_friction)),
        ("cloth.collision_settings.self_distance_min", BValue::Float(hit.self_distance)),
        ("cloth.collision_settings.self_impulse_clamp", BValue::Float(hit.self_impulse_clamp)),
        ("cloth.collision_settings.vertex_group_self_collisions", BValue::Text(hit.self_group.clone())),
    ] {
        put(state, key, value);
    }
}
