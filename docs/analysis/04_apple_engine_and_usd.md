# 04 - Apple-stack second engine for phiber, and OpenUSD

Read-only research, 2026-11. Nothing built, run or edited. "not checked" marks anything I did not open.

## Summary

- Every Apple bridge in the tree is a **stub**: Rust traits and structs with no objc/FFI. `MTLDevice::system_default()` returns a hardcoded "Simulated GPU" with `supports_raytracing: false`. Nothing there can create a Metal acceleration structure.
- The only **real** Metal in the tree is `phikit/crates/appkit/gpu/gpui_macos` (UI rendering). It uses the `metal` 0.33, `objc` and `objc2` crates, which gives us a working crate set to build on.
- Headers on disk cover Metal (incl. MTL4), MetalFX (denoised scaler is macOS 26+), MetalKit, ModelIO, RealityKit/Foundation, SceneKit, and MPS ray intersector with SVGF. **USDKit and USDObjCKit are empty folders.**
- **No OpenUSD source and no Rust USD crate** exist in phi, phikit or ext. phiphx has hand-written USDA and partial USDZ readers and a USDA writer. `usdc-parser` is an empty feature flag.
- Proposal: a new crate `phiray-metal` (macOS only) behind one `RenderEngine` trait with phiray. Metal ray tracing (primitive + instance acceleration structures, intersect from compute) is the core. MetalFX denoise/upscale and MPS SVGF are optional post stages. The input is phiphx `SceneModel`. USD comes in via phiphx, with ModelIO as a Mac-only cross-check.
- Fair comparison: same scene, same camera, same sample counts, same AOVs. Report time to equal noise (RMSE against a long converged reference), not only samples per second.
- On Windows we can build and test the trait, the scene lowering, the benchmark harness, the metrics and the MSL source text. We cannot run any Metal. Everything that touches the GPU needs a Mac.
- Rough size: about 8 stages, ~60-90 engineer-days for a path tracer with parity of the basic passes. See section 5.

---

## 0. What I read

- Headers (all `phi/apple-source/ruskit/Interfaces/MacOSX/`): `Metal/Headers/` (96 files incl. `MTL4*`), `MetalFX/Headers/` (10), `MetalKit/Headers/` (5), `ModelIO/Headers/` (21), `RealityKit/Headers/` (shader headers only) plus `SwiftInterfaces`, `RealityFoundation/SwiftInterfaces`, `SceneKit/Headers/` (41), `USDKit/` and `USDObjCKit/` (both **0 files**). `ShaderGraph/` and `TextureIO/` also empty (not checked beyond the count).
- MPS headers are not under `Interfaces/`. They are at `phi/apple-source/ruskit/MetalPerformanceShaders.framework/Versions/A/Frameworks/MPSRayIntersector.framework/Versions/A/Headers/` (MPSRayIntersector.h, MPSSVGF.h, MPSTemporalAA.h, MPS*AccelerationStructure.h). Listing seen, contents not read by me; API names below come from `phikit/crates/phiray/docs/APPLE.md`.
- iPhoneOS and XROS interface trees exist with the same framework set (Metal, MetalFX, MetalKit, ModelIO, RealityKit, SceneKit). I did not diff them against MacOSX (not checked).
- Bridges: `phi/crates/{macos,realitykit,reality-foundation,kitpro,iosar,xrosar,phi-oml,appkit,phiphx}`, `phikit/crates/apple/src/{macos,metal_performance_shaders,...}`. `rust-kit-pro` is only the package name of `phi/crates/kitpro`; there is no `phi/crates/rust-kit-pro` folder.
- phikit: `phiray/{README.md,ARCHITECTURE.md,jobs.md,Cargo.toml,docs/APPLE.md,src/lib.rs}`, `appkit/gpu/{gpui_macos,gpui_windows,gpui_wgpu}`, `phicore/phiecs/docs/APPLE.md`, `phiphx/src`.
- Searched Cargo.toml files under phi, phikit, ext for `usd|openusd|pxr`; looked for directories named pxr/OpenUSD/openusd.

## 1. Inventory

Header line numbers are in the file named. "Stub" means types and traits only, no call into Apple.

