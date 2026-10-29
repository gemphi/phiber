//! Every graph-style editor paints its text where it can be read.
//!
//! The Timeline, Dope Sheet, Graph editor and NLA paint their ruler numbers,
//! channel names, value labels and strip names on a `canvas`, and so do the
//! node editors their socket names and values (`canvas/node/paint/dot.rs`):
//! shaped and painted glyph by glyph rather than laid out as divs. A glyph
//! can be painted and still not be seen, when something opaque is painted
//! over it later — the node editors' cards were divs painted after the canvas
//! that wrote their sockets' names, so every name inside a card was under the
//! card's fill and only the ends that ran past its edge showed.
//!
//! The no-op text system the GPUI tests run on rasterizes nothing, so a glyph
//! never reaches the scene there. These windows run on [`BoxText`], which
//! shapes as the no-op system does and gives every glyph a box the size of
//! its font, so each one lands in the scene as a sprite and
//! `painted_glyphs` can say which are covered.

use gpui::{
    Bounds, DevicePixels, Font, FontId, FontMetrics, FontRun, GlyphId, LineLayout, NoopTextSystem, Pixels,
    PlatformTextSystem, RenderGlyphParams, Size, TestAppContext, TestDispatcher, TextRenderingMode, point, px, size,
};
use phiber::app::BPhiberApp;
use phiber::host::BHostType;
use phiber::phiphx::blend::shade::graph::{BlendNode, BlendNodeGraph, BlendNodeLink};
use phiber::phiphx::blend::shade::socket::BlendSocket;
use phiber::phiphx::blend::{BFCurve, BKey, BNlaStrip, BNlaTrack, BlendMarker, BlendNla, EXTEND_HOLD};
use phiber::phiphx::scene::{SceneAction, SceneMaterial, SceneMesh, SceneModel, SceneNormals, SceneObject};
use phiber::space::BSpaceType;
use phiber::state::{BState, CtxKey, ModalKey};
use std::borrow::Cow;
use std::sync::Arc;

/// The no-op text system with ink: every glyph rasterizes to a box as wide
/// as half its em and as tall as seven tenths, sitting on the baseline.
struct BoxText(NoopTextSystem);

impl PlatformTextSystem for BoxText {
    fn add_fonts(&self, fonts: Vec<Cow<'static, [u8]>>) -> gpui::Result<()> {
        self.0.add_fonts(fonts)
    }
    fn all_font_names(&self) -> Vec<String> {
        self.0.all_font_names()
    }
    fn font_id(&self, descriptor: &Font) -> gpui::Result<FontId> {
        self.0.font_id(descriptor)
    }
    fn font_metrics(&self, font_id: FontId) -> FontMetrics {
        self.0.font_metrics(font_id)
    }
    fn typographic_bounds(&self, font_id: FontId, glyph_id: GlyphId) -> gpui::Result<Bounds<f32>> {
        self.0.typographic_bounds(font_id, glyph_id)
    }
    fn advance(&self, font_id: FontId, glyph_id: GlyphId) -> gpui::Result<Size<f32>> {
        self.0.advance(font_id, glyph_id)
    }
    fn glyph_for_char(&self, font_id: FontId, ch: char) -> Option<GlyphId> {
        self.0.glyph_for_char(font_id, ch)
    }
    fn glyph_raster_bounds(&self, params: &RenderGlyphParams) -> gpui::Result<Bounds<DevicePixels>> {
        let em = f32::from(params.font_size) * params.scale_factor;
        let (w, h) = ((em * 0.5).max(1.0) as i32, (em * 0.7).max(1.0) as i32);
        Ok(Bounds::new(point(DevicePixels(0), DevicePixels(-h)), size(DevicePixels(w), DevicePixels(h))))
    }
    fn rasterize_glyph(&self, _params: &RenderGlyphParams, bounds: Bounds<DevicePixels>) -> gpui::Result<(Size<DevicePixels>, Vec<u8>)> {
        Ok((bounds.size, vec![255; (bounds.size.width.0 * bounds.size.height.0).max(0) as usize]))
    }
    fn layout_line(&self, text: &str, font_size: Pixels, runs: &[FontRun]) -> LineLayout {
        self.0.layout_line(text, font_size, runs)
    }
    fn recommended_rendering_mode(&self, font_id: FontId, font_size: Pixels) -> TextRenderingMode {
        self.0.recommended_rendering_mode(font_id, font_size)
    }
}

fn socket(name: &str, limit: i32) -> BlendSocket {
    BlendSocket { identifier: name.into(), name: name.into(), limit, ..Default::default() }
}

