# phiphx: original plan vs what exists now

Read-only research. Nothing was edited, built or run. "not checked" = I did not open the code or run the test.

## Summary
- The original `phi/crates/phiphx` (crate `phx`, "PHI Point Cloud eXtension") was a point-cloud converter for USD/glTF. A native `.blend` -> PHX Scene/USDZ path was added in April 2026 and stopped at Phase 1 (names, transforms, collections, modifier/instance flags).
- The original plan for .blend was: DNA geometry (Phase 2), materials/nodes (3), animation/rigging/shape keys (4), then modifiers, particles, physics, curves, GP, volumes (5). Output targets were PHX, and per-object USDZ for RealityKit.
- The current `phikit/crates/phiphx` has far exceeded the .blend reader plan (Phases 1-4 mostly, plus modifier evaluation, constraints, drivers, NLA, cloth cache, node shading, a byte-exact writer). It is checked against Blender 5.x (`tests/*_truth.rs`).
- The USD side is the gap: the writer is USDA-only (mesh + point cloud; no materials, lights, cameras, skeleton or animation), the USDC reader is a float-sniffing heuristic, and `openusd` is a stated TODO that is not in any Cargo.toml.
- Nothing converts a .blend to USD on load. `Phx::open` returns a `SceneModel`. The only .blend to USD code is `BlenderScene::to_usda` (a second, thinner walk, per `blend/asset/jobs.md`) and there is no USD round-trip test.
- Still missing from the .blend reader for a faithful USD target: linked libraries, shape keys, layered-action coverage, particles, Geometry Nodes beyond Smooth by Angle, grease pencil, volumes, point cloud/Curves objects.
- Recommended order: (1) one `SceneModel` -> USD writer with the full prim set, (2) USD reader via a real library, (3) blend->usd on load with Blender/usdchecker truth tests, (4) the reader gaps above.

## 1. Original plan (intended capabilities)

Source folder: `phi/crates/phiphx/` (crate name `phx`).

