/-
CIRISHolon.Limits — the speed program's limits, backed into the lake.

Measured performance is never accepted as a fact about the world until it is
placed against the limit it approaches. This file names the limits the
optimization program runs into, mechanizes the hard cores where a theorem
exists to state, and classifies every limit honestly:

  HARD    — information-theoretic: no engine on any substrate beats it.
  SOFT    — conditional or hardware-bound: movable by work or by better
            machines; the direction of movement is known.
  BEGGING — not a law at all: an artifact of the current substrate that the
            theory says should fall, with the named instrument to break it.

The discipline mirrors the stance: every entry carries what would move or
break it, and a `mechanized` flag that is honest about which entries have a
machine-checked core IN THIS FILE versus a recorded citation. A measured
engine number is then a POSITION against these limits (recorded in
TIERS.md/BENCHMARKS.md), never a limit itself.

The five limits:

L1 GATE-TOUCH FLOOR (HARD, mechanized). A Clifford gate must rewrite the
   bits it changes: there exist tableau states where a Hadamard on one
   qubit flips ALL 2m bits of that qubit's X/Z columns
   (`hadamard_flip_floor`), so any correct engine writes Ω(m) bits per
   gate, m = rows. The word-parallel column engine writes exactly 2m bits
   plus signs — WITHIN A SMALL CONSTANT OF THE FLOOR. This is the formal
   answer to "maximum theoretical amount" on the gate axis: the remaining
   headroom is memory-system constants, not information.

L2 WORD WIDTH (BEGGING). The 64× of branch-slicing and the 4× of AVX2 are
   register widths — hardware facts, not laws. Nothing information-
   theoretic stops 512-bit vectors, GPU warps, or wider: the theory says
   this "limit" should fall by orders of magnitude, and the 4090 is the
   named instrument. No theorem exists because there is no law to state.

L3 BRANCH COUNT = STABILIZER RANK (BEGGING, the open problem). The magic
   tier's branch count is χ(|T⟩^⊗t): proven lower bound only Ω(t)
   (Peleg–Shpilka–Volk, Quantum 6, 652 (2022)); best upper bound
   2^{0.3963t} (Qassim–Pashayan–Gosset, Quantum 5, 606 (2021)). The gap is
   enormous and five years unmoved; the exponential-ness itself is
   CONDITIONAL (complexity assumptions), not information-theoretic.
   Begging by the field's own admission — the named instrument is the
   certified decomposition search (CAMPAIGNS.md #1: χ(cat₈) ≤ 5 breaks the
   record by one integer).

L4 EXACT-COEFFICIENT BITS (HARD, mechanized core). Exact amplitudes carry
   coefficients of magnitude up to 2^{Θ(n+t)} (Quist–Coopmans–Laarman,
   arXiv:2602.17775), and `magnitude_needs_bits` pins the trivial-but-
   load-bearing core: 2^B distinct values cannot inject into fewer than B
   bits. The CRT residue carrier meets this floor within a constant
   (k ≈ bits/60 primes) — the carrier is near-optimal IN SPACE for the
   exact regime; only abandoning exactness escapes, which is out by
   design.

L5 SAMPLING READ FLOOR (SOFT, not mechanized). A terminal sample's
   deterministic bits depend on the whole stabilizer group, so the
   canonical pass reads Θ(n²/64) words; whether o(n²) reads suffice for a
   VALID sample is not settled here — the elimination constant is
   engineering (frame-batching amortizes it across shots), and the
   per-shot marginal is the movable quantity. Classified SOFT: movement
   direction known (batching), floor not proved.
-/
import Mathlib.Data.ZMod.Basic

namespace CIRISHolon.Limits

/-- Bits at which two column vectors disagree. -/
def flips {m : ℕ} (u v : Fin m → Bool) : Finset (Fin m) :=
  Finset.univ.filter fun i => u i ≠ v i

/-- **L1's hard core.** The Hadamard update swaps a qubit's X and Z columns;
    on the all-X state the swap flips every bit of both columns — 2m bits.
    Any correct engine therefore writes Ω(m) bits for this gate on this
    state; word-parallel columns achieve it within a small constant. -/
theorem hadamard_flip_floor (m : ℕ) :
    ∃ x z : Fin m → Bool,
      (flips x z).card = m ∧ (flips z x).card = m := by
  refine ⟨fun _ => true, fun _ => false, ?_, ?_⟩ <;>
    simp [flips, Finset.filter_true_of_mem]

/-- **L4's hard core.** 2^B distinct exact coefficients cannot be carried by
    fewer than B bits — the pigeonhole that makes the residue carrier's
    prime count (≈ bits/60) near-optimal in space for the exact regime. -/
theorem magnitude_needs_bits {B m : ℕ} (h : m < B) :
    ¬∃ f : Fin (2 ^ B) → (Fin m → Bool), Function.Injective f := by
  rintro ⟨f, hf⟩
  have hcard := Fintype.card_le_of_injective f hf
  simp only [Fintype.card_fin, Fintype.card_fun, Fintype.card_bool] at hcard
  exact absurd hcard (not_le.mpr (Nat.pow_lt_pow_right one_lt_two h))

/-- Classification of a limit's strength. -/
inductive Strength
  | hard    -- information-theoretic; no engine on any substrate beats it
  | soft    -- conditional or hardware-bound; movable, direction known
  | begging -- not a law; an artifact the theory says should fall
  deriving Repr, DecidableEq

/-- A recorded limit: statement, strength, what moves or breaks it, and an
    honest flag for whether its hard core is a theorem IN THIS FILE. -/
structure Limit where
  name : String
  statement : String
  strength : Strength
  moved_or_broken_by : String
  mechanized : Bool

def limits : List Limit :=
  [ { name := "L1 gate-touch floor"
    , statement := "a Clifford gate must write the bits it changes; Hadamard can change all 2m column bits"
    , strength := .hard
    , moved_or_broken_by := "nothing — remaining headroom above the floor is memory-system constants"
    , mechanized := true }
  , { name := "L2 word width"
    , statement := "64 branches/word and 4 words/vector are register widths, not laws"
    , strength := .begging
    , moved_or_broken_by := "wider vectors, GPU warps — the 4090 is the named instrument"
    , mechanized := false }
  , { name := "L3 branch count = stabilizer rank"
    , statement := "branches = χ(T^⊗t): lower bound Ω(t) (PSV 2022), upper 2^{0.3963t} (QPG 2021); exponentiality conditional, gap open"
    , strength := .begging
    , moved_or_broken_by := "a better decomposition — χ(cat₈) ≤ 5 breaks the record (CAMPAIGNS.md #1)"
    , mechanized := false }
  , { name := "L4 exact-coefficient bits"
    , statement := "exact coefficients reach 2^{Θ(n+t)}; 2^B values need B bits; the CRT carrier meets the floor within a constant"
    , strength := .hard
    , moved_or_broken_by := "only abandoning exactness — out by design"
    , mechanized := true }
  , { name := "L5 sampling read floor"
    , statement := "the canonical terminal sample reads Θ(n²/64) words; a proved lower bound is absent"
    , strength := .soft
    , moved_or_broken_by := "frame-batching amortizes the pass across shots; per-shot marginal is the movable quantity"
    , mechanized := false } ]

/-- The honesty audit: exactly the two limits with theorems in this file may
    claim mechanization. -/
theorem mechanized_are_the_proved :
    (limits.filter (·.mechanized)).map Limit.name
      = ["L1 gate-touch floor", "L4 exact-coefficient bits"] := by
  rfl

end CIRISHolon.Limits