fn node(name: &str, x: f32, inputs: &[&str], outputs: &[&str]) -> BlendNode {
    BlendNode {
        name: name.into(),
        x,
        y: 0.0,
        width: 160.0,
        inputs: inputs.iter().map(|i| socket(i, 1)).collect(),
        outputs: outputs.iter().map(|o| socket(o, 0xFFF)).collect(),
        ..Default::default()
    }
}

/// A cube with a three-node material, a keyed action, an NLA track with a
/// strip, and a marker: something for every editor to write.
fn scene() -> SceneModel {
    let mesh = SceneMesh {
        vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        indices: vec![0, 1, 2],
        normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
        corners: Default::default(),
    };
    let mut cube = SceneObject::identity("Cube", mesh);
    let graph = BlendNodeGraph {
        nodes: vec![
            node("Image Texture", -600.0, &["Vector"], &["Color", "Alpha"]),
            node("Principled BSDF", -300.0, &["Base Color", "Metallic", "Roughness"], &["BSDF"]),
            node("Material Output", 0.0, &["Surface", "Volume"], &[]),
        ],
        links: vec![BlendNodeLink { from: "Principled BSDF".into(), to: "Material Output".into(), from_socket: "BSDF".into(), to_socket: "Surface".into() }],
        ..Default::default()
    };
    cube.materials = vec![SceneMaterial { graph, ..SceneMaterial::neutral() }];
    let key = |frame: f32, value: f32| BKey::new([frame - 5.0, value], [frame, value], [frame + 5.0, value], 2);
    let action = SceneAction {
        name: "CubeAction".into(),
        frame_start: 1.0,
        frame_end: 120.0,
        fcurve_count: 2,
        keys: vec![1.0, 60.0, 120.0],
        users: vec!["Cube".into()],
        curves: vec![
            BFCurve::new("location", 0, vec![key(1.0, 0.0), key(60.0, 4.0), key(120.0, -2.0)]),
            BFCurve::new("location", 1, vec![key(1.0, 1.0), key(120.0, 3.0)]),
        ],
        ..Default::default()
    };
    let strip = BNlaStrip {
        name: "Walk Cycle".into(),
        start: 20.0,
        end: 140.0,
        actstart: 1.0,
        actend: 120.0,
        repeat: 1.0,
        scale: 1.0,
        extend: EXTEND_HOLD,
        action: Some("CubeAction".into()),
        ..Default::default()
    };
    let nla = BlendNla {
        object: "Cube".into(),
        action: Some("CubeAction".into()),
        extend: EXTEND_HOLD,
        tracks: vec![BNlaTrack { name: "Base Track".into(), flag: 0, strips: vec![strip], address: 0 }],
        ..Default::default()
    };
    let mut model = SceneModel { objects: vec![cube], actions: vec![action], nla: vec![nla], ..Default::default() };
    model.markers = vec![BlendMarker { frame: 90, name: "Hit".into() }];
    model.selection.selected = vec!["Cube".into()];
    model.selection.active = Some("Cube".into());
    model
}

/// One editor in one workspace: the area whose default editor is
/// `shown_in`, switched to `host`.
struct Case {
    space: BSpaceType,
    shown_in: BHostType,
    host: BHostType,
}

/// What one editor's frame painted inside its main region.
struct Painted {
    glyphs: usize,
    hidden: Vec<String>,
}

/// The playhead's stalk and its frame box are painted over the window on
/// purpose (`ED_time_scrub_draw_current_frame`): the box covers the ruler's
/// number under it, the stalk crosses a strip's name, in Blender as here.
/// Nothing else in a graph editor that narrow is opaque.
const PLAYHEAD_WIDEST: f32 = 48.0;

