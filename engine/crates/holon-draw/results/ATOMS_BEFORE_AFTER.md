# atoms3d (Bevy) against atoms-draw (holon-draw): the same harness on both pages

Every cell is `median (min to max)`.

**What this measures.** Rendering is SwiftShader (CPU rasterisation in headless Chromium), so absolute frame times are not a GPU device's. The comparison between pages measured under the same conditions is what this measures.

- BEFORE: `docs/atoms3d/index.html` at HEAD with the live site's webgl2 build (wasm 31,729,601 bytes), served from `before_head_root`.
- AFTER: `docs/atoms-draw/index.html`; `?dpr=1` for the third phone column.
- Both took the WebGL2 path: ANGLE (Google, Vulkan 1.3.0 (SwiftShader Device (Subzero) (0x0000C0DE)), SwiftShader driver). `navigator.gpu` exists but returns no adapter.
- Runs are interleaved (one of each page in turn), a new browser process per run, 10 s frame window, `--pixels canvas` on both.

## Headline tables: sweeps 2 and 3 pooled, host busy (load 21 to 38 on 32 cores)

These are the sweeps that carry the heartbeat stall measure. The host was running other jobs, so ranges are wide.

### Desktop, 1280x800, ratio 1, no throttle (10 pairs)

| measure | BEFORE atoms3d | AFTER atoms-draw |
|---|---|---|
| runs ok | 10/10 | 10/10 |
| canvas backing store | 1280x800 | 1280x800 |
| graphics | Bevy/wgpu on webgl2 | holon-draw on webgl2 |
| ready, page claim, s | 4.18 (2.28 to 7.79) | 1.76 (1.39 to 2.53) |
| ready, pixels seen, s | 4.20 (2.49 to 8.34) | 1.77 (1.39 to 2.54) |
| longest main-thread stall before ready, s (heartbeat) | 3.61 (2.11 to 6.99) | 1.59 (1.28 to 2.36) |
| total stalled before ready, s (heartbeat) | 3.88 (2.32 to 7.87) | 1.76 (1.38 to 2.48) |
| longtask API: longest before ready, ms | 2456 (1487 to 5051) | 99 (67 to 523) |
| stalls over 50 ms in the 10 s window (heartbeat) | 1 (0 to 3) | 0 (0 to 3) |
| longest stall in window, ms | 92 (0 to 5423) | 0 (0 to 2644) |
| frames per second in window | 23.1 (13.1 to 34.2) | 56.7 (12.2 to 60.0) |
| frame interval median, ms | 32.3 (19.5 to 49.6) | 16.7 (16.7 to 75.6) |
| frame interval p95, ms | 45.4 (33.6 to 127.1) | 17.4 (16.8 to 175.1) |
| frame interval max, ms | 200 (41 to 5414) | 164 (17 to 2635) |
| page rAF callback median, ms | 4.8 (4.3 to 5.7) | 0.2 (0.2 to 0.3) |
| page rAF callback p95, ms | 7.4 (5.1 to 14.7) | 0.3 (0.3 to 1.6) |
| transferred, MB (uncompressed loopback) | 31.85 (31.85 to 31.85) | 0.91 (0.91 to 0.91) |
| if gzip -6, MB (estimate) | 9.74 (9.74 to 9.74) | 0.33 (0.33 to 0.33) |
| JS heap used at end, MB | 68.4 (66.8 to 70.3) | 5.6 (5.3 to 5.6) |
| wasm linear memory, MB | 61.0 (59.2 to 62.7) | 1.6 (1.6 to 1.6) |
| console errors | 0 | 0 |
| host load (1 min) per run | 25 to 38 | 25 to 42 |

### Phone, 390x844, ratio 3, 6x CPU throttle (6 triples)

