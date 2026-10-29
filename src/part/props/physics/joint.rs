//! Physics ▸ Rigid Body Constraint
//! (`properties_physics_rigidbody_constraint.py`): the panel tree Blender
//! draws for `Object.rigid_body_constraint`, and the active object's
//! values from `BRigidJoint`. The six axes' limits and springs are keyed
//! as `rna_rigidbody.cc` names them (`limit_lin_x_lower`,
//! `spring_stiffness_ang_z`); an angular limit is shown in degrees, as the
//! Object tab shows a rotation.

use super::{clear, put};
use crate::part::props::prop::*;
use crate::part::props::set::BSectionSpec;
use crate::part::props::set::sec;
use crate::phiphx::blend::physics::{BRigidJoint, BSpringKind};
use crate::state::{BState, val::BValue};

static KIND: &[BEnumItem] = &[
    en("FIXED", "Fixed"),
    en("POINT", "Point"),
    en("HINGE", "Hinge"),
    en("SLIDER", "Slider"),
    en("PISTON", "Piston"),
    en("GENERIC", "Generic"),
    en("GENERIC_SPRING", "Generic Spring"),
    en("MOTOR", "Motor"),
];
static SPRING: &[BEnumItem] = &[en("SPRING1", "Blender 2.7"), en("SPRING2", "Blender 2.8")];
static NONE: &[BPropSpec] = &[];

