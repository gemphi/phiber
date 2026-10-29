//! The Physics tab shows what the opened file holds.
//!
//! `physics.blend` (`phiphx/tests/fixtures/make_physics.py`) sets every
//! physics setting away from its default. Opening it fills the panels the
//! way the app does - `seed_view_layer` on open, then the rule behind
//! `ctx.object` as objects are made active - and every value is read back
//! the way the panel reads it (`prop_view::shown`): the state's value, else
//! the file's for a Scene-tab row, else the property's RNA default. A value
//! that comes back as the default is a row the file never reached.

use std::path::Path;

use phiber::app::seed::seed_view_layer;
use phiber::part::props::prop_view::shown;
use phiber::part::props::set::prop_named;
use phiber::phiphx::Phx;
use phiber::phiphx::scene::SceneModel;
use phiber::scene::{clear_loaded_scene, set_loaded_scene};
use phiber::state::{BKey as _, BState, ButsKey, CtxKey, action::apply_action, val::BValue};
use phiber::view::BAction;

/// The file open as the app opens it: its view layer seeded, its model in
/// scope for the rule that follows `ctx.object`.
fn opened() -> (BState, SceneModel) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/physics.blend");
    let model = Phx::open(&path).expect("physics.blend opens");
    let mut state = BState::new();
    seed_view_layer(&mut state, &model);
    set_loaded_scene(Some(model.clone()));
    (state, model)
}

/// Make `name` active through the reducer, as a click in the Outliner does.
fn activate(state: &mut BState, name: &str) {
    apply_action(state, &BAction::Set(CtxKey::Object.path(), name.into()));
}

/// What the row for `key` shows.
fn row(state: &BState, key: &str) -> BValue {
    let spec = prop_named(key).unwrap_or_else(|| panic!("{key} is a Properties row"));
    shown(spec, None, state)
}

fn float(state: &BState, key: &str) -> f32 {
    match row(state, key) {
        BValue::Float(v) => v,
        other => panic!("{key} is a number, not {other:?}"),
    }
}

fn near(state: &BState, key: &str, expected: f32) -> bool {
    (float(state, key) - expected).abs() < 1e-4
}

fn on(state: &BState, key: &str) -> bool {
    row(state, key) == BValue::Bool(true)
}

fn text(v: &str) -> BValue {
    BValue::Text(v.to_string())
}

fn pick(v: &str) -> BValue {
    BValue::Enum(v.to_string())
}

#[test]
fn opening_the_file_fills_the_panels_for_its_active_object() {
    let (s, model) = opened();
    // The file's active object is the last one the script made.
    assert_eq!(model.selection.active.as_deref(), Some("Jelly"));
    assert!(near(&s, "soft_body.mass", 2.0), "Jelly's soft body is in the panel before any click");
    clear_loaded_scene();
}

#[test]
fn the_crates_rigid_body_fills_the_rigid_body_panel() {
    let (mut s, _) = opened();
    activate(&mut s, "Crate");
    assert_eq!((row(&s, "rigid_body.type"), row(&s, "rigid_body.collision_shape")), (pick("ACTIVE"), pick("BOX")));
    assert!(near(&s, "rigid_body.mass", 2.5) && near(&s, "rigid_body.friction", 0.3) && near(&s, "rigid_body.restitution", 0.6));
    assert!(on(&s, "rigid_body.use_margin") && near(&s, "rigid_body.collision_margin", 0.02));
    assert!(near(&s, "rigid_body.linear_damping", 0.1) && near(&s, "rigid_body.angular_damping", 0.2));
    assert!(on(&s, "rigid_body.use_deactivation") && on(&s, "rigid_body.use_start_deactivated"));
    assert!(near(&s, "rigid_body.deactivate_linear_velocity", 0.5) && near(&s, "rigid_body.deactivate_angular_velocity", 0.6));
    assert!(on(&s, "rigid_body.enabled") && !on(&s, "rigid_body.kinematic"));
    assert_eq!(row(&s, "rigid_body.collision_collections"), text("0, 3"));

    // The tab follows the active object: Ball is animated, Pill is not
    // dynamic and is built from its final mesh.
    activate(&mut s, "Ball");
    assert!(near(&s, "rigid_body.mass", 0.7) && on(&s, "rigid_body.kinematic"));
    assert_eq!(row(&s, "rigid_body.collision_shape"), pick("SPHERE"));
    activate(&mut s, "Pill");
    assert!(!on(&s, "rigid_body.enabled"));
    assert_eq!(row(&s, "rigid_body.mesh_source"), pick("FINAL"));
    clear_loaded_scene();
}