| measure | BEFORE atoms3d | AFTER atoms-draw (native ratio) | AFTER atoms-draw ?dpr=1 |
|---|---|---|---|
| runs ok | 6/6 | 6/6 | 6/6 |
| canvas backing store | 390x844 | 1170x2532 | 390x844 |
| graphics | Bevy/wgpu on webgl2 | holon-draw on webgl2 | holon-draw on webgl2 |
| ready, page claim, s | 16.46 (15.77 to 27.35) | 9.55 (8.14 to 12.31) | 9.59 (8.03 to 15.84) |
| ready, pixels seen, s | 18.06 (17.03 to 28.48) | 9.58 (8.16 to 12.33) | 9.61 (8.04 to 15.85) |
| longest main-thread stall before ready, s (heartbeat) | 15.76 (15.12 to 25.32) | 8.72 (7.88 to 11.62) | 9.31 (7.80 to 15.00) |
| total stalled before ready, s (heartbeat) | 17.98 (17.02 to 27.11) | 9.53 (8.05 to 12.18) | 9.52 (7.91 to 15.67) |
| longtask API: longest before ready, ms | 11670 (10391 to 16802) | 72 (57 to 3028) | 144 (66 to 1648) |
| stalls over 50 ms in the 10 s window (heartbeat) | 98 (85 to 127) | 1 (1 to 3) | 2 (0 to 5) |
| longest stall in window, ms | 206 (91 to 1460) | 157 (56 to 2677) | 263 (0 to 806) |
| frames per second in window | 29.4 (15.2 to 31.7) | 48.2 (29.2 to 51.1) | 58.2 (51.4 to 60.0) |
| frame interval median, ms | 32.7 (27.1 to 53.9) | 17.1 (16.6 to 20.2) | 16.7 (16.7 to 16.7) |
| frame interval p95, ms | 59.6 (36.2 to 126.4) | 40.7 (17.7 to 59.9) | 17.8 (17.6 to 22.9) |
| frame interval max, ms | 224 (59 to 1436) | 194 (53 to 2683) | 239 (21 to 800) |
| page rAF callback median, ms | 30.9 (25.4 to 44.3) | 1.3 (1.1 to 1.4) | 1.3 (1.1 to 1.4) |
| page rAF callback p95, ms | 47.2 (33.5 to 93.8) | 2.1 (2.0 to 5.2) | 2.0 (1.8 to 5.6) |
| transferred, MB (uncompressed loopback) | 31.85 (31.85 to 31.85) | 0.91 (0.91 to 0.91) | 0.91 (0.91 to 0.91) |
| if gzip -6, MB (estimate) | 9.74 (9.74 to 9.74) | 0.33 (0.33 to 0.33) | 0.33 (0.33 to 0.33) |
| JS heap used at end, MB | 59.7 (56.4 to 64.1) | 4.1 (4.1 to 4.4) | 5.6 (5.5 to 5.7) |
| wasm linear memory, MB | 52.6 (49.0 to 57.1) | 1.6 (1.6 to 1.6) | 1.6 (1.6 to 1.6) |
| console errors | 0 | 0 | 0 |
| host load (1 min) per run | 23 to 43 | 23 to 45 | 23 to 37 |

## Sweep 1: quieter host (load 7 to 20), 5 desktop pairs and 3 phone triples

Transcribed from the run log; the raw JSON was overwritten by the next sweep, and this sweep predates the heartbeat. It is the cleaner view of frame rate and time to ready.

| measure | BEFORE desktop | AFTER desktop |
|---|---|---|
| ready, pixels seen, s | 2.63 (2.51 to 3.99) | 1.45 (1.26 to 2.04) |
| frames per second | 31.8 (31.1 to 32.3) | 60.0 (58.5 to 60.1) |
| frame interval median, ms | 30.8 (30.5 to 31.4) | 16.7 (16.7 to 16.7) |
| frame interval p95, ms | 36.6 (34.6 to 40.9) | 16.8 (16.8 to 17.9) |
| page rAF callback median, ms | 4.3 (4.2 to 4.4) | 0.2 (0.2 to 0.2) |
| host load (1 min) | 8 to 21 | 12 to 20 |

| measure | BEFORE phone | AFTER phone (native) | AFTER phone ?dpr=1 |
|---|---|---|---|
| ready, pixels seen, s | 13.18 (10.96 to 13.18) | 6.52 (5.91 to 6.57) | 6.59 (6.30 to 6.82) |
| frames per second | 48.9 (48.9 to 50.9) | 60.0 (60.0 to 60.1) | 60.0 (60.0 to 60.0) |
| frame interval median, ms | 19.1 (18.6 to 19.3) | 16.7 (16.7 to 16.7) | 16.7 (16.6 to 16.7) |
| frame interval p95, ms | 27.4 (26.7 to 28.6) | 17.6 (17.6 to 17.7) | 17.7 (17.7 to 17.7) |
| page rAF callback median, ms | 17.8 (17.3 to 18.1) | 1.0 (0.9 to 1.0) | 0.8 (0.6 to 1.1) |
| host load (1 min) | 11 to 18 | 10 to 20 | 9 to 17 |

## What the numbers say

