# Phiber — Issues: physics, animation, rendering

What stands between phiber and a performant, Blender-parity feature set in
**physics**, **animation actions**, and **rendering**. Read against the working
tree on 2026-10-29 (phiber / phiphx / phiray under `phikit/crates`). Every claim
names the file it comes from. This file supersedes the physics section of
`phiray/docs/ROADMAP.md` (dated 2026-10-26), which predates `phiray/src/sim/`.

Legend for each item:

- **[WORKS]** implemented and driving the scene / producing pixels
- **[WIRE]** the capability exists in a lower crate but phiber does not call it
- **[STUB]** UI/menu/panel present, no behaviour behind it
- **[MISSING]** not present anywhere

---

## 0. Triage — the one thing to check first

**[WORKS/verify] GPU adapter.** Rendering is a real GPU path tracer, not a
placeholder, so if the viewport looks dead the likeliest cause is the shared
GPU adapter failing and the mesh-painter fallback kicking in. Check the session
journal for `no GPU adapter` or `the GPU declined this scene` before chasing any
single feature (`view/draw/canvas/view3d/rendered/stepwise.rs`:
`phiray::Gpu::shared()` guard; `scene/render_job.rs` `note_failure`). This may
account for most of the "nothing works" impression on its own.

---

## 1. Physics

### 1.1 The headline gap

**[WIRE] The rigid-body solver exists but is not connected to phiber.**
`phiray/src/sim/` is a complete CPU XPBD rigid-body solver — substeps, broadphase
over phiray's BVH, narrowphase (closed forms, SAT + clipping, GJK/EPA), contacts
with static/dynamic friction and restitution, Blender-style deactivation/sleep.
It has passing physical tests in `phiray/tests/sim.rs` (free-fall matches the
closed form, a ball bounces to e² of its height, a box slides a slope by the
friction law). Its public API is phiber-facing by design:

- `phiray::sim::World::from_blend(&BlendWorld, &[BlendObject]) -> World`
  (`sim/blend.rs`)
- `World::step(dt)` (`sim/world.rs`)
- `World::poses() -> impl Iterator<(usize /*obj*/, [[f32;4];4] /*world*/)>`

A grep for `phiray::sim` / `World::step` across `phiber/src` returns **nothing**.
So pressing Play on a rigid-body scene moves nothing, though every piece needed
is present. The settings are already read into the model
(`phiphx/blend/physics/` → `SceneModel.physics: BlendPhysics`), and the
per-frame "moved world" seam animation uses (`scene/motion.rs` `set_moved`) is
exactly what `poses()` feeds.

**What's missing:** one integration module, `scene/sim/`, that
1. translates `SceneModel.physics` (`BlendPhysics`) into phiray's `BlendWorld` /
   `BlendObject`,
