//! An item: the one row every list, menu, tree, table and property panel is
//! made of. A row has a lead (a glyph, a checkbox or radio mark, a colour
//! swatch), a label, a value (a shortcut, a read-out), a trail (the arrow of
//! a row that opens a menu of its own), its state and what it does. Which of
//! them it has is data; how it is framed (a popup's row, a source list's, a
//! tree's, a table's, a property's) is where it stands, decided by the
//! renderer (`view/draw/item`), never by the row.

use super::{BAction, BControl, BMark};
use crate::theme::BColor;

/// What leads the row, in the glyph slot every row of its list reserves.
#[derive(Clone, Debug, PartialEq)]
pub enum BLead {
    /// Nothing, but the slot is kept so the labels share one left edge.
    None,
    /// A glyph on the row's tile.
    Glyph(String),
    /// A checkbox or radio mark (`draw::mark`).
    Mark(BMark),
    /// A colour, as a swatch.
    Swatch(BColor),
}

/// What the row says after its label.
#[derive(Clone, Debug, PartialEq)]
pub enum BItemValue {
    None,
    /// A key binding, quieter than the label, at the row's end.
    Shortcut(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct BItem {
    /// The row's id, for clicks and for tests: `menu-<label>` for a menu's,
    /// the button's own for a list's.
    pub id: String,
    pub lead: BLead,
    pub label: String,
    pub value: BItemValue,
    /// It opens a menu of its own: an arrow at its end.
    pub submenu: bool,
    pub active: bool,
    pub muted: bool,
    pub action: Option<BAction>,
    /// Fired when the pointer enters (a submenu opens on hover).
    pub hover: Option<BAction>,
}

impl BItem {
    /// A control as a row: its glyph, mark or swatch, its label, its
    /// shortcut, and its id.
    pub fn of(ctl: &BControl, id: String) -> Self {
        let lead = match (ctl.mark, ctl.swatch, ctl.icon.as_deref().unwrap_or("")) {
            (Some(mark), _, _) => BLead::Mark(mark),
            (None, Some(color), _) => BLead::Swatch(color),
            (None, None, "") => BLead::None,
            (None, None, icon) => BLead::Glyph(icon.to_string()),
        };
        let value = match ctl.shortcut.as_deref().unwrap_or("") {
            "" => BItemValue::None,
            shortcut => BItemValue::Shortcut(shortcut.to_string()),
        };
        Self {
            id,
            lead,
            label: ctl.label.clone().unwrap_or_default(),
            value,
            submenu: ctl.submenu,
            active: ctl.active,
            muted: ctl.muted,
            action: ctl.action.clone(),
            hover: ctl.hover.clone(),
        }
    }
}