// Defaults are `rna_rigidbody.cc`'s (`BKE_rigidbody_create_constraint`).
static CONSTRAINT: &[BPropSpec] = &[e("rigid_body_constraint.type", "Type", KIND).picked("FIXED")];
static SETTINGS: &[BPropSpec] = &[
    b("rigid_body_constraint.enabled", "Enabled").on(),
    b("rigid_body_constraint.disable_collisions", "Disable Collisions").on(),
    b("rigid_body_constraint.use_breaking", "Breakable"),
    f("rigid_body_constraint.breaking_threshold", "Threshold", BUnit::None).about(10.0),
];
static OBJECTS: &[BPropSpec] = &[
    p("rigid_body_constraint.object1", "First", "Object"),
    p("rigid_body_constraint.object2", "Second", "Object"),
];
static ITERATIONS_HEADER: &[BPropSpec] = &[b("rigid_body_constraint.use_override_solver_iterations", "Override Iterations")];
static ITERATIONS: &[BPropSpec] = &[i("rigid_body_constraint.solver_iterations", "Iterations", 1, 1000).at(10)];
static LIMITS_LINEAR: &[BPropSpec] = &[
    b("rigid_body_constraint.use_limit_lin_x", "X Axis"),
    f("rigid_body_constraint.limit_lin_x_lower", "X Lower", BUnit::Length).about(-1.0),
    f("rigid_body_constraint.limit_lin_x_upper", "Upper", BUnit::Length).about(1.0),
    b("rigid_body_constraint.use_limit_lin_y", "Y Axis"),
    f("rigid_body_constraint.limit_lin_y_lower", "Y Lower", BUnit::Length).about(-1.0),
    f("rigid_body_constraint.limit_lin_y_upper", "Upper", BUnit::Length).about(1.0),
    b("rigid_body_constraint.use_limit_lin_z", "Z Axis"),
    f("rigid_body_constraint.limit_lin_z_lower", "Z Lower", BUnit::Length).about(-1.0),
    f("rigid_body_constraint.limit_lin_z_upper", "Upper", BUnit::Length).about(1.0),
];
static LIMITS_ANGULAR: &[BPropSpec] = &[
    b("rigid_body_constraint.use_limit_ang_x", "X Angle"),
    f("rigid_body_constraint.limit_ang_x_lower", "X Lower", BUnit::Angle).about(-45.0),
    f("rigid_body_constraint.limit_ang_x_upper", "Upper", BUnit::Angle).about(45.0),
    b("rigid_body_constraint.use_limit_ang_y", "Y Angle"),
    f("rigid_body_constraint.limit_ang_y_lower", "Y Lower", BUnit::Angle).about(-45.0),
    f("rigid_body_constraint.limit_ang_y_upper", "Upper", BUnit::Angle).about(45.0),
    b("rigid_body_constraint.use_limit_ang_z", "Z Angle"),
    f("rigid_body_constraint.limit_ang_z_lower", "Z Lower", BUnit::Angle).about(-45.0),
    f("rigid_body_constraint.limit_ang_z_upper", "Upper", BUnit::Angle).about(45.0),
];
static MOTOR_ANGULAR_HEADER: &[BPropSpec] = &[b("rigid_body_constraint.use_motor_ang", "Angular")];
static MOTOR_ANGULAR: &[BPropSpec] = &[
    f("rigid_body_constraint.motor_ang_target_velocity", "Target Velocity", BUnit::None).about(1.0),
    f("rigid_body_constraint.motor_ang_max_impulse", "Max Impulse", BUnit::None).about(1.0),
];
static MOTOR_LINEAR_HEADER: &[BPropSpec] = &[b("rigid_body_constraint.use_motor_lin", "Linear")];
static MOTOR_LINEAR: &[BPropSpec] = &[
    f("rigid_body_constraint.motor_lin_target_velocity", "Target Velocity", BUnit::Velocity).about(1.0),
    f("rigid_body_constraint.motor_lin_max_impulse", "Max Impulse", BUnit::None).about(1.0),
];
static SPRINGS: &[BPropSpec] = &[e("rigid_body_constraint.spring_type", "Type", SPRING).picked("SPRING2")];
static SPRINGS_ANGULAR: &[BPropSpec] = &[
    b("rigid_body_constraint.use_spring_ang_x", "X Angle"),
    f("rigid_body_constraint.spring_stiffness_ang_x", "X Stiffness", BUnit::None).about(10.0),
    f("rigid_body_constraint.spring_damping_ang_x", "Damping", BUnit::None).about(0.5),
    b("rigid_body_constraint.use_spring_ang_y", "Y Angle"),
    f("rigid_body_constraint.spring_stiffness_ang_y", "Y Stiffness", BUnit::None).about(10.0),
    f("rigid_body_constraint.spring_damping_ang_y", "Damping", BUnit::None).about(0.5),
    b("rigid_body_constraint.use_spring_ang_z", "Z Angle"),
    f("rigid_body_constraint.spring_stiffness_ang_z", "Z Stiffness", BUnit::None).about(10.0),
    f("rigid_body_constraint.spring_damping_ang_z", "Damping", BUnit::None).about(0.5),
];
static SPRINGS_LINEAR: &[BPropSpec] = &[
    b("rigid_body_constraint.use_spring_x", "X Axis"),
    f("rigid_body_constraint.spring_stiffness_x", "X Stiffness", BUnit::None).about(10.0),
    f("rigid_body_constraint.spring_damping_x", "Damping", BUnit::None).about(0.5),
    b("rigid_body_constraint.use_spring_y", "Y Axis"),
    f("rigid_body_constraint.spring_stiffness_y", "Y Stiffness", BUnit::None).about(10.0),
    f("rigid_body_constraint.spring_damping_y", "Damping", BUnit::None).about(0.5),
    b("rigid_body_constraint.use_spring_z", "Z Axis"),
    f("rigid_body_constraint.spring_stiffness_z", "Z Stiffness", BUnit::None).about(10.0),
    f("rigid_body_constraint.spring_damping_z", "Damping", BUnit::None).about(0.5),
];
static LIMITS_SUBS: &[BSectionSpec] = &[sec("Linear", LIMITS_LINEAR), sec("Angular", LIMITS_ANGULAR)];
static MOTOR_SUBS: &[BSectionSpec] = &[
    sec("Angular", MOTOR_ANGULAR).header(MOTOR_ANGULAR_HEADER),
    sec("Linear", MOTOR_LINEAR).header(MOTOR_LINEAR_HEADER),
];
static SPRINGS_SUBS: &[BSectionSpec] = &[sec("Angular", SPRINGS_ANGULAR), sec("Linear", SPRINGS_LINEAR)];
static SUBS: &[BSectionSpec] = &[
    sec("Settings", SETTINGS),
    sec("Objects", OBJECTS),
    sec("Override Iterations", ITERATIONS).header(ITERATIONS_HEADER),
    sec("Limits", NONE).subs(LIMITS_SUBS),
    sec("Motor", NONE).subs(MOTOR_SUBS),
    sec("Springs", SPRINGS).subs(SPRINGS_SUBS),
];
pub(super) static SECTION: BSectionSpec = sec("Rigid Body Constraint", CONSTRAINT).subs(SUBS);

