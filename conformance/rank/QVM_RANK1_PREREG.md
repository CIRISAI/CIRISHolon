# QVM-RANK-1 — six stabilizer terms for seven copies of the H state? PREREGISTRATION

*Frozen 2026-09-24, committed alone before any code. The open cell, as the Unitary
Foundation's `stabrank` project states it after closing `χ(|H⟩^{⊗6}) = 6` on 2026-09-22 (PR 89,
a rank-5 exclusion over 5,939,465 covers, 73 CPU-hours, attested): **is there a six-term
stabilizer decomposition of `|H⟩^{⊗7}`?** A witness would give `χ(|H⟩^{⊗7k}) ≤ 6^k`, a per-copy
exponent `log₂6 / 7 ≈ 0.369`, below the published `log₂3 / 4 ≈ 0.396` (Kissinger–van de
Wetering–Vilmart 2022; Qassim–Pashayan–Gosset 2021; Bravyi–Gosset 2016) that this engine's
magic tier ports. This is a SHOT, not a symmetric test: a witness is decisive and
checkable by others; its absence within budget proves nothing, since a rank-6 exclusion at
seven copies is far beyond the six-copy exclusion's cost. The stakes are written so that
either outcome is banked honestly.*

## 1. The object and the referee

`|H⟩ = cos(π/8)|0⟩ + sin(π/8)|1⟩`, the `+1` eigenstate of the Hadamard. A witness is a set of six
seven-qubit stabilizer states `{s₁…s₆}` and coefficients `c_i` in the ring `ℤ[ζ₈, 1/2]` (the
engine's `Cyc`, with `√2 = ζ₈ + ζ₈⁻¹`) such that `Σ c_i s_i = |H⟩^{⊗7}` EXACTLY, with each `s_i`
given by its stabilizer tableau (or its affine-form amplitude description). **The referee is
stabrank's own verifier** (numerical at machine precision, exact rational in SymPy, and its
Lean 4 route where its format admits the witness) — not ours. A witness that our exact ring
accepts and their verifier does not is NOT a witness; the disagreement is the finding.

## 2. Mechanisms, declared (the engine's best, and the field's)

- **Exact residuals only.** Every candidate span is tested in `Cyc`; no floating-point
  acceptance anywhere on the accepting path. Floats may PRUNE (a numerically far residual
  is discarded), never accept.
