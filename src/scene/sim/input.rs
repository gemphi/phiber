//! What the solver is handed: the file's Rigid Body World and bodies
//! (`SceneModel::physics`, phiphx `blend/physics/`) as phiray's own
//! `BlendWorld` and `BlendBody`, each body with its object's triangles and
//! world matrix — and the key a bake is filed under.
//!
//! Two things `phiray::sim::World::from_blend` asks of its caller:
//! - the triangles are **object space** and the matrix is the raw
//!   `matrix_world`: it takes the scale out of the matrix and bakes it into
//!   the shape, so world-space vertices would be scaled twice;
//! - `BlendObject::tris` is a borrow, so the buffers live in [`BSimInput`]
//!   for as long as the world is built from them.

use crate::phiphx::blend::physics::{BRigidBody, BRigidKind, BRigidShape, BRigidWorld};
use crate::phiphx::scene::SceneModel;
use crate::state::BState;
use phiray::sim::{BlendBody, BlendObject, BlendShape, BlendWorld, World};
use std::hash::{DefaultHasher, Hash, Hasher};

/// The Scene tab's Rigid Body World rows a bake reads, as the panel shows
/// them: typed values over the file's (`part::props::scene`).
#[derive(Clone, Debug, PartialEq)]
pub struct BWorldRows {
    pub enabled: bool,
    pub substeps: u32,
    pub iterations: u32,
    pub speed: f32,
}

impl BWorldRows {
    /// The rows as `s` shows them over `world`.
    pub fn read(s: &BState, world: &BRigidWorld) -> Self {
        use crate::state::val::BValue;
        let shown = |key: &str| crate::part::props::set::prop_named(key).map(|spec| crate::part::props::prop_view::shown(spec, None, s));
        let count = |key: &str, file: u32| match shown(key) {
            Some(BValue::Int(n)) => u32::try_from(n.max(1)).unwrap_or(file),
            _ => file,
        };
        Self {
            enabled: match shown("rigidbody_world.enabled") {
                Some(BValue::Bool(on)) => on,
                _ => world.enabled,
            },
            substeps: count("rigidbody_world.substeps_per_frame", world.substeps_per_frame),
            iterations: count("rigidbody_world.solver_iterations", world.solver_iterations),
            speed: match shown("rigidbody_world.time_scale") {
                Some(BValue::Float(v)) => v,
                _ => world.time_scale,
            },
        }
    }

    /// The file's own rows, for a caller with no panel.
    pub fn file(world: &BRigidWorld) -> Self {
        Self { enabled: world.enabled, substeps: world.substeps_per_frame, iterations: world.solver_iterations, speed: world.time_scale }
    }
}

/// One body's object: its settings, triangles and world, by name.
pub struct BSimObject {
    pub name: String,
    pub body: BlendBody,
    pub tris: Vec<[[f32; 3]; 3]>,
    pub world: [[f32; 4]; 4],
}

/// Everything one bake needs, owned.
pub struct BSimInput {
    pub world: BlendWorld,
    pub objects: Vec<BSimObject>,
    /// The point cache's frames: the first is the start pose.
    pub start: i32,
    pub end: i32,
    pub fps: f32,
}

impl BSimInput {
    /// `model`'s rigid bodies with `rows` over its world; `None` when it has
    /// no world, the world is off, or no object carries a body.
    pub fn of(model: &SceneModel, rows: &BWorldRows) -> Option<Self> {
        let physics = &model.physics;
        let world = physics.world.as_ref().filter(|_| rows.enabled)?;
        let objects: Vec<BSimObject> = physics
            .bodies
            .iter()
            .filter_map(|b| model.objects.iter().find(|o| o.name == b.object).map(|o| (b, o)))
            .map(|(b, o)| {
                let m = &o.mesh;
                let at = |k: u32| m.vertices.get(k as usize).copied().unwrap_or_default();
                BSimObject { name: o.name.clone(), body: body(b), tris: m.indices.chunks_exact(3).map(|t| [at(t[0]), at(t[1]), at(t[2])]).collect(), world: o.matrix_world }
            })
            .collect();
        if objects.is_empty() {
            return None;
        }
        let weight = world.effector_weights.gravity;
        Some(Self {
            world: BlendWorld { substeps_per_frame: rows.substeps, solver_iterations: rows.iterations, time_scale: rows.speed, gravity: physics.gravity.map(|g| g * weight) },
            objects,
            start: world.cache.frame_start,
            end: world.cache.frame_end.max(world.cache.frame_start),
            fps: model.timeline.fps.max(1.0),
        })
    }

    /// The world at the start frame. The borrows of `tris` end with the call.
    pub fn world(&self) -> World {
        let objects: Vec<BlendObject> = self.objects.iter().enumerate().map(|(obj, o)| BlendObject { body: o.body.clone(), tris: &o.tris, world: o.world, obj }).collect();
        World::from_blend(&self.world, &objects)
    }
}

/// Whether `model` has a world whose bodies fall: one active, dynamic body.
pub fn moves(model: &SceneModel) -> bool {
    let p = &model.physics;
    p.world.as_ref().is_some_and(|w| w.enabled) && p.bodies.iter().any(|b| b.kind == BRigidKind::Active && b.enabled && !b.kinematic)
}

/// The bake `model` and `rows` file under, and the scene part of it: the
/// physics settings, the frame rate and the rows over each body's object
/// as it starts. Another file with the same settings is another bake; a
/// scrub is not one. The scene part alone is what Bake pins a cache to.
pub fn key(model: &SceneModel, rows: &BWorldRows) -> (u64, u64) {
    let mut h = DefaultHasher::new();
    for o in model.objects.iter().filter(|o| model.physics.bodies.iter().any(|b| b.object == o.name)) {
        o.name.hash(&mut h);
        o.matrix_world.as_flattened().iter().for_each(|v| v.to_bits().hash(&mut h));
        (o.mesh.vertices.len(), o.mesh.indices.len()).hash(&mut h);
    }
    let scene = h.finish();
    format!("{:?}{rows:?}", model.physics).hash(&mut h);
    model.timeline.fps.to_bits().hash(&mut h);
    (h.finish(), scene)
}

/// One rigid body as phiray takes it.
fn body(b: &BRigidBody) -> BlendBody {
    BlendBody {
        passive: b.kind == BRigidKind::Passive,
        shape: match b.shape {
            BRigidShape::Box => BlendShape::Box,
            BRigidShape::Sphere => BlendShape::Sphere,
            BRigidShape::Capsule => BlendShape::Capsule,
            BRigidShape::Cylinder => BlendShape::Cylinder,
            BRigidShape::Cone => BlendShape::Cone,
            BRigidShape::ConvexHull => BlendShape::ConvexHull,
            BRigidShape::Mesh => BlendShape::Mesh,
            BRigidShape::Compound => BlendShape::Compound,
        },
        enabled: b.enabled,
        kinematic: b.kinematic,
        mass: b.mass,
        friction: b.friction,
        restitution: b.restitution,
        use_margin: b.use_margin,
        margin: b.margin,
        linear_damping: b.linear_damping,
        angular_damping: b.angular_damping,
        use_deactivation: b.use_deactivation,
        start_deactivated: b.start_deactivated,
        linear_sleep: b.linear_sleep,
        angular_sleep: b.angular_sleep,
        collections: b.collections,
    }
}
