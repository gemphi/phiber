//! Simulation workspace — its areas, and which hosts each one offers.
//!
//! The viewport in the centre, the Timeline under it to run and scrub the
//! simulation, the Outliner and Properties on the right; a new file started
//! here opens Properties on its Physics tab (`BSpaceType::properties_tab`).
//! Blender ships no such workspace; this is Animation's frame with the
//! Timeline in the Dope Sheet's place.

use crate::area::{BAreaSpec, BAreaType};
use crate::host::BHostType;
use crate::zone::BZoneType;

pub const LABEL: &str = "Simulation";
pub const ICON: &str = "physics";

pub static AREAS: &[BAreaSpec] = &[
    BAreaSpec {
        kind: BAreaType::Top,
        zone: BZoneType::Top,
        default_host: BHostType::Topbar,
        hosts: &[BHostType::Topbar],
    },
    // The activity rail: the same fixed strip down the far left of every space.
    BAreaSpec {
        kind: BAreaType::Left,
        zone: BZoneType::Rail,
        default_host: BHostType::Activity,
        hosts: &[BHostType::Activity],
    },
    BAreaSpec {
        kind: BAreaType::Center,
        zone: BZoneType::Center,
        default_host: BHostType::View3D,
        hosts: BHostType::SWITCHABLE,
    },
    BAreaSpec {
        kind: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Outliner,
        hosts: &[BHostType::Outliner],
    },
    BAreaSpec {
        kind: BAreaType::Right,
        zone: BZoneType::Right,
        default_host: BHostType::Buts,
        hosts: &[BHostType::Buts],
    },
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Time,
        hosts: &[
            BHostType::Time,
            BHostType::Play,
            BHostType::Project,
            BHostType::Action,
            BHostType::Ipo,
            BHostType::Drivers,
            BHostType::Nla,
        ],
    },
    BAreaSpec {
        kind: BAreaType::Bottom,
        zone: BZoneType::Bottom,
        default_host: BHostType::Statusbar,
        hosts: &[BHostType::Statusbar],
    },
];
