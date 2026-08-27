# THE PROGRAM — three paths from 2026-08-23

Pure dependencies, no dates. Common trunk first; the three paths run in parallel after it.
House rules bind throughout: pre-register, separable kills, mutation-test every gate,
misfit → cash into the metaphysics, everything back into the Lean.

## Trunk (now): one clean main
T1. Frozen agents commit — **items defined by artifact, 2026-08-24.** The original shorthand
    ("E1 fracture; axial P2; SR+curvature slice") was defined NOWHERE in the repository, so
    "N-a done" was never checkable against a written definition: a rung marked green against
    prose. Naming the artifacts is the fix; a claim whose warrant is prose is not yet a claim.
    - **E1 fracture** → `ciris-sim-core/src/fracture.rs`, `fracture::` 9 tests.
    - **axial P2** → `ciris-sim-core/src/homogenization.rs`, `homogenization::` 4 tests.
      **NARROW FORM ONLY**, in the module's own words: it "closes the narrow form of P2
      without inventing another calibration constant" (`homogenization.rs:3`); `:145` and
      `holon-sandbox/src/scene.rs:315` both hedge that a number outside the positive-softening
      branch "would be papering over P2". **A narrow-form closure is not a closure**, and the
      hedge belongs on this spine, not only in the source.
      **NOT AN INDEPENDENT WITNESS**: this is the same work as N-b's ν = 0.24 wiring below
      ("merge homogenization.rs → demo realizes DEMO_CALIBRATION's real ν"). One witness, not
      two — the shared-lemma over-grade, caught here on the spine rather than in a campaign.
    - **SR+curvature slice** → `ciris-sim-core/src/{relativity,curvature}.rs`, `relativity::` 12
      and `curvature::` 7 tests. **Mapping RECONSTRUCTED 2026-08-24, not original**: the phrase
      is defined nowhere, so this is the best available reading of which artifacts it meant, not
      a verified correspondence. Struck or corrected by anyone who knows the original intent.
    All counts inside the 165/165 `--features alloc --release` run at `0d1a59c`.
T2. Merge the four branches: research/dm-gauge-vacuum (repaired, verified),
    research/factorization-kills (repaired, verified), feat/holon-closure-repairs
    (112/112), TEMP-revert already on main. Resolve lib.rs unions.
T3. Adopt FactorizationKill's REPAIR formally: whole-only is FACTORIZATION-RELATIVE —
    one REG_GAPS line; the frame (M15/M20) already says it, the kill makes it mandatory.

## PATH K — fastest kill (independent items, all parallel)
K1. **CLOSED 2026-08-23 — KILLED** (`929ad8a`, on main). Route → gauge identification dies
    on its own pre-registered criterion: the decomposition H(φ) = (U + U†) + (e^{iφ}U² + h.c.)
    is exact but worthless as evidence, because the truncated ladder generates the full 3×3
    algebra. A route-side diagonal gauge move that changes no observable relocates the phase
    out of the flux-change-2 corner, and the Gauss-generated link gauge flow can follow only
    if the Wilson phase is trivial — so the grading, the entire gauge content of the proposed
    map, is pinned to the hand-selected R1 representative. Survivors, exactly: link charge
    conjugation = route time reversal with exact amplitude pairings; and no similarity carries
    the nilpotent ladder onto the cyclic shift. 29 theorems, sorry-free, standard axioms.
    **Pre-commitment honoured: shared-fundamental-dynamics drops below 15%.**