fn paint(cx: &mut gpui::VisualTestContext, app: &gpui::Entity<BPhiberApp>, model: &Arc<SceneModel>, case: &Case) -> Painted {
    let area = case.space.spec().areas.iter().find(|a| a.default_host == case.shown_in).copied().expect("the area");
    assert!(area.hosts.contains(&case.host), "{:?} is offered in the {:?} area of {}", case.host, case.shown_in, case.space.slug());
    app.update(cx, |app, cx| {
        app.scene_cache = Some(("graph-text".into(), Arc::clone(model)));
        app.state = BState::new();
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Space, case.space.slug());
        app.state.set(CtxKey::Object, "Cube");
        app.state.set(CtxKey::Frame, "30");
        app.state.set(&phiber::area::area_key(case.space, &area), case.host.slug());
        // Tall enough for the strip, a few channel rows and value lines.
        app.state.set_encoded(&phiber::zone::zone_size_key(case.space, phiber::zone::BZoneType::Bottom), "360");
        cx.notify();
    });
    cx.run_until_parked();
    // Again: the node canvas places its cards against the size it measured
    // on the frame before.
    app.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    let selector: &'static str = Box::leak(format!("region-Center-{:?}", case.host).into_boxed_str());
    let region = cx.debug_bounds(selector).unwrap_or_else(|| panic!("{selector} is painted"));
    let glyphs: Vec<_> = cx.painted_glyphs().into_iter().filter(|g| region.contains(&g.bounds.center())).collect();
    let hidden = glyphs
        .iter()
        .filter_map(|g| g.hidden_by.map(|by| (g.bounds, by)))
        .filter(|(_, by)| f32::from(by.size.width) > PLAYHEAD_WIDEST)
        .map(|(at, by)| format!("glyph at {:?} under {:?}", at.origin, by))
        .collect();
    Painted { glyphs: glyphs.len(), hidden }
}

/// The Timeline, Dope Sheet, Graph editor and NLA in the three workspaces
/// that show them, each with its ruler, channels, markers and strips.
#[test]
fn the_animation_editors_paint_text_nothing_covers() {
    let mut cx = TestAppContext::build_with_text_system(TestDispatcher::new(0), None, Arc::new(BoxText(NoopTextSystem)));
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(1000.0)));
    let model = Arc::new(scene());
    let cases = [
        Case { space: BSpaceType::Layout, shown_in: BHostType::Time, host: BHostType::Time },
        Case { space: BSpaceType::Layout, shown_in: BHostType::Time, host: BHostType::Ipo },
        Case { space: BSpaceType::Layout, shown_in: BHostType::Time, host: BHostType::Action },
        Case { space: BSpaceType::Layout, shown_in: BHostType::Time, host: BHostType::Nla },
        Case { space: BSpaceType::Rendering, shown_in: BHostType::Time, host: BHostType::Time },
        Case { space: BSpaceType::Rendering, shown_in: BHostType::Time, host: BHostType::Ipo },
        Case { space: BSpaceType::Animation, shown_in: BHostType::Action, host: BHostType::Action },
        Case { space: BSpaceType::Animation, shown_in: BHostType::Action, host: BHostType::Ipo },
        Case { space: BSpaceType::Animation, shown_in: BHostType::Action, host: BHostType::Nla },
    ];
    for case in &cases {
        let painted = paint(cx, &app, &model, case);
        let name = format!("{} {:?}", case.space.slug(), case.host);
        assert!(painted.glyphs > 0, "{name} paints its text");
        assert!(painted.hidden.is_empty(), "{name} paints text that is covered: {:#?}", painted.hidden);
    }
}

/// The node editors write every node's title and every socket's name, and
/// each one has to be on top of the card it is in — in the workspaces built
/// round them and switched into Layout's and Rendering's main area.
#[test]
fn the_node_editors_paint_their_socket_names_over_their_cards() {
    let mut cx = TestAppContext::build_with_text_system(TestDispatcher::new(0), None, Arc::new(BoxText(NoopTextSystem)));
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(1000.0)));
    let model = Arc::new(scene());
    // Every title and every socket name, one glyph a character.
    let graph = &model.objects[0].materials[0].graph;
    let written: usize = graph
        .nodes
        .iter()
        .map(|n| n.name.chars().count() + n.inputs.iter().chain(&n.outputs).map(|s| s.name.chars().count()).sum::<usize>())
        .sum();
    let cases = [
        Case { space: BSpaceType::Shading, shown_in: BHostType::Shader, host: BHostType::Shader },
        Case { space: BSpaceType::Compositing, shown_in: BHostType::Compositor, host: BHostType::Compositor },
        Case { space: BSpaceType::Layout, shown_in: BHostType::View3D, host: BHostType::Shader },
        Case { space: BSpaceType::Rendering, shown_in: BHostType::Image, host: BHostType::Compositor },
    ];
    for case in &cases {
        let painted = paint(cx, &app, &model, case);
        let name = format!("{} {:?}", case.space.slug(), case.host);
        assert!(painted.glyphs >= written, "{name} paints every title and socket name: {} glyphs of {written}", painted.glyphs);
        assert!(painted.hidden.is_empty(), "{name}: {} of {} glyphs are under a card: {:#?}", painted.hidden.len(), painted.glyphs, painted.hidden);
    }
}
