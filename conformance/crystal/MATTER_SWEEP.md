# MATTER_SWEEP — from a fermion that is not an input to an asymmetry that is not put in by hand: the ladder PRICED before any freeze

*2026-11-23. A pricing note in the form of `LEPTON_LADDER.md`: it carries no claim, runs no
compute, freezes nothing, and types no number from outside the record except the exact
integers and identities it credits. It was asked for by the lead as "the
quantum/fermion/baryogenesis sweep". The record has no document of that name and had not
used the word baryogenesis before today. The lane is `conformance/crystal/`: FERMION-0
(read), KINK-0 (read), ANOMALY-0 (sent back as a redesign). This note says what the next
rungs would cost, what each would earn, and what convicts each. Everything physical in it is
textbook or prior art and is credited as such. Where a credit is from the author's memory and
was not checked at source today, it is marked (memory).*

## 0. The question, and the one thing that must be said first

Why there is matter and no equal amount of antimatter needs three things at once (Sakharov,
1967): a count that is not conserved, a violation of the symmetries that would make the
process and its mirror equally likely, and a departure from equilibrium. In the Standard
Model the count is baryon number, it fails to be conserved through an anomaly, and it is
anomalous only because the weak force is CHIRAL.

**No rung of this ladder predicts the observed asymmetry** (about six baryons per ten
billion photons). Every carrier this engine has is vector-like, in one space dimension, at
sizes below the registry's own floor for a continuum statement (`M-VOLUME-SCALE`). What the
ladder can do is put each of the three conditions on a carrier where it is exact or nearly
so, read which view carries what, and say where the wall is. §6 says what would have to
exist for a rung to bear on the real number.

On a vector-like carrier the only anomalous count is the HANDED one. So the most this
carrier can make is a handed asymmetry: the analogue of the chiral asymmetry in front of a
bubble wall, which in the electroweak story is then converted to baryons by a second,
chiral, process. The conversion is behind the wall of `LEPTON_LADDER.md` and is rung V.

## 1. What is banked, at the strength it carries

| | record | what it gives the sweep |
|---|---|---|
| a fermion is not an input | `FERMION0_RESULTS.md`: on the toric code a fermion is a closed view of a vacuum that holds none; one bit per bound pair, on the relationship. A re-finding (Kitaev; Levin and Wen) | the label "fermion" follows from the parts; nothing about a lepton |
| a charge is its field's step, and is free only where the vacuum allows | `KINK0_RESULTS.md`: massive Schwinger model, `x = 3`, `m/g = 0.8` and `0.15`, `N = 12…18`, exact diagonalisation. A re-finding of Coleman (1976), read as a CONFIRMATION and not blind | the instrument `kink0/kink0.py`: mass, a background on every link, charged sectors, gated against a Kronecker referee to `8.5 × 10⁻¹⁴` |
| no charge comes alone; the vacuum can pair | `SceneCharge.lean`: `no_lone_charge`, `vacuum_can_pair` | the section law for a count that can cancel |
| a count changes by what crosses its faces plus a declared source | `SceneRhythm.lean`: `continuity_coarse`, `source_is_the_defect`; `engine/crates/holon-scene/src/rhythm.rs`: `check_continuity`, `implied_source` | the checker a sourced count is read with. It DEFINES the source as the defect. It says nothing about the source's form |
| the anomaly draft was not an anomaly reading | `KINK0_ANOMALY0_REVIEW.md` §1 | three exact operator identities (below), and the order: instrument first |
| the vector gap | `SCHWINGER3_RESULTS.md`: `M_∞/g = 0.664, 0.627, 0.609` at `x = 4, 9, 16`; continuum `0.5531` against `1/√π` | a referee for the frequency of rung I, at three couplings |

The review's identities, with `dO/dτ = i[W, O]`, `K_n` the hop and `J_n` the current on link
`n`, verified there at `N = 8` to `2 × 10⁻¹⁴`:

- `dL_n/dτ = −x J_n`. The field on a link changes by the current through it. Exact.
- `dJ_n/dτ = −{E_n, K_n} + x(σᶻ_n − σᶻ_{n+1}) + x(K⁽²⁾_n − K⁽²⁾_{n−1})`. The current's source
  is the field times the hop, plus terms that are not the field.