| Apple framework | Relevant APIs (header:line) | Rust bridge today | Use for the engine |
|---|---|---|---|
| **Metal** (ray tracing) | `MTLPrimitiveAccelerationStructureDescriptor` (MTLAccelerationStructure.h:201), triangle geometry (:244), bounding-box (:315), motion triangle (:368), instance descriptors (:787 default, :820 UserID, :822 type enum incl. motion, indirect), `buildAccelerationStructure` / `refit` / `copyAndCompact` (MTLAccelerationStructureCommandEncoder.h:44,75,180), device queries `supportsRaytracing`, `supportsRaytracingFromRender`, `supportsFunctionPointers`, `supportsPrimitiveMotionBlur`, `accelerationStructureSizesWithDescriptor`, `newAccelerationStructure...` (MTLDevice.h:1253-1306), MTL4 variants (MTL4AccelerationStructure.h:17-317, includes curve geometry) | `phi/crates/macos/src/metal/` (39 files, trait-based, `MTLDevice` is "Simulated GPU"); duplicate at `phikit/crates/apple/src/macos/metal/` (78 files, same shape; `acceleration_structure_encoder.rs` is a trait with `u64` ids). **Stub.** Real: `phikit/crates/appkit/gpu/gpui_macos` uses `metal` 0.33 for UI only (metal_renderer.rs, 1799 lines). | Primary: hardware BVH build, compact, refit; intersect from compute kernels written in MSL. |
| **Metal** (shaders) | `newLibraryWithSource` (MTLDevice.h:783), `MTLIntersectionFunctionTable` (:126) with `setOpaqueTriangleIntersectionFunctionWithSignature` (:150), `MTLVisibleFunctionTable`, `MTLBinaryArchive`, `MTLCounters` (timestamps), `MTLIndirectCommandBuffer`, `MTLHeap`, `MTLResidencySet` | Same stubs. gpui_macos has a `shaders.metal` and `build.rs` (UI shaders; not read in depth) | MSL path tracer kernel; GPU timestamps for the benchmark; binary archive removes the first-use compile cost. |
| **MetalFX** | Temporal denoised scaler: descriptor (MTLFXTemporalDenoisedScaler.h:31, availability macOS 26 at :27), inputs color, depth, motion, diffuse albedo, specular albedo, normal, roughness (:257-299), optional specular hit distance, denoise-strength mask, transparency overlay, reactive mask. Temporal scaler (MTLFXTemporalScaler.h:337), spatial scaler (MTLFXSpatialScaler.h:37), frame interpolator (:MTLFXFrameInterpolator.h), MTL4FX variants | `phi/crates/macos/src/metalfx/` and `phikit/crates/apple/src/macos/metalfx` exist as modules; per `phiray/docs/APPLE.md` they are "transcriptions with nothing inside". **Stub.** | Optional post stage: denoise+upscale of the traced frame. Needs motion vectors and albedo/roughness AOVs that phiray does not emit today (APPLE.md). Compare as its own row, never mixed into the raw engine numbers. |
| **MetalPerformanceShaders** | `MPSRayIntersector` (MPSRayIntersector.h:645), `MPSSVGF` (MPSSVGF.h:109,284,430,541), `MPSTemporalAA`, `MPSTriangle/Instance/PolygonAccelerationStructure` (line numbers per APPLE.md; headers listed on disk, not read) | `phikit/crates/apple/src/metal_performance_shaders/mod.rs`, 78 lines, **stub**. `phi/crates/macos` has no MPS dir. | SVGF is a reference denoiser for the comparison. MPSRayIntersector is deprecated for Metal ray tracing, so do not build on it. |
| **MetalKit** | `MTKView` (MTKView.h), `MTKTextureLoader` (MTKTextureLoader.h), `MTKMesh` (MTKModel.h) | `macos/src/metalkit`, `apple/src/iosar/metalkit/mod.rs` - **stub** (not read in depth) | Texture loading and a preview view on a Mac. Not needed headless. |
| **ModelIO** | `MDLAsset initWithURL` (MDLAsset.h:65-108), `exportAssetToURL` (:119); mesh, submesh, material (`MDLMaterialSemanticBaseColor/Metallic/Roughness/Emission`, MDLMaterial.h:34-130), `MDLCamera`, lights (`MDLPhysicallyPlausibleLight`, `MDLAreaLight`, `MDLLightProbe`, MDLLight.h:66-144), `MDLTransformStack`. `MDLAsset.h` has no "usd" string (grep found none), so USD loading goes through the file-extension importer registry at runtime (not verified) | `macos/src/modelio` - **stub** (not read in depth) | Mac-only USD/USDZ/OBJ/ABC loader and exporter for cross-checking phiphx's own USD reader. |
| **USDKit / USDObjCKit** | **No headers: folders are empty** | `phikit/crates/apple/src/macos/{usdkit,usdobjckit}` directories exist (contents not checked) | Cannot design against these. Treat as unavailable. |
| **RealityFoundation / RealityKit** | `RealityRenderer` class (RealityFoundation.swiftinterface:1907, MainActor entity collection, camera output to `MTLTexture`, tone mapping flag), `PostProcessEffect` with `MTLCommandBuffer` (:14679), `ImageBasedLight` (:1973). RealityKit `.h` files are shader headers only (surface, geometry modifier). | `phi/crates/realitykit` (1444 lines, **wgpu-based replica**, no Apple call), `phi/crates/reality-foundation` (911 lines, ECS replica), `iosar`, `xrosar` (replicas; iosar lists `objc = "0.2"` but I found no use). **Stubs / replicas.** | Optional real-time raster path via `RealityRenderer` rendering to our own `MTLTexture`. Swift API only: needs a Swift shim or a C-callable wrapper. Not a path tracer, so a different comparison class. |
| **SceneKit** | `SCNRenderer rendererWithDevice:` and `renderAtTime:viewport:commandBuffer:passDescriptor:` (SCNRenderer.h:44-57) | `macos/src/scenekit` - **stub** | Cheapest Apple raster baseline: render into our own Metal command buffer. Deprecated by Apple (deprecation not verified in the headers). |
| **QuartzCore / CAMetalLayer** | per APPLE.md: `CAMetalLayer.h:54`, `CAMetalDisplayLink.h:33` | `macos/src/quartzcore` stub | Present path only. |
| **Existing wgpu-on-Metal** | wgpu 29.0.4 has `EXPERIMENTAL_RAY_QUERY` on Metal and `create_blas/create_tlas` (APPLE.md row 11) | phiray requests no features (APPLE.md, `gpu/mod.rs:231-235`) | A third option: hardware ray query inside phiray itself. See 2.6. |

