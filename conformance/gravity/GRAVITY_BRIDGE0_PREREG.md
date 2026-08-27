# Pre-registration — GRAVITY-BRIDGE-0: curvature as dynamical holonomy, exactly

*Frozen 2026-08-27, before the instrument existed. Method and the meaning of
every possible answer staked here; the instrument is `bridge0.py`, exact
integer arithmetic throughout (unnormalized state vectors over group
configurations — no float in any value path).*

## The model, and the honest fence

Kitaev quantum double of G = S3 on a triangulated disk — a discrete 2+1D
gauge theory of BF type: Gauss constraints (gauge averages Ā_v) and
flatness constraints (holonomy projectors B_p), with MATTER as a flux
excitation pinned to a conjugacy class at the central plaquette. S3 ≅ D3 ⊂
O(2): the 3-cycle class ρ is rotation by ±120°, the transposition class τ
is a reflection — so a ρ-flux is a LITERAL discrete conical deficit of
120°. FENCE: this is a finite-group BF toy; SU(2) BF (2+1 gravity proper)
is the named successor, and nothing here is 3+1D gravity. Curvature must
arise from constraints and matter content only — there is NO prescribed
potential anywhere in the instrument.

## Geometry

Base graph: triangle fan — central vertex c, boundary 1,2,3; spokes c→1,
c→2, c→3; boundary edges 1→2, 2→3, 3→1. V=4, E=6, F=3 plaquettes;
state space 6^6 = 46,656. Central plaquette p0 := (c→1, 1→2, 2→c⁻¹)... the
plaquette words are fixed in the instrument header and shared by every
gate. Encircling loop ℓ := the disk boundary 1→2→3→1.
Refined graph: split edge 1→2 at new vertex 4 and add spoke c→4 (V=5,
E=8, F=4; 6^8 = 1,679,616) — same p0 class content, same boundary loop
through 4.

## Gates, each separable, all exact (no tolerances anywhere)

- **G1 vacuum flatness**: the gauge-invariant flat state's support has
  hol(ℓ) = e on EVERY configuration. Any non-identity holonomy on any
  support state fires.
- **G2 conical deficit**: with p0 pinned to class C (run BOTH C = τ and
  C = ρ), every support configuration has hol(ℓ) in class C exactly. The
  ρ case is the discrete 120° deficit.
- **G3 orientation**: hol(ℓ reversed) = hol(ℓ)⁻¹ on every support
  configuration. HONEST NOTE staked now: S3 is ambivalent (every class is
  inverse-closed), so the CLASS-level inverse statement is trivially
  satisfied here; the posable exact content is element-level inversion,
  and a non-ambivalent group (e.g. F21) is the follow-up where class-level
  inversion becomes falsifiable.
- **G4 reciprocity with locality control**: (i) matter intervention
  (re-pin p0 from τ to ρ) changes the DISTANT loop reading from τ to ρ;
  (ii) geometry intervention (left-multiply one edge of p0's boundary by a
  fixed g — a local holonomy kick) changes the matter reading at p0;
  (iii) CONTROL: the same kick on an edge bounding only outer plaquettes
  leaves p0's reading exactly unchanged. All three must hold.
- **G5 constraints Held**: on every state used in G1–G4: Ā_v ψ = 6·ψ for
  every vertex (exact integer equality), B_p ψ = ψ for every unpinned
  plaquette, and B^C_{p0} ψ = ψ where pinned.
- **G6 refinement invariance**: G1 and G2 verdicts identical on the
  refined graph — exact, not asymptotic (the theory is topological; any
  drift fires).
- **G7 independent oracle**: measured support sizes equal the closed forms
  staked NOW, derived from the pure-gauge parametrization before the
  instrument ran: vacuum support = |G|^(V−1) (base: 216; refined: 1296);
  flux-C support = |C|·|G|^(V−1) (base: τ 648, ρ 432; refined: τ 3888,
  ρ 2592). Disagreement fires G7 even if G1–G6 pass.
- **G8 the plants (a harness that cannot fail is not a harness)**:
  (i) a broken Gauss average (one group element skipped at one vertex)
  must FIRE G5; (ii) a wrong holonomy composition (one edge's inverse
  dropped in the plaquette word) must FIRE G1 or G7. Both plants must
  fire, and their firing is part of the record.

## Meaning of the answers

ALL of G1–G8 as staked → "curvature emerges dynamically from constraints
and matter content; matter–geometry influence is reciprocal with local
scope; the reading is refinement-invariant; the harness can fail and
didn't" — the bridge's rung 0, at finite-group scope, nothing more. Any
gate failing → that rung's claim dies and is kept marked; the instrument
is not rescued post-hoc (any follow-up is a new prereg). A plant failing
to fire VOIDS the whole campaign regardless of G1–G7.
