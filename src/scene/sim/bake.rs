//! The bake: a thread that steps the world from the cache's first frame to
//! its last, one frame at a time, and a shelf the UI reads the frames from.
//!
//! The solver carries velocities and contacts from frame to frame, so a
//! frame can only be had by stepping every frame before it. The thread
//! does that once per key (`input::key`) and keeps every frame's worlds,
//! so a scrub reads a frame rather than stepping to it, and playback reads
//! what the thread has already done. A new key — a settings edit, another
//! file — starts the bake over; the frames of the old one are dropped as
//! soon as it is asked for, and the thread drops its job at the next frame.
//!
//! A bake the Rigid Body World panel's Bake pinned (`baked`) is kept for
//! its scene through settings edits, as Blender locks a baked cache, until
//! Delete Bake.

use super::input::BSimInput;
use glam::Mat4;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock, PoisonError};

/// The object worlds of one frame, by object name.
pub type BSimFrame = Arc<HashMap<String, Mat4>>;

/// The frames baked for one key.
#[derive(Default)]
pub struct BBaked {
    /// Bumped by every new bake and every delete: a job whose number is not
    /// the shelf's stops.
    pub run: u64,
    pub key: u64,
    /// The scene part of the key: what Bake pins the cache to.
    pub scene: u64,
    pub start: i32,
    pub end: i32,
    /// `frames[k]` is frame `start + k`; the first is the start pose.
    pub frames: Vec<BSimFrame>,
    /// Pinned by Bake.
    pub baked: bool,
    /// Concave active bodies the solver takes as their hull (`World::hull_fallback`).
    pub hull: Vec<String>,
    /// The solver panicked: the frames stop where it did.
    pub failed: bool,
}

impl BBaked {
    /// Every frame of the range is in.
    pub fn done(&self) -> bool {
        self.failed || self.frames.len() as i64 > i64::from(self.end - self.start)
    }

    /// The frame to show at `frame`: before the range the start pose, after
    /// it the last; the newest baked while the bake has not got there.
    pub fn at(&self, frame: f32) -> Option<(usize, BSimFrame)> {
        let k = (frame.round() as i64 - i64::from(self.start)).clamp(0, i64::from(self.end - self.start)) as usize;
        let k = k.min(self.frames.len().checked_sub(1)?);
        Some((k, Arc::clone(&self.frames[k])))
    }

    /// Whether frame `frame` itself is baked.
    pub fn has(&self, frame: f32) -> bool {
        let k = (frame.round() as i64 - i64::from(self.start)).clamp(0, i64::from(self.end - self.start));
        self.failed || (k as usize) < self.frames.len()
    }
}

struct BJob {
    run: u64,
    input: BSimInput,
}

struct BBakery {
    shelf: Mutex<BBaked>,
    /// Signalled on every frame put and every new run.
    put: Condvar,
    tx: Mutex<Sender<BJob>>,
}

impl BBakery {
    /// The shelf, even after a panic poisoned its lock: what is on it is
    /// whole frames either way (`appkit/gpu/assets/async_loader.rs` in the
    /// reference code reads its queues the same way).
    fn shelf(&self) -> MutexGuard<'_, BBaked> {
        self.shelf.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn bakery() -> &'static BBakery {
    static BAKERY: OnceLock<BBakery> = OnceLock::new();
    BAKERY.get_or_init(|| {
        let (tx, rx) = channel();
        let _ = std::thread::Builder::new().name("phiber-sim".into()).spawn(move || run(&rx));
        BBakery { shelf: Mutex::new(BBaked::default()), put: Condvar::new(), tx: Mutex::new(tx) }
    })
}

/// Read the shelf.
pub fn with<R>(f: impl FnOnce(&BBaked) -> R) -> Option<R> {
    Some(f(&bakery().shelf()))
}

/// Make sure the shelf holds `key`'s bake (or a pinned one of `scene`):
/// when it does not, start it from `input()`.
pub fn ask(key: u64, scene: u64, input: impl FnOnce() -> Option<BSimInput>) {
    let b = bakery();
    let mut shelf = b.shelf();
    if shelf.key == key || (shelf.baked && shelf.scene == scene) {
        return;
    }
    let Some(input) = input() else { return };
    let run = shelf.run + 1;
    *shelf = BBaked { run, key, scene, start: input.start, end: input.end, ..BBaked::default() };
    b.put.notify_all();
    let _ = b.tx.lock().unwrap_or_else(PoisonError::into_inner).send(BJob { run, input });
}

/// Wait until `frame` of the bake on the shelf is in, or the shelf moves on.
pub fn wait(frame: f32) {
    let b = bakery();
    let mut shelf = b.shelf();
    let run = shelf.run;
    while shelf.run == run && !shelf.has(frame) {
        shelf = b.put.wait(shelf).unwrap_or_else(PoisonError::into_inner);
    }
}

/// Bake: pin what is on the shelf to its scene.
pub fn pin() {
    bakery().shelf().baked = true;
}

/// Delete Bake: the frames go; the next ask bakes again.
pub fn free() {
    let b = bakery();
    let mut s = b.shelf();
    *s = BBaked { run: s.run + 1, ..BBaked::default() };
    b.put.notify_all();
}

/// The thread: each job, frame by frame, until the shelf moves on.
fn run(rx: &Receiver<BJob>) {
    while let Ok(job) = rx.recv() {
        let b = bakery();
        if b.shelf().run != job.run {
            continue;
        }
        let baked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| bake(b, &job)));
        if baked.is_err() {
            log::error!("Rigid Body World: the solver stopped - the bake ends at the last frame it reached");
            let mut s = b.shelf();
            let ours = s.run == job.run;
            s.failed |= ours;
            drop(s);
            b.put.notify_all();
        }
    }
}

fn bake(b: &BBakery, job: &BJob) {
    let input = &job.input;
    let mut world = input.world();
    let hull: Vec<String> = world.hull_fallback.iter().map(|&i| input.objects[i].name.clone()).collect();
    if !hull.is_empty() {
        log::warn!("Rigid Body World: {} collide as their convex hull - an active Mesh shape is not concave here", hull.join(", "));
    }
    let frames = (input.end - input.start) as usize + 1;
    for k in 0..frames {
        if k > 0 {
            world.step(1.0 / input.fps);
        }
        let poses: HashMap<String, Mat4> = world.poses().map(|(i, m)| (input.objects[i].name.clone(), Mat4::from_cols_array_2d(&m))).collect();
        let mut shelf = b.shelf();
        if shelf.run != job.run {
            return;
        }
        shelf.hull.clone_from(&hull);
        shelf.frames.push(Arc::new(poses));
        b.put.notify_all();
    }
}
