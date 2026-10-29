//! Shared geometry reaches the tracer as one mesh placed many times: two
//! objects holding the same `Arc<SceneMesh>` (a collection's member and
//! its instance, `phiphx::scene::share`) put their triangles up once
//! (`Scene::place`), while two meshes alike in content but not in `Arc`
//! stay two uploads.

use super::scene;
use crate::phiphx::scene::{SceneCorners, SceneMesh, SceneNormals};
use crate::scene::{BObject3D, BShading};
use gpui::white;
use std::sync::Arc;

fn palette() -> super::super::mesh::BMeshPalette {
    let grey = white();
    super::super::mesh::BMeshPalette { shading: BShading::Solid, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey, matcap: None, texture: false, studio: false }
}

fn triangle() -> Arc<SceneMesh> {
    Arc::new(SceneMesh {
        vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        indices: vec![0, 1, 2],
        normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
        corners: SceneCorners::default(),
    })
}

/// One object's geometry as `phiphx::scene::share` hands it over: the mesh,
/// the per-triangle slots and the UVs, each behind an `Arc` that every
/// object sharing the geometry holds a clone of.
type Geometry = (Arc<SceneMesh>, Arc<Vec<u16>>, Arc<Vec<[f32; 2]>>);

fn geometry() -> Geometry {
    (triangle(), Arc::default(), Arc::default())
}

/// `name` placed at `x`, drawn with `geo`. Sharing is keyed on all three
/// `Arc`s together, as `share::alias` copies all three.
fn placed(name: &str, geo: &Geometry, x: f32) -> BObject3D {
    BObject3D::Loaded {
        name: name.into(),
        mesh: geo.0.clone(),
        location: [x, 0.0, 0.0],
        scale: [1.0; 3],
        matrix_world: phiray::Instance::translation([x, 0.0, 0.0]),
        materials: Vec::new(),
        material_indices: geo.1.clone(),
        uvs: geo.2.clone(),
        color: [1.0; 4],
        shadow: Default::default(),
    }
}

#[test]
fn objects_holding_one_arc_go_up_as_one_mesh_placed_twice() {
    let car = geometry();
    let tree = geometry();
    let objects = [placed("Car", &car, 0.0), placed("Lot/Car", &car, 5.0), placed("Tree", &tree, 9.0)];
    let mut s = scene::build(&objects, &palette());
    s.seal();
    assert_eq!((s.meshes.len(), s.insts.len(), s.len()), (2, 3, 2), "two meshes, three placements, two triangles");
    assert_eq!((s.mesh_of(0), s.mesh_of(1), s.mesh_of(2)), (0, 0, 1));
    assert_eq!(s.insts[1].world[3], [5.0, 0.0, 0.0, 1.0], "the instance keeps its own place");
    // The instance order follows the objects, which is what `reinstance`
    // hands matrices by.
    assert_eq!(scene::worlds(&objects).len(), 3);
}

/// Geometry alike in content but held behind its own `Arc` — two separate
/// reads of one mesh — is two uploads: sharing is by identity, never by
/// walking the vertices (`BSceneId`'s rule).
#[test]
fn alike_geometry_behind_its_own_arc_is_not_shared() {
    let objects = [placed("A", &geometry(), 0.0), placed("B", &geometry(), 5.0)];
    let mut s = scene::build(&objects, &palette());
    s.seal();
    assert_eq!((s.meshes.len(), s.insts.len(), s.len()), (2, 2, 2));
}