Counts of Rust LOC: macos 41147, kitpro 41262, realitykit 1444, reality-foundation 911 (Get-Content line counts of `.rs` files). Other crates' counts not reported: the scan timed out before they printed.

Other findings:
- `phi/crates/macos` pulls `wgpu = "0.19"`, `image`, `nalgebra`, `tokio`; there is no objc/metal dependency in its Cargo.toml. That confirms "no real FFI" for that crate. Its `metal` feature is `metal = []`.
- `phi/crates/kitpro` (`rust-kit-pro`) depends on `rust-macos` with features `metal, metalkit, quartzcore, modelio`, so it consumes the stubs.
- `phikit/crates/apple/src/lib.rs` exposes: accelerate, appkit, buikit, coreml, gameplay_kit, iosar, macos, metal_performance_shaders, phiphx, puikit, reality-foundation, realitykit, services, suikit, swiftui, uikit, vision_kit, xrosar (directory listing). Whether `apple` is a workspace member was not checked.

## 2. Proposed design: `phiray-metal`

Name: `phiray-metal` is fine. Better is a family: `phiray-core` (trait + shared scene lowering + AOV types), `phiray` (wgpu, existing), `phiray-metal` (Apple). I would not rename phiray now; add the trait crate and let phiray implement it.

### 2.1 One trait

Shape (names tentative; the phiray surface I read is `Ray`, `Gpu`, `Tracer`, `Scene`, `Camera`, `View`, `Mode`, `Mat`, `Light` in `phiray/src/lib.rs`):

```rust
pub trait RenderEngine {
    fn name(&self) -> &str;                          // "phiray-wgpu", "phiray-metal"
    fn caps(&self) -> EngineCaps;                    // hw_rt, denoiser, aovs, max_tris
    fn load(&mut self, scene: &SceneModel, opt: &LoadOpts) -> Result<SceneStats>;
    fn set_camera(&mut self, cam: &Camera);
    fn sample(&mut self, spp: u32) -> SampleTiming;  // GPU time, not wall time
    fn resolve(&mut self, passes: &[Aov]) -> Frame;  // beauty, albedo, normal, depth, motion
    fn reset(&mut self);                             // zero accumulation
}
```

