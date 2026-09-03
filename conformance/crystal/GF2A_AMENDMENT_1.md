# GF2a — Amendment A1: the MPS arm re-instrumented on the colour lanes, and G1 restated for the convergence the freeze itself assumed

*Frozen 2026-09-03, committed ALONE before the instrument (E7) exists, against
`GF2A_QCD2_PREREG.md` (frozen 2026-09-02) and `GF2A_QCD2_RESULTS.md` (the run that fired
G0 on the MPS arm). An amendment moves nothing that was read: G1–G3 were NOT read, so
nothing below re-stakes a reading. What it does: retires one instrument by the gate that
convicted it, admits its successor with its own gate and plants, and corrects one
criterion the freeze wrote against its own extrapolation.*

misfits: contacts M-NULL-MISSTAKE (G1's ratio criterion was staked against the shape
the freeze's own `1/N` extrapolation assumes: for a pure `a/N` tail the ratio of successive
differences at N = 6, 8, 10 is 0.60, so the criterion could not be met by data converging
exactly as the freeze expected — a mis-staked null, corrected here and NOT re-read),
M-VOLUME-SCALE (the volume standard N ≳ 20√x stands; the exact arm's N ≤ 10 is below it and
was never read as a volume), M-STALE-INSTRUMENT (the retired arm's binary is named by hash
in the results document; the successor is committed with its own), M-PLANT-OBS and
M-PLANT-SECTOR (two new plants, carriers asserted), M-ONE-MODEL-DELTA (the referee is the
exact colour-lane arm, itself gated against the two-string route and the dense Slater–Condon
referee; no external number), M-DEVICE-CLASS (the exact arm's device class is on every
row; the MPS arm is host-only until it rides the vector space), M-CHEAPER-THAN-ITS-PRICE
(the χ-ladder's price per point is printed). Not contacted: the rest of the freeze's list,
unchanged.

defects: none (no registry keyword contacted; the sector's failure mode is the freeze's
own M-STALE-INSTRUMENT / M-NULL-MISSTAKE contacts above, not a registered defect).

gauge: /home/emoore/CIRISHolon/conformance/crystal/GF2A_QCD2_RESULTS.md — the planted-truth
evidence for this amendment is the run it amends: G0 PASS on the exact arm and G0 FIRED on
the MPS arm in one document, the two sides of one gate on one tensor.

Family-wise correction: none is applied and none is owed — G0′, G1′, G2 and G3 are
separable kills read as individual pass/fire verdicts at their staked digits (rule 2 of
the discipline), not a family of p-values; no verdict is a statistic over a sample.

## A1.1 The retirement, by the gate that fired

The MPS arm as frozen — JW modes, 42-channel accumulator MPO, the sector enforced by the
penalty `λ(N̂ − n_q)²` — fired G0 at N = 8: χ = 64 energies 1.2e-4 (B=2) to 2.8e-2 (B=0,
x=4) above the exact arm, and the χ = 40 x=4 B=0 point read −51.8346 under the absolute
sweep tolerance and −51.3016 under the relative one. A discarded weight of 6e-6 cannot buy a
3e-2 error: the arm was STUCK in a penalty-sector metastable state, not truncated. It is
retired for this rung. Its JSONs stay as the evidence (`qcd2_dmrg/`), its binary hash is in
the results document, and nothing it produced is read.

## A1.2 The successor instrument (E7), admitted before it exists

**The same integer lanes the exact arm runs on, in the MPS.** Every bond index carries a
charge label `(n_r, n_g, n_b)`: the colour counts to its left. Site `j` is one JW mode of
colour `c(j) = j mod 3`, so an occupied site adds `e_c` to the label. The two-site update is
the dense one, with three changes and no fourth:

1. **No penalty term.** The MPO is the Hamiltonian, `λ = 0`.
2. **The local solve stays in the sector.** The two-site wavefunction is masked to the
   label-consistent entries (`q(l) + a·e_{c1} + b·e_{c2} = q(r)`) after every matvec, so
   roundoff cannot leak charge across a sweep.
3. **The SVD is blockwise by charge.** Rows `(l, a)` and columns `(b, r)` are grouped by the
   charge of the cut bond; each block is decomposed alone; the kept singular values are the
   largest `χ` across blocks and every kept bond state inherits its block's label. χ counts
   in-sector states only, which is the whole reason the arm is expected to converge where
   the penalised one stuck.

The start is the product state the freeze already uses (`product_start`), which has equal
colour counts by construction; its bond labels are computed from its occupations. The total
charge is fixed by the boundary label, never by a penalty.

## A1.3 G0′ — the successor's own gate (EXACT)

At N = 8, both x, all three sectors: `|E₀(exact arm) − E₀(MPS-sym, χ)| ≤ 1e-6` at some
χ ≤ 256 on a warm χ-ladder 32 → 64 → 128 → 256, the χ at which it is first met PRINTED
per sector, and the arm's convergence test stated in advance and gated: a sweep is
converged only when ALL of (a) the energy change between successive sweeps is
`≤ 1e-10·max(1, |E|)`, (b) the maximum discarded weight of the last sweep is `≤ 1e-8`, and
(c) at least four sweeps have run. A sector that meets none of the ladder's χ fires G0′
for the arm; the ladder is not extended past 256 without a further amendment. Digit-bearing:
1e-6, 256, 1e-10, 1e-8, 4. witness: none (a cross-solver identity; no Lean object covers
the number).

## A1.4 G1′ — the convergence criterion the freeze's own extrapolation implies

G1 as frozen demanded a ratio of successive differences below 0.5 over the three largest
N. A pure `a/N` tail — the very form G1 then fits — gives 0.60 at N = 6, 8, 10 and 0.67 at
N = 16, 24, 40, so the criterion was unreachable by data behaving exactly as staked. It is
replaced, NOT re-read (no G1 value has been read on any arm):

**G1′:** for each x, fit `M_B(N) = M_∞ + a/N + b/N²` over the four largest N of the ladder,
print the fit residual per point, and require (i) `M_∞ > 0`, (ii) every residual
`< 1e-3·M_∞`, and (iii) `M_∞` from the three largest N alone within 5% of `M_∞` from all
four. A mass whose `M_∞` drifts between windows by more than 5% is NOT converged and the
reading is not cashed; a negative or non-finite `M_∞`, or a mass that rises with N, kills
the reading that the B = 1 ground state is a hadron, exactly as G1 said. Digit-bearing:
1e-3, 5, four. witness: `macro_law_forced` (Fold I's "the coarse law is forced", the
CIRISOntology name of the theorem G1 cited as `closure_determines_dynamics` in the
CIRISHolon tower: the hadron's mass is derived, and the theorem covers the derivation, not
the number).

G2 and G3 are unchanged in wording and in digits.

## A1.5 Plants (M-PLANT-OBS, M-PLANT-SECTOR)

- **(iv) The labels are load-bearing.** Mutation: the SVD ignores the labels (one dense
  decomposition, as the retired arm did) while the penalty stays off. At N = 8, x = 4,
  B = 0, χ = 64, the mutant must land more than `1e-3` from the exact arm's energy, and
  the successor must not. Carrier nonzero in the sector: the retired arm's own 2.8e-2 miss
  at this point is the effect the mutant must reproduce in kind.
- **(v) The sector is the boundary's, not a slider's.** A product start with unequal colour
  counts (n_r ≠ n_g) must be REFUSED by the symmetric sweep by name — it names the
  Cartan-neutral block it cannot leave — rather than solved to some number. EXACT.
- Plants (i)–(iii) of the freeze stand and are re-run on the successor at N = 4.

## A1.6 What is read, and when

Nothing until G0′ passes at N = 8 on the successor. Then the ladder as frozen (x = 4:
8, 16, 24, 40; x = 9: 8, 24, 40, 60) runs on the successor at the χ that met G0′ and one
rung above it (the χ-premise), and G1′, G2, G3 are read for the first time, on the MPS
arm's points with the exact arm's N ≤ 10 as the referee where both exist. The exact arm's
N = 10 (16 million determinants, resident on the device) is added to the FCI column of the
grid as a referee point; it changes no gate.

## A1.7 Cost model, stated

Per MPS point: 24 → 180 sites, χ ≤ 256, the two-site solve's cost `∝ χ³` per site per
Lanczos iteration; the χ = 64 N = 8 point measured 30 min under the retired arm's
metastability and is expected to fall with the penalty gone (a prediction, printed against
the measurement). The exact referee: minutes per column on the device.