| # | Intended capability | Source |
|---|---|---|
| O1 | Import USD (usda/usdc/usdz) and glTF/glb | `README.md`, `Cargo.toml` (gltf, zip deps; `usd-support`, `gltf-support` features) |
| O2 | Export PHX (PLY-style), PCD, LAS, XYZ, JSON, Binary | `README.md` format table |
| O3 | Mesh -> point cloud sampling (area-weighted/uniform), point cloud ops (transform, filter, downsample, merge, stats) | `README.md` |
| O4 | Native Rust `.blend` parser (header, blocks, DNA, no Blender needed); 64/66-byte ID names, 4/8-byte pointers, endianness | `BLEND_CONVERSION_GUIDE.md` steps 1-3; `BLENDER_API_COVERAGE.md` "Notes on DNA Versions" |
| O5 | Decode Object/Collection/Scene; objects with name, type, loc/rot/scale, obmat, modifiers, instances (dup_group, instance_collection) | `BLENDER_API_COVERAGE.md` Phase 1 (marked done) |
| O6 | Phase 2 geometry: MVert, MPoly+MLoop, normals, MLoopUV, MLoopCol | `BLENDER_API_COVERAGE.md` Phase 2; `CONVERSION_SUMMARY.md` |
| O7 | Phase 3 materials: colours, slots, image texture refs, PBR, node tree | `BLENDER_API_COVERAGE.md` Phase 3 |
| O8 | Phase 4: armature bones, vertex-group weights, actions/keyframes, shape keys | `BLENDER_API_COVERAGE.md` Phase 4 |
| O9 | Phase 5: modifier stack details (subsurf, mirror, array, bevel, boolean, armature, displace, solidify, cast, wave, curve, lattice, meshdeform, uvproject, vertexweight), particles, physics, curves/surfaces, metaballs, grease pencil, volumes | `BLENDER_API_COVERAGE.md` Phase 5 and modifier table |
| O10 | Scene props: camera, world, active object, layers, master collection; Armature/Bone fields | `BLENDER_API_COVERAGE.md` Scene/Armature/Bone tables |
| O11 | PHX Scene format: `PHX_SCENE\x01` + u32 JSON length + JSON + binary geometry; camera, lights, meshes, instances, materials, collections | `BLEND_CONVERSION_GUIDE.md` step 5-6; `src/exporters/phx_scene.rs` (file exists, not read) |
| O12 | Project extraction: `.blend`/`.usd` -> `[name].phx/` folder mirroring the collection tree, one `.usdz` per object (uncompressed zip with `.usda`), skip `WGT` collections | `PROJECT_EXTRACTION.md`; `src/project.rs`; `examples/extract_project.rs` |
| O13 | Hand off to RealityKit / rust-kit-pro: Entity <- ID.name, MeshResource <- PHX buffers, modifier/instance flags -> MetadataComponent; `import_asset_from_path` treats `.blend` via native parser then PHX | `PHX_TO_RUST_KIT_PRO.md` |
| O14 | Prefer USDZ output so RealityKit loads natively ("Option B", BOT-anist pattern); Rust does orchestration/extraction only | `SWIFT_INTEGRATION_ANALYSIS.md` |
| O15 | Fix the two EOF-failing files (114 MB city, 169 MB Smile&Driveaway) or fall back to Blender CLI; activate geometry from ME blocks | `CONVERSION_SUMMARY.md` "Workflow & Next Steps" |
| O16 | Future: collection hierarchy, instance/dupli, streaming/multithreaded parse, version-specific struct layouts, 2.7x legacy files | `BLEND_CONVERSION_GUIDE.md` "Future Improvements"; `BLENDER_API_COVERAGE.md` |
| O17 | Deprecated Blender-subprocess/Python-script route (`importers/blender.rs`, `blender_export_script.rs`) | `BLEND_CONVERSION_GUIDE.md` Files table |
| O18 | Known limitations stated at the time: no materials, modifiers baked/not preserved, static frame only | `BLEND_CONVERSION_GUIDE.md` |

Wider repo mentions (`phi/crates`, grep only, not read in depth): appkit/kitpro/Jingle (RC Pro clone) docs assume USDZ as the asset format and "PHX material format" partial (`appkit/doc/realitycomposerpro/STEP_BY_STEP_SHADER_GRAPH_GUIDE.md:28`); `PHIPRO_6_STEP_EXECUTION_PLAN.md:36` notes many `.usdz` hold binary USDC payloads; `appkit/src/gpu/assets/asset_loading.rs:31` routes usd/usda/usdz/usdc through `phx::importers::usd`; `appkit/doc/platform_evaluation.md:190` "Load GLTF/USD via PHX crate". No separate OpenUSD plan was found in realitykit / reality-foundation / phidoc (grep matched nothing relevant; the files were not read in full).

## 2. Plan vs now

Current crate: `phikit/crates/phiphx` (sources: `README.md`, `jobs.md`, `TODO.md`, `ARCHITECTURE.md`, and `src/*/README.md` + `jobs.md` for blend, blend/shade, blend/object, blend/asset, scene, import, import/usd, import/gltf, export, export/usd, export/gltf, export/blend, export/scene, plus directory listings of `src/blend/*` and `tests/`).

