# Cycles vs phiray: what we have, what we lack

## Summary

- phiray's Rendered mode is a real GPU path tracer (wgpu compute megakernel, SAH BVH, GGX VNDF, glass, MIS, env importance sampling, adaptive sampling, a-trous denoise, Filmic/AgX). It is not a Cycles clone and has no node-graph evaluator.
- Materials are a fixed 164-word record (`trace/mat.rs`). phiphx bakes Blender shader graphs into it (`phiphx/src/blend/shade/`). Anything outside the record is approximated or dropped.
- Absent entirely (grep of `phiray/src/trace` found nothing; docs agree): volumes, hair/curves, motion blur, true displacement, light tree, random-walk SSS, light linking, shadow catcher, holdout, Cryptomatte, light groups, AOVs, baking, panoramic cameras, OIDN/OptiX, spectral/other BSDFs.
- phiber calls Cycles and EEVEE "names only": both are path-traced by phiray (`phiber/issues.md` section 3.2). Any parity claim today means "phiray parity".
- Stage 1 (a typical Blender scene looks like Cycles): a GPU material graph evaluator, Principled v2 completeness, volumes (homogeneous), real SSS, passes/AOVs/shadow catcher, panoramic cameras, hair, light tree. Rough total 110 to 170 engineer-days.
- Biggest structural risks: megakernel + WGSL limits (no f64, no recursion, binding limits), no OSL/SVM-like dispatch, CPU readback every frame, 128 MB binding cap.
- Nothing was built or run. All statements come from reading source and docs. Items marked "not checked" were not verified.

Cycles reference: Blender v5.2.0, `ext/phiber-refs/blender/blender/intern/cycles` (abbrev. `CY/`). phiray: `phikit/crates/phiray` (abbrev. `PR/`). phiber: `phikit/crates/phiber`. Day estimates are my rough sizing for one engineer who knows the codebase (the same convention as `PR/docs/ROADMAP.md`); they are not measured.

## Method and evidence read

- Cycles: directory listings of `kernel/{closure,svm,light,integrator,geom,film,bvh,sample,bake,osl,camera,device}`, `scene/`, `integrator/`, `bvh/`, `session/`, `subd/`, `device/`. I did not read Cycles function bodies; capabilities are inferred from file names.
- phiray: `README.md`, `ARCHITECTURE.md`, `jobs.md`, `docs/ROADMAP.md` (dated 2026-10-26, its section 7 lists drift), `docs/frame.md`, `docs/PORTS.md` (head), `reference/warp/ANALYSIS.md` (head), file list of `src/`, and targeted reads/greps of `trace/wgsl/{shade,light,camera,view,procedural}.wgsl`, `trace/{mat,kernel,mod}.rs`.
- phiber: `issues.md` (section 3), `jobs.md` and `CONSISTENCY_PLAN.md` (grep only), file list of `view/draw/canvas/view3d/rendered/`.
- phiphx (the asset side that bakes materials): `src/blend/shade/README.md`, `src/sky/*.rs` heads, `blend/shade/jobs.md` open items.
- Ignored as asked: `phiray/reference/*` except `warp/ANALYSIS.md`.
- Where docs and code disagree I trust code. Known drift: `ROADMAP.md` says `LIGHTS = 32`; `trace/kernel.rs:493` says `LIGHTS: usize = 64` and `camera.wgsl` has `lights: array<Light, 64>`.

## Architecture contrast (why gaps are not just "add a feature")