/// The active object's constraint into its rows; none clears them.
pub(super) fn seed(state: &mut BState, joint: Option<&BRigidJoint>) {
    let Some(joint) = joint else {
        clear(state, &SECTION);
        return;
    };
    let spring = match joint.spring_kind {
        BSpringKind::Spring1 => "SPRING1",
        BSpringKind::Spring2 => "SPRING2",
    };
    for (key, value) in [
        ("rigid_body_constraint.type", BValue::Enum(joint.kind.blender().to_string())),
        ("rigid_body_constraint.enabled", BValue::Bool(joint.enabled)),
        ("rigid_body_constraint.disable_collisions", BValue::Bool(joint.disable_collisions)),
        ("rigid_body_constraint.use_breaking", BValue::Bool(joint.use_breaking)),
        ("rigid_body_constraint.breaking_threshold", BValue::Float(joint.breaking_threshold)),
        ("rigid_body_constraint.object1", BValue::Text(joint.first.clone())),
        ("rigid_body_constraint.object2", BValue::Text(joint.second.clone())),
        ("rigid_body_constraint.use_override_solver_iterations", BValue::Bool(joint.override_iterations)),
        ("rigid_body_constraint.solver_iterations", BValue::Int(i64::from(joint.solver_iterations))),
        ("rigid_body_constraint.spring_type", BValue::Enum(spring.to_string())),
        ("rigid_body_constraint.use_motor_lin", BValue::Bool(joint.use_motor_linear)),
        ("rigid_body_constraint.motor_lin_target_velocity", BValue::Float(joint.motor_linear_velocity)),
        ("rigid_body_constraint.motor_lin_max_impulse", BValue::Float(joint.motor_linear_impulse)),
        ("rigid_body_constraint.use_motor_ang", BValue::Bool(joint.use_motor_angular)),
        ("rigid_body_constraint.motor_ang_target_velocity", BValue::Float(joint.motor_angular_velocity)),
        ("rigid_body_constraint.motor_ang_max_impulse", BValue::Float(joint.motor_angular_impulse)),
    ] {
        put(state, key, value);
    }
    // A limit's key carries its space (`limit_lin_x_lower`); a spring's
    // only the angular one (`spring_stiffness_x`, `spring_stiffness_ang_x`).
    // The file holds an angular limit in radians.
    for (space, spring, degrees, axes) in [("lin", "", false, &joint.linear), ("ang", "ang_", true, &joint.angular)] {
        for (a, axis) in ["x", "y", "z"].into_iter().zip(axes.iter()) {
            let angle = |v: f32| match degrees {
                true => v.to_degrees(),
                false => v,
            };
            put(state, &format!("rigid_body_constraint.use_limit_{space}_{a}"), BValue::Bool(axis.use_limit));
            put(state, &format!("rigid_body_constraint.limit_{space}_{a}_lower"), BValue::Float(angle(axis.lower)));
            put(state, &format!("rigid_body_constraint.limit_{space}_{a}_upper"), BValue::Float(angle(axis.upper)));
            put(state, &format!("rigid_body_constraint.use_spring_{spring}{a}"), BValue::Bool(axis.use_spring));
            put(state, &format!("rigid_body_constraint.spring_stiffness_{spring}{a}"), BValue::Float(axis.stiffness));
            put(state, &format!("rigid_body_constraint.spring_damping_{spring}{a}"), BValue::Float(axis.damping));
        }
    }
}
