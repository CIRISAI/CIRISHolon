# The holon's data object — designed once, priced by the ladder

*The requirement: one crate, one recursive object, every tier an instance.
This document designs the data objects to MATCH SOTA at tiers 0/1, then
answers whether the same objects serve the remaining tiers, and prices every
delta by a theorem in `lean/`. Benchmarks are contract obligations: each
claim below names its measured baseline and its SOTA reference.*

## Tier 0 (classical/diagonal) — SOTA and its object

SOTA is bit-packed word-parallelism: one BIT per degree of freedom, u64/SIMD
blocks, gates as masked XOR/AND. The object:

```rust
/// One observable across MANY degrees of freedom: the plane.
/// F2 planes pack 64 dof/word; ops are word-parallel.
pub struct BitPlane { words: Vec<u64>, len_bits: usize }
```

Reference: any production bit-slice engine. Our current tier-0 (byte-per-bit)
is ~64× off this layout by construction. Baseline to beat: measured 15×
FASTER than qiskit's numpy statevector on classical-routable circuits already
— the packed plane makes that structural.

## Tier 1 (stabilizer/Clifford) — SOTA and its object

SOTA is Stim (Gidney): the tableau as **paired bit-planes** — a Pauli row is
an X-plane bit and a Z-plane bit per qubit, plus a sign — all operations
column-XORs and word-ANDs over the planes.

```rust
/// A Pauli row IS two tier-0 planes and a sign.
pub struct PauliPlane { x: BitPlane, z: BitPlane, sign2: u8 /* mod 4 */ }
pub struct Tableau { destab: Vec<PauliPlane>, stab: Vec<PauliPlane> }
```

**The structural fact this exposes: the tier-1 SOTA object is two tier-0
objects plus a phase ledger.** The recursion is already present at the
bottom: a "higher" tier's state is planes of the lower tier's planes, plus a
richer ledger. Baseline: our unpacked tableau measured within 2.2× of
qiskit's `StabilizerState` (n=256, depth 5120: 52 ms vs 24 ms) and an
estimated ~10²–10³ from Stim; the packed PauliPlane closes the layout share
of that gap. Every phase bug the campaigns caught (S-phase, CX-phase, the
Gauss-sum XOR expansion) lived in the sign/ledger channel — the ledger is
where correctness risk concentrates, which is why it is a named object, not
a field scattered through code.

## The universal object, stated once

```rust
pub struct Holon<L: Ledger> {
    /// Struct-of-arrays: one plane per OBSERVABLE. F2 planes bit-packed;
    /// R planes are f64 lanes (SIMD). A plane is the engine-native "view".
    planes: Planes,
    /// Phases / coefficients / weights, with their ring made explicit:
    /// Z2 sign -> Z4 -> Z[ω]·2^{-m/2} (exact cyclotomic) -> f64.
    /// Also carries the rent accounting (budgets are data, not comments).
    ledger: L,
    /// Charts: partitions/binnings with their CONDITIONING declared
    /// (coherence of every exposed aggregate), per lean/Object.lean.
    chart: Chart,
    /// The certified square: view id, step id, rate id, battery receipt.
    /// A tier without this is not a tier (Tier.certifies is a FIELD).
    cert: Certificate,
    /// Recursion: children in an APPEND-ONLY arena — identity is the arena
    /// index forever (LESSONS.md rule 1; bought by three campaigns).
    children: Arena<Holon<L>>,
}
```

## Do tiers 2+ need the same object? The delta, and why — each priced

**The shape survives everywhere. Exactly three things deform, and each
deformation is a ladder rung with a theorem and a measured slope:**

1. **The plane FIELD deforms: F₂ → Z[ω] amplitudes → ℝ.**
   Why: exactness is free only while the dynamics keeps the view Closed over
   a finite alphabet. Gottesman–Knill-as-closure
   (`tableau_closed_under_hadamard`) is WHY tier 1 gets to stay on F₂ planes;
   the wall (`tableau_not_closed_under_rotation`) is where F₂ ends. On ℝ
   planes, conditioning becomes load-bearing (`sum_perturb_le`: a
   near-cancelling ℝ-aggregate amplifies noise by 1/coherence — measured:
   momx χ collapsing 0.998→0.125 while ke held 1.0). Design rule: keep the
   field exact as deep as closure allows; declare conditioning wherever ℝ
   enters.

