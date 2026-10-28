//! Welcome window — the Phiber home screen.
//!
//! This was Blender's splash (`WM_OT_splash`, `WM_MT_splash`): a banner
//! over New File | Recent Files and Open… | links. It is a **home** now:
//!
//! ```text
//!                 P H I B E R            ← on the window's own backdrop
//!      Imagine, Apply, Unleash - The Phi way ∞
//!          ( New File | Recent Files | Learn )   ← one ground, the tabs
//!        ┌──┐ ┌──┐ ┌──┐ ┌──┐                  ← the tab's cards
//!        └──┘ └──┘ └──┘ └──┘
//! ```
//!
//! 1. **The logo stands on the backdrop.** Nothing is painted under it or
//!    its line: it is the window's name, not a card.
//! 2. **The tabs are the only bar.** One ground holds them
//!    (`BStrip::Tabs`); no other background is drawn above the cards.
//! 3. **The tab changes what is under it** (`tab::BWelcomeTab`): New File's
//!    workspaces; Open…, Recover and the recent files; or Learn.
//! 4. **Every choice is a card**, four to a row, centred
//!    (`BView::Thumbnail`), as large as the window makes room for
//!    ([`card_size`]).
//!
//! Opened by `Phiber ▸ Welcome…`, and shown once at startup (the seed
//! writes `ctx.modal = "splash"`). Dismissed like any modal.

pub mod home;
/// Six terrains traced off the welcome video's frames.
pub mod terrain;
pub mod recent;
pub mod set;
pub mod tab;

use crate::state::BKey as _;
use crate::state::AppKey;
use crate::state::OpKey;
use crate::state::ModalKey;
use crate::state::FileKey;
use crate::state::BState;
use crate::state::val::V_EMPTY;
use crate::state::val::V_FALSE;
pub use tab::BWelcomeTab;
use crate::state::val::V_MODAL_FILE_BROWSER;
use crate::view::prim::BAction;
use crate::view::prim::BCard;
use crate::view::prim::BStrip;
use crate::view::BThumb;
use crate::view::BView;
use crate::view::BViewItem;
pub use set::APP_NAME;
pub use set::BSplashEntry;
pub use set::EMPTY_RECENT;
pub use set::FOOT_LINKS;
pub use set::GETTING_STARTED;
pub use set::LINKS;
pub use set::RECENT_MAX;
pub use set::RECENT_ROWS;
pub use set::TEMPLATES;

#[derive(Clone, Copy, Debug, Default)]
pub struct BSplashModal;

/// A card's edge on the Welcome window at its design size (the Splash
/// spec's frame), in px: four to a row fit that frame with room either side.
pub const CARD: f32 = 112.0;
/// The largest a card grows: Blender's largest thumbnail
/// (`FileSelectParams.thumbnail_size` runs to 256).
pub const CARD_MAX: f32 = 256.0;
/// How many cards stand in one row at most: the design frame holds two rows
/// of four, and a tab with more cards than that (New File's ten
/// workspaces) takes a fifth across rather than a third row, which pushed
/// the logo up under the title bar (`across`).
pub const ACROSS: usize = 5;
/// The fewest cards to a row, while there are that many.
const ACROSS_LEAST: usize = 4;

/// How many of `n` cards stand in a row: two rows when they fit in two of
/// at most `ACROSS`, never fewer than four to a row.
pub fn across(n: usize) -> usize {
    n.div_ceil(2).clamp(ACROSS_LEAST, ACROSS)
}

/// A card's edge for the room the window gives the Welcome form.
///
/// The Splash spec is the form's design frame: the logo, the tabs and two
/// rows of `CARD` cards fill it. A larger window scales the cards by as much
/// as it is larger than that frame on its tighter axis, so a bigger window
/// shows a bigger grid rather than the same small one on more backdrop, and
/// two rows of the scaled cards still fit under the logo and the tabs.
/// Restored, the form is `RESTORED_SHARE` of the window. Before the window
/// has reported its size (or in a test), the cards are `CARD`.
pub fn card_size(s: &BState) -> f32 {
    let size = |key| s.get(key).and_then(|v| v.parse::<f32>().ok());
    let (Some(width), Some(height)) = (size(AppKey::WindowWidth), size(AppKey::WindowHeight)) else {
        return CARD;
    };
    let share = match s.get(ModalKey::Maximized) {
        Some(V_FALSE) => crate::modal::set::RESTORED_SHARE,
        _ => 1.0,
    };
    // The form stands under the modal's title bar.
    let title = crate::theme::style::metric::DEFAULT_DIMENSIONS.header_height;
    let frame = crate::modal::BModalType::Splash.spec();
    let scale = (width * share / frame.width).min((height * share - title) / frame.height);
    (CARD * scale).clamp(CARD, CARD_MAX)
}

/// One choice as a card `size` px across: its icon (or the file's own
/// preview), its name, and a line under it.
fn tile(icon: &str, label: &str, subtitle: &str, preview: &str, action: Option<BAction>, size: f32) -> BView {
    BView::thumbnail(BThumb {
        preview_path: preview.to_string(),
        icon: icon.to_string(),
        label: label.to_string(),
        subtitle: subtitle.to_string(),
        size,
        selected: false,
        action,
    })
}