| Planned item | Original source | Current phiphx (paths) | Status | Notes |
|---|---|---|---|---|
| O1 USD import | README.md | `src/import/usd/` (usda*.rs, usdc.rs, usdz.rs) | partial | USDA line-scanner builds meshes/point clouds; USDZ unzip needs feature `usdz-import`; USDC heuristic only. `import/jobs.md`: "USD via openusd (replace ASCII-only scanner)" open |
| O1 glTF import | README.md | `src/import/gltf/` | partial | Meshes, node transforms, point sampling. `import/gltf/jobs.md`: feeding `SceneModel` (materials, hierarchy) through `Phx::open` is open. Feature `gltf-import` is optional |
| O2 exports | README.md | `src/export/{ply,pcd,las,xyz,json,binary,gltf,usd,phx}` | done (tests for LAS/PCD/XYZ round trips open) | `export/jobs.md` |
| O3 sampling/cloud ops | README.md | `src/cloud/`, `src/geom/` | done (not checked in detail) | |
| O4 native .blend parser, DNA | BLEND_CONVERSION_GUIDE | `src/blend/file/` (header, blocks, sdna, zstd, quick peek) | done | Reads 2.8 -> 5.2 per `blend/jobs.md`; zstd for 4.5+; wasm path. 2.7x legacy: not checked. Header v1 (5.x) handled per jobs "Header / v1 / zstd" |
| O5 objects/collections/scene | COVERAGE Phase 1 | `src/blend/object/`, `src/blend/scene/` | done | Local/world matrices, all rotation modes, parent chain, view layer, collections, instances (`instance.rs`, `instance_members.rs`). `object/jobs.md` still lists "Collection instances and dupli (transflag)" and PARBONE/PARSKEL open, but `TODO.md` says collection instances done and CRYSTAL-CAVES-CITY matches exactly. The jobs file is stale |
| O6 geometry | COVERAGE Phase 2 | `src/blend/mesh/` | done | Positions, faces, UVs, normals, attributes (5.x), CustomData (<=4.x), vertex groups. 1,276 meshes equal Blender 5.0.1 digests (`tests/blender_truth.rs`, `tests/truth/mesh_digest.txt`). MLoopCol/colour attributes: not checked |
| O7 materials | COVERAGE Phase 3 | `src/blend/shade/` | done for display, partial for export | Slots, Principled, node links, image textures + packed bytes, node groups, procedural textures, 29+ node types. Open: node-group instanced twice, >2 textures per socket, 4D textures, colour space/UDIM, `Undefined[...]` idnames (`shade/jobs.md`). It is baked to a viewport shader record, not a portable material graph |
| O8 armature/rigging | COVERAGE Phase 4 | `src/blend/rig/`, `modifier/armature.rs`, `scene/rig.rs`, `docs/character.md` | done | Bones, constraints, IK, Rigify poses (0/404 bones over 1e-4 per TODO.md), skinning with vertex groups |
| O8 actions/keyframes | COVERAGE Phase 4 | `src/blend/anim/` (fcurve, nla*, drivers, motion, pose) | partial | Bezier and NLA done. `blend/jobs.md` Todo still says "legacy `bAction.curves` and layered actions (4.4+)" open. `make_actions.py` and `actions.blend` fixtures exist; layered-action coverage: not checked |
| O8 shape keys | COVERAGE Phase 4 | none found | missing | No `Key`/`KeyBlock` reader (grep found only a comment saying shape keys on curves are "not read", and a modifier name "Shape Key" in `modifier/report.rs`). `tests/keys_truth.rs` is for keyframes/dope keys. Not confirmed from the code |
| O9 modifiers | COVERAGE modifier table | `src/blend/modifier/` (`KINDS`) | changed: more than planned | Evaluated, not just detected: Subsurf, Armature, Solidify, Mirror, Lattice, Array, Shrinkwrap, Curve, CorrectiveSmooth, Multires, Mask, SimpleDeform, SurfaceDeform, Cloth (baked cache), Smooth by Angle (TODO.md says open, but `tests/smooth_by_angle_truth.rs` exists, state not checked). Boolean, Bevel, Cast, Wave, Displace, MeshDeform, UVProject, VertexWeight: not checked individually |
| O9 particles | COVERAGE Phase 5 | none (`modifier/report.rs` only names the type) | missing | `TODO.md`: none of the 230 files use them |
| O9 physics | COVERAGE Phase 5 | `src/blend/physics/` | partial | Settings only, no solver. `TODO.md`: 3 of 6 physics truth tests fail on flag names (as of 2026-10-28) |
| O9 curves, text | COVERAGE Phase 5 | `src/blend/curve/`, `src/blend/text/` | done | Bevel/taper/profile/fill; Type 1 + TrueType text (`tests/curve_mesh_truth.rs`, `text_truth.rs`) |
| O9 metaballs, grease pencil, volumes, point cloud/Curves(hair) objects | COVERAGE Phase 5 | only `ObjectKind` names (`blend/object/kind.rs`) | missing | TODO.md: all 8 GP objects are empty annotation layers; volumes "not planned" |
| O10 camera, light | COVERAGE Scene | `blend/object/data.rs`, `scene/extras.rs` (`SceneCamera`, `SceneLight`), `blend/object/dof.rs` | done | Lens, sensor, shift, ortho, clip, DOF; light kinds, energy (area-light scale verified vs Cycles), drivers on lights |
| O10 world | COVERAGE Scene | `blend/object/world.rs`, `blend/shade/world.rs` | done | Sky/HDRI nodes, missing-HDRI handling; `tests/world_truth.rs`, `sky_truth.rs` |
| O11 PHX Scene format | BLEND_CONVERSION_GUIDE | `src/export/scene/` (PhxScene), `src/import/phx.rs`, `src/blend/asset/phx.rs`, `src/export/phx*` | done, still thin | JSON + binary container with camera, lights, meshes, instances, materials. Sidecar `.phx` written on import. `blend/asset/jobs.md`: build it from `SceneModel` rather than a second walk, and use real triangulation not fans |
| O12 project extraction (.phx folder of per-object .usdz) | PROJECT_EXTRACTION | `src/project.rs`, `src/import/project.rs` | partial | Still present in lib (`project.rs`); per-object usdz via `export/usd/mesh.rs`. Whether it still mirrors the collection tree: not checked |
| O13 rust-kit-pro / RealityKit hand-off | PHX_TO_RUST_KIT_PRO | consumers are now phiber / phiray / phiecs (`ARCHITECTURE.md`) | changed | Target is no longer Apple frameworks. phiecs mapping ("a word an object") is dashed/planned in the architecture diagram |
| O14 USDZ output for native loaders | SWIFT_INTEGRATION_ANALYSIS | `src/export/usd/` (USDA only) | partial | No usdz packing in the export tree (only `import/usd/usdz.rs`). Not checked: whether `project.rs` packs zip itself |
| O15 EOF-failing big files | CONVERSION_SUMMARY | `src/blend/file/` | likely done | HEXA-COMBINED (193 MB) opens (memory work in `scene/jobs.md`); the exact two files are not named in current docs. "Smile&Driveaway" and the 114 MB city: not checked |
| O16 legacy/2.7x, streaming | guide | progressive open (`Phx::open_progressive`, `scene/progressive*.rs`) | partial/changed | Streaming is replaced by progressive open. 2.7x: not checked |
| O17 Blender-subprocess route | guide | not present | dropped | Intentional: native only |
| Linked libraries (`LI`, `ID.lib`) | not in original | `blend/jobs.md` Todo | missing | New requirement |
| Writer (.blend back out) | not in original | `src/blend/write/` | added | Byte-for-byte round trip + patches (`tests/save_truth.rs`). New objects/meshes not yet writable |

