//! A file's own Render settings reach the panel.
//!
//! Cycles stores its settings as ID properties on the scene
//! (`scene["cycles"]`), and only the ones that differ from the RNA default
//! are written. So opening Female-01 reads the counts it saved, and the
//! panel opens at those, the quality counts filling in only what it never
//! saved (`BRenderQuality::OPENING`), while `diffuse_bounces`, which the
//! file never wrote, stays at Blender's 4.

use phiber::phiphx::Phx;
use phiber::app::seed::seed_render_props;
use phiber::scene::render_set;
use phiber::state::BState;

const FILE: &str = "assets/CW-Fashion/Epic-Pods-Female-01/Female-01-Chatty-Whims-Rigged-Lace.blend";

fn opened() -> Option<(BState, phiber::phiphx::scene::SceneModel)> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FILE);
    let model = Phx::open(&path).ok()?;
    let mut state = BState::new();
    seed_render_props(&mut state, &model);
    Some((state, model))
}

#[test]
fn the_scenes_cycles_group_is_read() {
    let Some((_, model)) = opened() else { return };
    let cycles = &model.render.cycles;
    // The file was re-saved with 2022 and 2000 (Blender reads the same,
    // 2026-10-27); the counts are checked as present rather than pinned, so
    // the next re-save of the asset does not read as a broken reader.
    assert!(cycles.get("samples").and_then(|v| v.as_int()).is_some_and(|n| n > 0));
    assert!(cycles.get("preview_samples").and_then(|v| v.as_int()).is_some_and(|n| n > 0));
    // What the file does not store is absent, not zero — that distinction is
    // the whole reason the panel keeps RNA defaults.
    assert!(cycles.get("diffuse_bounces").is_none());
}

#[test]
fn the_panel_opens_the_files_own_samples() {
    use phiber::scene::render_quality::BRenderQuality;
    let Some((state, model)) = opened() else { return };
    let set = render_set(&state);
    let saved = |name: &str| model.render.cycles.get(name).and_then(|v| v.as_int()).map(|n| n.max(0) as u32);
    // The file's own count where it saved one, the quality count where it
    // saved none (`BRenderQuality::OPENING`).
    assert_eq!(Some(set.samples), BRenderQuality::opening("samples", saved("samples")));
    assert_eq!(Some(set.preview_samples), BRenderQuality::opening("preview_samples", saved("preview_samples")));
    // Untouched by the file, so Blender's own defaults stand.
    assert_eq!(set.diffuse_bounces, 4);
    assert!(set.denoise);
}

#[test]
fn a_cycles_file_opens_on_the_engine_that_renders_it() {
    let Some((state, model)) = opened() else { return };
    assert_eq!(model.render.engine, "CYCLES", "the file's own engine is read");
    // The row says the engine that will draw it (`seed_render_props`):
    // every engine but Workbench resolves to the tracer.
    assert_eq!(
        state.get(phiber::state::ButsKey::Value("engine")),
        Some(phiber::scene::render_quality::BRenderQuality::ENGINE),
    );
}

/// A file opens at its own counts (`BRenderQuality::OPENING`): one saved at
/// 25 opens at 25, and the quality counts fill in only what it never saved.
/// The panel shows the count that will render, and a count typed after
/// opening is believed.
#[test]
fn a_file_opens_at_its_own_counts_and_a_typed_count_is_believed() {
    use phiber::phiphx::blend::{BIdGroup, BIdValue};
    use phiber::phiphx::scene::SceneModel;
    use phiber::scene::render_quality::BRenderQuality;
    use phiber::scene::render_set;
    use phiber::state::{BState, ButsKey, val::BValue};

    let seeded = |saved: Option<i32>| {
        let mut model = SceneModel::default();
        let mut cycles = BIdGroup::default();
        if let Some(saved) = saved {
            cycles.set("samples", BIdValue::Int(saved));
        }
        model.render.cycles = cycles;
        let mut s = BState::new();
        phiber::app::seed::seed_render_props(&mut s, &model);
        phiber::scene::source::set_loaded_scene(Some(model));
        let samples = render_set(&s).samples;
        phiber::scene::source::set_loaded_scene(None);
        (s, samples)
    };

    let (s, samples) = seeded(Some(25));
    assert_eq!(samples, 25, "a file saved at 25 opens at 25");
    assert_eq!(render_set(&s).preview_samples, 1024, "a viewport count it never saved: Blender's 1024");
    assert_eq!(render_set(&s).adaptive_min_samples, 256, "a Min Samples it never saved: 256");
    let (_, samples) = seeded(Some(8192));
    assert_eq!(samples, 8192, "a file asking for more keeps its count");

    // Never set: the default, and Max Samples SHOWS it.
    let (s, samples) = seeded(None);
    assert_eq!(samples, BRenderQuality::SAMPLE_FLOOR);
    assert_eq!(s.get(ButsKey::Value("cycles.samples")), Some(BValue::Int(i64::from(BRenderQuality::SAMPLE_FLOOR)).encode()).as_deref());

    // Typed over afterwards: the panel wins.
    let (mut s, _) = seeded(Some(25));
    s.set(ButsKey::Value("cycles.samples"), BValue::Int(64).encode());
    assert_eq!(render_set(&s).samples, 64, "a count typed into the panel is the count that renders");
}
