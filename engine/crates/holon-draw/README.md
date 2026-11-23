# holon-draw

A small generic draw core with no dependency (`INTERFACE.md` at the repository's root is its
contract). It packs bytes and answers questions; it does not own a canvas and has no
graphics backend of its own yet.

- `lib.rs`: tiles of tagged 52-byte instance records, packed one batch per tile and kind
  (images one batch per texture slot); a pure pick that names an instance; a report per
  instance, with what the core can attest of its legibility (`Shown::why`) and the SHA-256
  of the source string its text was shaped from (`Shown::source`); a digest of a frame.
- `ui.rs`: panels, text and buttons as screen tiles. Tagged text carries its source digest.
- `digest.rs`: SHA-256, so a source digest is comparable with the screen standard's.
- `registry/` and `tools/check_registry.py`: the screen standard's registry, vendored at a
  pinned tag, and the check of `INTERFACE.md` §9 against it (run by ci-gates).
- `alloc.rs`: the detail allocator behind one interface, with a distance ring and a
  read-per-work rule. Neither has been compared on a consumer's scene.
- `twin.rs`: the CEWP twin's street leaf as the twin's own vertex stream (10 floats a
  vertex), built on this side of the seam. A compatibility backend: the twin's program,
  uniforms and draw order are untouched.
- `abi.rs`: the wasm boundary, plain `extern "C"`.

## A matrix per batch and additive blending: what was built and measured, 2026-11-23

Two changes to both browser backends, both asked for by the CEWP twin's move to drawing only
through holon-draw. The contract is in `web/draw.js`'s header (BATCH MATRIX, BLEND, GL STATE,
WHAT A BATCH COSTS IN GL CALLS) and in `INTERFACE.md` §15. `lib.rs` is unchanged.

- **A matrix per batch.** A world batch may carry `vp` (16 numbers, column-major) and
  `pixelsPerUnit`, and is then drawn with them in place of the call's. The call's `vp` may be
  null when every world batch has its own. Everything derived from the matrix (the eye, the
  camera-facing glyph squares, the estimated pixels per unit) is taken from the batch's own.
  Screen batches ignore both fields. `draw()` throws, before it changes any state, on a world
  batch with no matrix at all, on a `vp` of fewer than 16 numbers, and on any batch's own `vp`
  in a call with `opts.shadow`. Shadows are refused rather than half-supported: a batch's
  matrix does not say where it is in the light's world. On WebGPU a batch's matrix goes in a
  frame slot of its own in the uniform buffer, so the shaders are unchanged.
- **Cheaper batches.** In GL every uniform now goes through a record of what the program
  holds, and an upload is skipped when the numbers are the same, across `draw()` calls too.
  The tile settings are not even compared when a batch's `params` is the same object as the
  batch before's. So switching matrix is one `uniformMatrix4fv`, plus the eye's `uniform4f`
  when the eye moves (it does not between flat-map tiles).
- **`blend: 'add'`**, per batch, world or screen, resident or not. The colour adds (source rgb
  times source alpha) and the destination alpha is kept. An additive batch writes no depth and
  casts no shadow. GL switches with `blendFuncSeparate` and `depthMask` only where consecutive
  batches differ, and `draw()` returns with the frame's blend and `depthMask(true)` in every
  case. WebGPU makes additive pipeline variants on first use.

**Checked with `tools/webgpu_parity.py`** (SwiftShader in headless Chrome 152, 960 x 600):

- **New mode `tiles`** (48 flat-map tiles, 5 batches each, every tile with its own affine,
  pixelsPerUnit, settings and the RING). One `draw()` with a matrix per batch is identical (max
  difference 0) to one `draw()` per tile with the twin's GL restore after each. That holds
  resident and `bytes`, panels on and off, on WebGL 2 and on WebGPU, and on WebGL 1, mediump,
  WebGL 1 with mediump, the no-highp gate and bare WebGL 1 (gl error 0 on each).
