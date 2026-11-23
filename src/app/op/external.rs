//! File - External Data: the images a file could not read.
//!
//! `file.report_missing_files` lists them in the Info editor;
//! `file.find_missing_files` searches a folder the user points at for
//! them, by file name (`phiphx`'s `texture_find`, Blender's
//! `BKE_bpath_missing_files_find`). Blender repoints each image and
//! reloads it in place; phiphx resolves images while it reads the file, so
//! the file is read again with the folder among the places searched. The
//! session's edits live in the state and stay.

use super::super::BPhiberApp;
use crate::part::text::info::{self, BReportLevel};
use crate::state::log;
use gpui::Context;

/// What a read warning about an image that was not found starts with
/// (`phiphx::scene::texture_miss`, and Blender's own line for the world).
const MISSING_LEAD: [&str; 3] = ["Cannot read", "World: cannot read", "… and"];

impl BPhiberApp {
    /// The read's lines about images that were not found.
    fn missing_lines(&self) -> Vec<String> {
        let Some((_, model)) = self.scene_cache.as_ref() else { return Vec::new() };
        model.report.warnings.iter().filter(|line| line.contains("images not found") || MISSING_LEAD.iter().any(|lead| line.starts_with(lead))).cloned().collect()
    }

    /// Report Missing Files: one line per image, or that none is missing.
    pub(crate) fn report_missing(&mut self) {
        if self.scene_cache.is_none() {
            self.say(BReportLevel::Info, "No file is open".to_string());
            return;
        }
        let lines = self.missing_lines();
        if lines.is_empty() {
            self.say(BReportLevel::Info, "No missing files".to_string());
            return;
        }
        self.say(BReportLevel::Warning, format!("{} missing file line(s) - File - External Data - Find Missing Files searches a folder for them", lines.len()));
        for line in lines {
            self.say(BReportLevel::Warning, line);
        }
    }

    /// Find Missing Files in `folder`: search it from here on, and read
    /// the file again so the images found there are used.
    pub(crate) fn find_missing_in(&mut self, folder: &str, cx: &mut Context<Self>) {
        let dir = std::path::Path::new(folder);
        if !dir.is_dir() {
            self.say(BReportLevel::Error, format!("Not a folder: '{folder}'"));
            return;
        }
        let listed = crate::phiphx::scene::point_at_texture_folder(dir);
        self.say(BReportLevel::Info, format!("Searching '{folder}' for missing files: {listed} file(s) listed"));
        let Some((path, _)) = self.scene_cache.clone() else { return };
        let before = self.missing_lines().len();
        if before == 0 {
            self.say(BReportLevel::Info, "No missing files".to_string());
            return;
        }
        log::push("Reading the file again with that folder searched".to_string());
        self.start_import(path, cx);
    }

    fn say(&mut self, level: BReportLevel, text: String) {
        log::push(text.clone());
        info::push(&mut self.state, level, text);
    }
}
