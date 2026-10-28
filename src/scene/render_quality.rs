//! The quality policy: what a render defaults to when nothing else says.
//!
//! One owner for two rules that used to live only on the command line
//! (`cli::set`), while the app's own F12 took neither of them:
//!
//! - **Phiray is the engine.** A `.blend` saved by Blender names EEVEE or
//!   Cycles and neither is ours to run, so the file's engine is a
//!   *preference* and not an instruction. `BLENDER_WORKBENCH` is the
//!   exception, because Workbench is an engine phiber really does have
//!   (`render_engine.rs`, `workbench_render.cc`) — a file saved under it
//!   still rasterizes.
//! - **4096 samples is the DEFAULT** for a final render — Blender's own
//!   (`addon/properties.py`), and the top of the 2048–4096 the team's
//!   quality research settled on, adaptive sampling stopping the pixels
//!   that settle sooner.
//! - **A file opens at its own counts** ([`Self::OPENING`]). Max Samples,
//!   Viewport and Min Samples take the quality counts only where the file
//!   saved none, as Blender fills in an unsaved property with its default.
//!   For a while a file saved at 25 opened at 4096 (the team's turntable
//!   files saved 25 and every F12 was noise); that overrode what the
//!   file's author chose, and the user asked for the file's count back on
//!   2026-10-27. A file meant for final renders saves its counts.
//!
//!   Until 2026-10-19 the render itself took `asked.max(2048)`, so typing
//!   64 rendered 2048 and the row silently did nothing; a count typed after
//!   the file is open is believed, as Blender believes Max Samples.

use super::render_engine::BRenderEngine;

pub struct BRenderQuality;

impl BRenderQuality {
    /// The engine a render uses unless something names another — phiber's
    /// own tracer, named as itself rather than as Cycles.
    pub const ENGINE: &'static str = "PHIRAY";

    /// The samples a render takes when nothing asks for a count.
    ///
    /// Still named `SAMPLE_FLOOR` because that is what it was and what
    /// the command line calls it; it is a default now, not a minimum.
    pub const SAMPLE_FLOOR: u32 = 4096;

    /// The counts a file opens at where it saved none: `(Cycles property, quality)`.
    /// Max Samples at the render default; the viewport at Blender's 1024;
    /// and every pixel at least 256 samples before adaptive sampling may
    /// stop it, which the research asks for over Cycles' automatic 64.
    pub const OPENING: &'static [(&'static str, u32)] = &[
        ("samples", Self::SAMPLE_FLOOR),
        ("preview_samples", 1024),
        ("adaptive_min_samples", 256),
    ];

    /// The count the Render panel opens `name` at, given what the file
    /// saved: the file's own, else the quality count — `None` for a
    /// property this rule leaves to the file altogether.
    pub fn opening(name: &str, saved: Option<u32>) -> Option<u32> {
        let quality = Self::OPENING.iter().find(|(n, _)| *n == name)?.1;
        Some(saved.unwrap_or(quality))
    }

    /// The samples a render actually takes: **what was asked for**, and
    /// the default only when nothing asked. A render takes at least one.
    ///
    /// `draft` no longer changes this. It existed to lift a floor that no
    /// longer exists, and is kept as an argument so the command line does
    /// not have to change.
    pub fn samples(asked: Option<u32>, _draft: bool) -> u32 {
        asked.map_or(Self::SAMPLE_FLOOR, |asked| asked.max(1))
    }

    /// The engine to render a file with when the Engine row has never been
    /// touched. Workbench is honoured because phiber has it; every other
    /// name a `.blend` can carry resolves to the tracer.
    pub fn engine_for_file(named: &str) -> BRenderEngine {
        match BRenderEngine::of(named) {
            BRenderEngine::Workbench => BRenderEngine::Workbench,
            _ => BRenderEngine::Phiray,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_count_that_was_asked_for_is_the_count_that_renders() {
        assert_eq!(BRenderQuality::samples(None, false), 4096, "nothing asked: the default");
        // The row is believed. This used to assert the opposite — that
        // twenty was "lifted to the floor" — which is exactly why typing
        // 64 into Max Samples rendered 2048 and looked like a dead
        // control.
        assert_eq!(BRenderQuality::samples(Some(20), false), 20, "pose 26's twenty is honoured");
        assert_eq!(BRenderQuality::samples(Some(64), false), 64, "a 64-sample render takes 64");
        assert_eq!(BRenderQuality::samples(Some(4096), false), 4096, "and a big count is kept");
        assert_eq!(BRenderQuality::samples(Some(0), false), 1, "a render takes at least one sample");
        assert_eq!(BRenderQuality::samples(Some(64), true), 64, "draft changes nothing now");
    }

    #[test]
    fn a_file_opens_at_its_own_counts_and_the_quality_ones_fill_in() {
        // What the file saved is what it opens at, fewer or more.
        assert_eq!(BRenderQuality::opening("samples", Some(25)), Some(25));
        assert_eq!(BRenderQuality::opening("preview_samples", Some(25)), Some(25));
        assert_eq!(BRenderQuality::opening("adaptive_min_samples", Some(0)), Some(0));
        assert_eq!(BRenderQuality::opening("samples", Some(8192)), Some(8192));
        // Never saved: the quality counts.
        assert_eq!(BRenderQuality::opening("samples", None), Some(4096));
        assert_eq!(BRenderQuality::opening("preview_samples", None), Some(1024));
        assert_eq!(BRenderQuality::opening("adaptive_min_samples", None), Some(256));
        // Everything else is the file's.
        assert_eq!(BRenderQuality::opening("diffuse_bounces", Some(1)), None);
    }

    #[test]
    fn a_files_engine_resolves_to_the_one_we_have() {
        // The two Blender engines phiber cannot run are rendered by the
        // tracer, and say so.
        for named in ["CYCLES", "BLENDER_EEVEE_NEXT", "BLENDER_EEVEE", "", "SOME_ADDON"] {
            assert_eq!(BRenderQuality::engine_for_file(named), BRenderEngine::Phiray, "{named}");
        }
        // Workbench is ours, so it stays itself and keeps rasterizing.
        assert_eq!(BRenderQuality::engine_for_file("BLENDER_WORKBENCH"), BRenderEngine::Workbench);
        assert!(!BRenderQuality::engine_for_file("BLENDER_WORKBENCH").traces());
        assert!(BRenderQuality::engine_for_file("CYCLES").traces());
    }
}
