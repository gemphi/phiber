//! The fonts that ship in `assets/phi-fonts`, found at start-up rather than
//! listed in code. Every `.otf` / `.ttf` under a family folder is registered
//! with the text system, grouped by the family name inside the file (name
//! ID 16, else 1), and offered in *Preferences ▸ Typography ▸ Family* — so
//! adding a typeface is dropping its folder in. A folder's optional
//! `font.toml` sets the defaults its families start on (`weight`, `body`,
//! `header`, `caption`); see `assets/phi-fonts/README.md`.
//!
//! The browser build has no disk: it gets the one face compiled into the
//! binary (`font::PHI_SOFT_FACES`), as does a desktop build whose folder is
//! missing, so the window always has its own face.

use super::BUiStyleDefaults;
use super::font::BFontFamily;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// One family found in the folder: every weight of one name.
pub struct BBundledFamily {
    /// The value `typography.family` holds: the name, lower-cased, words
    /// joined by `_` ("Phi-Soft Condensed" → `phi_soft_condensed`).
    pub slug: &'static str,
    /// The family name inside the files — what the text system is asked for
    /// and what Preferences shows.
    pub name: &'static str,
    /// The weights it ships (`usWeightClass`: 300 Light, 400 Regular, …).
    pub weights: Vec<u16>,
    /// What its folder's `font.toml` sets; `None` keeps the theme's own.
    pub defaults: BFamilyDefaults,
    faces: Vec<BFace>,
}

/// A family's own starting values, from its folder's `font.toml`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BFamilyDefaults {
    pub weight: Option<f32>,
    pub body: Option<f32>,
    pub header: Option<f32>,
    pub caption: Option<f32>,
}

enum BFace {
    #[cfg_attr(target_family = "wasm", allow(dead_code))]
    File(PathBuf),
    Embedded(&'static [u8]),
}

static BUNDLED: OnceLock<Vec<BBundledFamily>> = OnceLock::new();

/// Every bundled family, in folder then file order, found once.
pub fn bundled() -> &'static [BBundledFamily] {
    BUNDLED.get_or_init(discover)
}

/// The bundled family a slug names.
pub fn find(slug: &str) -> Option<&'static BBundledFamily> {
    bundled().iter().find(|family| family.slug == slug)
}

impl BBundledFamily {
    /// Every face's bytes, for `TextSystem::add_fonts`.
    pub fn face_bytes(&self) -> Vec<Cow<'static, [u8]>> {
        self.faces
            .iter()
            .filter_map(|face| match face {
                BFace::File(path) => std::fs::read(path).ok().map(Cow::Owned),
                BFace::Embedded(bytes) => Some(Cow::Borrowed(*bytes)),
            })
            .collect()
    }
}

fn discover() -> Vec<BBundledFamily> {
    #[cfg(not(target_family = "wasm"))]
    {
        let found = from_folder(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/phi-fonts"));
        if !found.is_empty() {
            return found;
        }
    }
    let mut families = Vec::new();
    for bytes in super::font::PHI_SOFT_FACES {
        add_face(&mut families, bytes, BFace::Embedded(bytes), BFamilyDefaults::default());
    }
    families
}

#[cfg(not(target_family = "wasm"))]
fn from_folder(root: PathBuf) -> Vec<BBundledFamily> {
    let sorted = |dir: &std::path::Path| {
        let mut paths: Vec<PathBuf> = std::fs::read_dir(dir).into_iter().flatten().flatten().map(|e| e.path()).collect();
        paths.sort();
        paths
    };
    let mut families = Vec::new();
    for folder in sorted(&root).into_iter().filter(|path| path.is_dir()) {
        let defaults = manifest(&std::fs::read_to_string(folder.join("font.toml")).unwrap_or_default());
        for file in sorted(&folder) {
            let font = file.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("otf") || ext.eq_ignore_ascii_case("ttf"));
            if let (true, Ok(bytes)) = (font, std::fs::read(&file)) {
                add_face(&mut families, &bytes, BFace::File(file), defaults);
            }
        }
    }
    families
}

fn add_face(families: &mut Vec<BBundledFamily>, bytes: &[u8], face: BFace, defaults: BFamilyDefaults) {
    let Some((name, weight)) = describe(bytes) else { return };
    let index = match families.iter().position(|family| family.name == name) {
        Some(index) => index,
        None => {
            let slug = slug_of(&name);
            let leak = |text: String| -> &'static str { Box::leak(text.into_boxed_str()) };
            families.push(BBundledFamily { slug: leak(slug), name: leak(name), weights: Vec::new(), defaults, faces: Vec::new() });
            families.len() - 1
        }
    };
    let family = &mut families[index];
    if !family.weights.contains(&weight) {
        family.weights.push(weight);
        family.weights.sort_unstable();
    }
    family.faces.push(face);
}