## 3. State by topic

### .blend reading
Sources: `src/blend/README.md`, `blend/jobs.md`, `TODO.md`, `ARCHITECTURE.md`, directory trees under `src/blend`.

| Area | State |
|---|---|
| Container/versions | Header, v1 header, BHead layouts, gzip and zstd, SDNA, DNA-size self-check; "reads Blender 2.8 -> 5.2". Pre-2.8 / 2.7x: not checked. 5.x attribute storage and <=4.x CustomData both handled in `mesh/` |
| Meshes | done, verified against Blender 5.0.1 digests for 1,276 meshes |
| Materials / node trees | Folded for display (Principled, mixes, groups, texture nodes, drivers). Not a portable node graph, so not suitable as UsdPreviewSurface/MaterialX without a new exporter. Open items in `shade/jobs.md` |
| Lights, cameras | done (`SceneLight`, `SceneCamera`) |
| Modifiers | Evaluated to final geometry for display; the stack itself is not stored in `SceneModel` (not checked) |
| Armatures | done, including constraints, IK and skinning. `Skinner` trait for CPU/GPU skin |
| Animation | f-curves, Bezier, NLA, drivers (objects, lights, materials, world, modifiers) done. Legacy and layered-action readers listed open in `blend/jobs.md` (contradicts the many `*_truth` tests, so verify) |
| Images / packed files | done: keys, packed bytes, decoded textures, missing-file search (`scene/texture_find.rs`) |
| Collections / instances | done (collection instances, view-layer flags, instance visibility) |
| Linked libraries | missing (`LI` blocks, `ID.lib`) |
| Shape keys, particles, GP, volumes, hair | missing |
| Geometry Nodes | only Smooth by Angle in the plan (`tests/smooth_by_angle_truth.rs` exists) |
| Versioning across Blender versions | reader handles 2.8-5.2; one example in TODO.md of a versioning gap fixed (Limit Rotation legacy clamp). Truth values come from Blender 5.0.1 (mesh digests) and 5.2.x (TODO.md) |

