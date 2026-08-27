# Q10 PREREG — the three-legged certificate at scale, and the refusal it exists to make

**STATUS (2026-08-26): §0's HARD PREREQUISITE IS DISCHARGED — see the amendment at the foot of
§0. The branch resolved to the FIRST named mechanism, loss of canonical form, and the regression
that convicted the engine is gone. This file is unblocked for freeze; §1's scope may now be
fixed. The status line below is kept as written for the record.**

**STATUS: DRAFT AGAINST A BRANCH POINT. NOT FROZEN.** The hard prerequisite in §0 is
unresolved, and §1's scope is a function of how it resolves. Nothing in this file is staked
until §0 is discharged and the file is re-committed as FROZEN. Published now for attack, per
freeze-design-early.

**Author: research-manager-2. Commissioned by team-lead through chief-of-staff-2, 2026-08-24.**

---

## 0. THE HARD PREREQUISITE, AND THE BRANCH

**Q10 cannot be pre-registered on an engine that is *proved* not to report its own manifold's
optimum.** `Core/Stagnation.lean`'s `not_optimal_of_regression` convicts the `chi=256` run at
N=8, U=16 on **two** witnesses — `chi=16` (`δE = 3.411e-5`, converged) and `chi=32`
(`δE = 8.595e-7`, converged) both beat it, and every MPS manifold embeds in every larger one.
The conviction is of the RUN, not of any mechanism. Three mechanisms remain live and none is
supported: rank-deficiency / loss of canonical form, initial-state trapping, and plain
non-convergence.

**The discriminator, named and not yet run:** same configuration, same `chi`, **different
initial state**, through `dmrg::run_from` (which already takes caller-supplied tensors).
Trapping moves or vanishes; conditioning persists. **It must not route through `pad_to_chi`** —
zero-padding manufactures the rank-deficient bonds the conditioning branch blames, and would
confound the contrast it is meant to settle. Team-lead's conditioning instruments
(`‖A†A − I‖` per bond, `⟨ψ|ψ⟩`, per-bond smallest singular value) characterise whichever
branch survives; the initial-state arm is what separates them.

**THE BRANCH, staked now so neither outcome is a surprise:**

- **BRANCH A — the defect is confirmed and fixed.** Q10 proceeds at its full scope (§1A).
- **BRANCH B — canonical form is NOT the cause.** **Q10's scope shrinks to the validated `chi`
  range, and this prereg says so in those words.** Not a smaller claim dressed as the same
  claim: the deliverable is explicitly bounded, and the bound is reported in the headline.

**VALIDATED CHI RANGE — a derived ruler, not a chosen number.** The maximal contiguous range,
starting from the smallest `chi` on the ladder, over which `E(chi)` is non-increasing. This is
`not_optimal_of_regression` used as an acceptance test rather than as a post-hoc conviction:
the first rung that regresses ends the range, and everything above it is out of scope by
theorem rather than by preference. **It is computed before any Q10 gate runs.**

**EMPTY-RANGE CLAUSE.** If the *smallest* `chi` on the ladder already regresses against a smaller
one, the validated range is **empty** and every gate downstream of it is undefined. That is not a
Q10 result: it means §0's prerequisite was not discharged after all, and **Q10 STOPS and returns
to the branch point.** Stated because a ruler with no valid readings is a failure mode a ruler
cannot report about itself.

---

### §0 AMENDMENT (2026-08-26) — DISCHARGED, on the first of the three mechanisms

`not_optimal_of_regression` convicted the χ=256 run at N=8 U=16 because χ=16 and χ=32 both beat
it, and every MPS manifold embeds in every larger one. Three mechanisms were named as live:
rank-deficiency / loss of canonical form, initial-state trapping, plain non-convergence.

**It was the first.** `4bcf0d2` repaired the two-site Jacobi SVD, which judged column
orthogonality on an ABSOLUTE Gram cross-term and treated a non-improving sweep as *converged*.
The full χ ladder at N=8 U=16, re-measured on the repaired engine against q-seam's
`E0 = -1.262136132334283`:

| χ | energy | δE = E − E₀ | beats χ=256? |
|---|---|---|---|
| 16 | −1.262102024104522 | 3.4108e−05 | no |
| 32 | −1.262135297768552 | 8.3457e−07 | no |
| 64 | −1.262136132207331 | 1.2695e−10 | no |
| 256 | −1.262136132335044 | −7.6095e−13 | — |

Variational ordering, lowest first: **256, 64, 32, 16.** Correct, and monotone in χ. **The
regression is gone and the conviction does not attach to a fresh run.**