- **New mode `tiles3d`** (42 perspective tiles, each tile's matrix the camera's times its own
  translation, with a METAL box, a sphere, a rod, a tube and two glyphs per tile). The same
  checks give the same result, max 0 everywhere.
- **Resident tiles** upload no record (`bytesUploaded` 0 with the panels off).
- **Refusals**, on both backends in both tiles modes: `draw()` throws on a batch's own `vp`
  with `opts.shadow`, on `blend: 'screen'`, on a 12-number `vp`, and on a world batch with no
  matrix in a call with none. A batch's own `pixelsPerUnit` with shadows draws without a throw.
  Its picture is NOT checked: no mode draws shadows with a per-batch `pixelsPerUnit`.
- **WebGL against WebGPU, one draw() each:** `tiles` mean absolute difference 0.147, max 42;
  `tiles3d` 0.174, max 63. These are reported, not gated, as for the other modes.
- **New mode `add`.** Two overlapping additive discs (rgb 200 40 20 at alpha 128, and 30 120
  200 at 255) are drawn over a dark ground and a bright one, as world batches and as screen
  batches. At all 17 points checked the screenshot is the arithmetic's colour (destination
  plus round(source x alpha), clamped at 255, one add after the other) to within 0 on both
  backends. That includes a quad drawn after an additive disc nearer the eye, which shows, so
  the disc wrote no depth. On WebGL the blend state read back after `draw()` is (SRC_ALPHA,
  ONE_MINUS_SRC_ALPHA, ONE, ONE_MINUS_SRC_ALPHA), FUNC_ADD, depth writes on. That holds after
  a call whose last batch was an additive world batch, and after one whose last was an
  additive screen batch. Resident equals `bytes` (max 0), and WebGL 1, mediump, the no-highp
  gate and bare WebGL 1 pass the same points.
- **Plants.** With an additive batch still writing depth, the depth point fails on every GL
  page and on WebGPU (white 255 255 255 where 90 160 110 was due). With the first batch's
  matrix used for every batch (GL), or no frame of its own (WebGPU), every `tiles` comparison
  fails (max 220).
- **Every earlier mode still passes**, and the cross-backend numbers of each are what they were.
  All 45 screenshots that origin/main's (`f2cca7c`) harness wrote are pixel-identical to this
  branch's, compared decoded, pixel for pixel. They cover the 9 earlier modes, panels on and
  off, on both backends, and the WebGL 1, mediump, no-highp and bare-WebGL-1 variants.

**Measured on the machine's GPU** (Chrome 152 through EGL, ANGLE on Mesa Intel RPL-S, 1280 x
800, vsync and the frame limit off; `tools/tiles_timing.py`, page `web/bench-tiles.html`). N
flat-map tiles of 11 records in 5 batches each, every tile resident in its own buffer. Three
arms were interleaved, each run in a fresh page, 5 runs each, 3 s to settle and 6 s measured:

- `old`: one `draw()` per tile with the draw.js before this change, plus the twin's 4-call
  restore after each. This is the twin's way today.
- `percall`: the same with this draw.js.
- `one`: a single `draw()`, each batch with its tile's `vp` and `pixelsPerUnit`.

The page was cross-origin isolated, so its timers were not coarsened to 0.1 ms. "Drawing" is
the script time spent issuing the frame's calls (performance.now() around them). It is not
the GPU's time. "Frame mean" is the measured time over the frames counted.

GL calls a frame, counted with every method of the context wrapped:

| tiles | old | percall | one |
|---|---|---|---|
| 8 | 411 (51.4 a tile) | 307 (38.4) | 170 (21.2) |
| 44 | 2 247 (51.1) | 1 675 (38.1) | 854 (19.4) |
| 200 | 10 203 (51.0) | 7 603 (38.0) | 3 818 (19.1; 3.8 a batch) |

