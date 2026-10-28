//! A scene built in phiber alone animates and renders — no `.blend` is read
//! at any point. From an empty model, the way a user builds it:
//!
//! - Add ▸ Mesh ▸ Plane, Cube and UV Sphere, Add ▸ Light ▸ Sun and Add ▸
//!   Camera, each put in place by typing into the Object tab's Transform
//!   panel;
//! - the Sun's rotation typed in and keyed (I) at frame 1 and at frame 60;
//! - the cube turned red through the Material tab's Base Color
//!   (`SceneKey::Recolored`).
//!
//! The scene is checked as the viewport and F12 read it — the model the
//! draw pass scopes, the lamps at a frame, what a render is handed — and,
//! on a GPU, rendered headlessly through the loop the command line runs
//! (`BHeadlessRender::run_in`) at frames 1, 30 and 60: the frames differ
//! where the sun moved, and the cube's pixels are red.
//!
//! The render needs a GPU and, the first time, minutes for the tracer's
//! kernels to compile, so it is asked for by name:
//!     cargo test --release -p phiber --test a_scene_built_in_phiber_animates_and_renders -- --ignored
//! The frames land in `PHIBER_BUILT_SCENE_OUT`, else under the target's tmp
//! folder (`built_scene/`).

use glam::{Mat4, Vec3};
use phiber::cli::render::BHeadlessRender;
use phiber::phiphx::scene::{SceneCamera, SceneModel};
use phiber::scene::motion::{active_camera_at, lights_at};
use phiber::scene::source::{clear_scene_objects, loaded_shared, scene_objects, set_loaded_at};
use phiber::scene::{BObject3D, recolor::BRecolors};
use phiber::state::val::BValue;
use phiber::state::{BKey as _, BPath, BState, ButsKey, CtxKey, OpKey, action::apply_action};
use phiber::view::BAction;
use std::sync::Arc;

const RED: [f32; 4] = [0.8, 0.05, 0.05, 1.0];
const CUBE_AT: [f32; 3] = [-1.4, -1.5, 0.0];
const SPHERE_AT: [f32; 3] = [1.4, 1.5, 0.0];
/// The sun's rotation (degrees, XYZ Euler) keyed at frame 1 — from the
/// front left, over the camera's shoulder — and at frame 60, from behind.
const SUN_FROM: [f32; 3] = [45.0, 0.0, 20.0];
const SUN_TO: [f32; 3] = [45.0, 0.0, 200.0];

/// The session: what the user did, over the model it was done to.
struct BBuilt {
    file: Arc<SceneModel>,
    state: BState,
}

impl BBuilt {
    /// A draw pass: the model the state's edits make, in scope at its frame.
    fn pass(&self) -> Arc<SceneModel> {
        set_loaded_at(Some(Arc::clone(&self.file)), &self.state);
        loaded_shared().expect("a model in scope")
    }

    /// One click or one typed field, then a draw pass.
    fn act(&mut self, path: BPath, value: &str) {
        apply_action(&mut self.state, &BAction::Set(path, value.to_string()));
        self.pass();
    }

    /// Click `object` in the Outliner: the Object and Material tabs fill.
    fn pick(&mut self, object: &str) {
        self.act(CtxKey::Object.path(), object);
        phiber::scene::select::BSelection { list: vec![object.to_string()] }.write(&mut self.state);
    }

    /// Type a vector into the Transform panel, axis by axis.
    fn type_in(&mut self, field: &'static str, value: [f32; 3]) {
        for (axis, v) in ["x", "y", "z"].into_iter().zip(value) {
            self.act(ButsKey::Axis(field, axis).path(), &v.to_string());
        }
    }

    fn add(&mut self, slug: &str, at: [f32; 3], rotation: [f32; 3], scale: [f32; 3]) -> String {
        self.act(OpKey::SceneEdit.path(), &format!("add:{slug}"));
        let name = self.state.get(CtxKey::Object).expect("the new object is active").to_string();
        self.pick(&name);
        self.type_in("location", at);
        self.type_in("rotation_euler", rotation);
        self.type_in("scale", scale);
        name
    }

    fn at_frame(&mut self, frame: u32) {
        self.state.set(CtxKey::Frame, frame.to_string());
        self.pass();
    }

