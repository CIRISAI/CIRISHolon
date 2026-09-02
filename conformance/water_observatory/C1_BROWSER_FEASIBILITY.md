# C1_BROWSER_FEASIBILITY — can the ring-polymer carrier be a browser exhibit?

**Status: MEASURED. VERDICT — REFUSED, on a gate this lane mis-staked; the probe is
removed.** The criterion below was committed at `6a45f59` BEFORE the probe was built or
timed (a correction to four transcribed configuration knobs followed at `5a43e2f`, still
before any measurement); the measurement and the verdict follow it, below the horizontal
rule. That ordering is the point of the file and `git log` is the receipt for it.

**Subject:** `engine/crates/holon-chem/src/rpmd.rs`, the C1 carrier certified natively in
`conformance/water_observatory/C1_GATE_RESULTS.md`, driven from
`engine/crates/holon-render`'s wasm artifact under node.

**Question the FSD asks (§9b): C1 is "not workbench-facing this pass."** Is that a price
or a preference? A measurement decides; this lane does not.

---

## THE STAKED CRITERION (written 2026-09-01, before any timing)

The exhibit ships **iff both** of the following hold:

1. **SPEED.** A `P = 16` call to the wasm probe completes in **≤ 5.0 wall-seconds**
   under node on this box, taken as the **median of three repetitions** on the
   configuration the exhibit would actually ship (the configuration is frozen below
   before it is timed).
2. **CORRECTNESS.** The `P = 1` call reproduces the classical limit `rpmd.rs`
   documents. `harmonic_ring_energy`'s docstring states the tell: *any expression for
   `E_P` that does not give exactly `kT` at `P = 1` is not the energy of this ensemble*,
   and `C1_GATE_RESULTS.md` §4 measures it on the real curve as
   `E_1 = 0.000953421 ± 1.98e-6` against `kT = 1/beta = 0.000950043`, i.e. `kT` plus a
   `+0.36%` classical anharmonic correction. **The check: `E_1 − V_min` must sit within
   `[kT, 1.02 kT]`** — above `kT` (the well is anharmonic and softens outward, so the
   classical `<V>` exceeds `kT/2`), and not more than 2% above it (the native reading is
   +0.36%; a single short chain has more noise than the native 8×400 000, and 2% is
   ~5× the native offset — wide enough for one short chain, narrow enough that a broken
   estimator, a wrong `beta`, or a collapsed ring cannot pass).

**The ladder criterion is a REPORT, not a gate.** The P-ladder must move MONOTONICALLY
UP from the classical value toward the banked anharmonic ZPE
`ZPE_DVR(H2) = 0.011288114850 Ha` (C1_GATE_RESULTS.md §3), and each rung is reported
against the native ladder row for the same `P` (§4, "the bead ladder"). **No convergence
is claimed beyond `P = 32`**: the native ladder puts `P = 32` at `0.010533410`, which is
**6.7% BELOW** the reference, so an exhibit capped at 32 beads is showing the APPROACH,
not the answer, and must say so. A rung landing far outside its native row's
neighbourhood is a defect in the probe, not a discovery.

**If the criterion fails, the probe is REMOVED** and this file carries the numbers, the
criterion, and the refusal, so the FSD's next revision cites a measurement.

### The configuration, frozen before timing

The probe takes ONE argument (`beads`); everything else is a constant compiled into it,
and these are the constants:

| knob | value | why |
|---|---|---|
| curve | `rpmd::BankedPes::h2(n_knots)` | the engine's own generated H–H curve, computed in memory at call time. NOTHING is read from disk — see the dependency note below |
| knots | 1024 | the native campaign used 4096; G2 measured the interpolant's departure from the model at 3.41e-14 Ha there, so the grid is not the error budget. Knots cost `h2_point` calls, which are the expensive ones |
| temperature | 300 K | the native campaign's |
| `dt` | 4.0 a.u. | the native campaign's staked step (G1 headline); its `dt/2` control moved the answer by 0.05% |
| `gamma_centroid` | `omega_harm` = `sqrt(h2_point(R_e).e2 / mu_H2)` | the native campaign's, **corrected — see below** |
| chains | **1** | `run_pimd_chains` uses `std::thread::scope`; wasm has no threads. The exhibit runs ONE chain via `run_pimd` |
| steps | `steps_sample` = 60 000, `steps_equil` = `steps_sample / 10` = 6 000 | the sampled count is a browser budget chosen before timing; the 1:10 equilibration ratio is the native campaign's, **corrected — see below** |
| seed | `0xC1_0001` | the native ladder's, **corrected — see below**; deterministic, because the exhibit is reproducible or it is not an exhibit |
| `r_start` | `equilibrium().0`, the MODEL's `R_e` | the native campaign's, **corrected — see below** |
| returned | `e_virial − V_min`, hartree | the centroid-virial estimator, which `C1_GATE_RESULTS.md` names primary |

#### Correction to this freeze, landed BEFORE any timing

The first version of this table stated four knobs as "the native campaign's" that were
not. Read against `engine/crates/holon-chem/examples/c1_campaign.rs::stage_ladder`, the
campaign uses `gamma_centroid: s.omega` (the curve's own harmonic frequency, ≈ 2.2e-2
a.u.) and not `0.001`; `steps_equil: steps / 10` and not a fixed 20 000; `seed:
0xC1_0001` and not `0xC1`; and `r_start: s.r_e`, the model's equilibrium separation from
`h2::equilibrium`, and not the interpolant's golden-section minimum. All four are
corrected above and the corrections are recorded rather than quietly applied.

`gamma_centroid` is the one that could have moved a number: the centroid friction is the
one thermostat parameter the module calls a free DECLARED choice, and running the exhibit
at a friction 22× below the campaign's would have lengthened the centroid correlation
time and widened the error bar on a single short chain — a systematic difference from the
certified run, dressed as the same configuration. The other three are reproducibility,
not physics.

**Two knobs remain DELIBERATE departures from the campaign, declared as such:**
`n_knots` = 1024 against the campaign's 4096, and one chain against eight. The knot count
is a browser cost decision: the cubic Hermite error scales as `h^4`, so quartering the
knot count multiplies G2's measured 3.41e-14 Ha interpolation departure by ~256, to
~9e-12 Ha — six orders below the statistical noise of a single short chain, so the grid
is still not this exhibit's error budget. The chain count is forced by wasm, not chosen.

`V_min` is `BankedPes::minimum().1` — the INTERPOLANT's own minimum, located by golden
section on the interpolant, which is what the native campaign refers its ZPE to.

**Cap: `P ≤ 32`.** Above that the exhibit would be claiming a convergence it has not
measured, and the `O(P^2)` normal-mode transform is the sampler's cost at large `P`
(C1_GATE_RESULTS.md §7).

**Refusal: H–H only.** One molecule, one curve. Any other species is refused BY NAME.

---

# THE MEASUREMENT, and the verdict it forces

## VERDICT: THE EXHIBIT DOES NOT SHIP. The probe has been removed.

**The staked criterion has two clauses and they split.**

* **SPEED: PASSES, by 2.26×.** A `P = 16` call completes in wasm under node in a median
  **2.216 s** against the staked 5.0 s, and the SLOWEST of the three repetitions is
  2.398 s — so the clause passes on every reading, not just on the median. `P = 32`, the
  cap, runs in a median 3.940 s and is also inside.
* **CORRECTNESS: FIRES.** `E_1 − V_min` = **0.000936213563 Ha** against
  `kT = 0.000950043469 Ha`, a ratio of **0.98544**, where the staked band is
  `[1.000, 1.020]`. It lands **below** the band.

The criterion says *ships iff both*. One fired, so it does not ship, and per the freeze's
own instruction the probe is removed rather than argued with.

