# What the megabytes buy

Measured 2026-08-28 on this machine. `build-web.sh` prints these numbers on every run, so
they are reproducible rather than quoted.

## The artifacts

| | raw | gzip -9 | JS glue |
|---|---:|---:|---:|
| `docs/atoms3d/webgpu/atoms3d_bg.wasm` | 29,823,319 | 9,058,417 | 108,419 |
| `docs/atoms3d/webgl2/atoms3d_bg.wasm` | 30,871,507 | 9,458,927 | 105,881 |
| `docs/atoms/holon_render.wasm` (the 2D shell) | 117,643 | 27,566 | — (none) |

Built with `cargo build --release` (opt-level 3, thin LTO, one codegen unit, stripped),
then `wasm-bindgen --target web --no-typescript`, then
`wasm-opt -Oz --strip-debug --strip-producers`. Before `wasm-opt` the module is 45,319,482
bytes, so the size pass removes about a third of it.

gzip is the number that matters for a phone on a slow link — it is what the browser
actually downloads, and it is 3.3x smaller than the raw figure. Brotli, which GitHub Pages
serves when the client accepts it, is typically another 20-25% below gzip; it was not
measured here because `brotli` is not installed on this machine, and an estimate has no
business in a table of measurements.

## 253x is not a fair comparison, and here is the fair one

The 3D artifact is 253 times the 2D one. That ratio is true and it is also close to
meaningless, because the two are different kinds of object:

- **`docs/atoms` is a physics kernel.** A raw `extern "C"` cdylib with no wasm-bindgen,
  no glue generator, no renderer, no ECS, no window and no font. The browser owns input
  and pixels; the wasm owns the potential, the integrator, the ledger and the bond
  predicate, and it hands back scalars. 117 KB is what that costs.
- **`docs/atoms3d` is an application.** It carries the same physics — literally the same
  rlib — plus an ECS, a PBR renderer, a wgpu backend with its shader compiler, a winit
  event loop, a UI layout and text stack, and an embedded font. It draws its own pixels.

The physics is a rounding error in both. What the megabytes buy is not better physics; it
is a renderer, and the renderer is the reason you can orbit the box, see the bond as a
thing between two spheres, and pull an atom around in depth.

## Why webgl2 is BIGGER than webgpu

30.9 MB against 29.8 MB, despite the webgl2 build dropping bloom and `bevy_post_process`
entirely. The GLES backend in wgpu carries its own downlevel-capability handling and
workaround paths, and those outweigh the post-process pass that was removed. Recorded
because the intuition runs the other way — the fallback build is the *smaller-featured*
one — and somebody checking these numbers should not have to rediscover that the
intuition is wrong.

## Levers not pulled, and what they would cost

Stated rather than quietly skipped, so the next person can decide differently:

- **`opt-level = "s"` or `"z"`.** Would shrink the module, at the cost of the per-frame
  O(N²) force loop, which is the entire compute budget of the app. The calibration burst
  measures that loop on the user's own device and reports `N_max` from it, so a slower
  loop would show up directly as fewer atoms on every phone. Not taken.
- **Fat LTO instead of thin.** Plausibly a few percent, at a large build-time cost. Not
  measured; worth a try if the size ever becomes the binding constraint.
- **Dropping `tonemapping_luts`.** The AgX tonemapper needs its LUT textures, which pull
  KTX2 and therefore zstd. Switching to a LUT-free tonemapper (Reinhard) would drop both,
  for maybe 1 MB raw, and would change how the scene looks. Not taken, because the look
  is shared with the 2D shell's identity and a megabyte is not worth it.
- **Splitting the physics out as a second, tiny wasm.** Would not help: the physics is
  already a rounding error here, and the split would add a boundary between the ledger and
  the thing that draws it.

## Consequence for the repository

At 30 MB each these artifacts are **not committed**. `.github/workflows/pages.yml` builds
them on every deploy and `.gitignore` keeps them out; only `docs/atoms3d/index.html` is
record. Committing them would put 60 MB into git history per rebuild, permanently. The 2D
shell's 117 KB artifact *is* committed, because at that size having the exact deployed
bytes in the record is worth the diff noise.
