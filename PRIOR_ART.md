# PRIOR ART — the adversarial sweep, and where the claim survives

*2026-08-27, six parallel lanes briefed to attack our own novelty claim.
Convergent art is a HIT, not a strike: every occupied axis below is cited
generously and strengthens the position by pinning exactly what remains.
Verification grades: lanes verified load-bearing claims against primary
sources (arXiv PDFs, shipped source code); the rank lane's numerical
verifications are banked at `conformance/srank/`. All six lanes are in.*

## The verdict, in one table

| claimed element | status | occupier / credit |
|---|---|---|
| Exact Z[ω] arithmetic for Clifford+T | **OCCUPIED — cite, never claim** | The ring theorem: Giles–Selinger 2013 (arXiv:1212.0506). The shipped library: Selinger's `newsynth` (`ZOmega`, bignum). **Our five-integer representation verbatim: SliQSim** (Tsai–Jiang–Jhang, DAC 2021, arXiv:2007.09304 — at tens of thousands of qubits on structured circuits). Exact algebraic DD entries: Niemann–Zulehner–Wille–Drechsler (DATE'19, TCAD'20). Exact ℤ[ζ₈] amplitudes via path sums: de Colnet et al. 2026 (arXiv:2605.29944). The scaling proof we lack: Quist–Coopmans–Laarman 2026 (arXiv:2602.17775, 2^t·poly bound) |
| Exact ring + stabilizer decomposition, together | **OCCUPIED** | **quizx** (Kissinger–van de Wetering, QST 7, 044001 (2022), arXiv:2109.01076): `Scalar4([Dyadic;4])` over D[e^{iπ/4}], with parallel branch evaluation as an implementation fact (no published parallelism/determinism claim) |
| Exact symbolic Clifford+T verification at scale | **OCCUPIED** | Amy's path sums / Feynman (arXiv:1805.06908): ~100 qubits, thousands of T — answers equivalence, not amplitudes; no machine-checked correctness of the tool itself |
| The exponent ladder (magic pricing) | **OCCUPIED, and imported** | BSS 2016 (6→7, α≤0.4679); Bravyi–Gosset 2016 (the simulator); **QPG 2021 α≤0.3963** (Quantum 5, 606 — the standing record, verified in `conformance/srank/verify_qpg.py`); Magic5FromCat (KvdW–Vilmart, TQC 2022) attains it at finite t, Apache-2.0 Rust over our ring; approximate/sampling (0.228) is a different category |
| Bit-identical shard invariance | **OCCUPIED as a property** | ReproBLAS (Demmel–Nguyen–Ahrens, TOMS 2020) states it verbatim for floats — the HARD version. For exact arithmetic it is a free corollary; we say "exactness buys it", never "we achieved it" |
| AC-merge-fold correctness argument | **OCCUPIED, textbook** | CRDTs (Shapiro–Preguiça–Baquero–Zawirski 2011), BSP (Valiant 1990), LVars (Kuper–Newton 2013/14), internal determinism (Blelloch et al. 2012) |
| poly(n) memory per branch | **OCCUPIED** | Inherent to the CH-form (BBCCGH, Quantum 3, 181 (2019)) |
| Parallel branch evaluation (single node) | **OCCUPIED** | quizx (multicore), Aer extended_stabilizer (OpenMP — see its issue #1932, a real non-reproducibility bug in exactly this path), Tsim (QuEra, GPU), SOFT (16×H800, shot-parallel), quEStab (ICS'26, multi-GPU — **UNREAD, must be read before any adjacent claim**), García 2026 (arXiv:2607.27075 — our coordinator/merge design in prose, distribution left as future work; watch it) |
| **Multi-node distributed exact branch sums, real cross-shard merge** | **THIN — contingent on reading quEStab** | Nearest art: tensor-network slice sums (Alibaba/Pan–Zhang/Sunway) — float folds, only approximately associative; pre-empt, don't ignore |
| **Corruption certificates on simulation shards** | **OPEN in quantum** | Machinery ancestry to credit: certifying algorithms (McConnell–Mehlhorn–Näher–Schweitzer, CSR 2011), ABFT (Huang–Abraham 1984), Freivalds 1979, BOINC (whose redundancy exists BECAUSE floats aren't reproducible — the thing exactness makes unnecessary). QCIVET audits a single-device pipeline, not shards |
| **Machine-checked simulator kernel** | **VACANT** | No verified Gottesman–Knill anywhere. Nearest: Pauli-algebra formalizations (Feng et al. CoqPL 2025; Lean-QEC arXiv:2605.16523 — our closest structural precedent and natural counterparty; cite prominently). VOQC/SQIR own verified TRANSFORMATIONS + extraction discipline (the gap a hostile referee probes first: we have no verified path Lean→kernel; never say "verified simulator") |
| Certificate-carrying reproducibility standard | **VACANT, standards-shaped** | Reproducibility crisis documented: 24.4% of QC papers ship code, 64.5% of that fails to run (arXiv:2607.08348). No bit-reproducible or certificate-carrying simulation standard exists. IBM+Algorithmiq (July 2026) frame "trusted quantum computation" statistically and admit conflicting classical predictions with no adjudication — the incumbent stating our problem |
| One engine WASM→cluster | **OPEN, product not research** | QuEST spans laptop→HPC (no browser); Qrack browser→GPU (no cluster); Maestro orchestrates engines rather than being one |

## What survives, exactly

The claim is the **conjunction plus the certificates plus the theorems**:
exact amplitudes with no coefficient envelope (the CRT carrier), distributed
with a machine-checked merge law (`MergeLaw.lean` — the move no cited system
has made), two-directional corruption certificates (convictions never wrong;
misses impossible on the design window), a refusal boundary instead of
silent approximation, and kernel theorems in Lean about the tier structure.
Each ingredient has an owner; the assembly has none. Three attacks and their
pre-empts are recorded in the sweep; the three pre-publication obligations
stand: **read quEStab (DOI 10.1145/3797905.3816723), watch García
arXiv:2607.27075, cite Aer issue #1932 by number.**

## Corrections the sweep forced on us (kept, marked)

- ω = e^{iπ/4} = ζ₈ (degree 4 over ℚ); an early brief said "e^{iπ/8}" — a
  reviewer-visible error, fixed.
- Our measured 0.500 exponent was BEHIND published SOTA (0.4679 since 2016,
  0.3963 since 2021) and is stated as such in TIERS.md.
- The unchecked-i128 exactness bug (two independent reviews converged on it)
  — fixed by refusal, then dissolved by the residue carrier.
- The 6→7 decomposition is Bravyi–SMITH–SMOLIN, not Bravyi–Gosset.
- Labib–Russo 2026 matches (not beats) the qubit exponent, on the face-state
  orbit (twice-verified: `conformance/srank/verify.py`).
- "Zero false positives" was one-directional as first stated; the no-miss
  direction is now proved on the design window (`digest_window_faithful`) —
  outside the window or on subset digests, only probabilistic, and writeups
  must say so.

## The campaigns this sweep opened

Ranked with referees and kills in `CAMPAIGNS.md`: the QPG linear-code
exponent hunt (χ(cat₈) ≤ 5?), the IBM tracker fstate instance, the
stabilizer-fidelity multiplicativity counterexample, the 13-year 3-qubit
T-count frontier, and the verified streaming LRAT checker.


## The gauge/TN lane (appended)

| claimed element | status | occupier / credit |
|---|---|---|
| Schwinger-model DMRG/MPS precision | **OCCUPIED, and it is the referee** | Byrnes et al. (PRD 66, 013002: M_V/g=0.56419(4), N≤256 — and their verdict that the Coulomb tail is "no impediment to DMRG"); Bañuls–Cichy–Jansen–Cirac (JHEP 11 (2013) 158); Buyens et al. (arXiv:1411.0020 — condensate to 7 figures); Dempsey et al. (PRR 4, 043133 — the exact mass shift); Arguello Cruz et al. (arXiv:2412.01902 — (m/g)_c to 6 digits at 3000 qubits, and the one published production-cost figure, App. C) |
| **Bit-reproducible DMRG in tensor networks** | **UNCLAIMED** | Zero determinism goals anywhere in TN (ITensor declares it out of scope, ITensors.jl #1699, with non-last-digit divergence quoted). Prior stake exists IN LATTICE QCD: QUDA `QUDA_DETERMINISTIC_REDUCE`, Kate Clark Lattice 2023; Feltor (arXiv:1807.01971) for bitwise-reproducible fluid codes — credit all three, claim TN only |
| Determinism-is-cheap (the synthesis) | **NEW — four legs independently sourced, assembly ours** | gauge invariance forces block-sparse; block-sparse gains only 1.44× from 8 BLAS threads; White's noise term and Hubig's expansion are RNG-free; the fastest published Schwinger DMRG already discards the random initial MPS for convergence. Bit-determinism costs ≲1.4× and surrenders nothing the SOTA wants |
| LGT-specific TN engine | **VACANT** | The canon ran on unnamed in-house codes, then generic ITensor. No incumbent |
| Exact-arithmetic gauge-sector bookkeeping | **VACANT (arithmetic), occupied (quantum numbers)** | Integer quantum numbers standard (Singh–Pfeifer–Vidal); nearest precedents: Rico et al. (arXiv:1312.3127, literal integer MPS matrices carrying the Gauss constraint), LSH integer charges (Raychowdhury–Stryker) |
| Free open question we'd answer in passing | — | whether a Gauss-law penalty term ill-conditions DMRG: grepped across the corpus, no coefficient ever published |

Traps staked by the lane: the 2-flavor factor-2.19 convention gap (do not
stake that rung); the M_S terminology collision (Dempsey's M_S = vector,
Bañuls's M_S = scalar); scalar errors 10–30× vector; "reproducibility" in
this community means cross-method physical agreement, not bitwise — define
our sense explicitly or be misread.