**The fired clause is MY error, not the instrument's, and the distinction is measured
rather than asserted — see §3.** The browser's ring polymer reproduces the natively
certified ladder at every one of the six bead counts, worst departure **1.26σ**. What
failed is the band I staked around the classical limit: I sized it without first
measuring the spread of the single short chain the exhibit would actually run, and the
band turns out to be narrower than the noise it was meant to grade. That is the same
shape as the C1 campaign's own G6 — a discriminator staked from an unmeasured proxy —
and it is reported fired for the same reason G6 was, not retro-fitted. §4 computes what
a correctly-sized successor stake costs, so the next attempt can be staked on a number.

---

## 1. THE TIMING TABLE (the staked measurement)

Load average **59–62** on a 32-core i9-13900HX throughout, other lanes live; both columns
pinned with `taskset -c 0` (a P-core). **No wall time here was taken on a quiet box, and
that makes the speed pass ADVERSARIAL rather than flattering** — the criterion is met at
the load the machine actually carries.

Wasm is the shipped-profile artifact from `build-web.sh` (`opt-level=z`, LTO, `panic=abort`,
stripped), loaded by a node driver whose loader is `smoke.mjs`'s `freshEngine()`: read the
bytes, `WebAssembly.instantiate(bytes, {})`, call the raw `extern "C"` export. A fresh
instance per call, so no run inherits state from the one before it. Native is the SAME
function — `holon_render::holon_c1_zpe_probe` — called from an example binary, so the
ratio is one function against itself and not two implementations that resemble each other.

### wasm, under node (`taskset -c 0`), three repetitions each

```
# wasm probe: holon_render.wasm (386,600 bytes, shipped profile)
# export present: true
#      P  rep          seconds                  value (Ha)
       1    1         0.960365              0.000936213563
       1    2         0.682574              0.000936213563
       1    3         0.743961              0.000936213563
       2    1         0.721869              0.001862296473
       2    2         0.672347              0.001862296473
       2    3         1.022056              0.001862296473
       4    1         0.686550              0.003621794762
       4    2         0.586394              0.003621794762
       4    3         0.627019              0.003621794762
       8    1         1.551328              0.006353455266
       8    2         1.737665              0.006353455266
       8    3         1.911244              0.006353455266
      16    1         2.397970              0.009051611770
      16    2         2.215628              0.009051611770
      16    3         1.275237              0.009051611770
      32    1         3.905841              0.010580385854
      32    2         4.152366              0.010580385854
      32    3         3.939931              0.010580385854
refusal P=  0  -> NaN
refusal P= 33  -> NaN
refusal P= 64  -> NaN
```

### native, same function, same ladder (`taskset -c 0`), three repetitions each

```
# native probe: holon_render::holon_c1_zpe_probe
#      P  rep          seconds                  value (Ha)
       1    1         0.510902              0.000936213563
       1    2         0.534856              0.000936213563
       1    3         0.499493              0.000936213563
       2    1         0.393712              0.001862296473
       2    2         0.323668              0.001862296473
       2    3         0.333441              0.001862296473
       4    1         0.365086              0.003621794762
       4    2         0.341437              0.003621794762
       4    3         0.307274              0.003621794762
       8    1         0.288355              0.006353455266
       8    2         0.321330              0.006353455266
       8    3         0.329712              0.006353455266
      16    1         0.533586              0.009051611770
      16    2         0.811537              0.009051611770
      16    3         0.934909              0.009051611770
      32    1         1.639706              0.010580385854
      32    2         1.202234              0.010580385854
      32    3         0.718950              0.010580385854
refusal P=  0  -> NaN
refusal P= 33  -> NaN
refusal P= 64  -> NaN
```

### medians, and the ratio

