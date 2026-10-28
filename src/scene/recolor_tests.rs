//! `recolor.rs`'s tests: the key round-trips, an object with no material
//! is given one, the rule reads the panel, and the scene the viewport and
//! the render draw carries the edit — an image plugged into Base Color
//! included.

use super::*;

fn object(name: &str, materials: Vec<SceneMaterial>) -> crate::phiphx::scene::SceneObject {
    crate::phiphx::scene::SceneObject {
        name: name.to_string(),
        mesh: crate::phiphx::scene::SceneMesh {
            vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: crate::phiphx::scene::SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
            corners: Default::default(),
        }
        .into(),
        location: [0.0; 3],
        scale: [1.0; 3],
        matrix_world: crate::phiphx::scene::IDENTITY_M4,
        materials,
        material_indices: Default::default(),
        uvs: Default::default(),
        color: [1.0; 4],
        shadow: Default::default(),
    }
}

fn first_material(o: &BObject3D) -> Option<&SceneMaterial> {
    match o {
        BObject3D::Loaded { materials, .. } => materials.first(),
        BObject3D::Primitive { .. } => None,
    }
}

const RED: BRecolor = BRecolor { color: [0.8, 0.05, 0.05, 1.0], metallic: 0.25, roughness: 0.3, image: None };

#[test]
fn the_list_round_trips() {
    let mut s = BState::new();
    let mut all = BRecolors::default();
    all.set("Cube", RED);
    all.set("Shoe Left", BRecolor { color: [0.1, 0.2, 0.9, 1.0], metallic: 0.0, roughness: 0.5, image: None });
    all.write(&mut s);
    assert_eq!(s.get(SceneKey::Recolored), Some("Cube=0.8,0.05,0.05,1,0.25,0.3|Shoe Left=0.1,0.2,0.9,1,0,0.5"));
    let again = BRecolors::now(&s);
    assert_eq!(again, all);
    assert_eq!(again.get("Cube"), Some(RED));
    assert_eq!(again.get("Sphere"), None);
}

/// A cube added from Add ▸ Mesh has no material; recoloured, it has one
/// in the colour asked for, and an untouched object keeps the file's.
#[test]
fn the_scene_drawn_carries_the_edit_and_a_bare_object_gets_a_material() {
    let mut s = BState::new();
    let mut textured = SceneMaterial::neutral();
    textured.base_color = [0.5, 0.5, 0.5, 1.0];
    textured.baked = true;
    let model = crate::phiphx::scene::SceneModel {
        objects: vec![object("Cube", Vec::new()), object("Floor", vec![textured.clone()]), object("Sphere", vec![textured.clone()])],
        ..Default::default()
    };
    let mut all = BRecolors::default();
    all.set("Cube", RED);
    all.set("Floor", RED);
    all.write(&mut s);
    let objects = super::super::current_scene(&s, Some(&model));
    let cube = first_material(&objects[0]).expect("the bare cube was given a material");
    assert_eq!((cube.base_color, cube.viewport_color, cube.bsdf.base), (RED.color, RED.color, RED.color), "every mode's colour");
    assert_eq!((cube.metallic, cube.roughness, cube.viewport_pbr), (0.25, 0.3, [0.3, 0.25]));
    let floor = first_material(&objects[1]).unwrap();
    assert_eq!(floor.base_color, RED.color);
    assert!(!floor.baked && floor.texture.is_none(), "the flat colour replaces the baked base");
    assert_eq!(first_material(&objects[2]), Some(&textured), "an unedited object keeps the file's material");
}

#[test]
fn the_rule_takes_the_panels_fields_for_the_active_object() {
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    s.set(ButsKey::Value(COLOR_FIELD), BValue::Color(RED.color).encode());
    s.set(ButsKey::Value(METALLIC_FIELD), BValue::Float(RED.metallic).encode());
    assert!(BRecolors::from_panel(&s).is_none(), "two of three fields are not an edit");
    s.set(ButsKey::Value(ROUGHNESS_FIELD), BValue::Float(RED.roughness).encode());
    let rule = BRecolorRule;
    assert!(rule.wants(&ButsKey::Value(COLOR_FIELD).path()) && !rule.wants(&ButsKey::Value("show_name").path()));
    rule.after(&ButsKey::Value(COLOR_FIELD).path(), "", &mut s);
    assert_eq!(BRecolors::now(&s).get("Cube"), Some(RED));
}

