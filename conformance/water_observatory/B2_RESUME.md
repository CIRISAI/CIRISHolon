# B2 — RESUME

*Session death kills narration, never computation. Every long run below is launched under
`setsid` with a done-marker; if this lane's session is gone, read the markers and the logs
and continue from them.*

## The commits, in order — the ordering proof is `git log --oneline` on `b2-ewald`

```
7c8866b The far sector was ADDING to the near one, and a gradient gate found it
b8a73a1 Three of my own gates were passing on nothing, and one of them found an engine fact
4ce3a9a A checkpoint would have dropped the far sector silently, and the suite says what it costs
07c1c4f B2's subsystem: a split kernel, an angular ledger, and the plants that refused
f2539f1 B2's freeze: the measurement fired a long-range method, and it is not Ewald
55c61b9 (branch point)
```

**The freeze landed before the subsystem existed.** `Audit/prereg_audit.py` returns
`ADMITTED B2_PREREG.md`.

## What exists

* **Freeze:** `conformance/water_observatory/B2_PREREG.md` — 14 gates, 5 refusals, 7 plants,
  6 VOID conditions, and the method argument (§2: this force law has no `r⁻¹` term, so
  Ewald's defining difficulty is absent and B2 does not build it).
* **Subsystem:** `engine/crates/holon-render/src/longrange.rs`. Wired into `Sim` as `e_far`
  with forces into `a_pair` and virial into `w_virial`; `list_cutoff` reaches `R_s`; the far
  sector's declaration is in `physics_digest` so a checkpoint cannot drop it silently.
* **Angular-momentum ledger, new to this engine:** `Sim::angular_momentum`,
  `angular_conserved`, `angular_residual`, `angular_bound`, `angular_gate` (returns
  `Option`, so "not applicable" cannot be read as a pass).
* **Instrument:** `engine/crates/holon-render/examples/b2_longrange.rs` — arms `engine`,
  `frames`, `refusals`.
* **Suite tests:** `engine/crates/holon-render/tests/b2_longrange.rs`, 21 tests, including a
  mutation check that every invariant is re-run under the plant that should break it.

## The runs

```
cd /tmp/claude-1000/b2-ewald-wt/engine
cargo build --release -p holon-render --example b2_longrange
```

| marker | log | command | cost |
|---|---|---|---|
| `b2_engine_full.DONE` | `b2_engine_full.log` | `--arm=engine --curves=full --steps=20000` | one O–O solve (~680 s CPU) |
| `b2_frames.DONE` | `b2_frames.log` | `--arm=frames --stride=400` | one O–O solve |
| `b2_tests.DONE` | `b2_tests.log` | `cargo test --release -p holon-render` | ~15 min |
| — | — | `--arm=refusals` | seconds |

Pinned to E-cores (24 and 26) because M-PLACEMENT-LOTTERY's remedy is quiet-and-pinned and
this box sat at loadavg 65–80 throughout. Timings are CPU seconds, not wall.

## Banked — see `B2_RESULTS.md` for the full record

**The frames arm is complete and reproduces B1b bit for bit.** All eight
`max|E_switch(c*)|` values and their frame indices match `B1B_RESULTS.md` §2 exactly,
including the worst: seed `0x0000000053415424`, frame 10144, **1.150526e-5** Ha. 160,000
frames scored, 8 of 8 admitted by digest, 0 refusals.

* **G1 — S-DOMINANT at S/T = 9.9e8.** B1b's headline discard is a RADIUS-BOOKKEEPING defect:
  channel S (real tabulated interaction in `(15, 20]` bohr, lost because the list radius came
  from a THREE-BODY table) is nine orders larger than channel T at the deciding frame.
* **G14 — PASS, 0 of 8 seeds over.** The three seeds B1b failed (1.898, 1.574, 2.496) come in
  at 0.0274, 0.0178, 0.0290. `beyond R_f` is exactly 0 on every seed because `R_f = 73.27`
  exceeds the box diagonal, so the residual is a bracket with ends 0 and `|model gap|`.
* **G2 — exact half PASS**, 0 missed pairs over 108,599.
* **G3 — O–O ADOPTING at `p_fit = 5.0049`**; H–H and O–H FENCED and nothing rests on them.
* **G11 — 10 of 10 refusals fire**, including the negative control.
* **Suite — 21 binaries, exit 0**, `t3_replay` included.

## The engine arm, after the fixes at `4d25135`

* **G9 stale-cache: PASS bit-identical in BOTH directions**, with **P1 still firing**
  (carrier 7.332040e-7 Ha) — the gate kept its power through the fix.
* **G8: still FIRED.** 1.0000e0 unfiltered (saturated by a numeric reference that underflows
  to exactly zero); **1.1870e-7 over the 187 components whose disagreement the reference can
  resolve, which is still 1.19× the staked 1e-7.** The saturation is an artifact; the 19%
  overshoot is not.
* **G7: still FIRED** on its coarsest staked step (3.8381e-5) with the finest at 3.8386e-9.
* **G13: exponent 2.123**, monotone. **G10: PASS** at 3 shells. **G4/G5/G6: PASS** complete
  and truncated. **G4's arm VOID under V2.**
* **Suite: 21 binaries, exit 0** against the final tree.

## The fired gates, kept fired

* **G7 (virial)** fires on its `h = 1e-3` prong, 1.01e-4 against 1e-6. The other two staked
  steps pass (3.75e-7 and 3.75e-9) and the Richardson extrapolation equals the virial to ten
  digits. The fired prong is the coarsest step's own `O(h²)` truncation error. **Not
  retuned.**
* **G8 (gradient)** fires at 4.3e-6 against 1e-7. The far term is now a DIFFERENCE of two
  nearly equal functions, so a per-component relative error is dominated by components
  carrying no force: the worst relative component holds `|F| = 8.2e-15` Ha/bohr against a
  largest far force of `8.1e-9`, and the largest absolute disagreement anywhere is 4.7e-10 of
  that largest force. **Not retuned.**
* **G4 (energy) is VOID under V2**: the staked 1.0e-6 Ha plant does not fire it. Power
  certificate: G4 resolves a zero-point step at 1.0e-2 Ha on this scene, a factor of 10⁴
  above the staked plant. The sweep is a measurement OF the gate and never a new criterion.
* **The periodic arm is VOID on H–H by construction**: at `p_fit = 20.67` the far sum reaches
  `R_f = 11.19` bohr while the smallest legal wrapping box is `2 R_s = 20.48`, so no legal
  periodic box can put an image in range. A tail that steep has no long-range content for an
  image sum to carry. It should score on the full curve set, where `R_f = 73.27` against
  `2 R_s = 40`.

## Corrections this lane made to its own instrument, all found by its own gates

1. **The far sector was ADDING to the near one** (fixed at `7c8866b`) — every pair past `R_s`
   counted twice, plus a step of `u(R_s)` at the handover. Found by widening G8 from one
   configuration probed 100 times to 100 distinct configurations, which is what the freeze
   stakes.
2. **P2 and P3 REFUSED on 0.0 carriers.** `Sim::close_grain` samples the momentum and angular
   peaks and only `step_frame` calls it; a loop over `step` leaves both at exactly 0.0.
3. **G2 compared the complete route against itself.** `CellList::rebuild` needs 64+ atoms AND
   3 cells per axis.
4. **The periodic arm scored 0.0 against 0.0** because the census box's nearest image sits
   outside `R_f`.
5. **G13 read three zeros** against a 10 ms clock tick; V6 correctly convicted it.
6. **The crossing counter** read every far pair as a fresh crossing on the first pass.
7. **The manifest key carried an extra path segment** (`census-traj/fenced/…` where the
   manifest says `fenced/…`), refusing all eight trajectories; and the digest was not being
   computed at all, only path presence. Both fixed; the sha256 is lifted verbatim from
   `longrange_audit.rs` with its standard-vector self-test.
8. **`build_scene` regenerated every pair table per call**, re-solving the 681-second O–O
   determinant space once per scene.

## An engine fact found on the way, outside B2's gates

**In an open or walled box the cell decomposition's extent is the ATOMS' bounding box, not
the nominal box** (`cells.rs:448`). A `Dims::Two` scene sits on one plane, so its z extent is
zero, `nc[2]` is 1, and `CellList::rebuild` falls to the COMPLETE route however many atoms
the scene holds. Every census scene is `Dims::Two` with walls. **The O(N) route B1b's
counterfactual is about does not engage on them at all.** Worth a look from whoever owns the
T3 cost story; it is not a B2 verdict and nothing here rests on it.

## Standing cautions

* **Do not retune a fired gate.** G7's coarse prong, G8, and G4's V2 are readings.
* Run-state markers (`.log`, `.DONE`) stay untracked.
* NEVER push. `cirisontology-b4` integrates.
