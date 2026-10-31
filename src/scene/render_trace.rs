//! Where a render's time actually goes — one line per phase.
//!
//! A render is not one cost, it is six, and until this existed the only
//! numbers anyone had were the two in `times.txt`
//! (`frame posed_ms render_ms samples`). That is enough to say a frame was
//! slow and nothing at all about *which part* was slow, which is how
//! "rendering got slower" stayed an argument instead of a measurement.
//!
//! What is counted, and why each one is a suspect:
//!
//! | Phase | What it is | Why it can dominate |
//! |---|---|---|
//! | `sample` | `Tracer::sample_in` — the tracing itself | the work; the baseline everything else is measured against |
//! | `wait` | every device drain in the process (`Gpu::waited`) | a sample is cut into `slices` dispatches with a drain after **each** (`Tracer::sample_in`) — but the readbacks drain too, so this is not slice cost alone and must not be read as such |
//! | `converge` | the adaptive-sampling readback | one word, but a readback: staging buffer, map, drain. Every `BAdaptive::STEP` samples |
//! | `peek` | the progress readback | the whole frame back to the CPU, plus a clone of the RGBA buffer, every `PEEK_EVERY` |
//! | `finish` | denoise + glare + compositor | once, on the settled frame |
//! | `read` | the final readback | once |
//!
//! The totals are per render and printed once, so a headless run says where
//! its minutes went and an in-app F12 puts the same line in the journal.
//! Nothing here is on the GPU's critical path: a phase is two `Instant`
//! reads and an add.

use std::time::Duration;

/// One phase's tally: how many times, and how long in all.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BPhase {
    pub times: u32,
    pub total: Duration,
}

impl BPhase {
    pub fn add(&mut self, took: Duration) {
        self.times += 1;
        self.total += took;
    }

    fn ms(&self) -> f64 {
        self.total.as_secs_f64() * 1000.0
    }

    /// `12.3s over 48 (257ms each)`, or `-` when it never ran.
    fn line(&self) -> String {
        if self.times == 0 {
            return "-".to_string();
        }
        format!("{:.2}s over {} ({:.0}ms each)", self.total.as_secs_f64(), self.times, self.ms() / f64::from(self.times))
    }
}

/// Every phase of one render.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BRenderTrace {
    pub sample: BPhase,
    pub wait: BPhase,
    pub converge: BPhase,
    pub peek: BPhase,
    pub finish: BPhase,
    pub read: BPhase,
    /// Pixels still open at the most recent adaptive checkpoint. `None`
    /// before the first check or when adaptive sampling is off.
    pub open_pixels: Option<u32>,
    /// How many dispatches the last sample was cut into.
    pub slices: u32,
    /// Bands the frame was split into (`BBandList`).
    pub bands: u32,
    /// Pixels traced — the band's width × height, not the frame's.
    pub pixels: u64,
}

impl BRenderTrace {
    /// Everything measured, in the order it happens.
    fn phases(&self) -> [(&'static str, BPhase); 6] {
        [
            ("sample  ", self.sample),
            ("wait    ", self.wait),
            ("converge", self.converge),
            ("peek    ", self.peek),
            ("finish  ", self.finish),
            ("read    ", self.read),
        ]
    }

    /// The sum of what was measured. Not the wall clock: the phases nest
    /// (a `wait` happens inside a `sample`), so this is a budget and not a
    /// total, and `share` says so.
    fn measured(&self) -> Duration {
        self.phases().iter().map(|(_, p)| p.total).sum()
    }

    /// The report, one phase per line, each with its share of what was
    /// measured. `wall` is the render's real elapsed time, so the gap
    /// between it and the phases is the part nothing counted yet.
    pub fn report(&self, wall: Duration) -> String {
        let mut out = format!(
            "Render trace: {:.2}s wall, {} samples over {} band(s), {} dispatches a sample, {} px\n",
            wall.as_secs_f64(),
            self.sample.times,
            self.bands.max(1),
            self.slices.max(1),
            self.pixels,
        );
        let measured = self.measured().as_secs_f64().max(1e-9);
        for (name, phase) in self.phases() {
            let share = phase.total.as_secs_f64() / measured * 100.0;
            out.push_str(&format!("  {name}  {:>5.1}%  {}\n", share, phase.line()));
        }
        if let Some(open) = self.open_pixels {
            let share = open as f64 / self.pixels.max(1) as f64 * 100.0;
            out.push_str(&format!("  adaptive  {open} pixels still open ({share:.1}% of this band)\n"));
        }
        // How much of the render was spent waiting on the device at all.
        //
        // Measured against the WALL, not against `sample`: `Gpu::waited` is
        // process-wide and counts every drain, including the ones inside
        // the `converge` and `read` readbacks and any on another thread
        // (`Tracer::warm`). Divided by `sample` it therefore reads as more
        // than everything — a real run printed "167.8% was waiting for the
        // device between slices", which is both impossible and the wrong
        // claim twice over: those waits were mostly readbacks, and that
        // render had one dispatch a sample, so there were no slice drains
        // to attribute them to.
        if self.wait.times > 0 && wall > Duration::ZERO {
            let idle = self.wait.total.as_secs_f64() / wall.as_secs_f64() * 100.0;
            out.push_str(&format!("  {idle:.1}% of the wall was waiting on the device (slice drains and readbacks both)\n"));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_phase_that_never_ran_says_so_and_one_that_did_averages() {
        let mut p = BPhase::default();
        assert_eq!(p.line(), "-", "never run");
        p.add(Duration::from_millis(100));
        p.add(Duration::from_millis(300));
        assert_eq!(p.times, 2);
        assert_eq!(p.line(), "0.40s over 2 (200ms each)");
    }

    #[test]
    fn the_report_names_every_phase_and_the_idle_share() {
        let mut t = BRenderTrace { slices: 8, bands: 2, pixels: 1440 * 1440, ..Default::default() };
        t.sample.add(Duration::from_millis(1000));
        t.sample.add(Duration::from_millis(1000));
        t.wait.add(Duration::from_millis(500));
        t.read.add(Duration::from_millis(200));
        let report = t.report(Duration::from_millis(2400));
        for name in ["sample", "wait", "converge", "peek", "finish", "read"] {
            assert!(report.contains(name), "{name} is reported: {report}");
        }
        assert!(report.contains("2 samples over 2 band(s), 8 dispatches a sample"));
        // 500ms of waiting against 2400ms of wall — against the wall and
        // not against `sample`, which cannot bound it (see `report`).
        assert!(report.contains("20.8% of the wall was waiting"), "{report}");
        // A phase that never ran is still listed, as a dash — asserted on
        // the shape rather than the column padding, which is cosmetic.
        let idle = report.lines().find(|l| l.trim_start().starts_with("converge")).expect("a converge line");
        assert!(idle.ends_with("  -"), "a phase that never ran reads as a dash: {idle:?}");
    }
}