/// A one-colour PNG, written where the test can open it.
fn png(name: &str, rgb: [u8; 3]) -> String {
    let dir = std::env::temp_dir().join(format!("phiber-recolor-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(name);
    image::RgbaImage::from_pixel(3, 2, image::Rgba([rgb[0], rgb[1], rgb[2], 255])).save(&file).unwrap();
    file.to_string_lossy().into_owned()
}

#[test]
fn an_image_round_trips_with_a_comma_in_its_path() {
    let mut all = BRecolors::default();
    all.set("Cube", BRecolor { image: Some("C:/maps/wood, oak.png".into()), ..RED });
    all.set("Floor", RED);
    let again = BRecolors::parse(&all.encode());
    assert_eq!(again, all);
    assert_eq!(again.get("Cube").and_then(|t| t.image).as_deref(), Some("C:/maps/wood, oak.png"));
    assert_eq!(BRecolors::parse("Cube=0.8,0.05,0.05,1,0.25,0.3").get("Cube"), Some(RED), "an edit from before images reads as a flat colour");
}

/// Image Texture ▸ Open on the Base Color: the file picked in the browser
/// is the material's base image — the whole colour, untinted — and the
/// Image field names it; Roughness keeps it, a colour unplugs it.
#[test]
fn an_opened_image_is_the_base_colour_until_a_colour_is_typed() {
    let green = png("green.png", [0, 255, 0]);
    let model = crate::phiphx::scene::SceneModel { objects: vec![object("Cube", vec![SceneMaterial::neutral()])], ..Default::default() };
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    for (field, value) in [(COLOR_FIELD, BValue::Color([0.2, 0.4, 0.6, 1.0])), (METALLIC_FIELD, BValue::Float(0.0)), (ROUGHNESS_FIELD, BValue::Float(0.5))] {
        s.set(ButsKey::Value(field), value.encode());
    }
    let set = |s: &mut BState, path: BPath, value: &str| {
        s.set(&path, value);
        BRecolorRule.after(&path, value, s);
    };
    // A browser committed for another purpose plugs nothing in.
    set(&mut s, FileKey::Selected.path(), &green);
    assert_eq!(BRecolors::now(&s).get("Cube"), None, "an Open of a .blend is not an image for Base Color");

    s.set(FileKey::Purpose, BFilePurpose::BaseColorImage.slug());
    set(&mut s, FileKey::Selected.path(), &green);
    let t = BRecolors::now(&s).get("Cube").expect("the cube has an edit");
    assert_eq!((t.image.as_deref(), t.color), (Some(green.as_str()), [0.2, 0.4, 0.6, 1.0]), "the image, over the colour it had");
    assert_eq!(s.get(ButsKey::Value(IMAGE_FIELD)), Some(green.as_str()), "the Image field names it");
    let drawn = super::super::current_scene(&s, Some(&model));
    let m = first_material(&drawn[0]).unwrap();
    let tex = m.texture.as_ref().expect("the image is the base texture");
    assert_eq!((tex.width, tex.height, &tex.rgba[..4]), (3, 2, &[0, 255, 0, 255][..]), "decoded as the file's own images are");
    assert!(m.baked && m.base_color_texture.as_deref() == Some(green.as_str()), "the whole Base Color: nothing tints it");
    assert_eq!(m.viewport_color, [0.2, 0.4, 0.6, 1.0], "Solid's Material colour stays the colour");

    set(&mut s, ButsKey::Value(ROUGHNESS_FIELD).path(), &BValue::Float(0.9).encode());
    let t = BRecolors::now(&s).get("Cube").unwrap();
    assert_eq!((t.roughness, t.image.as_deref()), (0.9, Some(green.as_str())), "Roughness keeps the image");

    set(&mut s, ButsKey::Value(COLOR_FIELD).path(), &BValue::Color([0.9, 0.1, 0.1, 1.0]).encode());
    let t = BRecolors::now(&s).get("Cube").unwrap();
    assert_eq!((t.color, t.image), ([0.9, 0.1, 0.1, 1.0], None), "a colour unplugs it");
    assert_eq!(s.get(ButsKey::Value(IMAGE_FIELD)), Some(""));
    let drawn = super::super::current_scene(&s, Some(&model));
    let m = first_material(&drawn[0]).unwrap();
    assert!(m.texture.is_none() && !m.baked && m.base_color_texture.is_none(), "the flat colour again");

    // Typed into the Image field: the same plug.
    set(&mut s, ButsKey::Value(IMAGE_FIELD).path(), &green);
    assert_eq!(BRecolors::now(&s).get("Cube").and_then(|t| t.image), Some(green.clone()));
}

#[test]
fn an_image_that_does_not_read_is_reported_and_changes_nothing() {
    let mut s = BState::new();
    s.set(CtxKey::Object, "Cube");
    let mut all = BRecolors::default();
    all.set("Cube", RED);
    all.write(&mut s);
    let missing = std::env::temp_dir().join("phiber-recolor-nowhere").join("missing.png").to_string_lossy().into_owned();
    s.set(ButsKey::Value(IMAGE_FIELD), &missing);
    BRecolorRule.after(&ButsKey::Value(IMAGE_FIELD).path(), &missing, &mut s);
    assert_eq!(BRecolors::now(&s).get("Cube"), Some(RED), "the edit stands as it was");
    let reports = crate::part::text::info::reports(&s);
    assert!(reports.iter().any(|(level, text)| level == "ERROR" && text.contains("missing.png")), "{reports:?}");
}
