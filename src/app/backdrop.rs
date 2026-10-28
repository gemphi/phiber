//! The window's backdrop: the desktop blurred behind the window while the
//! glass mode is in force (`theme/style/glass::backdrop`), opaque otherwise.
//!
//! The blur is the system compositor's (`WindowBackgroundAppearance::Blurred`:
//! acrylic on Windows 10 1809 and later, a visual-effect view on macOS, KWin
//! blur on Wayland), so it costs the app nothing. Where the system has none
//! (X11, older Windows, `GPUI_DISABLE_DIRECT_COMPOSITION`), the translucent
//! ground shows the desktop unblurred or black; Reduce Transparency is the way
//! back to solid. gpui turns subpixel text off while the window is not opaque.

use crate::state::BState;
use gpui::{Window, WindowBackgroundAppearance};
use std::cell::Cell;

thread_local! {
    /// What the window wears now; `None` until the first frame sets it.
    static WORN: Cell<Option<bool>> = const { Cell::new(None) };
}

/// Set the window's backdrop from the state, when it changes.
pub fn wear(window: &Window, state: &BState) {
    let blurred = crate::theme::style::glass::backdrop(state);
    if WORN.replace(Some(blurred)) == Some(blurred) {
        return;
    }
    window.set_background_appearance(match blurred {
        true => WindowBackgroundAppearance::Blurred,
        false => WindowBackgroundAppearance::Opaque,
    });
}