#[test]
fn the_flags_cloth_fills_the_cloth_panel_and_the_rigid_body_rows_return_to_their_defaults() {
    let (mut s, _) = opened();
    activate(&mut s, "Crate");
    activate(&mut s, "Flag");
    assert_eq!(row(&s, "cloth.settings.quality"), BValue::Int(7));
    assert!(near(&s, "cloth.settings.mass", 0.4) && near(&s, "cloth.settings.air_damping", 2.0));
    assert!(near(&s, "cloth.settings.tension_stiffness", 25.0) && near(&s, "cloth.settings.compression_stiffness", 20.0));
    assert!(near(&s, "cloth.settings.shear_stiffness", 8.0) && near(&s, "cloth.settings.bending_stiffness", 1.5));
    assert!(near(&s, "cloth.settings.tension_damping", 7.0) && near(&s, "cloth.settings.compression_damping", 6.0));
    assert!(near(&s, "cloth.settings.shear_damping", 4.0) && near(&s, "cloth.settings.bending_damping", 0.8));
    assert_eq!(row(&s, "cloth.settings.vertex_group_mass"), text("Pin"));
    assert!(near(&s, "cloth.settings.pin_stiffness", 3.0));
    assert!(on(&s, "cloth.settings.use_pressure") && near(&s, "cloth.settings.uniform_pressure_force", 2.0));
    assert_eq!(row(&s, "cloth.collision_settings.collision_quality"), BValue::Int(4));
    assert!(near(&s, "cloth.collision_settings.distance_min", 0.02) && near(&s, "cloth.collision_settings.friction", 4.0));
    assert!(near(&s, "cloth.collision_settings.impulse_clamp", 0.5));
    assert!(on(&s, "cloth.collision_settings.use_self_collision"));
    assert!(near(&s, "cloth.collision_settings.self_distance_min", 0.01) && near(&s, "cloth.collision_settings.self_friction", 7.0));
    // Flag has no rigid body: Crate's numbers do not linger, and the rows
    // read as Blender's defaults for a new one.
    assert!(near(&s, "rigid_body.mass", 1.0) && near(&s, "rigid_body.friction", 0.5) && near(&s, "rigid_body.collision_margin", 0.04));
    assert_eq!(row(&s, "rigid_body.collision_shape"), pick("CONVEX_HULL"));
    assert!(!on(&s, "rigid_body.use_deactivation"));
    assert_eq!(row(&s, "rigid_body.collision_collections"), text(""));
    clear_loaded_scene();
}

#[test]
fn the_scene_tab_reads_the_rigid_body_world_and_the_gravity() {
    let (mut s, _) = opened();
    assert_eq!(row(&s, "rigidbody_world.substeps_per_frame"), BValue::Int(20));
    assert_eq!(row(&s, "rigidbody_world.solver_iterations"), BValue::Int(25));
    assert!(near(&s, "rigidbody_world.time_scale", 1.5));
    assert!(on(&s, "rigidbody_world.use_split_impulse") && on(&s, "rigidbody_world.enabled"));
    let gravity = prop_named("gravity").expect("the Gravity row");
    assert_eq!((shown(gravity, Some(0), &s), shown(gravity, Some(2), &s)), (BValue::Float(0.0), BValue::Float(-5.0)));
    // A count typed into the panel is believed over the file's.
    s.set(ButsKey::Value("rigidbody_world.substeps_per_frame"), BValue::Int(60).encode());
    assert_eq!(row(&s, "rigidbody_world.substeps_per_frame"), BValue::Int(60));
    // No file open: Blender's startup values.
    clear_loaded_scene();
    let fresh = BState::new();
    assert_eq!(row(&fresh, "rigidbody_world.substeps_per_frame"), BValue::Int(10));
    assert!(near(&fresh, "rigidbody_world.time_scale", 1.0));
    assert_eq!(shown(gravity, Some(2), &fresh), BValue::Float(-9.81));
}

