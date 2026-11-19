//! Adaptive Sampling as Cycles resolves it: the Render panel's three rows
//! turned into the numbers the kernel actually uses.
//!
//! Split out of `render_set.rs`, which holds the panel's other rows. The
//! panel values live on [`BRenderSet`]; resolving them is this file's one
//! job, so it sits beside that type rather than inside it.

use super::render_set::BRenderSet;

/// Adaptive Sampling as Cycles resolves it, ported from
/// `intern/cycles/scene/integrator.cpp`'s `Integrator::get_adaptive_sampling`
/// and `integrator/adaptive_sampling.cpp`.
///
/// The three panel rows are not the numbers the kernel uses. Blender's
/// manual says so without saying what the numbers are:
///
/// > **Noise Threshold** — "Setting this to 0 makes Cycles guess an
/// > automatic value based on the total sample count."
/// > **Min Samples** — "When set to 0 (default), Cycles automatically picks
/// > a value determined by the Noise Threshold."
///
/// Those guesses are below, exactly as Cycles makes them. Until now phiber
/// showed all three rows and honoured none of them: `BRenderSet` did not
/// read them and phiray had no notion of adaptive sampling, so every pixel
/// of every render got the full sample count — 2048 on pose 26, where the
/// flat floor settles in a handful.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BAdaptive {
    /// Off: every pixel takes every sample.
    pub use_it: bool,
    /// The threshold the kernel compares against (already × 5).
    pub threshold: f32,
    /// No pixel stops before this many samples.
    pub min_samples: u32,
    /// Convergence is only tested on samples where `needs_check` holds.
    pub step: u32,
}

impl BAdaptive {
    /// Cycles tests convergence every sixteenth sample, and the step must
    /// be a power of two for its bitwise test to work
    /// (`adaptive_sampling.cpp` asserts it).
    pub const STEP: u32 = 16;

    /// What Cycles multiplies the panel's Noise Threshold by before the
    /// kernel compares against it — so a file's 0.02 is tested as 0.1.
    pub const SCALE: f32 = 5.0;

    /// Resolve the panel's three rows against `samples`, the render's total.
    pub fn of(set: &BRenderSet, samples: u32) -> Self {
        Self::resolved(set.adaptive, set.adaptive_threshold, set.adaptive_min_samples, samples)
    }

    /// The viewport's own three rows (`use_preview_adaptive_sampling`,
    /// `preview_adaptive_threshold`, `preview_adaptive_min_samples`) against
    /// the viewport's Max Samples, by the same rules: Blender keeps a
    /// coarser threshold for the view than for a render.
    pub fn of_viewport(set: &BRenderSet) -> Self {
        Self::resolved(set.preview_adaptive, set.preview_adaptive_threshold, set.preview_adaptive_min_samples, set.preview_samples)
    }

    /// What the tracer takes (`phiray::Tracer::set_adaptive`): `[threshold,
    /// min samples, step, on]`, all zero when off.
    pub fn words(&self) -> [f32; 4] {
        match self.use_it {
            true => [self.threshold, self.min_samples as f32, self.step as f32, 1.0],
            false => [0.0; 4],
        }
    }

    fn resolved(on: bool, row_threshold: f32, row_min: u32, samples: u32) -> Self {
        if !on {
            return Self { use_it: false, threshold: 0.0, min_samples: 0, step: Self::STEP };
        }
        // "if (clamped_aa_samples > 0 && adaptive_threshold == 0.0f)
        //      threshold = max(0.001f, 1.0f / (float)aa_samples);"
        let mut threshold = if samples > 0 && row_threshold == 0.0 {
            0.001_f32.max(1.0 / samples as f32)
        } else {
            row_threshold
        };
        // "Threshold 0.1 -> 32, 0.01 -> 64, 0.001 -> 128." Taken from the
        // threshold BEFORE the ×5 below, as Cycles takes it.
        let min_samples = if threshold > 0.0 && row_min == 0 {
            (16.0 / threshold.powf(0.3)).ceil() as u32
        } else {
            row_min
        }
        .max(4);
        // "Arbitrary factor that makes the threshold more similar to what
        // it was before, and gives arguably more intuitive values."
        threshold *= Self::SCALE;
        Self { use_it: true, threshold, min_samples, step: Self::STEP }
    }

