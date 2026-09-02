# workbench-engine — lane resume

*Written 2026-09-02 after a status request found this lane had no resume. Detached-compute
rule: a session death must only kill narration, never computation or context. Everything
below is verifiable from the tree; nothing here is the only copy of anything.*

## Verdict

The workbench is a REAL ENGINE PAGE, gated, at `docs/workbench/`. It is NOT yet the site.
Route B (the 3D renderer split) is **not started**; its two feasibility unknowns are
closed. Both CI gates this lane owns are green on main.

## What is landed

| what | where | state |
|---|---|---|
| real engine page (replaced the WB-7.1 mock) | `docs/workbench/{index,app,styles,smoke}` + committed `holon_render.wasm` | live, 87 checks |
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
* **17b** — `node docs/workbench/smoke.mjs`, 87 checks, ~30 s. Runs the SHIPPED artifact.

Both verified green on main at 508ea0b.

## Next: Route B (ruled, not started)

Ruling: cdylib stays the single authoritative Sim; Bevy becomes a pure renderer fed a
per-frame buffer. **One Sim** is the hard requirement — "one drawn, one instrumented" is
disqualifying.

* `world.rs` — stop owning `Box<Sim>`, receive a frame buffer. 54 Sim sites.
* `pick.rs` — delegate the hand to `holon_grab`/`holon_move_anchor_3d`/`holon_release`. 5 sites.
* `hud.rs` — retires from this page; the JS overlay IS the HUD. 18 sites.
* `scene.rs`/`bonds.rs`/`render.rs`/`lighting.rs` — **zero** Sim sites; the drawing layer
  already consumes buffers. This is why route B is the tractable one.

Feasibility CLOSED before starting: the crate compiles and its artifact builds end to end
(webgpu 40,655,672 B / webgl2 41,687,973 B, 9.9 MB gz, unoptimised — no wasm-opt on this
box); buffer extraction costs 0.2–0.6 ms at 64–98 atoms, ~1–3.5% of a 60 fps budget.

Verification will be BUILD-PLUS-BROWSER, not native-test: `cargo test` for that crate
cannot run here (default features need wayland-sys; no Wayland headers). Gate 15 covers
headless in CI.

## Blocking the site-root promotion (§9c)

1. Route B split — not started (above).
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
