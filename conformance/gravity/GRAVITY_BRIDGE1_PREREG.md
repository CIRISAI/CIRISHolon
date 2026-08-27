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

## AMENDMENT A1 — 2026-08-27, pre-instrument, pre-data

Recorded before `bridge1.py` exists. Two staked derivations above are
WRONG, provably, and are corrected here with the proofs; the freeze's
no-post-RESULT-rescue rule is untouched (no result exists).

1. **B3 held constraints are GAUSS ONLY.** In the dynamical theory
   flatness is not a constraint — it is the magnetic ENERGY (U_B), and
   [U_E, B_p] ≠ 0 is the quantum dynamics itself. The original B3 carried
   flatness-as-constraint over from the kinematic campaigns (misfit M7's
   sibling). B3 now: the JOINT Gauss projector (gauge+matter at vertex 1,
   gauge elsewhere) holds exactly on every registry state at every step.
   Plaquette classes become OBSERVABLES.
2. **B1's sharp-class prediction was false by derivation:** U_E's central
   right-multiplications toggle plaquette and loop classes in
   superposition. Corrected B1, with the exact algebra: R := R_{r²}(e) is
   an involution, so U_E(e) = (1+iR)/√2 = e^{iπR/4} EXACTLY; per edge the
   k-step electric action is e^{ikπR/4}, giving central-parity toggle
   weight sin²(kπ/4) — ½, 1, ½, 0 for k = 1..4 (full coherent revival at
   k=4). Corrected gates: (i) TWO-ROUTE agreement — the state-vector
   evolution's exact loop-class distribution must equal an INDEPENDENT
   combinatorial route (the closed 2^E-branch central-sector sum, a
   separate function sharing no code with the evolution); (ii) occupied vs
   empty initial occupation give exactly different distributions at every
   k ∈ 1..4.
3. **B4, the closure rung, corrected and sharpened:** the coarse view
   v = the loop's central-parity distribution. The staked classical
   bookkeeping T_cl is the one-step Markov toggle map (edge toggles with
   the k=1 weight ½, composed k times — memoryless by construction).
   STAKED: δ(k=1) = 0 (they agree at ½), δ(k=2) = ½ (quantum toggles with
   weight 1; Markov mixes to ½), δ(k=4) = ½ (quantum REVIVES to 0; Markov
   stays ½). The classical geometric description fails by COHERENCE, not
   noise, and revives — a memoryless view cannot close over a coherent
   geometry, measured exactly. A DEPHASED control (running T_cl itself)
   matches T_cl identically.
4. B2, B5, B6, B7 unchanged. Step counts k ≤ 4; the overflow refusal
   stands.

## AMENDMENT A2 — 2026-08-27, pre-instrument, pre-data

Two more derivation defects found while designing the instrument, both
corrected before it exists; both are physics the exact arithmetic
surfaces, recorded as such:

1. **A lone charge is forbidden by Gauss on a disk** (irrep orthogonality:
   Σ_g ρ₂(g) = 0, so the gauge average annihilates a single ρ₂-charged
   spinor — the staked one-site matter register has EMPTY charged physical
   space). Corrected: matter = an occupation qubit plus a SCREENED PAIR of
   ρ₂ spinors at vertices 1 and 2, initialized in the pair singlet (the
   invariant of ρ₂⊗ρ₂, integer vector |00⟩+|11⟩); Gauss at vertices 1 and
   2 acts jointly on gauge and the respective spinor. The spinor pair is
   inert in B1/B2/B4's readings (it exists to make the matter genuinely
   charged and B3 genuinely joint); the refined-graph runs use the
   occupation-only register, and the results must say so.
2. **The staked U_geo→mat phases cancel on the central ladder**
   (i^{κ(r²)}·(−1)^{κ₂(r²)} = (−1)(−1) = +1 — inert exactly where the
   dynamics lives). Corrected: U_geo→mat = i^{κ(c)} alone; on the r²
   class the controlled phase is i² = −1, giving B2 its observable
   relative phase.