**The mechanism confirms itself, which is why this is a discharge and not merely a moved number.**
χ=16's δE is `3.4108e−5` against §0's recorded `3.411e−5` — *the small-χ readings were always
right.* Only large χ was corrupted. That is exactly what a defect requiring a wide Schmidt spread
predicts: truncation at small χ kept the retained spectrum inside the range where an absolute
tolerance was accidentally adequate. A repair that merely loosened something would have moved all
four rungs.

**The named discriminator is therefore not needed to decide §0.** "Same configuration, same χ,
different initial state, through `dmrg::run_from`, not routed through `pad_to_chi`" was staked to
separate trapping from conditioning. Conditioning was found directly and repaired, so the branch
is resolved by mechanism rather than by discriminator. The discriminator remains available and
unrun; **it is not claimed here, and trapping is not positively excluded** — it is simply no
longer needed to explain a regression that no longer exists.

**One anomaly, reported rather than smoothed.** χ=256 sits `7.6e−13` BELOW the q-seam reference.
A variational method cannot go below the true ground state, so this is the reference's own
precision showing — q-seam's Door residual is `1.26e−13` — and the two disagree at the 1e−12
level. It bounds how finely this ladder can be read, and it is not evidence of anything else.

**Supporting evidence:** `output/q8_mps/canonicality_repair_receipt.md`, the eight-configuration
re-adjudication in `Q8_MPS_RESULTS.md` (0 of 8 VOID, sweep kill does not fire), and
`crates/q8-mps/tests/canonical_sweep.rs` (failure-first: `defect=1.7995e−5`, FAILED before).

---

## 1. SCOPE

**1A (Branch A).** `N ~ 100`, or the largest `N` at which the *fixed* engine validates against
`q-seam` at `N ≤ 12` — whichever is smaller. The validation ladder is unchanged from Q8: the
exact reference exists only where `q-seam` reaches, and correctness-against-truth extends only
as far as truth extends.

**1B (Branch B).** The same `N`, restricted to the validated `chi` range of §0. If that range
excludes the `chi` needed to represent the state at `N ~ 100`, **Q10's honest deliverable is
the refusal itself** — see §2.

> **AND HERE IS WHAT WOULD MAKE BRANCH B A FAILURE, named because otherwise every outcome reads
> as a success and that is the shape this whole discipline exists to prevent.** **If the refusal
> fires everywhere — including at configurations where `q-seam` demonstrably validates the engine
> at `N ≤ 12` — then the refusal is useless and BRANCH B HAS FAILED.** A machine that refuses
> unconditionally is `Q_SEAM_RESULTS.md`'s M2 mutant, which was rejected for exactly this. The
> deliverable is a refusal that *discriminates*, and Branch B is judged on discrimination, not on
> having produced a refusal.

**Family.** The 1D Hubbard chain of `Q_SEAM_PREREG.md` §1, half filling, `Sz = 0`, `t = 1`,
open boundaries. **Same family as Q8 deliberately**: this campaign is about the certificate at
scale, not about a new physics family, and changing both at once would make a null
uninterpretable.

---

## 2. THE PRODUCT CLAIM, STAKED IN THESE WORDS

> **The engine refuses to quote when motion is uninformative.**

That is the deliverable. Not "the engine is accurate at N~100" — which it cannot demonstrate,
because no exact reference exists there — and not "the engine quotes its own error", which
**`Core/Stagnation.lean`'s `error_not_computable_from_motion` forbids**: distance-to-truth does
not factor through the process's own motion ledger.

**A certificate of MOTION is not a certificate of ERROR.** Q10 may report where the engine
declines to answer and why; it may not convert a motion or truncation quantity into an error
bar. **G4's retraction (`Q8_MPS_PREREG.md` A4.2) is the measured face of that theorem** and is
cited, not re-derived.

---

## 3. THE THREE LEGS

### 3a. THE FENCE — a wrongness meter computable without a reference

`Q_SEAM_RESULTS.md`'s lesson: `D_bool`, the chart's own cap-not-Boolean fence, tracked chart
failure where the beyond-pair share did not. Q10's analogue must be **chart-internal**, since
at `N ~ 100` there is nothing to compare against.

**STAKED: the fence is the SPECTRUM FLOOR** — the smallest kept singular value relative to the
largest, minimised over bonds: `floor = min_b (s_min(b) / s_max(b))`. A bond whose kept spectrum
still reaches down to near-zero has budget to spare; a bond whose smallest *kept* value is
comparable to its largest has none. **It is not an error estimate and is never reported as one**
(§2); it is the chart's declaration of how close it is to its own limit.

