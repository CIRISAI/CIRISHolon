# workbench-engine — lane resume

*Written 2026-09-02 after a status request found this lane had no resume. Detached-compute
rule: a session death must only kill narration, never computation or context. Everything
below is verifiable from the tree; nothing here is the only copy of anything.*

## Verdict

The workbench is a REAL ENGINE PAGE, gated, at `docs/workbench/`. It is NOT yet the site.
Route B (the 3D renderer split) is **two increments in**: the renderer's two waists exist
and are gated — it consumes a `FrameBuffer` and produces a `HandIntent`, and the drawing
and interaction layers name no `Sim` at all. What remains is the workbench-side producer
and sink (JS), `hud.rs` retiring, and R7. Both CI gates this lane owns are green on main.

## What is landed

| what | where | state |
|---|---|---|
| real engine page (replaced the WB-7.1 mock) | `docs/workbench/{index,app,styles,smoke}` + committed `holon_render.wasm` | live, 149 checks |
| gravity, WB-2.4 + WB-2.4c world VECTOR | `holon-render/src/sim.rs` `set_gravity_vec`, `tests/gravity.rs` 11/11 | live |
| WB-2.2 pressure panel on the box-scale door | page + `holon_box_scale`/`holon_pressure` | live |
| scale ladder + §9c acuity law | `LADDER` / `acuityPopulation` in `app.js` | live |
| water story, six RECORD citations, gate-verified | `RECORD` in `app.js` | live |
| 3D shell compile fix + gate 15b | `holon-render-3d/src/hud.rs`, `engine/ci-gates.sh` | live |
| 3D build-failure stamp + fence M11 | `.github/workflows/pages.yml`, `docs/atoms3d/index.html`, `FENCES.md` | live |

Commits: 245f601, 8554c14, 0b79dcf, 46bfbd6 (plus earlier: the mock replacement, WB-2.4,
WB-2.4c, the water story, the ladder).

## The two gates this lane owns

* **15b** — `cargo check -p holon-render-3d --target wasm32-unknown-unknown --features render`.
  Exists because gate 15 runs `--features headless`, which is structurally blind to
  `hud.rs`/`pick.rs`/`render.rs`, and `pages.yml` swallows build failures. Those two greens
  hid a two-day compile break.
* **17b** — `node docs/workbench/smoke.mjs`, 149 checks, ~30 s. Runs the SHIPPED artifact.

Both verified green at 23fe9e6 plus this increment: 15b clean, 17b 149/149, and gate 15 headless now 27 tests (15 + 5 frame buffer + 7 hand intent).

## Route B (ruled; increments 1 and 2 landed)

Ruling: cdylib stays the single authoritative Sim; Bevy becomes a pure renderer fed a
per-frame buffer. **One Sim** is the hard requirement — "one drawn, one instrumented" is
disqualifying. The shape is ONE CONSUMER, TWO PRODUCERS: each page owns one Sim and the
renderer owns neither.

Two waists, at the two ends of the frame:

| waist | direction | type | producer / sink today | workbench's |
|---|---|---|---|---|
| `src/frame.rs` | engine → drawing | `FrameBuffer` | `AtomWorld::fill_frame` | JS, from the cdylib |
| `src/hand.rs` | interaction → engine | `HandIntent` | `HandIntent::apply_to` | JS, onto the three doors |

System order is now `calibrate → drag_atom → apply_hand_intent → advance_world →
fill_frame_from_world → sync_atoms → sync_bonds`. The workbench replaces exactly two of
those systems — the sink and the fill — and changes nothing else.

* **increment 1 (44c5673)** — `frame.rs`, `AtomWorld::fill_frame`, `scene.rs`/`bonds.rs`
  converted, 5 headless contract tests.
* **increment 2 (this)** — `hand.rs`, `pick.rs` converted, the sink system, 7 headless
  contract tests. The picker resolves against the buffer that was last DRAWN, which is
  what the user aimed at.
* `hud.rs` — **still holds the Sim**, 18 sites. Retires from the workbench page (the JS
  overlay IS that page's HUD); it stays on atoms3d. Named as an exclusion inside
  `tests/hand_intent.rs` so the debt sits in the file that would otherwise hide it.
* `world.rs` — keeps its `Box<Sim>` and SHOULD: it is atoms3d's producer. The workbench
  never constructs one.
* R7 (one Sim on the workbench page) — page-scoped, in the page's own gate, and now
  statable: "the drawn scene is fed only by the cdylib's buffer" is a property about one
  producer rather than a claim about ownership.

Feasibility CLOSED before starting: the crate compiles and its artifact builds end to end
(webgpu 40,655,672 B / webgl2 41,687,973 B, 9.9 MB gz, unoptimised — no wasm-opt on this
box); buffer extraction costs 0.2–0.6 ms at 64–98 atoms, ~1–3.5% of a 60 fps budget.

Verification is NATIVE-TEST plus build, corrected from an earlier note here that said it
could not be: `cargo test --no-default-features --features headless` runs this crate's
suite on this box in ~100 s (27 tests). It is only the DEFAULT features that need
wayland-sys, and the headless feature set is exactly what gate 15 runs — so Route B's
contract is checked by the gate that was blind to the render-only modules.

## Blocking the site-root promotion (§9c)

1. Route B split — two increments in, workbench-side producer/sink still owed (above).
2. `pages.yml` serving the workbench at root — not done.
3. The 1 km cube hero itself — the ladder and acuity law are live, the CUBE is not; the
   scene is a molecular box.
4. The retirement battery — "the old UI retires when the workbench is green under its full
   gate battery" needs that battery enumerated. Not yet defined.

Not blockers, by design: the three coarse bands are FENCED with owner and exit; that is
the ladder's story, not a gap.

## Standing hazards this lane has hit

* Never land during an open merge, by any mechanism — the completing merge silently drops
  the interposed commit. Park instead.
* `git diff HEAD -- <file>` before any pathspec commit; the index goes stale after a
  private-index landing.
* Plant every check. Five of this lane's own checks passed while establishing nothing
  (empty-string match, existence-not-tracking, indistinguishable failures, a threshold at
  zero, a readout that could not move).
