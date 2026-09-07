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
| **the surface** (FSD §11.5, 2026-09-02 evening): cube + four selectors + tier rail as the zoom + ☰ | `index.html` (header/tier-rail/control-rail/MORE CONTROLS card), `styles.css` (dock + four-tier pill deleted), `app.js` (`renderTierRail`, `renderSizeAxis`, `renderMixChips`) | live, 367 checks |
| **shipped artifacts through the doors**: `tables/HO.json` and `tables/O2.json` via `holon_bank_table_*`, `tables/s2_water_table.txt` via the water door; SHA-256 pins in `SHIPPED`, checked by smoke against the tree | `app.js` `SHIPPED`/`fetchPinned`/`pushShippedPair`/`pushWater`; engine `holon_water_table_alloc/load`, `holon_water_node`, `tests/water_door.rs` | live — **O:2H steps**; smoke 376/376 |
| **the solver, opened by the O–O curve**: thick-restart Davidson (16 Ritz vectors, images carried) + pair-sized sharding; stage timing behind `HOLON_SOLVE_TIMING`; warm start measured harmful, kept behind `HOLON_WARM_START` | `holon-chem/src/tier.rs` `DAVIDSON_RESTART_KEEP`, `lanes.rs` `MIN_ROWS_PER_SHARD`/`set_lane_threads_for_pool`, `examples/{pair_price,sigma_price,water_node_referee}.rs` | live; 8-knot O–O curve 307 s → 71 s, no iteration caps |
| **the re-bank** the solver forced | `conformance/atomworld/REBANK_THICK_RESTART.md`, `SATURATION2_RESULTS.md`'s re-bank section | water table + many-body receipt re-banked, superseded bytes kept marked |
| **the split re-pinned** on the shipped engine (fixed + per-knot, 5 pairs, `examples/pair_price.rs`), `holon_bank_browser_knots` | `holon-render/src/bank.rs` `predicted_load_seconds_at`, `BROWSER_COST_PROVENANCE` | live — H-H in-browser, H-O/O-O ship |
| wasm at opt-level 3 (2.15× on the H-O setup; law probe bit-equal) | `build-web.sh` | live |
| real engine page (replaced the WB-7.1 mock) | `docs/workbench/{index,app,styles,smoke}` + committed `holon_render.wasm` | live, 350 checks (FSD-W3: the ladder from the 1 km cube to the nucleus, 2026-09-03) |
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
* **17b** — `node docs/workbench/smoke.mjs`, 350 checks (149 before FSD-W3), ~40 s. Runs the SHIPPED artifact.

Both verified green at 23fe9e6 plus this increment: 15b clean, 17b 149/149, and gate 15 headless now 27 tests (15 + 5 frame buffer + 7 hand intent). FSD-W3 (1c4ac35, 95378dd): 17b is 350/350; the ladder runs cube → nucleus with the atom and nucleus bands export-gated and live from the shipped doors.

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
4. The retirement battery — enumerated and ruled in `RETIREMENT_BATTERY.md` (R1–R9, R5
   split into an automated stamp read and a manual receipt). R3 runs in full before
   promotion; R7 is a hard blocker; R9 is the lead's. Rows still unbuilt: R5a, R5b, and
   R6's residual sweep for page-local fences with no register row.

