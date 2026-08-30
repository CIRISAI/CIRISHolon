# BRIDGE-2 — adjudicated: five rungs pass frozen on both graphs; one fires by theorem

*2026-08-27. Prereg 4e26345, instrument committed separately, run log
`bridge2_run.log`. Separable gates; per-rung verdict.*

## PASSED under frozen gates, both triangulations
- **B1′ — matter→geometry by interference, and the result exceeds its
  stake**: the exact normalized overlap between the occupied-evolved and
  empty-evolved geometries reads **ω = 0, 1, 0, 1** at k = 1..4 — not
  merely ω<1 at odd k as staked, but EXACT ORTHOGONALITY: matter's imprint
  on geometry is maximal at odd steps and coherently, completely erased at
  even steps. Both independent routes (262,144-dim evolution; 8-dim
  corrected combinatorial recursion) agree exactly, both graphs.
- **B4 — the closure rung, replicated a third time** (δ = 0, ½, 0, ½).
- **B3 — full-scope Gauss** on every registry state, both graphs.
- **B6′ — the support oracle, both routes**: the corrected enumerator
  (weight (ρ(g₁)ρ(g₂)ᵀ)₀₀) reproduces the 512 uncharged calibration AND
  the measured 256 charged support — misfit M12 closed by derivation.
- **B5′ refinement** on the staked rungs; **both plants FIRE**.

## FIRED: B2′, both graphs, same integer — and the diagnosis is a theorem
The geometry-conditioned operator (A2's scalar phase) is OCCUPATION-BLIND:
any gauge-diagonal phase commutes with the occupation-coherence
observable (U†(σ⁻⊗I)U = σ⁻⊗I for occupation-blind U), so B2′ was inert
BY THEOREM at every k — the fire was guaranteed the moment A2 reduced the
matter action to scalars, and B1′'s own ω-structure closes the other
escape (branches identical at even k, orthogonal at odd). **Misfit M13.**
The lawful geometry→matter channel: condition on geometry, act on the
OCCUPATION qubit — the gauge-neutral matter degree of freedom. Per the
pilot-first house rule, BRIDGE-3's single re-staked gate is frozen only
after an exploratory pilot (labeled as such) measures the response
surface of that operator family.

## Standing after two frozen campaigns
Established under frozen gates on genuine unitary dynamics with exact
arithmetic: curvature-at-a-distance from matter (B1′, maximal), its
coherent self-erasure (ω revival), the classical view's closure failure
through coherence (B4, thrice), constraints held throughout (B3), all
oracles two-route (B6′), refinement-invariant (B5′), harness falsifiable
(plants). OPEN: the geometry→matter direction (B2 family), owed to
BRIDGE-3 with a pilot-informed stake. *[Annotation 2026-08-30: this debt
was PAID — BRIDGE-3 ran 2026-08-27, pilot-designed then frozen (prereg
8b12296), and passed all gates with both plants firing; see
BRIDGE3_RESULTS.md. The line above is kept as written because a record is
a history; it stopped being current the day after it was written.]* The fence is unchanged: finite
D4 toy; SU(2) is the ladder.


## Upgrade, 2026-08-28: from δ-against-one-model to the COLLISION theorem

An external review made a correct and sharp point: the measured δ compares
the coarse view against ONE preregistered Markov model, so it earns "worse
than that model", not "best memoryless". The measured sequence already
proves the stronger thing, and it is now machine-checked
(`lean/CIRISHolon/Closure.lean`):

    v₁ = v₃ = (½,½),  v₂ = (0,1),  v₄ = (1,0)

Any time-homogeneous memoryless map F would need F(v₁)=v₂ AND F(v₃)=v₄
simultaneously — but v₁ = v₃ while v₂ ≠ v₄, and a function cannot send one
input to two outputs. `collision_refutes_memoryless` is that argument and
uses nothing about geometry: it is the pigeonhole for functions, which is
exactly why it beats any model-specific comparison.
`minimax_error_at_least_half` gives the quantitative half: the two required
successors are at maximal total-variation distance, so the best single
prediction is wrong by ≥ ½ — which is where the measured δ = ½ comes from,
now derived rather than compared.

**Scope, kept explicit because the review was right to insist:** this
refutes MEMORYLESS closure of the DECLARED view. A classical model carrying
extra memory (a phase label) can reproduce the sequence, and
coarse-graining-induced memory is standard (Mori–Zwanzig; process-tensor
memory witnesses). The claim is that the declared coarse view is
non-autonomous and its missing memory has a measured cost — not that no
classical model exists.