2. **The LEDGER deforms: sign bit → cyclotomic ring → branch weights → bond
   indices.** Why: every wall crossed forces a richer ledger, and the
   ledger's size IS the measured price. Magic: branches of tier-1 objects
   with Z[ω] weights — cost 2 per T-gate (measured slope 1.005). Entangled
   bulk: bond indices between plane-blocks (MPS) — cost χ³, poly in n
   (measured 1.26 at fixed t). These two compressions are ALONG DIFFERENT
   RUNGS (emptiness vs fiber-size), and the dead-bridges enumeration says the
   rungs are not interconvertible — which is exactly why no single flat
   object compresses both, and why the ledger is a TYPE PARAMETER rather
   than one fixed struct. The statevector is not a new shape: it is the
   branch ledger saturated at 2ⁿ.

3. **The CHART deforms from exact to certified-approximate.** At tiers 0–2 a
   chart is exact (a bit's position is its meaning). From the grain tier up,
   charts are cells with budgets: the Aggregation theorems price the
   approximation (K ≤ 1 ⇒ linear error budgets, `horizonBudget_le_of_
   nonexpansive`; measured unbreached across seven geometries), and
   convergence premises VOID what they cannot certify. Same slot in the
   struct; the certificate carried in it changes from "exact by
   construction" to "battery receipt with budgets".

**What does NOT change, at any tier:** planes as the state layout (bit-packed
F₂ at the bottom, SIMD f64 at the top — one memory discipline); the ledger as
a first-class object (phases at the bottom, rent at the top — and note these
are the SAME slot: the thing the dynamics must pay to keep the
representation honest); charts with declared conditioning; the certificate as
a constructor requirement; and append-only arena identity for recursion.

## The benchmark contract

Every tier instance ships with: (a) conformance vs its referee (exact where
one exists, upstream tier otherwise), (b) planted-mutation observability
(null plants are convicted, not counted — twice measured this season),
(c) its declared cost slopes RE-MEASURED in CI, and (d) a SOTA ratio against
the named reference (Stim / Aer-qsim / ITensor-TeNPy / Bravyi–Gosset), with
the standing rule: **a performance claim without a moved benchmark ratio does
not merge.** Current measured baselines: statevector 15× over qiskit-numpy;
tableau 0.45× of qiskit StabilizerState; magic tier exact where no reference
offers exactness at all; DMRG unbenchmarked against ITensor (owed).

---

## DESIGN LOCK v1 (2026-08-27) — what implementation taught

The object shipped as designed, in ONE crate (`engine/crates/holon`): planes,
ledger, chart, certificate, arena — with tier instances certified against the
QASM-suite reference tiers and three deltas the implementation surfaced,
recorded here so the lock is honest:

1. **Row-product role order is part of the object.** Pauli phases
   anticommute, so `rowsum`'s source-vs-target slot assignment is
   load-bearing; it is now documented AT the operation, and the conformance
   suite is what caught the transposed draft.
2. **Signs live mod 4 in intermediates** even though physical rows are ±:
   the ledger's ring is wider in flight than at rest. This is the tier-1
   miniature of the general ledger law (rest ring ⊂ flight ring), and it is
   why the ledger is typed, not implicit.
3. **The conditioning declaration is COMPUTED, not asserted**: `RealHolon`
   derives its chart's coherences from the planes at construction. The
   season's measured pattern (signed near-cancelling ill-conditioned,
   all-nonnegative exactly 1) is a standing test.

Viability, tested: tier 0 (bit-planes), tier 1 (packed Pauli planes —
conformant to the certified tableau at peek and distribution level), tier 2
ledger (Z[ω] ring isomorphic in action to the certified magic tier's over 500
random elements), carrier/physics ℝ-planes (conditioning measured), bulk MPS
shape (minimal instance exact), recursion (arena-index identity, coarse
Closed view of children). Measured benchmark movement at lock:
**packed tableau 7.8 ms at n=256, depth 5120 including full measurement —
6.7× over the unpacked tier, 3.1× faster than qiskit's StabilizerState**;
Stim remains the named target (est. 1–2 orders; SIMD + transposed layouts).
