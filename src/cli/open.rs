//! What a background render opens: a `.blend` (or any file `Phx` reads),
//! `play:<choice>` — a match the Play host would build
//! (`scene::play::open`; `play:` alone is the default match) — or
//! `wear:<set>`, a footwear set on its stage (`scene::wear::open`).

use crate::phiphx::Phx;
use crate::phiphx::scene::SceneModel;

pub fn open(file: &str) -> Result<SceneModel, String> {
    if let Some(choice) = file.strip_prefix("play:") {
        return crate::scene::play::open(choice);
    }
    if let Some(set) = file.strip_prefix("wear:") {
        return crate::scene::wear::open(set);
    }
    // A background render is a final render: modifiers at their Render
    // settings, as Blender evaluates them for `-b … -f`
    // (`phiphx::blend::eval_for_render`).
    crate::phiphx::blend::eval_for_render(true);
    Phx::open(std::path::Path::new(file)).map_err(|e| format!("{e:?}"))
}
