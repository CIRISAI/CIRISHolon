# RESPONSE-1 — AMENDMENT 1: the reader was reading the quadrature the kick does not drive; the cycle-aligned average; the density mode rises before it decays

*Written 2026-09-19, after the 128-water smoke was read end to end and BEFORE any of the
eight 432-water arms (launched 10:46 CDT, `replace0_response1.sh`) is read. Nothing here
changes what the arms record — every readout's positions and velocities are banked — so the
arms run on. This amends the READER (`holon-lens/src/field.rs`, `rung2 --response`) and
§3 of `RESPONSE1_PREREG.md`; it is committed alone before the reader is changed.*

## What the smoke read

Three transverse cycles at 200 m/s on 128 waters, read by the reader as built:

```
cycle 0: REFUSED — kick amplitude 0.00 under 3 x noise 0.00
cycle 1: OVERDAMPED  λ = 7.376e12 /s  A = 0.00 | blind/spatial 0.352
cycle 2: REFUSED
R3: η = 4.569e-4 Pa s  IN BAND        R4: blind/spatial 5.4  FIRES
```

An "in band" viscosity from one cycle at amplitude `0.00`, and a scrambled partition that
carries FIVE times the spatial mode. Both are one error: **§3 defines the current mode as
`j_k = Σ_j v_j cos(k x_j)`, and §2's kick is `Δv_j = v_d (sin(k x_j) − ⟨sin⟩)`. The kick's
projection on the cos quadrature is `v_d Σ sin cos ≈ 0`.** The reader was reading the
quadrature the drive does not touch; the "overdamped" cycle was a fit to thermal noise, and
R4 "fired" because both partitions carried only noise and the ratio of two noises is
whatever it is. A print at two decimals (`0.00`) let this pass one run. The same fault class
as the kick's own "zero by symmetry": a quantity assumed to carry the signal that the
symmetry sends elsewhere.

## A1 — the driven quadrature, and the undriven one as a null

> The **driven current mode** is `j_k^s(t) = Σ_j v_{axis,j} sin(k x_j)`. At the kick it reads
> `v_d Σ_j (sin − ⟨sin⟩) sin = v_d (N/2)(1 + O(1/N))` plus thermal noise. The **density
> response** to a sin velocity mode is in the COS quadrature — `∂_t ρ = −ρ₀ ∂_x u` and
> `∂_x sin = k cos` — so R2's `ρ_k^c = Σ cos(k x_j)` stands.
>
> **The undriven quadrature of each mode is read as a null, R4′:** `j_k^c` on arm T and
> `ρ_k^s` on arm L, cycle-aligned as in A2, must have a kick-time amplitude under `3σ/√C`
> of the aligned noise. **It fires the reader's conviction** (branch (e)), like R4.

R3 and R4 are read on `j_k^s`; R4's ratio is of the aligned amplitudes (A2).

## A2 — the cycle-aligned average is the response; per-cycle fits are the spread

The prereg said "each cycle is an independent realisation of the same response" and then
staked per-cycle fits. The arithmetic per cycle at 432 waters, on the transverse current
mode: signal `v_d N/2`, thermal noise `√(N/2) · √(kT/M)` —

| `v_d` | signal, au | noise per cycle, au | SNR per cycle | SNR over 12 cycles | over 3 seeds |
|---|---|---|---|---|---|
| 50 m/s | `4.9e-3` | `2.5e-3` | **2.0** | 6.9 | 12 |
| 200 m/s | `2.0e-2` | `2.5e-3` | 7.9 | 27 | — |

**At 50 m/s a single cycle is UNDER PR-3's `3σ` bar and will be refused — correctly.** The
response is therefore read on the sign-aligned average over the cycles of a seed,
`ȳ(t) = (1/C) Σ_c s_c · y_c(t)` with `s_c = ±1` the cycle's kick sign, noise `σ/√C`
(estimated from the aligned average's own relaxed tail, the last quarter of the cycle),
**baseline the tail's MEAN** (the reader as built subtracted the last SAMPLE, adding one
noise to every point). The fit of §3 is of `ȳ`. The spread is (i) the per-cycle fits that
pass the bar, when three or more do, else (ii) the leave-one-cycle-out fits of the average.
The seeds' three `ȳ` fits give the second spread, pooled as before.

## A3 — the density mode starts at zero and rises before it decays