## 2. The symmetry that has to break is C, and half a unit does not break it

In one space dimension the background field (the theta term, `α = θ/2π`) is odd under charge
conjugation and odd under reflection, so it is EVEN under the two together. The condition
that in three dimensions is "CP violation" is, on this carrier, C violation (Coleman 1976;
stated in these words by Carena, Li, Ou and Singh, arXiv 2412.10365).

`α = ½` is the C-SYMMETRIC point: C sends `α` to `−α`, and `−½` is `+½` up to one unit. What
KINK-0 read there is C broken SPONTANEOUSLY above a critical mass, two vacua and a kink
between them. It is not an explicit violation and it cannot bias a process. An explicit
C-odd reading needs `α` away from both `0` and `½`. The sweep's rung II is therefore not a
re-reading of KINK-0; it is a new background on the same instrument.

## 3. The ladder

`P` is the Python exact-diagonalisation track (`kink0/kink0.py`'s lineage: sparse sector
Hamiltonians, `N ≤ 18` on the record; real time by a Krylov exponential of the same matrix;
no cargo build). `E` is the engine track (`engine/crates/q8-mps/src/schwinger.rs`,
`tdvp.rs`). Cargo builds are barred on the shared box today, so every `E` item below waits;
nothing here was built.

| rung | Sakharov's condition | carrier and instrument | owed BEFORE a freeze | referee | the kill |
|---|---|---|---|---|---|
| **0 INSTRUMENT** | none | `P`: `kink0.py` plus a complex reader for `J_n` and `{E_n, K_n}`, and a time step. `E`: the same readers on the evolver | `P`: three functions, gated on the identities of §1 at `N ≤ 12` against the dense propagator. `E` (cargo): `tdvp::expectation` returns a real part and the MPOs are real, so `⟨J⟩` reads exactly `0`; a complex two-site reader; a mass term and a non-integer background in `schwinger.rs`, added ONCE and gated against `P`; two-site evolution, since single-site cannot grow a bond | the dense propagator; the identities | an identity missed beyond round-off. Not a campaign: a gate, as the review ruled |
| **I ANOMALY** | a count that is not conserved | massless chain, `α` stepped by a small amount at `τ = 0`; read `E(τ)` at the centre and `Σ J` over an even window | rung 0 on `P`. A fresh coupling: the review saw `x = 5, 10, 20` at `N = 12` | `SCHWINGER3_RESULTS.md`'s `M_∞(x)` for the frequency; the free chain's `⟨K⟩ = −2/π` for the coefficient | the remainder of the source after `{E, K}` NOT shrinking as `x` grows; or the frequency off the banked gap by more than the two banked estimates differ (`3.3 %` at `x = 4`) |
| **II C-ODD** | the bias | `kink0.py` as it stands, `α` on a grid away from `0` and `½`, two or more masses; read the bulk field of the neutral vacuum | none on `P` beyond a fresh point and a declared pilot at another | the reflection `α → −α` (exact, already a plant: `E₀(Q; α) = E₀(−Q; α + Q)`); the mass-perturbation law of §4 | the bulk field not odd in `α`; or not vanishing with the mass |
| **III OUT OF EQUILIBRIUM** | the departure | `P`: `α(τ)` ramped from `0` to a non-symmetric `α_f` over a time `T`, mass on; read the centre cut's transferred charge and the late field against the equilibrium field of rung II | rung 0 with a time-dependent background (piecewise-constant steps) | the adiabatic limit (rung II's value) and the sudden limit (rung I's oscillation) | the two limits not recovered; the section law violated at any step (cannot fail on `P`: a gate) |
| **IV THE COMPOSITION** | all three | the same ramp, read as a handed count with its ledger: which view is closed, the source, the price, the remainder (§5) | rungs I–III read; an adversarial review that is told NOT to compute the staked point | none exists for the staked point; plants carry it | §5 |
| **V THE CONVERSION** | an anomalous count that is not handed | a CHIRAL gauge theory in one dimension (the 3-4-5-0 model) | a route through the wall, priced in §6. No instrument exists | — | not posable today |

**Order and dependencies.** 0 → II (needs nothing from 0 but the fresh point) → I → III → IV.
II can be drafted first: it is exact diagonalisation on a banked instrument. V waits on a
route. The `E` track is needed only for sizes beyond exact diagonalisation, which is where a
continuum sentence would need it (`N ≳ 20√x`); no rung above stakes one.

**Plants, by rung.** I: the Coulomb term switched off (the engine's own `Mutation::CoulombOff`;
the oscillation must vanish and the current must not). II: the mass sign reversed (banked,
fires at `3.03`); the background reflected. III: the ramp run backwards from the final
state, which on `P` must return the start to round-off; a ramp at `m = 0`, where the late
field must not depend on `α_f`. IV: a ramp between the two C-symmetric points `0 → ½`, where
the C-odd part of every reading must vanish.

## 4. What each rung earns, said before it is read

- **Rung I is a re-finding** of Schwinger (1962) and of the lattice anomaly (Banks, Kogut
  and Susskind 1976; Ambjørn, Greensite and Peterson 1983). The review already named its
  honest core: PLASMA-0, an oscillation at the gap. It also has the one sentence in this
  sweep that is a derivation in this programme's own terms, and the reader should decide
  what it is worth. If the two-variable view (field, current) on a link is CLOSED, then
  with `dL/dτ = −xJ` exact the current's rate must be a number times the field; the
  identity of §1 gives that number as the hop's expectation; the free half-filled band
  gives `−2/π`; and the result is a photon of mass `g/√π`. So: closure of that view, two
  exact identities and one band integral give the Schwinger mass. That is the textbook
  argument (the anomaly equation with Gauss's law) in other words. It is NOT blind, and
  the closure is an assumption that holds only for massless fermions in the continuum. The
  banked numbers show it open at finite coupling: `(M_∞/g)² = 0.441` at `x = 4` against at
  most `0.318` from the hop alone. What the rung measures is that defect and how it closes.
  The single-mode bound (Feynman; Bijl) is the same statement and is credited (memory).
- **Rung II is a re-finding** of Coleman (1976) and Coleman, Jackiw and Susskind (1975):
  with massless fermions the background is screened and nothing depends on it; with a mass
  the bulk field is, to first order, proportional to the mass times `sin 2πα`, with a
  coefficient fixed by the massless model's condensate (Adam 1997, mass perturbation
  theory; memory, and the coefficient is deliberately not typed here). KINK-0 already shows
  the trend at the symmetric point: the centre field `0.473` heavy, `0.260` light.
- **Rung III is a re-finding in kind.** A changing theta biasing an anomalous count is
  spontaneous baryogenesis (Cohen and Kaplan 1987; memory). Quenches of theta in this model
  have a literature (Zache et al. 2019; real-time string breaking, Hebenstreit and Berges
  2013, Pichler et al. 2016; memory, none read today).
- **Rung IV** is the only rung with room, and §5 is honest about how much.
- **Against the bar** ("a forced derivation of a nontrivial piece of known physics"): rung
  I's sentence is a derivation of known physics from the closure condition, but it was
  known first and the route is the standard one. Rung IV can be a number written before it
  is read. Neither is both at once. The author's judgement: rung IV is the first that could
  meet the bar, and only in the form of §5(c).

## 5. Rung IV: what the composition is, read as this record reads things

**(a) Two kinds of asymmetry, and the section law tells them apart.** The electric charge
is conserved exactly, so its asymmetry is only ever a SEPARATION: what the left of a cut
gains the right loses (`no_lone_charge`), and the amount is the step of the field through
the cut. On `P` that holds to round-off at every time; it is a gate and cannot be a stake.
The handed count is SOURCED: `implied_source` returns its defect, and the anomaly is the
statement that the defect is the field times a fixed number. The section law does not
derive that. It is the anomaly equation, restated, and this note says so.

**(b) The closed view.** In the continuum the handed count alone is not conserved, but the
handed count together with the gauge field's winding is: the sum of the two does not
change. That pair is the closed view, and the winding is the ledger entry that pays for the
asymmetry. This is standard (the conserved, gauge-variant current) and is credited to the
anomaly's authors. On this lattice the winding is the time integral of the field, which
`dL/dτ = −xJ` ties to the current exactly. So the closed view exists exactly on the lattice
too, and what is NOT exact is that its handed half is a count of anything: on a staggered
chain `Σ J` is the vector current, and calling it handed is the continuum's identification
(the review, §3). The remainder to price is the part of the current's source that is not
the field: the mass term and the hopping terms of §1.

**(c) The candidate stake, as sharply as the record allows.** For a linear ramp of the
background by a small `α_f` over a time `T`, in the massless limit, the field left ringing
at the centre has amplitude `α_f · |sin(MT/2) / (MT/2)|`, with `M` the gap: the ramp's
Fourier weight at the one frequency the closed view has, and no free parameter. With a
mass the view opens, and the stake is on the DEFECT: the late-time field minus rung II's
equilibrium value, as a fraction of `α_f`, at a mass and a ramp time chosen where the
adiabatic and the sudden pictures disagree. It could come out otherwise in three ways the
record already shows are live: the gap at finite `x` is not the continuum's; one whole
unit of field is not linear response here (the review's fitted frequency fell `13 %` at
`ε = 1`); and the ends send signals into the centre after about half a period.

**(d) What makes it blind, and what does not.** The massless formula is the driven
oscillator and is not blind. A defect at a staked `(x, m/g, α_f, T)` is blind only if nobody
on the record computes that point or a grid that brackets it. KINK-0 lost its blindness
exactly so: the author asked a reviewer where the transition sits, and the answer needed a
bracketing grid. For this rung the reviewer checks identities, limits and plants, at a
coupling that is not the campaign's, and is told in the brief not to evaluate the stake.
The pilot is declared and sits at a third point.

**(e) The kill.** The defect inside its bar at the staked point but the plants silent; or
the bar met only because the mass term's contribution was below the reader's resolution
(`M-EMPTY-SECTOR`, `M-VACUOUS-SUCCESS`); or the reading at `0 → ½` not C-even.

## 6. Rung V and beyond: the wall, and what has moved at it this year

A baryon-like asymmetry needs a count that is anomalous and NOT handed, and that needs a
chiral gauge theory. The smallest is the 3-4-5-0 model in one dimension. `LEPTON_LADDER.md`
prices six routes; the field has since added two, read today at abstract level only:

| route | what it does | what it costs here | source and status |
|---|---|---|---|
| **R7 mirror gapping** (symmetric mass generation) | interactions gap the mirror fermions without breaking the symmetry, leaving a chiral spectrum | a six-fermion interaction on a ladder or a non-local hop; density-matrix renormalisation at sizes the `E` track does not have. Zakharov et al. use a "tangent-fermion" lattice whose hopping is non-local: of R5's kind, and to be put to the object contract's locality budget before it is priced | Zakharov, Ueda, Verstraete and Beenakker, arXiv 2606.24713 (Phys. Rev. Research 8, 043009): a gap without a degenerate ground state. Chi et al., arXiv 2610.42103: quantised charge pumping across all three boundary phases of a four-leg ladder, under the word leptogenesis (pumped value not read here). AGAINST: Golterman and Shamir, arXiv 2505.20436 and 2603.15985: under stated conditions the massless spectrum must be vector-like |
| **R8 symmetry disentanglers** | a constant-depth circuit makes a not-on-site chiral symmetry on-site, so it can be gauged by ordinary means | rotor models, not the engine's qubit lanes; an exactly solvable 3-4-5-0 Hamiltonian is claimed | Thorngren, Preskill and Fidkowski, arXiv 2601.04304. A preprint; not read beyond its abstract |

Beyond one dimension, nothing on this engine reaches: electroweak sphalerons, a
first-order transition with walls, three generations. The work that does reach is
classical-field or ensemble-fermion lattice simulation (Saffin and Tranberg, arXiv
1111.7136, reproduce the anomaly in a one-family Standard Model; Kitajima et al., arXiv
2610.09257, cold baryogenesis in three dimensions with an effective CP-violating operator)
and proposals for quantum hardware (Huang et al., arXiv 2510.08910, a sphaleron in the
O(3) model). For a rung here to bear on the observed number there would have to exist: a
chiral gauge theory on a lattice in three dimensions that survives Golterman and Shamir's
constraint, an evolver for it, and the Standard Model's own CP violation as an input. None
of the three is priced in this note.

## 7. Prior art, and how much room it leaves

- **Closest to rung IV:** Carena, Li, Ou and Singh, arXiv 2412.10365 (reported as published
  in Phys. Rev. D in 2026; the journal reference was not confirmed today). Real-time
  tensor-network scattering of a fermion off a wall with a complex mass in one dimension, a
  C asymmetry in the charge density, two observables for it. No gauge field and no anomaly
  appear in its abstract; the paper was not read beyond that. Their numbers are not read
  here. What it leaves: the anomaly, the background field and the ramp on one carrier, and
  the reading by closed view, price and remainder. What it takes: any claim to the first
  real-time asymmetry on a one-dimensional lattice.
- **The theta-quench and string-breaking literature** (memory, §4) covers rungs I to III in
  kind. A screen at source is owed before any of them is drafted.
- **Bauer et al., arXiv 2611.00214:** a roadmap of toy models from string breaking to
  hadronisation. It bears on GF2's order, not on this sweep.
- **Esposito et al., arXiv 2601.15842:** enforcing a `ℤ_l` gauge constraint costs no magic.
  `GANTT2.md` says it is to be read before GF2. It has still not been read beyond its
  abstract, and it bears on `GF1_RESULTS.md`'s price.
- **A company's claim (2026-10-26) of existence for four-dimensional gauge theories with
  fermions:** unverified, existence only, not read at source. The dynamics is an input of
  this programme either way.

## 8. Misfits this sweep contacts

`M-VOLUME-SCALE` (no continuum sentence at `N ≤ 18`); `M-VACUOUS-SUCCESS` (the anomaly
draft's S3; any gate that cannot fail is named a gate); `M-EMPTY-SECTOR` (a mass
contribution below resolution); `M-PROBE-EIGENSTATE` (a uniform background on a uniform
state); `M-FIXED-POINT-TRAJECTORY` (a ramp that leaves the state where it was);
`M-TRUNCATION-AS-ERRORBAR` (the `E` track's bond dimension); `M-TAG-AS-PROPERTY` (calling
`Σ J` the handed charge); `M-BAR-FROM-THE-READ` and `M-UNTESTED-GAP` (rung IV's bar);
`M-PLANT-SECTOR` (plants in charged sectors); `M-FOREIGN-DOMAIN-CORROBORATION` (a
one-dimensional result read as evidence about the early universe).

## 9. Open to the reviewer

1. **Rung IV may be PLASMA-0 with a ramp.** If the defect of §5(c) is small wherever the
   instrument can resolve it, the rung reads a driven oscillator and earns nothing.
2. **"Handed" is the continuum's word.** Nothing on a staggered open chain at `N ≤ 18` is a
   handed count. The sweep may be about a current and a field and nothing more.
3. **The Sakharov mapping is an analogy at every rung.** One dimension, one flavour, a
   vector-like theory, a closed system with no expansion and no bath: the asymmetry made
   here rings and does not freeze in. A persistent one needs the mass and a transition, or
   a second species to hold it.
4. **Rung I's derivation is the textbook one.** A reader may fairly say it is the anomaly
   plus Gauss's law, with the word closure laid over it.
5. **Blindness is fragile here.** Every quantity is smooth in its parameters; a pilot, a
   referee and a review each bracket the point a little. Rung IV's brief has to be written
   by someone who has read KINK-0's §3.
6. **The memory credits** (Cohen and Kaplan; Zache et al.; Adam; the single-mode bound;
   the critical mass) were not checked at source. No stake may lean on one until it is.
7. **R7 and R8 were read at abstract level only**, and the locality question put to the
   tangent-fermion lattice is the author's reading, not the paper's.

## 10. What this note does not do

It does not claim an anomaly, a handedness, an asymmetry or a mass; it chooses no parameter
point; it types no cost. When a rung is frozen, its freeze cites this note, and this note is
not edited to agree with the result.