**Threshold: STAKED at `floor ≥ 1e-3` ⇒ the fence is UP.** Chosen, not derived, and declared as
chosen.

> **Why the floor and not bond entropy (ruled 2026-08-24, team-lead; the argument is coherence,
> not robustness).** An earlier draft staked `S_max / ln(chi)` and asserted it "→ 1 as a bond
> exhausts the budget." **That assertion was false**: `S_vN → ln(chi)` requires a *flat* Schmidt
> spectrum, and real MPS spectra decay fast, so a fully saturated bond can read well below any
> ratio threshold — the fence would have read "fine" everywhere for a reason unrelated to the
> chart's health. **The deciding argument is not that the floor is more robust but that it is
> COHERENT with §2 and §5**: the floor is a *production-ledger* quantity, which `Core/Habit.lean`
> licenses reporting as production, whereas an entropy ratio drifts toward exactly the error-bar
> reading this prereg forbids. The entropy ratio is **kept as a reported diagnostic, never as the
> gate.**

### 3b. THEOREM-PINNED ANCHORS — DERIVED FOR THE FAMILY, NEVER ASSUMED

Four, each with its warrant named and each holding at every `N` including where no reference
exists:

| anchor | value | warrant |
|---|---|---|
| particle–hole | `⟨n_jσ⟩ = 1/2` per spin-orbital | `Q_SEAM_PREREG.md` §1.1(iii) — **per spin-orbital, not per site**; the per-site total is 1 |
| magnetization | `m_i = 0` | spin-independence of `H` + ground-state uniqueness in the `Sz = 0` sector (`Q7_SEAM_PREREG.md` §2.2) — **not Lieb**, whose heavier hypotheses this family does not need |
| sector lock | `⟨Ŝz⟩ = 0` **and** `⟨(Ŝz)²⟩ = 0` | A1's pin, and it **earned its keep on first contact**: at N=8, U=16 `⟨Ŝz⟩ = 3.2e-3` would have read as noise while `⟨Ŝz²⟩ = 1.9e-2` convicted, 99.95% of it variance |
| particle number | `⟨N̂⟩ = N` | **conserved by `H`; only APPROXIMATELY realized by an unconstrained variational state** — see the correction below. Drift is unshifted with the **integer** `N_target` (§2 of Q8), never the measured value |

> **The `⟨N̂⟩` warrant is corrected against a measurement already in our own logs.** An earlier
> draft called it exact "by construction". It is not: D1 builds no charge-blocked tensor, so
> nothing in the ansatz enforces particle number, and `output/q8_mps/full_grid_gates.log` records
> `N=8 U=16: |N_tot-8| = 6.9706384007162114e-6` — **over its own 1e-6 band**. "By construction"
> for an unconstrained ansatz is a claim contradicted by our own data, which is why the anchor is
> *gated* rather than assumed: the realization is approximate and the gate is what says so.

**RECORDED COMMITMENT, not a proof, and labelled as such per the house rule on `True`-field
records: if Q10's family is ever changed, every anchor is RE-DERIVED for the new family before
use.** An anchor carried across a family change without re-derivation is an assumption wearing a
theorem's clothes. **Nothing in this file enforces this** — it is a commitment on whoever changes
the family, and if a mechanism is wanted it must be built, not asserted.

### 3c. THE MOTION CERTIFICATE

Two clauses, both refusals:

1. **Across `chi`** — `E(chi)` non-monotone ⇒ **REFUSE**, as the instance of
   `not_optimal_of_regression`. The witness point is realized by the warm start (§4), which is
   what makes the manifold inclusion concrete rather than abstract.
2. **Within a run** — `E(sweep)` non-monotone beyond slack ⇒ **REFUSE**. Q8's G3-primary, as a
   refusal rather than a gate.

---

## 4. WARM START IS SUBSPACE EXPANSION, AND IT IS ONE MECHANISM

**Zero-padding is banned as the growth rule.** A zero-padded bond is rank-deficient by
construction and, on the leading hypothesis, is the disease in a syringe. Growth is by
**subspace expansion** (Hubig, McCulloch, Schollwöck, Wolf — density-matrix / environment-drawn
expansion): new bond directions come from `H` applied to the current state, projected
orthogonally to what is already there, so they are non-null and physically relevant.

**Warm-start and perturbation are ONE mechanism, not two arms** — the expansion IS where the
perturbation enters. **Mutation-tested by removal:** an arm with expansion disabled must be
shown to behave *worse*, or the mechanism is not what is doing the work.

