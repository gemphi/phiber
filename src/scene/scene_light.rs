//! Material Preview's and Rendered's Lighting (`VIEW3D_PT_shading_lighting`
//! for MATERIAL and RENDERED): whether the file's lamps and the file's world
//! light the viewport, or Blender's studio HDRI does.
//!
//! Blender keeps one pair per shading (`View3DShading` in `rna_space.cc`):
//! `use_scene_lights` / `use_scene_world` for Material Preview, both off,
//! and `use_scene_lights_render` / `use_scene_world_render` for Rendered,
//! both on. Rendered with Scene World off is how a file whose world image
//! is missing — a magenta dome — is looked at under the studio light
//! instead. The viewport only: F12 renders the scene's own world and lamps
//! whatever these say, as Blender's does.

use crate::scene::BShading;
use crate::state::val::V_TRUE;
use crate::state::{BState, View3dKey};

/// Whether the scene's lamps and world light this shading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BSceneLight {
    pub lights: bool,
    pub world: bool,
}

impl BSceneLight {
    /// The two switches a shading has, `(rna, default)` — lamps, then world.
    /// Solid and Wireframe have their own studio light and no such pair.
    pub fn keys(shading: BShading) -> Option<[(&'static str, bool); 2]> {
        match shading {
            BShading::Material => Some([("use_scene_lights", false), ("use_scene_world", false)]),
            BShading::Rendered => Some([("use_scene_lights_render", true), ("use_scene_world_render", true)]),
            BShading::Solid | BShading::Wireframe => None,
        }
    }

    /// The switches as they stand for `shading`.
    pub fn now(s: &BState, shading: BShading) -> Self {
        let on = |(rna, default): (&'static str, bool)| s.get(View3dKey::Shade(rna)).map_or(default, |v| v == V_TRUE);
        match Self::keys(shading) {
            Some([lights, world]) => Self { lights: on(lights), world: on(world) },
            None => Self { lights: false, world: false },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_shading_starts_where_blender_does() {
        let s = BState::new();
        assert_eq!(BSceneLight::now(&s, BShading::Material), BSceneLight { lights: false, world: false }, "Material Preview: the studio light");
        assert_eq!(BSceneLight::now(&s, BShading::Rendered), BSceneLight { lights: true, world: true }, "Rendered: the scene's own");
        assert_eq!(BSceneLight::keys(BShading::Solid), None, "Solid has its own studio light");
    }

    #[test]
    fn rendered_without_the_scene_world_takes_the_studio_light() {
        let mut s = BState::new();
        s.set(View3dKey::Shade("use_scene_world_render"), "false");
        assert!(!BSceneLight::now(&s, BShading::Rendered).world, "a missing world image no longer paints the dome magenta");
        assert!(BSceneLight::now(&s, BShading::Rendered).lights, "the lamps still light it");
        s.set(View3dKey::Shade("use_scene_lights"), V_TRUE);
        assert!(BSceneLight::now(&s, BShading::Material).lights, "Material Preview with the scene's lamps");
    }
}
