//! The session journal: one file, from the moment the window opens, with
//! every line of the Info log at its time, every slow frame, the frame
//! summaries and the tracer's timings — so a slow session can be read
//! after the fact, action by action, with the gaps between them.
//!
//! It is written next to the executable (`target/release/phiber-session.log`
//! under a `cargo run`), where the workspace can see it, and falls back to
//! the temp directory. A new run starts it afresh. Only the running app
//! writes it (`app::run` arms it); a test's window writes nothing.
//!
//! The file itself, its line format, the `log` facade's sink and the panic
//! hook are `philog`'s — the same journal phiphx, phiray and simkit's
//! headless server can arm. What stays here is what is phiber's: the file's
//! name, the `PHIBER_NOTES` echo, and the per-frame drain of the in-app Info
//! log (`state::log`), which philog knows nothing about.

use crate::state::log;

pub struct BJournal;

impl BJournal {
    /// Where the journal is written: beside the executable, else in the
    /// temp directory.
    pub fn path() -> std::path::PathBuf {
        philog::beside_exe("phiber-session.log")
    }

    /// The running app: start the journal afresh at `path`.
    pub fn arm() {
        Self::arm_at(&Self::path());
    }

    fn arm_at(path: &std::path::Path) {
        if !philog::arm_at(path) {
            return;
        }
        Self::note(philog::header("Phiber session journal"));
        // A panic on any thread writes itself to the journal before the
        // process's own hook runs: a crash leaves its reason on disk, not
        // only on a console nobody had open.
        philog::note_panics();
    }

    /// Route the `log` crate — wgpu, gpui and the rest of the vendored stack —
    /// into the journal, so a diagnostic like the chosen GPU adapter and
    /// backend lands on disk (phiber installs no other logger, so these records
    /// were dropped before). Best effort: if a logger is already set, it is
    /// left in place. Info and above, which keeps the GPU adapter line
    /// without the debug/trace flood.
    pub fn bridge_log() {
        philog::install();
    }

    /// A line of its own — a slow frame, a frame summary, the tracer
    /// settling — at this moment. Nothing when the journal is not armed.
    pub fn note(text: impl AsRef<str>) {
        let at = web_time::Instant::now();
        // `PHIBER_NOTES=1`: a background render (`-b`) has no journal, so
        // the notes go to its console — the tracer's build cost among them.
        if std::env::var_os("PHIBER_NOTES").is_some() {
            println!("{}", text.as_ref());
        }
        philog::note_at(at, text.as_ref());
    }

    /// Each frame of the running app: the Info lines written since the
    /// last, each at its own time.
    pub fn each_frame() {
        let Some(mut feed) = philog::feed() else { return };
        let entries = log::entries();
        let written = feed.cursor();
        for entry in entries.iter().rev().filter(|e| e.seq > written) {
            feed.note_at(entry.at, &entry.text);
            feed.set_cursor(entry.seq);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_carries_the_time_from_the_start_and_the_gap_since_the_last() {
        let line = philog::line(std::time::Duration::from_millis(12_345), std::time::Duration::from_micros(2_500), "Opened Cube.blend");
        assert_eq!(line, "   12.345s  +     2.5ms  Opened Cube.blend");
    }

    #[test]
    fn an_armed_journal_files_the_info_lines_once_each_in_order() {
        let dir = std::env::temp_dir().join(format!("phiber-journal-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("phiber-session.log");
        BJournal::arm_at(&path);
        // Names no other test writes: the journal is one file for the
        // process, and the other tests' lines may land in it meanwhile.
        log::push("Opening Journal_Probe.blend");
        log::push("Opened Journal_Probe.blend: 3 objects");
        BJournal::each_frame();
        BJournal::each_frame();
        BJournal::note("slow frame 61.0ms (journal probe)");
        philog::disarm();
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[0].contains("session journal"), "{text}");
        let ours: Vec<_> = lines.iter().filter(|l| l.contains("Journal_Probe") || l.contains("journal probe")).collect();
        assert_eq!(ours.len(), 3, "each Info line once, then the note: {text}");
        assert!(ours[0].ends_with("Opening Journal_Probe.blend") && ours[1].ends_with("Opened Journal_Probe.blend: 3 objects"));
        assert!(ours[2].ends_with("slow frame 61.0ms (journal probe)"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
