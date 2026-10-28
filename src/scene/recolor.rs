//! The Material tab's Surface edits — Base Color, Metallic and Roughness
//! (`Material.diffuse_color`, `metallic`, `roughness`) typed in for the
//! active object's first material: the viewport and the render take them
//! at once. One key over the file (`ctx.scene.recolored`,
//! `name=r,g,b,a,metallic,roughness[,image]|…`), stepped by Undo like the
//! other scene edits.
//!
//! The panel showed the file's first material and nothing read an edit, so
//! a cube could not be turned red. An object with no material — every
//! primitive Add ▸ Mesh makes — is given Blender's default grey to carry the
//! edit (`SceneMaterial::neutral`). The edit is a flat colour: an image or a
//! procedural under Base Color gives way to it, as plugging a colour into a
//! linked socket does in Blender.
//!
//! Or an image: Image Texture ▸ Open on the Base Color — the folder beside
//! the panel's Image field opens the file browser
//! (`BFilePurpose::BaseColorImage`), or a path is typed into the field. The
//! image is decoded by the loader the file's own textures go through
//! (`phiphx`'s `TextureCache`), once per path. Metallic and Roughness keep
//! it; a colour typed in afterwards unplugs it. A save does not write
//! either yet.

use super::BObject3D;
use crate::part::file::purpose::{BFilePurpose, purpose};
use crate::phiphx::scene::{SceneMaterial, SceneTexture, TextureCache};
use crate::state::{BKey as _, BPath, BState, ButsKey, CtxKey, FileKey, SceneKey, rule::BRule, val::{BValue, BValueKind}};

/// The panel's Surface fields, and the keys they are drawn from.
pub const COLOR_FIELD: &str = "diffuse_color";
pub const METALLIC_FIELD: &str = "metallic";
pub const ROUGHNESS_FIELD: &str = "roughness";
/// The Base Color's image: its path (`Image.filepath`).
pub const IMAGE_FIELD: &str = "base_color_image";

/// What one object's first material was set to.
#[derive(Clone, Debug, PartialEq)]
pub struct BRecolor {
    /// Base Color, scene-linear RGBA as the panel's colour field holds it.
    pub color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    /// The image plugged into Base Color, by path; `None` is the flat colour.
    pub image: Option<String>,
}

impl BRecolor {
    fn encode(&self) -> String {
        let values = self.color.iter().chain([&self.metallic, &self.roughness]).map(|v| format!("{v}"));
        // The path last, so a comma in a file name stays in it.
        values.chain(self.image.clone()).collect::<Vec<_>>().join(",")
    }

    fn parse(raw: &str) -> Option<Self> {
        let parts: Vec<&str> = raw.splitn(7, ',').collect();
        let v: Vec<f32> = parts.iter().take(6).filter_map(|p| p.parse().ok()).collect();
        let image = parts.get(6).filter(|p| !p.is_empty()).map(|p| p.to_string());
        (v.len() == 6).then(|| Self { color: [v[0], v[1], v[2], v[3]], metallic: v[4], roughness: v[5], image })
    }

    /// `material` with this edit over it: the colour in every place a mode
    /// reads one (Base Color, the Principled's base, Viewport Display), the
    /// two scalars likewise, and the base image set aside — or this edit's
    /// own put in its place.
    pub fn over(&self, material: &mut SceneMaterial) {
        material.base_color = self.color;
        material.viewport_color = self.color;
        material.bsdf.base = self.color;
        material.metallic = self.metallic;
        material.roughness = self.roughness;
        material.bsdf.metallic = self.metallic;
        material.bsdf.roughness = self.roughness;
        material.viewport_pbr = [self.roughness, self.metallic];
        material.procedural.base = None;
        // A linked image is the whole Base Color, as a bake's result is
        // (`baked`: nothing tints it), sampled through the mesh's own UVs
        // with no Mapping of the file's; where there are no UVs the colour
        // shows. Solid's Material colour stays the colour.
        let texture = self.image.as_deref().and_then(image);
        if texture.is_some() {
            material.uv_affine = None;
        }
        material.baked = texture.is_some();
        material.base_color_texture = texture.as_ref().and(self.image.clone());
        material.texture = texture;
    }

    /// A material showing this edit, for the panel to seed from.
    pub fn material(&self) -> SceneMaterial {
        let mut material = SceneMaterial::neutral();
        self.over(&mut material);
        material
    }
}

/// `path` decoded as the file's own images are (`TextureCache`: read,
/// decoded, reduced to `MAX_TEXTURE`), once per path for the session — the
/// scene is built again on every edit and every frame of a playing scene,
/// and the image is read once.
pub fn image(path: &str) -> Option<SceneTexture> {
    static CACHE: std::sync::Mutex<Option<TextureCache>> = std::sync::Mutex::new(None);
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    cache.get_or_insert_with(TextureCache::default).get(path)
}

/// Every object the panel recoloured, by its name now.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BRecolors {
    pub list: Vec<(String, BRecolor)>,
}

impl BRecolors {
    pub fn now(s: &BState) -> Self {
        Self::parse(s.get(SceneKey::Recolored).unwrap_or(""))
    }

