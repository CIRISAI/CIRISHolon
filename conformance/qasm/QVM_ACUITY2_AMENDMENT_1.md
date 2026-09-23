# QVM-ACUITY-2 — AMENDMENT 1: the probe runs at cadence through the WHOLE circuit, the certificate is `Σ√w`, and the build's open choices declared before the campaign seed runs

*Written 2026-09-23 on building `holon::views`, the `qvm_acuity2` driver and the plants — AFTER
PQ-2 fired against the probe as frozen and after two scouts (seed 1: twelve instances, then
eight through the sweep), and BEFORE the campaign seed (2026, item 7) runs. So this is not an
amendment before all data: it is a repair the prereg's own branch (b) orders ("the search's
closure test for that view is the fault; named, repaired, re-frozen"), with the frozen text
kept (`QVM_ACUITY2_PREREG.md` unchanged) and the frozen reading RUN AND GRADED BESIDE the
amended one on every instance, labelled. No stake, kill, plant or threshold is moved.*

1. **The MPS probe as frozen is blind to depth, and PQ-2 fails under it.** §1: "the circuit
   run at bond caps `χ ∈ {2,4,8,16}` on a `2n`-gate prefix … closed at `χ` if the discarded
   weight is under the acuity's share", projected to the whole circuit. On PQ-2's carrier (a
   `20n`-deep brickwork) the first `2n` gates are ~1.3 layers, nothing is discarded, and the
   probe CLOSES at `χ = 2` at `n = 12, 16, 20`; the same `χ = 2` over the whole circuit
   certifies `2.0–3.4` against `ε ≈ 10⁻³–10⁻⁴` (test
   `pq2_as_frozen_the_prefix_probe_is_blind_to_depth`, asserted so the finding stays seen).
   On the scouts the frozen probe selected MPS at `χ = 2` on T, L and D instances, with
   certificates `1.5–3.4` and errors `2·10⁻³–7·10⁻²` — S3's "certificate `≤ ε`" violated on each.
   Under the frozen text that is branch (e): nothing is read. **Repair:** "at cadence" is read
   as a measurement REPEATED every `2n` circuit gates through the whole circuit: at each
   multiple of `2n` the certificate accumulated so far is projected linearly to the whole
   circuit (`× G/g`), and `χ` is closed only if no checkpoint's projection, and not the final
   certificate, exceeds `ε`; `χ` ascends through `{2, 4, 8, 16}` and the first to pass closes.
   Under this reading PQ-2 fires on all twelve of its cells (`n ∈ {12,16,20}` × amplitude /
   marginal × `ε_rel ∈ {10⁻¹, 10⁻²}`), every `χ ≤ 16` projecting `7`–`127` against `ε`. The
   price: at the closing `χ` the probe has run the whole circuit, so the search's own wall is
   reported per instance (`probe_wall`, `search_wall`) and is NOT added to the selected
   view's wall — S2 compares view walls, as staked — and it is printed so it cannot hide.
   Both probes run on every instance; S1–S3 are graded under both; **the cadence reading is
   the reading of record.**
