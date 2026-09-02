# B2 — RESUME

*Session death kills narration, never computation. Every long run below is launched under
`setsid` with a done-marker; if this lane's session is gone, read the markers and the logs
and continue from them.*

## Where things stand

* **Freeze:** `conformance/water_observatory/B2_PREREG.md`, ADMITTED by
  `Audit/prereg_audit.py`, committed at `f2539f1` on branch `b2-ewald` — one commit BEFORE
  `longrange.rs` existed. `git log --oneline` is the ordering proof.
* **Subsystem:** `engine/crates/holon-render/src/longrange.rs`, wired into `Sim`
  (`e_far` row, forces into `a_pair`, virial into `w_virial`, `list_cutoff` respecting
  `R_s`) plus a new angular-momentum ledger (`Sim::angular_momentum`,
  `angular_conserved`, `angular_residual`, `angular_bound`, `angular_gate`).
* **Instrument:** `engine/crates/holon-render/examples/b2_longrange.rs`, three arms.

## The runs

Worktree: `/tmp/claude-1000/b2-ewald-wt`. Build first:

```
cd /tmp/claude-1000/b2-ewald-wt/engine
cargo build --release -p holon-render --example b2_longrange
```

| marker | log | command |
|---|---|---|
| `b2_engine_full.DONE` | `b2_engine_full.log` | `--arm=engine --curves=full --steps=20000` |
| `b2_frames.DONE` | `b2_frames.log` | `--arm=frames --stride=400` |
| `b2_tests.DONE` | `b2_tests.log` | `cargo test -p holon-render --release` |

All three are launched by `conformance/water_observatory/b2_detached.sh`, which writes the
markers into `conformance/water_observatory/`. Markers and logs are RUN STATE and stay
untracked.

**Cost:** the `full` and `frames` arms each regenerate the O–O curve, which B1b measured at
961 s of CPU. Everything else in those arms is seconds. Both are pinned to an E-core
(`taskset -c 24`) because M-PLACEMENT-LOTTERY's remedy is quiet-and-pinned and this box was
at loadavg ~67 while they ran.

## Already banked (H–H arm, `--curves=hh`, pinned, loadavg ~67)

| gate | reading |
|---|---|
| G2 energy half | PASS, relative 0.000e0 |
| G3 | **1 curve FENCED**: H–H `p_fit = 20.6703`, `exp_index = 31.1423`, fit residual 3.3e-1 |
| G5 momentum | PASS, 2.518402e-13 vs bound 2.443462e-9; isolated pair `F_i + F_j` EXACT 0 |
| G6 angular | PASS, 9.982663e-12 vs bound 7.331421e-8 |
| G7 virial | **FIRED** on the `h = 1e-3` prong (1.350e-4); `h = 1e-4` 3.746e-7 PASS, `h = 1e-5` 3.747e-9 PASS |
| G8 gradient | PASS, worst relative 2.1105e-10 over 100 configurations |
| G9 stale cache | PASS, bit-identical at `f = 0.90` and `f = 1.10` |
| G10 image convergence | PASS at 2 shells |
| G11 refusals | PASS, 8 of 8 fired |
| G12 work count | PASS |
| G13 N-scaling | exponent 1.865, monotone, spreads 1.00–1.09 |
| G4 energy | PASS as run, but **VOID under V2**: the staked plant does not fire. Power certificate: G4 resolves a 1.0e-3 Ha step and not the staked 1.0e-6, a factor of 1000 |
| plants | P1, P2, P3, P5, P6, P7 all FIRED. **P3 fired G6 while G5 stayed green** — the staked independence demonstration. P4 did not fire (see G4) |

## What the full arm adds

The O–O curve is the one that matters: its `r_max = 20.0` bohr against `c* = 15.0` is the
mechanism B1b named, and G3's band on it decides whether the tail model may be adopted at
all. The H–H reading above is FENCED, so on present evidence the sector emits a bracket.

## Standing cautions for whoever picks this up

* **Do not retune a fired gate.** G7's `h = 1e-3` prong and G4's V2 are readings. The G4
  power certificate is a measurement OF the gate; the plant's verdict stays at the staked
  1.0e-6 Ha whatever the sweep says.
* **`Sim::close_grain` is where the momentum and angular peaks are sampled**, and only
  `step_frame` calls it. A loop over `step` leaves both at exactly 0.0. This instrument's
  first run did that and the plants REFUSED on 0.0 carriers rather than being scored.
* **The periodic arm's box is sized from `R_f`**, not from the census box. At `p ≈ 21` the
  far window is under a bohr wide and the census box's nearest image is far outside it, so
  every gate in that arm would score a zero against a zero.