Rules that matter for fairness:
- `sample` returns GPU-timestamp time plus CPU submit time separately. APPLE.md row 6 says phiray's current slice timing is submit time when there is one slice. Fix this first in phiray or the comparison is wrong.
- Both engines write the same linear-light float AOV set (section 3). Denoised output is a separate `Aov::BeautyDenoised`.
- Modes (Solid, Preview, Rendered): compare **Rendered only**. Solid/Preview are raster; `phiray` has them, `phiray-metal` would not initially.

### 2.2 Shared scene input

Options considered:

| Option | For | Against |
|---|---|---|
| phiphx `SceneModel` (`phikit/crates/phiphx/src/scene/`) | Already the lowering source for phiray (`Scene`, `Mat`, `Light`); has Blender data (materials, lamps, rigs, animation); no extra dependency; Windows-testable | Not an interchange format; need to confirm it carries everything (not checked field by field) |
| OpenUSD stage via ModelIO | Real interchange; Apple's own loader | Mac only; ModelIO's USD support limits not verifiable from headers; UsdPreviewSurface mapping is lossy; cannot run on Windows |
| OpenUSD via a Rust reader | Cross-platform | Does not exist in tree (section 4); big |

Recommendation: **`SceneModel` is the engine input; USD is an import/export edge of phiphx, not of the engine.** Both engines then render identical lowered data, so differences come from the engine, not the loader. Add a Mac-only check test: load the same `.usd` through phiphx and through ModelIO, compare triangle counts, bounds and material colours. That tells us how faithful phiphx's USD reader is.

Lowering step shared by both engines (new, in `phiray-core`): flatten `SceneModel` to meshes (indexed triangles, f32 positions/normals/uv), instances (4x3 transform + mesh id + material id), materials (the existing `Mat` record), lights, camera. phiray already does this into `trace::Scene`; the cleanest route is to make `trace::Scene` the neutral type and have `phiray-metal` consume it. That keeps one material model (`Mat`, `MAT_WORDS` = 164 words per `jobs.md`).

Material gap to flag: phiray materials include procedural slots, Mix Shader, Fresnel/Layer Weight nodes (jobs.md Done section). Porting that to MSL means translating `phiray/src/trace/wgsl/*.wgsl` (WGSL to MSL). Tools to try: naga's MSL backend (naga is already a dev-dependency; `wgsl-in` feature only, MSL out would need `msl-out`; not tested). That would give one source of truth for shading. Risk: naga MSL output has no access to Metal ray-tracing intrinsics, so the traversal part must be hand-written MSL and the shading part translated. I have not tried this.

### 2.3 Metal ray tracing core