2. bakes a deterministic point cache over `startframe..endframe` (the solver is
   stateful; scrubbing requires a cache, as Blender's `PointCache` is), and
3. feeds the baked frame's matrices into `motion::set_moved`.

Two correctness traps found while reading `sim/blend.rs`:
- `from_blend` bakes the object **scale** into the shape itself (`mul3(v, s)`)
  and uses the matrix only for position/rotation — pass **object-space**
  triangles and the raw `matrix_world`, never world-space verts, or scale is
  applied twice.
- `BlendObject.tris` is a borrow; the triangle buffers must outlive the
  `from_blend` call.

### 1.2 Rigid-body feature gaps (once wired)

- **[MISSING] Joints / rigid-body constraints solve.** Settings are read
  (`phiphx/blend/physics/joint.rs`; panel `part/props/physics/joint.rs`) and the
  XPBD mapping for each Blender constraint type is designed (ROADMAP §2 "Joints"
  table), but `phiray/src/sim/` has no joint projection. Fixed/Point/Hinge/
  Slider/Piston/Generic/Spring/Motor all need adding.
- **[MISSING] Kinematic / animated bodies driven by the depsgraph.**
  `from_blend` maps `kinematic || !enabled` to `Kind::Kinematic` and the solver
  has `World::drive(i, x, q)` to move them, but nothing feeds the animated world
  per substep yet. Until then, animated passive bodies won't carry dynamic ones.
- **[DIVERGENCE] Mesh active bodies become their convex hull.** A concave active
  body is simulated as its hull and listed in `World::hull_fallback`; Blender
  allows true mesh–mesh (Bullet GImpact). Report it like dropped modifiers.
- **[DIVERGENCE] Not Bullet.** XPBD will not byte-match Blender's bake. Defaults
  already match Blender (10 substeps / 10 iterations in `sim/world.rs` and
  `sim/blend.rs`); quality is tuned by substeps, not expected to replicate.

### 1.3 Cloth / soft body / particles / fields

- **[MISSING] Cloth solve.** Settings read (`phiphx/blend/physics/cloth.rs`,
  panel exists); display path reads the point cache
  (`phiphx/blend/modifier/cloth.rs`) rather than stepping. The XPBD distance /
  bend / goal / pressure design is in ROADMAP §2 "Cloth"; no solver code yet.
- **[MISSING] Soft body solve.** Settings and panel exist
  (`part/props/physics/soft.rs`), no solver.
- **[MISSING] Force fields / effectors.** `BForceField` / `BCollider` read
  (`phiphx/blend/physics/deflect.rs`); not consumed by any solver. Effector
  weights are read on the world (`physics/world.rs` `BEffectorWeights`) and only
  the gravity weight has an obvious consumer (gravity scaling in the bridge).
- **[MISSING] Particles, smoke, fluid, dynamic paint.** Modifier *names* are
  known for the "dropped modifiers" report (`phiphx/blend/modifier/report.rs`);
  no settings read, no sim. Menu entries are placeholders
  (`part/menubar/editor/anim/set.rs`: Particles, Cloth, Smoke, Dynamic Paint are
  `op(...)` with no handler).

### 1.4 Performance

- **[MISSING] GPU solver (ROADMAP M3).** `World::step` is CPU today. Every state
  array is a `Pod` twin of a `GpuArray` by design, so the port is mechanical but
  unwritten. Needed only at scene scale (thousands of bodies); hundreds bake
  fast enough on the CPU.
- **[PERF] Bake must run off the UI thread** for large scenes, mirroring
  `scene/pose/worker.rs` (`BPoser`), and be cached keyed on a hash of
  `model.physics` + fps so a settings edit re-bakes and a scrub does not.

---

## 2. Animation actions

The evaluation + playback core is real; the gaps are in the **editor
operators**. In the dope-sheet / graph / NLA menus the pattern is: `fire(...)`
is wired to an `OpKey` handler, `op(...)` is a menu entry with **no behaviour**
(`part/menubar/editor/anim/`).

### 2.1 What works

- **[WORKS] Playback drives the scene.** `scene/motion.rs` re-poses objects,
  lights, cameras and the active render camera at the shown frame; rigs deform
  on a worker (`scene/pose/`); the ticker keeps wall-clock time and drops frames
  like Blender (`app/playback.rs`).
- **[WORKS] Evaluation stack:** fcurves (Bézier + easing), channels, pose/IK,
  constraints, drivers, NLA eval (`phiphx/blend/anim/`), truth-tested against
  `tests/truth/thymus_anim.txt`.
- **[WORKS] Keyframe ops that are wired:** select all/none/invert, insert, jump,
  duplicate, delete, clean, snap (to current / nearest frame / second / marker),
  mirror (over current frame / marker), auto-key on G/R/S
  (`scene/action/`, `scene/action/autokey.rs`).
- **[WORKS] NLA ops that are wired:** select all/none/invert, add track / above /
  delete, frame all / scene range, push-down / stash action, delete strip
  (`anim/nla.rs`, `anim/set_more.rs` ACTION).

### 2.2 What's stubbed (menu present, no handler)

Dope sheet / graph (`anim/set_more.rs`, `anim/graph.rs`):
- **[STUB] Copy / Paste / Paste Flipped keyframes** (`KEY`: `op("Copy…")`,
  `op("Paste…")`).
- **[STUB] Box / Circle / Lasso select, More/Less, Select Linked, column
  selects** (`SELECT`: all `op(...)`).
- **[STUB] Channel ops:** Delete / Group / Ungroup / Toggle editability /
  Extrapolation / Expand / Collapse / Revive F-Curves / Bake Channels / Frame
  Selected (`CHANNEL`: only Clean Channels is `fire`).
- **[CORRECTED — was wrong above]** Handle Type, Interpolation Mode and
  Easing Mode are **wired**, not stubs: `HANDLE_TYPE`, `INTERPOLATION` and
  `EASING` in `anim/set.rs` are all `fire(... OpKey::Keys, "handle:…" /
  "ipo:…" / "easing:…")` against the working arms in
  `dope/set_edit.rs`. The earlier note here over-generalised from
  `KEYFRAME_TYPE` being `op(...)`. Keyframe Type was the one real stub in
  that group, and it is now done (Part VI).
- **[STUB] Bake Keyframes, Discontinuity (Euler) Filter, Sample/Sound/Decimate**
  (`anim/graph.rs` DENSITY/`KEY`).
- **[STUB] Dope-sheet transform beyond Move:** Extend / Slide / Scale are `op`
  (`TRANSFORM`); only Move (`grab:start`) fires.

NLA (`anim/nla.rs`):
- **[STUB] Strip transform:** Move / Extend / Scale / Swap / Move Up / Down all
  `op`.
- **[STUB] Split / Duplicate / Linked Duplicate / Make Meta / Remove Meta**
  (`STRIPS`).
- **[STUB] Snap submenu** (all four `op`), Transition / Sound add, Selected
  Objects.

Object menu (`part/menubar/editor/view3d/mod.rs`):
- **[STUB] Bake Action…, Bake Object Transform to Grease Pencil…** (`op`).

### 2.3 Rig / parenting limits feeding animation

- **[MISSING] Typed parenting.** Only `PAR_OBJECT` is wired; the ten typed
  kinds (`PAR_ARMATURE_NAME`, `PAR_BONE`, `PAR_VERTEX`, …) are gated on rig
  support and are `MISSING` (`scene/parent.rs` module header).
- **[WIRE] THYMUS `rigAction` end-to-end.** The open item in `jobs.md`
  ("Animation playback of THYMUS (`rigAction`) end to end — needs `phiphx/blend`
  animation + constraints"). The machinery is listed done; this is the remaining
  demo wiring, not absent capability.

---

## 3. Rendering

### 3.1 What works

- **[WORKS] GPU path tracer** (viewport + F12): multi-bounce, GGX, glass, MIS,
  environment importance sampling, adaptive sampling, denoise, DoF with bladed
  bokeh, Filmic/AgX (`phiray/src/trace/`, driven from
  `view/draw/canvas/view3d/rendered/` and `scene/render_job.rs`).
- **[WORKS] Progressive final render** with bands for 4K, dispatch slicing
  against the GPU watchdog, pause/resume/cancel, 8 render slots, Sample/Time/
  Remaining reporting, 8-state render phase (`scene/render_job.rs`,
  `rendered/stepwise.rs`, `scene/render_phase.rs`).
- **[WORKS] Workbench** render: Solid/Wireframe with cavity, SSAO, outline,
  matcap, shadow, DoF (`scene/render_engine.rs` `BStillLook::for_engine`).

### 3.2 Engine-parity gaps

- **[DIVERGENCE] Eevee and Cycles are names only.** Anything that "traces"
  (`engine.traces()`, i.e. every engine but Workbench) is rendered by phiray;
  a `.blend` naming Cycles/EEVEE is treated as a preference and path-traced by
  phiray, logged in the journal (`scene/render_engine.rs`). There is **no**
  Eevee-style rasterizer and **no** separate Cycles integrator. Parity claims
  should say "phiray," not "Cycles."
- **[MISSING] Cycles Properties rows.** Several Render/Sampling sub-panels are
  partial (denoiser options, clamping, Fast GI, tile size/memory, light paths,
  colour-management sub-panels). See the parity table in `panel-parity.md`.

### 3.3 Capability gaps (whole categories absent in phiray)

From `phiray/docs/ROADMAP.md` §1.3 / §3, none of these exist anywhere:

- **[MISSING] Volumes** (no volumetric integration).
- **[MISSING] Hair / curves rendering.**
- **[MISSING] Motion blur.**
- **[MISSING] True displacement** (bump only).
- **[MISSING] Light tree / many-light sampling** (`LIGHTS = 32` cap on file
  lights; mesh emitters via a CDF).
- **[MISSING] General material-graph evaluation on the GPU** — phiphx bakes node
  graphs down to one fixed ~120-word material record (`phiray/src/trace/mat.rs`,
  `proc.rs`); unsupported nodes are approximated or dropped.
- **[MISSING] Hardware ray tracing** (software BVH traversal in compute).

### 3.4 Performance

- **[PERF] Image crosses the CPU every frame.** The traced buffer is read back
  to the CPU and repainted through GPUI (`BStill::read`, `paint_image`). A
  5-second readback hang guard protects the viewport (Windows TDR); a final
  render raises it (`phiray::PatienceFor::render`). A GPU-to-GPU present path
  would remove the per-frame copy — not present.
- **[PERF] Scene size ceiling.** Scenes above the device's max binding are
  refused with a reason (`phiray::Tracer::why_not`); ~7.2–7.6 M triangles render,
  ~20 M are refused on a 128 MB binding limit (`phiray/src/trace/scene.rs`).
  Instancing shares geometry (`Scene::place`), and moves re-upload only the
  instance table / refit the moved mesh (`reinstance` / `repose`).
- **[PLATFORM] No final-render worker on web/wasm.** A browser build has no
  render thread, so F12 there degrades to viewport progressive refine
  (`scene/render_job.rs` offload fallback).

---

## 4. Suggested order of work

1. **M0 — verify GPU path** (§0). Hours.
2. **M1 — wire the CPU rigid-body solver** (§1.1): `scene/sim/` = translate →
   bake → `set_moved`, plus a Bake / Delete-Bake affordance on the Rigid Body
   World panel. Deliverable: a cube dropped on a plane falls on Play. No new
   solver code. Days.
3. **M2 — kinematic + animated bodies, then joints** (§1.2). Days–weeks.
4. **M3 — animation editor operators** (§2.2): copy/paste keyframes, channel
   ops, strip transforms — high user-visible value, independent of physics. Days.
5. **M4 — GPU solver** (§1.4) and **cloth/soft body** (§1.3). Weeks.
6. **M5 — render parity polish** (§3.2) and capability items (§3.3) as needed.

---

# Part II — Verified against Blender source (2026-10-29)

This section re-checks the claims above against the vendored Blender tree at
`ext/REFS/blender/blender/source`. It separates **[MISSING]** (not present),
**[INCORRECT/PARTIAL]** (present but wrong or half-done vs Blender), and
**[DUPLICATED]** (phiber implements the same thing twice).

## A. Physics — verified against `blenkernel/intern/rigidbody.cc` + `DNA_rigidbody_types.h`

### Correct (confirmed against ground truth)
- **Shape dimensions are right.** Blender `rigidbody_validate_sim_shape_helper`
  takes half-bbox `size`; capsule = `new_capsule(radius, (size.z−radius)*2)`,
  cone = `new_cone(radius, size.z*2)`, cylinder = `new_cylinder(radius, size.z)`,
  sphere radius = `max(size.x,size.y,size.z)`. phiray `sim/blend.rs` matches once
  you account for `Shape::capsule/cone/cylinder` storing *half* lengths
  (`sim/shape.rs`): capsule passes `half[2]−radius`, cone/cylinder pass `half[2]`.
  ✓
- **Substep model matches.** Blender `BKE_rigidbody_do_simulation`:
  `timestep = 1/fps * frame_diff * time_scale`, `substep = timestep/substeps_per_frame`,
  loops `substeps_per_frame` times. phiray `World::step` takes `frame_dt*time_scale`
  and divides by `substeps`. ✓
- **Solver iterations are per-substep, not a per-frame budget.** Confirmed:
  Blender calls `RB_dworld_set_solver_iterations(num_solver_iterations)` and Bullet
  runs that many *each* internal step. phiray `from_blend` sets
  `world.iterations = solver_iterations` (not divided) — the comment in
  `sim/blend.rs` already documents fixing exactly this. ✓
- **Defaults 10/10** match (`rbw->substeps_per_frame = 10; num_solver_iterations = 10`).✓

### [INCORRECT/PARTIAL]
- **Shape is not recentred on the bbox midpoint.** Blender's helper assumes
  "Object pivot is centralized in boundbox" and builds the primitive *centred on
  the object origin*. phiray `from_blend` computes `half` from the actual
  `min/max` of the scaled verts but then creates the shape at the body origin
  **without** offsetting by the bbox centre `(lo+hi)/2`. For a mesh whose origin
  is off-centre (common), the box/sphere/capsule will sit offset from where
  Blender puts it. `BodyConst.com_local` exists for exactly this but `from_blend`
  never sets it from the bbox centre. **Fix:** set `com`/shape offset to
  `(lo+hi)/2`.
- **External forces / force fields are per-substep in Blender, absent in phiray.**
  Blender calls `rigidbody_update_external_forces` **inside** the substep loop.
  phiray has no force-field consumption at all (gravity only). Effector weights
  other than gravity are read (`physics/world.rs`) and dropped.
- **Kinematic interpolation across the frame is missing.** Blender builds
  `kinematic_substep_targets` and interpolates kinematic bodies across the
  substeps (`rigidbody_update_kinematic_obj_substep`, `interp_step`). phiray
  `World::drive` snaps to the end pose only; a fast-moving animated collider will
  tunnel where Blender's interpolation would not.
- **Compound shapes are flattened to a hull.** Blender `RB_SHAPE_COMPOUND` builds
  a true compound of child shapes (`RB_compound_add_child_shape`, recursing the
  collection). phiray maps `Compound → hull` (`sim/blend.rs`). Concave compounds
  will be wrong. Acceptable v1, but note it as a divergence, not parity.

### [MISSING] (confirmed absent)
- Rigid-body **constraints/joints** solve — DNA has 8 live types (Point, Hinge,
  Slider, 6DOF, 6DOF_SPRING, Fixed, Piston, Motor; `DNA_rigidbody_types.h`
  `eRigidBodyCon_Type`), each with limits/springs/motor/breaking. phiray `sim/`
  has none. Settings are read (`physics/joint.rs`).
- **Split Impulse** (`RBW_FLAG_USE_SPLIT_IMPULSE`) read into `BRigidWorld.split_impulse`
  but no solver consumer (phiray already separates recovery from velocity via its
  own `push`/`spin`, which is the XPBD equivalent — so this is "N/A by design",
  worth a one-line note rather than a feature).
- Cloth, soft body, particles, fluid, dynamic paint, force fields — as in Part I.
- **The whole thing is still unwired to phiber** (Part I §1.1) — this is the
  gating issue; the solver correctness notes above only matter once it runs.

## B. Layout — verified against `panel-parity.md` (which cites Blender directly)

phiber declares a very wide set of editor hosts (`src/host/`): action, asset,
clip, compositor, console, drivers, file, geonode, image, info, ipo, logic, nla,
node, outliner, buts (properties), seq, shader, spreadsheet, text, uv, view3d,
plus topbar/statusbar/header. And 10 workspaces (`src/space/`: layout, modeling,
sculpting, uv_editing, texture_paint, shading, animation, rendering,
compositing, geometry_nodes, scripting, simulation). So **breadth is high**; the
gaps are depth and polish.

### [INCORRECT/PARTIAL] (from panel-parity.md, each cites the Blender fn)
- **Panel pin** (`PNL_PIN`, Shift-click) — not implemented.
- **Panel reorder by grip drag** (`PANEL_STATE_DRAG`, `sortorder`) — grip is
  drawn but inert.
- **Active-panel outline** (`TH_PANEL_ACTIVE`, `active_property`) — `BCard.active`
  exists but is never drawn; needs a `BEdge` colour token.
- **Open/close animation** (`ANIMATION_TIME 0.30`) — instant; `BFlex::div_unfolding`
  exists and could drive it.
- **Title dim on search-filter miss** (`panel_title_color_get`) — not done.
- **Triangle/grip fade when header narrows** — not done.
- **Region max-clamp against the area** (`SCREEN_OT_region_scale` clamps to
  `maxsize − UI_UNIT_Y`) — area/zone seams resize but don't clamp.
- **Scrollbar thumb dragging, horizontal bars, and non-panel scroll containers**
  (column/popover/thumbnails) — only vertical panel/sidebar bars drag-less-draw.
- **Keyboard panel fold (Enter/A on hovered header)** — not done.

### [MISSING] area operators (vs Blender `screen_ops.cc`)
- Area **split / join / swap** as interactive drags, and **duplicate area into a
  new window** (`SCREEN_OT_area_dupli`) — phiber has fixed area slots + maximize
  + editor-type switch (`area/switch`, `area/maximize`), not Blender's free
  split/join. This is the single biggest *layout* divergence: Blender layouts are
  user-resplittable; phiber's are templated per workspace.

### [DUPLICATED] — resolved, but worth knowing
The team has actively de-duplicated layout and tracks it in jobs.md:
- File browser dialog vs area — now **one** component (`BHostItem(File)`), pinned
  by `file_browser_layout::the_dialog_and_the_area_are_the_same_browser`.
- Header three-cell layout, region padding, pane cells — collapsed to single
  matches (`region/pad.rs`, the header match).
- `prim/panel/*` roles folded into one `BCard` (sync125–128).
No *unresolved* layout duplication found; the open jobs.md item is the ongoing
`div()` → `BFlex` migration (535 raw style calls left), which is cleanup, not a
correctness bug.

## C. Animation — editor is well-factored; the gap is unwired operators

### [DUPLICATED] — NOT a problem (checked)
`scene/action/mod.rs` `BSceneActions` is a **single** state model that the dope
sheet, graph, Action editor and NLA all edit through (`keyed`, `channels`, `nla`,
`ranges`, `assigned`), written once by `edits()` → `BlendEdits`. Dope and graph
do **not** each reimplement keyframe editing — they share `retime.rs`/`keys.rs`/
`channel.rs`. So the earlier worry about duplicated keyframe logic is unfounded.

### [INCORRECT/PARTIAL] / [MISSING] — the `op(...)` stubs (menu entry, no handler)
Confirmed by the `fire()` vs `op()` split in `part/menubar/editor/anim/`:
- **Copy / Paste / Paste-Flipped keyframes** — stub. (No clipboard op anywhere.)
- **Box / Circle / Lasso select in the dope sheet** — stub (the 3D viewport has
  these via `boxsel.rs`/`circlesel.rs`/`lasso.rs`; the dope sheet does not reuse
  them — a *reuse opportunity*, not duplication).
- **Channel ops**: Delete/Group/Ungroup/Toggle-editability/Extrapolation/Expand/
  Collapse/Revive-F-Curves/Bake-Channels — only Clean Channels fires.
- ~~**Keyframe Type, most Easing/Interpolation, Handle type** — stub.~~
  **CORRECTION:** Interpolation, Easing and Handle Type were already wired
  (`anim/set.rs`'s `INTERPOLATION` / `EASING` / `HANDLE_TYPE` all `fire`).
  This line over-generalised from `KEYFRAME_TYPE` being `op(...)`. Keyframe
  Type was the one real stub there, and it is done now (Part VI).
- **Graph-editor Bake / Sample / Sound-to-Samples / Decimate / Euler-filter** — stub.
- **NLA strip transform (Move/Extend/Scale/Swap), Split, Duplicate, Make/Remove
  Meta, Snap** — stub (track add/delete, select, push-down/stash *do* fire).
- **Bake Action, Bake Object Transform** (object menu) — stub.
- **Drivers editor** — host declared (`host/drivers`) but driver *editing* UI is
  not wired (driver *evaluation* works in `phiphx/blend/anim/driver/`).

### Correct / working (unchanged from Part I)
Playback, fcurve/NLA/driver/IK/constraint **evaluation**, insert/jump/duplicate/
delete/clean keyframes, snap, mirror, auto-key — all `fire()`-wired and truth-tested.

## D. Rendering — verified against `blenkernel/intern/camera.cc`

### Correct (confirmed)
- **Camera sensor fit is right.** Blender `BKE_camera_sensor_fit` (AUTO → HOR if
  `sizex ≥ sizey` else VERT) and `BKE_camera_sensor_size` (VERT reads `sensor_y`,
  else `sensor_x`). phiber `scene/render_job.rs` `camera_of` reproduces both
  exactly, incl. the portrait-divides-by-aspect case. ✓

### [INCORRECT/PARTIAL] / divergences (unchanged, confirmed)
- **Eevee / Cycles are name-only aliases to phiray** (`render_engine.rs`). Not a
  bug, but any "Cycles parity" claim is really "phiray parity".
- **Cycles Properties rows missing** — the exact list is enumerated in
  `panel-parity.md` "Render tab against Cycles' ui.py" (adaptive viewport
  sampling, denoiser options, clamping, Fast GI, tile/memory, color-management
  sub-panels, shutter curve, …). The full 670-panel Blender extract is at
  `assets/logo/blender_panels.json` for a systematic pass.

### [MISSING] operators / editors
- **Compositor editor** — backend `scene/composite.rs` runs a `BCompTree` over a
  rendered frame, and a `compositor` host is declared, but the **node-graph
  editing UI** for the compositor is not wired (same pattern as drivers: eval
  without an editor).
- **View layers / render passes beyond 4** — phiray exposes albedo/normal/depth +
  combined; Blender's full AOV/cryptomatte/pass set is absent.
- **Render animation to movie** — the still/F12 path and movie job exist
  (`app/op/render_entry.rs`, `render_save.rs`); confirm the movie *encoder* is a
  real codec path vs frame-dump before claiming parity (flagged, not verified
  to a codec here).

### [DUPLICATED] — clean, not a problem
Viewport refine and final render **share** one `phiray::Camera` and one
`still_stepwise` tracer by design (`rendered/` module headers); the two camera
*builders* differ only because the viewport is an orbit camera and F12 is the
scene camera — that is a real semantic difference, not duplication.

---

# Part III — The three biggest honest problems

1. **Physics is built but unplugged** (Part I §1.1). Everything else in physics
   (joints, cloth, recentring, kinematic interp) is secondary to wiring the
   existing solver into `scene/sim/`.
2. **Dozens of animation menu items are dead** (`op(...)` with no handler) —
   especially copy/paste keyframes and interpolation/handle editing, which users
   will reach for immediately. The editor model is sound; the operators are the
   gap.
3. **Layout is templated, not freely splittable.** Blender users expect to split/
   join/drag any area boundary; phiber gives fixed per-workspace slots + maximize.
   This is the largest *conceptual* layout gap, independent of panel polish.

No evidence of harmful duplicated logic was found — the team has already
collapsed the known layout/animation duplications and pins them with tests. The
remaining "repetition" is the in-progress `div()` → `BFlex` style migration,
which is tracked cleanup.

---

# Part IV — Reuse audit: `ext/phiber-refs/apple` (the old `apple` crate)

Checked `ext/phiber-refs/apple/src` (the RealityKit-shaped reference, excluded
from the live workspace) for code worth borrowing instead of writing fresh. The
rule: **borrow algorithms / WGSL into phiray; do not import the crate**, and
only borrow where the reference is actually better than what phiber/phiray
already has.

## Physics — DO NOT borrow; phiray's solver is already better

The apple physics is a **toy**, confirmed by reading it:
- `apple/src/realitykit/physics.rs` (284 L): explicit-Euler gravity on a
  `Transform`, `linear_drag`, no collision, no contacts. "Joints" are data
  structs + a `JointPinSystem` that just copies a target transform — no
  constraint solve.
- `apple/src/macos/realitykit/physics_advanced.rs` (726 L): a 60-line Verlet
  grid cloth with no collision (ROADMAP §2.1 already flagged it "toy, unbuilt").
- `apple/src/macos/scenekit/physics.rs` (284 L), `physics_fields.rs`,
  `collision_physics.rs` (104 L): SceneKit API-shaped wrappers, not a solver.

phiray's `sim/` (XPBD substeps, real broad/narrow phase, friction/restitution,
Blender-matched substep model, physical tests) is strictly stronger. **Nothing
to port.** Continue on `scene/sim/` wiring (Part I §1.1) + the COM-recentre fix
already landed in `phiray/src/sim/blend.rs`.

Only reusable bits on the physics side are **icons** (`rigid_body.svg`,
`mod_cloth.svg`, `mod_particles.svg`, `con_*solver.svg`, etc. under
`apple/src/assets/icons/`) for the Physics panels.

## Rendering — HIGH-VALUE reuse; this closes Part I §3.3 gaps

`apple/src/appkit/gpu` is a real wgpu renderer (deferred + forward) with working
WGSL that maps straight onto phiber's "missing rendering features". Port the
**WGSL + the algorithm**, re-expressed as phiray `Kernel`s / passes:

| phiber gap (Part I §3) | apple reference to port | File |
|---|---|---|
| Post-process: SSAO, Bloom, Tonemap, FXAA | `post_process/` + shaders | `gpu/shaders/{bloom,fxaa,tone_map}.wgsl`, `post_process/` |
| Shadows (atlas + cascaded/PSSM, point cubemap, spot) | `shadows/` | `gpu/shadows/` (ShadowAtlas, CascadedShadowMap) |
| Deferred lighting / many-light (vs phiray's 32-light cap) | `deferred_lighting/` + volume stencil | `gpu/shaders/deferred_{ambient,directional,point,spot}.wgsl` |
| Hi-Z occlusion culling (scene-size ceiling) | `occlusion/` HiZPyramid | `gpu/occlusion/` |
| **Per-frame CPU readback cost** (Part I §3.4 PERF) | `async_readback/` (buffer, engine, row_layout) | `gpu/async_readback/` — a staged, non-stalling readback; directly addresses the 5 s hang-guard copy |
| EXR save/load | `loader_exr.rs` | `gpu/loader_exr.rs` (phiber already uses `exr` crate, cross-check) |
| Video / movie encode | `video.rs` | `gpu/video.rs` — verify against phiber's movie job (Part II D "render animation to movie" unverified) |
| Glass / transmission pipeline | `glass_pipeline/` + shaders | `gpu/shaders/glass_{blur,composite}.wgsl` |
| Decals | `decals/` | `gpu/decals/` |

Caveats:
- phiray is a **path tracer / megakernel** model; apple is a **rasterizer**
  (G-Buffer MRT + deferred). SSAO/Bloom/Tonemap/FXAA/EXR/video/readback port
  cleanly (they are image-space / IO). Deferred lighting + shadow maps are a
  different rendering paradigm from path tracing — port them only for a
  **Workbench/Eevee-style raster path**, not into the tracer.
- The `async_readback` engine is the most immediately useful borrow: it targets
  the exact perf problem in Part I §3.4 (image crosses the CPU every frame) and
  is renderer-paradigm-agnostic.

## BVH — cross-check, likely no change

`apple/src/phiphx/bvh.rs` (219 L) and `apple/src/services/ecs/bvh.rs` (184 L)
are CPU BVHs. phiray already has a GPU+CPU LBVH/SAH (`phiray/src/bvh/`). Compare
only if phiray's broadphase needs a CPU query path the sim can share; otherwise
skip.

## Net correction to the roadmap

- Physics M4 (cloth/soft/GPU solver): the apple reference does **not**
  accelerate this. Budget it as real work on phiray's XPBD.
- Rendering §3.3 capability gaps: **do** accelerate via the apple `gpu` WGSL —
  especially post-process and async readback. These are the cheapest, highest-
  visibility wins in the reference, and they are image-space so they drop onto
  phiray without adopting its raster paradigm.

---

# Part V — Status after this pass (2026-10-29)

Three of the gaps named above are now closed. Each is tested.

## Closed

**Physics M1 — the solver is wired.** `scene/sim/` exists and works:
`input.rs` turns `SceneModel::physics` into phiray's `BlendWorld` /
`BlendBody` (object-space triangles + raw matrix, and it owns the `tris`
buffers the borrow needs — the two traps named in Part I §1.1), `bake.rs`
bakes the point-cache range on its own thread with a shelf the render pass
reads, `mod.rs` hands the shown frame's worlds to `motion::set_moved` and
draws the Rigid Body World ▸ Cache row with Bake / Delete Bake. Tests:
`a_dropped_cube_falls_lands_and_scrubs_from_the_bake`,
`the_solver_gets_object_space_triangles_and_the_raw_world`. So Part III
problem 1 is no longer true — Play drops a cube.

**Physics correctness — the shape recentre bug is fixed.** `phiray`
`sim/blend.rs` now sets `com` to the bounds' centre for primitive shapes and
tracks the body at its centre of mass (`x_com`), so an object whose origin
sits off its mesh keeps its box / sphere / capsule **on** the mesh, as
Blender's `rigidbody_validate_sim_shape_helper` does. Test:
`a_primitive_centres_on_its_bounds_not_its_origin`. `phiray --test sim`: 8
passed.

**Rendering — Bloom / Glare added.** A missing render feature, built and
GPU-verified: `phiray/src/trace/wgsl/glow.wgsl` + the `Glow` kernel +
`Tracer::glow` (threshold → separable Gaussian → composite, on **scene-linear
radiance before the view transform**, as EEVEE's Bloom and the Glare node
apply it). Algorithm adapted from `apple/src/appkit/gpu/shaders/bloom.wgsl`,
re-expressed for phiray's compute-buffer model with its own scratch binding
so `PX_WORDS` is untouched. Wired through `BRenderSet` (EEVEE's
`use_bloom` / `bloom_threshold` / `bloom_intensity` / `bloom_radius`) →
`BRenderPlan::glow` → `still_stepwise`, after denoise and before composite,
path-traced frames only. Panel: Properties ▸ Render ▸ Bloom. Tests
(`phiray --test glow`, on the GPU): `glare_reaches_out_from_the_bright_patch`
and `a_threshold_above_everything_is_a_no_op`.

**Animation — Copy / Paste Keyframes added.** The two dead menu items users
reach for first. `scene/action/dope/set_copy.rs` holds the runtime buffer in
`ctx.keys.buffer` (`CtxKey::KeyBuffer`, a `ctx` key: never saved, never sent
to a collaborator, as Blender's buffer is runtime), copies the picked
columns' channel values at their frames relative to the earliest, and pastes
with the earliest column landing on the playhead — Blender's default
`KEYFRAME_PASTE_OFFSET_CFRA_START`, merge `MIX`. The menu entries went from
`op(...)` to `fire(...)`, and because the keymap is read off the menus
(`app/keymap/mod.rs`), **Ctrl C / Ctrl V are now bound** by that change
alone. Tests: `copy_and_paste_land_the_buffer_on_the_playhead`,
`a_multi_column_paste_keeps_its_spacing`, `the_copy_buffer_round_trips`.

## Also fixed on the way

- **Build speed.** ~6.3 GB / ~11,500 files of stray nested `target/` trees
  under `crates/apple` and `crates/phiray` deleted — a workspace member must
  build into the root `/target`, and cargo was walking the duplicates on
  every invocation. `cargo metadata` went from stalling to 0.54 s.
  `.gitignore` now has `**/target/`, `**/pkg/`, `**/dist/` so it cannot
  recur. (Checked first that no *source* dir is named `pkg`/`dist`;
  `phiphx/src/scene/build` and `phiplay/src/build` are source and were left
  alone.)
- **A build-breaking bug** in `view/draw/track/wheel/paint.rs`: the cursor
  `quad()` passed a `Bounds` where the border colour belongs, so phiber did
  not compile at all. Now `gpui::white()`, per its own "colour in a white
  ring" comment.

## Still open, in order

1. **Animation operators** — the rest of the `op(...)` stubs: Paste Flipped,
   channel ops (delete / group / ungroup / extrapolation / expand /
   collapse), **interpolation / easing / handle type from the UI** (the
   evaluator supports them; `ipo` / `easing` / `handle` arms already exist in
   `set_edit.rs`, so these need menu wiring more than new logic), NLA strip
   transforms, Bake Action. Dope-sheet box/circle/lasso select could reuse
   the working viewport versions (`boxsel.rs` / `circlesel.rs` / `lasso.rs`)
   rather than being written again.
2. **Physics M2** — kinematic / animated bodies driven per substep
   (`World::drive` exists, nothing feeds it) and rigid-body joints (8 live
   DNA types, settings already read).
3. **Rendering** — the `async_readback` port (the cheapest remaining
   reference borrow; fixes the per-frame CPU copy), then SSAO / shadow
   passes for a raster path.
4. **Layout** — interactive area split / join (the deep one, Part III #3).

NOT accelerated by the `apple` reference: physics (its solver is a toy —
Part IV). Rendering is where that reference pays.

---

# Part VI — Keyframe Type (2026-10-29)

**Added: Key ▸ Keyframe Type** (`ACTION_OT_keyframe_type`) — the last real
stub in the Dope Sheet's key-shape group, end to end through both crates.

Blender keeps a key's type in `BezTriple.hide`, reached through the
`BEZKEYTYPE` macro: the member means "hidden" on a 3D curve point and "the
keyframe's type" on an F-curve's (`DNA_curve_types.h` L100-124,
`DNA_curve_enums.h` L229-241 — read from the vendored source, values
KEYFRAME 0, EXTREME 1, BREAKDOWN 2, JITTER 3, MOVEHOLD 4, GENERATED 5).

- **phiphx** `anim/fcurve.rs`: `BKey::keytype` + the six `KEYTYPE_*`
  constants, defaulting to `KEYTYPE_KEYFRAME` as `insert_vert_fcurve` does.
  Named for what the field *means* on a key, not for the DNA member.
- **phiphx** `anim/read.rs`: read off `hide`; a value outside 0..=5 reads as
  a plain keyframe rather than as a colour nothing draws.
- **phiphx** `write/owned.rs`: written back to `hide`, so a save keeps it.
  `blender_opens_saves` and `save_truth` still pass (1 + 4).
- **phiber** `action/reshape.rs`: `BKeyShape::keytype`, `keytype_of`, the
  `from_value` arm and the `apply` write. The mark's fourth field is
  **optional on parse**, so a three-field mark written before this still
  reads (it leaves the type alone) instead of failing and losing the edit.
- **phiber** `dope/set_edit.rs`: `keytype` joins the `ipo | easing | handle`
  arm — the logic was already there, it only needed the field.
- **phiber** `anim/set_more.rs`: `KEYFRAME_TYPE`'s six entries `op(...)` →
  `fire(... "keytype:<kind>")`, labels and order from Blender's
  `rna_enum_beztriple_keyframe_type_items`.

Tests: `keyframe_type_is_set_only_on_the_named_keys_and_changes_nothing_else`
(it sets the type on the named keys alone and moves no value and no
interpolation), `the_mark_round_trips_the_keyframe_type_and_tolerates_a_three_field_one`,
`every_keyframe_type_menu_value_resolves`. Green: phiber 50 action / 22 dope
/ 7 reshape, phiphx 40 anim / 20 write / 6 fcurve.

**Not done:** the editors do not yet *draw* a key differently by type —
Blender tints and reshapes the diamond (`TH_KEYTYPE_*`). The data is now
read, set and saved, so the drawing is a renderer change on top of it.