/// A face's family name (name ID 16, else 1, Windows platform) and its
/// `OS/2` weight class, read straight from the font's tables.
pub fn describe(bytes: &[u8]) -> Option<(String, u16)> {
    let u16_at = |at: usize| Some(u16::from_be_bytes(bytes.get(at..at + 2)?.try_into().ok()?));
    let u32_at = |at: usize| Some(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?) as usize);
    let table = |tag: &[u8; 4]| (0..u16_at(4)? as usize).find_map(|i| (bytes.get(12 + 16 * i..16 + 16 * i)? == tag).then(|| u32_at(20 + 16 * i)).flatten());
    let weight = u16_at(table(b"OS/2")? + 4)?;
    let name = table(b"name")?;
    let (count, strings) = (u16_at(name + 2)? as usize, name + u16_at(name + 4)? as usize);
    let read = |id: u16| {
        (0..count).find_map(|i| {
            let record = name + 6 + 12 * i;
            (u16_at(record)? == 3 && u16_at(record + 6)? == id).then_some(())?;
            let (len, at) = (u16_at(record + 8)? as usize, strings + u16_at(record + 10)? as usize);
            let units: Vec<u16> = bytes.get(at..at + len)?.chunks_exact(2).map(|pair| u16::from_be_bytes([pair[0], pair[1]])).collect();
            String::from_utf16(&units).ok().filter(|text| !text.is_empty())
        })
    };
    Some((read(16).or_else(|| read(1))?, weight))
}

/// `key = number` lines; `#` starts a comment; unknown keys are ignored.
fn manifest(text: &str) -> BFamilyDefaults {
    let mut defaults = BFamilyDefaults::default();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("");
        let Some((key, value)) = line.split_once('=') else { continue };
        let Ok(value) = value.trim().parse::<f32>() else { continue };
        let slot = match key.trim() {
            "weight" => &mut defaults.weight,
            "body" => &mut defaults.body,
            "header" => &mut defaults.header,
            "caption" => &mut defaults.caption,
            _ => continue,
        };
        *slot = Some(value);
    }
    defaults
}

fn slug_of(name: &str) -> String {
    let mut slug = String::new();
    for c in name.chars() {
        match c.is_ascii_alphanumeric() {
            true => slug.push(c.to_ascii_lowercase()),
            false if !slug.is_empty() && !slug.ends_with('_') => slug.push('_'),
            false => {}
        }
    }
    slug.trim_end_matches('_').to_string()
}

/// `base` with `family`'s own defaults written over its typography — what
/// Preferences resets to and what an unset value resolves to while that
/// family is chosen. A family without a `font.toml` keeps `base`.
pub fn with_family_defaults(base: &'static BUiStyleDefaults, family: BFontFamily) -> &'static BUiStyleDefaults {
    let BFontFamily::Bundled(slug) = family else { return base };
    let Some(own) = find(slug).map(|family| family.defaults) else { return base };
    let mut adjusted = *base;
    adjusted.typography.weight = own.weight.unwrap_or(base.typography.weight);
    adjusted.typography.body = own.body.unwrap_or(base.typography.body);
    adjusted.typography.header = own.header.unwrap_or(base.typography.header);
    adjusted.typography.caption = own.caption.unwrap_or(base.typography.caption);
    if adjusted == *base {
        return base;
    }
    static CACHE: Mutex<Vec<(usize, &'static str, &'static BUiStyleDefaults)>> = Mutex::new(Vec::new());
    let mut cache = CACHE.lock().unwrap_or_else(|poison| poison.into_inner());
    let key = base as *const BUiStyleDefaults as usize;
    if let Some((_, _, cached)) = cache.iter().find(|(at, name, _)| *at == key && *name == slug) {
        return cached;
    }
    let leaked: &'static BUiStyleDefaults = Box::leak(Box::new(adjusted));
    cache.push((key, slug, leaked));
    leaked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_read_like_the_old_ones() {
        assert_eq!(slug_of("Phi-Soft Condensed"), "phi_soft_condensed");
        assert_eq!(slug_of("Phi-Soft"), "phi_soft");
    }

    #[test]
    fn a_manifest_sets_only_what_it_names() {
        let defaults = manifest("# Phi-Soft\nweight = 300 # Light\nbody=13\nsize = 9\n");
        assert_eq!(defaults, BFamilyDefaults { weight: Some(300.0), body: Some(13.0), header: None, caption: None });
    }

    #[test]
    fn the_compiled_in_face_describes_itself() {
        let (name, weight) = describe(super::super::font::PHI_SOFT_FACES[0]).expect("a readable font");
        assert_eq!((name.as_str(), weight), ("Phi-Soft Condensed", 300));
    }
}
