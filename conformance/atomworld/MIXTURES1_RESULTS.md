# MIXTURES-1 — results

Contract: `MIXTURES1_PREREG.md`, frozen 2026-08-30. This file is the campaign's
record. Sections appear in the order they were written, and the P1 protocol
below was written and committed **before the mixed arm was run or its output
looked at**, as the prereg requires.

---

## P1 — THE PROTOCOL, FROZEN

*Committed before any arm ran. Everything below is a `const` or a `fn` in
`engine/crates/holon-render/examples/mixquench.rs`, not a flag, so a reported
run re-runs byte for byte and a run whose parameters were overridden cannot be
reported as one whose parameters were staked.*

### The three arms

| arm | scene | pair types banked |
|---|---|---|
| **mixed** | 8 H + 8 Cl | H-H, H-Cl, Cl-Cl |
| **control: hydrogen** | 16 H | H-H |
| **control: chlorine** | 16 Cl | Cl-Cl |

### The scene

| | |
|---|---|
| atoms | 16 (`MAX_ATOMS`) |
| dimensions | 2 — the `z = depth/2` slice |
| box | 40 × 24 bohr, soft quadratic walls, wall inset 0.6 bohr — SATURATION-1's box kept, so the hydrogen control is comparable to its bank |
| opening positions | a 4 × 4 lattice at `(w(col+½)/4, h(row+½)/4)` with a per-seed uniform jitter of ±0.8 bohr |
| **mixed composition** | **checkerboard: lattice cell with `(col + row)` odd is chlorine.** Eight of each, every chlorine with four hydrogen nearest neighbours and vice versa. Stated as a rule rather than a list so it cannot be quietly re-drawn; an opening that clustered the chlorines on one side of the box would be an opening that decided the answer |
| opening velocities | Box–Muller Gaussians from one seeded LCG stream at `T_init = 3000 K`, with the width taken **per species** (`sigma = sqrt(k_B T / m)`, so a chlorine opens 5.9× slower than a hydrogen at the same temperature) and the net **momentum** removed — not the mean velocity, which in a mixed box is a different quantity |
| thermostat | ON from the first step, Berendsen, `T_target = 300 K`, `tau = 2000` a.u. |
| integration | 20,000 grain boundaries × 64 substeps = 1,280,000 substeps |
| curves | 96 knots, engine-computed STO-3G FCI, generated once per process |
| three-body | ON, and **H3-ONLY** — see the fence below |
| RNG | one LCG (`x = 6364136223846793005 x + 1442695040888963407`, top 53 bits) seeded per run; nothing else is random |

### The eight staked seeds

```
0x000000004d495801  0x000000004d495802  0x000000004d495803  0x000000004d495804
0x000000004d495805  0x000000004d495806  0x000000004d495807  0x000000004d495808
```

### THE FENCE, displayed

The three-body term is **H3-only**: `Sim::accumulate_three_body` skips any triple
containing a non-hydrogen atom, so such a triple contributes an exact zero. The
mixed arm therefore runs MBE2-exact over all three pair types plus MBE3 over the
hydrogen triples only. **No reading in this campaign is beyond-pair-complete for
a triple containing chlorine.** The engine declares the fence (`holon_trimer_h_only`)
and both viewers display it, rather than each asserting it in a caption.

### Two protocol facts, disclosed rather than discovered later

**The arms cover different amounts of simulated time at the same boundary count**,
because `dt` is derived per scene from the fastest active mode. The chlorine arm's
`dt` opens about 18× the hydrogen arm's — Cl2 is stiffer than H2 but chlorine is
35× heavier, and frequency is what a timestep has to resolve — so 20,000 boundaries
is roughly 300 ps of chlorine and 17 ps of hydrogen. Equal boundary counts were
chosen over equal simulated time because equal simulated time would mean unequal
integration accuracy, and the accuracy contract is the thing this engine holds. Both
numbers are reported per arm.

**`dt` falls during a run and that is the design, not drift.** The curvature envelope
is monotone and widens as the trajectory reaches stiffer parts of the curve, so the
hydrogen arm opens at `dt = 1.0797` and refines to `0.5387` within a hundred
boundaries. The drift bound is re-derived from the current `dt` on every call, so
there is no stale bound behind it.

### Knot density, measured before it was frozen

`R_e`, `D_e` and `k_e` do not depend on knot count at all — they come from
`locate_well`'s own Newton solve on the solver, not from the interpolant. The
derived timestep does, weakly, because it reads the interpolant's curvature
envelope:

| knots | `dt` on the opened hydrogen scene |
|---|---|
| 24 | 1.079664 |
| 48 | 1.077481 |
| 96 | 1.077209 |
| 192 | 1.076929 |
| 384 | 1.076997 |

0.25% across a 16× range, converged well before 96. `CURVE_KNOTS = 96` is chosen
for the interpolant's accuracy *between* knots and for cost, not for the clock.
Cl2 is what prices it: 18 basis functions, 324 determinants, about 97 s at 48 knots.

### Measured cost, from which the schedule is frozen

| arm | curves | per boundary | per seed at 20,000 |
|---|---|---|---|
| hydrogen | 0.3 s | 0.0017 s | 33 s |
| chlorine | 112 s | 0.0007 s | 14 s |
| mixed | 121 s | 0.0008 s | 15 s |

### THE MEASUREMENT RULE

Taken at the final grain boundary, from `Sim::cluster_species_counts` and
`Sim::cluster_sizes` — **two readings of one union-find over one edge set**, the
same partition the headline `Sim::cluster_count` already reads. No new criterion
and no distance cutoff: an edge exists exactly where the pair layer says `bonded`.

* a component of ONE atom is a **free atom**, not a molecule;
* a component of two or more has a **formula**, the count of each nuclear charge
  in it, keyed by `Z` rather than by the bank's species index — the index depends
  on registration order, and a formula built from it would depend on which atom
  happened to be placed first;
* the **modal molecule** is the most common formula among components of size ≥ 2.
  Ties break toward the SMALLER component, then toward the LOWER maximum `Z`, then
  lexicographically. That is a total order, so the answer cannot depend on
  iteration order;
* the full formula histogram is published per seed and pooled, either way.

### The criteria, and what each decides

* **BRANCH (a)** — the mixed arm ends with **HCl as the modal molecule in ≥ 6 of
  8 seeds**.
* **BRANCH (b)** — anything else. Reported as plainly as a pass would be, and
  investigated. Not massaged.
* **CONTROLS.** Each single-species arm must (1) produce at least one molecule in
  ≥ 6 of 8 seeds — the instrument sees molecules at all — and (2) produce formulae
  containing ONLY its own element, which is a structural check that would catch a
  species-bookkeeping bug in the bank. **If either control fails, P1 is VOID** —
  protocol, not physics, per the detector-not-verdict rule — and the mixed arm's
  reading is not reported as a result.
* **C1 rides along.** Every seed of every arm reports its energy drift against the
  derived bound and its momentum residual against the roundoff bound. A gate firing
  there is reported; it does not silently invalidate the composition reading, because
  the two measure different things.

### What a pass would and would not mean

A pass means: *in this model, from Z, masses and the STO-3G basis alone, a hot gas
of hydrogen and chlorine cooled in a box ends up as hydrogen chloride.* Nothing
here is a claim about nature's thermochemistry, about rates, or about a triple
containing chlorine.

---

*Results sections follow below as each arm lands.*
