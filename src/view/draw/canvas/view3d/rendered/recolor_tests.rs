//! The Material tab's Base Color image (`scene::recolor`) as the tracer
//! takes it: the face's texture, untinted, in Material Preview and
//! Rendered; Solid keeps the colour; and plugging an image in, swapping it
//! or unplugging it is another scene to build, not a pose of the one held.

use crate::phiphx::scene::{SceneCorners, SceneMaterial, SceneMesh, SceneNormals};
use crate::scene::recolor::BRecolor;
use crate::scene::{BObject3D, BShading};
use gpui::white;

fn card(material: SceneMaterial) -> BObject3D {
    BObject3D::Loaded {
        name: "Card".into(),
        mesh: std::sync::Arc::new(SceneMesh {
            vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
            corners: SceneCorners::default(),
        }),
        location: [0.0; 3],
        scale: [1.0; 3],
        matrix_world: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
        materials: vec![material],
        material_indices: Default::default(),
        uvs: std::sync::Arc::new(vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]),
        color: [1.0; 4],
        shadow: Default::default(),
    }
}

/// A `w × h` PNG of one colour, written where the test can open it.
fn png(name: &str, rgb: [u8; 3]) -> String {
    let dir = std::env::temp_dir().join(format!("phiber-base-image-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(name);
    image::RgbaImage::from_pixel(4, 2, image::Rgba([rgb[0], rgb[1], rgb[2], 255])).save(&file).unwrap();
    file.to_string_lossy().into_owned()
}

#[test]
fn an_image_on_base_color_is_the_faces_texture_and_a_new_build() {
    let blue = [0.1, 0.2, 0.9, 1.0];
    let edit = |image: Option<String>| BRecolor { color: blue, metallic: 0.0, roughness: 0.5, image };
    // The still's palette for `shading` (`BStillLook`): what the viewport
    // builds for it, studio light and all.
    let palette = |shading| super::still::BStillLook::new(shading, white()).palette();
    let face = |scene: &phiray::Scene| scene.mats[(scene.faces[0][3] & 0xffff) as usize];

    let pictured = card(edit(Some(png("green.png", [0, 255, 0]))).material());
    for shading in [BShading::Material, BShading::Rendered] {
        let scene = super::scene::build(std::slice::from_ref(&pictured), &palette(shading));
        let mat = face(&scene);
        assert_ne!(mat.tex, phiray::NO_TEX, "{shading:?}: the image is the face's texture");
        assert_eq!(mat.color[..3], [1.0; 3], "{shading:?}: nothing tints it — the image is the whole Base Color");
    }
    let solid = super::scene::build(std::slice::from_ref(&pictured), &palette(BShading::Solid));
    assert_eq!((face(&solid).tex, face(&solid).color), (phiray::NO_TEX, blue), "Solid's Material colour stays the colour");

    let flat = card(edit(None).material());
    let scene = super::scene::build(std::slice::from_ref(&flat), &palette(BShading::Material));
    assert_eq!((face(&scene).tex, face(&scene).color), (phiray::NO_TEX, blue), "unplugged: the flat colour");

    // The same mesh in the same place each time: only a build takes a new
    // material, so each change is a new shape, never a pose.
    let other = card(edit(Some(png("red.png", [255, 0, 0]))).material());
    let shape = |o: &BObject3D| super::key::BRayKey::shape(std::slice::from_ref(o), &palette(BShading::Material));
    assert_ne!(shape(&pictured), shape(&flat), "plugging an image in builds again");
    assert_ne!(shape(&pictured), shape(&other), "and so does swapping it for another");
    assert_eq!(shape(&pictured), shape(&card(edit(Some(png("green.png", [0, 255, 0]))).material())), "the same image read once is the same scene");
}