The three coarse bands hold no node-G closure certificate, and a fence is a bug under repair,
never content (operator's law). Each names its debt, its owner and the build paying it in
present tense, and a band goes LIVE on that build landing a node-G closure certificate — not
on the fence being well worded. They are not blockers on the site promotion; they ARE the work
queue. Two of the three are now MEASURED rather than fenced — the H-bond network at WB-9 on
LIQUID-1's readouts, the fluid element at WB-10 on FLUID-0's and FLUID-1's, the second one
running its own instrument in the page. That word is a third state and not a softer fence: the
certificate direction is gated on a measured band exactly as on a fenced one, and neither of
the two has earned one.

## Door queue (bands flip only on node-G closure certificates)

Standing law: a band goes live ONLY on a node-G closure certificate whose citation the
gate resolves in both directions. Door (a), the aggregate-defect route, is REFUSED for
every band. LG certificates confer no band state at all.

* **C6 / MOL-PART, door (b) pair** — first in the queue, for the molecular band.
* **rung 2's (N,P) defect door** — pending their A2.
* **LG's defect-against-view curve** — pending `LG_RESULTS.md` / `lg_full.log`, which are
  not banked (third launch, past b=8; two earlier runs killed by their own lane, one for
  reading a gate at the wrong instant against the wrong wall length, one for a log that
  misdescribed its own scene). The citation gate refuses uncommitted artifacts, so this
  panel cannot be drawn before they land.

  **L = 64** is the closure probe's L, confirmed by lattice-tier: the staked points, the
  banked run and the 0.061523 vacuity gap are all at it. Two other L values are in their
  bank and are NOT the curve's — the conservation run (G1–G4, G13) is L = 256, and the
  post-freeze divisor extension is L = 12, 18, 24, 30. Any panel showing one of those says
  which.

  **The caption is a constraint, not a preference.** W(b) = 1 − max(0, b−2)²/b² counts a
  block's boundary layer and is DERIVED assuming the block has neighbours outside it. At
  b = L on a torus there are no inter-block edges, so the layer is empty and the count is
  0 rather than 4b−4. The dashed continuation past b = 32 must therefore be captioned as
  the same algebra applied where its premise no longer holds — NOT as "the curve". Caption
  it as the curve and the page asserts a disagreement between model and measurement, and
  there is none: the discontinuity is the domain boundary. The gap is 1 − (L−2)²/L² —
  0.0615 at L = 64, 0.0311 at 128, 0.0156 at 256 — so a smaller L draws it larger
  honestly; do not draw it at an L the bank does not report.

  Two constraints that travel with any LG wording: the page may not present `field_lg`'s
  chart and the LG tier as one object, and no wording may claim a Navier–Stokes limit for
  the LG tier. One result the page MAY carry: FHP-I has exactly three linear invariants —
  mass and the two momentum components — at every L from 4 to 16, zero spurious, gauged in
  both directions (identity collision returns 6L, HPP-4 returns its textbook 2L+1 per
  line). It is a fact about that configuration only; lattice-tier has NOT read Zanetti's
  statement or its scope, so no sentence may say it contradicts him.

## Standing hazards this lane has hit

* Never land during an open merge, by any mechanism — the completing merge silently drops
  the interposed commit. Park instead.
* `git diff HEAD -- <file>` before any pathspec commit; the index goes stale after a
  private-index landing.
* Plant every check. Five of this lane's own checks passed while establishing nothing
  (empty-string match, existence-not-tracking, indistinguishable failures, a threshold at
  zero, a readout that could not move).

## WB-9 — the ledger by channel, and three bands re-read (2026-09-06)

The page's four WB-9 items, all landed in the working tree (not committed; the lead's).

**The channel doors.** `holon_channel_count / plain / kind / reach / value` sit beside the
existing doors in `holon-render/src/lib.rs`, over `channel.rs`'s `CHANNELS` and
`Sim::channel_standing`. The two name doors return POINTERS to NUL-terminated statics —
strings do not cross the wasm ABI as values, and a code plus a lookup table on the page
would put the six words in two places, which is exactly what the panel exists not to do.
Gated by `tests/channel_door.rs`; a name changed in the record and not in the static table
fails there (planted and confirmed).

**The value door's rule, which is the whole honesty of the panel.** A channel is served a
number only where a ledger row carries it WHOLLY and carries nothing else. A folded row is a
bound and never a value; a row carrying several channels wholly — the seam row carries
exchange, dispersion and charge transfer that way — is not any one of their numbers either.
Today that serves presence (`e_field`) and attunement (`e_far`) and refuses the other four,
and the page prints the refusal rather than a zero.

**THE LIVE 16-WATER BOX: MEASURED, AND IT CANNOT BE DONE — not on price.** Measured on the
shipped artifact, one fresh instance per row, box sized to LIQUID-1's own density
(30.02 Å³ per water, from its 128-water cell):