---

## 5. WHAT MAY BE REPORTED, AND AS WHAT

**Discarded weight is a PRODUCTION ledger, never an error.** `Core/Habit.lean`: a chart whose
induced rate map is injective has zero production (`production_eq_zero_iff_rate_injective`,
`production_id_eq_zero_of_injective`), and unitary evolution is injective — so a truncating
quantum simulation's production lives **entirely** in its truncation. Reporting accumulated
discarded weight as *production* is exact and honest. Reporting it as *error* is forbidden by
§2's theorem and was measured false by G4.

---

## 6. GATES

| gate | statement | STAKED threshold |
|---|---|---|
| **H0** validated-`chi` ruler | `E(chi)` non-increasing over the reported range | exact, by theorem |
| **H1** correctness vs `q-seam` | at every `N ≤ 12` in range: energy, density, double-occupancy | Q8's G2 bands, unchanged |
| **H2** anchors | all four of §3b, at **every** `N` including `N ~ 100` | `≤ 1e-6` each |
| **H3** fence separates | the fence must take both values across the sweep | **≥ 1/4 of the grid up AND ≥ 1/4 down** — a FRACTION, because §7 sizes the grid from a cost probe that runs after this file, and an absolute count is demanding on 8 configs and trivial on 40 |
| **H4** refusal fires and discriminates | mutation-tested: `Typed` refuses, `Silent` does not, same numerics | joint, both halves |
| **H5** speed | total wall-clock | §7 |

---

## 7. SPEED IS A STAKED GATE

**TWO thresholds, and the absolute one is why this gate can fail at all.**

1. **STAKED: total wall-clock ≤ 12 HOURS, absolutely.** Calibration, from our own record: Q8's
   superseded run spent **4h39m on seven configurations at `N ≤ 10`** and never reached
   `N=10, U=16`. A grid that cannot fit inside 12 h is a grid that must be made smaller, and this
   ceiling is what forces that decision *before* the run rather than mid-run — which is how
   Amendment 2 happened.
2. **STAKED: total wall-clock ≤ 4× the cost probe's extrapolation.**

> **Clause 2 alone would be a gate that cannot fail**, because the probe sets its own target — the
> same defect as a certificate that quotes its own error. Clause 1 is independent of anything the
> campaign predicts about itself, and it is the one that binds.

The probe is run **before** the grid is fixed, and the grid is sized from it — not the reverse.

