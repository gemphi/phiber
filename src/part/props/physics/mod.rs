//! Physics context - `bl_ui/properties_physics_*.py`: the Add Physics
//! buttons, then one panel tree per physics kind an object can carry, in
//! the order Blender registers them. Each kind is a file beside this one
//! (`deflect.rs` for Collision and Force Fields, `cloth.rs`, `soft.rs`,
//! `body.rs`, `joint.rs`) holding its rows and what fills them.
//!
//! The rows show the active object's settings as the file has them
//! (`SceneModel::physics`, read by phiphx's `blend/physics/`): `seed` runs
//! from `active::seed` when a file opens and whenever `ctx.object` changes,
//! writing each kind the object carries under the RNA keys the rows read
//! (`ctx.buts.rigid_body.mass`). A kind the object does not carry has its
//! rows cleared, so the controls fall back to the RNA defaults declared on
//! the specs - what Blender's Add ▸ Rigid Body starts from - rather than
//! keep the last object's numbers. The Add Physics buttons are untouched,
//! and nothing simulates yet (`phiray/docs/ROADMAP.md` §2).

mod body;
mod cloth;
mod deflect;
mod joint;
mod soft;

use super::prop::*;
use super::set::BSectionSpec;
use super::set::sec;
use crate::phiphx::blend::physics::BlendPhysics;
use crate::state::{BState, ButsKey, val::BValue};

static ADD: &[BPropSpec] = &[e(
    "add_physics",
    "Add Physics",
    &[
        en("FORCE_FIELD", "Force Field"),
        en("COLLISION", "Collision"),
        en("CLOTH", "Cloth"),
        en("DYNAMIC_PAINT", "Dynamic Paint"),
        en("SOFT_BODY", "Soft Body"),
        en("FLUID", "Fluid"),
        en("RIGID_BODY", "Rigid Body"),
        en("RIGID_BODY_CONSTRAINT", "Rigid Body Constraint"),
    ],
)];

pub static SECTIONS: &[BSectionSpec] = &[
    sec("Add Physics", ADD),
    deflect::FORCE_FIELDS,
    deflect::COLLISION,
    cloth::SECTION,
    soft::SECTION,
    body::SECTION,
    joint::SECTION,
];

/// The active object's physics into the tab's rows: each kind it carries
/// from the file, each it does not cleared to the RNA defaults.
pub fn seed(state: &mut BState, physics: &BlendPhysics, name: &str) {
    deflect::seed_field(state, physics.fields.iter().find(|f| f.object == name));
    deflect::seed_collider(state, physics.colliders.iter().find(|c| c.object == name));
    cloth::seed(state, physics.cloths.iter().find(|c| c.object == name));
    soft::seed(state, physics.softs.iter().find(|s| s.object == name));
    body::seed(state, physics.bodies.iter().find(|b| b.object == name));
    joint::seed(state, physics.joints.iter().find(|j| j.object == name));
}

/// One row's value, stored under the key its own spec declares - so a
/// value never lands where no control reads it (`active::set::write`).
fn put(state: &mut BState, key: &str, value: BValue) {
    if let Some(spec) = super::set::prop_named(key) {
        state.set(ButsKey::Value(spec.key), value.encode());
    }
}

/// Every row of one kind's panel tree back to its RNA default.
fn clear(state: &mut BState, section: &BSectionSpec) {
    for spec in section.all_props() {
        state.remove(ButsKey::Value(spec.key));
    }
}