| waters | atoms | edge (bohr) | half-edge | law reach | `holon_set_boundary(2)` | ms/frame |
|---|---|---|---|---|---|---|
| 16 | 48 | 14.80 | 7.40 | 12.00 | **101 REFUSED** | 0.088 |
| 32 | 96 | 18.65 | 9.32 | 12.00 | **101 REFUSED** | 0.428 |
| 68 | 204 | 23.97 | 11.99 | 12.00 | **101 REFUSED** | 1.638 |
| 108 | 324 | 27.97 | 13.99 | 12.00 | 0 admitted | 4.742 |
| 128 | 384 | 29.60 | 14.80 | 12.00 | 0 admitted | 8.202 |

A wrapping box is legal only while half its shortest edge exceeds the force law's reach, and
a 16-water cell at liquid density is half the edge the law needs. The price is NOT the
obstacle: LIQUID-1's own 128-water cell steps at 8.2 ms per frame on this build. Two
qualifications on those milliseconds — they are the H-H pair law with no seam and no water
tables, so they are a FLOOR on the water price; and they are node's V8 on this box, not a
browser's. Independently of the door, this engine has no export that PLACES atoms, so a page
could not seed sixteen waters even in a cell the door admits.

The band therefore carries the refusal rather than a live box, and shows the door's own two
numbers LIVE (`holon_legality_radius` vs `holon_half_min_edge`) instead of asserting it —
no typed number, and the line moves with the size axis.

**The ladder's third state.** `measured` — the band's physics is read and its node-G
certificate is still owed. It is not a softer fence and not a flip: the certificate
direction is gated on it exactly as on a fence, and FSD-W3 §11.2's GATED for the H-bond band
stays true, because what is gated is the certificate.

**Two gaps in this gate, now closed.** `smoke.mjs` read app.js as TEXT and was perfectly
happy with a file the browser cannot parse — a dropped brace in a data table left all 431
checks green while the page threw on its first line. It now runs `node --check` on app.js
and smoke.mjs, and it RUNS both render passes under a DOM stub, so an exception inside
`renderStatics` fails the gate instead of silently blanking every panel after it.

Smoke: 431 → 522, all green. Five plants confirmed (a syntax error, a refusal drawn as a
zero, the MEASURED word removed, a reading's cite moved one line, a grid row keyed by a name
the engine does not serve).

## WB-10 — the fluid element band runs its own instrument (2026-09-06)

**Verdict.** The FLUID ELEMENT band goes from a fence carrying a finding to **MEASURED**, and
its live cell is FLUID-1's own orientation lattice stepping in the page. The node-G closure
certificate is untouched and still owed; `measured` is not `live`, and the certificate
direction is gated on this band exactly as on a fence. §11.2 still reads FENCED for it and
that stays true — what is gated is the certificate; what changed is that there is now
something to show.

**The instrument in the wasm.** `holon-lattice` is now a dependency of `holon-render`. It was
checked wasm-clean before the edit rather than after: `grep` over the whole of its `src/` for
`std::fs`, `std::thread`, `std::time`, `Instant`, `std::process`, `rayon`, `std::io` and
`std::env` returns NOTHING, its runtime graph is one crate (`ciris-sim-core` with `alloc`),
and `cargo check -p holon-render --target wasm32-unknown-unknown` is clean. **No `cfg` gate
was needed and no rule in the instrument was changed** — the door adds no `pub fn` to
`holon-lattice` and touches no file in it.

The doors live in a NEW file, `engine/crates/holon-render/src/fluid_door.rs`, reached by a
single `pub mod fluid_door;` line in `lib.rs` so the lead has one line to merge. **46 doors**,
refusing the way the seam door does — `0` on success, `FLUID_REFUSED = 220` plus a `k` that
names the reason, and no door panics (`rent_from_retention` asserts outside `(0,1)` and
`Lattice::seeded` asserts on a bad `L`, so both are guarded at the door instead of by editing
the instrument). The rule is PUSHED and never typed: six `holon_fluid_amplitude` calls and one
`holon_fluid_rent_from_retention`, and `holon_fluid_begin` REFUSES until all seven land. There
is no default table in the crate and none in the page.

