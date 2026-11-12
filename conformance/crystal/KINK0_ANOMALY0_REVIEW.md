# KINK-0 and ANOMALY-0 — adversarial review of the two drafts, 2026-11-11: neither freezes as written

*A second reader (a reviewer agent, read-only, briefed to attack) went through
`KINK0_PREREG_DRAFT.md` and `ANOMALY0_PREREG_DRAFT.md` on the day they were drafted. This file
is the record of what it found, in the author's condensation; every number below is the
reviewer's, from dense or sparse exact diagonalisation at `N ≤ 12`. Its scratch scripts were
left outside the repository. No draft has been frozen and no campaign has run.*

**Verdicts.** KINK-0: freeze only after named fixes, and at a NEW parameter point.
ANOMALY-0: REDESIGN.

## 0. A contamination, and whose fault it is

The author's brief asked the reviewer to check, at `N = 8, 10, 12`, whether KINK-0's stakes
"would be met" with the draft's own Hamiltonian at the draft's own parameters. Those sizes
are in the draft's ladder. So S2, S3, S4, S5 and P-PINNED of KINK-0 have been SEEN at three
of six sizes at `x = 4`, `m/g ∈ {1, 0.1}`, `α ∈ {0, ½}`, by the reviewer and by the author.
The fault is the brief's. That parameter point cannot be staked blind again: any freeze of
KINK-0 moves to a point not yet computed, with a declared pilot at a third. ANOMALY-0 was
kept clean (operator identities, the free chain, a pilot at `x = 5`, `N = 12`).

## 1. ANOMALY-0: three reasons to redesign

1. **S3 is not a reading of the anomaly.** The `τ = 0⁺` rate `d⟨J_n⟩/dτ = −2 ε_n ⟨K_n⟩` holds
   for ANY nearest-neighbour hopping with a field energy quadratic in `L`: massive fermions,
   any coupling. It is the lattice f-sum rule. By Hellmann–Feynman `Σ⟨K_n⟩ = ∂E₀/∂x`, a
   finite difference of energies SCHWINGER-3 already banked. Its approach to `−2/π` is
   guaranteed as `x → ∞`: the pilot reads `−π⟨K⟩/2 = 0.9920, 0.9983, 1.0013` at
   `x = 5, 10, 20`. The `0.10` bar is slack by an order of magnitude and the kill cannot
   fire (M-VACUOUS-SUCCESS; the shape of FERMION-0 Amendment 1).
2. **S4 was a comparison built to lose.** Before signals from the placed charges arrive, the
   midpoint field oscillates at the INFINITE-volume gap, not the open-chain box gap the draft
   named; SCHWINGER-3's own table has the two `3.8–4.8 %` apart, above the `3 %` bar. The
   repository's two estimates of that infinite-volume gap at `x = 4` differ by `3.3 %`
   themselves. The clean window is under half a period (`τ = 2.5/√x` against a period of
   `5.2/√x`). And one whole unit of field is not linear response on this lattice: the pilot's
   fitted frequency is `3.760` at `ε = 0.05` (the gap is `3.752`) and `3.270` at `ε = 1`,
   `13 %` low.
3. **The instrument cannot take the readings.** `tdvp::expectation` returns a real part and
   the engine's MPOs are real, so `⟨J_n⟩`, which is `i` times a real antisymmetric operator,
   reads exactly `0` from the only routine there is. `E_n K_n` is non-local and needs an
   accumulator channel. G1 ("bond dimension 40 against 64") may compare a run with itself,
   since single-site evolution inherits whatever bonds the ground state had. G0's energy bar
   cannot fail on an integrator that conserves energy to round-off.

**What the review derived and verified** (convention `dO/dτ = i[W, O]`,
`K_n = σ⁺_nσ⁻_{n+1} + h.c.`, `J_n = i(σ⁺_nσ⁻_{n+1} − h.c.)`; checked at `N = 8`, random
background, to `2 × 10⁻¹⁴`):

- `dL_n/dτ = −x J_n` and `dq_n/dτ = x (J_{n−1} − J_n)`. The draft's sign is right; its S1
  omitted the factor `x`.
- `dJ_n/dτ = −{E_n, K_n} + x(σᶻ_n − σᶻ_{n+1}) + x(K⁽²⁾_n − K⁽²⁾_{n−1})`, with `K⁽²⁾` the
  next-neighbour hop with its string. The hopping part does NOT vanish per link and
  telescopes to the ends of a window. `λQ²` plays no part.
