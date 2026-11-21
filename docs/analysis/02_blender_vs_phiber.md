# phiber vs Blender 5.2: what exists, what is partial, what is missing

## Summary
- phiber = GPUI desktop shell (`phikit/crates/phiber`, 1,026 .rs files, 229 test files, GPL-3.0) over **phiphx** (native .blend reader/writer/evaluator) and **phiray** (own path tracer/rasteriser). It is a *viewer, player and renderer with light object-level editing*, not a modeller.
- Strong: .blend read (mesh, materials, cameras, lights, world, render settings), ~20 modifiers, rigs/constraints/IK/NLA/drivers playback, F12 render, Properties/Outliner/Timeline/File browser, UI chrome for all Blender editors.
- Partial: Dope Sheet, Graph, NLA, Drivers, UV editor (select/move/unwrap), Spreadsheet, node editors (draw/edit graph, shader+per-pixel compositor evaluate), save (patch-in-place), import/export of static geometry.
- Missing: any mesh edit-mode tool (extrude, bevel, loop cut, knife, inset), sculpt, all painting, geometry-nodes evaluation (only "Smooth by Angle"), shape keys, particles/fluid/soft body, grease pencil, sequencer, text editor, Python/add-ons, real undo stack (state-step only), EEVEE (traced by phiray instead).
- Stage 1 (open/inspect/edit basics/render) is mostly done; the remaining stage-1 gaps are mesh edit mode, a real undo, full-file save, and Graph editing. Rough total to Blender parity: multi-year; stage 1 about 6-9 person-months, stage 2 about 2 person-years, stage 3 open-ended.
- Caveat: phiber's own status docs are known to drift (`docs/gap-list.md` found 3 of 3 spot-checked "open" lines already done; `docs/workflow.md` still says node editors draw a placeholder, but `src/view/draw/canvas/node/` has ~100 files of real graph code). Statuses below use code layout first, docs second. I read no function bodies deeply; items marked *not checked* were not verified.

## Method and sources read
Read: phiber `Cargo.toml`, `jobs.md`, `CONSISTENCY_PLAN.md`, `unity.md`, `docs/{space-status,gap-list,assessment,animation-rendering-status,blender-chapter-parity,blender-head-to-head,workflow}.md` (heads or key sections only), `docs/blend-read-parity.md` (head), `src/` module tree (all dirs listed), `src/scene/{README,edit/mod,save,import,export,sim/mod}.rs`, grep across `src` for tool names. `issues.md` was only listed, not read (not checked). phiphx: `README.md`, `jobs.md`, `src/blend/{README,write/mod,modifier/README,modifier/jobs,modifier/mod}`. Blender 5.2.0 tree: `source/blender/editors` dir list, `modifiers/intern/MOD_*.cc` list, `nodes/*/nodes` file counts, `scripts/startup/bl_ui/space_*.py` list, `io/` dirs. Specs `.kiro/specs/phiber-*`: task checkbox counts + requirements intro only.

Status vocabulary: done / partial / missing. "Shell" = lays out and draws chrome, no behaviour.

Scale for reference (Blender editors dir, .c/.cc files): sculpt_paint 126, interface 90, transform 82, space_outliner 66, space_view3d 49, mesh 36, object 30, space_node 26, space_file 27, grease_pencil 25, animation 25, space_sequencer 25, space_clip 19, armature 17.

## 1. Editors

