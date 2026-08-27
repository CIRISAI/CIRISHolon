/-
CIRISHolon.MergeLaw — shard-invariance as a THEOREM, not a test result.

The engine's mesh merges per-shard partial sums of exact branch contributions
and claims the result is bit-identical regardless of shard count or schedule.
Until this file, that claim rested on tests at 1/8/32 shards — which is
evidence, not proof (house rule: a test at three points establishes three
points). The 2026-08-27 prior-art sweep found the algebra itself is textbook
(CRDTs 2011, BSP 1990, LVars 2013) and ReproBLAS states the property verbatim
for floats — but NO quantum simulator in the record carries a machine-checked
merge law. This file is that move.

The model, honestly scoped:

* A RUN is a `Multiset` of contributions in an additive commutative monoid.
  `Multiset` quotients by permutation, so SCHEDULE-invariance is carried by
  the type itself: two schedules of the same work are the same multiset.
* A SHARDING is any multiset of shards whose join is the run — arbitrary
  count, arbitrary sizes, arbitrary assignment.
* The engine's actual fold merges aligned coefficient lanes componentwise
  (`holon-gpu/src/ring.rs::align_to`, `Cyc::add` after alignment): that is
  `Fin 4 → ℤ` addition, an instance of the monoid. The ALIGNMENT itself and
  the i128 envelope refusal are the Rust side's obligations (tested, and the
  envelope is enforced by refusal); this file covers the fold that follows.
* The CERTIFICATE lemma: any additive-monoid homomorphism (e.g. reduction of
  every lane mod p) commutes with the whole sharded fold. Contrapositive:
  if a claimed merged result's digest disagrees with the merge of per-shard
  digests, the claimed result is NOT the true merge — a corruption detector
  with zero false positives, which float-based ABFT cannot offer at any
  tolerance. (Framing: certifying algorithms, McConnell–Mehlhorn–Näher–
  Schweitzer 2011; mechanism ancestry: Huang–Abraham ABFT 1984.)

Credits, per the convergent-art rule: the algebra is Shapiro–Preguiça–
Baquero–Zawirski (CRDTs), Valiant (BSP), Kuper–Newton (LVars), Demmel–Nguyen
(ReproBLAS, the float-hard version); exactness is what makes our instance the
easy case, and this file is the part none of them state for a simulator.
-/
import Mathlib.Algebra.BigOperators.Ring
import Mathlib.Data.Int.Cast.Lemmas
import Mathlib.Data.ZMod.Basic

namespace CIRISHolon.MergeLaw

variable {M : Type*} [AddCommMonoid M]

/-- A sharding of a run: any multiset of shards that jointly carry exactly
    the run's contributions. Shard count, shard sizes, and assignment are all
    free; schedule is quotiented away by `Multiset` itself. -/
def IsSharding (shards : Multiset (Multiset M)) (run : Multiset M) : Prop :=
  shards.join = run

/-- The mesh fold: each shard reduces locally, then the partials merge. -/
def shardedFold (shards : Multiset (Multiset M)) : M :=
  (shards.map Multiset.sum).sum

/-- **The merge law.** Every sharding of a run folds to the run's own sum —
    shard-count- and schedule-invariance in one statement. -/
theorem shardedFold_eq_sum {shards : Multiset (Multiset M)} {run : Multiset M}
    (h : IsSharding shards run) : shardedFold shards = run.sum := by
  rw [shardedFold, ← Multiset.sum_join, h]

/-- Any two shardings of the same run agree bit-for-bit. -/
theorem shardedFold_invariant {s₁ s₂ : Multiset (Multiset M)} {run : Multiset M}
    (h₁ : IsSharding s₁ run) (h₂ : IsSharding s₂ run) :
    shardedFold s₁ = shardedFold s₂ := by
  rw [shardedFold_eq_sum h₁, shardedFold_eq_sum h₂]

/-- The single-shard run is the degenerate sharding: the mesh result IS the
    serial result. -/
theorem shardedFold_single (run : Multiset M) :
    shardedFold ({run} : Multiset (Multiset M)) = run.sum := by
  simp [shardedFold]

/-- **The certificate lemma.** A homomorphic digest commutes with the entire
    sharded fold: digesting the merged result equals merging the per-shard
    digests. -/
theorem digest_commutes {N : Type*} [AddCommMonoid N] (φ : M →+ N)
    (shards : Multiset (Multiset M)) :
    φ (shardedFold shards) = shardedFold (shards.map (Multiset.map φ)) := by
  simp only [shardedFold, Multiset.map_map, Function.comp_def,
    map_multiset_sum, Multiset.map_map]

/-- **Zero false positives.** If a claimed merged result's digest disagrees
    with the fold of the per-shard digests, the claim is not the true merge.
    This is the detection guarantee the certificate layer rests on; in an
    exact ring the digest is exact, so there is no tolerance and no false
    alarm. -/
theorem digest_convicts {N : Type*} [AddCommMonoid N] (φ : M →+ N)
    (shards : Multiset (Multiset M)) (claimed : M)
    (h : φ claimed ≠ shardedFold (shards.map (Multiset.map φ))) :
    claimed ≠ shardedFold shards := by
  intro he
  exact h (he ▸ digest_commutes φ shards)

/-! ### The engine's instance

The mesh fold's carrier: exact cyclotomic coefficient lanes at a common
denominator exponent, added componentwise — `Fin 4 → ℤ` (the alignment that
produces the lanes is `ring.rs::align_to`, refusal-guarded on the Rust side).
The digest the certificate uses: every lane reduced mod p. -/

/-- Aligned coefficient lanes of the exact scalar: (c₀, c₁, c₂, c₃) at a
    shared √2-exponent, as the mesh fold actually adds them. -/
abbrev Lanes : Type := Fin 4 → ℤ

/-- The mod-p lane digest, as an additive-monoid homomorphism — which is all
    `digest_commutes` and `digest_convicts` need. -/
def laneDigest (p : ℕ) [NeZero p] : Lanes →+ (Fin 4 → ZMod p) :=
  { toFun := fun c i => (c i : ZMod p)
    map_zero' := by funext i; simp
    map_add' := by intro a b; funext i; simp [Pi.add_apply] }

/-- The engine's merge law, instantiated at its own carrier. -/
theorem lanes_shardedFold_invariant {s₁ s₂ : Multiset (Multiset Lanes)}
    {run : Multiset Lanes}
    (h₁ : IsSharding s₁ run) (h₂ : IsSharding s₂ run) :
    shardedFold s₁ = shardedFold s₂ :=
  shardedFold_invariant h₁ h₂

/-- The engine's certificate, instantiated: a mod-p lane digest convicts any
    claimed merge it disagrees with. -/
theorem lanes_digest_convicts (p : ℕ) [NeZero p]
    (shards : Multiset (Multiset Lanes)) (claimed : Lanes)
    (h : laneDigest p claimed
        ≠ shardedFold (shards.map (Multiset.map (laneDigest p)))) :
    claimed ≠ shardedFold shards :=
  digest_convicts (laneDigest p) shards claimed h

end CIRISHolon.MergeLaw