### USD
- Read: `src/import/usd/` USDA text scanner (`def Mesh`, points, indices, transforms, point clouds); USDZ unzip behind feature `usdz-import`; USDC is `usdc.rs` "heuristic extraction ... largest contiguous run of valid f32 triples ... synthesise triangles" (reads vertices only; no hierarchy, materials or topology). Declared Todo: use `openusd`; `openusd`/`pxr` do not appear in any Cargo.toml I searched (`Cargo.toml`, `crates/*/Cargo.toml`).
- Write: `src/export/usd/` `UsdaWriter` writes `metersPerUnit = 1.0`, `upAxis = "Y"`, `def Xform`, `xformOp:transform`, `def Mesh`, `def PointInstancer` (for clouds and instances). `export_usdc` is a stub returning an error. Prims/schemas not found in the exporter (grep for Material, UsdShade, Skel, Light, Camera returned nothing): materials, lights, cameras, skeletons, animation (timeSamples), normals/UV primvars: not checked in the mesh writer but not found.
- Blender uses Z-up; the writer sets Y-up. Whether it rotates the data is not checked.
- Round-trip testing: `export/usd/tests.rs` only checks the text contains `def Xform`, `def PointInstancer`, `def Mesh`. No test reads a file back, runs `usdchecker`, or compares with Blender's own USD export.
- Not planned or built: USDZ writer, UsdSkel, UsdLux, UsdGeomCamera, MaterialX, USD import from Blender's exporter.

### glTF
- Read: `import/gltf/` (feature `gltf-import`): meshes, node transforms; no materials/hierarchy into `SceneModel`.
- Write: `export/gltf/` meshes and point clouds (GLB/embedded base64); `export/blend/script_gltf.rs` scene writer delegating to it. Skins, materials, lights, cameras, animation: not found/not checked. `ARCHITECTURE.md` lists "rig out: skins and joints in the glTF writer" as to-build.

### PHX format
- Two things use the name: PHX point cloud (PLY-flavoured, `export/phx*`, `import/phx.rs`) and PHX Scene (`PHX_SCENE\x01` JSON + binary, `export/scene/`). Both exist, read and write. Sidecar written on `.blend` import. Format spec lives only in module docs; no versioned spec document: not checked.

### .blend -> USD on load
- Not implemented. `Phx::open` (`src/phx.rs`) calls the .blend reader and returns a `SceneModel`. Other readers go through `import/asset.rs`. `BlenderScene::to_usda` (`export/blend`) and `PhxScene` are separate, thinner converters than `SceneModel`. `blend/asset/jobs.md` already lists "One converter: build AssetImportResult from SceneModel instead of a second walk".
- phiber's menu has a "Universal Scene Description (.usd*)" import entry (`phiber/src/part/menubar/topbar/set.rs:64`); what it does on click: not checked.