| Aspect | Cycles | phiray |
|---|---|---|
| Integrator shape | Wavefront on GPU (`CY/kernel/integrator/state*.h`, `path_trace_work_gpu.cpp`) plus megakernel on CPU (`megakernel.h`) | One megakernel dispatch per sample (`PR/src/trace/wgsl/shade.wgsl`, `light.wgsl`), sliced for the GPU watchdog (`Tracer::sample_in`, `trace/mod.rs`) |
| Materials | Shader graph compiled to SVM bytecode or OSL (`CY/scene/svm.cpp`, `kernel/svm/svm.h`, `kernel/osl/`) | Fixed record `Mat` + 3 procedural slots of 44 words (`trace/mat.rs`, `trace/proc.rs`, `wgsl/procedural.wgsl`) |
| Sampling | Sobol-Burley / tabulated Sobol (`kernel/sample/sobol_burley.h`, `tabulated_sobol.h`) | PCG-style hash `rnd(&seed)` (`wgsl/hash.wgsl`; exact sampler per bounce not checked) |
| BVH | BVH2 / Embree / OptiX / HIPRT / Metal (`CY/bvh/`) | One SAH tree per mesh + top tree (`PR/src/bvh/sah.rs`, `trace/trees.rs`, `trace/instance.rs`), software traversal with 32-slot stack (`wgsl/bvh.wgsl`) |
| Devices | CPU, CUDA, OptiX, HIP, HIPRT, Metal, oneAPI, multi (`CY/device/*`) | wgpu: Vulkan, Metal, DX12, GL (`PR/src/gpu/adapter.rs`); no CPU path tracer (CPU twins exist for BVH, skin, kernels only) |
| Scene limits | Out-of-core, device-memory management | Per-binding cap; ~7.2 to 7.6 M triangles render, ~20 M refused on 128 MB binding (`jobs.md`, `trace/scene.rs`) |

## Feature table

Status: done = feature present and tested against Blender values in some form; partial = approximated or limited subset; missing = nothing found.

### Integrator and sampling

