# GF1 — AMENDMENT 1: the instrument's true price, the non-local magic defined so it can vary, and the ladder re-sized to what can be read exactly

*Written 2026-09-19, committed alone, AFTER the instrument was built and its plants fired
(`q8-mps/src/magic.rs`, `tests/gf1_plants.rs`, merged at `526befb`) and BEFORE any vacuum
ladder is read. Amends `GF1_PREREG.md` in the three places its correction names. The stakes
S1, S2 and S4 stand; S3 is re-staked on a quantity that is not a theorem's constant.*

## A1 — the price, derived this time

The exact fourth-power sum over Pauli strings is a four-replica contraction: `χ⁸` doubles of
state per site and `64 χ⁹` multiply-adds per site. Measured on the merged instrument, single
thread: `N = 24, χ = 8` in `42` s; `N = 16, χ = 10` in `62` s; `χ = 12` refused at `13.8` GB.
**The exact instrument reads `χ ≤ 11`.** Its floor is `≤ 10⁻¹⁴` absolute (replica against brute
enumeration at `N = 10`: `3.6 × 10⁻¹⁵`; two contraction orders at `N = 24`: `7.9 × 10⁻¹⁴`, after
the χ⁸-term overlap was moved to pairwise summation — the sequential sum had left
`7.8 × 10⁻¹³`), which retires the prereg's `4^N ε` estimate.

Beyond `χ = 11` two exits exist and neither is built: a compressed Pauli MPS
(Tarabunga, Tirrito, Bañuls, Dalmonte 2024), which is approximate and would need its own
error gate; and perfect Pauli sampling with a statistical bar (Lami & Collura 2023), which
is exact in expectation and carries its own variance. **If a ladder point needs `χ > 11` to
pass its variance gate, the point is REFUSED rather than read approximately**, and the
refusal is a reading about the vacuum's bond dimension, banked as such.

## A2 — the non-local magic, defined so a theorem does not hold it fixed

`M₂` is invariant under every Clifford, so minimising it over local Cliffords does nothing.
What prices a stabilizer-rank simulation beyond a product-state prefactor is the magic that
survives the best LOCAL UNITARY frame — the minimum of `M₂` over `⊗_j U_j` with `U_j` a
continuous single-site rotation. On a real MPS that is one angle per site:

> **`M₂^nl = min_{θ_1..θ_N} M₂( ⊗_j R_y(θ_j) |ψ⟩ )`**, `R_y(θ)` the real rotation on the
> physical index, by coordinate descent over sites with a golden-section line search on each
> `θ_j ∈ [0, π)`, three sweeps, the minimum over sweeps; deterministic (sites in order, the
> search seeded from `θ = 0`).

A product state's `M₂^nl` is zero exactly (a single-site rotation takes each factor to
`|0⟩`) — plant P3 is re-staked on that, and it can now fire. A GHZ state's is not zero: its
magic is in the entanglement. `M₂^nl ≤ M₂` by construction, and the gap `M₂ − M₂^nl` is the
local, removable part.

**S3, re-staked:** at each coupling `x`, the least-squares slope of `M₂^nl(N)` in `N` over the
volumes read is under `0.01` per site. **Kill unchanged:** a slope `≥ 0.01` at any coupling
kills Fold III as written. **What would make this stake vacuous, said now:** if `M₂^nl` is
found extensive with the SAME density as `M₂` (i.e. the local frame removes nothing on the
vacuum), then Fold III's "area-law" has no quantity to be true of, and branch (b) is entered
on that finding rather than on a slope.

## A3 — the variance gate, declared

A state is a vacuum for this campaign when `q8-mps::variance::energy_variance` reports
**`⟨(H − E)²⟩ / N ≤ 10⁻³`** in the model's energy units — the threshold plant P6 used and
passed. A state above it is refused before magic is read.

## The ladder, re-sized to the exact instrument

Couplings `x ∈ {0.25, 1, 4, 16}`; volumes `N ∈ {8, 12, 16, 24, 32, 48}`; **`χ ≤ 11`**, the
smallest χ passing A3 at each point. P6 already showed `x = 4, N = 16` passes at `χ = 10`
(variance `6.8 × 10⁻⁵` per site) with `M₂ = 5.7004` stable to `6.9 × 10⁻⁴` against `χ = 8`.
Whether `N = 48` passes at `χ ≤ 11` is unknown and is the first thing the ladder reports;
if it does not, the volume range is what it is and S1/S3 are read on the points that pass.

## Cost, re-derived

DMRG for the 24 points at `χ ≤ 11`: minutes each, under an hour. Exact `M₂`: `N = 48,
χ = 11` at `64 χ⁹ ≈ 1.5 × 10¹¹` per site, seconds to a minute per state. `M₂^nl`: three sweeps
of `N` line searches of `~20` evaluations each — `~3,000` `M₂` evaluations per state at `N = 48`,
**an hour per state on one thread, under a day for the ladder on 16.** The prereg's "under
40 core-hours" survives; its "one to two days to build" was three hours of a subagent plus
the corrections here.