- **The cheap part first (OBJECT.md's procedure).** A six-term witness is a five-term
  partial span plus a residual that is itself a stabilizer state. The search therefore
  enumerates five-tuples (and four-tuples with a rank-≤2 residual) and applies an EXACT
  stabilizer-ness test to the projected residual (affine support with a quadratic phase,
  read off the amplitudes in the ring) — a test the tableau tier makes cheap. The
  combinatorial part is over fewer states; the arithmetic part is exact.
- **Symmetry reduction, declared.** The stabilizer of `|H⟩^{⊗7}` in the Clifford group
  contains `S₇` (qubit permutations) and `H^{⊗k}` on any subset (`|H⟩` is `H`-fixed): a group
  of order `7! · 2⁷ = 645,120`. Candidate tuples are enumerated up to this group; the orbit
  representatives are counted and the count is printed and checked against a brute count
  at `m = 3, 4`.
- **The field's pruning, credited.** stabrank PR 87's property (every term's direction space
  has dual distance `≥ 3`) and PR 16's Galois constraint on decomposition spans are applied
  as filters on the dictionary, each with a plant showing it keeps every known witness.
- **Seeds.** The magic tier's own decomposition (`magic5.rs`), the known rank-4 witness of
  `|H⟩^{⊗4}` and stabrank's rank-6 witness of `|H⟩^{⊗6}` are seeds for a local search
  (annealing with Pauli, Clifford and single-state moves, as stabrank does) with EXACT
  acceptance; the exhaustive branch runs beside it.
- **The GPU fold and the mesh** evaluate residual batches bit-identically to a CPU twin
  (`holon-gpu`'s i128 lanes; `QVM_GPUFOLD1_RESULTS.md`); the device class is declared.

## 3. Stakes

- **G1 — a witness.** Six seven-qubit stabilizer states and ring coefficients whose exact
  sum is `|H⟩^{⊗7}`, accepted by stabrank's verifier. **This is the shot; there is no kill on
  its absence.**
- **G2 — the controls, before any seven-copy search is read.** The same pipeline, told
  nothing, recovers a rank-4 witness of `|H⟩^{⊗4}` and a rank-6 witness of `|H⟩^{⊗6}` (any
  member of their symmetry classes), each accepted by the referee. **Kill:** either control
  not recovered within an hour — the pipeline cannot find what exists, and nothing at
  seven copies is read.
- **G3 — coverage, the honest alternative result.** If no witness lands in the budget, the
  campaign reports EXACTLY what was exhausted: the enumeration's orbit count, the
  fraction of the symmetry-reduced five-tuple space decided, the filters applied and the
  count each removed, and the annealing's move budget — a partial exclusion certificate
  in stabrank's terms, submitted to them as such if it is a form they take.
- **G4 — bit-identity of the device path.** Every residual batch evaluated on the GPU equals
  the CPU twin's struct; a corrupted lane is convicted.

## 4. Plants

| plant | must |
|---|---|
| **PR-1** | a witness perturbed by one ring unit in one coefficient is REJECTED by the exact residual and ACCEPTED by a float check at `10⁻¹²` — the exact path is load-bearing |
| **PR-2** | the exact stabilizer-ness test accepts every state of the three-qubit dictionary (1,080 states) and rejects `|H⟩^{⊗3}` and 1,000 random non-stabilizer states |
| **PR-3** | the symmetry-reduced enumeration's orbit counts at `m = 3, 4` equal the brute counts over the full group |
| **PR-4** | each filter (dual distance, Galois) keeps every term of every known witness at `m = 4, 6` |
| **PR-5** | the GPU twin: a flipped lane convicted by position |

## 5. Budget and placement

Two days of wall: cores 21–27 and the RTX 4090. Cores 0–19 carry running campaigns and are
not touched. The exhaustive branch is checkpointed so the coverage in G3 is exact whenever
it is stopped.

## 6. Branches

- **(a)** G1 met → the witness is submitted to `stabrank` with its exact certificate; the
  exponent `0.369` is theirs to attest; banked here only after their verifier accepts it.
- **(b)** G1 not met in budget → G3's coverage is the result; banked as a partial exclusion,
  no claim made about the rank.
- **(c)** G2 killed → the pipeline is wrong; nothing at seven copies is read.
- **(d)** a witness our ring accepts and the referee rejects → the disagreement is the
  finding and is resolved before anything is claimed.

## 7. What this does not test

Any other magic state or orbit; the qutrit cells; a lower bound at seven copies.

---
witness: `lanes_shardedFold_invariant` (the fold's schedule-independence, `holon-gpu`) for G4; the stabilizer-ness test's correctness is `tableau_closed_under_hadamard`'s engineering face (Stabilizer.lean) — the amplitude form of a stabilizer state; G1–G3 are computational
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-CHEAPER-THAN-ITS-PRICE, M-DEVICE-CLASS, M-PLACEMENT-LOTTERY, M-VACUOUS-SUCCESS, M-VALIDATED-NOT-WIRED, M-PARITY-PROTECT, M-HOMOG, M-FLOOR-UNSTAKED, M-COND-PROBE, M-STALE-INSTRUMENT, M-IDLE-CALIBRATED-TIMEOUT, M-MAINTENANCE-LENS — contacted by keyword, cited.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier (a perturbed witness, the three-qubit dictionary, the m = 3, 4 enumerations, the known witnesses, a flipped lane), and the sector the plant acts on is nonzero in that carrier by construction.
