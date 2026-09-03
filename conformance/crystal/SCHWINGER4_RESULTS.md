# SCHWINGER-4 — the residual interaction between two screened pairs decays at the banked meson mass: G1 BRANCH (a) ON BOTH COLUMNS (engine arm)

*2026-09-02. Prereg `SCHWINGER4_PREREG.md`, frozen alone (36fd4c1) with amendment A1
(pre-data, driver admission, a0de369). Node GF0 of the fold below the atom. This
document is APPENDED as arms complete, never edited: §1–§7 are the engine arm's
record; §8 is the Python arm's cross-check and is PENDING at the time of writing.*

## 0. Verdict, and its one open condition

**On the engine arm, G1 reads branch (a) on BOTH columns**: the four-point residual
interaction `V(d)` between two screened static pairs decays exponentially at the
banked per-column vector-meson mass —

| column | κ_fit (per site) | κ_pred = M(x,N,χ=64)/√x | ratio | R² | fit points |
|---|---|---|---|---|---|
| x = 9, N = 84 | 0.214360 | 0.215548 | **0.9945** | 0.9999992 | 7 (d = 8 … 24) |
| x = 4, N = 56 | 0.342937 | 0.340834 | **1.0062** | 0.9999976 | 5 (d = 8 … 16) |

against a staked band of `[0.8, 1.2]`. The rate is the gap to **0.6 %** on both
columns; the band was ±20 %. Plant (ii) FIRED as staked (§5). The χ-premise held
with eight to ten orders of margin (§2); both columns are screened (§3).

**The open condition.** Amendment A1 admits the engine arm for the staked verdict
only if three cross-check points per column agree with the banked Python driver
within the χ-premise band. Those Python points are RUNNING (its first N = 56 ground
state had not completed after 75 minutes; the engine arm computed 122 ground states
in the same interval). Until §8 is appended, the verdict above is the ENGINE ARM'S
READING and the campaign's staked verdict is formally pending. Nothing in §1–§7
will be changed by §8; it can only be confirmed or VOIDed for a column.

## 1. Provenance (M-PROVENANCE-OVERREACH, M-STALE-INSTRUMENT)

| field | value | |
|---|---|---|
| driver | `q8-mps::schwinger` + `examples/schwinger4.rs`, one process per configuration | engine arm |
| binary_sha256 | `d2522d1800ecac2027f0faf66b233c8877c94370832ee93af8e81a1844a0121e` | MEASURED |
| crate commit | `a0de369` (the driver, its gates and the scheduler, committed with the amendment) | MEASURED |
| rustc | 1.95.0 (59807616e 2026-04-14) | MEASURED |
| scheduler | `instrument/schwinger4.py` (numpy 2.4.0, scipy 1.16.3 for the Python arm and the fit) | MEASURED |
| machine | the campaign box, 32 logical cores, 31 GiB; loadavg 11–29 during the fan-out; the Python arm's three processes and the earlier SCHWINGER-3-style single-process runs shared it | MEASURED, not a gate |
| gauge before any staked point | plant (i) on the engine driver: three N = 12 referees, worst diff 4.0e-11 vs 1e-6; `tests/schwinger_gauge.rs` (MPO vs independent dense spectrum 1e-9, sweep vs dense ground state 1e-9, referees 1e-6): 3/3 | MEASURED |
| checkpoints | `ckpt4_rs_*.npz` beside this file; column files `schwinger4_rs_x{4.0,9.0}.json`; analysis `schwinger4_rs_analysis.json` | |

Every point: two-site DMRG from the Néel product state, no RNG, Lanczos residual gate
1e-10, sweep tolerance 1e-9 on the energy; all 122 points converged in 4 sweeps (8–9
under the coulomb-off mutation) with worst Lanczos residual 1.0e-11.

## 2. The χ-premise, per checked point (EXACT band, staked)