## Plants added

| plant | must |
|---|---|
| **P3′** | a product of H-type states rotated by random single-site unitaries: `M₂^nl = 0` to `10⁻⁹`, where `M₂` itself is `0.415 N` — the new minimiser removes what the Clifford one could not |
| **P7** | a GHZ state: `M₂^nl = M₂ = 0` (stabilizer) — no false magic manufactured by the search |
| **P8** | a random MPS at `N = 12, χ = 6`: `M₂^nl ≤ M₂`, the fourth sweep improves the third by under `10⁻⁶`, and two different site orders agree to `10⁻⁶` (the descent is not order-trapped at this size) |
| **P9** | `χ = 12` requested: REFUSED by name with the lease it would need |

### Correction on building (2026-09-19)

*Written on building §A2's instrument (`q8-mps::magic::sre2_nonlocal_min`; plants P3′, P7, P8,
P9 in `tests/gf1_plants.rs`), BEFORE any vacuum ladder is read. The text above is kept as
written. Five things §A2 and its plant rows have wrong, in order of weight; the instrument does
what is right and says so in its module doc.*

**C1 — the re-staked S3 is held fixed by a symmetry, on the very states it is for.** On a real
state of definite parity, `Π_j Z_j|ψ⟩ = ±|ψ⟩`, the identity frame is every site's own minimum
of `M₂` under `R_y`, so the descent §A2 prescribes never moves and `M₂^nl = M₂`. Proof:
`⟨P⟩ ≠ 0` needs `P` to commute with `Π Z`, i.e. an even count of `X`/`Y`; for any rest-string
`P_r`, `X_j P_r` and `Z_j P_r` differ by one in that count, so at most one of `⟨X_j P_r⟩`,
`⟨Z_j P_r⟩` is non-zero, and each site's landscape (C2) is `a + r cos 8θ_j` with phase zero. The
Jordan–Wigner Schwinger vacuum has definite charge `Σ Z_j`, hence definite parity. Measured at
`x = 4, N = 12, χ = 6` (variance `8.3 × 10⁻⁴` per site, under A3): `⟨Π Z⟩ = ±1` to `10⁻⁸`,
`M₂ = M₂^loc = M₂^nl = 3.886893106248`, every angle zero — a test in `gf1_plants.rs` measures
the parity first and then the equality. Six descents seeded from random frames on a
parity-projected random MPS all converge back down toward the identity's value and none below
it; a joint scan of both angles on `cos(π/8)|00⟩ + sin(π/8)|11⟩` finds nothing below the
identity either. So on the ladder `M₂^nl(N) = M₂(N)` at every point, and §A2's "what would make
this stake vacuous" fires by a theorem, not a finding — the prereg's own lesson, a second time.
The real frame is not the limitation. For a real state every string with an odd count of `Y`
reads zero, so with parity every fibre `(⟨X_j P_r⟩, ⟨Y_j P_r⟩, ⟨Z_j P_r⟩)` has at most ONE
non-zero entry — it already lies on an axis — and since `Σ_a (Rv)_a⁴ ≤ |v|⁴` with equality only
on an axis, the identity is each site's minimum over ALL of `SU(2)`, not just over `R_y`.
Jointly the evidence is numerical: on three even-parity real random states at `N = 4`, 3000
random `SU(2)^4` frames and a twelve-angle descent find nothing below the identity (an
unprojected real state at the same size is lowered by `0.37` under `R_y` alone); the tilted GHZ
at `N = 2, 3` likewise. On a real state of definite parity the local frame has nothing to
remove, complex or not: `M₂^nl = M₂` is the state's property, not the instrument's. **S3 as
re-staked asks of the Schwinger vacuum exactly what S1 asks** — the slope of `M₂(N)` itself —
and the amendment's own clause names branch (b) for that finding. The ladder reads `M₂` and
`M₂/N` (S1, S2, S4 stand) and banks `M₂^nl = M₂` as what it is; whether Fold III's area-law
sentence has a quantity left is the lead's verdict on this finding, not on a slope.

**C2 — the line search is exact, not golden-section, and costs 3 evaluations, not ~20.** With
the other sites fixed, `2^{−M₂}(θ_j) = a + r cos(8θ_j − φ)` EXACTLY — each pair
`⟨X_j P_r⟩, ⟨Z_j P_r⟩` turns as `cos, sin (2θ − δ)` and its two fourth powers sum to
`r⁴(¾ + ¼ cos(8θ − 4δ))` — with period `π/4`, since `R_y(π/4) = HZ` is a Clifford (verified: every
harmonic but the eighth is at `10⁻¹⁶` on a random MPS). Three readings fix the sinusoid and its
minimiser is closed-form; the instrument evaluates AT that minimiser and moves only to the
strictly lowest frame evaluated, so every banked value is an exact `M₂`, never a fit. §A2's
golden section on `[0, π)` is a unimodal method on four periods of that sinusoid; kept as the
referee `sre2_nonlocal_min_golden`, on a random MPS at `N = 8, χ = 4` it spends 19 evaluations
per line search to land `1.0 × 10⁻⁵` above the exact search at a `10⁻³` bracket, and 34 to land
`2 × 10⁻⁸` beside it at `10⁻⁶` (exact: 3, total 73 against 457 and 817). The cost is
`1 + 3N · sweeps` exact readings — 433 per three sweeps at `N = 48`, not ~3,000. Angles are
reported in the fundamental domain `[0, π/4)`; §A2's `[0, π)` holds four copies of it.

