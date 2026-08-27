/-
CIRISHolon.Boundaries — the honest boundaries, CHALLENGED one by one.

TIERS.md names six boundaries. This file sorts them into what is absolute,
what is merely engineering, and what is completion debt — and mechanizes the
absoluteness claims where a theorem exists to state. The verdicts:

1. ARITHMETIC ENVELOPE (i128 refusal) — BREAKABLE. Absolute in KIND, not in
   LOCATION: `no_fixed_width_carrier` proves NO fixed-width representation
   faithfully carries the exact ring's coefficients (ℤ is infinite), so any
   fixed-width engine must refuse or widen — that much is forced. But the
   width 128 is pure engineering, and the envelope itself can be REMOVED:
   bignum promotion, or CRT residue arithmetic (the LinBox pattern).
   `digests_jointly_faithful` is the kernel of that second route: the mod-p
   lane digests — the SAME homomorphisms MergeLaw's certificate uses —
   jointly separate any two distinct lane vectors, so computing in enough
   residues and reconstructing loses nothing. The boundary-breaking
   mechanism and the corruption detector are one mechanism.

2. STATEVECTOR CAP (24 qubits) — the NUMBER is ignorable (a routing default;
   published statevector runs reach ~50 qubits on exascale machines). The
   LAW behind it is absolute and mechanized: `generic_state_table_absolute`
   — even distinguishing the 2^(2^n) zero/nonzero support patterns of an
   n-qubit amplitude table requires 2^n bits, by pigeonhole. No
   representation whatsoever beats it FOR GENERIC STATES. The whole tier
   ladder exists because physical circuits are not generic: Clifford
   structure, low T-count, and bounded entanglement are exactly the
   non-genericity the router detects — the cap is a wall only for the
   fallback tier, and reaching it means no structure was found.

3. ROUTER MAGIC CAP (t ≤ 12) — IGNORABLE as stated: a latency default, not
   a law (the mesh already ran t = 28 exactly). The law behind it —
   exponential cost in t — is NOT information-theoretically absolute: the
   wall itself is machine-checked (`Stabilizer.pullback_not_pauli` — the
   non-Clifford motion provably leaves the tableau view), but the PRICE is
   only conjecturally exponential. The best unconditional lower bound on
   stabilizer rank is LINEAR in t (Peleg–Shpilka–Volk, Quantum 6, 652
   (2022)) against a best upper bound of 2^{0.3963t} (Qassim–Pashayan–
   Gosset, Quantum 5, 606 (2021)): the gap is enormous and open. This
   boundary is the one our instrument might genuinely BEND — a certified
   exact search for better decompositions attacks the open problem itself.

4. SAMPLER SCOPE (t ≤ 8..10) — BENDABLE, pure algorithm: the O(branches²)
   Gram sum squares the branch cost; an orbit-aware Gram is the named
   milestone. No law of its own beyond boundary 3's.

