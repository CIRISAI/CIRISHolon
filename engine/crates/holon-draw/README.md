# holon-draw

A small generic draw core with no dependency (`INTERFACE.md` at the repository's root is its
contract). It packs bytes and answers questions; it does not own a canvas and has no
graphics backend of its own yet.

- `lib.rs`: tiles of tagged 52-byte instance records, packed one batch per tile and kind; a
  pure pick that names an instance; a report per instance; a digest of a frame.
- `alloc.rs`: the detail allocator behind one interface, with a distance ring and a
  read-per-work rule. Neither has been compared on a consumer's scene.
- `twin.rs`: the CEWP twin's street leaf as the twin's own vertex stream (10 floats a
  vertex), built on this side of the seam. A compatibility backend: the twin's program,
  uniforms and draw order are untouched.
- `abi.rs`: the wasm boundary, plain `extern "C"`.

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
