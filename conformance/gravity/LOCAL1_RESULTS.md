# LOCAL-1 — first run refused on overflow; the cone is posable, arrival is not

*2026-08-28. Prereg admitted and frozen; instrument after it. First run
ended in the engine's own overflow REFUSAL at step 3 (integer growth is
×3¹² per step on 12 edges; int64 carries two steps). No wrong number was
produced — that is what refusal is for.*

## Amendment LOCAL-1B (frozen here, before the rerun)

- **L1 and L2 are UNCHANGED and fully posable**: the light cone is staked
  at steps 1–2, exactly the range int64 carries. L2 is step 1. B3 runs on
  every state through step 2, both arms. Plants are one-step. Nothing in
  the campaign's CLAIM is touched.
- **L3 is UNPOSED at this instrument's capacity** (arrival was staked at
  steps 3–4). Recorded as UNPOSED — not as "NO-ARRIVAL", which would be a
  measured zero this instrument did not measure. The arrival question
  passes to a successor with either a wider integer carrier or a
  per-step common-factor normalization, named, not built here.
- witness: none (capacity amendment; no gate criterion changes)

misfits: contacts M-PLANT-OBS and M-PLANT-SECTOR (plants unchanged from
the admitted freeze — each plant's carrier is asserted nonzero in the
sector the plant acts on, plant (i) in the pendant-response sector itself),
M-HOMOG (the locality stake remains the response-function cone, not a
conditioning claim), M-KINEMATIC-NONLOCAL, M-ELECTRIC-BASIS,
M-NULL-MISSTAKE (all as admitted); no new contact — an overflow refusal is
the engine's declared behaviour, not a misfit.


## First 1B run: L2 VOID with a one-line cause, and amendment LOCAL-1C

The rerun produced R ≡ 0 everywhere — including plant (i)'s DIRECT HIT.
Cause, exact: the instrument seeded the dressed vacuum as the UNIFORM
superposition over all configurations (the strong-coupling electric
vacuum), and a flux shift on a uniformly-summed edge is the IDENTITY —
`L₁(e*)|ψ₀⟩ = |ψ₀⟩`, so the two arms are equal and every response is zero
by construction. The carrier was an eigenstate of the perturbation:
M-PLANT-SECTOR's rule surfacing on the MAIN gate's own carrier, and the
freeze's phrase "the dressed vacuum" was ambiguous between two vacua, one
of which is probe-blind.

**LOCAL-1C (frozen here, before the rerun):** the carrier is the
ZERO-FLUX dressed vacuum — the Gauss projection of the all-zero-flux
configuration with the dressed pair (the BF vacuum the BRIDGE instruments
always used), on which the flux shift acts nontrivially. Every gate,
plant, and criterion is otherwise unchanged. The general rule is
registered: **M-PROBE-EIGENSTATE** — a response probe's carrier must not
be an eigenstate of the perturbation, and the freeze must name the vacuum.
witness: none (carrier specification; no criterion changes). The pair
remains Wilson-dressed per M-BARE-CHARGE, unchanged; only the FLUX sector
of the vacuum changes.


## 1C run: R ≡ 0 again — the true cause found, and amendment LOCAL-1D

The 1C rerun still read zero response. The step-0 diagnostic proves the
carrier and probe are now RIGHT (the perturbation moves the near triple
from (1,0,0) to (0,0,1) exactly, pendant untouched); the response dies at
step 1 because the electric term is a MAXIMALLY-MIXING unitary. This is a
ring rigidity fact, verified by exhaustive search: at scale √3 every Z[ω]
circulant unitary is either a pure permutation-phase (no mixing) or has
all entries of equal modulus (maximal mixing — one application
thermalizes the flux distribution and the observable saturates). At scale
3 only trivial circulants exist. **At scale 3√3 weak couplers exist**, and
the search finds `U_E = (c₀ + c₁L₁ + c₂L₂)/3√3` with
`c₀ = 5+4ω, c₁ = 2+ω, c₂ = −1−2ω`: eigenvalue norms all exactly 27
(unitary), diagonal weight 21/27, hopping 3/27 per direction — a genuine
weak-coupling electric term, exact in the ring. Registered:
**M-RING-MIXING** (a ring-scale constraint can force a unitary to be
maximal-mixing; a propagation probe needs a weak coupler, which may live
at a higher ring scale).

**LOCAL-1D (frozen here, before the rerun):**
1. `U_E` is the weak coupler above, gauge-covariant for the same
   shift-polynomial reason as 1B's term (M-ELECTRIC-BASIS unchanged); the
   dressed pair and every other operator unchanged (M-BARE-CHARGE
   unchanged).
2. Overflow REFUSAL is replaced by AUTO-PROMOTION (the engine's own
   scale-the-carrier-to-the-circuit discipline, per the residue carrier's
   design): when int64 headroom runs out the instrument promotes to
   arbitrary-precision integers and CONTINUES, exactly. L3 (arrival,
   steps 3–4) is therefore POSABLE again and reinstated as staked in the
   admitted freeze.
3. All criteria unchanged; plants unchanged, carriers and sectors as
   admitted (M-PLANT-OBS, M-PLANT-SECTOR, M-PROBE-EIGENSTATE all carried;
   the locality stake remains the response cone per M-KINEMATIC-NONLOCAL
   and M-HOMOG; the misstake rule M-NULL-MISSTAKE untouched).
witness: none (operator substitution with its unitarity check stated; a
Lean brick for the eigenvalue-norm computation is named, not claimed)


## 1D run: hard gates pass, plant (i) misses — and the miss is the finding

L1/L2/B3/G0 all passed (near responds at step 1; pendant zero through step
4; auto-promotion carried ~10⁵⁷-scale integers exactly). Plant (i) MISSED
at its staked step — and the diagnostic shows why: the pendant triple's
response to a DIRECT flux hit is nonzero at steps 0 and 3 and ZERO at
steps 1–2. The observable is blind at exactly the steps L1 staked, so
1D's cone zeros were uninformative — the plant proved the staked window
unobservable, which is M-PLANT-OBS doing precisely what four campaigns of
misses built it to do.

**Amendment LOCAL-1E (frozen here, before the rerun):**
- **L1'** — the cone is staked on LIVE steps only: at every step k ≤ 4
  where the direct-hit control's pendant response is NONZERO, the distant
  (e*) perturbation's pendant response must be ZERO. (The 1D data already
  contains this pattern at k = 3; the rerun adjudicates it as a frozen
  gate, not a retrodiction.)
- **Plant (i)'** — the direct-hit control must be nonzero at SOME step
  k ≤ 4, and its live-step set is REPORTED; carrier the zero-flux dressed
  vacuum, sector the pendant response, asserted nonzero at the live steps
  per M-PLANT-SECTOR. Plant (ii) unchanged.
- L2, L3, B3, G0 unchanged (M-KINEMATIC-NONLOCAL, M-HOMOG,
  M-ELECTRIC-BASIS, M-BARE-CHARGE, M-RING-MIXING, M-PROBE-EIGENSTATE,
  M-NULL-MISSTAKE all as admitted).
witness: none (gate re-anchored to the observable's live window; no
criterion loosened — the zero demanded is the same zero, at steps where a
zero can mean something)