### Round-trip testing against Blender (`tests/truth`)
- `tests/truth/` has 99 files (43 txt/tsv truth dumps plus about 50 `blender_*.py` scripts) and `tests/fixtures/` 111 files, with about 60 `*_truth.rs` tests in `tests/` (meshes, bsdf, nodes, curves, modifiers, drivers, NLA, cloth, lights, world, save...). Method: Blender dumps values, phiphx test compares (`TODO.md` "a port counts as done when its fixture test passes and the real files that use it diff clean").
- Gaps: no truth for USD (Blender -> `.usd` export compared to phiphx reading/writing), none for glTF, no scripted `usdchecker`/`usdcat` check. `tests/blender_opens_saves.rs` and `save_truth.rs` cover Blender reading phiphx's saved .blend.

### Spec: `.kiro/specs/blend-file-mesh-loading`
Phiber file-browser click -> import -> viewport render; Blend importer hardening (no panics), error state. It targeted `crates/apple/src/phiphx/...` (older layout). 42 tasks ticked, 9 open (counted by checkbox). No USD content. Other specs mention phiphx or .blend (`phiber-blender-space-parity`, `phiray*`, `reality-composer*`, `unified-appkit-apps`, `apple-missing-crates`); not read.

## 4. Prioritised missing list for "convert every .blend to OpenUSD on load"

Stage 0 - decide the contract (small)
- Pick the single source: `SceneModel` (as `blend/asset/jobs.md` says). Retire the second walk (`BlenderScene`, `AssetImportResult`) once the writer works.
- Choose USD stack: `openusd` Rust crate (planned in `import/jobs.md`) or an in-house usdc writer. Decide Z-up/Y-up and metersPerUnit conventions (Blender: Z-up, 1 unit = 1 m by default).
- Decide where the result lives: cache `.usd/.usdc` beside the file or in a project cache, keyed by file hash plus Blender-version/reader-version.

