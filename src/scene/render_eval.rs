//! The file as a final render evaluates it.
//!
//! The viewport's model is the file at its Viewport settings. A Subdivision
//! carries a second level for renders (`renderLevels`), and the CW pose
//! files set the bodies to Viewport 0, Render 2: F12 in the window drew
//! every figure unsubdivided, where the command line (which opens the file
//! for render, `cli::open`) drew them smooth.
//!
//! phiphx evaluates a modifier stack as it reads the file, so the render's
//! evaluation is a second read under `eval_for_render`. It is made on the
//! first F12 of a file that needs one (`render_levels_differ`) and kept
//! until another file is shown. The session's own edits are not lost by
//! it: they live in the state and are laid over whichever model is in
//! scope (`source::set_loaded_exact`).
//!
//! Not done: the read runs where F12 was pressed, so the window waits for
//! it (about a minute on the throne file) before the render starts.

use crate::phiphx::Phx;
use crate::phiphx::scene::SceneModel;
use std::sync::{Arc, Mutex};

/// The render's evaluation of the file shown under a key.
static HELD: Mutex<Option<(String, Arc<SceneModel>)>> = Mutex::new(None);

/// The model a final render of the file shown under `key` draws: `shown`
/// itself unless the file renders at other levels than it shows.
pub fn model(key: &str, shown: &Arc<SceneModel>) -> Arc<SceneModel> {
    let path = shown.source_path.as_deref().map(std::path::Path::new).filter(|p| p.is_file());
    let (Some(path), true) = (path, crate::phiphx::blend::render_levels_differ()) else {
        return Arc::clone(shown);
    };
    let mut held = HELD.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((_, kept)) = held.as_ref().filter(|(of, _)| of == key) {
        return Arc::clone(kept);
    }
    crate::state::log::push("Reading the file again at its Render subdivision levels".to_string());
    crate::phiphx::blend::eval_for_render(true);
    let read = Phx::open(path);
    crate::phiphx::blend::eval_for_render(false);
    match read {
        Ok(read) => {
            let read = Arc::new(read);
            *held = Some((key.to_string(), Arc::clone(&read)));
            read
        }
        Err(why) => {
            crate::state::log::push(format!("Could not read the file for render ({why:?}): rendering at the viewport's levels"));
            Arc::clone(shown)
        }
    }
}

/// Another file is shown: the held evaluation is of the last one.
pub fn forget() {
    *HELD.lock().unwrap_or_else(|e| e.into_inner()) = None;
}
