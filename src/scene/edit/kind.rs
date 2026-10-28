//! What Add makes besides a mesh — `object.light_add`, `object.camera_add`
//! and `object.empty_add` — and what each added object is: the slug the
//! menu fires (`ctx.op.scene_edit` = `add:<slug>`), the name the new
//! object takes (`BLI_uniquename` over it, as for a mesh), and the lamp,
//! camera or empty it stands in the scene as (`model.rs`).
//!
//! Blender ref: `VIEW3D_MT_light_add`, `object_add.cc` (a sun's strength
//! is set to 1 there, every other lamp keeps the default power), and the
//! DNA defaults a new light and camera start from (`DNA_light_defaults.h`,
//! `DNA_camera_defaults.h`).

use crate::phiphx::scene::{IDENTITY_M4, SceneCamera, SceneEmpty, SceneLight};

/// What an added object is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BAddKind {
    /// A primitive from `SHAPES`.
    Mesh,
    /// A lamp of a type: `"POINT"`, `"SUN"`, `"SPOT"` or `"AREA"`.
    Light(&'static str),
    Camera,
    /// An empty drawn as `"PLAIN_AXES"`, `"ARROWS"`, `"CUBE"`, …
    Empty(&'static str),
}

/// The objects Add offers that are not meshes: `(slug, name, kind)`.
/// Blender names a new lamp after its type, a camera and an empty plainly.
pub const EXTRAS: &[(&str, &str, BAddKind)] = &[
    ("light_point", "Point", BAddKind::Light("POINT")),
    ("light_sun", "Sun", BAddKind::Light("SUN")),
    ("light_spot", "Spot", BAddKind::Light("SPOT")),
    ("light_area", "Area", BAddKind::Light("AREA")),
    ("camera", "Camera", BAddKind::Camera),
    ("empty_plain_axes", "Empty", BAddKind::Empty("PLAIN_AXES")),
    ("empty_arrows", "Empty", BAddKind::Empty("ARROWS")),
    ("empty_single_arrow", "Empty", BAddKind::Empty("SINGLE_ARROW")),
    ("empty_circle", "Empty", BAddKind::Empty("CIRCLE")),
    ("empty_cube", "Empty", BAddKind::Empty("CUBE")),
    ("empty_sphere", "Empty", BAddKind::Empty("SPHERE")),
    ("empty_cone", "Empty", BAddKind::Empty("CONE")),
];

impl BAddKind {
    /// The kind of a slug Add fires, and the name its object takes.
    pub fn of(slug: &str) -> Option<(Self, &'static str)> {
        match super::SHAPES.iter().find(|(s, ..)| *s == slug) {
            Some(&(_, label, _)) => Some((Self::Mesh, label)),
            None => EXTRAS.iter().find(|(s, ..)| *s == slug).map(|&(_, label, kind)| (kind, label)),
        }
    }

    /// Blender's object type, as the Outliner lists it.
    pub fn object_type(self) -> &'static str {
        match self {
            Self::Mesh => "MESH",
            Self::Light(_) => "LIGHT",
            Self::Camera => "CAMERA",
            Self::Empty(_) => "EMPTY",
        }
    }

    /// `Object.type`: `OB_MESH`, `OB_LAMP`, `OB_CAMERA`, `OB_EMPTY`.
    pub fn ob_type(self) -> i16 {
        match self {
            Self::Mesh => 1,
            Self::Light(_) => 10,
            Self::Camera => 11,
            Self::Empty(_) => 0,
        }
    }
}

/// A lamp as Add ▸ Light makes it, at the origin looking down −Z: white,
/// 1000 W (a sun's strength 1), a 45° spot with a 0.15 blend, a 0.1 m
/// radius, a 0.25 m square area, a sun 0.526° across.
pub fn light(name: &str, kind: &'static str) -> SceneLight {
    let (energy, size) = match kind {
        "SUN" => (1.0, 0.526_f32.to_radians()),
        "AREA" => (1000.0, 0.25),
        _ => (1000.0, 0.1),
    };
    SceneLight {
        name: name.to_string(),
        location: [0.0; 3],
        matrix_world: IDENTITY_M4,
        color: [1.0; 3],
        energy,
        kind: kind.to_string(),
        spot_size: std::f32::consts::FRAC_PI_4,
        spot_blend: 0.15,
        size,
        size_y: size,
        shape: "SQUARE".to_string(),
    }
}

/// A camera as Add ▸ Camera makes it, at the origin looking down −Z: a
/// 50 mm lens on a 36 mm sensor, clipping 0.1 – 1000 m.
pub fn camera(name: &str) -> SceneCamera {
    SceneCamera {
        name: name.to_string(),
        location: [0.0; 3],
        matrix_world: IDENTITY_M4,
        kind: "PERSP".to_string(),
        focal_length_mm: 50.0,
        sensor_width: 36.0,
        sensor_height: 24.0,
        sensor_fit: "AUTO".to_string(),
        shift: [0.0, 0.0],
        ortho_scale: 6.0,
        clip_start: 0.1,
        clip_end: 1000.0,
        display_size: 1.0,
        dof: Default::default(),
    }
}

/// An empty as Add ▸ Empty makes it: 1 m across, drawn as asked.
pub fn empty(name: &str, display_type: &'static str) -> SceneEmpty {
    SceneEmpty { name: name.to_string(), matrix_world: IDENTITY_M4, display_type: display_type.to_string(), display_size: 1.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_slug_names_its_object_the_way_blender_does() {
        assert_eq!(BAddKind::of("cube"), Some((BAddKind::Mesh, "Cube")));
        assert_eq!(BAddKind::of("light_sun"), Some((BAddKind::Light("SUN"), "Sun")));
        assert_eq!(BAddKind::of("camera"), Some((BAddKind::Camera, "Camera")));
        assert_eq!(BAddKind::of("empty_cube"), Some((BAddKind::Empty("CUBE"), "Empty")));
        assert_eq!(BAddKind::of("teapot"), None);
        assert_eq!(BAddKind::Light("SPOT").object_type(), "LIGHT");
    }

    #[test]
    fn a_sun_is_lit_by_strength_and_the_other_lamps_by_watts() {
        assert_eq!(light("Sun", "SUN").energy, 1.0);
        assert_eq!(light("Point", "POINT").energy, 1000.0);
        assert!(light("Sun", "SUN").size < 0.01, "a sun's size is its angle");
    }
}