Stage 1 - USDA writer that covers the current reader (the target's first useful slice)
1. Prim set: Xform hierarchy from parent chain and collections (Collection as Xform/Scope), `UsdGeomMesh` with points, faceVertexCounts/Indices, normals (face-varying from `SceneMesh::corners`), UV primvars, subdivisionScheme none (modifiers evaluated), per-material `GeomSubset`.
2. Instancing: collection instances as USD instances/`PointInstancer` referencing shared prototype meshes (the data already shares `Arc<SceneMesh>`).
3. Cameras (`UsdGeomCamera`, aperture and focal length from `SceneCamera`), lights (`UsdLux`: Sphere/Distant/Rect/Disk/Cylinder, energy conversion verified), world (`DomeLight` with the HDRI or sky colour).
4. Materials: `UsdPreviewSurface` from `BlendBsdf` + images (textures as `UsdUVTexture`, packed images written beside or into a usdz); later MaterialX or a custom `ND_` mapping for node graphs.
5. Visibility, selection and view-layer exclusion as `visibility` / purpose; per-object `custom` metadata (original Blender name, ID address).
6. Test: write, read back with an independent reader, `usdchecker`; compare bounding boxes and triangle counts to `tests/truth` counts (TODO.md already has the census tooling: `tri_count`, `boxdiff.py`).

Stage 2 - USD reader that is real (needed for round trip and for any `.usd` input)
1. Replace the USDA scanner with a real parser: layers, references/payloads, variants, composition (`openusd`).
2. USDC: real crate decode (replaces the f32-sniffing heuristic) and USDZ via zip.
3. Read into `SceneModel` (not only point clouds): hierarchy, materials, lights, cameras, skeleton.
4. glTF into `SceneModel` (`import/gltf/jobs.md`).

Stage 3 - on-load conversion and verification
1. Hook `Phx::open*` (or the asset router) to produce/cache the USD and report it in the read report; progressive open should emit USD after the base scene.
2. Truth tests: for each fixture in `tests/fixtures`, Blender's own `bpy.ops.wm.usd_export` vs phiphx's USD (prim paths, mesh hashes, transforms, materials). Add a `blender_usd.py` in `tests/truth`. Include a Blender-generated USD as a read test.
3. Add the 230-file census run to compare USD output counts with the existing `diff.py` (TODO.md).
4. USDC writer (currently a stub) and USDZ packer.

Stage 4 - animation and rigs in USD
1. `UsdSkel`: Skeleton, SkelAnimation, skin weights and joint indices from `BRig` and vertex groups (the rig data and `Skinner` already exist).
2. Transform animation as `timeSamples` from f-curves / NLA / drivers evaluated per frame (`SceneMotion::solved_at`, `SceneRigs::*_at`). Frame range and fps from the scene.
3. Shape keys (needs a reader first; see Stage 5) as `BlendShape`.
4. Constraint results can be baked to samples; drivers likewise.

Stage 5 - reader gaps that limit a faithful conversion
1. Linked libraries (`LI` blocks, `ID.lib`): resolve and emit as USD references. Listed open in `blend/jobs.md`.
2. Shape keys (`Key`/`KeyBlock`, relative keys, vertex group masks); not found.
3. Layered actions (Blender 4.4+) and legacy `bAction.curves` as listed open in `blend/jobs.md` (verify against existing tests first).
4. Per-frame geometry for non-rigged objects and driven modifier settings in the prefix (`scene/jobs.md` items 2 and 3).
5. Shader node coverage still open in `shade/jobs.md` (node group reuse, texture stacks, UDIM, colour space) since materials export depends on it.
6. Geometry Nodes beyond Smooth by Angle, particles/hair, grease pencil, volumes, point cloud objects, metaballs: low priority by the census (none or few of the 230 files), but they are what "every .blend" would hit in other users' files.
7. Vertex colour attributes, custom attributes as primvars; check `MLoopCol`/colour attributes (not checked).
8. Pre-2.8 .blend (2.7x) if "every" must include them: not checked.

Stage 6 - housekeeping that blocks confidence
- Reconcile stale jobs files (`blend/object/jobs.md` says collection instances open; `TODO.md` says done; `blend/jobs.md` animation line).
- The two big failing files from the old summary: re-run and record.
- Physics truth tests failing (3 of 6) per `TODO.md`.
- Remove the duplicate reader copy in `crates/apple/src/phiphx` (`import/jobs.md`) and the `docs/from-phi` copy.

## Files read
Original: `phi/crates/phiphx/{README, CONVERSION_SUMMARY, PHX_TO_RUST_KIT_PRO, BLENDER_API_COVERAGE, BLEND_CONVERSION_GUIDE, SWIFT_INTEGRATION_ANALYSIS, PROJECT_EXTRACTION}.md`, `Cargo.toml`, `src` file listing (sources not read), `examples` and `tests` listings. Grep across `phi/crates` for phiphx/PHX/USDZ/OpenUSD (matches listed, not read).
Current: `phikit/crates/phiphx/{README, jobs, ARCHITECTURE, TODO}.md`, `Cargo.toml`; `src/*/README.md` and `jobs.md` for blend, blend/asset, blend/object, blend/shade, scene, import, import/usd, import/gltf, export, export/usd, export/gltf, export/blend, export/scene; `src/import/usd/usdc.rs` (head), `src/scene/extras.rs` (head), `src/phx.rs` (greps); directory listings of `src/blend`, `tests`, `tests/truth`, `tests/fixtures`; targeted greps for DNA names and shape keys/libraries/particles.
Specs: `.kiro/specs/blend-file-mesh-loading/{requirements, tasks}.md` (partly), plus a file-name grep over all specs.
Not checked: contents of `src/blend/*/jobs.md` other than those listed (mesh, modifier, anim, rig, curve, physics, write), `docs/character.md` beyond its head, `docs/from-phi`, `docs/archive`, `reference/`, any test body, and whether any test or build was passing.