- `⟨K⟩ = −2/π` for the half-filled free chain, and that value corresponds to the continuum
  coefficient `1/π`: the draft's correspondence is right. On an open chain `⟨K_n⟩`
  alternates link to link, so any "bulk" reading must average an even number of links.
- Under the continuous TDVP flow, S1 is exact at any bond dimension (single-site operators
  obey Ehrenfest exactly), so it tests the step size and the reader, not the law. S2 involves
  a two-site operator and its residual measures TRUNCATION: a better gauge than G1.

**What an honest reading of the anomaly would be.** That the source is the field times a
number AT ALL TIMES. On the lattice the source is `{E, K}` plus hopping terms, and the stake
is that the remainder shrinks with `x`. The banked numbers already show the naive factorised
relation fails at finite `x`: `(M_∞/g)² = 0.441` at `x = 4` against `−⟨K⟩/2 ≤ 0.318`.

**The split the review proposes.** (i) The operator identity and S1 as an exact-
diagonalisation note, no freeze. (ii) An INSTRUMENT campaign that gates the evolver and a
complex two-site reader on the Schwinger model against the dense propagator at `N ≤ 12`;
nothing real-time is staked before it. (iii) A plasma-frequency campaign (one static charge
at the end, amplitudes 1 and 2, a cosine fit before the ends' signal arrives, compared with
the infinite-volume gap). It should be named for what it is, PLASMA-0, and not called an
anomaly reading. The static `⟨K⟩` should not be a campaign at all.

## 2. KINK-0: what must change

1. **S3 does not tell a kink from no kink.** At `α = ½`, `m/g = 1` the charge's mean position
   is `X/N = 0.630, 0.636, 0.638`: outside the staked `[0.40, 0.60]`, not killed, and no
   branch covers that. At `m/g = 0.1`, where there is no doubled vacuum, the same reading
   gives `0.618, 0.649, 0.673` with nearly the same spread. What does discriminate: the sign
   of `ΔE` (`+3.71` above the transition, `−0.51` below) and the gap inside the charge sector
   (`1.435, 1.062, 0.814`, closing like a level in a box, against `3.00, 2.61, 2.33`).
2. **S4 passes by construction.** Three points and two parameters; the fit predicts the third
   to `0.2 %` against a `5 %` bar, and so does a constant. The kill is unfireable for any
   particle in a box.
3. **Two of three plants are broken.** P-BACKGROUND is false as worded: a unit of background
   entering at one end is a different boundary condition, not a relabelling (the energies
   differ by `6.92`). The exact identity is `E₀(Q; α) = E₀(−Q; α + Q)`, verified to `10⁻¹³`.
   P-PINNED adds a constant if read literally, and the unplanted reading already lies
   outside the band it was meant to leave. The witness `no_lone_charge` does not apply to a
   charged sector.
4. **S2 is true for a nearly trivial reason:** the last site can hold the charge at no field
   cost. Stake its distance from the end (`0.48` sites at every size), not `X/N`.
5. **Confirmed:** `Q = −1` is the degenerate sector at `α = +½`; the mass-sign plant fires;
   the anomaly arithmetic in the draft's note is right (`0.553116² = 0.30594`).
6. **Cleaner statement:** two static opposite charges at separation `d` in the neutral
   sector, energy linear in `d` at `α = 0` and flat at `α = ½` above the transition. That is
   Coleman's own form, it is where `no_lone_charge` applies, and it needs a mass term and a
   non-integer background in the engine's model: the same change ANOMALY-0 wanted.

**Smallest honest KINK-0.** One mass above and one below the transition at a fresh point
(the review suggests `x = 3`, `m/g = 0.8` and `0.15`, after a declared pilot at a third),
`N = 12…18`, staking the end-distance, the in-sector gap's closing, the sign of `ΔE`, and the
two corrected plants. Its sentence: at half a unit of background a unit charge is cheap and
mobile in the bulk; at zero background it is glued to the end.

## 3. Also flagged

- "Hamer and Kogut" as a two-author credit for the spin form is doubtful; the papers the
  reviewer can place are Banks–Susskind–Kogut (1976) and Hamer–Kogut–Crewther–Mazzolini
  (1982). The engine's module headers carry the same phrase and should be checked.
- ANOMALY-0's "Why" overreached: on a staggered chain `Σ J_n` is the vector current, and
  calling it the handed charge is the continuum's identification, not a lattice fact. Its
  branch (a) sentence would have been true of free fermions with no gauge field.
- Order: KINK-0 first (exact diagonalisation, no new instrument), then the instrument gate,
  then PLASMA-0. If the engine's Schwinger model gains a mass term and a non-integer
  background, it gains them once, gated against the Python exact diagonalisation.
