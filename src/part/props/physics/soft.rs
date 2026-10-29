//! Physics ▸ Soft Body (`properties_physics_softbody.py`): the panel tree
//! Blender draws for `Object.soft_body`, and the active object's values
//! from `BSoftBody`. The panel's own row, the collision collection, is not
//! read from the file and is left out; its sub-panels are all closed, as
//! Blender declares them.

use super::{clear, put};
use crate::part::props::prop::*;
use crate::part::props::set::BSectionSpec;
use crate::part::props::set::sec;
use crate::phiphx::blend::physics::{BSoftBody, BSoftCollision};
use crate::state::{BState, val::BValue};

static COLLISION: &[BEnumItem] = &[
    en("MANUAL", "Manual"),
    en("AVERAGE", "Average"),
    en("MINIMAL", "Minimal"),
    en("MAXIMAL", "Maximal"),
    en("MINMAX", "AvgMinMax"),
];
static NONE: &[BPropSpec] = &[];

// Defaults are `sbNew`'s, what Add ▸ Soft Body starts from.
static OBJECT: &[BPropSpec] = &[
    f("soft_body.friction", "Friction", BUnit::None).about(0.5),
    f("soft_body.mass", "Mass", BUnit::Mass).about(1.0),
    s("soft_body.vertex_group_mass", "Control Point"),
];
static SIMULATION: &[BPropSpec] = &[f("soft_body.speed", "Speed", BUnit::None).about(1.0)];
static GOAL_HEADER: &[BPropSpec] = &[b("soft_body.use_goal", "Goal").on()];
static GOAL: &[BPropSpec] = &[s("soft_body.vertex_group_goal", "Vertex Group")];
static STRENGTHS: &[BPropSpec] = &[
    f("soft_body.goal_default", "Default", BUnit::None).about(0.7),
    f("soft_body.goal_min", "Min", BUnit::None),
    f("soft_body.goal_max", "Max", BUnit::None).about(1.0),
];
static GOAL_SETTINGS: &[BPropSpec] = &[
    f("soft_body.goal_spring", "Stiffness", BUnit::None).about(0.5),
    f("soft_body.goal_friction", "Damping", BUnit::None),
];
static EDGES_HEADER: &[BPropSpec] = &[b("soft_body.use_edges", "Edges").on()];
static EDGES: &[BPropSpec] = &[
    s("soft_body.vertex_group_spring", "Springs"),
    f("soft_body.pull", "Pull", BUnit::None).about(0.5),
    f("soft_body.push", "Push", BUnit::None).about(0.5),
    f("soft_body.damping", "Damp", BUnit::None).about(0.5),
    i("soft_body.plastic", "Plasticity", 0, 100),
    f("soft_body.bend", "Bending", BUnit::None).about(0.1),
    i("soft_body.spring_length", "Length", 0, 200),
    b("soft_body.use_edge_collision", "Edge"),
    b("soft_body.use_face_collision", "Face"),
];
static STIFFNESS_HEADER: &[BPropSpec] = &[b("soft_body.use_stiff_quads", "Stiffness")];
static STIFFNESS: &[BPropSpec] = &[f("soft_body.shear", "Shear", BUnit::None).about(1.0)];
static SELF_HEADER: &[BPropSpec] = &[b("soft_body.use_self_collision", "Self Collision")];
static SELF: &[BPropSpec] = &[
    e("soft_body.collision_type", "Calculation Type", COLLISION).picked("AVERAGE"),
    f("soft_body.ball_size", "Ball Size", BUnit::None).about(0.49),
    f("soft_body.ball_stiff", "Stiffness", BUnit::None).about(1.0),
    f("soft_body.ball_damp", "Dampening", BUnit::None).about(0.5),
];
static SOLVER: &[BPropSpec] = &[
    i("soft_body.step_min", "Step Size Min", 0, 30000).at(10),
    i("soft_body.step_max", "Max", 0, 30000).at(300),
    b("soft_body.use_auto_step", "Auto-Step").on(),
    f("soft_body.error_threshold", "Error Limit", BUnit::None).about(0.1),
];
static HELPERS: &[BPropSpec] = &[
    i("soft_body.choke", "Choke", 0, 100).at(3),
    i("soft_body.fuzzy", "Fuzzy", 1, 100).at(1),
];
static GOAL_SUBS: &[BSectionSpec] = &[sec("Strengths", STRENGTHS).closed(), sec("Settings", GOAL_SETTINGS).closed()];
static EDGES_SUBS: &[BSectionSpec] = &[sec("Stiffness", STIFFNESS).closed().header(STIFFNESS_HEADER)];
static SOLVER_SUBS: &[BSectionSpec] = &[sec("Helpers", HELPERS).closed()];
static SUBS: &[BSectionSpec] = &[
    sec("Object", OBJECT).closed(),
    sec("Simulation", SIMULATION).closed(),
    sec("Goal", GOAL).closed().header(GOAL_HEADER).subs(GOAL_SUBS),
    sec("Edges", EDGES).closed().header(EDGES_HEADER).subs(EDGES_SUBS),
    sec("Self Collision", SELF).closed().header(SELF_HEADER),
    sec("Solver", SOLVER).closed().subs(SOLVER_SUBS),
];
pub(super) static SECTION: BSectionSpec = sec("Soft Body", NONE).subs(SUBS);

