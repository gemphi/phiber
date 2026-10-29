//! Scene context — `bl_ui/properties_scene.py`.
// Scene context set — sections + RNA properties for `bpy.types.Scene`.
// Blender ref: `bl_ui/properties_scene.py`, `rna_scene.cc`.

use super::prop::*;
use super::set::BSectionSpec;
use super::set::sec;
use crate::state::val::BValue;

static UNIT_SYSTEM: &[BEnumItem] = &[
    en("NONE", "None"),
    en("METRIC", "Metric"),
    en("IMPERIAL", "Imperial"),
];
static ROTATION_UNIT: &[BEnumItem] = &[en("DEGREES", "Degrees"), en("RADIANS", "Radians")];
static AUDIO_DIST: &[BEnumItem] = &[
    en("NONE", "None"),
    en("INVERSE", "Inverse"),
    en("INVERSE_CLAMPED", "Inverse Clamped"),
    en("LINEAR", "Linear"),
    en("LINEAR_CLAMPED", "Linear Clamped"),
    en("EXPONENT", "Exponent"),
    en("EXPONENT_CLAMPED", "Exponent Clamped"),
];

static SCENE: &[BPropSpec] = &[
    p("camera", "Camera", "Object"),
    p("background_set", "Background", "Scene"),
    p("active_clip", "Active Clip", "MovieClip"),
];

static UNITS: &[BPropSpec] = &[
    e("unit_settings.system", "Unit System", UNIT_SYSTEM),
    f("unit_settings.scale_length", "Unit Scale", BUnit::None),
    e("unit_settings.system_rotation", "Rotation", ROTATION_UNIT),
    e("unit_settings.length_unit", "Length", &[]), // itemf at runtime
    e("unit_settings.mass_unit", "Mass", &[]),
    e("unit_settings.time_unit", "Time", &[]),
    e("unit_settings.temperature_unit", "Temperature", &[]),
];

static KEYING_SETS: &[BPropSpec] = &[
    p("active_keying_set", "Active Keying Set", "KeyingSet"),
    b("use_active_keying_set_only", "Only Active Keying Set"),
];

static AUDIO: &[BPropSpec] = &[
    f("audio_volume", "Volume", BUnit::None),
    e("audio_distance_model", "Distance Model", AUDIO_DIST),
    f("audio_doppler_speed", "Speed", BUnit::Velocity),
    f("audio_doppler_factor", "Doppler", BUnit::None),
];

// `SCENE_PT_physics`: the scene's gravity, its switch in the header.
static GRAVITY_HEADER: &[BPropSpec] = &[b("use_gravity", "Gravity").on()];
static GRAVITY: &[BPropSpec] = &[v3("gravity", "Gravity", BUnit::None)];

// `SCENE_PT_rigid_body_world`, its switch in the header. The defaults are
// `BKE_rigidbody_world_create`'s, what a scene without a world reads.
static RIGID_BODY_WORLD_HEADER: &[BPropSpec] = &[b("rigidbody_world.enabled", "Rigid Body World").on()];
static RIGID_BODY_WORLD: &[BPropSpec] = &[
    p("rigidbody_world", "Rigid Body World", "RigidBodyWorld"),
    i("rigidbody_world.substeps_per_frame", "Sub Steps", 1, 1000).at(10),
    i("rigidbody_world.solver_iterations", "Solver Iterations", 1, 1000).at(10),
    f("rigidbody_world.time_scale", "Speed", BUnit::None).about(1.0),
    b("rigidbody_world.use_split_impulse", "Split Impulse"),
];

pub static SECTIONS: &[BSectionSpec] = &[
    sec("Scene", SCENE),
    sec("Units", UNITS).closed(),
    sec("Keying Sets", KEYING_SETS).closed(),
    sec("Audio", AUDIO).closed(),
    sec("Gravity", GRAVITY).closed().header(GRAVITY_HEADER),
    sec("Rigid Body World", RIGID_BODY_WORLD).closed().header(RIGID_BODY_WORLD_HEADER),
];

/// The open file's value for a Scene-tab row the file carries - the Rigid
/// Body World's settings and the gravity (`SceneModel::physics`) - which
/// the row shows until the panel is edited (`prop_view::fallback`). `axis`
/// is a vector's component. `None` for a row the file does not settle: no
/// file open, or a scene with no Rigid Body World, which reads as the RNA
/// defaults. The gravity is the one row with a value even then, Blender's
/// startup (0, 0, -9.81): a vector has no `BDefault`, and the factory
/// zero is wrong.
pub fn file_value(key: &str, axis: Option<usize>) -> Option<BValue> {
    crate::scene::source::with_loaded(|loaded| {
        let physics = loaded.map(|model| &model.physics);
        let gravity = physics.map_or([0.0, 0.0, -9.81], |p| p.gravity);
        let world = physics.and_then(|p| p.world.as_ref());
        Some(match key {
            "gravity" => BValue::Float(*gravity.get(axis?)?),
            "rigidbody_world.enabled" => BValue::Bool(world?.enabled),
            "rigidbody_world.substeps_per_frame" => BValue::Int(i64::from(world?.substeps_per_frame)),
            "rigidbody_world.solver_iterations" => BValue::Int(i64::from(world?.solver_iterations)),
            "rigidbody_world.time_scale" => BValue::Float(world?.time_scale),
            "rigidbody_world.use_split_impulse" => BValue::Bool(world?.split_impulse),
            _ => return None,
        })
    })
}
