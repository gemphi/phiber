//! Showing a scene: what an opened file and a built match both end in.

use super::BPhiberApp;

impl BPhiberApp {
    /// `model` as the shown scene under `key`: cached for the viewport and
    /// the Outliner, its large meshes' pick trees and outline points warmed
    /// on a thread before the first click or orbit asks for them, and — as
    /// another document — the old Render Result and images let go.
    pub(crate) fn adopt_scene(&mut self, key: String, model: std::sync::Arc<crate::phiphx::scene::SceneModel>) {
        crate::scene::warm::BWarm::of(&model).start();
        self.scene_cache = Some((key, model));
        crate::scene::render_job::forget();
        crate::scene::image_list::BImageList::forget();
        self.render_on_entry();
    }
}