    /// `AdaptiveSampling::need_filter`: convergence is tested after the
    /// minimum is past, on the last sample of each step.
    pub fn needs_check(&self, sample: u32) -> bool {
        self.use_it && sample > self.min_samples && (sample & (self.step - 1)) == self.step - 1
    }
}

#[cfg(test)]
mod adaptive_tests {
    use super::*;

    fn set(adaptive: bool, threshold: f32, min: u32) -> BRenderSet {
        BRenderSet { adaptive, adaptive_threshold: threshold, adaptive_min_samples: min, ..Default::default() }
    }

    /// The three rows resolve exactly as `Integrator::get_adaptive_sampling`
    /// resolves them, including both "automatic" cases.
    #[test]
    fn cycles_rules_for_the_automatic_values() {
        // Blender's own default row: 0.01, min 0.
        let a = BAdaptive::of(&set(true, 0.01, 0), 2048);
        assert!((a.threshold - 0.05).abs() < 1e-6, "0.01 × 5 = 0.05, got {}", a.threshold);
        // The comment in Cycles promises 0.01 -> 64.
        assert_eq!(a.min_samples, 64, "16 / 0.01^0.3");
        assert_eq!(BAdaptive::of(&set(true, 0.1, 0), 2048).min_samples, 32, "Cycles: 0.1 -> 32");
        assert_eq!(BAdaptive::of(&set(true, 0.001, 0), 2048).min_samples, 128, "Cycles: 0.001 -> 128");

        // Threshold 0 is "automatic": max(0.001, 1 / samples), then × 5.
        // Below 1000 samples 1/samples clears the 0.001 floor, so this is
        // the case that exercises the division; at 2048 it would not, and
        // the floor below would be tested twice.
        let auto = BAdaptive::of(&set(true, 0.0, 512), 512);
        assert!((auto.threshold - 5.0 / 512.0).abs() < 1e-6, "1/512 clears the floor, then ×5: got {}", auto.threshold);
        let floored = BAdaptive::of(&set(true, 0.0, 0), 100_000);
        assert!((floored.threshold - 0.005).abs() < 1e-6, "the 0.001 floor, then ×5");

        // An explicit minimum is honoured, but never below four.
        assert_eq!(BAdaptive::of(&set(true, 0.01, 7), 2048).min_samples, 7);
        assert_eq!(BAdaptive::of(&set(true, 0.01, 1), 2048).min_samples, 4, "max(4, …)");

        assert!(!BAdaptive::of(&set(false, 0.01, 0), 2048).use_it, "the switch is honoured");
    }

    /// The viewport resolves its own rows, and off packs to all zero.
    #[test]
    fn the_viewport_takes_its_own_rows() {
        let d = BRenderSet::default();
        let a = BAdaptive::of_viewport(&d);
        assert!(a.use_it, "Blender ships the viewport's adaptive sampling on");
        assert!((a.threshold - 0.5).abs() < 1e-6, "0.1 × 5, got {}", a.threshold);
        assert_eq!(a.min_samples, 32, "Cycles: 0.1 -> 32");
        assert_eq!(a.words(), [0.5, 32.0, 16.0, 1.0]);
        let off = BRenderSet { preview_adaptive: false, ..d };
        assert_eq!(BAdaptive::of_viewport(&off).words(), [0.0; 4]);
    }

    /// `need_filter`: nothing before the minimum, then every sixteenth.
    #[test]
    fn convergence_is_tested_every_sixteenth_sample_after_the_minimum() {
        let a = BAdaptive::of(&set(true, 0.01, 0), 2048); // min 64
        assert!(!a.needs_check(15), "inside the minimum");
        assert!(!a.needs_check(64), "the minimum itself is not past it");
        assert!(a.needs_check(79), "79 & 15 == 15, and 79 > 64");
        assert!(!a.needs_check(80));
        assert!(a.needs_check(95));
        assert!(!BAdaptive::of(&set(false, 0.01, 0), 2048).needs_check(95), "off means never");
    }
}