**C3 — three sweeps do not converge, and two orders do not agree, at P8's size.** On the random
MPS at `N = 12, χ = 6` (seed `0x0008_2424`): `M₂ = 6.350414296`; the forward sweeps read
`5.072057011, 5.058341329, 5.057632708, 5.057402824` — the fourth buys `2.3 × 10⁻⁴`, not
under `10⁻⁶`; the reversed order reads `5.068521597` after three, `1.1 × 10⁻²` from the forward.
Twelve sweeps in each order say why: the coordinate descent converges linearly at a ratio of
`≈ 0.65` per sweep (forward `5.057087819` after twelve, still buying `5 × 10⁻⁶`; reversed
`5.057338454`, buying `1.4 × 10⁻⁴`), and the two geometric tails extrapolate to the same limit
within `10⁻⁵`: not order-trapped, slow. P8 asserts the row's numbers as written and is
`#[ignore]`d with the measured values in its doc comment. "Three sweeps, the minimum over
sweeps" is not a convergence criterion; a reading must bank every sweep's minimum
(`NonlocalMagic::per_sweep`) and stop on a per-sweep gain, not on a count. On the ladder this is
moot by C1: the descent does not move.

**C4 — P3′'s "`M₂` itself is `0.415 N`" holds under random Cliffords, not random unitaries.**
`M₂` of `R_y(α)|H⟩` is `−log₂(1 − ¼ cos² 4α)`: `log₂(4/3) = 0.41503749927884` at `α = 0`, zero
at `α = π/8`. P3′ fires both draws at `N = 8`: random real Cliffords (P3's draw),
`M₂ = 3.320299994 = 8 log₂(4/3)`, the Clifford minimiser the same, `M₂^nl = 0` exactly with
every angle `π/8`; random angles, `M₂ = 2.288154047 = Σ_j m(α_j)` (`1.03` under `0.415 N`),
`M₂^nl = 5 × 10⁻¹⁶`. Only `M₂^nl = 0` is the stake, and it holds because `SO(2)` is transitive
on real single-site states.

**C5 — "A GHZ state's [`M₂^nl`] is not zero: its magic is in the entanglement" contradicts P7's
own row** and the theorem: GHZ is a stabilizer state, `M₂ = 0`, and `M₂^nl ≤ M₂`. P7 fires as
the row says, `1.3 × 10⁻¹⁵` for both at `N = 8`, every angle zero. The state the sentence
reaches for — `cos(π/8)|0…0⟩ + sin(π/8)|1…1⟩`, Schmidt coefficients no local frame changes — is
P7's control: `M₂ = M₂^nl = log₂(4/3)`, and being parity-symmetric it is C1's case too.

**Cost, measured, single thread, release** (`gf1_plants.rs`, the `#[ignore]`d cost tests, on
one pinned core with its hyperthread sibling idle): `N = 12, χ = 6` — one `sre2` `1.55` s,
three sweeps of `M₂^nl` `171` s for `109` evaluations (`1.57` s each, exactly `3` per line
search); `N = 16, χ = 8` — one `sre2` `23.6` s, three sweeps `3121` s for `145` evaluations
(`21.5` s each; `M₂ = 9.985570 → 9.198380, 9.112395, 9.105156`). The same runs with the
sibling core busy read `307` s and `3744` s: the price is bandwidth, and a ladder that runs
sixteen readings on sixteen threads of eight cores will not get sixteen times one core. One exact `M₂` of the vacuum at `x = 4, N = 16, χ = 11`
(variance `2.8 × 10⁻⁶` per site; `M₂ = 5.700346`, P6's `5.7004` again) takes `141` s, `8.8` s
per site — so `N = 48, χ = 11` is `~7` minutes per exact reading, not A1's "seconds to a
minute", and three sweeps of `M₂^nl` there (433 readings) would be `~50` hours per state, not
"an hour". By C1 those hours would buy nothing on the ladder. P9: `χ = 12` is refused by the non-local minimiser with the
same `Price` error as the exact reader, `13.76 GB` against the `8.59 GB` lease, before any
evaluation. The reader `examples/gf1_read.rs` now prints `m2_nl` with its per-sweep record
(`--nl-sweeps`, 0 to skip) and reports a refused secondary reading as JSON null with the refusal
named, so a ladder point at `χ = 11` still banks its `m2` where `sre2_local_min`'s
environment stack (`29.4 GB` at `N = 16, χ = 11`) is refused.