| Area | Cycles | phiray today | Status | Notes |
|---|---|---|---|---|
| Path tracing, bounce limits | `kernel/integrator/shade_surface.h`, `path_state.h`, `intersect_closest.h` | `trace/wgsl/shade.wgsl` (loop, total/transparent/transmission/glossy depth from `p.depth`, lines ~107-125), `light.wgsl` `indirect`/`bounce_light` | partial | Per-kind bounce limits, caustics flags, sample clamp direct/indirect, pixel filter (Box/Gauss/Blackman-Harris) present (`shade.wgsl`, `light.wgsl:234`). Diffuse bounce uses cosine rays with one-lobe selection by probability (`shade.wgsl` ~257-322), not full closure stack. Volume bounces and SSS absent. |
| MIS | `kernel/sample/mis.h`, `light/sample.h` | `light.wgsl` `direct`, `glow_mis`, `env_query`, `ggx_weight`; `tests/emitters.rs` | partial | Power heuristic between emitter/env sampling and diffuse/glossy. Tested against closed-form irradiance (`jobs.md`). BSDF/light MIS for every lamp type not verified. |
| Light sampling / light tree | `kernel/light/tree.h`, `scene/light_tree.cpp`, `distribution.h` | `trace/lamps.rs` (power CDF over mesh emitters), `light.wgsl` `lamp_chance`/`lamp_reach`; up to `LIGHTS = 64` file lights (`trace/kernel.rs:493`) | partial | One lamp picked by power, plus one emissive-triangle and one env sample per shading point. No importance by distance/orientation. ROADMAP sizes the light tree at 8-12 d. |
| Path guiding | `kernel/integrator/guiding.h` (OpenPGL) | none | missing | CPU-only in Cycles (requires OpenPGL). Not needed for parity of look; low priority. |
| Adaptive sampling | `integrator/adaptive_sampling.cpp`, `kernel/film/adaptive_sampling.h` | `trace/wgsl/adapt.wgsl`, `gather.wgsl`; `Tracer::set_adaptive`, `unconverged` | done | Port of Cycles' convergence test per ROADMAP. Actual numerical equivalence not checked. |
| Denoising: OIDN / OptiX | `integrator/denoiser_oidn*.cpp`, `denoiser_optix.cpp`, `kernel/film/denoising_passes.h` | `kernels/denoise.rs` + `denoise.wgsl` (edge-avoiding a-trous, albedo/normal/depth guides), `tests/denoise.rs` | partial | Not ML. ROADMAP suggests OIDN via its C API on CPU (3-5 d). |
| Tiling / progressive | `integrator/path_trace*.cpp`, `render_scheduler.cpp`, `work_tile_scheduler.cpp`, `session/` | `Tracer::sample_in` (slices), phiber `rendered/{stepwise,still,bands,slices,pace}.rs` | done | Progressive with bands for 4K, pause/resume/cancel. Not tile-by-tile but works. Image read back to CPU each frame (`phiber/issues.md` 3.4). |
| Sample pattern / seeds | `kernel/sample/{sobol_burley,tabulated_sobol,pattern,lcg}.h` | hash RNG; `trace/blue_noise.rs` (82 KB table, EEVEE's) | partial | Used for viewport modes; Rendered-mode decorrelation not checked. Seed/scrambling-distance options absent. |
| Branched / light-path features | `kernel/integrator/mnee.h`, `intersect_mnee.h` (manifold NEE) | none | missing | MNEE (caustics from glass) absent; phiray offers caustic toggles only (`light.wgsl:234-235`). |

### BSDFs and closures

| Area | Cycles | phiray today | Status | Notes |
|---|---|---|---|---|
| Principled BSDF v2 | `kernel/closure/bsdf.h`, `kernel/svm/closure.h` | `trace/mat.rs` fields: base colour, rough, metal, trans, ior, emit, sheen, coat, aniso, sss; `light.wgsl`/`shade.wgsl` | partial | Missing or unverified: thin film, coat tint/IOR/normal, specular tint, diffuse roughness (Oren-Nayar/Burley), transmission roughness separate from specular, multi-scatter GGX energy compensation, Dielectric/metal F82 tint. Not exhaustive; Principled v2 per-lobe check not done. |
| Diffuse (Lambert / Burley / Oren-Nayar) | `closure/bsdf_diffuse.h`, `bsdf_burley.h`, `bsdf_oren_nayar.h` | cosine-weighted Lambert (`light.wgsl` `cosine_dir`) | partial | Lambert only (checked). |
| Glossy / GGX / multi-GGX | `closure/bsdf_microfacet.h` | `light.wgsl` `ggx_lambda`, `ggx_sample_vndf`, `ggx_weight`, `ggx_pdf`, `env_brdf` | partial | VNDF sampling present. Beckmann, Ashikhmin-Shirley (`bsdf_ashikhmin_shirley.h`), multi-scatter GGX not found. Anisotropy exists (`aniso`, `aniso_rot`); path-traced use not checked. |
| Sheen | `closure/bsdf_sheen.h` | `light.wgsl` `charlie` | partial | Charlie lobe; Cycles v5 uses Microfacet sheen (exactness not checked). |
| Coat | clearcoat within Principled | `under_layer`, `coat`, `coat_rough` (`light.wgsl:173`) | partial | Layering approximation; `tests/layering.rs`. |
| Subsurface | `closure/bssrdf.h`, `integrator/subsurface_random_walk.h`, `subsurface_disk.h`, `intersect_subsurface.h` | wrap/bleed approximation: `scatter_bleed`, `scatter_tint` (`light.wgsl:125-137`) | partial | No transport. ROADMAP: 8-12 d. |
| Transmission / glass | `closure/bsdf_microfacet.h` (refraction), `kernel/integrator/shade_surface.h` | glass with IOR Fresnel, tint, up to 8 events (`jobs.md`, `shade.wgsl`) | partial | Smooth glass. Rough glass (GGX transmission) and thin-walled mode not verified. No dispersion. |
| Transparent / alpha | `closure/bsdf_transparent.h` | alpha cut-out by stochastic test (`shade.wgsl` ~213-227) | done | Counts toward `transparent_bounce`. |
| Emission | `closure/emissive.h` | `Mat::emit`, emission textures, emitter CDF | done | Emission strength textures supported (`jobs.md` maps). |
| Holdout | `closure/*` + `film` alpha | none (grep) | missing | Needed for compositing; small. |
| Hair (Chiang, Huang) | `closure/bsdf_principled_hair_chiang.h`, `bsdf_principled_hair_huang.h`, `bsdf_hair.h` | none | missing | Depends on the curve primitive (see Geometry). |
| Toon | `closure/bsdf_toon.h` | none | missing | Small once a closure stack exists. |
| Ray portal | `closure/bsdf_ray_portal.h` | none | missing | Rare. |
| Other (diffuse ramp, phong ramp, velvet) | `bsdf_diffuse_ramp.h`, `bsdf_phong_ramp.h`, `bsdf_ashikhmin_velvet.h` | none | missing | Legacy / OSL-only; low priority. |
| Closure mixing | `kernel/svm/closure.h`, `alloc.h` | `Mat::second` + `mix` (Mix Shader, one partner, chosen stochastically) | partial | Two-closure max; Add Shader and deep mix trees are folded in phiphx (`blend/shade/surface.rs`). |

### Volumes

| Area | Cycles | phiray today | Status | Notes |
|---|---|---|---|---|
| Homogeneous volumes | `closure/volume*.h`, `integrator/shade_volume.h`, `intersect_volume_stack.h`, `volume_stack.h` | none | missing | ROADMAP: 8-12 d. |
| Heterogeneous / procedural density | `shade_volume.h`, `geom/volume.h` | none | missing | Needs node-graph evaluation along a ray. |
| OpenVDB | `scene/image_vdb.cpp`, `kernel/bvh/volume.h` | none | missing | ROADMAP: 20+ d. |
| Phase functions | `volume_henyey_greenstein.h`, `volume_rayleigh.h`, `volume_draine.h`, `volume_fournier_forand.h` | none | missing | Small each. |
| Null scattering / delta tracking | `shade_volume.h` | none | missing | Part of volume integrator. |
| World volume, volume stack | `volume_stack.h`, `scene/volume.cpp` | none | missing | |

### Lights and world

| Area | Cycles | phiray today | Status | Notes |
|---|---|---|---|---|
| Point / spot / sun | `kernel/light/{point,spot,sun}.h` | `trace/kernel.rs` `Light` (`kind` 0 sun, 1 point, 2 spot, 3 area), `light.wgsl` `spot_mask`, `lamp_query`; `tests/lamp_power.rs` | done | Soft-shadow radius/angle supported. Spot blend present. |
| Area (rect/disk/ellipse) | `kernel/light/area.h` | `Light.shape`, `AREA_DISK`, `AREA_ELLIPSE` (`camera.wgsl`), `area_query` | done | Spread and normalised-power details not checked. |
| Mesh (emissive) lights | `kernel/light/triangle.h` | `light.wgsl` `glow_query`, `glow_mis`; `trace/lamps.rs` | done | Power CDF. |
| World / HDRI importance sampling | `kernel/light/background.h`, `scene/background.cpp` | `light.wgsl` `env_cdf*`, `env_sample`, `env_pdf`; `tests/environment.rs` | done | Equirect CDF. Sky Texture is baked to an equirect image (`phiphx/src/sky/mod.rs`). |
| Light linking / shadow linking | `kernel/integrator/shadow_linking.h`, `intersect_dedicated_light.h`, `shade_dedicated_light.h` | none | missing | Per-object light/blocker sets. Mid size (4-6 d). |
| Portals | `scene/light.cpp` (portal lights) | none | missing | Interior-scene noise; 3-5 d. |
| Light groups | `kernel/film/light_passes.h`, `scene/pass.cpp` | none | missing | Needs passes. |
| IES / spot texture | `kernel/svm/ies.h` | none (no IES match in grep) | missing | Small; needs light texture table. |
| Ray-visibility flags (camera/diffuse/glossy/shadow) | `kernel/object` flags | partial: `env_hidden` for world (`camera.wgsl`) | partial | Per-object visibility not checked. |

### Shader nodes and displacement

| Area | Cycles | phiray today | Status | Notes |
|---|---|---|---|---|
| Node evaluation model | `scene/shader_graph.cpp`, `svm.cpp`, `kernel/svm/*.h`, `kernel/osl/` | phiphx bake (`blend/shade/{graph,eval,fold,surface}.rs`) to `Mat` + slots; WGSL slots in `procedural.wgsl` | partial | No general graph on GPU. Mix of two textures on Base Color is carried; a stack of more is not (`blend/shade/jobs.md`). ROADMAP: node interpreter 20-30 d. |
| Image texture | `svm/image.h`, `scene/image*.cpp` | `Atlas`, `wgsl/texture.wgsl` (bilinear, mip chain) | partial | Colour space and UDIM tiles open (`blend/shade/jobs.md`). No tiled/closest/cubic interp options verified. Textures on Rough/Metal/Emission done (`tests/maps.rs`). |
| Noise / Voronoi | `svm/noise.h`, `fractal_noise.h`, `voronoi.h` | `wgsl/noise.wgsl`, `voronoi.wgsl` | partial | Ported; 4D (W) not supported (`blend/shade/jobs.md`). |
| Musgrave / fbm | in `fractal_noise.h` (v4+: Noise Texture fractal types) | within noise (not checked) | not checked | |
| Gabor | `svm/gabor.h` | `wgsl/gabor.wgsl` | done | `PROC_GABOR_*`. |
| Wave / gradient / brick / checker / magic / white noise | `svm/{wave,gradient,brick,checker,magic,white_noise}.h` | `wgsl/pattern.wgsl`, `procedural.wgsl` | done | `PROC_*` slots. |
| Sky (Nishita, Hosek, Preetham) | `svm/sky.h`, `scene/image_sky.cpp` | Nishita single/multi-scatter baked to equirect (`phiphx/src/sky/atmosphere.rs`) | partial | Hosek/Preetham absent (comment in `world.rs` says different model). Baked image, not procedural at hit. |
| IES texture | `svm/ies.h` | none | missing | |
| Math / vector math / map range / clamp / colour ramp | `svm/{math,map_range,clamp,ramp}.h` | all 41 Math ops (`wgsl/math.wgsl`), folded Vector Math (`fold.rs`, `vector.rs`), colour ramp baked to 256 stops | partial | Only when feeding the supported slots; ramps B-spline/cardinal absent. |
| Colour nodes (mix, HSV, gamma, invert, bright/contrast, separate/combine) | `svm/{mix,hsv,gamma,invert,brightness,sepcomb_*}.h` | Base Color Mix via `ramp_blend` (`mix.wgsl`); Separate/Combine (`sepcomb.rs`) | partial | Folded, not generic. |
| Mapping / tex coord | `svm/{mapping,tex_coord}.h` | Generated/Object/UV only (`blend/shade/jobs.md`) | partial | No Normal/Camera/Window/Reflection coords; Mapping rotation open. |
| Fresnel / Layer Weight | `svm/fresnel.h` | `PROC_FRESNEL`, `PROC_LAYER_*`, `trace/fresnel.rs` | done | Tested by quadrature. |
| Bump / normal map | `svm/bump.h`, `normal.h` | procedural bump slot, tangent-space normal maps (`tests/maps.rs`) | done | Bump from non-procedural height maps: not checked. |
| Displacement (bump-only / true / adaptive subdivision) | `svm/displace.h`, `scene/mesh_displace.cpp`, `mesh_subdivision.cpp`, `subd/*` | none for true displacement; phiphx evaluates Subsurf and Displace *modifiers* (`phiphx/src/blend/modifier`) | missing | Modifier-level displace exists; no shader displacement, no microdisplacement/adaptive subdivision. ROADMAP: 10-15 d. |
| Geometry/attribute/AO/bevel/wireframe/light path/camera nodes | `svm/{geometry,attribute,ao,bevel,wireframe,light_path,camera}.h` | Vertex Color = white (`fold.rs`); rest none | missing | Attribute (vertex colours) is a common need. AO/bevel need extra rays. |
| OSL | `kernel/osl/*` | none | missing | Out of scope. |
| Vector/raycast/radial tiling/scene time nodes | `svm/{raycast,radial_tiling,scene_time}.h` | none | missing | Low priority. |

### Geometry

| Area | Cycles | phiray today | Status | Notes |
|---|---|---|---|---|
| Triangles | `kernel/geom/triangle*.h`, `scene/mesh.cpp` | `trace/scene.rs`, `tri.rs`, `mesh.rs`; watertight test (`bvh.wgsl` `tri_t`, `jobs.md`) | done | 32 B vertices, 16 B faces; smooth shading with split/custom normals. |
| Instancing | `scene/object.cpp`, `geometry_bvh.cpp` | `trace/instance.rs`, `trees.rs`, `pose.rs`; `Scene::place`; `tests/instances.rs` | done | Shared geometry, no instance cap (`ROADMAP.md` 1.2). Particle/geometry-nodes instancing relies on phiphx output (not checked). |
| Curves / hair | `kernel/geom/curve*.h`, `scene/{curves,hair}.cpp` | none | missing | New primitive + BVH leaf type + intersection. ROADMAP: 10-15 d. |
| Point clouds | `kernel/geom/point*.h`, `scene/pointcloud.cpp` | none | missing | Small once a second primitive kind exists. `docs/primitives.md` plans a general seam (not checked). |
| Motion blur | `kernel/geom/motion_*.h`, `motion_curve.h`, `bvh` time handling | none; `pose.rs` is deformation/refit only | missing | ROADMAP: 15+ d. Needs per-ray time and time-sampled transforms. |
| BVH build and traversal | `bvh/{bvh2,embree,optix,hiprt,metal}.cpp`, `kernel/bvh/*` | `bvh/{sah,build,kernels,gpu}.rs`, `wgsl/bvh.wgsl` | partial | Software traversal only; no hardware ray query (`ARCHITECTURE.md` plans `hw`). Unaligned/spatial-split BVH not present. |
| Scene streaming / memory | `session/cache_eviction.cpp`, `scene/image_cache.cpp` | refused above ~2x binding limit (`jobs.md`) | partial | Per-object streaming open. |
| Ptex/UDIM/volume grids | `scene/image_*.cpp` | none | missing | |
| Dicing / subdivision at render | `subd/*` | none | missing | See displacement. |

### Camera

| Area | Cycles | phiray today | Status | Notes |
|---|---|---|---|---|
| Perspective / orthographic | `kernel/camera/camera.h`, `projection.h` | `wgsl/camera.wgsl` (`persp` flag), phiber `render_job.rs camera_of` (sensor fit verified in `issues.md` D) | done | |
| Panoramic / equirect / fisheye / mirror ball | `camera/projection.h` | none (`persp` is a bool-like u32; no panorama match in grep) | missing | Not exhaustively checked; `trace/ibl/panorama.rs` is for environment prefilter, not camera. 3-5 d. |
| DOF (thin lens, bokeh blades, anamorphic) | `camera.h` | `shade.wgsl` `unit_polygon`, `lens_offset`; `aperture`, `blades`, `blade_spin`, `aperture_ratio`; `tests/depth_of_field.rs` | done | Includes bladed bokeh and ratio. |
| Rolling shutter | `camera.h` | none | missing | Needs per-ray time. |
| Camera shift, clip, lens units | `camera.h` | shift/clip handled in phiber camera setup (`issues.md` D, details not checked) | not checked | |

### Film, passes and colour

| Area | Cycles | phiray today | Status | Notes |
|---|---|---|---|---|
| Combined + data passes | `scene/pass.cpp`, `kernel/film/{data_passes,light_passes,write}.h` | radiance, albedo, normal, depth buffers (`Tracer::{radiance,albedo,normal,depth}`) | partial | Missing: diffuse/glossy/transmission direct/indirect/colour, emission, env, AO, shadow, position, UV, object/material index, mist, motion vector. Each 1-2 d per ROADMAP. |
| AOVs | `svm/aov.h`, `film/aov_passes.h` | none | missing | Needs the AOV Output node in the material path. |
| Cryptomatte | `film/cryptomatte_passes.h` | none | missing | ROADMAP: 4-6 d. |
| Light groups | `film/light_passes.h` | none | missing | |
| Shadow catcher | `integrator/shadow_catcher.h` | none | missing | Mid size, depends on holdout and passes. |
| Transparent film / alpha | `film` alpha | `p.film.x` transparent film, black at alpha 0 (`shade.wgsl` ~132) | done | |
| Pixel filter | `scene/film.cpp` | `filter_offset2`, `blackman_harris_at` (`light.wgsl:186-205`) | done | Box/Gauss/B-H. |
| Colour management | OCIO in Blender (not Cycles) | `wgsl/view.wgsl`: Standard/Filmic/AgX sampled from Blender 5.0, 6 Filmic looks | partial | Curves sampled at 64 points. No OCIO config loading, no Khronos PBR Neutral / ACES / custom views, no exposure/gamma beyond what `view.w` carries (not checked). |
| Bloom/glare/denoise-in-compositor | Blender compositor | `glow` pass (`Tracer::glow`) | partial | `phiphx/src/comp` has a compositor tree (882 lines; not checked). |
| Baking | `scene/bake.cpp`, `kernel/bake/bake.h`, `integrator/init_from_bake.h` | none (phiphx `shade/bake.rs` bakes graphs to records, not Cycles bake) | missing | ROADMAP: 10+ d. |

### Devices and platform

| Area | Cycles | phiray today | Status | Notes |
|---|---|---|---|---|
| CPU rendering | `device/cpu`, `kernel/device/cpu/` | none (CPU painter is a Solid fallback; no CPU path tracer) | missing | Probably not needed except as a no-GPU fallback and a reference for tests. |
| CUDA / OptiX / HIP / HIPRT / Metal / oneAPI | `device/{cuda,optix,hip,hiprt,metal,oneapi}` | wgpu backends only (Vulkan/Metal/DX12/GL) via `gpu/adapter.rs` | partial | Portable but no RT cores. wgpu has ray-query as an experimental feature (not checked in this wgpu version). |
| Multi-device | `device/multi` | none | missing | |
| Hydra delegate / render API | `CY/hydra` | none | missing | Not relevant. |

## What phiber adds on top (consumer side)

- `phiber/src/view/draw/canvas/view3d/rendered/` (about 40 files): render thread, scene flatten (`scene.rs`, `build.rs`, `place.rs`), progressive stepping (`stepwise.rs`), bands (`bands.rs`), still render (`still.rs`), lamps (`lamp.rs`).
- `phiber/src/scene/render_{engine,job,phase,quality,color}.rs`: engine names; Cycles and EEVEE both map to phiray (`issues.md` 3.2: "Eevee and Cycles are names only").
- Missing Cycles Properties rows per `issues.md` 3.2: denoiser options, clamping, Fast GI, tile size/memory, light paths, colour management sub-panels (partly exist now; full list in `panel-parity.md`, not read).
- Per-frame CPU readback of the traced image (`issues.md` 3.4). A GPU-to-GPU present path is open (`jobs.md` in phiray: shared D3D texture between wgpu DX12 and gpui DX11).
- `phiber/CONSISTENCY_PLAN.md` item D9 ("Combined pass colours"): flat patchy colour, to chase in phiray once the engine is known (not investigated).

## Prioritised plan of what is missing

Sizing is rough (engineer-days, one engineer who knows the codebase, including CPU twin/tests, before review churn). Sizes marked with an asterisk are taken from `PR/docs/ROADMAP.md` section 3.3; the rest are my estimates.

### Stage 1: render a typical Blender scene like Cycles

| # | Item | Size | Why |
|---|---|---|---|
| 1.1 | General GPU material evaluation (node interpreter or generated WGSL) replacing the fixed `Mat` record | 20-30 d* | The largest fidelity gap. Every scene with more than two stacked textures or unsupported nodes degrades today. Also enables displacement, volumes and AOV nodes. Choose between bytecode interpreter (SVM-like, `CY/kernel/svm/svm.h`) and per-material WGSL codegen with pipeline caching; decide first. |
| 1.2 | Principled BSDF v2 completion: multi-scatter GGX, rough glass transmission, diffuse roughness, specular tint/F82, coat tint/IOR/normal, thin-walled, sheen exactness; one closure stack (diffuse, glossy, transmission, coat, sheen, emission) with proper per-lobe MIS | 12-20 d | Most scenes use Principled; current path picks one lobe and approximates layers. |
| 1.3 | Random-walk subsurface | 8-12 d* | Skin/wax/foliage. Needs volume-like stepping inside objects. |
| 1.4 | Homogeneous volumes (object and world), volume stack, HG phase, absorption/scattering/emission | 8-12 d* | Fog, smoke, god rays. Needs the interior/exterior stack (`intersect_volume_stack.h`). |
| 1.5 | Light tree (many lights) and light pick by importance | 8-12 d* | Scenes with many emissive meshes or lamps are noisy otherwise. |
| 1.6 | Passes and AOVs: diffuse/glossy/transmission direct+indirect+colour, emission, env, AO, shadow, mist, position, UV, index, AOV Output node; Cryptomatte | 12-18 d (1-2 d per pass*, Cryptomatte 4-6 d*) | Compositing workflows; also feeds denoise guides. |
| 1.7 | Holdout, shadow catcher, per-object ray visibility flags | 5-8 d | Compositing against plates. |
| 1.8 | Panoramic cameras (equirect, fisheye, mirror ball), camera nodes | 3-5 d | Standard Cycles feature, cheap. |
| 1.9 | Hair/curves: curve primitive, intersector, Principled hair (Chiang, Huang) | 10-15 d* + 5-8 d for hair BSDF | Characters, grass, fur. |
| 1.10 | OIDN denoise (CPU C API), with albedo/normal passes | 3-5 d* | Quality parity for final renders. |
| 1.11 | True displacement + bump from shader; Attribute and Geometry node support | 10-15 d* | Terrain and detail. Depends on 1.1. |
| 1.12 | Colour management: exposure/gamma, Khronos Neutral, standard views, read the file's view settings; optionally OCIO config loading | 3-6 d | Pixel-level match with Blender output. |
| 1.13 | Verification harness: render the same scenes in Cycles (v5.2) and compare per pass (RMSE, SSIM); no such harness exists | 5-8 d | Without it "same as Cycles" cannot be claimed. Existing tests compare to closed forms and fixtures. |

Stage 1 subtotal: about 110-170 d summing the items above. Treat as order-of-magnitude.

### Stage 2: broaden coverage

| # | Item | Size |
|---|---|---|
| 2.1 | Motion blur (object transform and deformation, camera shutter, rolling shutter) | 15-20 d* |
| 2.2 | Heterogeneous volumes, procedural density, OpenVDB grids, null-scattering delta tracking, world volume | 10-15 d, VDB 20+ d* |
| 2.3 | Light linking, shadow linking, light groups, portals | 8-12 d |
| 2.4 | Adaptive subdivision / microdisplacement | 10-15 d |
| 2.5 | Baking (Cycles bake types and bake to image) | 10+ d* |
| 2.6 | Point clouds and particle-like primitives | 4-6 d |
| 2.7 | Scene streaming / out-of-core for 20 M+ triangles; more face bindings | 8-12 d |
| 2.8 | GPU-resident present (remove CPU readback), keeps interactive frame rate | sizeable, open since `jobs.md` |
| 2.9 | Remaining nodes: IES, Bevel, AO, Wireframe, Raycast, Sky Hosek/Preetham, Radial tiling, 4D textures, UDIM | 10-15 d |
| 2.10 | Toon, ray portal, velvet, other legacy closures | 3-5 d |

### Stage 3: nice to have

| # | Item | Size |
|---|---|---|
| 3.1 | Hardware ray tracing behind a feature (`hw` seam per `ARCHITECTURE.md`) | depends on wgpu support; not checked |
| 3.2 | Path guiding (OpenPGL-like) | large; CPU-only in Cycles |
| 3.3 | MNEE caustics | 5-8 d |
| 3.4 | Spectral/dispersion (Cycles v5 has Wavelength node: `svm/wavelength.h`) | 5-10 d |
| 3.5 | CPU path-tracing fallback or WGSL-on-CPU reference | 10-20 d |
| 3.6 | OSL | not realistic (ROADMAP) |
| 3.7 | Multi-device, network render | not planned |

## Risks and unknowns for the plan

- WGSL limits: no f64, limited pointer use, no recursion, binding and storage-buffer limits (128 MB per binding seen). Material codegen (1.1) and volumes (1.4) both push against shader size and register pressure on a megakernel. Moving to a wavefront integrator (as Cycles' GPU path does, `CY/integrator/path_trace_work_gpu.cpp`) may become necessary; ROADMAP states the megakernel choice was deliberate (`ROADMAP.md` 1.3).
- Test oracle: today's tests compare with closed forms and Blender-derived tables (e.g. Filmic/AgX LUTs sampled from Blender 5.0; see `view.wgsl` header). A Cycles-image diff harness (1.13) is the missing foundation; without it parity cannot be measured.
- Doc drift: `PR/docs/ROADMAP.md` is dated 2026-10-26 and predates parts of the code (e.g. the 64 lights, `sim/`). Re-verify its "missing" claims before sizing. I re-checked only by grep.
- phiphx is the other half: many "missing" items (nodes, attributes, hair data, volumes, displacement) also need phiphx readers (`phiphx/src/blend/shade`, `modifier`, `object`, `scene`).
- Not checked: Cycles function bodies; phiray `light.wgsl` bodies beyond signatures and comments; actual numerical match to Cycles for any feature; `panel-parity.md`; `docs/primitives.md`, `accel.md`, `sampling.md`, `reconstruct.md`; the `comp` compositor; whether the vendored wgpu exposes ray queries; phiray `reference/*` other than `warp/ANALYSIS.md` head.

## Files read

- Cycles: directory listings under `ext/phiber-refs/blender/blender/intern/cycles/`.
- phiray: `README.md`, `ARCHITECTURE.md`, `jobs.md`, `docs/ROADMAP.md`, `docs/frame.md` (head), `docs/PORTS.md` (head), `reference/warp/ANALYSIS.md` (head), `src/trace/mat.rs`, `src/trace/wgsl/{camera,view,shade,light,procedural}.wgsl` (signatures and greps), `src/trace/kernel.rs` (greps), `src/trace/mod.rs` (public fns).
- phiphx: `src/blend/shade/{README.md,jobs.md}`, `src/sky/{mod,atmosphere}.rs` (head), `src/blend/shade/world.rs` (grep).
- phiber: `issues.md` (section 3, grep), `jobs.md` and `CONSISTENCY_PLAN.md` (grep), `view/draw/canvas/view3d/rendered/` (file list).