- **Time to ready.** AFTER is ready in roughly half the time in both modes. In both pages nearly all of that time is one main-thread stall, not download.
- **AFTER's stall is its scene load and calibration burst** (`ha_load` then `ha_burst(20000)`, synchronous). A probe put it between shader setup (about 0.2 s) and the first `requestAnimationFrame` request: about 1.3 s on desktop and about 8 s under the 6x throttle.
- **Frame rate.** AFTER sits at the 60 Hz display cap, so its frame interval is censored at 16.7 ms and its headroom is not measured. BEFORE runs at about 31 frames per second on desktop.
- **Per-frame main-thread cost.** The rAF callback rows are the uncensored figure: about 0.2 ms against 4.3 ms on desktop, and about 1 ms against 18 to 28 ms on the throttled phone.
- **Transfer.** 0.91 MB against 31.85 MB uncompressed; 0.33 MB against 9.74 MB if gzipped.
- **Memory.** `usedJSHeapSize` appears to include wasm linear memory (BEFORE's heap is about 8 MB above its wasm memory in every run), so do not add the two rows.

## Fairness caveats that still apply

1. **SwiftShader.** All rasterisation is on the CPU. Absolute frame times are not a GPU device's; only the comparison between pages is measured.
2. **Throttle scope.** The 6x throttle slows the renderer main thread only. Rasterisation in the GPU process is not throttled, so a page whose cost is draw calls is under-penalised against one whose cost is JS or wasm.
3. **Loopback transfer.** The server sends uncompressed with no latency or bandwidth limit, so download time is nearly absent from time to ready. On a real network BEFORE would pay for about 9.7 MB compressed against 0.33 MB, so this understates AFTER's advantage.
4. **Different content.** BEFORE draws a full text HUD, two gate panels, a census, a device panel and a button row inside the canvas every frame. AFTER has one small HTML panel outside the canvas, with the same physics and the same calibration burst. This compares pages, not renderers alone.
5. **Pixel counts.** BEFORE's canvas is 390x844 on the phone (it ignores the device ratio); AFTER's is 1170x2532 natively, nine times the pixels. The `?dpr=1` column is the like-for-like one. AFTER held the cap at both sizes on the quiet host.
6. **The 60 Hz cap.** It hides how much faster than 16.7 ms AFTER could draw.
7. **Ready claims differ.** BEFORE hides its overlay two frames after starting, drawn or not; AFTER sets its flag after a drawn frame. Use the pixels row, which has a 200 ms polling grain plus capture cost. For BEFORE the two rows differ by up to about 0.3 s on desktop and about 1.6 s on the phone; for AFTER they agree to within 0.03 s.
8. **The pixel test is weak.** It passes on any non-flat canvas, not on a complete scene.
9. **The longtask API under-reports.** It missed AFTER's whole calibration stall (it reports about 0.1 s where the heartbeat shows over 1 s). The heartbeat rows are the ones to compare; the longtask row is kept only to show the disagreement.
10. **Host load.** It varied from 7 to 38 across sweeps. Interleaving spreads it over both pages but does not remove it. Single-run outliers (a 2.6 s frame gap in each page, ready times doubling) track load, not the page.
11. **Harness overhead.** The harness wraps `requestAnimationFrame` and `WebAssembly.instantiate*` and runs a 10 ms timer. The cost is small and the same on both pages.
12. **Scene.** One scene only: the default H2 two-atom preset. Nothing here says how either page scales with atom count.

Footnote: an earlier BEFORE measurement used the stale Sep 2 artifact (41.7 MB) with the page as of 46bfbd6. It gave ready 2.99 s desktop and 16.4 s phone at about 31 frames per second. It is superseded by the HEAD numbers above.

## Screenshots (first run of each mode, from sweep 3)

- `before_head_desktop_ready1s.png`, `before_head_desktop_end.png`
- `after_desktop_ready1s.png`, `after_desktop_end.png`
- `before_head_phone_ready1s.png`, `before_head_phone_end.png`
- `after_phone_ready1s.png`, `after_phone_end.png`
- `after_dpr1_phone_ready1s.png`, `after_dpr1_phone_end.png`

## Files

- `compare.json`: pooled cells with every run.
- `compare_sweep2_loaded.json`, `compare_sweep3.json`: the two sweeps separately.
- Harness: `engine/crates/holon-draw/tools/compare_pages.py`; interleaving driver: `run_pairs.py` in this directory.

---

*Recorded 2026-11-15 in the repository. Measured by `tools/compare_pages.py` driven by
`tools/run_pairs.py` (both written by a subagent of this session); the BEFORE root was a
scratch directory holding the current `docs/atoms3d/index.html` and the live site's webgl2
build, which is not committed. Screenshots beside this file: `before_head_desktop_end.png`,
`after_desktop_end.png`, `before_head_phone_end.png`, `after_phone_end.png`.*

**What this does not show.** One scene of two atoms; nothing about many atoms or a map.
A CPU rasteriser, not a GPU. The two pages do not carry the same content: Bevy draws its
energy and momentum gates, clocks, curve, census and device panels inside the canvas every
frame, and the new page shows a five-line HTML panel. Moving those panels to HTML beside the
new page (the host draws text, `INTERFACE.md` §1) is owed before the pages are equal in
content. The atoms appear mirrored between the two because Bevy's world is y-up and this
scene is z-up: the same simulation, seen with a different vertical axis.