| P | native med (s) | native worst (s) | wasm med (s) | wasm worst (s) | wasm/native |
|---|---|---|---|---|---|
| 1 | 0.511 | 0.535 | 0.744 | 0.960 | 1.46 |
| 2 | 0.333 | 0.394 | 0.722 | 1.022 | 2.16 |
| 4 | 0.341 | 0.365 | 0.627 | 0.687 | 1.84 |
| 8 | 0.321 | 0.330 | 1.738 | 1.911 | 5.41 |
| **16** | **0.812** | 0.935 | **2.216** | 2.398 | **2.73** |
| 32 | 1.202 | 1.640 | 3.940 | 4.152 | 3.28 |

**The ratio column is not a measurement of wasm and must not be quoted as one.** It ranges
1.46–5.41 over six rows of the same code, which no compilation-target effect produces; it
is the shared box at load 60, and the `P = 8` row's 5.41 is the tell. The two largest bead
counts — where real compute dominates the fixed setup and the sample is least contaminated
— give **2.7–3.3×**, and that is the only reading here that should be carried forward.

### the cost that is NOT the sampling

```
# setup: BankedPes::h2(1024) 0.3888 s   (of which banked_range() 0.2433 s)   minimum() 0.0000 s
```

**A fixed 0.389 s native is spent before a single bead moves, and 0.243 s of that is
`banked_range()` alone** — `pair::derive_range`'s bisection walk, which calls the general
`solve_geometry` dual-number solver ~40 times to rediscover where the H–H table should
start and stop. The 1024 `h2_point` knots are only the other ~0.145 s. At `P = 1` that
fixed cost is **70%** of the call and at `P = 16` it is still **~48%**.

This is an ARCHITECTURE finding and it outlives the verdict: the probe regenerates the
whole curve on every call, where `holon_table_generate` fills the `Sim`'s bank once and
keeps it. Any successor door should hold the `BankedPes` in the static the rest of the ABI
already uses, and would get a third of `P = 16`'s wall back for free.

### what it would have cost the artifact

| | bytes |
|---|---|
| baseline, this source, no probe | 375,531 |
| with the probe | 386,600 |
| **delta** | **+11,069 (+2.95%)** |

Both built by `build-web.sh` at the shipped profile, to a scratch path.
`docs/workbench/holon_render.wasm` was never written.

---

## 2. THE CLASSICAL-LIMIT CHECK (the fired clause), both numbers

| | Ha |
|---|---|
| `E_1 − V_min`, measured in wasm and natively | **0.000936213563** |
| `kT = K_B × 300 K`, from `rpmd::K_B_HARTREE_PER_KELVIN` | **0.000950043469** |
| ratio | **0.985443** |
| staked band | `[1.000000, 1.020000]` |
| verdict | **OUTSIDE, below. FIRED.** |

For comparison, `C1_GATE_RESULTS.md` §4's native reading of the same quantity is
`E_1 = 0.000953421 ± 1.98e-6`, i.e. `1.00356 kT` — the `+0.36%` classical anharmonic
correction to `<V>` in a well that is not harmonic.

**Why it fired, measured.** The probe is deterministic in one seed, so repeating it
measures wall time and says nothing about spread. Running the frozen configuration across
twelve seeds instead:

```
# P=1 steps_sample=60000 knots=1024 gamma=omega=0.022790089  kT=0.000950043469
#     seed              value            err        tau        /kT
       0      0.000936213563  1.192e-5    10.10   0.985443     <- the seed the probe ships
       1      0.000970748178  1.363e-5    11.21   1.021793
       2      0.000936675991  1.423e-5    13.76   0.985930
       3      0.000948570586  1.167e-5     9.55   0.998450
       4      0.000938852112  1.235e-5    10.95   0.988220
       5      0.000968950134  1.744e-5    17.83   1.019901
       6      0.000938991573  1.413e-5    13.88   0.988367
       7      0.000954974131  1.365e-5    12.34   1.005190
       8      0.000958000771  1.293e-5    11.27   1.008376
       9      0.000945591429  1.412e-5    13.32   0.995314
      10      0.000944907466  1.473e-5    15.58   0.994594
      11      0.000954076656  1.371e-5    12.74   1.004245
mean 0.000949712716   sd(single chain) 1.1888e-5   sem 3.4316e-6   mean/kT 0.999652
```