| x | d | V(χ=40) | V(χ=64) | \|gap\| | band max(1e-4, 0.05\|V\|) | |
|---|---|---|---|---|---|---|
| 9 | 8 | −8.268864e-02 | −8.268864e-02 | 7.9e-11 | 4.13e-03 | ok |
| 9 | 12 | −3.500688e-02 | −3.500688e-02 | 4.6e-11 | 1.75e-03 | ok |
| 9 | 24 | −2.678151e-03 | −2.678151e-03 | 1.4e-11 | 1.34e-04 | ok |
| 4 | 6 | −7.145022e-02 | −7.145022e-02 | 1.4e-14 | 3.57e-03 | ok |
| 4 | 10 | −1.795828e-02 | −1.795828e-02 | 1.0e-12 | 8.98e-04 | ok |
| 4 | 16 | −2.300720e-03 | −2.300720e-03 | 1.4e-14 | 1.15e-04 | ok |

Bond dimension was never the limiting error (max discarded weight 6e-13 at χ = 40).

## 3. The screening premise, per column

| x | E1(s=2) − E0 | E1(s=3) − E0 | \|Δ\| vs 0.5·(E1(s=2) − E0) | |
|---|---|---|---|---|
| 9 | 1.692132 | 1.884640 | 0.193 vs 0.846 | screened |
| 4 | 1.578401 | 1.655288 | 0.077 vs 0.789 | screened |

A pair's energy saturates with its size: the object the fold names — a screened,
gauge-invariant pair — was constructed, not a string.

## 4. V(d), both columns (χ = 40; the fit window is d ≥ 8, noise floor 1e-3)

| d | V(d), x = 9 | V(d), x = 4 |
|---|---|---|
| 2 | −2.995643e-01 | −2.858169e-01 |
| 3 | −2.509728e-01 | −2.194778e-01 |
| 4 | −1.953287e-01 | −1.430799e-01 |
| 6 | −1.271211e-01 | −7.145022e-02 |
| 8 | −8.268864e-02 | −3.576583e-02 |
| 10 | −5.379261e-02 | −1.795828e-02 |
| 12 | −3.500688e-02 | −9.039323e-03 |
| 14 | −2.279115e-02 | −4.557876e-03 |
| 16 | −1.484389e-02 | −2.300720e-03 |
| 20 | −6.302520e-03 | — |
| 24 | −2.678151e-03 | — |

No point fell under the noise floor; every staked d entered its column's fit. The
interaction is attractive at every separation for like-oriented pairs.

## 5. Plant (ii) — the gate fires on the planted defect

Under `coulomb-off` (free staggered fermions in the static site potential, x = 4
column, 28 configurations): `V(2) = −8.170` (carrier nonzero, asserted), the
"decay" over the window fits κ = 0.0459 with R² = 0.9993 — a ratio of 0.135 to
κ_pred — so G1 reads **(b)**, never (a): **FIRES**, as staked. The free-fermion
response is a slow, near-linear drift (−6.45 at d = 8 to −4.47 at d = 16), nothing
like the gap-set exponential; a gate that could not tell the two apart would have
been theatre, and this one can.

## 6. G2 — the sign reading (recorded, not a gate), and a post-data bookkeeping correction

As frozen, `V_flip(d)` subtracted the UNFLIPPED single-pair energy at the second
pair's position. On a staggered chain a `(−, +)` pair is not the mirror of a `(+, −)`
pair at the same sites, and the difference is position-dependent (measured: E1f − E1
from −8.0e-04 at p = 46 to −4.1e-03 at p = 54), so the frozen reading was distorted by
a d-dependent offset (it reported "signs flip = False", κ_flip = 0.2607). The
correction — seven extra engine points, the flipped pair's OWN self-energy at each
position, computed after the column and labelled here as POST-DATA — gives:

| d | V (like-oriented) | V_flip (corrected) | sign flips |
|---|---|---|---|
| 8 | −8.268864e-02 | +8.301273e-02 | yes |
| 10 | −5.379261e-02 | +5.121360e-02 | yes |
| 12 | −3.500688e-02 | +3.557050e-02 | yes |
| 14 | −2.279115e-02 | +2.185724e-02 | yes |
| 16 | −1.484389e-02 | +1.519547e-02 | yes |
| 20 | −6.302520e-03 | +6.479380e-03 | yes |
| 24 | −2.678151e-03 | +2.759883e-03 | yes |

κ_flip = 0.210975 (R² 0.99947), within the recorded ±0.05·κ_pred band of the unflipped
fit (|Δ| = 0.0034 vs 0.0108). Seven of seven signs flip and the magnitudes agree to a
few per cent: the residual interaction is the dipole-like exchange, not a boundary
artefact. G2 was recorded, not gated, so this correction changes no verdict; it is
kept because the frozen bookkeeping was wrong and the record says so.

