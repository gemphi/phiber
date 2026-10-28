//! The animation editors' channel list as a side column: the Dope Sheet's,
//! the Graph Editor's, the NLA's and the Drivers editor's channels, built
//! as every side column is (`BCard::side` in a `BColumn::Sidebar`), each
//! channel a tree row as the Outliner's are — its fold, its icon, its name.
//!
//! The canvas beside it paints only the keys (`canvas::time`), on the same
//! rows: its row is the theme's row height and its skip the column's gap,
//! and its scrub band is a heading tall, so row `n` here and row `n` there
//! share one line (`tests/the_channel_list_lines_up_with_its_keys.rs`).

use crate::host::BHostType;
use crate::state::val::{V_FALSE, V_TRUE};
use crate::state::{BKey as _, BState, DopeKey, OpKey};
use crate::view::draw::canvas::time::curve::BAxisColors;
use crate::view::draw::canvas::time::row::{BRowKind, BTimeRow};
use crate::view::draw::canvas::time::view::BTimeView;
use crate::view::prim::{BAction, BCard, BColumn, BControl, BRowShade};
use crate::view::BView;

/// The axis colours only colour a driver's swatch, which this list does not
/// draw; any will do.
const AXIS: BAxisColors = BAxisColors { x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], z: [0.0, 0.0, 1.0], w: [1.0, 1.0, 0.0], other: [0.0, 1.0, 1.0] };

/// The heading over each editor's channels: its word and its icon.
fn heading(host: BHostType) -> Option<(&'static str, &'static str)> {
    match host {
        BHostType::Action => Some(("Channels", "keyframe")),
        BHostType::Ipo => Some(("Curves", "graph")),
        BHostType::Nla => Some(("Tracks", "nla")),
        BHostType::Drivers => Some(("Drivers", "driver")),
        _ => None,
    }
}

/// The channel list for `host`, or `None` for a host without one. Empty
/// while there is nothing to list in an editor that lists only what
/// animates (the canvas's own rule, `BTimeView::channel_width`).
pub fn list(s: &BState, host: BHostType) -> Option<BView> {
    let (title, icon) = heading(host)?;
    let view = BTimeView::now(s, host, AXIS);
    if view.channel_width <= 0.0 {
        return Some(BView::empty());
    }
    let rows = view.rows.iter().enumerate().map(|(i, row)| self::row(&view.rows, i, row)).collect();
    Some(BView::from(BCard::column(BColumn::Sidebar, vec![BCard::side(title, icon).body(rows).into()])))
}

/// One channel as a tree row: how deep it sits, its fold, its icon, its name.
fn row(rows: &[BTimeRow], index: usize, row: &BTimeRow) -> BView {
    let (depth, icon) = match &row.kind {
        BRowKind::Summary { .. } => (0, "keyframe"),
        BRowKind::Object { .. } => (0, "object"),
        BRowKind::Slot { .. } => (0, "action"),
        BRowKind::Action { .. } => (1, "action"),
        BRowKind::Track(_) => (1, "track"),
        BRowKind::Group { .. } => (1, "group"),
        BRowKind::Curve { grouped: true, .. } => (2, "curve"),
        BRowKind::Curve { grouped: false, .. } => (1, "curve"),
        BRowKind::Driver { .. } => (1, "driver"),
    };
    let fold = match &row.kind {
        BRowKind::Object { name, expanded } => Some((name.clone(), *expanded)),
        BRowKind::Slot { action, identifier, expanded, .. } => Some((BTimeRow::slot_key(action, identifier), *expanded)),
        BRowKind::Group { object, name, expanded } => Some((BTimeRow::group_key(object, name), *expanded)),
        _ => None,
    };
    let toggle = fold.as_ref().map(|(key, open)| {
        BAction::Set(DopeKey::Expanded(crate::state::intern(key)).path(), (if *open { V_FALSE } else { V_TRUE }).to_string())
    });
    // An NLA track's name picks the track (`BNlaOp::channel`), as a press
    // on it in the channel region did.
    let name = match BTimeRow::track_at(rows, index) {
        Some((object, track)) => BView::Button(
            BControl::new()
                .with_label(row.name().to_string())
                .with_action(BAction::Set(OpKey::Nla.path(), crate::scene::action::nla_editor::BNlaOp::channel(&object, track, false))),
        ),
        None => BView::label(row.name()),
    };
    let shade = match &row.kind {
        BRowKind::Track(track) if track.flag & crate::phiphx::blend::TRACK_SELECTED != 0 => BRowShade::Selected,
        _ => BRowShade::Flat,
    };
    // A curve's or a driver's colour, as a swatch before its icon
    // (`ANIM_channel_draw`'s colour square).
    let swatch = match &row.kind {
        BRowKind::Curve { color, .. } | BRowKind::Driver { color, .. } => {
            let (h, s, l) = crate::theme::palette::shade::hsl_of(*color);
            vec![BView::Button(BControl::new().with_swatch((h, s, l, 1.0)))]
        }
        _ => Vec::new(),
    };
    BView::tree_row(depth, fold.map(|(_, open)| open), toggle, icon, vec![name], Vec::new(), shade).with_lead(swatch)
}
