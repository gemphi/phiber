//! The channel rows of the animation window — what `ANIM_animdata_filter`
//! lists for the editor, at the height `ANIM_UI_get_channel_step` gives each.
//!
//! Structure: the rows and where they sit. Their backdrops, diamonds and
//! names are `set.rs`.

/// A row's fold: which rows have one, what it is keyed under, and whether
/// it stands open (`BTimeRow::fold`, `is_expanded`, `slot_key`, `group_key`).
mod fold;
pub mod kind;
pub mod name;
mod place;
pub mod set;
/// The Action Editor's slot rows (`BTimeRow::action_editor`).
mod slot;

pub use kind::BRowKind;

use crate::phiphx::scene::{SceneAction, SceneNla};
use crate::host::BHostType;
use crate::scene::BShownKeys;
use crate::state::BState;

/// `ANIM_UI_get_channel_height`: `0.8 × widget_unit`.
pub const ROW_HEIGHT: f32 = 0.8 * super::WIDGET_UNIT;
/// `NLATRACK_HEIGHT` with strip curves shown: `1.2 × widget_unit`.
pub const NLA_ROW_HEIGHT: f32 = 1.2 * super::WIDGET_UNIT;
/// The skip between rows: the side column's gap between its rows (the
/// theme's `gap_xs`), so the keys' rows and the channel list's rows beside
/// them (`part::channels::time`) share one pitch. Blender's own is
/// `ANIM_UI_get_channel_skip`, `0.1 × widget_unit`.
pub const ROW_SKIP: f32 = 0.2 * super::WIDGET_UNIT;
/// `ANIM_UI_get_channel_name_width` / `NLATRACK_NAMEWIDTH`: the channel
/// region, `10 × widget_unit`.
pub const CHANNEL_WIDTH: f32 = 10.0 * super::WIDGET_UNIT;

#[derive(Clone, Debug, PartialEq)]
pub struct BTimeRow {
    pub kind: BRowKind,
    /// Sorted, distinct keyed frames of the channels under this row.
    pub keys: Vec<f32>,
    /// Each column's Keyframe Type (`dope::fold_columns`): as long as
    /// `keys`, or EMPTY for "all plain keyframes" — which is what the rows
    /// built from frames alone say. Read it with [`Self::keytype_at`].
    pub keytypes: Vec<u8>,
}

impl BTimeRow {
    /// The rows the editor lists. The Timeline shows the summary alone
    /// (`ADS_FLAG_SUMMARY_COLLAPSED` in `action_create`); the Dope Sheet
    /// expands it and lists each shown object under it; the Graph editor
    /// lists the objects with no summary (`graph_channel_region_draw`);
    /// the Drivers editor lists each object over its drivers.
    pub fn list(s: &BState, host: BHostType, axis: super::curve::BAxisColors) -> Vec<Self> {
        let shown = BShownKeys::now(s);
        // `BShownKeys` is frames alone; the Dope Sheet's summary takes its
        // types from its objects' rows below.
        let summary = |expanded: bool| Self { kind: BRowKind::Summary { expanded }, keys: shown.columns().to_vec(), keytypes: Vec::new() };
        let mut out = match host {
            BHostType::Action => vec![summary(true)],
            BHostType::Ipo | BHostType::Nla => Vec::new(),
            BHostType::Drivers => return super::driver::BDriverList::now(s).rows(axis),
            _ => return vec![summary(false)],
        };
        crate::scene::source::with_loaded(|loaded| {
            let Some(model) = loaded else { return };
            if host == BHostType::Nla {
                // `animfilter_nla`: each object, its action line, then its
                // tracks last first.
                for name in crate::scene::action::BSceneActions::nla_shown(s, model) {
                    out.extend(model.nla.iter().filter(|nla| nla.object == name).flat_map(|nla| Self::nla(nla, &model.actions)));
                }
                return;
            }
            // `ANIMCONT_ACTION`: the active object's action, a channel per
            // slot (every slot, as the "Only Show Slot of Active Object"
            // filter is off by default), each over its groups.
            if host == BHostType::Action && crate::part::modebar::active_mode(s, host) == crate::part::action::template::ACTION_EDITOR {
                out.extend(Self::action_editor(s, model));
                return;
            }
            for name in crate::scene::action::dope::BKeyTarget::shown(s, model) {
                let expanded = Self::is_expanded(s, &name);
                out.push(Self::object_at(&name, &model.actions, expanded));
                if expanded {
                    out.extend(Self::channels(s, &name, &model.actions));
                }
            }
        });
        // The Dope Sheet's summary is its objects' columns together, folded
        // by the same rule.
        if host == BHostType::Action && out.len() > 1 {
            let keys: Vec<(f32, u8)> = out[1..].iter().flat_map(|r| r.keys.iter().enumerate().map(|(i, &f)| (f, r.keytype_at(i)))).collect();
            let (frames, keytypes) = crate::scene::action::dope::fold_columns(keys);
            out[0].keys = frames;
            out[0].keytypes = keytypes;
        }
        out
    }

