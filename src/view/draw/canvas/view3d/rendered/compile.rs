//! The kernels compiled off the render thread.
//!
//! A mode's kernel compiles the first time it is dispatched, and on a
//! driver with no shader cache that can take seconds — so switching from
//! Rendered to Solid used to stall on Workbench's compile with the old
//! image still on screen. The first frame up, the thread hands the rest to
//! a thread of its own (`Tracer::warm`). Each compile holds only its own
//! kernel's slot (`phiray::Gpu::pipeline`): Material Preview takes minutes
//! on some drivers, and the viewport's Solid draws through it meanwhile.

use phiray::{Gpu, Tracer};
use web_time::Instant;

/// Whether this render thread has already started the compiles.
pub(super) struct BWarmed(bool);

impl BWarmed {
    pub(super) fn new() -> Self {
        Self(false)
    }

    /// Compile the other kernels once, on a thread of its own. Does
    /// nothing on every call after the first.
    pub(super) fn once(&mut self, gpu: &Gpu) {
        if self.0 {
            return;
        }
        self.0 = true;
        let gpu = gpu.clone();
        crate::app::offload::offload("phiray.warm", move || {
            let at = Instant::now();
            let each: Vec<String> = Tracer::warm(&gpu).into_iter().map(|(name, ms)| format!("{name} {ms:.0}")).collect();
            crate::app::journal::BJournal::note(format!("Tracer: kernels compiled in {:.0}ms ({})", at.elapsed().as_secs_f64() * 1000.0, each.join(", ")));
        });
    }
}
