# 05 - Test plan, measurements and the gap list before Cycles resumes

Written 2026-11-12 from a read-only pass: nothing was built, run or edited. Every number
below is copied from a file (path given); nothing was re-measured. "doc says" marks a claim
I read in a doc and could not confirm in code or in a recorded measurement.

Notes on the sources (so nobody looks for what is not there):

- `docs\analysis\` did not exist, and so `01..04` and `README.md` there do not exist either. This file creates the folder.
- `panel-parity.md` is at `crates\phiber\src\panel-parity.md`, not in `docs\`.
- `docs\layout-space-full-impl.md` (30 Aug) describes `store/` and `bui/` paths that are gone. Treat it as history.
- `docs\gap-list.md` itself says its four status docs are unaudited and that three of three spot checks elsewhere were stale. The same holds for what I quote from them.
- `docs\animation-rendering-status.md` and `Downloads\animation-rendering-status.md` differ (SHA-256 differs). The docs copy is the later one (carries batches up to 171); the Downloads copy is the batch-101 text of 3 Oct.
- Only 14 `jobs.md` files exist under `src\`; their open `[ ]` items are all listed in section 5(b).

---

## 1. Summary

1. The only complete head-to-head that exists is pose 26 (Blender reference image vs phiray), plus partial pairs for poses 24 and 30 and LAVA poses 01-03. Phiber outputs exist for poses 24, 26, 30 only (CW-POSES\PHIBER-RENDERS). Poses 23, 25, 27, 28, 29, 31 have Blender references and no phiber render.
2. The plan was: (a) time phiber on pose 26 at 64 samples (`BENCH-POSE26.bat`, `BENCH-VARIANCE.bat`), (b) time Cycles on the same file (`BENCH-CYCLES.bat` + `bench_cycles.py`), (c) score the picture against Blender's with `compare_pose.py` (per-region error).
3. Only (a) produced numbers, and the variance run (`phiber_variance_log.txt`) was never produced. (b) never produced a number: `cycles_bench_log.txt` stops at "exit 1" (18 Oct), the direct call to Blender gives "Access is denied" (exit 5, `basketball_log.txt`). So **no Cycles time exists for any scene** except a 6-sample, 360 px CPU check render in `LAVA-POSES\check_show.log` (about 3 min wall, includes load).
4. Phiber time for pose 26, 64 samples, 1440 x 1440 (as `bench_cycles.py` assumes): 253,983 ms (18 Oct baseline), 308,447 ms and 328,444 ms for the two reverted changes. Pose 30 on 12 Nov: 176,920 ms for 64 samples (the last `times.txt`).
5. No per-region error number was saved from `compare_pose.py`; only the compare sheets (`*_compare.png`, ref | phiber | diff x4) exist. The recorded quality numbers are from `rendering.md` / `assessment.md` (white-override figure mean 233.0 vs 226.9; THYMUS mean abs diff 15.6/255 on 5 Oct, 13.0 after the env fix; Sky texture within 1.6 %).
6. The old paths point at `gemphi\phiace`; that folder is now an unrelated robotics crate. Everything rebuilds and runs from `gemphi\phikit`. `phikit\target\release\phiber.exe` is dated 2026-10-31 07:32, older than the 12 Nov source, so rebuild first.
7. Rendering gaps (section 5a): 22 items, biggest are speed on heavy scenes (about 4 s/sample at pose 26), Cycles and EEVEE being names only, 19 Cycles settings listed "IGNORED by Phiray", general shader graphs baked to one fixed record, no volumes, hair, motion blur, light tree, true displacement, fewer passes.
8. Layout gaps (5b): 22 items. Shells: Texture Paint, Scripting, Sequencer, Movie Clip. Partial: Shading and Geometry Nodes, Compositing (per-pixel nodes only), Graph/NLA/Dope Sheet/Drivers editing, Spreadsheet, UV editing, Properties tabs and Cycles rows. No interactive area split/join. About 380 menu entries inert (doc says).
9. Recommended order (section 6): re-baseline and capture numbers first (steps 0-1), then light/world/colour, materials, speed, settings honoured, then the render-path spaces, then the rest of the spaces; Cycles resumes when steps 0-5 hold.

---

## 2. The original test plan(s)

### 2.1 What was meant to be measured

| Plan | Question | Instrument | Scene | Reference |
|---|---|---|---|---|
| BVH benchmark | Does a traversal change make pose 26 faster at 64 samples? | `BENCH-POSE26.bat` -> `times.txt` column 3 (drew_ms), compared with the baseline 253,983 ms printed into the log | pose 26 | self (baseline) |
| Variance | How noisy is one timing, before judging any change? 3 runs, 90 s cool-down each | `BENCH-VARIANCE.bat` -> `phiber_variance_log.txt` (never produced) | pose 26 | self |
| Cycles speed | Seconds per sample for Cycles on the same file and GPU, adaptive sampling off, denoise off, 32 samples, 1440 x 1440 | `BENCH-CYCLES.bat` + `bench_cycles.py` -> `cycles_bench_log.txt` (failed) | pose 26 | n/a |
| Render + draft | Does the CLI produce a 64-sample draft and a 2048-sample full render of pose 26? | `RENDER-POSE26.bat`, `phiber_render.ps1`, `phiber_pose26.ps1` (build, `--help`, `scene_sizes`, two renders) | pose 26 | Blender `BS-POSE-26.png` |
| Poses 23-29 | 64-sample draft of each pose | `RENDER-POSES-23-29.bat` -> `phiber_poses_23_29_log.txt` (not present; only pose 24 and 26 outputs exist from it) | poses 23-29 | `BS-POSE-nn.png` |
| Picture score | Per-region mean absolute difference (0-255) vs the reference, over black, and a sheet ref / phiber / diff x4 | `CW-POSES\compare_pose.py` (needs `bpy` + numpy; regions hard-coded for pose 26: floor_near, floor_far, suit_standing, suit_crouching, skin_*, face_standing, glasses_crouching, hair_crouching, bag_*, car_hood, car_flank, grille, headlight, sill, tyre) | pose 24, 26, 30 | `RENDERS-WITH-TRANSPARENT-BG\BS-POSE-nn.png` |
| LAVA bench | s/sample and RMSE vs a phiber 1024-sample image (not Cycles) | `LAVA-POSES\bench\bench.sh`, `bench_show.sh`, `score.py` -> `results.txt`, `results-show.txt` | `FOUR-phantom_gold.blend` camera FOUR-front; `LAVA-SHOW-working.blend` camera SHOW-line | `reference.png`, `reference-show.png` (phiber, 1024 samples) |
| Phiber vs Blender, LAVA | picture compare | `LAVA-RENDERS\*_compare.png` | LAVA poses 01-03 | `blender_pose0n.png` |
| Open time | phiphx vs Blender open time | `docs\blender-head-to-head.md` (`blender_open_time.py`, `examples/bench_open`) | 7 fixtures | Blender 5.0.1 |
| Render-vs-Blender statistics | mean, p50/p95, patches, white-override, edge softness | `examples/render_compare.rs`, `animation_compare.rs`, `tests/truth/*.py` | THYMUS, FLORIS, female01 | Blender 5.0.1 Cycles |

The `times.txt` line is `frame posed_ms drew_ms samples` (`src\cli\cost.rs:14-17`; the code comment calls column 3 `render_ms` in some places, same column). `posed_ms` is scene open + pose; `drew_ms` is the trace.

### 2.2 CLI flags the scripts use

| Flag | Meaning (from `src\cli\set.rs`, `mod.rs`) |
|---|---|
| `-b FILE` | background render of FILE |
| `-f N` | render frame N (also `a..b`, comma lists, +N / -N relative) |
| `-e N` | end frame |
| `--samples N` | sample count; clamped up to the 2048 floor unless `--draft` |
| `--draft` | lifts the 2048 floor (so 64 means 64) |
| `-o PREFIX` | output prefix, Blender style; ends with a separator = directory. `-o ...\pose26_fast64_` writes `pose26_fast64_0001.png` |
| `-F PNG` | format |
| `-E name` | engine |
| `--camera A,B,..` | render through named camera(s) from one open scene |
| env `PHIBER_SCALE=n` | divide resolution by n (used in `bench.sh`, so LAVA bench times are at reduced size) |
| env `PHIRAY_DENOISE=0/1` | denoiser off/on |
| env `PHIBER_OVERRIDE=x` | one grey Lambertian material (light measurement) |

### 2.3 Exact command lines, rerun from `C:\Users\phiac\Workspace\gemphi\phikit`

The scripts hard-code `gemphi\phiace`. Replace `WS` and `EXE` as below (the data stays in Downloads). Build once (the 18 Oct build took 9 min 53 s; the first run after a toolchain change can take longer):

```bat
cd /d C:\Users\phiac\Workspace\gemphi\phikit
taskkill /IM phiber.exe /F
cargo build --release -p phiber
set EXE=C:\Users\phiac\Workspace\gemphi\phikit\target\release\phiber.exe
set POSES=C:\Users\phiac\Downloads\phiber\CW-POSES
set OUT=%POSES%\PHIBER-RENDERS
set BLEND=%POSES%\CHATTY-POSES-RENDER-SCENE-03_Duo pose 26_1.blend
```

Timed 64-sample draft (BENCH-POSE26.bat step 2; then read the third column):

```bat
"%EXE%" -b "%BLEND%" -f 1 --samples 64 --draft -o "%OUT%\pose26_fast64_" -F PNG
type "%OUT%\times.txt"
```

Variance (BENCH-VARIANCE.bat; 3 runs, 90 s idle before each; outputs go to `PHIBER-BENCH`):

```bat
for /L %R in (1,1,3) do (timeout /t 90 /nobreak >nul & "%EXE%" -b "%BLEND%" -f 1 --samples 64 --draft -o "%POSES%\PHIBER-BENCH\run%R_" -F PNG & type "%POSES%\PHIBER-BENCH\times.txt")
```

(`%R` doubles to `%%R` inside a .bat file.) Edit `WS` to `...\phikit` in each .bat/.ps1: `BENCH-POSE26.bat`, `BENCH-VARIANCE.bat`, `RENDER-POSES-23-29.bat` (EXE line), `RENDER-POSE26.bat`, `phiber_render.ps1`, `phiber_pose26.ps1`.

Full-quality render (2048 floor; took 29 min 47 s on 18 Oct, 3:01:04 -> 3:30:51 by the log's own stamps):

```bat
"%EXE%" -b "%BLEND%" -f 1 -o "%OUT%\pose26_phiray_" -F PNG
```

Poses 23-29 at 64 samples (RENDER-POSES-23-29.bat; scene names are exact):

```bat
"%EXE%" -b "%POSES%\CHATTY-POSES-RENDER-SCENE-03_Single_pose 23_2.blend" -f 1 --samples 64 --draft -o "%OUT%\pose23_" -F PNG
"%EXE%" -b "%POSES%\CHATTY-POSES-RENDER-SCENE-03_Single pose 24.blend"   -f 1 --samples 64 --draft -o "%OUT%\pose24_" -F PNG
"%EXE%" -b "%POSES%\CHATTY-POSES-RENDER-SCENE-03_Single pose 25_2.blend" -f 1 --samples 64 --draft -o "%OUT%\pose25_" -F PNG
"%EXE%" -b "%POSES%\CHATTY-POSES-RENDER-SCENE-03_Duo pose 26_1.blend"    -f 1 --samples 64 --draft -o "%OUT%\pose26_" -F PNG
"%EXE%" -b "%POSES%\CHATTY-POSES-RENDER-SCENE-03_Duo pose 27_1.blend"    -f 1 --samples 64 --draft -o "%OUT%\pose27_" -F PNG
"%EXE%" -b "%POSES%\CHATTY-POSES-RENDER-SCENE-03_Duo pose 28_1.blend"    -f 1 --samples 64 --draft -o "%OUT%\pose28_" -F PNG
"%EXE%" -b "%POSES%\CHATTY-POSES-RENDER-SCENE-03_Single pose 29_1.blend" -f 1 --samples 64 --draft -o "%OUT%\pose29_" -F PNG
```

(Pose 30 and 31 files also exist: `...Single pose 30.blend`, `...Single pose 31.blend`; pose 30 was run by hand: `fix4_pose30_`.) Note `times.txt` is overwritten by each run; copy it after each call.

Does the scene reach the GPU (phiber_pose26.ps1 step 3):

```bat
cargo run --release -p phiber --example scene_sizes -- "%BLEND%"
```

Score a picture (compare_pose.py; Blender's `bpy` module + numpy; the default regions are pose 26's; add `name=x0,y0,x1,y1` for others):

```bat
py -3.13 -I C:\Users\phiac\Downloads\phiber\CW-POSES\compare_pose.py "%POSES%\RENDERS-WITH-TRANSPARENT-BG\BS-POSE-26.png" "%OUT%\fix4_pose26_0001.png" > "%OUT%\fix4_pose26_score.txt"
```

Cycles time (BENCH-CYCLES.bat; failed on 18 Oct). `blender_where.txt` shows the real executable. Use it by full path, not the WindowsApps alias:

```bat
"C:\Program Files\WindowsApps\BlenderFoundation.Blender_5.2.2.0_x64__ppwjx1n5r4v9t\Blender\blender.exe" --background "%BLEND%" --python C:\Users\phiac\Downloads\phiber\bench_cycles.py
```

This also failed with "Access is denied" (exit 5) from the same WindowsApps folder on 18 Oct (`basketball_log.txt`). The fix is a Blender copy outside `WindowsApps` (portable zip) or running it from a shell the Store app allows. Until then there is no Cycles time. The script itself is sound: 32 samples, adaptive off, denoise off, 1440 x 1440, GPU if OPTIX/CUDA/HIP/ONEAPI exists, prints `PHIBER_BENCH_BEGIN ... total_s= per_sample_s=`.

Other re-runs:

```bat
:: phiphx open time against Blender
blender --background --factory-startup --python crates\phiphx\tests\truth\blender_open_time.py -- -n 3 crates\phiphx\tests\fixtures\*.blend
cargo run --release -p phiphx --example bench_open -- -n 7 crates\phiphx\tests\fixtures\*.blend
:: LAVA bench (git-bash). Reference first, then a label per change
bash LAVA-POSES\bench\bench.sh reference
bash LAVA-POSES\bench\bench.sh mychange
:: animation
cargo run --release -p phiber --example animation_render ...   (see docs\ui-parity-status.md batch 152)
cargo run --release -p phiber --example animation_compare -- BLENDER_DIR PHIBER_DIR contact.png
:: two renders of one view
cargo run --release -p phiber --example render_compare -- blender.png phiber.png side.png
:: truth tests for layout
cargo test --release -p phiber --test workspace_regions_truth --test menu_truth --test channel_name_truth
```

`bench.sh` defaults to `Downloads\phiber\phiber-render.exe` (a 4 Nov copy). `check_poses.sh` already uses `gemphi/phikit/target/main-ui/debug/phiber.exe` (a debug build; do not time with it).

---

## 3. Measurements so far

Columns follow `times.txt`: posed_ms / drew_ms. Resolution is not written by `times.txt`; where I give it, it comes from the script or doc named. "?" = not recorded. Dates are file times unless the log carries one.

### 3.1 Pose 26 (CW-POSES, 1440 x 1440 per `bench_cycles.py` and `render-fix-report.md`)

| Scene | Samples | Res | posed_ms | drew_ms | Cycles | Date | Source |
|---|---|---|---|---|---|---|---|
| pose 26 draft | 64 | 1440 sq (doc) | 6,512 | **253,983** (baseline; log wall 2:55:25 -> 3:01:03 = 338 s) | none | 2026-10-18 | `CW-POSES\phiber_render_log.txt` |
| pose 26 full | 2048 | same | ? | not logged (wall 3:01:04 -> 3:30:51 = 1,787 s; `times.txt` not written, output went into a directory `pose26_phiray_` because of the old `-o` bug) | none | 2026-10-18 | same |
| pose 26, divide->shift | 64 | same | 6,650 | **308,447** (+21 %) | none | 2026-10-18 | `CW-POSES\phiber_bench_log.txt`; `docs\render-fix-report.md` "performance attempt" |
| pose 26, shift + shadow material moved | 64 | same | ? | **328,444** (+29 %) | none | 2026-10-18 | `docs\render-fix-report.md` |
| pose 26 diag | 32 | ? | 7,962 | 387,488 | none | 2026-10-18 | `diag_render_log.txt` |
| scene fit | - | - | - | 615 objects, 3,656,728 vertices (111.6 MiB), 7,281,868 faces (111.1 MiB), 95 materials, atlas 0.7 MiB: fits 128 MiB/binding, 4 chunks | - | 2026-10-18 | `phiber_pose26_log.txt`; build 517.6 s, `scene_sizes` 290.6 s |
| pose 26 per sample | - | 1440 sq | - | about 4 s/sample (from 253,983/64 = 3.97); a lighter scene about 0.1 s/sample | "Cycles would not be 40x slower" is an opinion, not a measurement | 2026-10-18 | `docs\blender-chapter-parity.md` lines 78-83 |
| pose 26 `p26*` probe renders | 128 | ? (likely reduced with PHIBER_SCALE; not logged) | 2,700-3,604 | 31,343 / 31,420 / 32,965 / 38,104 / 39,843 / 47,579 / 68,507 | none | 2026-10-20 | `run\probe\p26_raw.txt, p26_den.txt, p26b..e_*.txt` (raw 68,507; den 38,104; b raw 31,343 / den 31,420; c 32,965; d 47,579; e 39,843) |
| `fix*`/`sky_pose26`/`pilot_pose26` | 64? | ? | ? | not recorded (`times.txt` overwritten) | none | 2026-11-11/12 | `CW-POSES\PHIBER-RENDERS\` pictures only |

### 3.2 Poses 24, 30 and others (CW-POSES)

| Scene | Samples | posed_ms | drew_ms | Date | Source |
|---|---|---|---|---|---|
| pose 30 (`fix4_pose30`) | 64 (times line) | 1,635 | **176,920** | 2026-11-12 14:27 | `PHIBER-RENDERS\times.txt` (matches `fix4_pose30_0001.png` mtime 14:27:30) |
| pose 24 | 16 | 1,014 / 978 | 66,339 / 33,108 | 2026-10-20 | `run\probe\p24.txt` |
| pose 24 | 256 | 946 | 531,069 | 2026-10-20 | `run\probe\p24_record.txt` |
| pose 25 | 256 | 908 | 728,962 | 2026-10-20 | `run\probe\p25_record.txt` |
| "rec" scenes | 256 | 789-980 | 122,451; 486,658; 510,144; 446,826; 458,105 | 2026-10-20 | `run\probe\rec_256.txt, rec2..5_256.txt` |
| "rec" | 16 / 256 | 3,043 / 1,102 | 74,039 / 2,644,084 | 2026-10-20 | `run\probe\record.txt` |
| A/B ("OLD / NEW / SAH") at 64 samples, `PHIBER_SCALE` unknown | 64 | 892-1,451 | OLD: 43,299; 47,761; 42,518; 44,553; 49,119; 43,687; 47,944. NEW: 85,178; 64,305; 34,361; 35,982; 37,781. SAH24: 87,006; 63,965. SAH: 38,035; 39,334. NOSAH: 102,153; 66,995 | 2026-10-20 | `run\probe\ab_*.txt, ab2_*.txt, ab3_*.txt` (scene not logged; spread inside one variant is up to 2x, so these tell us the noise, not a winner) |
| "col", "cry" | 64 | 649-1,500 | 11,700; 42,581; 37,362; 50,347; 29,416; 57,891 | 2026-10-20 | `col_q.txt, col2_q.txt, col3_q.txt, cry3q*.txt, cry_den.txt` |
| lamps / world override | 64 | 721-779 | 13,560; 11,454; 13,681; 12,380; 12,404 | 2026-10-20 | `p8_lamps.txt, p8_world.txt, ph_ovr*.txt` |
| bisect (nodenoise / white / nolights / noworld) | 16 | 2,791-3,300 | 23,126; 23,351; 14,203; 20,632 | 2026-10-20 | `run\probe\bisect.txt` |
| CLI full vs white override | 16 | 2,998 / 2,990 | 67,712 / 67,383 | 2026-10-20 | `run\probe\cli.txt` |
| animation, 2048 samples | 2048 | 1-114 | frame 30: 22,878; 60: 58,427; 60: 70,608; 60: 41,409; 60: 51,687; 60: 46,343; 60: 43,738; 112: 146,437; 300: 123,907; 150: 334,947; 240: 865,435; 60: 316,678 (`times.txt` 22 Oct) | 2026-10-21/22 | `run\probe\demo_r1..7.txt, v40_render.txt, v43_build.txt, v44_build.txt, times.txt` (first column is frame; sample counts 64/128/256/384 also appear in `v44_build.txt`, `v45_build.txt`) |
| root `times.txt` | 2048 | 1,864 | **1,395,485** (23.3 min) | 2026-11-01 | `Downloads\phiber\times.txt` (scene not stated) |

### 3.3 LAVA scenes (phiber only, no Cycles pair timed)

| Scene | Samples | posed_ms | drew_ms | Date | Source |
|---|---|---|---|---|---|
| LAVA poses 01-03 | 64 | 2,801 | 306,191 | 2026-11-02 15:44 | `LAVA-RENDERS\times.txt` |
| same, nov02 | 64 | 2,495 | 556,691 | 2026-11-02 16:43 | `LAVA-RENDERS\nov02\times.txt` |
| LAVA `times.txt` | 16 | 378 | 27,215 (frame 5) | 2026-11-02 | `LAVA-POSES\times.txt` |
| bench | 32 | 1,100 | 30,854 | 2026-11-03 | `LAVA-POSES\bench\times.txt` |
| check (13 frames) | 16 | 3,023 | 119,260 | 2026-11-02 | `LAVA-POSES\check\times.txt` |
| check2 / check3 / checktest | 32 / 24 / 8 | 2,131 / 1,135 / 1,642 | 152,305 / 41,871 and 41,610 / 30,528 | 2026-11-02/04 | `LAVA-POSES\check2, check3, checktest` |
| four (FOUR colours x 4 cameras) | 96 | 2,134 | 276,121; 408,545; 270,915; 461,274 | 2026-11-03 | `LAVA-POSES\four\renders\times.txt` |
| renders16 (16 frames) | 48 | 1,119 | 139,017 | 2026-11-04 | `LAVA-POSES\renders16\times.txt` |
| show (74 camera lines) | 48 | 1,417 | 87,043 ... 241,122 (all 74 lines in file; median about 150,000) | 2026-11-04 | `LAVA-POSES\show\times.txt` |
| `_temp\show-*` | 24-64 | 1,916-3,083 | zigzag 118,601 (32); checks 143,239 (32); solid-net 153,005 (32); before-colour-rule 158,182 (48); cam-inside 196,120 (48); first-colours 208,071 (48); v3-checks 206,301 (48); v2-plants 256,429 (48); v1 295,934 (64); glass-bug 312,578 (24) | 2026-11-02/03 | `_temp\*\times.txt` |
| MONDAY-FIX | 64 / 128 | 521-587 | final 56,034 (128); renders 42,571 (64); renders2 21,251 (64) | 2026-11-02 | `MONDAY-FIX\*\times.txt` |
| LAVA-SHOW Cycles check | 6, 360 px, CPU, 4 threads, denoise off | - | **Blender wall 02:59.266 at "Saved"** (includes file read, 7,415 instances from 474 empties) | 2026-11-02 | `LAVA-POSES\check_show.log` (file asks 2048 samples at 1920, viewport 256, threshold 0.008; `check_show.py` overrides to 6 / 360 px) |

### 3.4 LAVA bench scored against phiber's own 1024-sample image (not Cycles)

Source: `LAVA-POSES\bench\results.txt` and `results-show.txt` (3 Nov). RMSE in 0-255 sRGB, 32 samples, `PHIBER_SCALE=2`, camera FOUR-front / SHOW-line.

| Variant | s/sample raw | s/sample denoised | RMSE raw | RMSE denoised |
|---|---|---|---|---|
| baseline (x3 runs) | 0.628 / 0.786 | 0.764 / 0.766 | 9.88 | 7.37 |
| unchecked (x4) | 0.501-0.855 | 0.512-0.675 | 9.88 | 7.37 |
| qmc (x3) | 0.824-0.933 | 0.838-0.879 | 9.78 | 7.12 |
| lobe (x2) | 0.581 / 0.590 | 0.742 / 0.563 | 9.93 | 7.19 |
| lean (x3) | 0.416-0.451 | 0.451-0.462 | 10.25 | 7.52 |
| lamps (x2) | 0.475 / 0.487 | 0.510 / 0.570 | 10.12 | 7.37 |
| onepass (x2) | 0.474 / 0.407 | 0.489 / 0.422 | 10.24 | 7.41 |
| show-lean / show-lamps / show-onepass | 0.275 / 0.257 / 0.249 | 0.267 / 0.250 / 0.240 | 19.78 / 17.95 / 18.47 | 16.19 / 14.74 / 14.99 |

Same-variant s/sample differs by up to 1.7x run to run (unchecked 0.501 to 0.855), so the speed claims below 30 % are inside the noise. This is the variance run that `BENCH-VARIANCE.bat` was meant to settle.

### 3.5 Quality numbers in the docs (Blender 5.x Cycles as the reference)

| Measure | Value | Date | Source |
|---|---|---|---|
| THYMUS, Material Preview, white override, Standard view, figure mean | Blender 226.9; phiber 198 -> 225.4 -> **233.0** | 2026-09-26 | `docs\assessment.md` 4 |
| figure mean, real materials | Blender 56.5; phiber 87.8 -> **61.4**; p50/p95 43/139 vs 53/130 | 2026-09-26 | same |
| patches: shin 32.5 vs 33.7; chest 73.0 vs 85.8; head 92.5 vs 78.7 | | 2026-09-26 | same |
| animation, THYMUS frames 1-121 step 20, mean abs diff | 35.2 -> 13.0 /255 after env fix; 15.6/255 on the figure; phiber about a quarter darker | 2026-10-05 | `ui-parity-status.md` batch 152 |
| posing 29 ms/frame vs Blender 83 ms | | 2026-10-05 | same |
| point light power constant | 5.066 vs Cycles 5.046 | 2026-10-05 | `rendering.md` ~2806 |
| uniform image vs uniform colour | 0.8043 vs 0.8121 (0.96 %); both within 1.5 % of Cycles | 2026-10-06 | batch 159 |
| banded HDRI | Cycles 1.2910, phiray 1.3417 (x1.039) | 2026-10-06 | batch 159 |
| Sky texture above horizon | within 1.6 % of Cycles | batch 163 | `space-status.md` row "Materials" |
| camera framing, pose 26 | IoU 0.9962 | 2026-10-18 | `blender-chapter-parity.md` line 24 |
| modifier silhouettes | IoU >= 0.999 | batch 170 | `space-status.md` |
| compositor per-pixel nodes | within 2e-4 of Blender | batch 167 | same |
| drivers | 9.5e-7 of Blender's depsgraph | batch 171 | `blender-chapter-parity.md` line 66 |
| open time THYMUS / FLORIS | 173.0 / 107.9 ms vs Blender 621.8 / 247.9 ms (3.6x / 2.3x faster) | 2026-10 (b184) | `docs\blender-head-to-head.md` |
| open time, 5 small fixtures | 4.4-7.4 ms vs 4.7-8.5 ms, "a wash" | same | same |
| UV unwrap, sphere band | 4.29 deg worst / 1.363 deg mean vs Blender 4.68 / 1.377 | batch 170 | `space-status.md` |
| UI frames on a real file | 120-400 ms (75-255 ms rebuilding the view tree) | 2026-10-17 | `gap-list.md` check of 17 Oct |
| sandbox speed | 32 samples, 720 x 1080, lavapipe: phiber 30 s, Blender CPU 34 s (explicitly not quotable) | 2026-09-26 | `assessment.md` 4 |

Missing altogether: a phiber vs Cycles time on any scene; a `compare_pose.py` score for any pose; the variance of one timing.

---

## 4. Reference images vs phiber images

Roots: `D` = `C:\Users\phiac\Downloads`. Sizes are bytes. I did not open images; "Blender" means the file name or the folder says so (`BS-POSE-nn` = Blender render), engine (Cycles vs EEVEE) is not recorded in any file I read.

### 4.1 Pairs (both sides exist)

| Scene / pose | Blender reference | phiber output(s) | Compare sheet | Score |
|---|---|---|---|---|
| pose 26 | `D\phiber\CW-POSES\RENDERS-WITH-TRANSPARENT-BG\BS-POSE-26.png` 2,640,269; `RENDERS-WITH-BG\BS-POSE-26.png` 2,207,765 (17 Oct); `PHIBER-RENDERS\pose26_blender_relinked_.png` 2,214,273 (31 Oct); `phiber\docs\images\00-blender-reference.jpg` | 18 Oct: `pose26_draft64_0001.png` 1,229,533; `pose26_fast64_0001.png` 1,225,104; `diag32_0001.png`. 31 Oct: `pose26_baseline4/64/256`, `alpha64`, `cut64`, `uv64`, `relink64`, `adaptive_gate64`, `gate256`, and 16-sample probes (`anyhit16`, `carry_node16`, `metal_special16`, `moller16`, `shadow_late16`, `tri_reuse64`, `watertight16`, `zero_lobe*16`). 11-12 Nov: `pilot_pose26` 1,887,360; `fix2_pose26` 1,690,894; `fix3_pose26` 1,505,732; `fix4_pose26` 1,496,700; `sky_pose26` 1,689,462 | `pose26_baseline4_compare` 3.5 MB, `baseline64_compare`, `alpha64_compare`, `cut64_compare`, `relink64_compare`, `uv64_compare`, `pilot_pose26_0001_compare` 6.05 MB, `fix2/3/4_pose26_0001_compare`, `sky_pose26_0001_compare` | none saved as text |
| pose 24 | `RENDERS-WITH-TRANSPARENT-BG\BS-POSE-24.png` 1,785,558; `RENDERS-WITH-BG\BS-POSE-24-BG.png` 2,048,561 | `pilot_pose24` 1,108,214; `fix_pose24` 918,797; `fix2_pose24` 929,508; `fix3_pose24` 929,106; `fix4_pose24` 932,117; `sky_pose24` 864,797 | `*_compare.png` for each | none |
| pose 30 | `RENDERS-WITH-TRANSPARENT-BG\BS-Pose-30.png` 1,740,907; `RENDERS-WITH-BG\BS-POSE-30.png` 2,155,860 | `pilot_pose30` 1,082,755; `fix4_pose30` 948,032 | `pilot_pose30_0001_compare`, `fix4_pose30_0001_compare` | none |
| LAVA pose 01 | `D\phiber\LAVA-RENDERS\blender_pose01.png` 2,974,344 (1 Nov) | `phiber_pose01_0001.png` 1,893,632; `phiber_v2_pose01_0001.png` 1,912,569 | `phiber_pose01_0001_compare` 6.99 MB; `phiber_v2_pose01_0001_compare` 7.17 MB | none |
| LAVA pose 02 | `blender_pose02.png` 3,228,329 | `phiber_pose02_0001.png` 1,902,505; `phiber_v2_pose02_0001.png` 2,025,107 | `..._compare` 7.63 MB; `v2` 7.92 MB | none |
| LAVA pose 03 | `blender_pose03.png` 2,362,346; `blender_pose03_nocar.png` 2,429,787 | `phiber_pose03_0001.png` 1,380,237; `phiber_v2_pose03_0001.png` 1,432,281; `phiber_v2_pose03_nocar.png` 1,432,281 | `..._compare` 5.4 MB; `v2` 5.5 MB; `nocar_compare` 5.2 MB | none |
| LAVA show | `LAVA-POSES\show_blender_check.png` 377,665; `show\blender_check.png` 356,658 (6-sample, 360 px CPU) | `LAVA-POSES\show\` 75 PNGs, `renders16\` 16, `check\` 9 | none | none |
| female01 | `phiber\docs\female01_blender_vs_phiber.png` 165,999; `female01_world_blender_vs_phiber.png` 847,303 (already a side-by-side) | in the same files | - | none |
| docs manual | `docs\images\00-blender-reference.jpg`, `09-compare.jpg` | `rendering-manual.md` | - | none |
| bench self-reference | - | `LAVA-POSES\bench\reference.png` 630,970; `reference-show.png` 1,077,593 (phiber at 1024 samples, not Blender) | - | RMSE in 3.4 |

### 4.2 Blender-only (no phiber render found)

| Folder | Content |
|---|---|
| `CW-POSES\RENDERS-WITH-BG` (also `D\RENDERS-WITH-BG-20261017T214552Z-1-001`, 36 files, 79 MB) | `BS-POSE-01..31`, 17 Oct. Phiber has none of 01-23, 25, 27-29, 31. |
| `CW-POSES\RENDERS-WITH-TRANSPARENT-BG` | `BS-POSE-01..32` (32 is extra), 17 Oct. Same missing list. (The Downloads zip `RENDERS-WITH-TRANSPARENT-BG-...` is empty.) |
| `D\phiber\CITY-POSES\10-CRYSTAL-CAVES-...` | 20 PNGs: `RENDER-3Q-02`, `RENDER-CLOSE-UP-01..04`, `RENDER-FRONT/SIDE/TOP`, `LAND-TYPE-10-RENDER-{BEAUTY,FRONT,SIDE,TOP}-2560-1440`, logos, sketches. 3.3-6.6 MB each |
| `D\SINGLE-RENDERS-20261004T173343Z-1-001` | 36 outfit renders (OUTFIT-06-DRESS-3Q/BACK/BOTTOM/FRONT/SIDE/TOP, ...), 74 MB |
| `D\SINGLE-RENDERS-20261004T173453Z-1-001` | 73 renders (BOOTS\ANKLE-BOOT-*, ...), 195 MB |
| `D\SINGLE-RENDERS-20261019T000948Z-1-001` | 18 (OUTFIT-05-DRESS-*), 42 MB |
| `D\SINGLE-RENDERS-20261020T165303Z-1-001` | 27 (LS-STYLE-07-LOOK-02-*-OPTION-0n), 59 MB |
| `D\POSE-RENDERS-20261004T173520Z-1-001` | 5: `LS-POSE-03-HORSE-PERSPECTIVE-01..04`, `DRESS-03-OPTIONS` |
| `D\drive-download-20261014T1927*..T1928*` | only `.blend` shoe/mannequin files (SHOE OPTION 02..06 LOOK nn), no images |
| `D\drive-download-20260628...`, `20261018...`, `20261020...`, `20261104...` | 101 + 12 + 20 + 3 + 17 images, mostly inspiration photographs and screenshots (e.g. `Phi-Pill-preview.png`, `Phi-Soft-specimen.png`, `IMG_2026*.jpg`), plus a 29.7 MB screen recording. No render pairs |
| `D\MPHO-POSES-...` | `.blend` only (pose 26 and 30 copies; the `-001` zip is empty) |

### 4.3 Phiber-only (no Blender side)

| Folder | Content |
|---|---|
| `D\phiber\MONDAY-FIX` | `renders` (5), `renders2` (3), `final` (5: `DESIGN-08-SOFT-front/side/back/close/three_quarter`); `cameras.txt` names 10 cameras. 54 s - 42 s per frame |
| `D\phiber\LAVA-POSES\four\renders` | 16 images, 4 colours x 4 cameras (`phantom_{gold,red,blue,black}-FOUR-{front,side,back,other_side}`) |
| `D\phiber\LAVA-POSES\_temp` | 91 images in `show-*` iteration folders (e.g. `show-glass-bug`, `show-zigzag`) |
| `D\phiber\_temp` | 71 images in `show-*` folders and `four-fireflies`, `four-hueshift` |
| `D\phiber\run\probe\v45_checks` | 1 folder of checks |
| `D\phiber\this-is-phiber-topo-flesh-and-welcome-video` | 11 screenshots (UI, not renders) |

Only three pose pairs and three LAVA pairs can be scored today. Scoring 23-31 needs the phiber side first.

---

## 5. The gap list

Size: S = under a day, M = a few days, L = weeks, XL = months. Owner crates: phiber (UI and scene glue), phiray (renderer), phiphx (file reader/evaluator), appkit (UI toolkit), cycles (the port).

### 5(a) Rendering - what phiber/phiray draw differently from Blender/Cycles

Ordered by what the pose 26 head-to-head needs first.

| # | Gap | Evidence | Size | Owner |
|---|---|---|---|---|
| R1 | **No Cycles timing and no recorded score.** `bench_cycles.py` never ran; `compare_pose.py` prints but nothing is saved; variance run absent | `cycles_bench_log.txt` "exit 1"; `basketball_log.txt` "Access is denied" exit 5; `blender_where.txt`; no `phiber_variance_log.txt` | S | phiber (tools) |
| R2 | **Heavy-scene speed.** About 4 s/sample at pose 26 (7.28 M faces, 615 instances) vs 0.1 s on a lighter scene. Two traversal changes were slower (+21 %, +29 %); the fix is unproven; thermal confound on record. Later gains: "a third faster a sample" (commit ac53e823e, 3 Nov), pose 30 at 176,920 ms for 64 | `blender-chapter-parity.md` 78-83; `render-fix-report.md` "performance attempt"; `phiray\docs\APPLE.md` rows 1-12 (shadow-ray order, double node loads, no adaptive in viewport, Preview compile 247,997 ms, display path, leaf locality, node format, blue noise, temporal reuse, hardware ray query, GPU present); `phiray/jobs.md:15,17,18,20` | L | phiray |
| R3 | **EEVEE and Cycles are names.** Every non-Workbench engine is traced by phiray; there is no EEVEE raster and no Cycles integrator, so "Cycles parity" means "phiray parity" | `issues.md` 3.2; `scene\render_engine.rs` (per issues.md); `space-status.md` Rendering row says "EEVEE / Cycles / PHIRAY path-trace" (consistent) | XL (this is the Cycles port) | cycles / phiray |
| R4 | **19 Cycles/render settings printed "IGNORED by Phiray":** `blur_glossy`, `caustics_reflective`, `caustics_refractive`, `denoiser`, `device`, `pixel_filter_type`, `render.filter_size`, volume_bounces / volume_max_steps / volume_step_rate / volume_preview_step_rate, `cycles_curves.shape/subdivisions`, `motion_blur_position/shutter`, `use_motion_blur`, `threads`, `threads_mode`, `use_persistent_data` | `run\probe\ab3_NEW_1.txt` lines 4-30; `scene\render_audit.rs:81-97,155` | M (pixel filter, filter-glossy, caustics) / XL (volumes, motion blur) | phiray + phiber |
| R5 | **Adaptive sampling: docs conflict.** `ROADMAP.md` and `ab*.txt` say applied; `blender-chapter-parity.md` line 40 and `workflow.md` 275-279 say read, saved, not applied; the code calls `tracer.set_adaptive` only in the final render (`stepwise.rs:100`); the viewport never does (`APPLE.md` row 3). Time limit and noise threshold: doc says not applied | `stepwise.rs:100` (confirmed by grep: only caller) | S-M | phiray |
| R6 | **Light: no light tree, light cap, no light linking, no shadow catcher / holdout.** Power CDF over up to 32 lamps (the code is 64 per `APPLE.md`; docs drift). Light linking is not started | `ROADMAP.md` 3.3 and section 7 drift table; `blender-chapter-parity.md` 26; `ui-parity-status` | M light tree 8-12 d; S holdout | phiray |
| R7 | **World and missing HDRI rules are inconsistent.** Doc says a missing environment image is magenta "exactly as Blender"; commit `0e0c2e183` (3 Nov) changed it to white. Pose 26's HDRI is missing in the file (`hdri.txt`: OVERCAST_MISSING); a sky EXR now sits in `CW-POSES\sky` and `sky_pose24/26` renders exist | `blender-chapter-parity.md` 27; `git log` 0e0c2e183; `run\probe\hdri.txt` | S (decide, then test) | phiray + phiphx |
| R8 | **Colour management.** Standard, Filmic, AgX with exposure and gamma. Look is not read (doc says); Working space and curves rows missing | `blender-chapter-parity.md` 30; `panel-parity.md` 96 | S-M | phiray + phiber |
| R9 | **Denoiser is a guided a-trous filter, not OIDN.** `cycles.denoiser` setting ignored. OIDN via C API on CPU estimated 3-5 d (doc says) | `ROADMAP.md` 3.3; `assessment.md` 4 item 7 | M | phiray |
| R10 | **Passes.** Only Combined, Depth, Normal, Diffuse colour (albedo). Missing Mist, AO, Position, Vector, UV, indices, Emission, light-path passes, Cryptomatte, AOVs. Each is 1-2 d (doc says), Cryptomatte 4-6 d | `blender-chapter-parity.md` 33; `ROADMAP.md` 3.3; `animation-rendering-status.md` | M | phiray + phiber |
| R11 | **Materials: shader graphs are baked to one fixed record** (about 120 words, `phiray/src/trace/mat.rs`, `proc.rs`); unsupported nodes approximated or dropped; no general node interpreter (20-30 d, doc says). Textures on roughness, metallic and emission inputs are not done (`phiray/jobs.md:16`). Principled multi-scatter is single-scatter; Preetham and Hosek/Wilkie sky declined; `Undefined[...]` nodes (pose 26 has 1, Epic Female 01 has 110) render socket defaults. Pose 26 hands 95 of its 1335 materials to the tracer | `issues.md` 3.3; `space-status.md` Materials row; `node\spatial\jobs.md:31`; `phiber_pose26_log.txt` | L | phiphx + phiray |
| R12 | **Subsurface is a wrap-lighting approximation**, not random-walk SSS (8-12 d); skin on poses 24-30 depends on it (`compare_pose.py` regions `skin_standing`, `skin_crouching`) | `ROADMAP.md` 3.3 (`scatter_bleed`, `sss_tint` in `brdf.wgsl`) | M-L | phiray |
| R13 | **Geometry that reaches the render is not identical to Blender's:** loose-edge subdivision (192 vertices in rigs, 48 curve vertices; `SHORT` has 2 rows), curve objects not tessellated, Bevel display, Array caps/merge/fit, Solidify complex, Displace with texture, Armature envelopes / dual quaternion, `PARBONE/PARSKEL/PARVERT`, collection instances and dupli | `gap-list.md` Tier 1 Geometry; `open_parity.rs` `SHORT`; `blender-head-to-head.md` "Two parity gaps" (note: the batch-194 loose-edge change over-counts, 150 vertices over on THYMUS, 6,194 on FLORIS, `gap-list.md` 221-226) | M-L | phiphx |
| R14 | **Per-frame geometry for non-rigged objects and driven modifiers.** A mesh evaluates once at open; driven modifier settings do not reach the mesh while playing. Matters for animation renders | `gap-list.md` Tier 0 (3 cases; in progress, uncommitted at the time) | M-L | phiphx |
| R15 | **Film and camera.** Film transparent glass/roughness unread; no motion blur, no panoramic cameras, no volumes, no caustics, no shadow catcher; true displacement (bump only); hair and curves absent (10-15 d); `camera view zoom / passepartout` open | `panel-parity.md` 94; `assessment.md` 4 item 7; `ROADMAP.md` 3.3; `scene/jobs.md` | L each | phiray + phiphx |
| R16 | **Scene size ceiling.** About 7.2-7.6 M triangles render; about 20 M refused on a 128 MB binding; streaming per object not built | `phiray/jobs.md:12,65`; `issues.md` 3.4 | L | phiray |
| R17 | **Display path.** Frame is read back to the CPU each time (5 s hang guard, TDR); no GPU-to-GPU present; viewport temporal reuse absent | `issues.md` 3.4; `APPLE.md` rows 5, 10, 12; `phiray/jobs.md:17` | M-L | phiray + appkit |
| R18 | **Viewport modes.** Material Preview screen-space reflections and AO not done; Solid outline has no AA line, no occlusion fade, X-ray hull on CPU; Preview compile 247,997 ms (doc says) | `workflow.md` 315-316; `space-status.md` row 3D Viewport; `APPLE.md` row 4 | M | phiray |
| R19 | **Output.** No bundled encoder (uses system ffmpeg or MJPEG AVI), Overwrite / Placeholders / frame step ignored, no metadata stamp, no audio in movie, Encoding panel not written back (doc says) | `blender-chapter-parity.md` 37-39; `animation-rendering-status.md` | S-M | phiber |
| R20 | **Find Missing Files** (pose 26 needs it for the HDRI) | `blender-chapter-parity.md` 84 | S | phiber |
| R21 | **Render-quality evidence is old.** Absolute numbers are Sept-Oct on THYMUS; pose 24-30 sheets of Nov have no number; pose 26 needs a recorded region table | section 3.5 | S | phiber (tools) |
| R22 | **Port ledger check.** `cycles` crate has no picture yet; its plan gates "first plain image" (path state, `intersect_closest`, one diffuse and emission closure, film, megakernel, scene from phiphx, image file). An image that matches needs SVM + Principled + lights + world | `crates\cycles\PLAN.md` "Where it stands" | XL | cycles |

### 5(b) Layout and spaces

Status words follow `space-status.md` (last updated at batch 170 per its own header; later batches exist, so check). Row "doc says" means I did not find a test or a line of code that confirms it.

| # | Gap | Evidence | Size | Owner |
|---|---|---|---|---|
| L1 | **Texture Paint is a shell:** lays out Image editor beside the viewport; no brushes, strokes, slots | `space-status.md` workspace table; `space\texture_paint\set.rs` exists (1,897 bytes), spec only | XL | phiber |
| L2 | **Scripting is a shell:** Text, Console, Info; no Python, no console, no text editing; "not planned" | `space-status.md` queue #15; `host\text`, `host\console` are 483/498-byte mod files | XL | phiber |
| L3 | **Sequencer and Movie Clip are shells:** `seq` and `clip` hosts declare `parts: &[BPartType::Custom]` (a bare custom part) (confirmed in code: `host\seq\set.rs:18`, `host\clip\set.rs:18`); Clip is "n/a, not started" | code, `CONSISTENCY_PLAN.md` C7 | L | phiber |
| L4 | **Geometry Nodes: no evaluation.** Shares node canvas; Spreadsheet works for Vertex / Face / Face Corner only | `space-status.md` #13 queued, "very large" | XL | phiphx + phiber |
| L5 | **Shading / node editor.** `space-status.md`: partial (add menu, links, sockets, frames); the node graph is not yet the file's own tree for every node type. `workflow.md` (14 Oct) still says "placeholder two-card graph" (stale). Open items: node spatial view long-link folding, three-plane stack, preview plane showing a node's output (needs per-node evaluation), `Undefined[...]` nodes, relink and material assignment; "Add menu, relinking and material assignment do not [exist]" (`CONSISTENCY_PLAN.md` C6, undated) | `space-status.md`; `node\spatial\jobs.md:25-31`; conflict between three docs | L | phiber + phiphx |
| L6 | **Compositing:** per-pixel nodes evaluated and run over a render (within 2e-4); neighbourhood filters, transforms, keyers, curve nodes are declined and named by `BCompTree::declined`; Blur first (queue 12c); `issues.md` D says the node editing UI for the compositor is not wired | `space-status.md` row and queue 12/12b/12c; `issues.md` D | L | phiphx + phiber |
| L7 | **Animation editors, editing:** Graph editor edits nothing (no select/move/handle drag); Dope Sheet has one row per object, not per F-curve; NLA tweak mode and strip dragging; Drivers editor lists but does not run drivers on materials, modifiers, shape keys; shape keys and Shape Key editor absent; F-curve modifiers beyond Generator; Spline IK and Shrinkwrap; keying sets; motion paths | `space-status.md` Animation row and queue 5, 6, 8, 9; `blender-chapter-parity.md` 54-70; **blocked**: "the other agent owns `canvas/time/`" | L | phiber + phiphx |
| L8 | **About 390 menu entries are inert** (doc says; `CONSISTENCY_PLAN.md` says about 380). Code check: `src\part\menubar` has 278 `op(` and 213 `fire(` occurrences. That is not the inert count but shows the stubs persist. Dope Sheet about 95, Graph about 130, Drivers about 120, NLA about 50, Image editor about 50; Copy / Paste Keyframes landed 29 Oct | `animation-rendering-status.md`; `issues.md` Part V; grep | L | phiber |
| L9 | **No interactive area split / join / swap / duplicate-into-window;** layouts are templated per workspace plus maximize and editor-type switch. Called the largest conceptual layout gap. Related open: saved area grid and editor types from the file (#28, tracked in two places), area sizes restored from the file's screen | `issues.md` B and Part III #3; `space\jobs.md:9`; `area\jobs.md:9`; grep found no `area_split`/`area_join` symbols | L | phiber |
| L10 | **Properties tabs.** Works per docs: Render, Output, View Layer, Scene, World, Object, Modifier, Material (`space-status.md`). `assessment.md` (26 Sep) lists scene, world, data, texture, modifier, particles, physics, constraint, collection, tool as declared but not seeded (stale or not: unchecked). Properties tab expand state open (`gap-list.md`). Physics panel: doc says UI spec only (ROADMAP) vs solver wired 29 Oct (`issues.md` Part V) | docs conflict | M | phiber |
| L11 | **Cycles Render-tab rows missing:** viewport adaptive sampling + preview denoise, denoising input passes / prefilter / quality / GPU, Path Guiding, Advanced, Lights; Clamping, Fast GI; `volume_biased`; rolling shutter and shutter curve; `film_exposure`, transparent glass/roughness; Performance (tile size, texture cache, acceleration structure, viewport, compositor); Colour Management working space, curves, white balance. Output: Views, Pixel Density, Encoding Video/Audio, Metadata. Object: Instancing, Visibility ray + culling. Material: Settings | `panel-parity.md` 81-106 (dated 20 Oct; the table lists `sample_clamp_*` as missing while `ab3_NEW_1.txt` shows `sample_clamp_direct/indirect` applied: stale in part) | M | phiber |
| L12 | **Panel behaviour:** pin (PNL_PIN), reorder by grip, active-panel outline, open/close animation (0.30 s), title dim on search miss, header fade when narrow, region max-clamp, scrollbar thumb dragging and horizontal bars, keyboard fold | `issues.md` B | M | phiber + appkit |
| L13 | **Region toggles:** Toolbar (T) a placeholder (N and Ctrl Space done) (doc says) | `CONSISTENCY_PLAN.md` A2 (undated) | S | phiber |
| L14 | **Spreadsheet:** no Edge domain, no row filters or pinning, meshes only | `space-status.md` | M | phiber + phiphx |
| L15 | **UV editor:** no box or island select; Angle Based's ABF pre-pass not ported | `space-status.md` | S-M | phiber |
| L16 | **Outliner expand state not restored;** Timeline: middle-mouse pan, Set Preview Range (P), Show Seconds, Sync Visible Range, marker duplicate / lock / bind camera, file's own `v2d.cur` | `gap-list.md`; `animation-rendering-status.md` Timeline | S-M | phiber |
| L17 | **Surface consistency:** one zone/area/host look, one menu look, elevation ladder base / zone / popover / modal; `BEffects` has one `surface_shadow` slot; the spatial coat default; 535 raw `div()` style calls left to move onto `BFlex`; 50 free `fn`s in the draw layer; 8 off-token values | `view\jobs.md:9-11`; `theme\jobs.md:9`; `zone\jobs.md:9`; `jobs.md:9,14-17`; `unity.md` "Still open" | M-L | phiber + appkit |
| L18 | **Declarative audit** (host, pane, region, space, part): every part built from its `set.rs` table | `host/pane/region/space/part/jobs.md:9-12` | M | phiber |
| L19 | **State verbs and persistence:** `get/has/set/remove`; `NavbarExpanded` has two spellings | `state\jobs.md:9-11` | S-M | phiber |
| L20 | **Modals:** Help, Profile, Command as elevated spaces; collaboration views; Preferences sections vs Blender's; content modals diverge in layout; colour wheel / swatch sphere / font chooser | `modal\jobs.md:9-10`; `jobs.md:10-11` | M-L | phiber + appkit |
| L21 | **File and Asset browser as reusable hosts with per-instance state** (not one global `file.*`) | `part\jobs.md:9` | M | phiber |
| L22 | **UI speed on real files:** 120-400 ms frames, 75-255 ms of it rebuilding the view tree, worst with the File Browser open (17 Oct); 240-line-cap backlog (21 files in `phiber/src`, 10 in phiray/phiphx per `assessment.md`, 26 Sep; the cap guard only watches `phiber/src`) | `gap-list.md` check of 17 Oct; `assessment.md` 2 | M | phiber |

Layout truth that exists: `tests\truth\workspaces.txt` (49 lines: every factory workspace's areas with type, mode, x, y, w, h, regions in the 1920 x 1080 startup window, from `blender_workspaces.py`) is checked by `tests\workspace_regions_truth.rs`; menus by `menu_truth.rs` (71 lines); channel names by `channel_name_truth.rs`. `assessment.md` says only Layout was verified against Blender's default screen; these tests supersede that but I did not run them, so whether the other nine pass is "doc says" (not confirmed).

Conflicts to settle before trusting any count: `space-status.md` marks Rendering, Layout, Animation, UV Editing "works"; `animation-rendering-status.md` still has 390 inert menu entries and Graph-editing missing; `issues.md` Part III names three problems (physics unwired - closed 29 Oct; dead animation menu items; templated layout).

---

## 6. Order to close them before Cycles resumes

Each step ends with a recorded number written to `crates\phiber\docs\analysis\` (or a `results-*.txt` beside the logs) so it can be compared later.

| Step | Work | Gaps | Proof it is done (what is measured) |
|---|---|---|---|
| 0 | **Re-baseline the tools.** Point the six scripts at `gemphi\phikit`; build release; run `BENCH-VARIANCE.bat` (3 runs, 90 s cool-down) for pose 26; get Blender running from a non-Store copy and run `bench_cycles.py`; make `compare_pose.py` write its text (`> *_score.txt`) | R1, R21 | `phiber_variance_log.txt` exists with 3 drew_ms values and their spread; `cycles_bench_log.txt` has `total_s=` and `per_sample_s=` for 1440 x 1440 / 32 samples; one score text per sheet. Only then are changes under about 30 % judgeable (section 3.4 shows 1.7x spread) |
| 1 | **Fill the pair table.** Render poses 23, 25, 27, 28, 29, 31 at 64 samples (`RENDER-POSES-23-29.bat` plus 31); re-render 24, 26, 30; score all with `compare_pose.py` against `BS-POSE-nn` | R21 | A table of overall error and per-region error for every pose 23-31; the worst three regions per pose named. Set the target (for example overall under 8/255, each region under 15) in this file |
| 2 | **Light, world, colour.** Decide the missing-HDRI rule and test it; Find Missing Files; Look; confirm the sky EXR path; light tree decision (only if step 1 blames lamps) | R7, R8, R20, R6 | Pose 26 `floor_near`, `floor_far`, `car_hood`, `car_flank` errors drop; white-override figure mean within 1 % of Cycles (`PHIBER_OVERRIDE`); HDRI banded test stays at x1.04 or better |
| 3 | **Materials.** Roughness/metallic/emission textures, `Undefined` nodes, SSS for skin, glass, remaining nodes; geometry parity (loose edges, curves, modifiers) | R11, R12, R13 | Pose 26 `suit_*`, `skin_*`, `hair_crouching`, `glasses_crouching`, `bag_*` errors drop; `open_parity.rs` `SHORT` table empty; no `Undefined` socket reaches the tracer unreported |
| 4 | **Speed.** With step 0's variance, A/B interleaved with idle gaps (not single runs): APPLE rows 1-4 first; then leaf locality, node format; viewport adaptive; Preview compile | R2, R5, R16, R17, R18 | Pose 26, 64 samples: drew_ms median of 3 vs the 253,983 ms baseline and vs Cycles s/sample from step 0. State the ratio. Pictures must score no worse (step 1 table) |
| 5 | **Settings honoured and passes.** Pixel filter, filter-glossy, caustics, clamps; Mist / Position / UV / Vector / AO passes; OIDN; time limit | R4, R9, R10 | The `IGNORED by Phiray` list in the run log shrinks to those deliberately not built (volumes, motion blur); each new pass has a Blender-written EXR comparison |
| 6 | **Spaces on the render path first.** Properties Cycles rows (L11), Image editor inert entries, Rendering workspace, Shading/node editor edit + file tree (L5), Compositing neighbourhood nodes (L6), Toolbar T (L13), render-related menus | L5, L6, L8 (render menus), L10, L11, L13 | `workspace_regions_truth`, `menu_truth`, `channel_name_truth` pass for all workspaces; inert-entry count per editor (grep `op(` vs `fire(`) written down and lower; Properties rows listed in `panel-parity.md` become zero |
| 7 | **Animation spaces and layout engine.** Per-frame geometry (Tier 0), Graph editing, channel rows per F-curve, NLA tweak, drivers on materials/modifiers; then interactive area split/join and the saved screen grid | R14, L7, L9, L16 | per-frame truth test: N frames of Blender's evaluated matrices and meshes against ours, worst error stated (`gap-list.md` definition of done); a split/join test against `screen_ops.cc` behaviour; saved grid round trips |
| 8 | **Shell spaces and polish.** Sequencer, Texture Paint, Geometry Nodes evaluation, Spreadsheet domains, UV selects; surface unity, `div()` to `BFlex`, declarative audits, UI frame time | L1-L4, L14, L15, L17-L22 | Each shell row in `space-status.md` becomes "works" with a named test; UI frame time on a real file under about 33 ms (measure, then set the target); `div()` count falls (535 at last count) |
| 9 | **Gate to resume Cycles.** Re-run steps 0-1 on a fresh build; freeze the numbers in a table; the Cycles port is compared against this table with the same commands | R3, R22 | `crates\cycles` produces a picture; the same `compare_pose.py` score against `BS-POSE-26` is printed for phiray and for cycles side by side |

Steps 2, 3 and 5 can run in parallel with 6 (different owners). Step 4 should wait for step 0's variance. Step 7's animation editors are blocked on the owner of `canvas/time/` named in `space-status.md`.

---

## Appendix - Where the raw data is

- Scripts and logs: `C:\Users\phiac\Downloads\phiber` (`BENCH-*.bat`, `bench_cycles.py`, `phiber_render.ps1`, `RENDER-*.bat`, `*_log.txt`, `times.txt`).
- Pose scenes, references and phiber renders: `...\CW-POSES` (`RENDERS-WITH-BG`, `RENDERS-WITH-TRANSPARENT-BG`, `PHIBER-RENDERS`, `sky`, `textures`, `compare_pose.py`).
- Probes: `...\run\probe` (A/B logs, `times.txt` of 22 Oct, 310 files, `phiber-session.log` 87 MB).
- LAVA: `...\LAVA-POSES` (`bench`, `check*`, `four`, `show`, `renders16`, `make_*.py`), `...\LAVA-RENDERS` (Blender vs phiber pairs).
- Binaries kept for A/B: `phiber-lamps.exe`, `-lean`, `-lobe`, `-onepass`, `-qmc`, `-render` (4 Nov), `-subsurf` (4 Nov), `-unchecked` (about 51.7 MB each, 3-4 Nov).
- Code: `crates\phiber\src\cli\{cost.rs,render.rs,set.rs,mod.rs}`, `crates\phiber\examples\{render_compare,animation_compare,scene_sizes,pose_probe}.rs`, `crates\phiber\tests\truth\`.