    /// The scene: built from nothing, a small frame to render.
    fn new() -> Self {
        clear_scene_objects();
        let mut file = SceneModel::default();
        (file.render.resolution_x, file.render.resolution_y, file.render.resolution_percent) = (320, 180, 100);
        let mut state = BState::new();
        phiber::app::seed::seed_render_props(&mut state, &file);
        phiber::app::seed::seed_output_props(&mut state, &file);
        let mut built = Self { file: Arc::new(file), state };
        built.at_frame(1);
        built.add("plane", [0.0, 0.0, -1.0], [0.0; 3], [8.0; 3]);
        let cube = built.add("cube", CUBE_AT, [0.0; 3], [1.0; 3]);
        built.add("uv_sphere", SPHERE_AT, [0.0; 3], [1.0; 3]);
        // Blender's startup camera, where its factory file puts it.
        built.add("camera", [7.3589, -6.9258, 4.9583], [63.559, 0.0, 46.692], [1.0; 3]);
        let sun = built.add("light_sun", [0.0, 0.0, 6.0], SUN_FROM, [1.0; 3]);
        // I on the Sun at frame 1, then turned and keyed at frame 60.
        built.act(OpKey::Keyframe.path(), "insert");
        built.at_frame(60);
        built.pick(&sun);
        built.type_in("rotation_euler", SUN_TO);
        built.act(OpKey::Keyframe.path(), "insert");
        // The cube's Base Color, in the Material tab.
        built.pick(&cube);
        built.act(ButsKey::Value("diffuse_color").path(), &BValue::Color(RED).encode());
        built.at_frame(1);
        built
    }
}

fn world_of(objects: &[BObject3D], name: &str) -> [[f32; 4]; 4] {
    match objects.iter().find(|o| o.name() == name) {
        Some(BObject3D::Loaded { matrix_world, .. }) => *matrix_world,
        other => panic!("{name} is not in the scene as a mesh: {other:?}"),
    }
}

/// Where the sun's light travels at `frame`: its −Z axis.
fn sun_heading(scene: &SceneModel, frame: f32) -> Vec3 {
    let sun = lights_at(scene, Some(frame)).into_iter().find(|l| l.name == "Sun").expect("the Sun lights the scene");
    -Mat4::from_cols_array_2d(&sun.matrix_world).z_axis.truncate()
}

#[test]
fn a_scene_built_with_no_file_keys_its_sun_and_reds_its_cube() {
    let built = BBuilt::new();
    let scene = built.pass();
    assert!(scene.source_path.is_none(), "no file behind it");
    assert_eq!(scene.lights.iter().map(|l| (l.name.as_str(), l.kind.as_str())).collect::<Vec<_>>(), [("Sun", "SUN")]);
    assert_eq!(scene.cameras.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["Camera"]);
    assert!(scene.motion.animated(), "the Sun's keys move the scene");

    // The Sun turns between its keys: lit from the front left at 1, from
    // behind at 60, somewhere between at 30.
    let expect = |deg: [f32; 3]| {
        let [x, y, z] = deg.map(f32::to_radians);
        -(glam::Quat::from_euler(glam::EulerRot::ZYX, z, y, x) * Vec3::Z)
    };
    let (first, middle, last) = (sun_heading(&scene, 1.0), sun_heading(&scene, 30.0), sun_heading(&scene, 60.0));
    assert!(first.abs_diff_eq(expect(SUN_FROM), 1e-4), "frame 1: {first} vs {}", expect(SUN_FROM));
    assert!(last.abs_diff_eq(expect(SUN_TO), 1e-4), "frame 60: {last} vs {}", expect(SUN_TO));
    assert!(!middle.abs_diff_eq(first, 1e-2) && !middle.abs_diff_eq(last, 1e-2), "played between: {middle}");

    // The meshes stand where they were typed in, the cube red in every
    // place a mode reads its colour.
    let objects = scene_objects(&built.state);
    assert_eq!(Mat4::from_cols_array_2d(&world_of(&objects, "Cube")).w_axis.truncate(), Vec3::from(CUBE_AT));
    assert_eq!(Mat4::from_cols_array_2d(&world_of(&objects, "Sphere")).w_axis.truncate(), Vec3::from(SPHERE_AT));
    assert_eq!(BRecolors::now(&built.state).get("Cube").map(|r| r.color), Some(RED), "Base Color went to ctx.scene.recolored");
    let cube = objects.iter().find(|o| o.name() == "Cube").unwrap();
    let BObject3D::Loaded { materials, .. } = cube else { unreachable!() };
    assert_eq!((materials[0].base_color, materials[0].bsdf.base), (RED, RED));

    // A render is handed the three meshes, through the added camera.
    let cast = phiber::scene::render_cast::BRenderCast::of(&built.state, Some(&scene));
    let mut names: Vec<&str> = cast.objects.iter().map(|o| o.name()).collect();
    names.sort();
    assert_eq!(names, ["Cube", "Plane", "Sphere"]);
    let camera = active_camera_at(&scene, Some(1.0)).expect("the added camera");
    assert!((Mat4::from_cols_array_2d(&camera.matrix_world).w_axis.truncate() - Vec3::new(7.3589, -6.9258, 4.9583)).length() < 1e-3);
}