K2. **CLOSED 2026-08-23 — KILLED** (`scratchpad/h3ere2_eval/RESULTS_K2.md`, tracked).
    VERDICT: NOT SUPPORTED, within its pre-committed scope. C over B = 0.531 (26W/23L of 49
    decisive), paired sign test p = 0.775 — rules out a large effect, cannot exclude a modest
    one (<0.58). Soft encoding did not change the verdict, so the categorical bottleneck was
    not the limiting factor. Scope, sealed: this falsifies THIS PIPELINE's use of the engine
    for response generation, not the engine, taxonomy, or classifier.
    **The secondary result is larger than the primary and is a product-direction finding:
    the scaffold does not help, and it costs ~40% more tokens (78 vs 56) and ~40% more
    wall time to not help.** Primary judge: C worse than the bare 0.6B at win rate 0.303,
    p = 0.0019 (gold 0.362, p = 0.0295).
    **LENGTH-CONTROLLED RESTATEMENT, 2026-08-24 — the number above is not the quotable
    one.** A second judge (llama3.1:8b, the one of four candidates passing both calibration
    gates) *reversed* this in sign at high significance. The reversal is a **LENGTH
    ARTIFACT**, established with the protocol's own §5 logistic guard (`choice ~ length_diff
    + arm`) — which `analyze.py` had never implemented, so it had never been run for ANY
    judge, the primary included. Under it: **no arm effect survives length for the second
    judge** on any of four comparisons (arm p = 0.110/0.394/0.306/0.162), while the
    **primary survives clean** (arm β = −0.907, p = 0.00045 soft; −0.716, p = 0.0020 gold).
    Model-free confirmation: C is ~40% longer than A by construction, and under the primary
    **C loses in BOTH length strata and loses HARDER when it is shorter** (0.167 vs 0.317)
    — the confound runs in C's favour and C still loses.
    **THE QUOTABLE LINE IS THEREFORE: "C is worse, or at best no better, and costs ~40%
    more tokens and wall time" — NOT the unqualified 0.303.** Controlling for length the
    primary says WORSE and the second judge says INDISTINGUISHABLE; **neither says better.**
    Per this file's own cross-path coupling, that feeds the CIRIS UI direction regardless
    of sign. **CAVEAT owed on one existing line:** RESULTS_K2.md's gold side-by-side
    C-vs-B (0.608, p = 0.161) IS length-confounded (length p = 0.0396, arm p = 0.098) — no
    verdict changes, it was already non-significant, but the line may not be quoted clean.
    **CALIBRATION 3 — the length gate, implemented and GATING (`3406079`). The second judge
    would now be REFUSED ADMISSION by our own machine, and THE RECORD STANDS.** It was
    admitted under the gates that existed, produced a reversal, and that reversal was
    diagnosed as length-driven by the protocol's own §5 guard; a third gate — staked from
    that diagnosis and confirmed by forward prediction — would now refuse it at admission.
    **The single-judge caveat is partially discharged: not by concurrence, but because the
    reason there is one judge is now measured TWICE, INDEPENDENTLY** — the §5 guard on real
    pairs (no arm effect survives length, 4/4) and Calibration 3 on constructed pairs
    (0.783, p = 4.6e-08), **neither borrowing the other's evidence.**
    **WHAT MAKES THIS EVIDENCE RATHER THAN A STORY:** the amendment staked ADMIT/REJECT from
    the real-pair marginals (0.466 vs 0.595) **BEFORE the instrument existed**, and the
    instrument agreed on **constructed pairs consuming no real pair** — primary 0.505
    (p = 1) ADMIT, second judge 0.783 (p = 4.6e-08) REJECT. **A confirmed advance
    prediction, this programme's highest evidence grade**, and what makes the gate
    non-fitted in a way no prose could.
    **THE RULE, because this will recur: A GATE BUILT FROM A FINDING DOES NOT RETROACTIVELY
    INVALIDATE THE FINDING THAT PRODUCED IT.** Refusing the second judge after the fact
    would be tightening a bar after seeing a disliked result, run backwards — the same move
    whichever direction it runs. **And the deeper reason is a Record-axis fact this
    programme already owns: the warrant for the improvement LIVES IN THE DATA YOU WOULD BE
    DELETING.** Calibration 3 exists *because* llama3.1's reversal was measured; delete the
    reversal and the gate's own origin becomes unprovable from the record. That is
    `repairable_does_not_factor` in the campaign coordinate — what can still be established
    depends on what survives — **and it is why a record is a HISTORY, not a
    current-best-state.** The second judge was valid under the rules in force, its
    measurement is what taught us the gate was missing, and it is neither withdrawn nor
    described as wrong.
    OWED, not blocking: the verdict rests on ONE judge — the secondary (qwen3:14b) was
    DISQUALIFIED on both gates (identical-pair slot-1 = 1.000; sensitivity 0.870 against a
    0.90 bar) and judged no real pairs, so the protocol's split-verdict provision never ran.
    A second qualified judge is **firming a fired kill, not re-running one**, and is picked
    up after Q9 closes.
    **REPRODUCIBILITY, recorded here because a reader of this spine sees CLOSED-KILLED and
    reasonably infers a reproducible result. THE KILL'S NUMBERS STAND** — they rest on
    committed judgment artifacts (`scratchpad/h3ere2_eval/judge_*.jsonl`, `RESULTS_K2.md`),
    unaffected by any of the below. **The RESPONSES ARE REGENERABLE — repaired and PROVEN,
    `db6b4b7`, 2026-08-24.** The build check owed with `9f95754` was first discharged with a
    NEGATIVE (`bin/generate` failed to compile: (A) nothing declared `pub mod chat;` though
    `chat.rs` was committed, and (B) `Session::generate` did not exist — `native.rs` carried
    only a private, llguidance JSON-grammar-constrained `complete()`, structurally unable to
    emit prose, while (B)'s original had been reconstructed from a shipped binary's embedded
    fragments, never committed, and was gone from the worktree). **Cause (B) proved
    RECOVERABLE, and the rebuild is PROVEN identical rather than plausible:** the original
    `Qwen3-0.6B-Q4_K_M.gguf` survives and sampling is greedy hence deterministic, so the
    first 5 items of `encoded_soft92.jsonl` were regenerated and diffed against
    `responses_soft92.jsonl` — **60/60 records byte-identical across all three arms and all
    ten scramble draws, matching response text AND path AND gen_tokens, zero mismatches.**
    Rerunnable: `verify_repro.py` + `repro_soft5.jsonl`, both committed. This restores the
    ABILITY to regenerate; it does not re-run K2 and does not touch the verdict, which
    stands on the judgment artifacts. **The standard this had to meet, and did:** a
    plausible reconstruction that silently differed from the generator which produced these
    responses would have been worse than the loss, because it would look like regenerability
    without being it — so faithfulness was demonstrated by byte-diff, not asserted.
    Side effect worth recording: `chat.rs` carried **3 tests that had never been compiled**
    because nothing declared the module — dead tests that read as live. `ciris-nl`'s default
    count goes 3 → 6.
K3. **Modular locality at 2–3 plaquettes.** quantum_link + the Jacobi path at growing
    size; kill = modular locality fails to persist beyond one link. Their own 30% line.

## PATH N — all of Newton (serial where shown)
N-a. Land frozen work [= T1].
N-b. (k_n, k_t) wiring: merge homogenization.rs → demo realizes DEMO_CALIBRATION's real
     ν = 0.24 (the stencil's ν = 1/3 Cauchy restriction is the measured reason scalar
     stiffness was never enough) → THEN P3 knob rerun → P4 specimen pin.
N-c. MEET-2: T4 rigid chart ↔ T5 Newtonian chart over the same holon, Rapier as the
     limiting-case control. Needs N-a (curvature slice) + E2 (done).
N-d. **N1, the aggregation theorem** — fine REG+ evolution bounded by a
     boundary-supported residual. locality.rs's z^d/d! bound is the executable shape;
     the Lean theorem is the deepest single item on the board and the certificate's
     mathematical warrant.
N-e. Full fracture composition: E1 adaptive crack-tip + node-node contact + derived
     (k_n, k_t) laws, certified end to end (B4 regimes B/C).
     **GREEN, with FOUR independent checks — and one claim explicitly NOT among them.**
     Verified `4262553` (165/165, prize gate 3.91 mm ≤ 4.27 required) and `5471bea`
     (190/190). The four checks are the first things here that are not the engine checking
     itself: two LOOSE — the momentum window (J = 6.609 inside [3.6, 7.2] N·s) and the
     energy inequality — and two SHARP — an **exact** Griffith bound and a **measured**
     fracture threshold (between 2.50 and 4.00 m/s) with a sub-threshold elastic anchor
     (zero crack area, impulse 1.8594 inside [1.000, 2.000] N·s, restitution 0.859).
     The Griffith bound accumulates the engine's own per-bond `law.fracture_energy_j`
     rather than estimating from `crack_area`: **fracture work 0.6737 J against KE lost
     4.8836 J.** The area estimate reads 0.5917 J, and **that 14% gap IS the quenched
     per-bond roughness** — a measured mechanism, NOT a tolerance the anchor tolerates.
     **THE MULTI-RESOLUTION CLAIM IS REFUTED IN THE NORMAL CHANNEL — 2026-08-24, `5fee581`.
     NOT "unverified". MEASURED FALSE.** The earlier statement here — refinement-stable,
     not verified — is RETRACTED and replaced, at the place it was made.
     **INSTRUMENT: the uniform-strain patch test (Irons & Razzaque 1972)** at a refinement
     interface — impose the exact continuum solution `u(x) = ε·x` and require every INTERIOR
     node's net force to vanish. **The reference is the continuum, not the engine**, which is
     what makes it not `SelfAudit`.
     ```
     uniform interior (n=616)  median 2.220e-10   max 1.320e-9     <- CONTROL PASSES
     INTERFACE        (n= 78)  median 8.149e-2    max 5.159e-1
     ```
     **THE SIGNATURE IS THE DIAGNOSIS, not the size.** Residuals are normalised by the
     largest bond force, linear in strain, so round-off reads `~1/strain` and a consistency
     error reads FLAT. Over a 100× strain drop: **uniform floor ×100.0 (round-off);
     INTERFACE 8.1490e-2 → 8.1490e-2, ×1.0000 — FLAT TO FOUR DIGITS.** A CONSISTENCY ERROR
     that does not shrink with strain. **The `width_fraction = interface_m / pitch` share
     rule — a stated design principle, never a theorem — does not put a hanging-node
     interface in equilibrium under uniform strain.**
     **SCOPE, and it must not be widened. TESTED: the normal channel only.** NOT TESTED: the
     tangential channel — it is rate-integrated (`tangential_disp += slip_speed * dt`), so a
     STATIC patch test sees zero shear and is structurally blind to it. **And the normal
     channel alone is the central-force lattice with the Cauchy restriction that `(k_n, k_t)`
     exists to repair — so this refutes the half that was never supposed to be the problem,
     which makes it worse rather than better.**
     **WHAT IT DOES NOT SAY:** that certified answers are wrong. It says the discrete
     operator is **locally inconsistent at refinement interfaces at ~8% of a bond force**,
     and that `certified END TO END` over a multi-resolution frontier **has no
     elastic-consistency warrant.**
     **THE WARRANT NARROWS, and this is the operative sentence: "certified end to end" is
     warranted by MEASUREMENT against external references, NOT by an elastic-consistency
     argument.** The discrete operator is locally inconsistent at refinement interfaces at
     ~8% of a bond force, so **refinement safety cannot be argued from theory — only
     observed per run.** Substance survives; warrant fails.
     **DID THE ANCHOR RUNS CROSS REFINEMENT INTERFACES? YES — checked, not assumed.** The
     external anchors are emitted inside `lac_du_bonnet_certifies_where_the_constructor_
     refuses()`, the adaptive certified run (289 materializations, 284 near / 0 far). **So
     their agreement with external references happened DESPITE the interface inconsistency**
     — which is newly interesting rather than reassuring: the local defect does not
     propagate into these integrated observables, and that is a finding, not a silence.
     **THE ANCHORS PER ITEM, corrected by the lane that built them — two of the integrator's
     first arguments were wrong and are replaced:**
     - **Impulse window — SAFE, structurally.** `force[a] += n*axial; force[b] -= n*axial`:
       equal and opposite **by construction**, so no operator inconsistency can move total
       momentum.
     - **Griffith — the TERMS are safe, the SUM is not.** Each term is a material property,
       so no term is a solver output — but **which bonds are in the sum is decided by forces
       that are wrong at interfaces**, so the value 0.6737 J is touchable exactly as the
       threshold is. **What is untouchable is the INEQUALITY**: both sides come from the same
       run, so `fracture work ≤ KE lost` remains a valid statement about that trajectory
       whatever the operator does.
     - **The energy margin is NOT an apples-to-apples comparison and the numeric form is
       WITHDRAWN.** 13.8%-of-KE-lost and 8%-of-a-bond-force are different quantities in
       different units; the first does not bound the second. The inequality is safe for the
       same-run reason above, **not** because 13.8 exceeds 8.
     - **Threshold — VALUE touchable; the zero/nonzero transition is robust HERE BY MARGIN,
       NOT BY STRUCTURE.** At 2.50 m/s nothing broke anywhere and fracture work was exactly
       0.0 — not a marginal miss an 8% force error could flip. **On a scene sitting close to
       the threshold that robustness would not follow.**
     **THE FRAMING THAT SURVIVES ALL FOUR:** each anchor is a statement about the composed
     observables of a **returned trajectory**, and that trajectory is a real trajectory of
     the discrete system. **What the refutation weakened is not the anchors but the claim
     that the checked object approximates the continuum at its interfaces.** The anchors
     still check what they always checked; we now know more about what the thing they check
     is.
     **NEWTON STAYS CLOSED under its own criterion** — the criterion was never *"the
     multi-resolution claim holds"* — **but the close now carries a fired kill inside it, and
     this line says so rather than letting a closed Newton hide it.**
     Recorded here because a reader meeting "N-e green" beside four fresh anchors would
     otherwise reasonably infer it settled.
     Measured and kept, not deleted: damage is **NOT monotone in the drive** (6 m/s
     exceeds 9 m/s on both crack area and fracture work). Discriminator run rather than a
     mechanism invented — all rungs certify at the same finest 0.00391 m, differing only
     in frontier EXTENT (289/277/264/228), so damage feeds extent feeds quench feeds
     damage and a 5% inversion is quench variability, not physics.
**NEWTON IS CLOSED — 2026-08-24**, in the only form the world currently allows, which is
the form this line always specified: `N-a…N-e green, wild pins DECLARED on the closing
certificate`.
**READ THIS BEFORE THE CENSUS BELOW — "closed with three pins owed" invites a misreading
and the document, not a person, should answer it. THE THREE `OwedNoSource` PINS ARE NOT
MISSING PARTS OF NEWTON. They are missing parts of the WORLD, and declaring them is
precisely what the close criterion ASKS FOR** — a certificate that could not say "no
fracture-grade feldspar potential exists anywhere" would be hiding the thing it exists to
expose. The criterion is MET, and that is a different sentence from "the Newtonian engine
is finished."
**WHAT IS GENUINELY OPEN, three items, each with an owner:**
1. **N-e's MULTI-RESOLUTION CLAIM — RESOLVED 2026-08-24, and the resolution is a FIRED
   KILL, not a verification.** REFUTED in the normal channel by the uniform-strain patch
   test (`5fee581`); see N-e above for numbers, signature and scope. The shear half remains
   **unverifiable statically** — a rate-integrated channel is invisible to a static test —
   and that half is a **REG_GAPS pin, deliberately NOT on the closing certificate**: its
   discharging event is internal (reformulate the channel configurationally, or demonstrate
   path-independence over a quasi-static protocol), so it is a PROJECT debt, not one of the
   world's absences, and an `OwedInternal` variant would invite every internal to-do onto
   Newton's closure artifact. **The certificate's three `OwedNoSource` entries keep meaning
   exactly one thing.** The assignment asked for a verification or a precise statement of
   why one cannot exist; it returned **both, plus a refutation** — the outcome nobody listed.
2. **§3.4's BYERLEE INVERSION — owner: team-lead**, adjudication against the sources. Stated
   below.
3. **THE ENERGY-CREATION FINDING — owner: holon-mesh-2, top priority, and the most serious
   open item on this board.** The D-ledger's first run says the sandbox scene CREATES
   energy, and the one-sided gate could never have seen it (`49245f8`).
   **WARRANT CORRECTED 2026-08-24 — the first one cited was a CONFLATION and is withdrawn.**
   It said `Core/Habit.lean` proves a stable explicit step injective and therefore
   producing nothing, so energy creation is unpredicted by the theory. **Injectivity is
   about INFORMATION, not energy** — production is the log-degree of the rate map, and an
   injective map can create energy freely (scale every velocity by 1.01: injective, and it
   manufactures joules). **`Habit.lean` says NOTHING about this finding.** That is this
   programme's own one-ledger-per-quantity rule — dissipated energy and produced entropy
   are two quantities — broken hours after it was adopted.
   **THE CORRECT WARRANT, on which the finding stands undiminished:** at the SANDBOX tier
   the chart is flat, Newtonian, fixed volume, fixed particle number, so energy conservation
   **is a theorem of the continuum equations via time-translation symmetry (Noether)**, and
   a secular gain is a defect — integrator, unaccounted declared channel, or bug. **Until it
   is answered, "the Newtonian engine is correct" carries an asterisk**, and an
   accounting-gap verdict requires the argument, not the conclusion.
   **AND A DESIGN CONSTRAINT THE LEDGER DOES NOT YET CARRY: ENERGY CONSERVATION IS
   CHART-RELATIVE, and this engine has charts where it genuinely fails.** In GR there is no
   global time-translation symmetry and so no globally conserved energy; in an expanding
   universe total vacuum energy grows with volume — **predicted and observed**. The COSMIC
   tier carries an expansion background, i.e. exactly that case, so a balance gate flagging
   non-conservation there would enforce a law the chart does not have and **fire as a false
   positive on correct physics**. Requirement: the D-ledger's balance gate must be
   **CHART-RELATIVE** — it holds where the chart has time-translation symmetry (flat /
   Newtonian, static Φ) and must **DECLARE ITS INAPPLICABILITY** rather than pass or fail
   where the chart does not. Whether a STATIC CURVED chart counts must be stated with its
   argument: a static metric has a timelike Killing vector, so conserved energy exists
   there — a real discriminator rather than a hedge. **This is the refusal discipline
   applied to a conservation law: a gate that cannot hold should REFUSE, not report.** N-d landed at `546ba51` (`Core/Aggregation.lean`, the aggregation warrant
with its residual); the certificate at `48b93f1` (`ciris-sim-core/src/closing.rs`, 16 tests,
CI-enforced at `.github/workflows/verify.yml:45`).
**WHAT "DECLARED" COST, on this line so nobody has to go looking: 0 Measured · 1 Published ·
1 Stipulated · 3 OwedNoSource.**
- **feldspar potential — OWED.** 0.70 of the rock's mass (afs 0.35 + plag 0.25 + mica 0.10)
  has no fracture-grade potential anywhere; only quartz's 0.30 is covered. Its falsifier
  **discriminates fracture-grade from elastic — a Brillouin/RUS tensor does NOT lift it.**
- **Charles law — OWED.** Seven decades ungated (1e0–1e7 /s), the demo's own impact inside
  the gap. The BRIDGE is absent; the GAP is already in code (`sim.rs:99 STRAIN_RATE_GAP`).
- **grain-boundary data — OWED.** Zero measured interface records; the cohesive network's
  parameters are a continuum back-derivation.
- **compressive mode — PUBLISHED.** 200 ± 22 MPa, Martin & Chandler 1994, band [178, 222].
- **damping/restitution — STIPULATED.** ζ = 2.0e-3 at Q = 250. Stipulated rather than
  Published **because nobody published that value for this rock.**
These are the WORLD's absences, not the lane's: a count-based standard was withdrawn as the
wrong instrument — **WHY a pin is owed is the measurement**, and a pin whose falsifier names
the exact event that would discharge it is populated in the sense that matters. Structural
result, enforced by biconditional: **for an owed pin the falsifier and the unlock are THE
SAME EVENT** — an absence claim dies exactly when its source lands, by construction.
OPEN against §3.4, adjudication owned by team-lead, NOT encoded as any pin's kill: the
closed-crack criterion `R = σc/σt` brackets [12.5, 18.7], and the PINNED specimen violates
it (LAC_DU_BONNET 200/6.9 = 29.0, μ = 1.16; Brazilian 22.7, μ = 0.98) while
`DEMO_CALIBRATION` — the preset §3.4 says is NOT granite — sits INSIDE at 15.8, μ = 0.74.
Game/DX wiring only after.

## PATH Q — quantum sim benchmark (Q1→Q2 serial; Q3, Q4 parallel; Q5→Q6 serial)
Q1. **DONE 2026-08-23** (T2_DFT_REFERENCE.md, 617e1f2 lineage): structure +0.95%/+0.88%;
    elastic tensor 10–14% soft, SIGNED (volume mechanism), shape sub-percent. The
    few-percent-absolute class is REFUTED and the gate consumes its own bias.
Q2. B2q proper, RESHAPED by Q1's verdict: the reference cannot serve absolute C_IJ at
    ±2%, so the gate is SHAPE (sub-percent, where the reference is good) plus
    bias-consumed stiffness bands — pre-stated, signed, never symmetric.
Q3. Quantum-link scaling: exact diagonalization 1→N plaquettes; gates = Gauss exactness
    (machine), modular-locality persistence, boundary-channel entropy vs region size;
    99.9/99.9/2x against the one-plaquette closed form where it exists.
Q4. **The chemistry blind protocol** — from elemental/electronic identity +
    stoichiometric inventory ONLY, recover withheld H2O/CO2/CH4/NH3 topology; the
    fail-closed validator enforces that bond graphs never parameterize dynamics.
    Their 40% line, and the single largest probability swing on the board.

### The Q-seam (Q5–Q6, commissioned 2026-08-23) — the unique-tools play
The wrong game is out-DFT-ing DFT (Q1 measured why). The right game: **the certified
meet between a quantum reference and a classical chart** — match where certified,
REFUSE where correlation makes the chart lie. Nobody else ships refusals.
Q5. **The seam certificate.** An exact quantum reference we own (Q3's machinery: small
    exactly-diagonalizable family, e.g. Hubbard/transverse-field sweep at N small
    enough for machine-checked residuals in the vacuum-tier class, ~1e-16) + the
    Boolean-occupancy/mean-field chart over the same system (`Core/ModeChart.lean` is
    the Lean home: the cap survives mixing, Booleanity is exact only for determinate
    states). Certificate criterion staked in the prereg BEFORE the instrument; the
    chart must match the exact reference inside certified regions at staked tolerance
    and refuse outside them. GATE IS MUTATION-TESTED: a planted correlated state (the
    U-sweep's far end) must be REFUSED; a certificate that never refuses proves
    nothing. Speed clause explicitly SCOPED OUT: Q5 is a correctness product.
    KILL (separable): if no staked criterion separates certified from refused better
    than certify-everywhere, the certificate is decoration — record dead, keep marked.
Q6. **The share as the failure instrument** (research rung; prereg before computing).
    Does the whole-only/beyond-pair structure of the exact state PREDICT where the
    mean-field chart fails? Predictor and null staked first; correlated-error curve
    over the same sweep. KILL (separable, its own): share at floor where chart error
    is large — or large share where the chart is fine — across the sweep kills
    instrument-as-certificate without touching Q5.
Q-seam discipline: freeze-design-early — prereg published for attack before any
instrument runs; findings batch at avenue close (no stance edits mid-avenue); every
long computation detached (setsid + done-markers + RESUME).

## Cross-path couplings, named
- K1's outcome gates Q3/Q4's INTERPRETATION (not their execution). **This gate has now
  FIRED (`929ad8a`, 2026-08-23): route → gauge died, so the lattice work continues as
  physics and THE SHARED-CARRIER CLAIM DROPS.** Q3 and Q4 execute unchanged; anything
  written about them must not read the lattice results as evidence for a shared carrier.
- K2 is also the h3ere2 deliverable Eric originally asked for; its verdict feeds the
  CIRIS UI direction regardless of sign. **That verdict is in (2026-08-23) and the sign
  is negative twice over** — the coupling shows no large effect, and the scaffold costs
  quality against the bare 0.6B baseline. See K2 above; the finding is with Eric.
- N-d (N1) and Q2 share the homogenization-certificate shape — one theorem, two faces
  (the descriptor DAG edge downward; the dynamics aggregation upward).