The kick puts velocity, not density, into the box. On the L arm `ρ_k^c(0⁺) = 0` and the
linearised equations give `ρ_k^c(t) ∝ e^{−λ₁ t} − e^{−λ₂ t}` when overdamped, with
`λ_{1,2} = Γ ∓ √(Γ² − ω²)`, `Γ = ν_l k²/2`, `ω = c_s k`; the slow rate `λ₁ ≈ ω²/(2Γ) =
c_s²/ν_l` is R2's number (the prereg's, now with the form that carries it stated). With
water's numbers at `L = 44.4` bohr: `λ₂ ≈ 2.6e13` /s (the rise, `~40` fs), `λ₁ ≈ 6e11` /s
(the decay, `~1.7` ps — half a cycle). **R2's fit begins at the aligned average's peak `|ρ̄|`
and reads `λ₁` from there** as the prereg's exponential; the two-exponential form is fitted
instead when the rise is resolved (three or more readouts before the peak) and `λ₂` is
then reported as `ν_l k²` with no band. PR-1's classifier is not applied to the rise.

The arithmetic for the density mode, per cycle at 432 waters: peak signal `(N/2)(k v_d /
λ₂) ≈ 1.1` counts at 50 m/s against noise `√(N S(k)/2) ≈ 3.7` counts — **SNR 0.3 per cycle,
1.0 per seed, 1.8 over three seeds. R2 at 50 m/s is expected to REFUSE or sit at the
edge; the 200 m/s control (SNR 1.2 per cycle, 4 per seed) is where `λ₁` is readable**, and
it is reported as the control's number. R2 carries no band and its refusal kills nothing —
the prereg said so; this states in advance that the refusal is the likely outcome and why.
R1 (the continuity leg) remains the L arm's stake.

## A4 — the print

Amplitudes and noises print in scientific notation. A print that showed `0.00` and an
"in band" verdict beside it hid the quadrature error for one run.

## Plants added, each to fire before an arm is read

| plant | must |
|---|---|
| **PR-6** | thermal velocities on random positions, kicked by §2's rule at `v_d` with `v_d N/2 = 4σ`: the sin reader returns the kick amplitude to `2 %`; the cos reader returns under `3σ` at the kick |
| **PR-7** | twelve synthetic cycles of alternating sign at SNR `2` each (an exponential at `λ`): the aligned-average fit reads `λ` to `5 %` and at least nine of the twelve per-cycle fits are REFUSED |
| **PR-8** | a synthetic `e^{−λ₁t} − e^{−λ₂t}` with `λ₂ = 40 λ₁` and noise at a tenth of the peak: `λ₁` to `5 %` from the peak; and the same series is NOT classified underdamped |

## Stakes

R1, R1′, R3, R4 and the branches are unchanged. R4′ is added to branch (e). R2's
expected refusal at 50 m/s is declared. No number in §5 moves.

### Correction on building (2026-09-19, same day, before any arm is read)

A2 said "baseline the tail's MEAN". Built that way and run on the smoke, the aligned kick
amplitude read `8.3e-3` au against the direct projection's `5.8e-3` — because the smoke's
600-fs cycles at 128 waters do NOT relax: every tail sits at `−0.3` of its kick (11–18
standard errors from zero), a rebound of the opposite sign, and subtracting it inflated
the amplitude by 40 %. **The baseline is the physical zero** — both modes have zero mean at
equilibrium (uniform density; zero total momentum with velocities uncorrelated with
positions) — **and each cycle's tail mean is REPORTED in units of its standard error as the
relaxation check**: a tail beyond 3 SE means the cycle has not returned by its end, and the
reader says so on every line. With that, the smoke reads: aligned amplitude `6.3e-3` au
(SNR 12.7 over three cycles), R4 `0.022`, R4′ at noise, per-cycle `Γ_s` from `4.9e12` to
`1.2e13` /s — the chain is proven and the smoke is not a reading (128 waters, unrelaxed).

The rebound itself is a finding to expect: at `k = 0.40 Å⁻¹` (128 waters) and `0.27 Å⁻¹`
(432) water's transverse current is at the onset of propagating shear modes (MD literature
puts it near `0.3 Å⁻¹`). **If the aligned mode on the 432-water arms classifies UNDERDAMPED,
R3 reports `ρΓ/k²` as the k-dependent viscosity with the band applied as staked and `ω/k`
as the shear-wave speed with no band; the classification is the finding**, and the
hydrodynamic `η` would need the larger box §0 already named. The 3.14-ps cycles of the arms
hold three to six periods of such a wave, so the tail check will say whether it relaxed.
