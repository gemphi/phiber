//! The identity of a list of objects: one number that changes when an
//! object, its mesh, its materials or its place does — and is kept for the
//! list while the list is kept.
//!
//! The mesh memo (`cache`) and the tracer (`rendered::key`) each hashed
//! every object on every frame to know whether the scene was still the one
//! they held. The list they are handed is kept between frames (`shown`), so
//! the number is kept with it: the same list is the same scene.

use crate::scene::BObject3D;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// A list's identity.
pub(super) struct BSceneId;

thread_local! {
    /// The list last asked about and its number; the list is held so its
    /// address stays its own while the number is kept.
    static KEPT: RefCell<Option<(Rc<Vec<BObject3D>>, u64)>> = const { RefCell::new(None) };
}

impl BSceneId {
    /// The identity of the kept list `scene`: hashed once per list.
    pub fn of(scene: &Rc<Vec<BObject3D>>) -> u64 {
        if let Some(hit) = KEPT.with(|kept| kept.borrow().as_ref().filter(|(list, _)| Rc::ptr_eq(list, scene)).map(|(_, id)| *id)) {
            return hit;
        }
        let id = Self::hash(scene);
        KEPT.with(|kept| *kept.borrow_mut() = Some((Rc::clone(scene), id)));
        id
    }

    /// The identity of any list, hashed now.
    ///
    /// By identity, not content: mesh data is behind an `Arc`, so its
    /// pointer says whether this is the same mesh, and the world matrix
    /// whether it is in the same place. Neither walks the vertices.
    pub fn hash(scene: &[BObject3D]) -> u64 {
        let mut h = DefaultHasher::new();
        scene.len().hash(&mut h);
        for object in scene {
            match object {
                BObject3D::Primitive { name, shape, x, y } => {
                    (name, *shape as u8, x.to_bits(), y.to_bits()).hash(&mut h);
                }
                BObject3D::Loaded { name, mesh, matrix_world, materials, material_indices, uvs, .. } => {
                    (name, mesh.vertices.len(), mesh.indices.len(), materials.len()).hash(&mut h);
                    (std::sync::Arc::as_ptr(mesh) as usize).hash(&mut h);
                    (std::sync::Arc::as_ptr(material_indices) as usize).hash(&mut h);
                    (std::sync::Arc::as_ptr(uvs) as usize).hash(&mut h);
                    matrix_world.iter().flatten().for_each(|v| v.to_bits().hash(&mut h));
                    Self::materials(materials, &mut h);
                }
            }
        }
        h.finish()
    }

    /// The materials' looks: the colours by value and the base image by
    /// its pixels' address. A Material tab edit keeps the mesh and the
    /// place, so the tracer must still take it again — and build it again,
    /// not pose it (`BRayKey::shape`): one image swapped for another is the
    /// same mesh too.
    pub(super) fn materials(materials: &[crate::phiphx::scene::SceneMaterial], h: &mut impl Hasher) {
        for m in materials {
            m.base_color.iter().chain(&m.viewport_color).chain([&m.metallic, &m.roughness]).for_each(|v| v.to_bits().hash(h));
            (m.baked, m.texture.as_ref().map(|t| std::sync::Arc::as_ptr(&t.rgba) as usize)).hash(h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{BShape, scene_source};

    #[test]
    fn the_same_list_is_one_number_and_a_moved_object_another() {
        let list = Rc::new(scene_source(&crate::state::BState::new()));
        let first = BSceneId::of(&list);
        assert_eq!(BSceneId::of(&list), first, "kept: the same list");
        assert_eq!(BSceneId::hash(&list), first, "kept or hashed, one number");
        let mut moved = (*list).clone();
        if let Some(BObject3D::Primitive { x, .. }) = moved.iter_mut().find(|o| matches!(o, BObject3D::Primitive { shape: BShape::Cube, .. })) {
            *x += 1.0;
        }
        assert_ne!(BSceneId::of(&Rc::new(moved)), first, "a moved object: another");
    }
}