**Gated natively** by `crates/holon-render/tests/fluid_door.rs`, 5 tests: the door-driven
ledger integer-identical over 500 steps at L = 64 (mass, both momenta, the tracer count, the
per-orientation census entry by entry, and the bond balance `formed − rent − blocked = held`,
with every branch of the bond rule asserted to have fired); the no-bond control bit-identical
to `Lattice::advance_with_colour` over 500 steps in BOTH chiralities, against a referee built
from the crate's own constructors; every refusal firing by its own name with nothing
half-applied; the drawn buffers being the lattice (the image's alpha IS the occupancy, every
listed bond an intact link on the donor's own arm, the six directions embedding at unit
length); and the absences. Two plants confirmed: a control that flips the flag instead of
rebuilding, and a `blocked` count short by one.

**THE BOX, AND THE RULE IT WAS CHOSEN BY.** Measured on the shipped artifact under node
(`taskset -c 24-31`), on FLUID-1's own density and FLUID-1's own seed — the scene the page
actually begins:

| L | cells | particles | one step | image + bonds | bond graph | ONE FRAME |
|---|---|---|---|---|---|---|
| 32 | 1,024 | 1,226 | 0.140 ms | 0.055 ms | 0.049 ms | **0.244 ms** |
| 64 | 4,096 | 4,899 | 0.536 ms | 0.151 ms | 0.056 ms | **0.743 ms** |
| 128 | 16,384 | 19,514 | 2.161 ms | 0.589 ms | 0.203 ms | **2.953 ms** |
| 256 | 65,536 | 78,465 | 9.086 ms | 2.407 ms | 0.790 ms | **12.283 ms** |

**The rule: the largest L whose ENGINE FRAME costs under a quarter of a 60 fps budget
(16.67/4 = 4.17 ms)**, because this lattice is not the only thing on the page — the molecular
scene integrates in the same frame and the rest of the telemetry renders after it. That admits
**L = 128** at 2.953 ms and refuses 256 at 12.283 ms. It is a rule with a cut in it: 256 is
FLUID-1's own box, the door BUILDS it, and what refuses it here is a measured price and not an
inability. The page steps ONE step per frame; a batch would give the lattice a clock that is a
multiple of the frame rate, which is a second clock nobody asked for.

**What the table does NOT contain, said rather than implied.** The three columns are the
ENGINE's work. The canvas work on top — one `putImageData`, one scaled `drawImage`, one
stroked path — is the browser's, and there is no browser on this box to measure it on, so it
is bounded and named rather than guessed. Reviewing that boundary found a real defect and it
is fixed: the bond loop was asking `holon_fluid_dir_euclidean` twice per bond for six
constants, which at L = 128 is over eleven thousand allocating calls into the engine per
frame. The six are now read ONCE into `FLUID.dirs` at load — a constant of the direction set,
not a reading, so a cached copy cannot go stale the way a cached ledger row would — and the
loop is arithmetic on a typed array with no call into the engine at all.

**The band's face** carries FLUID-0's census and FLUID-1's readings, every figure pinned to
the line of the record it came from and checked in both directions by the gate:

| what | value | line |
|---|---|---|
| Schmidt number over 4,608 laws | 0.107 – 0.449 | `conformance/mesh/FLUID0_RESULTS.md:18` |
| water's Schmidt number — THE KILL | 435 | `conformance/mesh/FLUID0_RESULTS.md:18` |
| the branch, staked before any law was read | Branch (c) | `conformance/mesh/FLUID0_RESULTS.md:15` |
| what the census concludes | needs a closure in its carrier | `conformance/mesh/FLUID0_RESULTS.md:76` |
| FLUID-1's Schmidt number at the chart | 0.1973 / 0.2086 | `conformance/mesh/FLUID1_RESULTS.md:79` |
| the cold control and the no-bond floor | 0.3290 / 0.3434 and 0.1083 / 0.1103 | `conformance/mesh/FLUID1_RESULTS.md:79` |
| bonds per particle, ceiling 1 | 0.2894 | `conformance/mesh/FLUID1_RESULTS.md:78` |
| the graph never spans | 0/200 steps | `conformance/mesh/FLUID1_RESULTS.md:78` |
| the W excess grows with the block | 0.045, 0.082, 0.086, 0.109 | `conformance/mesh/FLUID1_RESULTS.md:80` |
| instrument finding — the closure must be able to wait | no rest state on FHP-6 | `conformance/mesh/FLUID1_RESULTS.md:105` |
| instrument finding — the arm sets the ceiling | capped by the arm, not the rent | `conformance/mesh/FLUID1_RESULTS.md:109` |

