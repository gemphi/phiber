//! The stepwise still render — the long path that yields partial images
//! while the tracer refines. Its helpers — how a sample is sliced, the
//! compositor pass, the refusal note — are `slices.rs`. Split so each file
//! sits under the 240-line read cap (`tests/level_structure.rs`).

use super::*;
use super::slices::{BSliceCut, FIRST_SLICES, composite, note_render_refused};



/// The same render, one sample at a time: `step` is called after each with
/// the samples taken so far and what was read back (`BStillAt`): usually
/// nothing, every `PEEK_EVERY` the bytes as they stand, and once the frame
/// has settled the finished image. Returning `false` from it stops the
/// render early, which is how a job is cancelled: the image as far as it got
/// — the bands finished and the one under way at its samples so far — is
/// handed to `step` as the finished one, as Blender keeps a cancelled
/// render's tiles.
///
/// A frame with more pixels than the device binds as one image
/// (`Tracer::max_pixels`) is rendered in horizontal bands, each a tile of
/// the frame (`Camera::tile`), and the bands are joined — so a 4K frame
/// renders whole on a device that binds 128 MiB. The samples reported step
/// over the bands together, so progress still ends at `samples`.
///
/// `still` is this without the reporting; a final render uses this one so the
/// Rendering space can show progress rather than freeze (`scene/render_job.rs`).
pub fn still_stepwise(
    objects: &[BObject3D],
    look: BStillLook,
    cam: phiray::Camera,
    w: u32,
    h: u32,
    samples: u32,
    denoise: bool,
    glow: Option<[f32; 3]>, // `Tracer::glow`: [threshold, intensity, radius]
    // Adaptive Sampling as Cycles resolves it (`BAdaptive::of`):
    // `[threshold, min samples, step, on]`. All zero is off, which is what
    // Workbench and the thumbnail paths pass.
    adapt: [f32; 4],
    // Render ▸ Sampling ▸ Time Limit in seconds, `0` for none. Shared out
    // across the bands by their rows, so a frame cut into four bands gives
    // each a quarter of the time rather than the first band all of it and
    // the last none — Cycles has no bands and simply stops when the clock
    // runs out (`Session::run_main_render_loop`).
    time_limit: f32,
    region: super::super::region::BStillRegion,
    // How often the image so far is read back for someone watching
    // (`PEEK_EVERY` for the Rendering space); `None` for a caller that only
    // wants the finished frame. A readback stalls the device and copies the
    // whole frame, so a headless render that drew nothing with them paid
    // for one every sample.
    peek: Option<std::time::Duration>,
    mut step: impl FnMut(u32, BStillAt) -> bool,
) -> Option<u32> {
    use crate::scene::BShading;
    // The process's device, not one of its own: a device opened per
    // render compiled every kernel again before its first sample (seconds
    // on a warm driver cache, half a minute on a cold one) and held a
    // second copy of the scene beside the viewport's. The shared one has
    // the kernels the viewport already built, and is the device Preferences
    // ▸ System chose.
    let Some(gpu) = phiray::Gpu::shared() else {
        note_render_refused("no GPU adapter".to_string());
        return None;
    };
    let (shading, palette) = (look.shading, look.palette());
    let cam = with_studio(cam, shading);
    let built = scene::build(objects, &palette);
    // The file summary counts every object's placed triangles. The GPU
    // uploads shared geometry once, so report both sides of that distinction
    // before moving the compact scene into the tracer. Pose 26 says 967
    // objects / 27.99M placed triangles at open but reaches this path as
    // about 615 instances over 7.28M unique faces.
    let scene_line = format!(
        "Render scene: {} visible instances, {} unique meshes, {} unique vertices, {} unique faces, {} materials, {:.1} MB texture atlas",
        built.insts.len(),
        built.meshes.len(),
        built.pos.len(),
        built.faces.len(),
        built.mats.len(),
        built.atlas.words().len() as f64 * 4.0 / 1_048_576.0,
    );
    crate::app::journal::BJournal::note(scene_line.clone());
    if crate::cli::headless() {
        println!("{scene_line}");
    }
    // A scene the device will not take is reported with its reason rather
    // than leaving a blank Render Result behind: pose 26 declined here and
    // the info bar still said "Finished" (`phiray::Tracer::why_not`).
    if let Some(why) = phiray::Tracer::why_not(&built, gpu.max_binding(), phiray::Tracer::chunks_for(&gpu)) {
        note_render_refused(format!("the GPU declined this scene - {why}"));
        return None;
    }
    let mut tracer = phiray::Tracer::from_scene(&gpu, built)?;
    // A Workbench render's `render_aa` is its anti-aliasing count; the
    // path tracer's samples are its own.
    tracer.set_samples(samples);
    tracer.set_aa_samples(samples);
    tracer.set_adaptive(adapt);
    // What Adaptive Sampling resolved to, said once: the three panel rows
    // are "automatic" by default and Cycles' rules turn them into quite
    // different numbers, so without this there is no way to tell from a
    // log which settings a render actually used.
    let adaptive_line = if adapt[3] != 0.0 {
        format!(
            // The panel's Noise Threshold, then the one the kernel compares
            // (× `BAdaptive::SCALE`): the log said 0.1000 for a file's 0.02,
            // which read as the wrong threshold when it was the right one.
            "Adaptive sampling: noise threshold {:.4} (compared as {:.4}), min {} samples, checked every {}",
            adapt[0] / crate::scene::render_set::BAdaptive::SCALE,
            adapt[0],
            adapt[1] as u32,
            adapt[2] as u32,
        )
    } else {
        "Adaptive sampling: off — every pixel takes every sample".to_string()
    };
    crate::app::journal::BJournal::note(adaptive_line.clone());
    if crate::cli::headless() {
        println!("{adaptive_line}");
    }
    tracer.set_mode(match shading {
        BShading::Wireframe => phiray::Mode::Wire,
        BShading::Solid => phiray::Mode::Solid,
        BShading::Material => phiray::Mode::Preview,
        BShading::Rendered => phiray::Mode::Rendered,
    });
    // The pixels drawn: the region of the frame — two pixels wider each
    // way, for the same reason as the bands' overlap, and cut back after —
    // or the frame itself.
    // A band is drawn some rows over into its neighbours and laid in
    // without them, and the region with that margin around it: the effects
    // and the anti-aliasing gather each read a pixel's neighbours and clamp
    // at a tile's edge, and the overlap is what makes the seam invisible —
    // two rows, or SMAA's reach when the single pass leaves it on the
    // finished frame (`Tracer::seam_rows`).
    // A final render keeps the GPU busy far longer than the viewport's
    // five-second readback guard allows, and a readback that gives up
    // returns an EMPTY buffer that every caller reads as "the GPU
    // declined" — which is exactly how pose 26 came back blank while the
    // card was working. The budget is the render's for as long as it runs
    // (`phiray::PatienceFor`), and the viewport keeps its own.
    let _patience = phiray::PatienceFor::render(std::time::Duration::from_secs(600));
    let reach = tracer.seam_rows();
    let (rx, ry, rw, rh) = region.drawn(w, h, reach);
    let bands = BBandList::of(rw, rh, BBandList::most(&gpu), reach);
    let mut drawn = BStill::empty(rw, rh);
    // The first sample is shown at once, then one every `PEEK_EVERY`.
    let mut peeked = web_time::Instant::now() - peek.unwrap_or_default();
    // How a sample is cut into dispatches. The viewport has always done
    // this (`BSlicer`); the final render did not, and that is what killed
    // the app: a compute dispatch that runs past the platform's GPU
    // watchdog — two seconds on Windows — has the display driver reset
    // under it, and wgpu reports the device lost. Pose 26 was ~4 s a
    // sample, so every final render of it was over the limit.
    let mut slices = FIRST_SLICES;
    let mut seen = 0u32;
    let mut told = false;
    let mut told_done = false;
    let mut told_limit = false;
    // Where the time goes, phase by phase (`scene::render_trace`). Two
    // `Instant` reads an event; nothing here touches the device.
    let began_all = web_time::Instant::now();
    let mut trace = crate::scene::BRenderTrace { bands: bands.list.len() as u32, pixels: u64::from(rw) * u64::from(rh), ..Default::default() };
    // `Gpu::waited` is process-wide and monotonic, so the render's share is
    // the difference across it.
    let waited_before = phiray::Gpu::waited();
    for (k, (y0, rows)) in bands.list.iter().enumerate() {
        let (over, under) = (reach.min(*y0), reach.min(rh - y0 - rows));
        // This band's share of the time limit, by its rows of the frame.
        let budget = if time_limit > 0.0 { time_limit * (*rows as f32 / rh.max(1) as f32) } else { 0.0 };
        let began = web_time::Instant::now();
        let tiled = bands.list.len() > 1 || region.pixels.is_some();
        let band = phiray::Camera { tile: if tiled { [rx, ry + y0 - over, w, h] } else { [0; 4] }, ..cam };
        loop {
            let at = web_time::Instant::now();
            let drained_before = phiray::Gpu::waited().0;
            // `go` always says carry on: a final render is never abandoned
            // for a newer view, so `sample_in` never returns `None` here.
            // `go` always says carry on, so the only `None` here is a
            // device that did not drain inside its budget
            // (`Tracer::sample_in`). That used to be impossible and the
            // call used to `expect` it; making the drain bounded made it
            // reachable, and a lost device must be reported rather than
            // panicked on — the render keeps whatever it has drawn.
            let Some((taken, done)) = tracer.sample_in(&gpu, band, rw, rows + over + under, slices, || true) else {
                note_render_refused(format!(
                    "the GPU did not finish a dispatch within {:.0}s — the display driver may have reset it (sample {}, {was_slices} dispatches)",
                    gpu.patience().as_secs_f64(),
                    trace.sample.times + 1,
                    was_slices = slices,
                ));
                note_trace(&mut trace, waited_before, began_all);
                step(bands.progress(k, trace.sample.times, tracer.samples()), BStillAt::Settled(region.framed(drawn, w, h, reach)));
                return Some(trace.sample.times);
            };
            let took = at.elapsed();
            // The GPU work this sample did: its wall time less the device
            // drains the slicing itself cost. That is what the watchdog
            // limits, and the only honest input to the slice count —
            // see `BSliceCut`.
            let drained = std::time::Duration::from_nanos(phiray::Gpu::waited().0.saturating_sub(drained_before));
            let work = took.saturating_sub(drained);
            trace.sample.add(took);
            trace.slices = slices;
            let was = slices;
            seen += 1;
            // Settled ONCE, from the second sample, and deliberately
            // conservative. Re-deriving it per sample from
            // `took - drained` was tried and REVERTED: `Gpu::waited` is
            // process-wide, so that subtraction can underflow to zero, the
            // count then falls to a single dispatch covering the whole
            // frame, and one dispatch that long is what the Windows GPU
            // watchdog resets the driver for. The device is then lost and
            // the drain below never returns — a 64-sample render that took
            // 39 s became a 25-minute hang. The over-slicing is the thing
            // preventing that, not an inefficiency to tune away.
            if seen == 2 {
                slices = BSliceCut::from(took);
                if !told {
                    told = true;
                    crate::app::journal::BJournal::note(format!(
                        "Render pace: {:.0}ms a sample over {rw}×{} px ({:.0}ms of it draining {was} dispatches); the rest take {slices}",
                        took.as_secs_f64() * 1000.0,
                        rows + over + under,
                        drained.as_secs_f64() * 1000.0,
                    ));
                }
            }
            let _ = work;
            // Every pixel settled: the band is finished, whatever the
            // sample count says. Cycles ends a render here rather than
            // running on.
            // The convergence check is a readback — staging buffer, map,
            // device drain — so it is timed even though it returns one
            // word (`Tracer::unconverged`).
            // The same three conditions the kernel itself applies before it
            // will write a convergence flag (`adapt.wgsl`, Cycles'
            // `AdaptiveSampling::need_filter`): adaptive sampling on, the
            // minimum samples past, and a step boundary. Asking before all
            // three hold cannot return anything but "not settled", because
            // no flag has been written — and the question is not free: the
            // readback measured 25ms against 1ms of tracing a sample, so
            // on a 256-sample frame the sixteen pointless asks were 0.40s
            // of a 1.59s render. The minimum was the condition missing
            // here, and it is the one that matters, since Phiber's default
            // minimum is 256 samples.
            let min_samples = adapt[1] as u32;
            let settled = adapt[3] != 0.0
                && taken > min_samples
                && (taken & (crate::scene::render_set::BAdaptive::STEP - 1)) == crate::scene::render_set::BAdaptive::STEP - 1
                && {
                    let at = web_time::Instant::now();
                    let open = tracer.unconverged(&gpu);
                    trace.converge.add(at.elapsed());
                    trace.open_pixels = open;
                    open == Some(0)
                };
            if settled && !told_done {
                told_done = true;
                crate::app::journal::BJournal::note(format!(
                    "Adaptive sampling: every pixel settled at sample {} of {}",
                    taken + 1,
                    tracer.samples(),
                ));
            }
            // The clock: a band past its share of the time limit is
            // finished at this sample, whatever Max Samples says.
            let out_of_time = budget > 0.0 && began.elapsed().as_secs_f32() >= budget;
            if out_of_time && !told_limit && !done && !settled {
                told_limit = true;
                crate::app::journal::BJournal::note(format!(
                    "Time limit: {time_limit:.0}s reached at sample {} of {}",
                    taken + 1,
                    tracer.samples(),
                ));
            }
            let done = done || settled || out_of_time;
            // Reading the image back is the slow part, so progress carries a
            // count, the bytes now and then (`PEEK_EVERY`), and the whole
            // picture only at the end.
            let go_on = done || {
                let at = if peek.is_some_and(|every| peeked.elapsed() >= every) {
                    peeked = web_time::Instant::now();
                    // The whole frame back to the CPU plus a clone of the
                    // RGBA buffer: the most expensive thing that is not
                    // tracing, which is why it is on a timer at all.
                    let began = web_time::Instant::now();
                    let mut peek = drawn.rgba.clone();
                    BStill::lay_rgba(&mut peek, &tracer.pixels(&gpu), *y0 as usize, *rows as usize, over as usize, rw as usize);
                    let shown = BStillAt::Peek(BStill::peek(region.framed_rgba(peek, w, h, reach)));
                    trace.peek.add(began.elapsed());
                    shown
                } else {
                    BStillAt::Counting
                };
                step(bands.progress(k, taken + 1, tracer.samples()), at)
            };
            if done || !go_on {
                let finishing = web_time::Instant::now();
                if denoise && done {
                    tracer.denoise(&gpu);
                }
                if done {
                    // Glare runs scene-linear, before the overlays.
                    if let Some([t, i, r]) = glow { tracer.glow(&gpu, t, i, r); }
                    composite(&gpu, &mut tracer);
                }
                trace.finish.add(finishing.elapsed());
                let reading = web_time::Instant::now();
                let bytes = BStill::read(&tracer, &gpu);
                trace.read.add(reading.elapsed());
                drawn.lay(&bytes, *y0 as usize, *rows as usize, over as usize, rw as usize);
                if !go_on {
                    note_trace(&mut trace, waited_before, began_all);
                    step(bands.progress(k, taken + 1, tracer.samples()), BStillAt::Settled(region.framed(drawn, w, h, reach)));
                    return Some(taken + 1);
                }
                break;
            }
        }
    }
    note_trace(&mut trace, waited_before, began_all);
    step(tracer.samples(), BStillAt::Settled(region.framed(drawn, w, h, reach)));
    Some(tracer.samples())
}

/// Fill in the device-wait phase from phiray's counters and publish the
/// report: the session journal (the Info editor reads it) and, for a
/// headless run, stdout — `cli::render` has no journal to show.
fn note_trace(trace: &mut crate::scene::BRenderTrace, before: (u64, u32), began: web_time::Instant) {
    let (ns, times) = phiray::Gpu::waited();
    trace.wait = crate::scene::BPhase {
        times: times.saturating_sub(before.1),
        total: std::time::Duration::from_nanos(ns.saturating_sub(before.0)),
    };
    let report = trace.report(began.elapsed());
    for line in report.lines() {
        crate::app::journal::BJournal::note(line.to_string());
    }
    if crate::cli::headless() {
        print!("{report}");
    }
}

/// Material Preview is lit by the studio image `scene::build` puts in the
/// atlas: a camera that was not told so (built by hand rather than by
/// `lit`) gets the lighting words here, or every surface comes out a dark
/// mirror of the raw image.
pub fn with_studio(mut cam: phiray::Camera, shading: crate::scene::BShading) -> phiray::Camera {
    if shading == crate::scene::BShading::Material && cam.preview == [0; 4] && super::super::studio::environment().is_some() {
        super::super::world::studio_lighting(&mut cam);
    }
    cam
}