#[test]
fn a_joint_a_collider_a_field_and_a_soft_body_fill_their_panels() {
    let (mut s, _) = opened();
    activate(&mut s, "Hinge");
    assert_eq!(row(&s, "rigid_body_constraint.type"), pick("HINGE"));
    assert_eq!((row(&s, "rigid_body_constraint.object1"), row(&s, "rigid_body_constraint.object2")), (text("Crate"), text("Ball")));
    assert!(on(&s, "rigid_body_constraint.use_breaking") && near(&s, "rigid_body_constraint.breaking_threshold", 15.0));
    assert!(!on(&s, "rigid_body_constraint.disable_collisions"));
    assert!(on(&s, "rigid_body_constraint.use_override_solver_iterations"));
    assert_eq!(row(&s, "rigid_body_constraint.solver_iterations"), BValue::Int(30));
    // The Z angle limit, -0.5 and 1.0 radians in the file, in degrees as
    // the panel prints an angle.
    assert!(on(&s, "rigid_body_constraint.use_limit_ang_z") && !on(&s, "rigid_body_constraint.use_limit_ang_x"));
    assert!(near(&s, "rigid_body_constraint.limit_ang_z_lower", -28.6479) && near(&s, "rigid_body_constraint.limit_ang_z_upper", 57.2958));

    activate(&mut s, "Spring");
    assert_eq!((row(&s, "rigid_body_constraint.type"), row(&s, "rigid_body_constraint.spring_type")), (pick("GENERIC_SPRING"), pick("SPRING1")));
    assert!(on(&s, "rigid_body_constraint.use_spring_x"));
    assert!(near(&s, "rigid_body_constraint.spring_stiffness_x", 12.5) && near(&s, "rigid_body_constraint.spring_damping_x", 0.7));
    assert!(on(&s, "rigid_body_constraint.use_limit_lin_x"));
    assert!(near(&s, "rigid_body_constraint.limit_lin_x_lower", -0.2) && near(&s, "rigid_body_constraint.limit_lin_x_upper", 0.3));

    activate(&mut s, "Motor");
    assert!(on(&s, "rigid_body_constraint.use_motor_lin") && on(&s, "rigid_body_constraint.use_motor_ang"));
    assert!(near(&s, "rigid_body_constraint.motor_lin_target_velocity", 2.0) && near(&s, "rigid_body_constraint.motor_lin_max_impulse", 50.0));
    assert!(near(&s, "rigid_body_constraint.motor_ang_target_velocity", 3.0) && near(&s, "rigid_body_constraint.motor_ang_max_impulse", 20.0));

    activate(&mut s, "Floor");
    assert!(near(&s, "collision.damping", 0.3) && near(&s, "collision.thickness_outer", 0.05) && near(&s, "collision.thickness_inner", 0.3));
    assert!(near(&s, "collision.cloth_friction", 6.0));
    assert!(!on(&s, "collision.use_culling") && on(&s, "collision.use_normal"));
    assert!(near(&s, "collision.permeability", 0.2) && near(&s, "collision.stickiness", 0.4) && on(&s, "collision.use_particle_kill"));
    assert!(near(&s, "collision.damping_factor", 0.25) && near(&s, "collision.friction_factor", 0.35));
    // Floor carries no constraint: Motor's rows are gone.
    assert_eq!((row(&s, "rigid_body_constraint.type"), row(&s, "rigid_body_constraint.object1")), (pick("FIXED"), text("")));
    assert!(!on(&s, "rigid_body_constraint.use_motor_lin"));

    activate(&mut s, "Gust");
    assert_eq!((row(&s, "field.type"), row(&s, "field.shape")), (pick("WIND"), pick("PLANE")));
    assert!(near(&s, "field.strength", 4.5) && near(&s, "field.flow", 0.3) && near(&s, "field.noise", 0.2));
    assert_eq!(row(&s, "field.seed"), BValue::Int(7));
    assert!(near(&s, "field.falloff_power", 1.5));
    assert!(on(&s, "field.use_max_distance") && near(&s, "field.distance_max", 12.0) && !on(&s, "field.use_min_distance"));

    activate(&mut s, "Jelly");
    assert!(near(&s, "soft_body.friction", 0.7) && near(&s, "soft_body.mass", 2.0) && near(&s, "soft_body.speed", 1.2));
    assert!(on(&s, "soft_body.use_goal"));
    assert!(near(&s, "soft_body.goal_default", 0.6) && near(&s, "soft_body.goal_min", 0.1) && near(&s, "soft_body.goal_max", 0.9));
    assert!(near(&s, "soft_body.goal_spring", 0.4) && near(&s, "soft_body.goal_friction", 0.3));
    assert_eq!(row(&s, "soft_body.vertex_group_goal"), text("Goal"));
    assert!(near(&s, "soft_body.pull", 0.6) && near(&s, "soft_body.push", 0.5) && near(&s, "soft_body.damping", 0.7));
    assert!(near(&s, "soft_body.bend", 0.3) && near(&s, "soft_body.shear", 0.8));
    assert_eq!((row(&s, "soft_body.plastic"), row(&s, "soft_body.spring_length")), (BValue::Int(2), BValue::Int(50)));
    assert!(on(&s, "soft_body.use_self_collision") && near(&s, "soft_body.ball_size", 0.3));
    assert_eq!(row(&s, "soft_body.collision_type"), pick("MANUAL"));
    assert_eq!(
        (row(&s, "soft_body.step_min"), row(&s, "soft_body.step_max"), row(&s, "soft_body.choke")),
        (BValue::Int(15), BValue::Int(350), BValue::Int(2))
    );
    clear_loaded_scene();
}