Times in ms. Each cell gives the median of the 5 runs' values, then the lowest and highest run:

| tiles | arm | frame median | frame p95 | frame mean | drawing median |
|---|---|---|---|---|---|
| 8 | old | 0.090 (0.090-0.095) | 0.260 (0.235-0.275) | 1.007 (0.999-1.012) | 0.035 (0.035-0.040) |
| 8 | percall | 0.085 (0.080-0.090) | 0.215 (0.195-0.240) | 0.989 (0.982-1.000) | 0.025 (0.020-0.035) |
| 8 | one | 0.075 (0.070-0.075) | 0.180 (0.160-0.185) | 0.983 (0.963-0.994) | 0.015 (0.015-0.020) |
| 44 | old | 0.235 (0.225-0.350) | 14.805 (14.190-15.565) | 1.574 (1.551-1.623) | 0.180 (0.165-0.225) |
| 44 | percall | 0.190 (0.170-0.250) | 10.440 (6.595-11.215) | 1.529 (1.516-1.584) | 0.115 (0.110-0.185) |
| 44 | one | 0.155 (0.145-0.190) | 0.825 (0.620-0.960) | 1.506 (1.484-1.552) | 0.100 (0.095-0.110) |
| 200 | old | 1.272 (1.210-1.380) | 7.295 (7.160-10.405) | 2.538 (2.412-2.683) | 1.205 (1.135-1.310) |
| 200 | percall | 0.960 (0.940-1.055) | 10.100 (9.770-15.075) | 2.290 (2.194-3.005) | 0.885 (0.845-0.965) |
| 200 | one | 0.755 (0.740-0.780) | 14.275 (14.050-14.715) | 2.149 (2.121-2.274) | 0.685 (0.675-0.715) |

What this says, and what it does not:

- **Median frame.** One call against the old way per tile is 0.83 x at 8 tiles, 0.66 x at 44
  and 0.59 x at 200. The script time spent drawing drops by 43 to 57 %. At every N, the
  highest `one` run is below the lowest `old` run. Against `percall` (the new draw.js, one
  call per tile) the ranges overlap at 44 tiles.
- **Mean frame.** The ratios are 0.98, 0.96 and 0.85. Most of a frame here is Chrome's own:
  about 1 ms at 8 tiles, where the tiles cost almost nothing.
- **The p95 is NOT better everywhere.** At 200 tiles it is worse in one call (14.3 ms, against
  10.1 per call and 7.3 old). At 44 tiles it is far better (0.8 against 10.4 and 14.8). In
  every arm the p95 is a periodic stall inside the draw calls: the drawing p95 equals the frame
  p95, which looks like the GPU process pushing back on a full command queue. When it lands
  depends on how fast frames are issued. This test does not say which arm stalls less in a
  real page.
- **The machine is shared.** The load average was 3 to 9 during the run, and 40 earlier in the
  day. The spread is printed above for that reason.
