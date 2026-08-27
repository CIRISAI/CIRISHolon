# Pre-registration — GRAVITY-BRIDGE-1: Hamiltonian-class dynamics, and the closure rung

*Frozen 2026-08-27, committed ALONE before any instrument. Successor to
BRIDGE0 v1 (VOID) and v2 (VOID); carries misfits M1–M8. The results
document must cite this commit's hash. Instrument: `bridge1.py`.*

## The pivot, stated

One-step coupling maps were synthesis. This campaign runs GENUINE
DISCRETE-TIME UNITARY DYNAMICS — a Floquet/Trotter step T built from local
gauge-invariant unitary terms, applied repeatedly — on a joint
matter⊗gauge system, and measures, alongside curvature and backreaction,
the thing that makes this OUR object rather than a re-implementation:
**the closure defect of a coarse geometric view under the true dynamics**
(the square: does v∘T = T_cl∘v within a staked defect, and where does the
classical description FAIL to close). Honest label staked now: Floquet
dynamics is Hamiltonian-class (Trotterized local terms), not a continuum
Hamiltonian; the fence from v1 stands (finite-group toy; SU(2) is the
ladder's continuum end).

## The model (all derivations staked here, per M1/M6)

Gauge group **D4** = ⟨r, s | r⁴=s²=e, srs=r⁻¹⟩ (order 8), on the v1 fan
disk (V=4, E=6, F=3; refined variant as in v1). Amplitudes live in
**Z[i] · 2^(−k/2)** — Gaussian integers with a global √2 exponent (a
subring of the engine's Z[ζ8] ledger). Matter: a register at vertex 1
carrying D4's 2-dim irrep ρ₂ (INTEGER matrices: r ↦ [[0,−1],[1,0]],
s ↦ [[1,0],[0,−1]]) plus an occupation qubit — total matter dim 2 (spinor)
— gauge transformations at vertex 1 act jointly on gauge edges AND rotate
the spinor by ρ₂(g); the Gauss constraint at vertex 1 includes the matter
charge. THAT is what makes the matter matter (M3).

**The Floquet step** T = U_B · U_E · U_geo→mat · U_mat→geo, each term
unitary, exact, and gauge-invariant by the derivations below:
- U_B (magnetic): diagonal phase i^{κ(class of hol(p))} per plaquette,
  κ(e)=0, κ(r²)=2, κ({r,r³})=1, κ(reflections)=3. Diagonal in a
  gauge-invariant quantity ⇒ commutes with Gauss. Entries: powers of i.
- U_E (electric): per edge, (1/√2)(I + i·R_{r²}) with R right-
  multiplication by the CENTRAL involution r². Unitary because r² is an
  involution (exp(iπ/4 · R) form); gauge-invariant at every endpoint
  because r² is CENTRAL (commutes with all left and right multiplications).
  Entries: Z[i]·2^(−1/2).
- U_geo→mat: Σ_c P_{class(hol p0)=c} ⊗ M_c on the spinor, with M_e = I,
  M_{r²} = ρ₂(r²) = −I → use M_{r²} = σ_z·, M_{r-class} = ρ₂(r),
  M_{refl} = ρ₂(s) (integer matrices, unitary). P diagonal gauge-invariant;
  M acts on matter ⇒ the product is gauge-invariant PROVIDED M_c commutes
  with the matter gauge action ρ₂(g)... it does NOT for all g; the staked
  form uses only c-controlled CENTRAL matter rotations: M_c = (−I)^{κ₂(c)}
  with κ₂(r²-class)=1 else 0, plus a c-controlled PHASE i^{κ(c)} ⊗ I.
  Central/scalar matter actions commute with every ρ₂(g) ⇒ gauge-invariant.
  (Derivation note, staked honestly: richer non-central M_c would break
  Gauss at vertex 1; the scalar/central family is the exactly-lawful one.)
- U_mat→geo: occupation-controlled application of L_{r²}(e*) on the seed
  edge (e* rims only p0): central ⇒ commutes with all gauge actions;
  controlled on the matter occupation bit (a matter-diagonal projector,
  commutes with ρ₂ up to... ρ₂ acts on the spinor index, occupation is a
  separate qubit ⇒ commutes). Effect: an occupied matter site pumps a
  CENTRAL flux r² (a literal 180° conical deficit) into p0 each step.

**Overflow refusal staked (house rule)**: entries are int64 pairs; the
instrument asserts max |entry| < 2^62 before every step and REFUSES the
run (VOID as instrumentation, not physics) on breach. Step counts staked:
k = 1..6 Floquet steps (within the bound by construction; the assert is
the backstop).

## Gates

- **B1 (dynamical curvature)**: from vacuum ⊗ |occupied, spinor +⟩, after
  k ≥ 1 steps the DISTANT boundary loop's exact class distribution puts
  ALL weight on the r²-class ladder pumped by the matter (k odd: class r²;
  k even: class e — the central flux toggles), while from
  vacuum ⊗ |empty⟩ the loop stays exactly class-e at every k ≤ 6.
  Prediction derived from centrality: fluxes compose as r²ᵏ.
- **B2 (geometry→matter)**: with an r² flux prepared at p0 (by one
  occupied step, matter then EMPTIED by staked projection — a preparation,
  labeled as such), an empty-but-spinored matter register's phase
  observable (exact ⟨σ_z⟩-type integer form) after one U_geo→mat differs
  from the flat-geometry case. Locality control: the same with the flux's
  plaquette replaced by a distant flat plaquette's loop in U_geo→mat's
  control is EXACTLY inert.
- **B3 (constraints Held under evolution, M1/M7)**: the full Gauss
  projector (joint gauge+matter action at vertex 1; gauge-only elsewhere)
  and the state-DEPENDENT flatness bookkeeping (p0's expected class at
  step k is r²^(k·occ), staked as a function of the trajectory) hold
  EXACTLY on every state in the registry (every step, every branch of
  B1/B2, both graphs).
- **B4 (the closure rung — the headline)**: define the coarse geometric
  view v(ψ) := the exact per-plaquette class distribution triple. Define
  the staked classical update T_cl: central-flux bookkeeping (p0's class
  advances by r² iff matter occupied; other plaquettes stay flat; loop =
  product). Measure the closure defect δ_k := (v∘T^k)(ψ0) vs (T_cl^k∘v)(ψ0)
  as exact distributions. STAKED PREDICTIONS: δ_k = 0 for the
  occupation-diagonal initial states above (the classical view CLOSES —
  central dynamics is classical); δ_k ≠ 0 from the staked SUPERPOSED
  initial state (occupation in (|0⟩+|1⟩)/√2): the coarse view's update is
  NOT autonomous (the distribution mixes with weights the classical
  bookkeeping cannot carry) — the measured NON-closure is the point: the
  classical geometric description fails exactly where superposition
  enters, and both the closure and its failure are EXACT statements.
- **B5 (refinement)**: B1 and B4 verdicts identical on the refined graph.
- **B6 (oracles)**: vacuum gauge-support = 8^(V−1) per matter component
  (512 base / 4096 refined); after k occupied steps support unchanged
  (central L is a permutation) — staked. Matter-op/Gauss commutation
  verified exactly (as v2's G7b).
- **B7 (plants, observability re-derived for THIS instrument, M8)**:
  (i) wrong-side action in the gauge average must fire B3 — observable
  because it breaks the group-action property (non-uniform orbit weights;
  argument independent of state family). (ii) a NON-central pump
  (L_r instead of L_{r²}) planted in U_mat→geo must fire B3's Gauss check
  at e*'s endpoints — observable BECAUSE r is not central: the derivation
  that makes the real pump lawful is exactly what makes the plant fire.
  A missed plant VOIDS. No post-run replacement.

## Meaning

All gates + plants → "on an exact joint gauge+matter system under genuine
discrete-time unitary dynamics: matter dynamically creates curvature at a
distance, geometry acts back on matter with an inert control, constraints
hold at every step, the coarse classical-geometry view provably CLOSES on
classical sectors and provably FAILS to close under superposition — the
closure boundary measured exactly." Any gate fires → that rung dies, kept.
VOID semantics as before. Successors: non-central matter hops (charged
matter proper, needs the full Z[ζ8] ring), then SU(2).
