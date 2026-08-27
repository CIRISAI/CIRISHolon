# Pre-registration — GRAVITY-BRIDGE-0 V2

*Frozen 2026-08-27, committed ALONE, before any v2 instrument exists (M5).
The results document must cite this file's commit hash. Instrument to be
`bridge0_v2.py`; exact integer arithmetic; no float in any value path.*

## The retrodiction: six misfits from the v1 VOID, each shaping this design

- **M1 — the frozen gates were jointly unsatisfiable** (raw edge kicks vs
  full-scope constraints): a contradiction discoverable before running.
  → RULE: every intervention below carries its constraint-preservation
  DERIVATION in this prereg, and the instrument re-verifies preservation
  at runtime; a runtime violation VOIDs as a design failure, never reads
  as a discovery.
- **M2 — three plants were provably unobservable** (gauge-orbit
  laundering; single-vertex Gauss redundancy on a disk; parity
  protection): the model's symmetries can hide whole defect classes.
  → RULE: each plant below is staked WITH its observability argument.
- **M3 — "matter" was the flux itself**: sector relabeling posed as
  backreaction. → v2 has an INDEPENDENT matter register, and a gate (G7b)
  that its algebra commutes with every gauge constraint.
- **M4 — no dynamics**: states were prepared, not evolved. → v2's
  curvature reading is taken on the IMAGE of a joint coupling map T, and T
  is honestly labeled a constraint-preserving discrete interaction step
  (a class-sum of unitaries — not itself unitary; Hamiltonian evolution
  remains the successor's job).
- **M5 — freeze order unevidenced**: prereg/instrument/results one commit.
  → three commits, hashes chained in the results.
- **M6 — an intervention was group-theoretically inert** (ρ-kick on a
  τ-flux): staked effects must come with their derivation. → every
  predicted reading below has one.

## The model

H = H_matter ⊗ H_gauge. H_matter = span{|0⟩, |ρ⟩, |τ⟩} — an independent
3-level register (occupation label), NOT a gauge sector. H_gauge as in v1
(quantum double of S3 on the fan disk; refined graph as in v1). States:
integer vectors indexed (matter row, gauge configuration).

**The creation operator, with its physicality derivation (M1):**
K_C := Σ_{h∈C} L_h(e*), where L_h left-multiplies the seed edge e* = (1→2)
(base) / (1→4) (refined) and C is a full conjugacy class.
DERIVATION: (a) Gauss: at e*'s tail vertex, action_g∘L_h = L_{ghg⁻¹}∘action_g,
and summing h over a class is conjugation-invariant; at e*'s head the two
commute outright; other vertices don't touch e*. (b) Flatness: e* belongs to
exactly one plaquette (p0) in the fan, so all other plaquette constraints
commute with L_h. Hence K_C preserves every constraint except p0's, which
it maps flat→class-C — that is its job.

**The joint coupling (M4):** T := Σ_m |m⟩⟨m| ⊗ K_{C_m}, C_0 = {e} (K = id).
Constraint-preserving by the derivation above, per matter row.

**Geometry→matter operator:** G[ℓ] := Σ_c P_{class(hol ℓ)=c} ⊗ S_c, with
P diagonal (gauge-invariant) and S_c the cyclic matter shift by the class
index of c (S_e = id). Constraint-preserving: P commutes with everything
gauge; S acts only on matter. DERIVATION of the control (M6): for a
contractible loop in a flat region, class(hol) = e identically on the
support, so G[ℓ_ctrl] = id ⊗ S_e = identity — the control is inert by
theorem, so any matter change under it fires.

## Gates (all exact; each separable)

- **G1**: vacuum⊗|0⟩ — boundary loop reads {e} on every support config.
- **G2 (curvature from the coupling)**: ψ_m := T(vacuum⊗|m⟩), m ∈ {ρ,τ}:
  the DISTANT boundary loop reads exactly {class m} on ψ_m's gauge
  support. Derivation of prediction: K_{C_m} seeds hol(p0) ∈ C_m; outer
  flatness reduces the boundary loop to p0's class.
- **G3**: loop reversal inverts holonomy element-wise on ψ_m's support.
- **G4 (reciprocity, all operators physical)**:
  (i) matter→geometry: X_ρ := (id_gauge ⊗ matter-shift 0→ρ) applied to
  vacuum⊗|0⟩, THEN T: loop reading moves {e}→{ρ}. Without X_ρ: stays {e}.
  (ii) geometry→matter: G[ℓ around p0] on ψ_ρ shifts the matter row from
  ρ to (ρ+ρ)-index — the occupied matter row CHANGES. Derivation: ψ_ρ's
  support has loop class ρ uniformly, so G acts as S_ρ on matter.
  (iii) control: G[∂p1] (a flat outer plaquette's boundary) on ψ_ρ leaves
  the state IDENTICAL (inert by the derivation above).
- **G5 (FULL SCOPE, M1)**: a state REGISTRY records every state
  instantiated in G1–G4 (vacuum, every T image, every X image, every G
  image, on both graphs); the gate checks constraints on EVERY registry
  entry: Gauss at every vertex per matter row; flatness at every plaquette
  except p0 on rows m≠0 where class C_m is required (row 0 requires p0
  flat). Any violation on any registry state fires G5.
- **G6**: identical G1/G2/G4 verdicts on the refined graph.
- **G7 oracles, staked now**: gauge support of vacuum⊗|0⟩ = 6^(V−1)
  (216 base / 1296 refined); gauge support of ψ_m's occupied row =
  |C_m|·6^(V−1) (τ: 648/3888; ρ: 432/2592). **G7b (M3)**: every matter
  operator used (S_c, X_ρ) commutes with every gauge constraint operator,
  verified exactly.
- **G8 plants, with observability arguments (M2)**:
  (i) wrong-side group action in the gauge average — OBSERVABLE because it
  breaks the group-action property itself, so orbit weights become
  non-uniform (demonstrated in the v1 record); must fire G5.
  (ii) dropped-inverse holonomy word read on the ρ-sector — OBSERVABLE
  because that sector is parity-even and the identity provably leaks into
  the broken reading (v1 derivation); must fire G2's reading.
  A missed plant VOIDS the campaign. No post-run plant replacement.

## Meaning

All gates + both plants as staked → "on an exact discrete gauge model with
an INDEPENDENT matter register and constraint-preserving operators
throughout: curvature at a distance emerges from one joint interaction
step, matter→geometry and geometry→matter influences are both real with an
inert control, all constraints hold on every state touched, the reading is
refinement-invariant, and the harness can fail." Still a finite-group BF
toy — the fence and the SU(2) ladder are unchanged from v1. Any gate
firing kills its rung; VOID rules as in v1; no rescue.