`measuredBy` moved to `conformance/mesh/FLUID1_RESULTS.md:15` (FLUID-1's own verdict line) and
`liveBoxCite` to `:77` (its price row). Rung 2's two figures stay where they were — 5.95e6 and
+0.598, cited to `RUNG2_RESULTS.md`, which is the molecular route's requirement and is checked
in both directions by the gate that already existed. **The exit is EDGE-0 — the edge exists**,
on FLUID-1's own carrier at its own chart, with its four staked kills named on the face. (No
GANTT line is cited: the section is named in prose and the lead renumbers GANTT cites.)

**What is live, and what is quoted.** Live: the lattice's STRUCTURE and its conserved integers
— bond count, largest component, whether the graph spans, and every audited integer with the
instrument's own per-step audit as its tick. NOT live and never fitted here: the transport.
FLUID-1's `Sc` was read at L = 256 over millions of member-steps with a fitted decay per
wavevector, and a Schmidt number fitted from a few thousand browser frames would be a
measurement nobody made. The band says so on its face and the door's header says so too.

**The rule's numbers are the campaign's.** `docs/workbench/tables/fluid1_amplitude_table.json`
and `tables/fluid1_run.json` are byte-identical copies of `conformance/mesh/fluid1/`'s own
records, digest-pinned in `SHIPPED` and **diffed against the conformance tree by the gate** —
a new check, because a pin only says the served bytes are the ones the page certifies and says
nothing about whether they are the campaign's. The page reads the six amplitudes, the
retention, the density and the seed out of them; it types none of the four. The rent is read
BACK from the engine after the retention goes in, so what is displayed is the engine's
arithmetic on the page's push.

**Two gaps in the gate, now closed.** `getContext` returned `null` in the DOM stub, so every
drawing path on this page was skipped by the gate and tested nowhere but by eye — there is now
a 2D context stub implementing EXACTLY the methods a browser has, so a call to a method
Canvas2D lacks fails here as it would in Chrome. And `fetch` THREW in the stub, so the shipped-
artifact path — fetch, digest, refuse on a mismatch, push through the doors — was unreachable
from the gate; it is now file-backed over the served tree, which is what a bare checkout
serves, so `loadFluid` runs end to end under the gate exactly as it runs in a browser.

**Smoke: 522 → 563**, all green. New checks: all 46 fluid doors present in the artifact's
export table AND the gate's list covering the whole family (both directions, spelled out
rather than prefix-matched — a prefix test passes on an artifact exporting one symbol); the
served records byte-identical to the conformance tree; the live cell stepping once per frame
for 120 frames with every integer identical to its start and the bond balance closing exactly,
with the work it held over asserted branch by branch; the control rebuilding (bonds off, clock
at zero, no bond surviving) through the page's own button; the rule panel drawing every angle
with its CT record; and the picture's own path running with an ImageData exactly L×L. Six
plants confirmed on the page side: a control that does nothing, a lattice that never steps, a
served table drifted one digit from its record, an ImageData of the wrong size, an unscaled
`drawImage`, and (engine side) the two above.

**The artifact grew.** `docs/workbench/holon_render.wasm` 712,848 → 773,856 bytes (+8.6 %),
which is `holon-lattice`'s code; it is NOT stripped by LTO because `holon_fluid_*` reaches it,
and that is the point. Rebuilt exactly as `pages.yml` does
(`HOLON_RENDER_WASM_OUT=… bash engine/crates/holon-render/build-web.sh`), `taskset -c 24-31`.

**Not done, named.** No browser check: the Chrome extension is not connected on this box, so
the picture has been exercised through a strict 2D-context stub and never seen. The band's
fenced-band registration in `FENCES.md` is still owed, as the ladder card already says of all
three coarse bands.
