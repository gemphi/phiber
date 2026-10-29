//! Rigid bodies: the file's Rigid Body World simulated by phiray's CPU
//! solver (`phiray::sim`, XPBD) and shown at the timeline's frame.
//!
//! Blender steps its world as the frame advances and keeps each frame in
//! the world's point cache, so going back reads the cache instead of
//! stepping. Here the whole cache range is baked on a thread of its own
//! (`bake.rs`) as soon as a scene with falling bodies is shown, and each
//! render pass reads the shown frame's object worlds off the shelf
//! ([`worlds`]); `source::set_loaded_at` hands them to `motion::set_moved`,
//! where the scene's objects take them over their animation. While the
//! bake has not reached the shown frame the pass shows the newest frame
//! there is, and the window draws again ([`waiting`]) until it lands; F12
//! and the command line wait for it (`exact`).
//!
//! `input.rs` translates `SceneModel::physics` into phiray's `BlendWorld`
//! and `BlendBody`, and files the bake under a key of the settings, the
//! frame rate and the bodies' objects, so a settings edit bakes again and a
//! scrub does not. The Rigid Body World panel's Cache row ([`bake_row`])
//! pins a bake (Bake) or drops it (Delete Bake) through [`BRigidBakeRule`].
//!
//! Not yet: kinematic and animated bodies stay where the file put them
//! (`World::drive`), joints are not solved, and a body edited with G / R /
//! S bakes from where the file had it (ROADMAP.md §2, M2).

mod bake;
mod input;
#[cfg(test)]
mod tests;

pub use bake::BSimFrame;
pub use input::{BSimInput, BWorldRows, moves};

use crate::phiphx::scene::SceneModel;
use crate::state::{BKey as _, BPath, BState, OpKey, rule::BRule};
use crate::view::BView;
use crate::view::prim::{BAction, BControl};
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Weak};

thread_local! {
    /// The bake and frame the pass showed, folded: what the scene's object
    /// list is rebuilt on (`source::scene_objects`).
    static STAMP: Cell<u64> = const { Cell::new(0) };
    /// The pass showed another frame than the one it wanted.
    static WAITING: Cell<bool> = const { Cell::new(false) };
    /// The keys of the model asked last, so a pass on the same model and
    /// rows does not hash its physics again.
    static KEYS: RefCell<Option<(Weak<SceneModel>, BWorldRows, (u64, u64))>> = const { RefCell::new(None) };
    /// The bake whose hull fallback the Info log was told of.
    static NOTED: Cell<u64> = const { Cell::new(0) };
}

/// The object worlds the simulation puts `model`'s bodies at `frame`, by
/// object name; `None` for a scene with nothing falling. `exact` waits for
/// the frame; otherwise the newest baked frame stands in for it.
pub fn worlds(model: &Arc<SceneModel>, s: &BState, frame: Option<f32>, exact: bool) -> Option<BSimFrame> {
    let shown = frame.filter(|_| moves(model)).and_then(|frame| {
        let rows = BWorldRows::read(s, model.physics.world.as_ref()?);
        let (key, scene) = keys(model, rows.clone());
        bake::ask(key, scene, || BSimInput::of(model, &rows));
        if exact {
            bake::wait(frame);
        }
        bake::with(|b| (b.key == key || (b.baked && b.scene == scene)).then(|| (b.at(frame), !b.has(frame), b.run, b.key, b.hull.clone()))).flatten()
    });
    let Some((at, waiting, run, key, hull)) = shown else {
        WAITING.with(|w| w.set(false));
        STAMP.with(|t| t.set(0));
        return None;
    };
    WAITING.with(|w| w.set(waiting));
    STAMP.with(|t| t.set(run.rotate_left(20) ^ at.as_ref().map_or(u64::MAX, |(k, _)| *k as u64)));
    if !hull.is_empty() && NOTED.with(|n| n.replace(key)) != key {
        crate::state::log::push(format!("Rigid Body World: {} collide as their convex hull - an active Mesh shape is not concave here", hull.join(", ")));
    }
    at.map(|(_, frame)| frame)
}

/// The bake and frame the last pass showed; 0 for none.
pub fn stamp() -> u64 {
    STAMP.with(Cell::get)
}

/// The pass showed another frame than the shown one: draw again.
pub fn waiting() -> bool {
    WAITING.with(Cell::get)
}

fn keys(model: &Arc<SceneModel>, rows: BWorldRows) -> (u64, u64) {
    KEYS.with(|cell| {
        let mut memo = cell.borrow_mut();
        let hit = memo.as_ref().filter(|(m, r, _)| std::ptr::eq(m.as_ptr(), Arc::as_ptr(model)) && *r == rows).map(|(_, _, keys)| *keys);
        match hit {
            Some(keys) => keys,
            None => {
                let keys = input::key(model, &rows);
                *memo = Some((Arc::downgrade(model), rows, keys));
                keys
            }
        }
    })
}

/// The Cache row's key (`part::props::scene`).
pub const CACHE_ROW: &str = "rigidbody_world.point_cache";

/// The Scene tab's Rigid Body World ▸ Cache row: what the cache holds,
/// then Bake and Delete Bake (`ptcache.bake` / `ptcache.free_bake`).
pub fn bake_row(_: &BState) -> BView {
    let said = bake::with(|b| match (b.key, b.baked, b.done()) {
        (0, _, _) => "No cache".to_string(),
        (_, true, _) => format!("Baked {} frames", b.frames.len()),
        (_, false, true) => format!("{} frames in memory", b.frames.len()),
        (_, false, false) => format!("Baking {} of {}", b.frames.len(), b.end - b.start + 1),
    });
    let button = |label: &str, value: &str| BView::Button(BControl::new().with_label(label).with_action(BAction::Set(OpKey::RigidBake.path(), value.to_string())));
    BView::column(vec![BView::label(said.unwrap_or_default()), BView::row(vec![button("Bake", "bake"), button("Delete Bake", "free")])])
}

/// Bake and Delete Bake: `bake` bakes the open scene's world, if it is not
/// on the shelf, and pins it; `free` drops the cache.
pub struct BRigidBakeRule;

impl BRule for BRigidBakeRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::RigidBake.path()
    }

    fn after(&self, _: &BPath, value: &str, s: &mut BState) {
        s.remove(OpKey::RigidBake);
        match value {
            "bake" => {
                let Some(model) = crate::scene::source::loaded_shared() else { return };
                let Some(world) = model.physics.world.as_ref() else { return };
                let rows = BWorldRows::read(s, world);
                let (key, scene) = keys(&model, rows.clone());
                bake::ask(key, scene, || BSimInput::of(&model, &rows));
                bake::pin();
                crate::state::log::push("Rigid Body World: baked".to_string());
            }
            "free" => {
                bake::free();
                crate::state::log::push("Rigid Body World: bake deleted".to_string());
            }
            _ => {}
        }
    }
}