    /// One object's NLA rows: the object, the action line with the action's
    /// key columns, then the tracks as the editor stacks them (last first).
    pub fn nla(nla: &SceneNla, actions: &[SceneAction]) -> Vec<Self> {
        let keys = nla
            .action
            .as_deref()
            .and_then(|name| actions.iter().find(|action| action.name == name))
            .map_or(Vec::new(), |action| action.keys.clone());
        let mut out = vec![
            Self { kind: BRowKind::Object { name: nla.object.clone(), expanded: false }, keys: Vec::new(), keytypes: Vec::new() },
            // `SceneAction::keys` is frames alone: plain keyframes.
            Self { kind: BRowKind::Action { name: nla.action.clone(), tweaking: nla.flag & crate::phiphx::blend::ADT_EDIT_ON != 0 }, keys, keytypes: Vec::new() },
        ];
        out.extend(nla.tracks.iter().rev().map(|track| Self { kind: BRowKind::Track(track.clone()), keys: Vec::new(), keytypes: Vec::new() }));
        out
    }

    /// One object's row: the columns of the slot it is animated through in
    /// every action it uses (`dope::BKeyTarget`).
    pub fn object(name: &str, actions: &[SceneAction]) -> Self {
        Self::object_at(name, actions, false)
    }

    /// The same row, told whether it is unfolded.
    pub fn object_at(name: &str, actions: &[SceneAction], expanded: bool) -> Self {
        let (keys, keytypes) = crate::scene::action::dope::BKeyTarget::columns_typed(actions, name);
        Self { kind: BRowKind::Object { name: name.to_string(), expanded }, keys, keytypes }
    }

    /// Column `i`'s Keyframe Type; a plain keyframe when the row carries
    /// frames alone (see `keytypes`).
    pub fn keytype_at(&self, i: usize) -> u8 {
        self.keytypes.get(i).copied().unwrap_or(crate::phiphx::blend::KEYTYPE_KEYFRAME)
    }

    /// The F-curve rows of every action `name` uses, in the file's own order,
    /// ungrouped.
    pub fn curves(name: &str, actions: &[SceneAction]) -> Vec<Self> {
        Self::object_curves(name, actions).map(|curve| Self::curve(curve, false)).collect()
    }

    /// What an expanded object lists (`animfilter_act_group` under
    /// `animfilter_action`): each channel group in the order its first curve
    /// comes — closed, its curves' keys summarised on it; open, those curves
    /// under it — then the curves in no group.
    pub fn channels(s: &BState, name: &str, actions: &[SceneAction]) -> Vec<Self> {
        Self::grouped(s, name, Self::object_curves(name, actions).collect())
    }

    /// `curves` as group rows (folds keyed under `owner`) and the
    /// ungrouped curves after them.
    fn grouped(s: &BState, name: &str, curves: Vec<&crate::phiphx::blend::BFCurve>) -> Vec<Self> {
        let mut groups: Vec<&str> = Vec::new();
        for group in curves.iter().filter_map(|c| c.group.as_deref()) {
            if !groups.contains(&group) {
                groups.push(group);
            }
        }
        let mut out = Vec::new();
        for group in groups {
            let members: Vec<&&crate::phiphx::blend::BFCurve> = curves.iter().filter(|c| c.group.as_deref() == Some(group)).collect();
            let (keys, keytypes) = crate::scene::action::dope::fold_columns(
                members.iter().flat_map(|c| c.keys.iter().map(|k| (k.co[0], k.keytype))).collect::<Vec<_>>(),
            );
            let expanded = Self::is_expanded(s, &Self::group_key(name, group));
            out.push(Self { kind: BRowKind::Group { object: name.to_string(), name: group.to_string(), expanded }, keys, keytypes });
            if expanded {
                out.extend(members.into_iter().map(|c| Self::curve(c, true)));
            }
        }
        out.extend(curves.iter().filter(|c| c.group.is_none()).map(|c| Self::curve(c, false)));
        out
    }

    /// The curves `name` is animated by: in each action it uses, the slot
    /// it is bound to — else, unbound, every curve (as `BKeyTarget::of`
    /// reads its columns), so a curve row and its object's columns agree.
    fn object_curves<'a>(name: &'a str, actions: &'a [SceneAction]) -> impl Iterator<Item = &'a crate::phiphx::blend::BFCurve> + 'a {
        actions.iter().filter(move |action| action.used_by(name)).flat_map(move |action| match action.slot_of(name) {
            Some(slot) => action.curves.get(slot.curves.clone()).unwrap_or(&[]).iter(),
            None => action.curves.iter(),
        })
    }

    fn curve(curve: &crate::phiphx::blend::BFCurve, grouped: bool) -> Self {
        Self {
            kind: BRowKind::Curve { name: name::fcurve_name(curve), color: curve.color, grouped },
            // `BezTriple.co` is (frame, value); the frame is the x.
            keys: curve.keys.iter().map(|key| key.co[0]).collect(),
            // One curve, so no folding: each key keeps its own type.
            keytypes: curve.keys.iter().map(|key| key.keytype).collect(),
        }
    }

    /// The channel's name as the channel region prints it
    /// (`acf_nlaaction_name`: `<No Action>` for an empty action line).
    pub fn name(&self) -> &str {
        match &self.kind {
            BRowKind::Summary { .. } => "Summary",
            BRowKind::Object { name, .. } => name,
            // `acf_action_slot_name`: the slot's name past its ID code; a
            // legacy action's lone slot has none.
            BRowKind::Slot { identifier, .. } => identifier.get(2..).filter(|n| !n.is_empty()).unwrap_or("Legacy Slot"),
            BRowKind::Curve { name, .. } | BRowKind::Group { name, .. } => name,
            BRowKind::Action { name, .. } => name.as_deref().unwrap_or("<No Action>"),
            BRowKind::Track(track) => &track.name,
            BRowKind::Driver { name, .. } => name,
        }
    }

}

#[cfg(test)]
mod tests;