- **This is not the twin's stake.** The twin's witness (its `tools/page/hd_timing/run.py`, at
  most 1.10 x its own program's frame) needs the twin to draw its tiles in one call. That
  change is in the CEWP repository and is not made here, so the stake is not yet answered.
  Per tile, one call saved about 2.6 µs of script time (200 tiles: 1.205 to 0.685 ms) and
  about 2 µs of mean frame (2.538 to 2.149 ms). The twin's village stops were 0.6 to 1.2 ms
  slower. If those stops hold a few hundred tiles, this explains part of the gap, not all of
  it. Its number of tiles there was not measured here.

## Glyph shapes, FLOW on links, and the RING per pixel: what was built and checked, 2026-11-22

Three changes, in both browser backends, with one contract (`web/draw.js`'s header: GLYPHS,
FLOW, TILE SETTINGS; `INTERFACE.md` §14).

- **Glyph shapes.** The low four bits of a glyph's `variant` are now its shape, from a closed
  list of 16 (`GLYPH` in draw.js, `GLYPH_*` and `VARIANT_GLYPH_SHAPE` here). Shape 0 is the
  old disc and is unchanged. Shapes 1..15 are a square of half size `size[0]`, turned by
  `angle`. In the world the square faces the camera, is unlit, has no shadow, and writes no
  depth where it covers under half. Each shape is cut from signed distances, in a body tone
  and a detail ink. The shapes are written once, in a GLSL subset (`GLYPH_GLSL`), and
  draw-webgpu.js translates that source to WGSL mechanically.
- **FLOW.** `VARIANT_FLOW` (0x04) on a line or arc, world or screen, draws dashes moving from
  `pos` to `aux`. Their phase and count are tile settings 22 and 23 (`PARAM_FLOW`,
  `PARAM_FLOW_N`), which nothing read before. An arc's dashes are spaced along the curve as
  drawn (Simpson's rule in the vertex stage).
- **The RING, per pixel (a CORRECTION).** The header said the ring was taken per pixel, but
  it was taken per vertex. It is now taken from the fragment's own window position. In the
  shadow pass it is now the camera's ring at the caster's own point; before, it was measured
  in the shadow map's own pixels.

`tools/webgpu_parity.py` gained two modes. `icons` draws the 16 shapes in the world (two
rows, two sizes, light and dark bodies, some turned), and on a screen panel at 16 and 48 px.
It also draws flow links: tubes, bent arcs, flat ribbons, a screen line, and one arc from a
resident buffer. Under all of it is one 400-unit ground patch with the ring on. `iconsmove`
is the same scene at another phase. At 960x600, on SwiftShader in headless Chrome, the
results were:

- **Old pictures are unchanged.** All 32 screenshots of the seven earlier modes are
  byte-identical to origin/main's (`e822856`), on both backends, including WebGL 1 with and
  without vertex arrays. Six further views are identical too: `?gl=1`, `?mediump=1`,
  `?gl=1&mediump=1`, `?nohigh=1`, and `?ortho=1` on both backends. The ring views changed
  only by rounding. `?map=1&ring=1` differs from before by at most 1 (of 255) on both
  backends, in 1 241 pixels (0.22 %), all within the ring's fade band. `?gl=1&nohigh=1&map=1&ring=1`
  differs by at most 1 in 922 pixels. Under that view's orthographic map the old
  interpolation was already exact.
- **Parity, WebGL 2 against WebGPU.** For `icons`, mean_abs is 0.048 without panels and
  0.215 with them. The share of pixels over 8 is 0.23 % and 0.39 %. The max difference is 60
  without panels; with panels it is 200, from the panel's own text, which names its backend.
  Draw calls are 12/12 and 30/30, and instances 48/48 and 149/149. The earlier modes' numbers
  are exactly as before.
- **Flow.** The resident arc was drawn alone at three phases (0.25, 0.62 and 1000.87).
  `bytesUploaded` was 0 each time, with one `paramUploads` per draw, on both backends. At
  phase 0.75 against 0.25 the picture differs, with max 155 and 1.70 % of pixels over 8, on
  both backends.
- **The huge patch is drawn.** Three pixels inside the ring are green, not the clear colour,
  on both backends. Before this change such a patch vanished.
- **WebGL 1 and mediump.** `icons` with the panels drew with gl error 0 on `?gl=1`,
  `?mediump=1`, `?gl=1&mediump=1`, `?nohigh=1` (no fragment source still says highp) and
  `?gl=1&novao=1`. The `mediump=1` and `nohigh=1` pictures are identical to WebGL 2's. On
  WebGL 1 the only difference is the panel's context label (max 200, 0.07 % of pixels). The
  world program with shadows now uses eight varying vectors, which is WebGL 1's guaranteed
  minimum.
- **The shadow pass's ring.** `?ring=1` without `?map=1` puts a small ring on the 3D scene's
  lit marks. The marks the picture fades out no longer cast shadows; this was checked by
  looking at the picture, on both backends.
- **A real GPU, once.** Chrome with `--use-angle=gl-egl` ran WebGL on ANGLE (Intel, Mesa,
  RPL-S, OpenGL ES 3.2). `test.html?icons=1` and `?icons=1&gl=1` drew with gl error 0, and
  the resident flow check held. On that launch `test-webgpu.html?icons=1` got SwiftShader,
  not the GPU, as its adapter, so WebGPU on hardware was not checked.
- **The core.** `cargo test -p holon-draw` passes 45 tests. Two are new: the shape bits never
  touch the colour mode, settings 22 and 23 were free, and a glyph of shape 1..15 is bound
  by its square's corner (`size[0]` times the square root of 2) and, on screen, by its turned
  square's box.

Which shapes are weak at 16 px: KEY (thin, especially when turned), STOP (its octagon reads
as a disc with a bar, which is a no-entry sign), BALLOT (the slot and tick are about one
pixel), and DOCUMENT's three text lines (about one pixel each, with gaps of about 1.5 px).
The rest read clearly at 16 px. All 16 look clean at 48 and 200 px.

Not done: the right-to-left mirror flag that `INTERFACE.md` §6 owes. No frame time was
measured. Also found, and not fixed: on a large mark the procedural FIELD's pixels-per-unit
(`v_fp`) is still estimated per vertex, so a vertex behind the eye skews its octave fade.
The `icons` patch therefore runs with the field off.

## Resident buffers: what was built and checked, 2026-11-22

Both browser backends can now draw a batch from records already on the GPU. A host calls
`resident(bytes)` to get a handle, `update(handle, bytes)` to change the records and
`release(handle)` to free them. A batch then names `{ handle, first, count }` in place of
`bytes`, and nothing is uploaded for it. The contract is in `web/draw.js`'s header (RESIDENT
BUFFERS, and the GL STATE section). It is the design on `wip/holon-draw-2026-11-16`, ported
onto the current file. A resident world batch casts shadows like any other world batch, and
`castShadow === false` still turns that off. A resident image batch must carry `slot`,
because the slot byte of its first record is on the GPU and the drawer cannot read it. On
WebGPU a handle is one GPUBuffer, and a resident batch is drawn with `setVertexBuffer(1,
buf, first * 52)`.

`tools/webgpu_parity.py` gained three modes, `resident`, `residentmap` and `residentimages`.
Each draws the default, map or images scene with its world batches in two resident buffers.
In `residentimages` the image panel goes into a third buffer. Each mode is gated against the
same page drawn from `bytes`, on the same backend. At 960x600 the results were:

- **Pixels.** Every resident screenshot is identical to its `bytes` screenshot (largest
  difference 0). That holds for 10 page pairs: 5 scenes on WebGL 2 and on WebGPU, plus the
  resident scene with panels on WebGL 1 with vertex arrays and on bare WebGL 1 without them.
- **Shadows.** The resident scene differs from `?shadow=0` by up to 176 (4.67 % of pixels
  over 8), so its batches do cast shadows.
- **Uploads.** `bytesUploaded` is 0 for the resident scene, down from 14 664 bytes, and 0 for
  the resident map, down from 35 828. With the panels on, it is exactly the panels' bytes:
  3 328, or 3 432 with the images. WebGL and WebGPU agree with each other as in the other
  modes, and draw calls and instances are equal.
- **The page's own checks.** `update()` grows a buffer by reallocating and shrinks one in
  place, keeping its allocation (`residentBytes` 14 820 = 14 664 + the 3 junk records it
  grew by). A released handle number is handed out again. `draw()` throws on a range past
  the shrunk end, on a released handle, and on a resident image batch with no `slot`.
- **The `bytes` path is unchanged.** The 16 screenshots of the four earlier modes are
  byte-identical to origin/main's.

As everywhere above, this is SwiftShader in headless Chrome. No frame time was measured, and
the twin's timing test was not run.

## Issue #6, Profile B's owed items: what was built and checked, 2026-11-22

`INTERFACE.md` §13 has the contract. In short:

- **Images (D12, `Kind::Image`).** A host puts a texture in a slot with `setImage(slot, source,
  { nearest })` on either backend. A tagged image must be `Bind::Media` (its alt text and AI
  disclosure each drawn in the same frame as a run with a source string) or `Bind::Qr`;
  otherwise the frame is refused (L10). An image in a world tile is refused, because no
  backend draws one there.
- **Source strings (L12).** A run carries the SHA-256 of the string it was shaped from, and
  what the provider said of the drawing (`Drawn`: whole, cut, changed, or nothing said). The
  report returns the digest when every run of the instance carries the same one.
- **Legibility (L12).** `Shown::legible` now means every condition the core can attest held:
  wholly in the viewport, the size floor, unoccluded, contrast of at least 4.5:1 (WCAG AA)
  against an opaque ground the core can see whole, untruncated, and one source string.
  `Shown::why` gives each part. A world mark is never legible, because world-against-world
  occlusion needs the backend's depth buffer.
- **Checked.** The crate's tests cover each condition and each refusal. Five planted faults
  were each caught when written: own marks occluding, no contrast floor, a translucent ground
  taken as opaque, an unsaid drawing taken as whole, and media slots not checked. The registry
  check refused seven planted faults. `tools/webgpu_parity.py` gained an `images` mode (a
  nearest-sampled QR-like grid, a translucent linear gradient, and an image naming an empty
  slot). At 960x600 with the panels, WebGL and WebGPU agree to a mean |diff| of 0.425 of 255,
  1.26 % of pixels over 8, the same as the plain overlay. Draw calls (39) and instances (574)
  are equal, and both skip the one image with no texture. The three earlier modes still pass.
  This is SwiftShader in headless Chrome only.
- **Fixed on the way.** Both backends grew their per-kind instance buffers for kinds 1 to 11
  only, so a twelfth kind would have uploaded to no buffer. A screen line's report and pick
  rectangle were a box of `size` about one end; they are now its two ends and its width.

## The WebGPU backend: what was measured, 2026-11-20

`web/draw-webgpu.js` is a second browser backend beside `web/draw.js` (WebGL 1 and 2), under
the same contract: the contract is `draw.js`'s header, and the same batches draw on either
(`createDrawerWebGPU(device, { format, text, sampleCount })`, then `draw(batches, vp, opts)`
with draw.js's options plus `target` and `clear`). The records are uploaded as they are into
instance vertex buffers; the vertex, fragment and shadow stages are WGSL ports of the GLSL,
on copies of the same meshes. Without `opts.shadow` no shadow pipeline is made or bound.
`web/text.js` gains `createTextProviderWebGPU(device, opts)`, the same provider (shared code)
with r8unorm pages; `createTextProvider(gl, opts)` is unchanged for WebGL hosts.
`web/test-webgpu.html` draws `web/test.html`'s scene, now shared from `web/test-scene.js`.

`tools/webgpu_parity.py` draws both test pages in one headless Chrome (156) at 960 x 600, dpr 1,
and compares the canvases. Both are 4x multisampled. Per channel, 0..255:

| mode | panels | mean abs. difference | pixels over 8 | pixels over 32 | largest |
|---|---|---|---|---|---|
| scene, shadows on | off | 0.33 | 1.34 % | 0.57 % | 117 |
| scene, shadows on | on | 0.44 | 1.35 % | 0.60 % | 200 |
| `?map=1` | off | 0.29 | 1.75 % | 0.12 % | 66 |
| `?map=1` | on | 0.35 | 1.36 % | 0.21 % | 200 |
| `?shadow=0` | off | 0.34 | 1.41 % | 0.60 % | 117 |
| `?shadow=0` | on | 0.46 | 1.42 % | 0.63 % | 200 |

Draw calls and instances are equal in every mode. The numbers were the same in two runs.

What this does and does not say:

- The differing pixels are on the edges of marks only (the diff pictures the script writes
  show outlines, no filled area): where 4x coverage is resolved, the two paths (ANGLE's GL
  and Dawn's Vulkan, both on SwiftShader) differ. The insides agree, which includes the
  lighting, the materials, the ground, canopy and roof fields (the twin's hash), the ring,
  the class alphas and the shadow lookup. Drawn at one sample (`?msaa=0`) against the 4x
  WebGL page, the scene's mean difference goes from 0.33 to 0.74, which is what says the
  remainder is coverage and not shading.
- With the panels on, the largest difference (200) is the panel's own words: its rows say
  "WebGL 2.0" and "gl error 0" on one page, "WebGPU swiftshader" and "gpu errors 0" on the
  other. The other strings, in every script, are the same pixels.
- Everything here is SwiftShader: CPU rasterisation on both sides. It says the WGSL computes
  what the GLSL does on one software rasteriser. No hardware GPU, and no browser but
  Chrome, has drawn the WebGPU backend yet.
- Frames per second with `?spin=1` for 4 s, same browser: scene with shadows 7.9 (WebGL)
  and 6.4 (WebGPU); `?map=1` 10.4 and 9.8; `?shadow=0` 8.8 and 8.4. These are SwiftShader's
  numbers and say nothing about any device's frame time. Why the WebGPU page is slower
  there has not been looked into.
- Not shown: a host's own `depthTarget`; a `viewport` that is not the whole target; a target
  that is a view with `size`; a provider page emptied and refilled while a run batch is drawn
  across frames on WebGPU (eviction was run once, `?page=256&pages=2`, and drew the same).

To repeat it (headless WebGPU on Linux needed `--use-vulkan=swiftshader`; the script's
docstring says which flags and why):

```
python3 crates/holon-draw/tools/webgpu_parity.py --fps 4   # every mode, the resident ones included
```

## The twin's leaf builder: what was measured, 2026-11-15

`tools/twin_leaf_check.mjs` runs the twin's own `tlBuildLeaf`, read from the twin's tree at
run time (its `docs/tiling.js` at `bb58ce2`), against this crate on synthetic layout buffers
made by the script.

| | |
|---|---|
| leaves compared | 2,000 synthetic (0 to 8 roads, 0 to 39 buildings, 8 household levels, tones on two thirds) |
| floats compared | 27,090,780 (2,709,078 vertices) |
| floats differing | 0, through the native example and through the wasm artifact |
| plant (a wrong household size given to this side) | the check fails |
| wasm artifact | 51,044 bytes; 20,261 gzipped (stripped, link-time optimised) |
| time per leaf, the twin's JavaScript | 0.288 ms |
| time per leaf, this crate in wasm, including copying the buffers in and the result out | 0.042 ms |
| ratio | 6.8 |

What this does and does not say:

- The same bytes on SYNTHETIC buffers. The twin's kernel was replaced by a stub, so this
  checks the builder, not the kernel, and real leaves have not been compared.
- The times are node on one desktop core, unthrottled, a median of 7 passes. The twin's
  protocol is a phone-sized page at 6 times throttle with a 200 ms longest-task budget; that
  has not been run, and only the twin can run it.
- Building the leaf is one of the costs the twin's renderer has. Re-joining a tile's buffer
  when its leaves change, the per-frame mosaic and the upload are untouched by this.
- The plant is weak: a wrong household size changes only leaves near a density threshold
  (1 of 300 in a short run). A plant that changes every leaf is owed.

To repeat it:

```
cargo build --release -p holon-draw --example twin_leaf
node crates/holon-draw/tools/twin_leaf_check.mjs <twin>/docs/tiling.js target/release/examples/twin_leaf 2000
```
