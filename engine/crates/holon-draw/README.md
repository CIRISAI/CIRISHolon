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