| Area | Blender (paths) | phiber today (paths) | Status | Notes |
|---|---|---|---|---|
| 3D viewport | `editors/space_view3d` (49), `transform`, `gizmo_library`; `bl_ui/space_view3d*.py` | `src/host/view3d`, `src/part/view3d`, `src/view/draw/canvas/view3d/` (mesh, guide, rendered 39 files), `src/scene/{camera,orbit,navigate,select,boxsel,lasso,circlesel,grab,xform,transform,pick}` | partial | Orbit/pan/dolly/numpad, camera view, box/lasso/circle select, G/R/S object transform, gizmo compass, overlays (grid, outline, extras), X-ray. Camera-view zoom/passepartout open (`gap-list.md`). Selection outline has no AA / occlusion fade (`space-status.md`). |
| Outliner | `editors/space_outliner` (66), `bl_ui/space_outliner.py` | `src/host/outliner`, `src/part/outliner/` (11 + filter/mode/row_type/toggle), `src/scene/links` | done (common path) | Hierarchy, collections, filters, eye/render/select toggles, drag moves/links/parents saved to file. Expand state not restored from file (`gap-list.md`). Other display modes (Blender File, Data API, Library Overrides): not checked. |
| Properties | `editors/space_buttons` (4), `bl_ui/properties_*.py` | `src/host/buts`, `src/part/props/` (19 + physics, pref), `src/scene/render_set`, `output_set`, `render_edit` | partial | Render, Output, View Layer, Scene, World, Object, Modifier, Material, Physics (rigid body) tabs; values round-trip to file. Constraints/Data tabs are listed (`part/props/{constraint,data}.rs`); edit depth not checked. Properties tab not restored from file. |
| Shader node editor | `editors/space_node` (26), `nodes/shader` (95 node files) | `src/host/{shader,node}`, `src/view/draw/canvas/node/` (add/catalog, link, wire, socket, frame, select, paint, remove, drag, view), `phiphx/src/blend/shade/` | partial | Real graph draw/add/link/frame (older docs say placeholder; stale). Material semantic = Principled folded to tracer parameters; Mix/Add Shader as two lobes; most closure nodes folded; volumes fold to nothing; multiple scattering single only (`blender-chapter-parity.md`, `space-status.md`). Node graph is "not yet the file's own tree for every node type". |
| Geometry node editor | `nodes/geometry` (270 node files), `nodes/function` (55), `geometry/`, `editors/space_node` | `src/host/geonode` ("ECS Wand" in some docs), same node canvas; `phiphx/blend/modifier/nodes*.rs` | partial (UI) / missing (eval) | Only the bundled "Smooth by Angle" group is evaluated (Set Shade Smooth, Edge Angle, Boolean Math, Compare). `space-status.md`: "no geometry-node evaluation"; ~15 user files depend on it (`phiphx/jobs.md`). |
| Compositor | `nodes/composite` (87), `compositor/` | `src/host/compositor`, `phiray`/`phiphx/src/comp`, `scene/composite.rs` | partial | All per-pixel nodes + 19 Mix modes within 2e-4 of Blender; run over a render. Blur/filters, transforms, keyers, curve nodes declined by name and pass through. |
| UV editor | `editors/uvedit` (11), `space_image` | `src/host/uv`, `src/part/image/uv`, `src/scene/uv_edit.rs`, `phiphx/src/uv` | partial | Draw, pick, Shift-add, move; LSCM unwrap with pins, Cube/Cylinder/Sphere/View projection, island packing (batch 170). No box/island select, no ABF, no stitch/seam marking (not checked). |
| Image editor | `editors/space_image` (7), `render/` | `src/host/image`, `src/part/image/`, `src/scene/{image_gen,image_list,render_slot}.rs` | done (for render result) | Render Result, slots, passes, New/Open/Save As (PNG-8/16, EXR half/float, multilayer), render region. Image painting: missing. ~50 inert menu entries (old count, `animation-rendering-status.md`). |
| Timeline | `editors/space_action` + `animation` | `src/host/time`, `src/part/time`, `src/view/draw/canvas/time/` | done | Transport, range, markers, keying, zoom/scroll. Inert: Show Seconds, Set Preview Range, bind camera to markers (older note). |
| Dope Sheet | `editors/space_action` (7) | `src/host/action`, `src/scene/action/dope/` (11) | partial | Pick, box select, move, delete, duplicate, snap, mirror, copy. Channel rows per F-curve still "blocked" in `space-status.md` (batch 170); may have moved since (not checked). Keying sets missing. |
| Graph editor | `editors/space_graph` (10) | `src/host/ipo`, `src/view/draw/canvas/time/curve/{handle,line}` | partial | Curves + Bezier handles draw; **no editing** (select/move/handle drag). |
| Drivers editor | `space_graph` drivers mode, `animrig/` | `src/host/drivers`, `src/view/draw/canvas/time/driver`, `phiphx/.../modifier/driver*.rs` | partial | Object and rig drivers evaluated incl. all variable kinds except Context Property (batch 171). Material, shape-key, modifier-target drivers listed but not all run. |
| NLA | `editors/space_nla` (7) | `src/host/nla`, `src/scene/action/{nla,nla_editor}.rs`, `phiphx/.../anim/nla_eval.rs` | partial | Tracks/strips draw, evaluate (all blend modes), create tracks/strips. No tweak mode, no strip dragging. |
| Sequencer (VSE) | `editors/space_sequencer` (25), `sequencer/` | `src/host/seq`, `src/part/seq`, `src/part/strip` | missing (shell) | Lays out only (`space-status.md`). |
| Text editor | `editors/space_text` (12) | `src/host/text`, `src/part/text` | missing (shell) | Layout only; no text editing. |
| Python console / Info | `space_console`, `space_info` | `src/host/{console,info}` | shell / partial | Info shows report log; console is not Python (`space-status.md`: "no Python"). |
| Spreadsheet | `editors/space_spreadsheet` (12) | `src/host/spreadsheet`, `src/part/spreadsheet/` | partial | Vertex/Face/Face-Corner columns (position, UVMap, material_index, colour attr). No Edge domain, filters, pinning; meshes only (no curves, instances, geometry-nodes eval domains). |
| File browser | `editors/space_file` (27) | `src/host/file`, `src/part/file/` (20), `src/modal/file_browser` | done | Open, Save As, Revert, recents, filters, bookmarks (not checked). Asset browser: `src/host/asset`, `src/part/assetshelf` (not checked). |
| Preferences | `editors/space_userpref` (4), `bl_ui/space_userpref.py` | `src/host/pref`, `src/part/pref/` (6 + control), `src/part/props/pref` | partial | Themes, fonts (typography), GPU adapter, many sections; `jobs.md`: "Fill the missing Preferences sections against Blender's". Keymap prefs, add-ons: not checked (no add-ons exist). |
| Movie Clip / Masks | `space_clip` (19), `mask` (9) | `src/host/clip`, `src/part/clip` | missing (shell, "n/a") | Not started. |
| Logic / other hosts | n/a | `src/host/{logic,activity,play,project}` | phiber-specific | `play` runs a `phiplay` game; `project` is an asset DB browser. Not Blender features. |
| Top bar / status bar / menus | `space_topbar`, `space_statusbar`, `bl_ui/space_topbar.py` | `src/host/{topbar,statusbar,header}`, `src/part/menubar/` | partial | Menus fully laid out against Blender's, but ~380-390 entries are inert placeholders (`CONSISTENCY_PLAN.md` A1; `animation-rendering-status.md`). |

