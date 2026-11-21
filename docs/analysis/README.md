# phiber against Blender: overview and plan

Four read-only analyses, written 2026-11-10 against Blender v5.2.0 source (`ext/phiber-refs/blender`; v5.2.4 is in `reference/blender` upstream). Nothing was built or run. Day counts are estimates for one engineer who knows the code; they are not measured.

| Report | Question |
|---|---|
| [01_cycles_vs_phiray.md](01_cycles_vs_phiray.md) | What phiray has and lacks against Cycles |
| [02_blender_vs_phiber.md](02_blender_vs_phiber.md) | What phiber has and lacks against Blender's editors, modes, modifiers, animation, physics, IO |
| [03_phiphx_plan_vs_done.md](03_phiphx_plan_vs_done.md) | phiphx's original plan in `phi/crates/phiphx` against the current crate; the path to .blend to OpenUSD on load |
| [04_apple_engine_and_usd.md](04_apple_engine_and_usd.md) | A second engine on Apple's stack (Metal ray tracing, MetalFX, MPS), the benchmark, and the state of OpenUSD |

## Where things stand

- **phiber** is a viewer, player and renderer with light object-level editing, not yet a modeller. Reading `.blend` files is strong (mesh, materials, lights, cameras, rigs, constraints, IK, NLA, drivers, about 20 modifiers). Missing: mesh edit mode and tools, sculpt and paint, geometry-nodes evaluation (only Smooth by Angle), shape keys, particles, fluid, grease pencil, sequencer, Python, and a real undo stack.
- **phiray** is a real GPU path tracer (wgpu megakernel, SAH BVH, GGX VNDF, MIS, environment importance sampling, adaptive sampling, a-trous denoise, Filmic and AgX). Materials are a fixed 164-word record that phiphx bakes Blender node graphs into, so unsupported nodes are approximated or dropped. Missing: volumes, hair, motion blur, true displacement, light tree, random-walk SSS, light linking, shadow catcher, holdout, Cryptomatte, AOVs, baking, panoramic cameras, OIDN.
- **Cycles and EEVEE are names only in phiber.** Both are path traced by phiray, so any parity claim today is a phiray claim.
- **phiphx** has gone far past its original plan for `.blend` reading (about 60 tests compare against values Blender dumped). The original plan was a USD and glTF point-cloud converter with per-object USDZ output for RealityKit. **USD is now the gap:** the writer is USDA only (meshes and point clouds), the USDC reader is a heuristic, there is no USDZ writer, no OpenUSD dependency, no `.blend` to USD on load, and no USD round-trip test.
- **Apple side:** every Apple bridge in `phi/crates` and `phikit/crates/apple` is a stub ("Simulated GPU", ray tracing false). The only real Metal is `phikit/crates/appkit/gpu/gpui_macos` (UI, `metal` 0.33 and `objc2`). USDKit and USDObjCKit header folders are empty. No OpenUSD source or Rust USD crate exists anywhere in the tree.

## The one thing to fix first

The viewport's timing is wrong: with one slice it records submit time, so GPU work is booked as readback (`phiray/docs/APPLE.md`, row 6). No speed measurement, and no engine comparison, can be trusted until that clock uses GPU timestamps. This is stage 0 below.

## Plan

### Stage 0: measure honestly (about 1 week, Windows)
1. Fix the slice clock: GPU timestamps per slice, CPU submit time reported separately.
2. Baseline phiray numbers on the cube and the pose-26 scene.
3. A Cycles comparison harness: render the same scenes in Blender 5.2 Cycles and compare per pass (RMSE, SSIM). No such harness exists, so "looks like Cycles" cannot be claimed yet. (01, item 1.13)

### Stage 1: a typical Blender scene renders like Cycles (about 110 to 170 days, Windows)
In order of payoff (01, stage 1): a GPU material evaluator to replace the fixed record (20 to 30 d, the largest gap, and a prerequisite for displacement, volumes and AOVs), Principled v2 completion with a proper closure stack, random-walk SSS, homogeneous volumes, light tree, passes and AOVs plus Cryptomatte, holdout and shadow catcher, panoramic cameras, hair, OIDN, true displacement, colour management.

### Stage 2: OpenUSD as the interchange, `.blend` converted on load (Windows)
(03, stages 0 to 4)
1. Decide the contract: `SceneModel` is the single source; one USD writer replaces the second walks (`BlenderScene::to_usda`, `AssetImportResult`). Pick the USD stack: grow phiphx's own reader for demo scenes, or wrap OpenUSD C++ through a C ABI for full composition. The maintainer decides; 04 section 4 recommends the wrapper for real applications.
2. A full USDA writer: Xform hierarchy, meshes with normals, UVs and subsets, instancing, cameras, UsdLux lights, dome light, UsdPreviewSurface materials.
3. A real USD reader (layers, references, payloads, variants, USDC, USDZ).
4. Convert on load and cache by file hash and reader version; truth tests against Blender's own USD export.
5. UsdSkel and timeSamples animation.
6. Reader gaps that block a faithful conversion: linked libraries, shape keys, layered actions, per-frame geometry for non-rigged objects.

### Stage 3: a second engine on Apple's stack (about 75 to 125 days, needs a Mac)
(04, sections 2, 3 and 6)
1. `phiray-core`: one `RenderEngine` trait and the shared scene lowering (`trace::Scene`) that phiray already uses. Buildable and testable on Windows.
2. `phiray-metal` (macOS only, behind `cfg`): Metal primitive and instance acceleration structures with compaction and refit, an MSL megakernel matching phiray's material record, then shading parity (try naga's MSL backend for the shading half).
3. Optional post stages compared as their own rows: MetalFX temporal denoised scaler (macOS 26), MPS SVGF, phiray's a-trous.
4. Benchmark: same scene, camera, bounces, seeds and passes. The headline is time to equal RelMSE against a converged reference. Run phiray-wgpu on the same Mac (wgpu has a Metal backend) so both engines share a GPU. Never mix Windows and Mac numbers on one plot.
5. Optional third arm: hardware ray query inside phiray through wgpu's experimental API, on both platforms.

### Stage 4: phiber as an editor (about 6 to 9 person-months for the basics)
(02, stage 1) Per-frame evaluation completeness, an edit-mode mesh structure and the core tools (extrude, inset, bevel, loop cut, knife), a real undo stack, Graph editor editing, saving a `.blend` from scratch, Properties editing breadth, and the sweep of about 380 inert menu entries. Stages 2 and 3 in 02 (geometry nodes, sculpt, paint, physics, grease pencil) are multi-year.

## What phiphx's original plan left undone
From 03: shape keys (planned in Phase 4, no reader found), particles, physics solvers, metaballs, grease pencil and volumes (Phase 5), USDZ output for native loaders (only a USDA writer), and the RealityKit hand-off (the target moved to phiber, phiray and phiecs). Linked libraries were never in the plan and are still open.

## Caveats
- phiber's and phiray's own status docs drift (stale "open" items that are done, `ROADMAP.md` says 32 lights where the code has 64). Statuses here come from code layout first, docs second; function bodies were mostly not read.
- Each report lists what was not checked at its end.