    /// The list as `ctx.scene.recolored` spells it.
    pub fn parse(raw: &str) -> Self {
        let list = raw.split('|').filter_map(|e| e.split_once('=').and_then(|(n, t)| BRecolor::parse(t).map(|t| (n.to_string(), t)))).collect();
        Self { list }
    }

    pub fn encode(&self) -> String {
        self.list.iter().map(|(n, t)| format!("{n}={}", t.encode())).collect::<Vec<_>>().join("|")
    }

    pub fn write(&self, s: &mut BState) {
        s.set(SceneKey::Recolored, self.encode());
    }

    pub fn get(&self, name: &str) -> Option<BRecolor> {
        self.list.iter().find(|(n, _)| n == name).map(|(_, t)| t.clone())
    }

    pub fn set(&mut self, name: &str, t: BRecolor) {
        self.list.retain(|(n, _)| n != name);
        self.list.push((name.to_string(), t));
    }

    /// The panel's three fields as the state holds them, all or none.
    pub fn from_panel(s: &BState) -> Option<BRecolor> {
        let float = |field: &'static str| match s.value(&ButsKey::Value(field).path(), BValueKind::Float) {
            Some(BValue::Float(v)) => Some(v),
            _ => None,
        };
        let color = match s.value(&ButsKey::Value(COLOR_FIELD).path(), BValueKind::Color) {
            Some(BValue::Color(c)) => c,
            _ => return None,
        };
        Some(BRecolor { color, metallic: float(METALLIC_FIELD)?, roughness: float(ROUGHNESS_FIELD)?, image: None })
    }

    /// The recoloured objects' first materials take their edits; one with
    /// no material is given one.
    pub fn apply(&self, objects: &mut [BObject3D]) {
        if self.list.is_empty() {
            return;
        }
        for object in objects.iter_mut() {
            if let BObject3D::Loaded { name, materials, .. } = object {
                if let Some(t) = self.get(name) {
                    if materials.is_empty() {
                        materials.push(SceneMaterial::neutral());
                    }
                    t.over(&mut materials[0]);
                }
            }
        }
    }

    /// One number for the object list's memo.
    pub fn fingerprint(s: &BState) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        s.get(SceneKey::Recolored).hash(&mut h);
        h.finish()
    }
}

/// The image at `path` — `//` read against the open `.blend`'s folder, as
/// the file's own images are — plugged into Base Color over what the object
/// had (`was`, else the panel's values, else Blender's grey). `None`, and a
/// report, when it does not read as an image.
fn opened(s: &mut BState, path: &str, was: Option<BRecolor>) -> Option<BRecolor> {
    let blend = s.get(&super::loaded_path_key()).map(std::path::Path::new).and_then(std::path::Path::parent).map(std::path::Path::to_path_buf);
    let file = match (path.strip_prefix("//"), blend) {
        (Some(rel), Some(dir)) => dir.join(rel),
        _ => std::path::PathBuf::from(path),
    };
    let file = file.to_string_lossy().into_owned();
    if image(&file).is_none() {
        crate::part::text::info::push(s, crate::part::text::info::BReportLevel::Error, format!("Cannot read image '{path}'"));
        return None;
    }
    let grey = || BRecolor { color: [0.8, 0.8, 0.8, 1.0], metallic: 0.0, roughness: 0.5, image: None };
    Some(BRecolor { image: Some(file), ..was.or_else(|| BRecolors::from_panel(s)).unwrap_or_else(grey) })
}

/// A Surface field set: the active object takes the panel's three. Its
/// Image field typed in, or a file picked in the browser it opened, plugs
/// that image into its Base Color (emptied, unplugs it); Metallic and
/// Roughness keep the image, a colour unplugs it.
pub struct BRecolorRule;

impl BRule for BRecolorRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == FileKey::Selected.path() || matches!(path.segs.as_ref(), ["buts", field] if [COLOR_FIELD, METALLIC_FIELD, ROUGHNESS_FIELD, IMAGE_FIELD].contains(field))
    }
    fn after(&self, path: &BPath, value: &str, s: &mut BState) {
        let Some(active) = s.get(CtxKey::Object).map(str::to_string) else { return };
        let mut all = BRecolors::now(s);
        let was = all.get(&active);
        let field = match path.segs.as_ref() {
            ["buts", field] => Some(*field),
            _ => None,
        };
        let next = match (field, value.is_empty()) {
            (Some(COLOR_FIELD), _) => BRecolors::from_panel(s),
            (Some(IMAGE_FIELD), true) => was.or_else(|| BRecolors::from_panel(s)).map(|t| BRecolor { image: None, ..t }),
            (Some(IMAGE_FIELD), false) => opened(s, value, was),
            (Some(_), _) => BRecolors::from_panel(s).map(|t| BRecolor { image: was.and_then(|w| w.image), ..t }),
            (None, false) if purpose(s) == BFilePurpose::BaseColorImage => opened(s, value, was),
            (None, _) => None,
        };
        let Some(t) = next else { return };
        s.set(ButsKey::Value(IMAGE_FIELD), BValue::Text(t.image.clone().unwrap_or_default()).encode());
        all.set(&active, t);
        all.write(s);
    }
}

#[cfg(test)]
#[path = "recolor_tests.rs"]
mod tests;
