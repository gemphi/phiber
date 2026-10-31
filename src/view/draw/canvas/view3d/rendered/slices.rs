//! The stepwise render's helpers: how many slices a sample is cut into,
//! the compositor pass over a finished band, and the note a refused render
//! leaves.

/// Why a render produced nothing, said once: the Info editor's report and
/// the session journal both carry it, so a blank Render Result is never the
/// only evidence that something went wrong.
pub(super) fn note_render_refused(why: String) {
    let line = format!("Render failed: {why}");
    crate::state::log::push(line.clone());
    crate::app::journal::BJournal::note(line);
}

/// A slice's intended length in the final render. Short enough that no
/// single dispatch approaches a GPU watchdog (two seconds on Windows),
/// long enough that the drain between slices stays a small share of the
/// work — `Tracer::sample_in` waits for the device after each one.
const RENDER_SLICE_MS: f64 = 120.0;

/// Slices for the very first sample, before anything has been timed.
pub(super) const FIRST_SLICES: u32 = 32;

/// The most a sample is cut into; a slice costs one device drain.
const MOST_SLICES: u32 = 64;

/// How many slices a sample takes, from how long the **tracing** took.
///
/// Settled from the **second** sample, never the first: `Tracer::warm`
/// compiles the other shading modes on another thread and holds the
/// device's pipeline lock while it does, so the first sample can spend
/// minutes blocked on a lock rather than on the GPU.
///
/// # The drain time is in this measurement, and that is tolerable
///
/// The watchdog cares about how long **one dispatch** runs on the GPU. A
/// sample's wall time is not that: it is the GPU work *plus* one device
/// drain per slice, so deciding the count from wall time measures the cost
/// of the slicing with the slicing already in it, at [`FIRST_SLICES`] — 32
/// drains deep.
///
/// This file used to claim that loop was ruinous on a cube — that 32 drains
/// make a one-millisecond sample read as ~2.2 s, settling the count at
/// nineteen for the whole render and turning seconds into most of an hour.
/// That was reasoning, not measurement, and measuring it refutes it.
/// `phiray`'s own `examples/slice_cost.rs` times a 12-triangle cube at
/// 1920 × 1080:
///
/// | slices | ms/sample |
/// |---|---|
/// | 1 | 18.0 |
/// | 4 | 15.6 |
/// | 16 | 15.3 |
/// | 32 | 20.3 |
/// | 64 | 26.6 |
///
/// A drain costs well under a millisecond here, not tens of them: 64 of
/// them cost 1.5× one, not hundreds. A cube at 2048 samples is therefore
/// about a minute at any count in this table, and the over-slicing is
/// cheap enough to leave alone. What actually made a trivial render *look*
/// like an hour was the `Remaining:` estimate charging every sample for the
/// scene setup (`render_status::remaining`), and what cost real time on a
/// short render was the convergence readback firing before the minimum
/// sample count (`stepwise.rs`) — neither of them this.
///
/// # Why subtracting the drains was tried and REVERTED
///
/// Deriving the count from `took - drained` (`Gpu::waited`) and re-reading
/// it every sample looks obviously right and is not. `Gpu::waited` is
/// **process-wide**, so that subtraction can underflow to zero; the count
/// then falls to one dispatch covering the whole frame; and a dispatch that
/// long is exactly what the Windows GPU watchdog resets the driver for. The
/// device is lost, the next drain never returns, and a 64-sample render
/// that took 39 s became a 25-minute silent hang.
///
/// The over-slicing is therefore a **safety margin, not an inefficiency**.
/// It does cost a drain per slice, and on a trivial scene that is most of
/// the sample — but the fix for that is a cheaper drain or a real
/// per-dispatch GPU timing, not a smaller margin guessed from wall clock.
pub(super) struct BSliceCut;

impl BSliceCut {
    /// How many slices a sample takes, from how long one took.
    ///
    /// Wall time, including the drains — see the warning above about why
    /// subtracting them is not safe here.
    pub(super) fn from(took: std::time::Duration) -> u32 {
        let ms = took.as_secs_f64() * 1000.0;
        if ms <= 0.0 {
            return FIRST_SLICES;
        }
        ((ms / RENDER_SLICE_MS).ceil() as u32).clamp(1, MOST_SLICES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn the_count_follows_the_sample_and_stays_inside_its_bounds() {
        // A sample at the slice budget is one dispatch; past it, more.
        assert_eq!(BSliceCut::from(Duration::from_millis(100)), 1);
        assert_eq!(BSliceCut::from(Duration::from_millis(2_400)), 20);
        // Never zero, never past the cap — a slice costs a device drain.
        assert_eq!(BSliceCut::from(Duration::ZERO), FIRST_SLICES, "unmeasured: the conservative start");
        assert_eq!(BSliceCut::from(Duration::from_secs(600)), MOST_SLICES);
    }
}

/// The scene's compositor over the finished render, as Blender's render
/// pipeline runs it: on the scene-linear pixels, before the view transform
/// (`scene/composite.rs`). Nothing happens when the file has no tree, it is
/// switched off, or nothing reaches its output. A band renders as its own
/// image, so each band is composited on its own — which is sound only
/// because every node that runs is per-pixel; a node that read a
/// neighbourhood would see a seam, and those are exactly the ones
/// `BCompTree::declined` refuses.
pub(super) fn composite(gpu: &phiray::Gpu, tracer: &mut phiray::Tracer) {
    use crate::scene::composite::BComposite;
    if let Some(tree) = crate::scene::composite::tree() {
        let run = tracer.composite(gpu, &tree);
        if run.any() {
            crate::app::journal::BJournal::note(run.line());
        }
    }
}
