//! The Dope Sheet's key operators on `ctx.op.keys` (see the module above):
//! `all`, `none`, `invert`, `click:<row>:<frame>` (`click+` with Shift,
//! `click:none`), `box:<f0>,<f1>,<r0>,<r1>` and `box:end`, `delete`, and
//! the move's `grab:start[:<frame>]`, `grab:at:<frame>`, `grab:confirm`,
//! `grab:cancel`. Row 0 is the summary, then the listed objects.
//!
//! The dispatcher lives here; the two halves of the match that got long split
//! out alongside — `set_edit` for the keyframe edits (clean, duplicate,
//! snap, mirror, reshape, grab) and `set_jump` for the playhead one — so
//! each file sits under the 240-line read cap (`tests/level_structure.rs`).

use super::super::BSceneActions;
use super::{BKeySelection, BKeyTarget};
use crate::state::{BKey as _, BPath, BState, CtxKey, OpKey, log, rule::BRule};

/// `ctx.op.keys` (see the module).
pub struct BKeyOpRule;

impl BRule for BKeyOpRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::Keys.path()
    }

    fn after(&self, _: &BPath, op: &str, s: &mut BState) {
        let Some(model) = crate::scene::source::loaded_shared() else { return };
        let shown = BKeyTarget::shown(s, &model);
        let mut picks = BKeySelection::now(s);
        let every = |f: &dyn Fn(&mut BKeySelection, &str, f32), picks: &mut BKeySelection| {
            for name in &shown {
                BKeyTarget::columns(&model.actions, name).into_iter().for_each(|c| f(picks, name, c));
            }
        };
        let mut part = op.split(':');
        let head = (part.next().unwrap_or(""), part.next(), part.next());
        match head {
            ("all", ..) => every(&|p, n, c| p.add(n, c), &mut picks),
            ("none", ..) => picks = BKeySelection::default(),
            ("invert", ..) => every(&|p, n, c| p.flip(n, c), &mut picks),
            (click @ ("click" | "click+"), Some(row), Some(frame)) => {
                let (Ok(row), Ok(frame)) = (row.parse::<usize>(), frame.parse::<f32>()) else { return };
                let rows: Vec<&String> = if row == 0 { shown.iter().collect() } else { shown.get(row - 1).into_iter().collect() };
                let rows: Vec<&String> = rows.into_iter().filter(|n| BKeyTarget::columns(&model.actions, n).iter().any(|c| (c - frame).abs() < 5e-4)).collect();
                let on = rows.iter().any(|n| picks.has(n, frame));
                if click == "click" {
                    picks = BKeySelection::default();
                }
                for name in rows {
                    match click == "click+" && on {
                        true => picks.0.retain(|p| !(p.object == *name && (p.frame - frame).abs() < 5e-4)),
                        false => picks.add(name, frame),
                    }
                }
            }
            ("click", Some("none"), _) => picks = BKeySelection::default(),
            ("box", Some("end"), _) => return s.remove(CtxKey::KeyBox),
            ("box", Some(range), _) => {
                let v: Vec<f32> = range.split(',').filter_map(|v| v.parse().ok()).collect();
                let [f0, f1, r0, r1] = v[..] else { return };
                let base = match s.get(CtxKey::KeyBox).and_then(|b| b.split_once('#')) {
                    Some((_, base)) => BKeySelection::parse(base),
                    None => picks.clone(),
                };
                s.set(CtxKey::KeyBox, format!("{range}#{}", base.encode()));
                picks = base;
                let (rows, frames) = ((r0.min(r1) as usize)..=(r0.max(r1) as usize), f0.min(f1)..=f0.max(f1));
                for (i, name) in shown.iter().enumerate() {
                    if rows.contains(&0) || rows.contains(&(i + 1)) {
                        BKeyTarget::columns(&model.actions, name).into_iter().filter(|c| frames.contains(c)).for_each(|c| picks.add(name, c));
                    }
                }
            }
            ("delete", ..) => {
                let mut edits = BSceneActions::now(s);
                edits.keyed.extend(BKeyTarget::steps(&model.actions, &picks, None));
                edits.write(s);
                log::push(format!("Deleted {} keyframe column(s)", picks.0.len()));
                picks = BKeySelection::default();
            }
            // Jump moves the PLAYHEAD, not the keys — it writes `CtxKey::Frame`
            // and returns, so the picks never travel through `picks.write`.
            ("jump", ..) => return super::set_jump::jump(s, &picks),
            // Select ▸ Columns on Selected Keys / Column on Current Frame /
            // Columns on Selected Markers / Between Selected Markers
            // (`ACTION_OT_select_column`, `action_select.cc`). Every mode
            // ADDS: Blender's column select never clears what is picked.
            // The first three name a list of columns; `between` is a range
            // over the selected markers' extremes, widened half a frame
            // each way (`markers_selectkeys_between`).
            ("column", Some(mode), _) => {
                let marks = || crate::scene::marker::BMarkerList::selected(s).into_iter().map(|f| f as f32);
                let span = || {
                    let (lo, hi) = marks().fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), f| (lo.min(f), hi.max(f)));
                    lo.is_finite().then_some((lo - 0.5, hi + 0.5))
                };
                let range = match mode {
                    "between" => match span() {
                        Some(range) => Some(range),
                        None => return log::push("No selected markers to select between".to_string()),
                    },
                    _ => None,
                };
                let columns: Vec<f32> = match mode {
                    "keys" => picks.0.iter().map(|p| p.frame).collect(),
                    "cfra" => vec![super::set_jump::frame_now(s)],
                    "markers" => marks().collect(),
                    "between" => Vec::new(),
                    _ => return,
                };
                for name in &shown {
                    for c in BKeyTarget::columns(&model.actions, name) {
                        let hit = match range {
                            Some((lo, hi)) => c >= lo && c <= hi,
                            None => columns.iter().any(|f| (f - c).abs() < 5e-4),
                        };
                        if hit {
                            picks.add(name, c);
                        }
                    }
                }
            }
            // Select ▸ Before / After Current Frame
            // (`ACTION_OT_select_leftright`): this one REPLACES, and the
            // keys standing ON the playhead belong to both sides — Blender
            // runs the range to `cfra ± 0.1`.
            ("leftright", Some(side), _) => {
                let now = super::set_jump::frame_now(s);
                let (lo, hi) = match side {
                    "left" => (f32::NEG_INFINITY, now + 0.1),
                    "right" => (now - 0.1, f32::INFINITY),
                    _ => return,
                };
                picks = BKeySelection::default();
                for name in &shown {
                    BKeyTarget::columns(&model.actions, name).into_iter().filter(|c| *c >= lo && *c <= hi).for_each(|c| picks.add(name, c));
                }
            }
            // Copy / Paste go through the runtime buffer (`set_copy`), which
            // owns whether anything is written: a copy changes no keys and
            // leaves the selection alone, a paste selects what it landed.
            (op @ ("copy" | "paste"), ..) => {
                match super::set_copy::copy_paste(op, s, &model, picks) {
                    Some(next) => picks = next,
                    None => return,
                }
            }
            // Every remaining arm (clean / duplicate / snap / mirror / ipo /
            // easing / handle / grab:*) edits the keys. The helper owns its
            // own `picks` and writes them when it is done.
            _ => return super::set_edit::edit(head, s, &model, &shown, picks),
        }
        picks.write(s);
    }
}
