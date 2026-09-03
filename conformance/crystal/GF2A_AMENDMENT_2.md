# GF2a — Amendment A2: the successor instrument after G0′ fired, staked on the E14 base

*Frozen 2026-09-03, committed ALONE before its instrument exists, against
`GF2A_QCD2_PREREG.md` (frozen 2026-09-02), `GF2A_AMENDMENT_1.md` (frozen 2026-09-03) and
the E7/E14 sections of `GF2A_QCD2_RESULTS.md`. G0′ FIRED on A1's instrument and stays
fired; nothing here re-reads it. What this does: names the mechanism that fired it, stakes
the successor's gate with the same digits, stakes the quantity that IS an error bar, restakes
the χ-ladder from the rank of the cut rather than the count of its labels, adds the
resumability gate the volume rungs need, and re-derives the plants for THIS instrument.*

misfits: contacts M-EXIT-DISCRIMINATOR (A1's record kept a residual that sat at its own
stopping tolerance on right and wrong answers alike, and a `converged` flag that read true
on both; this instrument records the exit and reads it, and no residual is a criterion),
M-TRUNCATION-AS-ERRORBAR (the discarded weight is REPORTED and never gated as error; the
variance is the error bar, staked in V1), M-PLANT-OBS and M-PLANT-SECTOR (three plants
re-derived for this instrument, each with its carrier and the sector it acts in),
M-STALE-INSTRUMENT (every binary named by hash in the results document), M-DEVICE-CLASS
(the device backend is admitted only by its bit-identity gate and every row states its
class; host and device may not be mixed in one row), M-CHEAPER-THAN-ITS-PRICE (cost
per point printed as seconds and Lanczos iterations; the cost model below is a scaling
statement, not a price), M-VOLUME-SCALE (N ≳ 20√x is the volume standard — 40 at x = 4, 60
at x = 9 — and only the ladder's tops reach it; N = 8 is an exam, never a volume),
M-MAX-OVER-SUCCESSES (the χ per N is derived from the cut's rank, never taken as the
largest χ that happened to pass), M-UNTESTED-GAP (V1 is staked only where data exists,
N = 8), M-NULL-MISSTAKE (G1′ stands as A1 corrected it), M-BUDGET-LAUNDER (a rung the
lease refuses VOIDs its point loudly and is never scored), M-PROVENANCE-OVERREACH (a
sha256 names the file it hashed and nothing else), M-PLACEMENT-LOTTERY (wall clock is
printed as context; the price is the Lanczos count). Not contacted: the rest of the
registry.

defects: none (no registry keyword contacted beyond the misfits above).

gauge: /home/emoore/CIRISHolon/conformance/crystal/GF2A_QCD2_RESULTS.md — the planted-truth
evidence is the E7 record: G0′ FIRED on the frozen warm ladder (+6.63e-5 at x = 4, B = 1),
the cold diagnostic PASSED at the same χ (+4.27e-7), and the E14 mixed warm ladder PASSED
(+4.27e-7) — three arms of one gate on one sector, in one document.

Family-wise correction: none is applied and none is owed — G0″, V1, R1, the plants and the
inherited G1′, G2, G3 are separable kills read as individual pass/fire verdicts at their
staked digits (rule 2), not a family of p-values.

## A2.1 The mechanism, named

A labelled two-site update can only produce labels that combine its two neighbours'
existing ones (A1.8.2), and a block the state carries no weight in is never rescued by a
split that ranks blocks by the state's own weight. A warm χ-ladder therefore inherits the
truncated label set and the basin of its lowest rung: measured, the frozen ladder read
+6.63e-5 at χ = 256 where a cold start at the same χ read +4.27e-7, and label re-seeding
alone reached only +2.65e-5. The remedy is the growth rule `engine/Q10_PREREG.md` §4 names:
new bond directions drawn from the Hamiltonian applied to the state — White's
density-matrix perturbation in the labelled split (`SymConfig::mixing`), with its schedule
(mixing while the sweep's energy change exceeds 100·rtol·|E|, unmixed once it does not,
convergence declared only on an unmixed sweep). With it the same warm ladder read +4.27e-7.

## A2.2 The instrument (E14, `GF2A_QCD2_RESULTS.md`)

The labelled two-site sweep of A1.2 with: (1) the block-sparse operator, bit-identical to
the dense one; (2) the change instrumentation; (3) label re-seeding between rungs AND White's
mixing at α = 1e-4 with the schedule above; (4) the exact variance `⟨H²⟩ − ⟨H⟩²` through the
squared MPO, priced by bytes and refused by name above its lease; (5) the operator on the
device where it fits, host otherwise, the class printed per row. The start of every ladder is
the seeded random labelled start (A1.8.1). Convergence per rung is A1.3's test unchanged:
(a) energy change ≤ 1e-10·max(1,|E|), (b) the last sweep's maximum discarded weight
≤ 1e-8, (c) at least four sweeps — with (b) understood as a production leg per Q10 §5, never
as an error bar.

## A2.3 G0″ — the successor's gate (EXACT)

At N = 8, both x, all three sectors: the MIXED warm ladder 64 → 128 → 256 lands within
`|E₀(exact arm) − E₀(MPS)| ≤ 1e-6` at χ = 256, and the cold start at χ = 256 agrees with
the mixed top rung within 1e-6 (two routes to one floor). A sector that misses either fires
G0″ for the arm; the ladder is not extended past 256 without a further amendment.
Digit-bearing: 1e-6, 256, 1e-4. witness: none (a cross-solver identity; no Lean object
covers the number).

## A2.4 V1 — the variance is the error bar (a forward prediction, discipline rule 6)

On every N = 8 sector the variance at the three rungs is STRICTLY DECREASING in χ, and on
the sector that fired G0′ (x = 4, B = 1) the variance of the frozen unmixed χ = 256 state
(miss +6.63e-5) exceeds that of the mixed χ = 256 state (miss +4.27e-7) by at least 10×. A
non-monotone sequence, or a ratio under 10, fires V1: the quantity is then not the error
bar this instrument claims, and the claim is withdrawn rather than the digits moved. The
magnitudes are REPORTED beside every row; no magnitude is staked, because none was
measured before this freeze (M-UNTESTED-GAP). Digit-bearing: 10, three rungs.
witness: none (a prediction about the instrument, not a theorem).

## A2.5 R1 — resumability (EXACT)

Every rung writes its full state (tensors, labels, sweeps done, the convergence test's
memory) after each completed sweep, and a run interrupted after any complete sweep and
resumed from that state reproduces the uninterrupted run's energy, Lanczos count and
discarded weights TO THE BIT. Gated on N = 6, B = 2 and on one N = 8 rung before any volume
rung is launched; a resumed volume rung carries `resumed_from_sweep` in its row.
Digit-bearing: EXACT. witness: none (an identity of the instrument with itself).

## A2.6 The ladder, restaked from the rank

A1.6's ladder ran "at the χ that met G0′ and one rung above". The rank counting
(`GF2A_QCD2_RESULTS.md`, "the ladder was scaled to the label count") shows the label
count at the middle cut grows as (N/2 + 1)³ — 125, 729, 2,197, 9,261, 29,791 at N = 8, 16,
24, 40, 60 for B = 0 — so a fixed χ is a different fraction of every N. The χ per N is
therefore STAKED, two rungs each, the lower cold-started and the upper warm with mixing:

| N | χ_lo → χ_hi |
|---|---|
| 8 | 128 → 256 |
| 16 | 256 → 512 |
| 24 | 512 → 1024 |
| 40 | 1024 → 2048 |
| 60 | 1024 → 2048 |

The χ-premise per point: `|M_B(χ_hi) − M_B(χ_lo)| ≤ 1e-3·|M_B(χ_hi)|`, else the point is
VOID (printed, never read). A rung the VRAM or host lease refuses VOIDs its point by name.
The grid is the freeze's: x = 4 at N = 8, 16, 24, 40 and x = 9 at N = 8, 24, 40, 60, all
three sectors. Digit-bearing: the table, 1e-3. witness: none.

## A2.7 G1′, G2, G3

Unchanged in wording and in digits from A1.4 and the freeze. The one correction is to the
witness line G1′ carried: `macro_law_forced` is CIRISOntology's theorem
(`Core/Closure.lean`) and resolves in no Lean file of this repository, so its honest form
here is witness: none (the theorem lives in the sibling repository and this audit cannot
resolve across repositories; the reading it covers is the derivation, not the number).

## A2.8 Plants (M-PLANT-OBS, M-PLANT-SECTOR)

- **(vi) Mixing is load-bearing.** The x = 4, B = 1 warm ladder with α = 0 must MISS the
  1e-6 stake at χ = 256 by more than 1e-5, and with α = 1e-4 must MEET it. Carrier, nonzero in the sector the plant acts on (x = 4, B = 1, the cut of χ = 256): the
  inherited-basin excess, recorded at +6.63e-5 on this sector; both halves are run and both
  are read. EXACT.
- **(vii) The labels are load-bearing, restated where the successor passes.** At x = 4,
  B = 1, χ = 256, cold: the labels-ignored mutant (`ignore_labels`, A1.5's shape) must land
  more than 1e-3 from the exact arm while the successor meets 1e-6. A1.5's plant (iv) was
  staked at χ = 64, under the cut's rank, where the successor could not pass either
  (`GF2A_QCD2_RESULTS.md`: void, corrected). Carrier, nonzero in the sector the plant acts on (the same cut): the retired arm's
  wandering, in kind — its recorded 2.8e-2 at B = 0 is the effect the mutant reproduces.
  EXACT.
- **(viii) The variance separates what the residual cannot.** On x = 4, B = 1 at χ = 256,
  the variance of the frozen unmixed state exceeds the mixed state's by ≥ 10× while their
  Lanczos residuals agree within a factor of 2 (both at the stopping tolerance, as
  recorded). Carrier, nonzero in the sector the plant acts on (x = 4, B = 1, χ = 256): the recorded 156× miss ratio between the two states. Digit-bearing:
  10, 2.
- Plants (i)–(iii) of the freeze and (v) of A1 stand and are re-run on this instrument at
  N = 4.

## A2.9 What is read, and when

Nothing until G0″, V1 and R1 pass at N = 8 and the three plants fire as staked. Then the
ladder of A2.6 runs, every rung checkpointed, and G1′, G2, G3 are read for the first time on
this arm, with the exact arm's N ≤ 10 as the referee where both exist. A resumed run is a
run; an interrupted run whose rung did not complete is not a point.

## A2.10 Cost, as a scaling statement

Measured at N = 8 on the E14 instrument: one matvec 3.98 ms host (4 threads) / 0.59 ms
device at χ = 256; the mixed warm ladder 64 → 128 → 256 on x = 4, B = 1 in 1,001 s host.
Beyond N = 8 no price is predicted: the live fraction of the two-site tensor falls as
1/labels and the cost per matvec rises as χ³ times that fraction, so the only honest
statement is that each rung prints its own seconds and Lanczos count, and a rung that does
not fit its lease VOIDs.