5. NO-ADAPTIVITY — REMOVABLE, and now COMPLETION DEBT: tiers 0/1 are not
   done until the front-end EXCEEDS OpenQASM. Adaptive Clifford (mid-
   circuit Pauli measurement + feed-forward) stays inside the efficient
   tier by Aaronson–Gottesman §III — outcomes are deterministic or fair
   coins, and the update is a tableau operation (mechanization of the
   measurement-update closure is OWED, stated here so the debt has a name).
   Arbitrary-angle gates (OpenQASM's parameterized family) enter the exact
   engine by Ross–Selinger synthesis: approximate the angle ONCE, at the
   front door, by an exact Clifford+T word with T-count O(log(1/ε)) — the
   approximation step is explicit, isolated, and priced in the same T
   currency as everything else, so exactness discipline survives the full
   gate surface.

6. CI EXEMPTIONS — NO LAW AT ALL. Process debt: write the gates. Nothing
   here for Lean; listing it beside the others would launder a chore into
   a limit.
-/
import Mathlib.Data.ZMod.Basic
import CIRISHolon.MergeLaw
import CIRISHolon.Stabilizer

namespace CIRISHolon.Boundaries

open MergeLaw

/-! ### Boundary 1 — the envelope is absolute in kind … -/

/-- No fixed width carries the exact coefficients: there is no injection of
    ℤ into any finite alphabet, so EVERY fixed-width exact engine must refuse
    or widen somewhere. The width 128 is engineering; the existence of an
    envelope is not. -/
theorem no_fixed_width_carrier (k : ℕ) :
    ¬∃ f : ℤ → Fin k, Function.Injective f := by
  rintro ⟨f, hf⟩
  haveI : Finite ℤ := Finite.of_injective f hf
  exact not_finite ℤ

/-! … and breakable in location: the certificate's own homomorphisms are the
escape route. Distinct lane vectors are separated by SOME mod-p digest, so
residue arithmetic over enough primes carries the ring faithfully (CRT), with
`MergeLaw.digest_commutes` already proving each residue commutes with the
whole sharded fold. -/

/-- The mod-(p+1) lane digests jointly separate distinct lane vectors: for
    any two different aligned-lane values there is a modulus that tells them
    apart. Joint faithfulness of the certificate family — and the kernel of
    the CRT route that removes the width envelope entirely. -/
theorem digests_jointly_faithful {x y : Lanes} (hxy : x ≠ y) :
    ∃ p i, ((x i : ZMod (p + 1)) ≠ (y i : ZMod (p + 1))) := by
  obtain ⟨i, hi⟩ := Function.ne_iff.mp hxy
  refine ⟨(x i - y i).natAbs, i, fun h => ?_⟩
  have hd : x i - y i ≠ 0 := sub_ne_zero.mpr hi
  have hz : ((x i - y i : ℤ) : ZMod ((x i - y i).natAbs + 1)) = 0 := by
    push_cast [h]
    ring
  have hdvd : (((x i - y i).natAbs + 1 : ℕ) : ℤ) ∣ (x i - y i) :=
    (ZMod.intCast_zmod_eq_zero_iff_dvd _ _).mp hz
  have hdvd' : ((x i - y i).natAbs + 1) ∣ (x i - y i).natAbs := by
    have h2 := Int.natAbs_dvd_natAbs.mpr hdvd
    rwa [Int.natAbs_ofNat] at h2
  have hpos : 0 < (x i - y i).natAbs := Int.natAbs_pos.mpr hd
  exact absurd (Nat.le_of_dvd hpos hdvd') (by omega)

/-! ### Boundary 2 — the statevector law is absolute for generic states -/

/-- Pigeonhole at the fallback tier: an n-qubit amplitude table has 2^(2^n)
    zero/nonzero support patterns, so no encoding into fewer than 2^n bits
    can distinguish them — no representation beats exponential FOR GENERIC
    STATES. (The ladder exists because physical circuits are not generic;
    this theorem is why the fallback tier can never be rescued by cleverness
    alone.) -/
theorem generic_state_table_absolute {n m : ℕ} (h : m < 2 ^ n) :
    ¬∃ f : ((Fin n → Bool) → Bool) → (Fin m → Bool), Function.Injective f := by
  rintro ⟨f, hf⟩
  have hcard := Fintype.card_le_of_injective f hf
  simp only [Fintype.card_fun, Fintype.card_bool, Fintype.card_fin] at hcard
  exact absurd hcard (not_le.mpr (Nat.pow_lt_pow_right one_lt_two h))

/-! ### Boundary 3 — the wall is proved; the price is open

The non-closure wall is `CIRISHolon.Object.pullback_not_pauli`: the
rational 3-4-5 rotation provably carries the tableau view out of itself, so
magic is priced, not forbidden. What is NOT proved anywhere — here or in the
literature — is that the price must be exponential: linear lower bound
(PSV 2022) versus 2^{0.3963t} upper bound (QPG 2021). This boundary is a
LIVE open problem, and the honest Lean statement is the wall alone; a
mechanized exponential lower bound would be a major theorem, not an audit. -/

/-- Re-export of the wall, so this file's map from boundary to theorem is
    complete inside one namespace: the non-Clifford motion leaves the
    signed-Pauli view — the tableau tier ends where magic begins. -/
theorem magic_wall : ∀ i, CIRISHolon.Object.pauli i ≠
    CIRISHolon.Object.R345t * CIRISHolon.Object.pauli 1 *
      CIRISHolon.Object.R345 :=
  CIRISHolon.Object.pullback_not_pauli

end CIRISHolon.Boundaries