## 7. Cost, as it was and as it should have been stated

The prereg's cost model said "under 40 ground states" per campaign. It undercounted:
the per-position `E1` energies, the seven flipped configurations at x = 9, the χ = 64
checks and the screening point make 50 configurations at x = 9, 37 at x = 4, 28 for
the plant and 7 for the G2 correction — **122 ground states**. Not a gate; recorded
because a stated cost model that is wrong by three is the kind of number this record
does not launder. On the engine arm a χ = 40 point cost 5–28 s and a χ = 64 point
70–210 s; the whole engine arm completed inside the time the Python arm needed for
part of one point — the DRY finding the operator's question surfaced: the campaign
had been LAUNCHED on the referee, not on the engine.

## Meaning (from the freeze, applied)

Branch (a) on both columns ⇒ **Fold II has its first measurement below the atom**: the
residual interaction between two screened, gauge-invariant objects decays at the gap —
at the banked vector-meson mass to 0.6 % — so the hadron tier's far field is free at
this scope, and its closure defect is a DERIVED number (the gap) and not a fit. It
licenses GF2's use of an exponentially convergent expansion over hadrons in the 1+1D
toy and **nothing in three dimensions**. Fence, unchanged: QED₂, one flavour, U(1), the
vector meson the lightest exchange; static (infinitely heavy) sources; open chain;
the columns' own finite-N masses as referee (the continuum limit is SCHWINGER-3's,
not re-taken here).

## 8. The Python arm's cross-check (amendment A1) — CONFIRMED on both columns

*Appended 2026-09-02 (evening) when the Python driver's staked points completed;
`schwinger4.py crosscheck 4.0` / `9.0` read against the checkpoints, verbatim.*

| x | point | Python (`dmrg_schwinger.py`, χ = 40) | engine arm (`schwinger4`, χ = 40) | \|diff\| | band | |
|---|---|---|---|---|---|---|
| 4 | E0 | −127.2036725856 | −127.2036725856 | 1.54e-11 | ≥ 1e-4 | ok |
| 4 | E2(d = 6) | −124.1180890897 | −124.1180890897 | 1.44e-11 | ≥ 1e-4 | ok |
| 4 | E2(d = 10) | −124.0641389572 | −124.0641389572 | 1.47e-11 | ≥ 1e-4 | ok |
| 9 | E0 | −454.7034805665 | −454.7034805667 | 1.06e-10 | ≥ 1e-4 | ok |
| 9 | E2(d = 8) | −451.4016421246 | −451.4016421247 | 1.14e-10 | ≥ 1e-4 | ok |
| 9 | E2(d = 12) | −451.3536347296 | −451.3536347297 | 1.13e-10 | ≥ 1e-4 | ok |

Three of three staked points per column agree six to seven orders inside the χ-premise
band `max(1e-4, 0.05·|V|)`. **The engine arm's admission (A1) stands for both columns,
and the verdict of §0 — G1 branch (a) on both columns — is no longer formally pending:
it is the campaign's staked verdict.** Nothing in §1–§7 changed.

Beyond the three staked points, the Python arm's other completed configurations
(x = 4: E2 at d = 2, 3, 4, 8; x = 9: E2 at d = 2, 3, 4, 6) agree with the engine arm's
to ≤ 1.6e-10 on every one (column logs `schwinger4_x4.0.log`, `schwinger4_x9.0.log`
against `schwinger4_rs_x{4.0,9.0}.json`); they are recorded, not gated. The two drivers
are independent codes on one Hamiltonian — the Python driver banked for SCHWINGER-3
(its provenance in `instrument/PROVENANCE.sha256`) and the engine's `q8-mps` behind the
`schwinger4` example (binary SHA-256 in `schwinger4_rs_x*.log`) — and their agreement at
1e-10 is a cross-solver identity on the energies, not a statement about the fit.

Cost, recorded (M-CHEAPER-THAN-ITS-PRICE): the Python points ran 1,000–3,600 s each
under machine loads of 6–35; the engine arm's matching points ran 8–25 s. The ratio is
the reason A1 admitted the engine arm and the reason this section is the last one the
Python driver is asked to write for this campaign.