- Per mesh: `MTLPrimitiveAccelerationStructureDescriptor` with triangle geometry; opaque flag set on geometry where the material is opaque (this is the same win as APPLE.md speedup 1). Build, then `writeCompactedSize` and `copyAndCompact` (MTLAccelerationStructureCommandEncoder.h:144-180).
- Top level: instance acceleration structure from `MTLAccelerationStructureInstanceDescriptor` / `UserID` (MTLAccelerationStructure.h:787,820). UserID carries the material/instance index.
- Animation: refit (`:75`) for deforming meshes, rebuild the instance buffer for rigid movement. That maps to phiray's `reinstance` / `repose`.
- Kernel: compute (not render) with MSL `metal::raytracing` intersector; `supportsRaytracing` is the gate, `supportsFunctionPointers` is only needed if we use intersection function tables, which APPLE.md `PORTS.md 3.5` recommends against. Start with built-in triangle intersection, no custom functions.
- Wavefront vs megakernel: phiray is a megakernel. Start megakernel in MSL for like-for-like. Wavefront is a later experiment.
- Output: `MTLTexture` RGBA32F accumulation plus AOV textures; readback only for benchmarks (never inside the timed region unless the metric asks for it).
- Pipeline caching: `MTLBinaryArchive` so benchmark runs do not measure compile time (phiray's Preview compile is 248 s per APPLE.md; report compile time as its own metric).
- Device selection: `MTLCopyAllDevices` (MTLDevice.h:137); require `supportsRaytracing`. Apple Silicon has unified memory so no upload cost. Intel/AMD Macs are possible but not checked.

### 2.4 Denoising and upscaling (optional stage)

- `MTLFXTemporalDenoisedScaler` is the closest fit: it takes color, depth, motion, diffuse albedo, specular albedo, normal, roughness. Availability is macOS 26 / iOS 26 for the protocol (MTLFXTemporalDenoisedScaler.h:211 and :27 say 26.0 and 18.0 for the descriptor; the two lines disagree in the header, so check the SDK actually installed). Needs a Mac on a recent OS.
- Needs new AOVs from our tracer: motion vectors, specular albedo, roughness. phiray lacks motion and specular planes (APPLE.md row on TemporalDenoisedScaler). Add them to both engines so the AOV set stays identical.
- Fallback / reference: MPS SVGF (`MPSSVGF.h`) available on older systems; phiray's own a-trous denoiser (`kernels/denoise.wgsl`) is the apples-to-apples spatial baseline.
- Rule: report "raw" and "denoised" separately. MetalFX is a temporal, reprojecting filter; comparing it with a single-frame spatial filter is not a like-for-like quality test and must be labelled.

### 2.5 Optional real-time raster path

- `SCNRenderer renderAtTime:viewport:commandBuffer:passDescriptor:` (SCNRenderer.h:57) is the least-work Apple raster baseline: it draws into our own command buffer and texture. SceneKit needs an `SCNScene`, so a conversion from `SceneModel` is needed (ModelIO can bridge: `SCNScene(mdlAsset:)` is a known API but not in the headers I read; not checked).
- `RealityRenderer` (Swift) renders RealityKit entities to an `MTLTexture`. It needs a Swift shim; Rust cannot call it directly. Low priority: it is a different class of engine (raster, not path traced) and adds a Swift build step.
- Keep this out of the first milestone; it only matters to show that phiray's Solid/Preview modes compare to Apple's raster stack.

### 2.6 Alternative: phiray with hardware ray query in wgpu

APPLE.md row 11: wgpu 29.0.4 exposes `EXPERIMENTAL_RAY_QUERY` on Vulkan, DX12 and Metal, with `create_blas` / `create_tlas`. That would give hardware ray tracing from **one** WGSL kernel family on Windows and Mac, with no FFI. Cost 10-20 days, high risk (experimental API). Worth keeping as the "engine B-prime" in the benchmark: phiray-wgpu (software BVH), phiray-wgpu-rq (hardware ray query), phiray-metal (native). This also lets Windows test the rq path on an RT-capable GPU (the baseline GPU is a GTX 1660 Ti per APPLE.md, which has no RT cores; rq may fall back or fail; not checked).

### 2.7 FFI choice

Bridges to build on, in order of preference:
1. The `metal` 0.33 crate (already a workspace dependency, used by gpui_macos). Check that it exposes acceleration-structure descriptors and `MTLIntersectionFunctionTable`; I did not check, and `metal` is in maintenance mode.
2. `objc2-metal` and `objc2-metal-fx` (the `objc2` 0.6 family is already in the workspace for AppKit/Foundation). Newer, has full header coverage, generated from the SDK. Not present in Cargo.toml today (not checked in lockfile).
3. Hand-written `msg_send!` for any gaps.

Do **not** extend `phi/crates/macos` or `phikit/crates/apple/src/macos/metal`: these are trait-only mock APIs ("Simulated GPU") and would need a rewrite to be real. APPLE.md already says "Port from the metal or metalfx transcriptions" is on the not-to-do list.

## 3. Benchmark plan

Goal: answer "which engine gets to a given image quality sooner on the same scene" and "how fast is each stage".

**Fixed inputs**
- Scene set (from `SceneModel`, same lowered data for both): (a) 12-triangle cube (the 18 ms/sample reference in APPLE.md), (b) a mid scene with ~1M triangles and a few materials, (c) the heavy "pose 26" character scene (APPLE.md: ~4 s/sample), (d) one scene with many instances, (e) one with glass/emission/procedural materials to exercise shading. Files stored under a benchmark dir with hashes.
- Camera, resolution (1920x1080 and 960x540), max bounces, light settings, seed policy, pixel filter, same sky/IBL. Fixed RNG seeds per pixel where both engines allow; otherwise run N seeds.
- Same output passes: beauty (linear f32), albedo, normal, depth, instance id; motion only if both have it.

**Reference**
- A converged image per scene (e.g. 16k spp, run on phiray, averaged over two seeds). Verify that the two engines agree on the mean before using it: compare mean luminance and per-pixel bias at high spp; a large bias means a shading difference, not noise, and must be fixed first.

**Metrics**
1. Setup: scene lowering time, BVH build time, peak GPU memory, shader compile time (cold, warm-cache).
2. Throughput: Mrays/s for primary, shadow and path rays; ms per sample (GPU timestamps; hard sync like `phiray/examples/slice_cost.rs`); report median and p95 of ≥30 samples after 5 warm-up samples.
3. Quality at equal samples: RMSE, relative MSE (RelMSE), and SSIM / FLIP against the reference at 1, 4, 16, 64, 256, 1024 spp.
4. **Time to equal noise**: the wall time (GPU time + required readback if shown) to reach a target RelMSE (e.g. 0.05, 0.02, 0.01). This is the headline number. Plot RelMSE vs time for each engine.
5. Interactive: time to first frame after a camera move; refit time for an animated frame.
6. Denoised row: raw vs denoised for each of: phiray a-trous, MPS SVGF, MetalFX denoised scaler (Mac only). Report time cost and quality (RMSE vs reference) for each.

**Hygiene**
- Same machine for the Metal vs wgpu comparison: run phiray's wgpu on the **Mac too** (wgpu has a Metal backend), so the Mac result has both engines on one GPU. Windows numbers (DX12/Vulkan on the GTX 1660 Ti) are an extra row; never mix Windows and Mac numbers on one plot.
- Pin power state, no other GPU load, record GPU model, OS, driver, wgpu and Rust versions, git commit.
- Do not count image readback or UI present in the engine time; report them separately (APPLE.md says phiray's display path costs three CPU copies).
- Output a single JSON per run plus a CSV summary; harness lives in `phiray-core` (builds on Windows).
- Honest caveat to record: phiray's current numbers in APPLE.md are all from code comments, none measured. The first benchmark task is to measure phiray alone.

## 4. OpenUSD: what exists

| Item | Finding |
|---|---|
| OpenUSD C++ source (`pxr/`, `OpenUSD`) | **None found.** No directory named pxr/OpenUSD/openusd/usd-rs/usd-core under the scanned tree (search ran under a 90 s job limit; completed without hits, but a slow tree could hide results: partial). |
| Rust USD crate in any Cargo.toml (phi, phikit, ext) | **None.** Only feature flags: `phi/crates/phiphx/Cargo.toml:44 usd-support = []`, `phikit/crates/phiber/Cargo.toml:77 usd-import = []`, `:78 usdz-import = ["phiphx/usdz-import"]`, `phikit/crates/phiphx/Cargo.toml:63 usdz-import = ["dep:zip"]`, `:64 usdc-parser = []` (empty). |
| USDA/USDZ reader | `phi/crates/phiphx/src/importers/usd.rs` (1046 lines): header says "Full USD support requires the OpenUSD library (C++). This module provides a basic implementation" (USDA text, simple USDZ zip, Mesh and Points prims). Detects `PXR-USDC` magic but the `usdc-parser` feature is empty; actual USDC decoding not checked. |
| USDA writer | `phi/crates/phiphx/src/exporters/usd_native.rs` (436 lines), `UsdaWriter`, plain text. |
| Newer phiphx | `phikit/crates/phiphx/src/` has `blend/`, `scene/`, `import.rs`, `export.rs`, `phx.rs`; `SceneModel` is used in `lib.rs`, `phx.rs`, `scene/{build,collection}.rs`. Its USD importer: not found in the file listing I printed (first 80 rows of a longer list), not checked. |
| Apple's USD | USDKit and USDObjCKit header folders are empty. ModelIO can import USD at runtime on macOS; confirmed only that `MDLAsset initWithURL` exists. |

Consequence: OpenUSD as the interchange for **real applications** needs a decision, not a build task:
1. Keep phiphx's own reader and grow it (USDA done, USDC compressed "crate" binary format is the big missing piece, plus composition: references, payloads, variants, layers). USD composition is the hard part and is not in a basic reader.
2. Link OpenUSD C++ through a thin C ABI wrapper (new, heavy build, works on both OS; Windows build of OpenUSD is possible but large).
3. On Mac only, use ModelIO (limited to what Apple's importer supports).
I recommend 2 for "real applications" if full composition matters, 1 for the demo scenes. Size in section 5.

## 5. Constraints: Windows vs Mac

This machine is Windows (Metal frameworks do not exist here). `metal`/`objc2` crates compile only for `cfg(target_os = "macos"/"ios")`, so every Apple-calling line must sit behind that cfg or the workspace stops building on Windows. Note `gpui_macos` already follows this pattern (`[target.'cfg(target_os = "macos")'.dependencies]`).

| Can be done and tested on Windows | Needs a Mac |
|---|---|
| `RenderEngine` trait, `EngineCaps`, AOV types, neutral scene lowering (`SceneModel` to flat meshes/instances/materials) with unit tests | Creating any `MTLDevice`, buffer, acceleration structure, pipeline |
| Making phiray implement the trait; running the **phiray-wgpu** side of the benchmark on DX12/Vulkan | Running `phiray-metal` at all; MetalFX; MPS; SceneKit; RealityKit |
| Benchmark harness, metrics (RMSE, RelMSE, SSIM, FLIP), reference-image tools, JSON/CSV output, plots | Any timing of Metal; GPU timestamp counters |
| Writing the MSL source and checking it parses where tools allow: naga cannot read MSL, so MSL compile errors only show on a Mac (`xcrun metal`) | Compiling `.metal` to `.metallib` (needs Xcode tools) |
| Cross-compile `cargo check --target aarch64-apple-darwin` for Rust type errors only: needs the target installed and, for crates with C/ObjC build scripts, a macOS SDK, so it may not be fully possible on Windows (not tried) | Linking and running |
| phiphx USD reader improvements; USD fixtures; comparing against golden files | ModelIO cross-check; Apple's USD import |
| Hardware ray query experiment via wgpu on an RT-capable Windows GPU (the GTX 1660 Ti listed in APPLE.md has no RT cores; unknown) | wgpu-on-Metal ray query numbers |
| CI config (GitHub Actions `macos-14` runner for build and a unit-test run; GPU tests may not work on hosted Mac runners because of virtualized GPU, not checked) | Real benchmark numbers need physical Apple Silicon hardware |

Other constraints:
- OS version: MetalFX denoised scaler needs macOS 26 per the header; older Macs fall back to SVGF or the a-trous filter.
- `metal` crate 0.33 is in maintenance mode; `objc2-metal` is the long-term path. Pick one before writing code.
- License: the workspace is MIT/Apache; the Apple headers on disk are SDK headers for reference, not to be copied into published code (licensing not checked).
- phiray memory model: binding limit forces 2 or 4 chunks (APPLE.md). Metal has no such limit; scene size caps differ between engines, so the benchmark scenes must fit both.

## 6. Staged missing list with rough sizes

Sizes are engineer-days for one person who knows the code; they are estimates, not measurements.

| # | Stage | Content | Days | Windows? |
|---|---|---|---|---|
| 0 | Measure phiray alone | Fix slice timing and add timestamps (APPLE.md row 6); first baseline numbers on cube and pose 26 | 3-5 | Yes |
| 1 | `phiray-core` trait + neutral scene + AOV types | Extract from `trace::Scene`; phiray implements it | 5-8 | Yes |
| 2 | Benchmark harness | Scenes, reference image, metrics, runner, JSON | 5-8 | Yes |
| 3 | Mac bring-up | `phiray-metal` crate skeleton with `cfg(target_os="macos")`, device selection, command queue, texture/buffers, `.metal` build step, binary archive | 4-6 | No (build gate only) |
| 4 | Acceleration structures | Per-mesh BLAS with compaction, TLAS from instances, refit path, residency | 6-10 | No |
| 5 | MSL path tracer, basic | Camera rays, diffuse/glossy/emission/dielectric, lights, sky, RR, accumulation, AOVs; match phiray's `Mat` record | 15-25 | No |
| 6 | Shading parity | Translate procedural/mix/Fresnel nodes (WGSL to MSL; try naga msl-out for the shading half), textures/atlas, pixel filter, adaptive sampling | 15-25 | Partly (translation can be drafted) |
| 7 | Denoise/upscale | Add motion, specular albedo, roughness AOVs to both engines; MetalFX denoised scaler wrapper; MPS SVGF wrapper; compare with a-trous | 8-12 | AOV part yes |
| 8 | Scene input via USD | Grow phiphx USD reader (USDC binary, references/payloads/variants) or wrap OpenUSD C++ via C ABI; ModelIO cross-check on Mac | 10-20 reader, or 15-30 for OpenUSD wrapper including its build | Reader yes; wrapper partly |
| 9 | Raster baseline (optional) | SceneKit renderer into our command buffer, `SceneModel` to `SCNScene` conversion; RealityRenderer via Swift shim | 5-10 | No |
| 10 | wgpu ray-query variant (optional) | Hardware ray query behind a feature in phiray (APPLE.md row 11) | 10-20 | Yes, if RT hardware |
| 11 | Polish | Viewport integration in phiber (a second engine in the Rendered mode picker), docs, CI | 5-8 | Partly |

Totals: stages 0-8 roughly 75-125 days (my range, about 3 to 5 months for one person); stage 5 and 6 are the bulk. A first useful milestone is stages 0-5 plus a minimal harness: about 40-65 days, giving a cube/Cornell-style comparison before any material parity.

## 7. Risks and open questions

- Matching two path tracers closely enough that RelMSE comparisons are valid is the main quality risk; verify mean-image agreement at high spp first.
- WGSL to MSL: one shading source would remove drift, but traversal differs; decide after a one-day spike with naga's MSL backend (not tried).
- `metal` vs `objc2-metal` choice not yet made; whether `metal` 0.33 exposes the ray tracing descriptors is unchecked.
- USD scope (composition, USDC) is a product decision for the maintainer; the engine does not depend on it.
- Contents of the three duplicated Apple trees (`phi/crates/macos`, `phikit/crates/apple`, `phi/apple-source`) overlap; I did not diff the two Rust copies.
- Work ledger note: the `work_brief` tool call failed (session identity not verifiable), so I worked from the task text alone.

## 8. Files cited

Headers: `Metal/Headers/{MTLAccelerationStructure.h, MTLAccelerationStructureCommandEncoder.h, MTLAccelerationStructureTypes.h, MTLDevice.h, MTLIntersectionFunctionTable.h, MTL4AccelerationStructure.h}`, `MetalFX/Headers/{MTLFXTemporalDenoisedScaler.h, MTLFXTemporalScaler.h, MTLFXSpatialScaler.h}`, `ModelIO/Headers/{MDLAsset.h, MDLMaterial.h, MDLLight.h}`, `RealityKit/Headers/{RealityKit.h, RealityKitSurfaceShader.h}`, `RealityFoundation/SwiftInterfaces/arm64e-apple-macos.swiftinterface`, `SceneKit/Headers/SCNRenderer.h`.
Rust: `phi/crates/macos/{Cargo.toml, src/metal/{device.rs, acceleration_structure.rs, ARCHITECTURE.md}}`, `phi/crates/realitykit/{Cargo.toml, src/rendering.rs}`, `phi/crates/phiphx/src/{importers/usd.rs, exporters/usd_native.rs}`, `phikit/crates/apple/src/macos/metal/{device.rs, acceleration_structure_encoder.rs}`, `phikit/crates/apple/src/metal_performance_shaders/mod.rs`, `phikit/crates/appkit/gpu/gpui_macos/{Cargo.toml, src/metal_renderer.rs}`, `phikit/Cargo.toml`, `phikit/crates/phiray/{README.md, ARCHITECTURE.md, jobs.md, Cargo.toml, src/lib.rs, docs/APPLE.md}`, `phikit/crates/phicore/phiecs/docs/APPLE.md`.
Not read: MPS header bodies, `metal_atlas.rs`/`shaders.metal` in gpui_macos, `phi/crates/{kitpro,iosar,xrosar,phi-oml,appkit}` source beyond Cargo and listing, `phikit/crates/phiphx` USD code (if any), `phikit/crates/phiber` rendered view, `ext/REFS/apple`, `ext/phiber-refs/apple`.