/// The active object's soft body into its rows; none clears them.
pub(super) fn seed(state: &mut BState, soft: Option<&BSoftBody>) {
    let Some(soft) = soft else {
        clear(state, &SECTION);
        return;
    };
    let collision = match soft.collision {
        BSoftCollision::Manual => "MANUAL",
        BSoftCollision::Average => "AVERAGE",
        BSoftCollision::Minimal => "MINIMAL",
        BSoftCollision::Maximal => "MAXIMAL",
        BSoftCollision::AverageMinMax => "MINMAX",
    };
    for (key, value) in [
        ("soft_body.friction", BValue::Float(soft.friction)),
        ("soft_body.mass", BValue::Float(soft.mass)),
        ("soft_body.vertex_group_mass", BValue::Text(soft.mass_group.clone())),
        ("soft_body.speed", BValue::Float(soft.speed)),
        ("soft_body.use_goal", BValue::Bool(soft.goal)),
        ("soft_body.vertex_group_goal", BValue::Text(soft.goal_group.clone())),
        ("soft_body.goal_default", BValue::Float(soft.goal_default)),
        ("soft_body.goal_min", BValue::Float(soft.goal_min)),
        ("soft_body.goal_max", BValue::Float(soft.goal_max)),
        ("soft_body.goal_spring", BValue::Float(soft.goal_spring)),
        ("soft_body.goal_friction", BValue::Float(soft.goal_friction)),
        ("soft_body.use_edges", BValue::Bool(soft.edges)),
        ("soft_body.vertex_group_spring", BValue::Text(soft.spring_group.clone())),
        ("soft_body.pull", BValue::Float(soft.pull)),
        ("soft_body.push", BValue::Float(soft.push)),
        ("soft_body.damping", BValue::Float(soft.damping)),
        ("soft_body.plastic", BValue::Int(i64::from(soft.plastic))),
        ("soft_body.bend", BValue::Float(soft.bending)),
        ("soft_body.spring_length", BValue::Int(i64::from(soft.spring_length))),
        ("soft_body.use_edge_collision", BValue::Bool(soft.edge_collision)),
        ("soft_body.use_face_collision", BValue::Bool(soft.face_collision)),
        ("soft_body.use_stiff_quads", BValue::Bool(soft.stiff_quads)),
        ("soft_body.shear", BValue::Float(soft.shear)),
        ("soft_body.use_self_collision", BValue::Bool(soft.self_collision)),
        ("soft_body.collision_type", BValue::Enum(collision.to_string())),
        ("soft_body.ball_size", BValue::Float(soft.ball_size)),
        ("soft_body.ball_stiff", BValue::Float(soft.ball_stiffness)),
        ("soft_body.ball_damp", BValue::Float(soft.ball_damping)),
        ("soft_body.step_min", BValue::Int(i64::from(soft.step_min))),
        ("soft_body.step_max", BValue::Int(i64::from(soft.step_max))),
        ("soft_body.use_auto_step", BValue::Bool(soft.auto_step)),
        ("soft_body.error_threshold", BValue::Float(soft.error_limit)),
        ("soft_body.choke", BValue::Int(i64::from(soft.choke))),
        ("soft_body.fuzzy", BValue::Int(i64::from(soft.fuzzy))),
    ] {
        put(state, key, value);
    }
}