* The single-chain standard deviation is **1.19e-5 Ha = 1.25% of `kT`**, against a band
  **2% wide**. The gate is a coin flip on the seed and always was.
* The shipped seed's draw sits **1.14 sd below its own mean** — an unremarkable draw, not
  an outlier.
* The twelve-seed mean, `0.99965 kT`, is **0.94σ** from the campaign's `1.00356 kT`. The
  physics is right; the precision is not what I staked against.

**The band's defect is its LOWER EDGE, not its width, and more sampling does not fix it.**
The truth sits only `+0.36%` above `kT`, so a band whose lower edge IS `kT` needs
`sd ≪ 0.12%` of `kT` to be a reliable one-sided check — 108× the sampling. Even a 10×
longer run (`sd → 0.40%`) leaves the lower edge 0.9σ from the truth and fails roughly one
seed in five. **A criterion that a correct instrument fails one time in five is not a
criterion**, and no amount of the compute headroom §1 measured would have rescued it.

---

## 3. THE INSTRUMENT IS NOT WHAT FAILED

Two independent receipts, and neither is a symbol lookup — per the census lane's law,
absence proves nothing and these are RETURNED VALUES.

### 3a. The browser computes what the native build computes

The wasm and native columns of §1 agree to all twelve printed digits at every `P`. They
are **not** bit-identical, and the difference is the one `lib.rs` already names — the two
builds link different `libm`s:

```
  P   native_bits        wasm_bits         ulps   abs_diff(Ha)   rel_diff
  1   0x3f4ead8751745000 0x3f4ead8751745000      0   0.000e+00     0.000e+00
  2   0x3f5e83099cb7d800 0x3f5e83099cb7d800      0   0.000e+00     0.000e+00
  4   0x3f6dab7441baf800 0x3f6dab7441bafc00   1024   4.441e-16     1.226e-13
  8   0x3f7a0614a9540a00 0x3f7a0614a9540900    256   2.220e-16     3.495e-14
 16   0x3f8289a6c4375b00 0x3f8289a6c4375a00    256   4.441e-16     4.906e-14
 32   0x3f85ab2b59c42d00 0x3f85ab2b59c42d00      0   0.000e+00     0.000e+00
```

Worst disagreement **4.44e-16 Ha**, which is **ten orders of magnitude** below the
1.19e-5 Ha of statistical noise the same run carries. The browser and the native build
are computing the same physics; the last two digits are `exp`, `sin` and `sqrt` disagreeing
in their final bits.

### 3b. The browser's ladder IS the certified ladder

Every rung against `C1_GATE_RESULTS.md` §4's row for the same `P` (8 chains × 400 000
sampled steps). The comparison uses the spread measured AT EACH `P` — using the `P = 1`
spread everywhere would have been a one-directional check, and the spread is not flat in
`P`:

| P | shipped seed | 8–12 seed mean | sd (1 chain) | campaign, 8×400k | (mean − campaign)/σ | seed ÷ `ZPE_DVR` |
|---|---|---|---|---|---|---|
| 1 | 0.000936213563 | 0.000949712716 | 1.19e-5 | 0.000953421 | **−0.94** | 0.08294 |
| 2 | 0.001862296473 | 0.001886267691 | 3.26e-5 | 0.001882803 | **+0.27** | 0.16498 |
| 4 | 0.003621794762 | 0.003640918637 | 5.46e-5 | 0.003626162 | **+0.70** | 0.32085 |
| 8 | 0.006353455266 | 0.006340460057 | 5.02e-5 | 0.006326055 | **+0.74** | 0.56284 |
| 16 | 0.009051611770 | 0.009015188869 | 5.54e-5 | 0.009018816 | **−0.17** | 0.80187 |
| 32 | 0.010580385854 | 0.010553297817 | 3.67e-5 | 0.010533410 | **+1.26** | 0.93730 |

