//! Adaptive Sampling in the viewport: which modes take it, and when a
//! refining view has settled (`thread.rs` asks after each sample).

use phiray::{Gpu, Mode, Tracer};

/// The viewport's resolved rows for `mode`: the path tracer's alone. The
/// other modes are one deterministic dispatch or Workbench's fixed
/// anti-aliasing count, and take none.
pub(super) fn adaptive(mode: Mode, adapt: [f32; 4]) -> [f32; 4] {
    match mode {
        Mode::Rendered => adapt,
        _ => [0.0; 4],
    }
}

/// Whether the convergence flags can have changed at `sample`: the kernel
/// writes them only past the minimum and on the last sample of a step
/// (`adapt.wgsl`, Cycles' `AdaptiveSampling::need_filter`).
pub(super) fn checked(adapt: [f32; 4], sample: u32) -> bool {
    let step = (adapt[2] as u32).max(1);
    adapt[3] != 0.0 && sample > adapt[1] as u32 && (sample & (step - 1)) == step - 1
}

/// Whether every pixel has settled. Asked only where [`checked`] holds: the
/// answer is a readback of one word, and before a flag is written it can
/// only be "no". A device that declines the read is "not settled".
pub(super) fn settled(tracer: &Tracer, gpu: &Gpu, adapt: [f32; 4], sample: u32) -> bool {
    checked(adapt, sample) && tracer.unconverged(gpu) == Some(0)
}

#[cfg(test)]
mod converge_tests {
    use super::*;

    #[test]
    fn only_the_path_tracer_takes_adaptive_sampling() {
        let rows = [0.5, 32.0, 16.0, 1.0];
        assert_eq!(adaptive(Mode::Rendered, rows), rows);
        assert_eq!(adaptive(Mode::Solid, rows), [0.0; 4]);
        assert_eq!(adaptive(Mode::Wire, rows), [0.0; 4]);
        assert_eq!(adaptive(Mode::Preview, rows), [0.0; 4]);
    }

    #[test]
    fn the_question_is_asked_at_step_ends_past_the_minimum() {
        let rows = [0.5, 32.0, 16.0, 1.0];
        assert!(!checked(rows, 15), "inside the minimum");
        assert!(!checked(rows, 31), "a step end, still inside the minimum");
        assert!(!checked(rows, 32), "the minimum itself is not past it");
        assert!(checked(rows, 47), "47 & 15 == 15 and 47 > 32");
        assert!(!checked(rows, 48));
        assert!(!checked([0.0; 4], 47), "off is never asked");
    }

    /// The render thread ends a view whose pixels have all settled: a cube
    /// asked for sixteen million samples is done within minutes, which Max
    /// Samples alone could never be. The limit is long because the first
    /// run after a build compiles the kernels cold, which alone passed a
    /// minute here; warm, the view ends in under two seconds.
    #[test]
    fn a_settled_view_ends_long_before_max_samples() {
        use super::super::worker::{BRayJob, BRayShot, BRayWorker};
        use super::super::{scene, world};
        use crate::scene::{BObject3D, BShading, BShape};
        use gpui::{black, white};
        let Some(worker) = BRayWorker::start() else { return };
        let grey = white();
        let palette = super::super::super::mesh::BMeshPalette {
            shading: BShading::Rendered, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey, matcap: None, texture: false, studio: false,
        };
        let cube = BObject3D::Primitive { name: "Cube".into(), shape: BShape::Cube, x: 0.0, y: 0.0 };
        worker.send(BRayJob::Scene(1, 1, vec![cube], palette));
        let look = world::BRayLook { sky: black(), grid: white().opacity(0.35), grid_hide: 0, world: None, lights: Vec::new(), wire: None, view: [0.0, 0.0, 1.0, 0.0], environment: None, set: Default::default(), studio: false };
        let setup = crate::scene::camera::setup_from_state(&crate::state::BState::default());
        let cam = scene::camera(45.0, 30.0, (0.0, 0.0), 1.0, setup, 1.0, &look);
        let rows = crate::scene::render_set::BAdaptive::of_viewport(&Default::default()).words();
        worker.send(BRayJob::View(7, BRayShot { cam, w: 48, h: 48, mode: Mode::Rendered, samples: 1 << 24, aa: 8, denoise_from: None, adapt: rows, outline: std::sync::Arc::from([]) }));
        let (mut seq, mut done) = (0, false);
        let deadline = web_time::Instant::now() + std::time::Duration::from_secs(300);
        while !done && web_time::Instant::now() < deadline {
            if let Some(f) = worker.take(7, seq) {
                seq = f.seq;
                done = f.done && (f.w, f.h) == (48, 48);
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(done, "every pixel settled, so the view is finished");
    }
}