**Scheduling is serialized by default** (Q8 Amendment 2's binding rule), and the environment is
probed with one timing under load before any launch. **The two-solve rule applies to every
staked contrast, compute cost included: both arms in ONE run, in ONE environment.** This is not
a formality — a load-average-23 measurement of this campaign's own gate was disowned on
2026-08-24 for exactly this defect, by its author.

---

## 8. THE KILLS — separable, each taking down its own claim

> **K1 CORRECTNESS.** Fires if H1 fails at any validated `N`. Fatal to the deliverable; no
> `N ~ 100` number may be reported. STOP.

> **K2 FENCE.** Fires if H3 fails — the fence reads the same everywhere, or never moves. Kills
> the fence leg only: the anchors and the refusal are untouched. **A fence that never fences is
> decoration**, and this is the clause that says so in advance.

> **K3a ANCHOR UNSOUND.** Fires if any anchor of §3b fails at a configuration where the exact
> reference says the state is right. The anchors are theorem-pinned, so this convicts either the
> derivation or the implementation, and the results file must say which.

> **K3b ANCHOR INERT.** Fires if no anchor fires anywhere across the sweep. An anchor that cannot
> fail on this family certifies nothing here — `Q7_SEAM_RESULTS.md`'s D1b was reported UNTESTED
> rather than null for exactly this reason, and only became a measurement when a family finally
> exercised it.

> *(K3a and K3b were ONE kill in the first draft, which violated the separability rule three
> sections above it. Unsoundness and inertness are different failures with different remedies and
> they get different falsifiers.)*

> **K4 REFUSAL.** Fires if H4's joint condition fails. Kills the refusal feature only; energies
> may still be reported, flagged that undetected non-optimality is unmitigated.

> **K5 SPEED.** Fires if H5 fails. A scheduling and sizing result, not a correctness one.

None implies another. All are reported in the results file's title line, as loudly as a
survival.

---

## 9. TWO-SOLVE PROBES REQUIRED BEFORE FREEZE

Every staked contrast gets a two-point probe **before** this file is frozen. Non-negotiable,
and the reason is Q7: *"the physics varies along this axis" does NOT imply "the error varies
along it."* Q7 VOIDed at its family gate because nobody measured the spread first.

1. **The fence must be shown to VARY** across the intended sweep — H3's precondition, measured,
   not hoped. If it does not vary, the family does not pose the question and Q10 is
   **VOID-not-killed**.
2. **The anchors must be shown to be violable** — at least one configuration where an anchor
   fires. An anchor that cannot fail on this family proves nothing here.
3. **Cost**: both arms of every timed contrast, one run, one environment (§7).
4. **The §0 discriminator** — different initial state, not through `pad_to_chi`.

---

## 10. CARRIED FORWARD

**M1–M6 ARE DROPPED, and the reason is recorded rather than quietly fixed.** The first draft said
"misfit fixes M1–M6 carry over from Q9's brief unchanged." **There is no Q9 file in this
repository** — `find . -iname "*Q9*"` returns nothing, and the only document in the tree that
mentions M1–M6 is *this one*. It was not a citation to something I had not read; it was **a
citation to a document that was never written**, and message-only content is not record. There is
nothing to carry them over *from*, so they are dropped. If six such fixes exist, they must be
enumerated here in Q10's own words before this file freezes.

*(Third instance today of one shape — T1's "axial P2 / SR+curvature slice" shorthand, the closing
certificate, and now M1–M6: **a cross-reference is a warrant only if its target exists.**)*

**M1–M6, AUTHORED HERE — not recovered, and labelled that way so nobody later reads them as the
originals.** The originals are unrecoverable; these six are written from what this campaign
actually measured on 2026-08-24, each with the incident that produced it, and they bind Q10:

- **M1 — every filter, floor or exclusion is DECLARED in this prereg before it exists in code.**
  Cause: G4's `FLOOR = 1e-14` lived only in `g4_certificate.rs:24` and removed two of five ladder
  rungs from a staked fit. Its provenance was exculpatory (written nine hours before any data) and
  the non-declaration was still a defect.
- **M2 — both arms of every timed contrast run in ONE run and ONE environment.** Cause: a
  release-vs-debug gate timing taken at load 23 and at an earlier load, disowned by its author
  before it hardened into a CI requirement — the same defect this campaign had just caught in an
  unrelated warm-start probe.
- **M3 — a gate is SELF-VERIFYING, never self-describing.** Cause: `ci-gates.sh` gate 9 ran green
  while covering 118 of 165 tests and zero of `fracture::`/`impact::`, with its own comment
  claiming those modules. A gate's scope must be asserted by the gate, not by prose beside it.
- **M4 — a VOID configuration's readings are DIAGNOSTIC, never gate data.** Cause: N=8 U=16 failed
  G2 by six orders while VOID under G7; §7 names G2 in the correctness kill, and the loud reading
  was one sentence from being written down.
- **M5 — a check that cannot distinguish two causes is a DETECTOR: name the discriminator and run
  it before concluding.** Cause: `/proc/<pid>/exe → (deleted)`, sound at launch time and
  false-positive mid-run; and, in this campaign's own physics, a damage non-monotonicity whose
  discriminator (same finest spacing, different frontier extent) was run rather than guessed.
- **M6 — a cross-reference is a warrant only if its target exists**, and message-only content is
  not record. Cause: this very section.

Q8's Amendment 2 scheduling rule,
Amendment 3's VOID-configs-are-diagnostic ruling, and Amendment 4's declaration discipline for
in-code filters all bind here. **Every filter, floor, or exclusion applied to any Q10 fit is
declared IN THIS FILE before it is written in code** — A4's defect, not repeated.

## 11. CREDITS

DMRG — White 1992. MPS formulation and sweep/SVD machinery — Schollwöck 2011. Subspace
expansion — Hubig, McCulloch, Schollwöck & Wolf. Area laws — Hastings 2007. Jordan–Wigner 1928.
The Hubbard exact solution — Lieb & Wu 1968. **This programme's own**: the ledger reading of
bond dimension; the refusal discipline of `GrainFloor.lean` applied to a new resource; the
motion certificate as an instance of `not_optimal_of_regression`; and the production reading of
discarded weight via `Core/Habit.lean`.

## 12. FILES AND DETACHED COMPUTE

Prereg: this file. Crate: `sim_engine/crates/q8-mps` (extended, not forked). Every run over the
house-rule wall-clock line gets `setsid` + done-marker + `RESUME.md`, and **every detached
launch asserts its binary's provenance at launch time** (`/proc/<pid>/exe` against the build's
mtime — a launch-time assertion, which is where that check is sound; mid-run it is a detector,
not a verdict).