2. **The certificate is `Σ_k √w_k`, not "the summed discarded weight".** `Σ w` bounds the
   SQUARED error (Verstraete–Cirac's `‖φ−ψ‖² ≤ 2Σw` is for one canonical sweep); one truncation
   of weight `w = 10⁻⁴` makes an error of `10⁻²`. With the orthogonality centre moved onto every
   pair before it is split, each truncation is an orthogonal projection removing a component
   of norm exactly `√w_k`, and the triangle inequality gives `‖φ − ψ‖ ≤ Σ√w_k` with no
   renormalisation anywhere; on the observable, `δ` for an amplitude and `δ(1 + ‖ψ‖)` for a
   marginal. PQ-3 measured on a four-pair state with the analytic spectrum: never below the
   true 2-norm error, worst ratio `1.83` (`4` staked), exactly `1.000` where one cut truncates.
3. **S3's inequality for the exact views is read with the referee's floor.** The tableau view
   is exact in `Z[ω]` and the dense view is `f64`; their certificate is `0`, the referee is an
   `f64` statevector, and `|value − referee|` is `~10⁻¹⁷`, not `0`. S3 is read as
   `|value − referee| ≤ certificate + τ` with `τ = 10⁻¹²` (ACUITY-1's exactness threshold) and
   `certificate ≤ ε` as written.
4. **The grid §2 left open, fixed.** Instance `i` of a family: `n = [12,16,20][i mod 3]`
   (D: `12`); an amplitude for even `i`, the 4-qubit marginal on wires `0–3` for odd `i`;
   `ε_rel = 10⁻¹` when `⌊i/2⌋` is even, else `10⁻²`; T: `t = [12,16,20][⌊i/3⌋ mod 3]`; L and D:
   `t = 40 + (s mod 9)`. The amplitude's `y` is the referee's argmax (ACUITY-1's rule, declared
   before any view runs), the marginal's pattern its first four bits. **L's "depth ≤ 6" counts
   ENTANGLING layers**: an `H` layer, then six layers each of `CX` on alternating
   nearest-neighbour pairs followed by one single-qubit gate per wire, exactly `t` of the `6n`
   single-qubit slots `T`/`T†`, the rest `H` or `S` (depth counted over all layers could not
   carry `t ≥ 40` at `n = 12`: 36 slots). A `CX` brick has operator Schmidt rank 2, so at depth 6
   no bond exceeds `2³ = 8` — named HERE, before the read, as what S4 will meet: an MPS capped at
   `χ = 64` never holds more than `8`.
5. **The prices, with the constants measured once.** TABLEAU `c_tab·n²·G`; DENSE
   `c_dense·2ⁿ·G`; SUM `N(t_eff)·(G+t)(n+t)·c_sb + N·legs·(n+t)²·c_sa` (branch construction with
   the certified bound, then the fold; `legs = 2^{|L|−4}` for a marginal, each leg at acuity
   `ε/(4√legs)` so that `Σ(2|a|R + R²) ≤ ε`); MPS `ops·(c_m0 + c_m3·χ³)` with `ops` the
   swap-routed two-site count (`≤ n·G`, which §1's `n·G·χ³` is the worst case of). Calibrated
   once at the start of each driver run on fixed circuits (seed `0xCA11`, in no family) and
   printed as `calibrate:`.
6. **The by-hand runs.** TABLEAU where closed (else not run); SUM at `ε` in a child process
   killed at `60 s` (the branch construction cannot be interrupted in-process), `N > 2¹⁸` not
   attempted and recorded `> 60 s`; MPS by hand = the smallest `χ ∈ {2,4,8,16,32,64}` whose
   certificate reaches `ε` (each attempt stopped the moment its certificate passes `ε`), its
   wall that attempt's; DENSE on its own gate kernels, sharing none with the referee's. "The
   best wall" is over the views that REACHED `ε`. A run under `50 ms` is timed as the median of
   up to five.
7. **The campaign seed is 2026**; the scouts ran seed 1 and are not read. stim's column on
   family C is `TableauSimulator.do` plus peek/postselect of the observable's qubits (`|a|²` for
   an amplitude: stim carries no phase), construction untimed (`bakeoff.py`'s rule), on the same
   cores.
8. **What is printed before the run**: `select[cadence]: view=… price=… alternatives=[…]` and
   `select[prefix]: …`, both before any view executes.

---
witness: `Closed` (Object.lean), `tableau_not_closed_under_rotation` and `tableau_closed_under_hadamard` (Stabilizer.lean) for the TABLEAU view's closure, unchanged from the prereg; none new (items 1–8 are measured gates and a triangle inequality, stated in words)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-CHEAPER-THAN-ITS-PRICE, M-TRUNCATION-AS-ERRORBAR, M-DEVICE-CLASS, M-PLACEMENT-LOTTERY, M-HOMOG, M-PARITY-PROTECT, M-VACUOUS-SUCCESS, M-FLOOR-UNSTAKED, M-MAINTENANCE-LENS, M-COND-PROBE, M-STALE-INSTRUMENT — contacted by keyword, cited.
Carrier-sector statement (M-PLANT-SECTOR): PQ-2's carrier (a `20n`-deep brickwork) and PQ-3's (four pairs pulled across the middle cut by a SWAP network) are named above; the sector each acts on — the entanglement the probe must see, the truncated Schmidt weight — is nonzero in that carrier by construction and measured nonzero.