/// `point` in the frame `w × h` the camera renders, as pixel column and row
/// (`BKE_camera_sensor_fit` AUTO: the sensor spans the wider side).
fn pixel_of(camera: &SceneCamera, point: [f32; 3], w: u32, h: u32) -> (u32, u32) {
    let p = Mat4::from_cols_array_2d(&camera.matrix_world).inverse().transform_point3(Vec3::from(point));
    let focal = camera.focal_length_mm / camera.sensor_width * w.max(h) as f32;
    let (x, y) = (w as f32 * 0.5 + p.x / -p.z * focal, h as f32 * 0.5 - p.y / -p.z * focal);
    (x.round() as u32, y.round() as u32)
}

/// The mean colour of the 7 × 7 pixels around `(x, y)`, 0–1.
fn patch(image: &image::RgbaImage, (x, y): (u32, u32)) -> [f32; 3] {
    let mut sum = [0.0; 3];
    let mut n = 0.0;
    for py in y.saturating_sub(3)..=(y + 3).min(image.height() - 1) {
        for px in x.saturating_sub(3)..=(x + 3).min(image.width() - 1) {
            let p = image.get_pixel(px, py).0;
            (0..3).for_each(|c| sum[c] += p[c] as f32 / 255.0);
            n += 1.0;
        }
    }
    sum.map(|s| s / n)
}

/// The mean absolute difference of two frames, per channel, 0–1.
fn difference(a: &image::RgbaImage, b: &image::RgbaImage) -> f32 {
    let total: u64 = a.pixels().zip(b.pixels()).map(|(p, q)| (0..3).map(|c| p.0[c].abs_diff(q.0[c]) as u64).sum::<u64>()).sum();
    total as f32 / (a.width() * a.height() * 3) as f32 / 255.0
}

#[test]
#[ignore = "renders on the GPU: the tracer's kernels take minutes to compile the first time"]
fn a_scene_built_with_no_file_renders_its_sun_moving_and_its_cube_red() {
    let out = std::env::var("PHIBER_BUILT_SCENE_OUT").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("built_scene"));
    let built = BBuilt::new();
    let scene = built.pass();
    let camera = active_camera_at(&scene, Some(1.0)).expect("the added camera");
    let render = |prefix: &str, frames: &[f32]| {
        let job = BHeadlessRender {
            file: String::new(),
            out: out.join(prefix).to_string_lossy().to_string(),
            frame: frames.to_vec(),
            samples: Some(64),
            draft: false,
            camera: None,
            look: Default::default(),
        };
        let mut state = built.state.clone();
        state.set(CtxKey::Frame, "1");
        job.run_in((*built.file).clone(), state).unwrap_or_else(|why| panic!("render: {why}"));
        frames.iter().map(|&f| image::open(BHeadlessRender::frame_path(&job.out, f as i64).1).expect("the frame was written").to_rgba8()).collect::<Vec<_>>()
    };
    let frames = render("frame_", &[1.0, 30.0, 60.0]);
    let again = render("again_", &[1.0]).remove(0);
    let (w, h) = frames[0].dimensions();
    assert_eq!((w, h), (320, 180), "the resolution the scene asked for");

    // Where the sun moved: the frames differ by far more than the noise
    // between two renders of the same frame.
    let noise = difference(&frames[0], &again);
    let (to_middle, to_end) = (difference(&frames[0], &frames[1]), difference(&frames[0], &frames[2]));
    println!("mean difference: same frame {noise:.4}, frame 1 → 30 {to_middle:.4}, frame 1 → 60 {to_end:.4}");
    assert!(to_end > 0.02 && to_end > 4.0 * noise, "frame 60 is lit from elsewhere: {to_end} vs noise {noise}");
    assert!(to_middle > 2.0 * noise, "so is frame 30: {to_middle} vs noise {noise}");

    // The cube's face toward the camera: red, lit at frame 1 and in the
    // sun's shadow at 60. The sphere beside it is grey.
    let (cube, sphere) = (pixel_of(&camera, CUBE_AT, w, h), pixel_of(&camera, SPHERE_AT, w, h));
    let lit = patch(&frames[0], cube);
    let shaded = patch(&frames[2], cube);
    let grey = patch(&frames[0], sphere);
    println!("cube at {cube:?}: {lit:?} at frame 1, {shaded:?} at frame 60; sphere at {sphere:?}: {grey:?}");
    assert!(lit[0] > 0.2 && lit[0] > 2.0 * lit[1] && lit[0] > 2.0 * lit[2], "the cube is red: {lit:?}");
    assert!(lit[0] > shaded[0] + 0.05, "the sun turned away from it: {lit:?} → {shaded:?}");
    assert!(grey[0] < 1.5 * grey[1].max(0.01), "the sphere is not: {grey:?}");
}