fn link_tile(l: &BSplashEntry, size: f32) -> BView {
    tile(l.icon, l.label, "", "", Some(l.link_action()), size)
}

/// `views` centred on their own line, as tall as they are: a row of three
/// groups (`draw::row`) — an empty one either side and these between — so
/// the middle group sits on the window's centre.
fn centred(views: Vec<BView>) -> BView {
    BView::row([vec![BView::indent(0), BView::spacer()], views, vec![BView::spacer(), BView::indent(0)]].concat())
}

/// `tiles`, `across` of them to a row, each row centred.
fn grid(tiles: Vec<BView>) -> Vec<BView> {
    tiles.chunks(across(tiles.len())).map(|row| centred(row.to_vec())).collect()
}

/// New File: one card per workspace a file can start in. A card starts the
/// file (`wm.read_homefile`), which closes this window, as File ▸ New does.
fn new_file(size: f32) -> Vec<BView> {
    TEMPLATES.iter().map(|t| tile(t.icon, t.label, "", "", Some(BAction::Set(OpKey::NewFile.path(), t.target.to_string())), size)).collect()
}

/// Recent Files: Open… and Recover Last Session (the file opened last, open
/// again), then the files opened before it, each on its own preview with the
/// folder it is in under its name, or Getting Started when nothing has been
/// opened yet.
fn recent(s: &BState, size: f32) -> Vec<BView> {
    let mut tiles = vec![tile("folder_open", "Open\u{2026}", "", "", Some(BAction::Set(ModalKey::Open.path(), V_MODAL_FILE_BROWSER.to_string())), size)];
    let paths = recent::recent_paths(s);
    let Some((last, earlier)) = paths.split_first() else {
        tiles.extend(GETTING_STARTED.iter().map(|l| link_tile(l, size)));
        return tiles;
    };
    // The file the last session had open, opened again, on its own preview
    // and named once: this card is that file, so the list after it starts
    // with the one before. Listing it again repeated its name and folder in
    // the card beside this one.
    let name = recent::display_name(last);
    tiles.push(tile("history", "Recover Last Session", name, last, Some(BAction::Set(FileKey::Selected.path(), last.clone())), size));
    tiles.extend(earlier.iter().take(RECENT_ROWS).map(|path| {
        let name = recent::display_name(path);
        let folder = recent::display_name(recent::parent_dir(path));
        tile("file_blend", name, folder, path, Some(BAction::Set(FileKey::Selected.path(), path.clone())), size)
    }));
    if earlier.len() > RECENT_ROWS {
        tiles.push(tile("viewzoom", "More\u{2026}", "", "", Some(BAction::Set(ModalKey::Open.path(), V_MODAL_FILE_BROWSER.to_string())), size));
    }
    tiles
}

/// Learn: the quests, the library, the manual and the links.
fn learn(size: f32) -> Vec<BView> {
    let rows = home::GROUPS.iter().flat_map(|group| group.rows.iter());
    let mut tiles: Vec<BView> = rows.map(|r| tile(r.icon, r.label, "", "", r.goes.map(home::BHomeTarget::action), size)).collect();
    tiles.extend(FOOT_LINKS.iter().map(|l| link_tile(l, size)));
    tiles
}

/// What Phiber is, in one line under the logo.
pub const TAGLINE: &str = "Imagine, Apply, Unleash - The Phi way \u{221e}";

impl BViewItem for BSplashModal {
    /// The logo and its line on the backdrop, the tabs on their one ground,
    /// then the open tab's cards.
    fn build(&self, s: &BState) -> BView {
        let open = BWelcomeTab::of(s);
        let tabs = BWelcomeTab::ALL.into_iter().map(|t| t.view(open)).collect();
        // One size for every tab, so switching tabs never resizes the cards.
        let size = card_size(s);
        let mut cards: Vec<BView> = match open {
            BWelcomeTab::New => grid(new_file(size)),
            BWelcomeTab::Recent => grid(recent(s, size)),
            BWelcomeTab::Learn => grid(learn(size)),
        };
        // Learn ends on the version, one more line under the cards.
        if open == BWelcomeTab::Learn {
            cards.push(centred(vec![BView::label(format!("{APP_NAME} v{}", env!("CARGO_PKG_VERSION")))]));
        }
        // Every row of cards is a line of the window's own column, as tall
        // as its cards (`draw::modal::welcome`): a column of rows inside it
        // shared its height out between them and squashed them together.
        let mut lines = vec![
            centred(vec![BView::icon(crate::view::draw::brand::WORDMARK_3D)]),
            centred(vec![BView::label(TAGLINE.to_string())]),
            BCard::strip(BStrip::Tabs, tabs).into(),
        ];
        lines.extend(cards);
        BView::column(lines)
    }
}

/// Action that dismisses the welcome window without changing anything else.
pub fn dismiss() -> BAction {
    BAction::Set(ModalKey::Open.path(), V_EMPTY.to_string())
}