## 2. Workspaces, layout, UI system

| Area | Blender | phiber today | Status | Notes |
|---|---|---|---|---|
| Workspace tabs | `editors/screen` (14), `startup.blend` | `src/space/` : layout, modeling, sculpting, uv_editing, texture_paint, shading, animation, rendering, compositing, geometry_nodes, scripting, simulation | partial | Layout/Animation/Rendering/UV: work. Shading/Compositing/Geometry Nodes: partial. Texture Paint, Scripting: shells. Modeling/Sculpting exist as specs but are not offered as tabs (`space-status.md`). Simulation tab: not checked. |
| Area/zone/region split, maximize, resize | `screen/` | `src/area/{current,maximize,size,switch}`, `src/zone`, `src/region`, `src/pane`, `src/part` | done (own design) | Hierarchy Space > Zone > Area > Host > Region > Pane > Part. Ctrl+Space maximise, area sizes. Join/split areas by drag, area swap: not checked. Saved screen layout from .blend not restored (`gap-list.md` #28). |
| Theme and UI style | `interface/` (90), `datafiles/userdef` | `src/theme/` (descriptor, palette, semantic, style), `src/view/draw/flex`, `docs/bui-*.md` | done (own system) | Token layer (`BFlex`), resolved UI style, `unity.md` audit (two hover colours etc. - may be fixed since). 535 raw style calls left to migrate. |
| Keymaps/operators | `windowmanager/`, `bl_keymap` | `src/app/{keymap,op}`, `src/state/key` | partial | Operator layer exists (`app/op`, 11 files); keymap editing not checked. No operator search / redo panel (not checked). |
| Modals | `wm` popups | `src/modal/` : about, command (find), file_close, help, new_image, profile, reader, rename, render, revert, setting, splash, workspace | done | Command palette with find; manual reader; splash. |
| Browser build | n/a | `src/wasm.rs`, `web/`, `assets/` embedded via `include_dir` | extra | wasm32 target exists. |

## 3. Viewport shading and rendering

| Area | Blender | phiber today | Status | Notes |
|---|---|---|---|---|
| Wireframe / Solid | `draw/engines/{workbench,overlay}` | `view/draw/canvas/view3d/mesh/{raster,studio}`, `scene/{shading,solid_fx,solid_shade,solid_dof,matcap}.rs` | done | Workbench ported on GPU: studio/flat light, colour modes, outline, cavity, matcap, X-ray, DoF. Loose edges for wireframe open (192-vertex parity gap, `gap-list.md`). |
| Material Preview (EEVEE look) | `draw/engines/eevee`, `eevee_next` | same canvas via phiray (deterministic, studio light) | partial | Not EEVEE: phiray traces it. "EEVEE is not implemented; a file naming it is traced" (`blender-chapter-parity.md`). GPU renderer 2 (appkit wgpu) listed open in `view/jobs.md`. |
| Rendered (Cycles look) | `intern/cycles`, `render/` | `view/draw/canvas/view3d/rendered/` (39 files), `phiray` | partial | Progressive, denoised path tracing; Principled whole; adaptive sampling/noise threshold read but not applied; Diffuse bounces only; no motion blur, volumes, baking, OSL, light linking, Freestyle; passes Combined/Depth/Normal/Diffuse Color only. Within 1% (white override) / 3% (metal) of Cycles in measured tests (`assessment.md`). |
| Render output | `render/`, `imbuf/`, `io/` | `scene/{render_job,render_save,render_slot,video,mixdown}`, `scene/video/{avi}` | partial | F12/Ctrl+F12, worker with cancel, region, stills PNG/EXR, movies via system `ffmpeg` else MJPEG AVI, audio mixdown FLAC/WAV. No metadata stamp, no audio in movies, no frame step/overwrite options. |
| Cameras | `blenkernel/camera.cc` | `scene/{camera,camera_view,startup_camera}.rs` | done | Active camera, sensor fit, ortho, DoF; IoU 0.9962 on measured pose (`blender-chapter-parity.md`). Stereo, camera markers binding: not checked. |
| Lights / world | `DNA_light_types.h`, `DNA_world_types.h` | `scene/scene_light.rs`, `phiphx/blend/object`, phiray env | done / partial | Sun, point, spot, area; world colour/env image (importance sampled), Sky baked within 1.6% of Cycles. Sky Preetham/Hosek declined; light linking missing; light-probe volumes and IES: not checked. |
| Colour management | `ocio`, `imbuf/colormanagement` | `scene/render_color.rs` | partial | Standard, Filmic, AgX, exposure, gamma. Look not read. |

## 4. Modes and mesh editing

| Area | Blender | phiber today | Status | Notes |
|---|---|---|---|---|
| Object mode ops | `editors/object` (30), `transform` (82) | `scene/edit/` (add, delete, duplicate, rename, hide), `scene/{xform,grab,transform,parent}.rs`, 13 primitives (`edit/shape.rs`) | partial | Add primitives (Plane, Cube, Circle, UV/Ico Sphere, Cylinder, Cone, Torus, Grid, Monkey, Metaball, Text, Gem), delete, duplicate (linked copy), parent, transform with G/R/S. Join, apply transform, convert, origin set, snap, pivot, proportional edit: not checked. |
| Edit mode switch | `editors/mesh` (36), `bmesh/` | mode bar in `src/part/modebar/`; "Edit Mode" string used for UV gating only (`part/image/uv/mod.rs`) | missing | No BMesh equivalent. Mode dropdown is UI; edit mode only enables UV-corner picking/moving. Greps for extrude/loop cut/knife in `src` found nothing implemented; "bevel" only as modifier names. |
| Mesh edit tools (extrude, bevel, loop cut, knife, inset, merge, bridge, subdivide, dissolve, etc.) | `editors/mesh` (36 files), `bmesh/operators` | none | missing | Largest functional gap for modelling. `BlendNewMesh` writes primitive meshes into a file but there is no topology-editing data structure. |
| Vertex/edge/face select, select ops | `mesh/editmesh_select*.cc` | none (object-level select only; UV corner select) | missing | |
| Sculpt mode | `sculpt_paint` (126) | none; Sculpting space spec only | missing | Brush system, dyntopo, multires sculpt, masks. Multires is *read* (`modifier/multires*.rs`), not edited. |
| Texture paint | `sculpt_paint/paint_image*.cc` | `space/texture_paint` shell | missing | "no painting - brushes, strokes, texture slots" (`space-status.md`). |
| Weight paint / vertex paint | `sculpt_paint/paint_vertex*.cc` | only sidebar mode labels (`part/sidebar/editor/view3d.rs`) | missing | Vertex groups are *read* (`phiphx/blend/mesh`). |
| Pose mode / armature edit | `editors/armature` (17) | pose is evaluated, not edited (`scene/pose`, `phiphx/blend/rig`) | partial (view) / missing (edit) | Rigs animate; no bone edit, no pose-mode manipulation (keying exists via Insert Keyframe on objects). Rigify pose feet 13-16 cm low on some files (`phiphx/jobs.md`). |
| Curves / surfaces / metaballs / lattices editing | `editors/curve` (10), `curves` (15), `metaball`, `lattice` | Lattice and Curve *modifiers* evaluate; metaball polygonised on add | missing (edit) | |

## 5. Modifiers (Blender 5.2 `MOD_*.cc`: ~100 files incl. 28 grease-pencil ones)

Implemented in phiphx (`blend/modifier/mod.rs` registry + files): Armature (with B-Bone, `bbone.rs`), Array (merge; caps/fit modes open), Displace (no texture), Mirror (no creases carried), Solidify (extrude + creases; complex mode open), Subdivision Surface (Catmull-Clark/Simple, creases, vertex order differs from Blender; UV smoothing open), Triangulate, Weld / Merge verts, Lattice, Corrective Smooth, Mask, Shrinkwrap (project, snap, target), Curve, Multires (read/evaluate), Simple Deform, Surface Deform, Cloth (cached/disk, Cloth "at the frame" has 12 files open), Skin, Geometry Nodes (only Smooth by Angle + legacy Auto Smooth conversion), plus Subsurf creases. Skipped modifiers are reported per object (batch 194).

| Area | Blender | phiber/phiphx | Status | Notes |
|---|---|---|---|---|
| Listed above (~20) | `modifiers/intern` | `phiphx/src/blend/modifier/` | done/partial | Silhouettes match Blender at IoU >= 0.999 (`space-status.md`). Armature envelopes and dual quaternions open. |
| Bevel | `MOD_bevel.cc` | UI label only (`part/props/modifier.rs`) | missing | "Bevel (display)" in `modifier/jobs.md`. |
| Boolean, Decimate, Remesh, Screw, Wireframe, Edge Split, Build, Cast, Explode, Hook, Smooth, Laplacian x2, Mesh Deform, Normal Edit, Weighted Normal, UV Project, UV Warp, Wave, Warp, Ocean, Data Transfer, Mesh Cache, Mesh Sequence Cache, Shape Key, Vertex Weight x3, Volume x3, Line Art | `MOD_*.cc` | none found in `modifier/` | missing | Not in the registry (checked file listing, not every `KINDS` row). |
| Physics modifiers: Collision, Soft Body, Fluid, Particle System/Instance, Dynamic Paint, Explode | `MOD_collision/softbody/fluid/particle*/dynamicpaint` | none (cloth only) | missing | |
| Grease Pencil modifiers (28) | `MOD_grease_pencil_*` | none | missing | |
| Modifier editing UI | `properties_data_modifier.py` | `part/props/modifier.rs`, `time/row/name/modifier.rs` | partial | Modifier tab lists/reads values; edit round-trip depth not checked. Adding a modifier from UI: not checked. |

## 6. Geometry nodes, shader nodes, compositor nodes (counts of node files in Blender)

| Area | Blender | phiber/phiphx | Status | Notes |
|---|---|---|---|---|
| Geometry nodes (270 geo + 55 function node files) | `nodes/geometry`, `nodes/function`, `geometry/`, `blenkernel/geometry_*`, field/lazy-function evaluators | `phiphx/.../modifier/nodes*.rs` | missing (1 group) | Needs a node-graph evaluator, field system, instances/curves/volumes/point-cloud geometry set. Very large (Blender: tens of thousands of lines). |
| Shader nodes (95) | `nodes/shader` | `phiphx/blend/shade`, phiray kernels | partial | Textures: Noise, Voronoi, Checker, Brick, Gradient, Magic, Wave, White Noise, Gabor, Sky; Math (41 ops), Mapping, Bump, Color Ramp, Principled. Image colour space/UDIM, per-socket textures, volumes, many closures (SSS, hair, toon) folded or declined. |
| Texture nodes (23) | `nodes/texture` | not checked | not checked | |
| Compositor nodes (87) | `nodes/composite` | `phiphx/src/comp` | partial | Per-pixel only (see §1). |

## 7. Animation and rigging

| Area | Blender | phiber/phiphx | Status | Notes |
|---|---|---|---|---|
| Keyframes / F-curves | `animrig/`, `blenkernel/fcurve*.cc` | `phiphx/blend/anim/`, `scene/action/{keying,autokey,keys,clean,reshape,retime}.rs` | partial | Layered (4.4+) and legacy actions; constant, linear, Bezier, 10 easings; auto/auto-clamped handles; extrapolation; Insert Keyframe (I), auto-key, Alt+I; written back to the file. Keying sets missing. F-curve modifiers: only Generator (Cycles, Noise, Limits, Envelope, Stepped missing). |
| Graph/Dope editing | `space_graph`, `space_action` | see §1 | partial | |
| Drivers | `animrig/`, `blenkernel/anim_sys.cc` | `modifier/driver*.rs`, `scene/..`, `docs/driver-parity.md` | partial | Objects, bones, constraint influence, modifier settings (batch 188, subset); drivers on materials, shape keys, custom props partial. |
| Armatures / rigging | `armature/`, `blenkernel/armature*.cc` | `phiphx/blend/rig/`, `scene/pose/`, `scene/skin.rs` | partial | Pose evaluation matches Blender on THYMUS (328 bones, 6 frames); GPU skinning; background pose thread. Edit bones/weights: missing. Spline IK, Shrinkwrap constraint: not evaluated. |
| Constraints | `blenkernel/constraint.cc` | `phiber/src/part/props/constraint.rs`, `scene/xform/constraint.rs`, `phiphx/blend/rig` | partial | Child Of, Track To, Damped Track, Locked Track, Limits, Copies, Stretch To, Transformation, Armature, Action, Pivot, Maintain Volume, Floor, Follow Path, Clamp To. Spline IK, Shrinkwrap, a few others missing. Constraint editing UI: not checked. |
| IK | `ik_solver`, `intern/iksolver` | `phiphx/blend/rig` ("legacy IK ported step for step") | done (legacy IK) | |
| Shape keys | `DNA_key_types.h`, `blenkernel/key.cc` | UI labels only (`part/props/data.rs`) | missing | Shape-key animation not evaluated (`animation-rendering-status.md`). |
| NLA | see §1 | | partial | |
| Per-frame geometry | depsgraph | `scene/pose`, `docs/gap-list.md` Tier 0 | partial | Rigged meshes re-pose per frame. Driven modifier settings and non-rigged objects are not re-evaluated per frame (ownership issue; batch 192 added `clone_box` toward it - check status). |
| Vertex parenting etc. | `object/` | missing: PARBONE/PARSKEL/PARVERT, dupli/instances (`gap-list.md`) | partial | Vertex parenting affects ~50 of the user's files (`phiphx/jobs.md`). |

## 8. Physics

| Area | Blender (`editors/physics`, `blenkernel/*`) | phiber today | Status | Notes |
|---|---|---|---|---|
| Rigid body | `rigidbody.cc`, Bullet | `scene/sim/` (bake, input, input_body) over `phiray::sim` XPBD | partial | World baked on a thread over the cache range, scrub reads the shelf, bake pin/delete. Kinematic/animated bodies, joints, edited bodies not handled (`sim/mod.rs`). |
| Cloth | `cloth.cc` | `phiphx/.../modifier/cloth*.rs` (disk cache) | partial | Read cached results; "Cloth at the frame" has 12 open files; no cloth solver (not checked beyond file names). |
| Soft body | `softbody.cc` | none | missing | |
| Fluid (Mantaflow) | `fluid.cc`, `intern/mantaflow` | none | missing | |
| Particles / hair | `particle*.cc`, `curves/` | none | missing | UI strings only. |
| Force fields / collision / dynamic paint | | none | missing | |

## 9. Materials, textures, lights

Covered in §3 and §6. Material slots/index, viewport colour, UV maps and images (packed and external) read. Image textures: colour space and UDIM tiles open; per-socket texture selection open (`gap-list.md`). Geometry/vertex colour attributes: Colour Attribute on face corner read; 5.x `custom_normal` open.

## 10. .blend IO, versioning, import/export, undo, scripting

| Area | Blender | phiber/phiphx | Status | Notes |
|---|---|---|---|---|
| .blend read | `blenloader/`, `makesdna/` (SDNA), `blenkernel` versioning | `phiphx/src/blend/file/` (header, block table, DNA, pointer scope), per-type readers | done (scene subset) | Reads legacy 12-byte and 17-byte (v1, 5.x) headers, 4.x CustomData and 5.x attribute storage, compressed files (not fully checked). Matches Blender mesh digests across 1,276 meshes in 8 files; open times 2.3-3.6x faster than Blender on large rigs (`blender-head-to-head.md`). Only data phiphx understands is surfaced; many ID types (curves, grease pencil, volume, point cloud, sound, text, brushes, palettes, linked libraries): not checked / likely ignored. No `do_versions` port: files are read by DNA name matching, so old-file semantics rely on per-reader fallbacks. |
| .blend write | `blenloader/intern/writefile.cc` | `phiphx/src/blend/write/` : patch-in-place of the opened file (`BlendFile::to_bytes`/`write_to`), new blocks for markers, new mesh/object, actions, NLA, render/output settings, collections | partial | Not a general writer: "rather than a second writer of every struct". Cannot create a .blend from scratch (default scene/new file "has nothing to write through"); cannot write new meshes into files older than Blender 3.6. `.blend1` backups. Thumbnails, packed images, library paths: not checked. |
| Import | `io/{wavefront_obj,ply,stl,usd,alembic,fbx,csv,grease_pencil}`; glTF is an add-on | `scene/import.rs`, `phiphx/src/import` | partial | OBJ, USD/USDA/USDC, glTF/GLB (behind `gltf-import` feature), PLY (phiphx reader). Geometry only into the scene; STL import: not checked. FBX, Alembic: missing. |
| Export | same | `scene/export.rs`, `phiphx/src/export` | partial | OBJ, PLY, STL, USDA, glTF 2.0 (embedded buffer): geometry only, still frame, **no materials or animation**. USDC/FBX/Alembic: missing. |
| Undo/redo | `ed_undo.cc`, `editors/undo` (3), memfile + per-mode undo | `AppKey::EditUndo` is text-field cancel (`state/edit.rs`); `scene/edit` doc: "Undo is a state step"; `state/` is a key-value store | partial | Undo exists as snapshot steps of the key-value store (edits are stored as keys applied over the loaded model), so undo is cheap but only covers things expressible as state keys. No undo history panel / step list: not checked. Will not scale to mesh edits. |
| Python API / add-ons | `python/`, `bpy`, `scripts/addons_core` | none; Python Console host is chrome only; `space/scripting` is a shell | missing | "not planned" in `space-status.md`. The project's own extension model is Rust (appkit). |
| Grease Pencil | `editors/grease_pencil` (25), `gpencil_legacy` (7), `MOD_grease_pencil_*` | filters/menu labels only | missing | |
| Asset browser/libraries | `editors/asset` (22), `blenkernel/asset*` | `src/host/asset`, `part/assetshelf`, `phiphx/blend/asset/` | partial | Asset import pipeline and `.phx` sidecar; Blender asset libraries/catalog: not checked. |
| Preferences/keymap persistence | `userdef` | `app/session` (durable/throttled/volatile), `app/local_keys.rs` | done (own design) | Session resume, collab sync (`modal/collab`). |

## 11. Prioritised missing list

Sizes: S <= 2 person-weeks, M = 1-2 months, L = 3-6 months, XL = 6+ months, one engineer each, rough.

### Stage 1 - "open a typical .blend, inspect, edit basic things, render"
Most of the viewing path is done. What still blocks the goal:
1. **Per-frame evaluation completeness** (Tier 0, `gap-list.md`): driven modifier settings, non-rigged object re-evaluation, vertex parenting, collection instances/dupli. M. Many user files show wrong frames today.
2. **Edit-mode foundation**: an editable mesh structure (half-edge/BMesh-like), vertex/edge/face select, G/R/S on elements, delete, merge, undo for it, write back through `write/mesh.rs`. L.
3. **Core mesh tools**: extrude, inset, bevel, loop cut and slide, knife, subdivide, dissolve, bridge, fill. L (after 2).
4. **Real undo/redo** that covers mesh and animation data (command or snapshot-diff), plus history list. M.
5. **Graph editor editing** and Dope Sheet per-F-curve channel rows (currently "blocked" on another owner); keying sets. M.
6. **Save**: create a .blend without a source file, write arbitrary new datablocks (meshes with attributes, materials, modifiers added in UI), versions < 3.6. M-L.
7. **Properties editing breadth**: add/remove/reorder modifiers and constraints, material slots, object data tabs, custom properties. M.
8. **Inert-menu sweep**: ~380 menu entries that do nothing (`CONSISTENCY_PLAN.md` A1). M (ongoing).
9. **Render completeness**: remaining passes, adaptive sampling, bounces/clamping, light linking, metadata stamp, motion blur. M.
10. **Shading space finish**: node graph = file's own tree for every node type; image colour space/UDIM; remaining closures. M.
11. Texture node tree support (23 node files), `custom_normal`, colour attributes in render. S-M. (not checked what already works)

### Stage 2 - "real modelling and animation work"
- Geometry nodes evaluator (field system, instances, curves) plus the common nodes first (Mesh Primitives, Transform, Set Position, Join, Instance on Points, Distribute Points, Realize). XL. Opens ~15 user files immediately via Smooth by Angle (done) and more later.
- Armature edit/pose mode: bone edit, weight paint, auto-weights, Spline IK, Shrinkwrap constraint, B-Bone completeness, dual-quaternion/envelope skinning. L.
- Shape keys (read, evaluate, drive, animate, edit) and F-curve modifiers (Cycles, Noise, Limits, Envelope, Stepped). M.
- Missing modifiers by user value: Bevel, Boolean, Decimate, Remesh, Screw, Wireframe, Edge Split, Smooth, Weighted Normal, Data Transfer, Vertex Weight x3, Hook, Mesh Deform, Wave, Normal Edit, UV Project/Warp. Each S-M; total L.
- Sculpt mode (brush engine, masks, dyntopo, multires) L-XL; vertex paint/weight paint L; texture paint L.
- Compositor neighbourhood/transform/keyer nodes (Blur, Glare, Defocus...) L; Movie Clip basic. M.
- NLA tweak mode, Drivers on all datablock types, UV box/island select, seam marking, UV stitch. M.
- Mesh edit: spin, bevel weights, edge slide, proportional editing, snapping, mirror editing. L.
- Import/export: materials and animation in glTF/USD export, FBX import/export, Alembic. L.
- Lights/world: light linking, shadow catcher, EEVEE-look viewport (decide: GPU renderer or keep traced). L-XL.

### Stage 3 - "Blender parity extras"
- Particles/hair and curves editing; fluid, soft body, force fields, dynamic paint. XL each.
- Grease Pencil (v3) incl. modifiers, drawing tools. XL.
- Video Sequencer, Movie Clip tracking/masking, Text editor. L each.
- Python/`bpy` or a compatible scripting layer and add-on system. XL (design choice: embed Python vs Rust extension API).
- Eevee-class real-time renderer, Cycles features (volumes, OSL, baking, motion blur). XL.
- Full `do_versions` fidelity for very old files, linked libraries and overrides, asset libraries with catalogs. L-XL.

## 12. Items not checked
`issues.md` (36 KB), phiber `tests/` contents, `docs/ui-parity-status.md` (395 KB) and `docs/rendering.md` (244 KB), individual `src/**/jobs.md`, the Simulation tab's content, texture nodes, Properties tab edit depth, asset libraries, STL import, compressed .blend support, exact modifier `KINDS` rows, sequencer/text beyond directory layout, the other phiber specs' tasks beyond checkbox counts. Spec status: `phiber-blender-space-parity` 131/131 tasks done; `phiber-viewport-layout-fixes` 12/12; `phiber-resolved-ui-style-settings` 38 done / 7 open; `phiber-host-visual-fixes` 0/15 recorded (status likely stale, files/hover already exist in the tree); `phiber-ui-polish-parity-closeout` has design only, no tasks.md.