**Worst departure 1.26σ over six bead counts.** The ladder is monotone up, as the report
criterion required, and it climbs toward `ZPE_DVR(H₂) = 0.011288114850 Ha`.

**And it does not get there, exactly as the freeze said it would not.** At the `P = 32`
cap the browser reads **93.73%** of the reference — **6.27% short** — against the
campaign's own `P = 32` shortfall of 6.69%. A capped exhibit shows the APPROACH and never
the answer, and the number by which it falls short is now measured on the browser's own
artifact rather than inherited.

---

## 4. DEPENDENCIES: what the browser can and cannot have

The step-1 question was whether the ZPE entry point needs the DVR reference or banked
curve FILES a browser cannot have. **It does not, and that is the load-bearing finding
under everything above.**

* `BankedPes::h2(n)` → `banked_range()` → `pair::derive_range` → `solve_geometry`, then
  `table::generate_table` → `h2::h2_point`. **All closed-form arithmetic, computed in
  memory at call time. Nothing is read from disk, and no banked artifact is required.**
  This is the same route `holon_table_generate` already ships in the browser.
* `run_pimd` is `std`-and-`Vec` only. It runs in wasm unmodified.
* **`run_pimd_chains` does NOT.** It is `std::thread::scope`, and wasm has no threads. The
  campaign's eight-chain average is unavailable in the browser BY CONSTRUCTION, and that is
  the whole origin of §2's problem: the browser gets one chain, so it gets `√8 = 2.8×` the
  error bar at equal steps per chain before any budget question is asked. A worker-pool
  route exists in principle (`holon-md` is the workspace's precedent for exactly this) and
  is not free.
* **`dvr_reference` was never on the path and is not needed.** The exhibit shows the
  ring-polymer number; the reference it is graded against is a constant read out of
  `C1_GATE_RESULTS.md`. A browser-side DVR would be a second, much larger question — 601
  grid points, four self-convergence solves, an independent Numerov — and nothing here
  measured it.

---

## 5. WHAT A SUCCESSOR SHOULD STAKE

Left for the next freeze, computed from the numbers above rather than guessed:

1. **Do not put the lower edge of the classical band at `kT`.** The quantity's truth is
   `1.0036 kT` and a single browser chain carries `sd = 1.25%` of `kT`. Stake the check
   two-sided about the MEASURED truth at a width the measured spread supports —
   `|E_1/kT − 1.0036| ≤ 4 sd` is `±5%`, passes a correct instrument ~100 times in 100, and
   still catches every failure the check exists for: a wrong `beta`, a wrong reduced mass,
   a collapsed ring, a broken estimator. A ±2% check is not available at this budget and a
   ±0.4% one costs 108× the sampling.
2. **Cache the curve.** §1 measured 0.389 s of per-call setup, 0.243 s of it
   `banked_range()`. Hold the `BankedPes` in the ABI's existing static and `P = 16` gets
   roughly a third of its wall back — which is where the sampling increase in (1) should
   be spent.
3. **The speed door is genuinely open, and by a lot.** `P = 16` at 2.216 s and `P = 32` at
   3.940 s, both measured at load 60 on a shared box. Speed is not what stands between C1
   and the workbench.
4. **Say the 6.27% out loud.** Whatever ships must state that a 32-bead exhibit reads
   93.7% of the certified answer and is showing convergence in progress. The honest exhibit
   is the LADDER — six rungs climbing from `kT` toward the reference — and not a single
   number claiming to be the zero-point energy.

## 6. WHAT RAN, AND WHAT IS LEFT BEHIND

The probe was a temporary `holon_c1_zpe_probe(beads: u32) -> f64` export appended to
`engine/crates/holon-render/src/lib.rs`, driven natively by one example and under node by
a `smoke.mjs`-shaped loader. **All of it is removed**: `lib.rs` is byte-identical to its
parent commit, no `smoke.mjs` block was added, and `docs/workbench/holon_render.wasm` was
never rebuilt or touched. The wasm artifacts of §1 were built to a scratch path and are
not in the tree.

**Nothing on the page becomes a lie because of this refusal.** `smoke.mjs`'s
`FENCE_JUSTIFYING_ABSENCES` — the inverted check that fails when a fenced capability
arrives — carries five entries (`holon_set_pressure`, `holon_phase_call`, `holon_q_tet`,
`holon_water_table_begin`, `holon_refinement_active`) and **none of them is C1**. The
workbench does not currently fence quantum nuclei; it simply does not mention them, so no
displayed claim depended on this door either way. The full gate was run against the tree
this lane leaves behind and passes 43/43.

A successor that DOES land the door inherits an obligation from that same block: a
capability that arrives and is not gated is worse than one that was honestly fenced, so
the door ships with its smoke block or it does not ship.

What remains is this document. The probe body is reproduced below so a successor can
recreate the measurement exactly rather than re-deriving which entry points it wired.

```rust
const C1_MAX_BEADS: u32 = 32;
const C1_KNOTS: usize = 1024;
const C1_TEMPERATURE_K: f64 = 300.0;
const C1_DT: f64 = 4.0;
const C1_STEPS_SAMPLE: u64 = 60_000;

#[no_mangle]
pub extern "C" fn holon_c1_zpe_probe(beads: u32) -> f64 {
    use holon_chem::elements::Species;
    use holon_chem::rpmd::{run_pimd, BankedPes, PimdConfig, Vib1D};

    if beads == 0 || beads > C1_MAX_BEADS {
        return f64::NAN;
    }

    let banked = BankedPes::h2(C1_KNOTS);
    let (_, v_min) = banked.minimum();
    let sys = Vib1D::h2(&banked);

    // The centroid friction the campaign declares: the curve's OWN harmonic frequency,
    // computed from the engine's curvature and the engine's masses, never a literal.
    let (r_e, _, _) = holon_chem::equilibrium();
    let curv = holon_chem::h2_point(r_e).e2;
    let mu = Vib1D::reduced_mass_me(Species::HYDROGEN.mass_u, Species::HYDROGEN.mass_u);
    let omega = (curv / mu).sqrt();

    let cfg = PimdConfig {
        p: beads as usize,
        temperature_k: C1_TEMPERATURE_K,
        dt: C1_DT,
        gamma_centroid: omega,
        steps_equil: C1_STEPS_SAMPLE / 10,
        steps_sample: C1_STEPS_SAMPLE,
        seed: 0xC1_0001,
    };
    let rep = run_pimd(&sys, &cfg, r_e);
    if rep.excursions != 0 {
        return f64::NAN;
    }
    rep.e_virial - v_min
}
```

The refusal arm returned `NaN` for `P ∈ {0, 33, 64}` on both builds, printed in §1's two
listings. It is recorded for the successor and it is NOT a gate result: the door it would
have guarded does not exist.

---

## 7. THE ONE-LINE ANSWER FOR THE FSD

§9b says C1 is "not workbench-facing this pass." **Measured: the reason is not speed.**
A 16-bead ring-polymer ZPE runs in the shipped browser artifact in 2.2 s at load 60,
returns the natively certified ladder to within 1.26σ at every rung, and needs no file
the browser cannot have. What the browser cannot have is `std::thread::scope`, so it gets
one chain instead of eight and roughly three times the error bar — and the gate this lane
staked around the classical limit was sized for the eight-chain precision and fired.
The door is buildable; this freeze was not the one to build it through.
