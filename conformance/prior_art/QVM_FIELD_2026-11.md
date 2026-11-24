# The QVM's field in 2026, screened against this record

*2026-11-23. Asked for by the lead: "pursue the QVM, major advances there also we need to look
at and benchmark against." Method: each item read at its primary source (the arXiv abstract
through the arXiv API, and the full text for the nine that bear hardest: 2607.27075, 2605.29944,
2608.24370, 2607.28600, 2604.27058, 2603.14641, 2611.06119, 2610.17500, 2604.01059, plus
2507.03092); each code repository's licence and last push read from GitHub on the same day;
the stabilizer-rank board read at its head (`unitaryfoundation/stabrank` `7d0cea2`, 2026-11-23).
**Nothing was built, run or timed.** Every number below is the source's own, with its
conditions, and none has been reproduced here. Verification status is the source's unless a
line says otherwise. Read beside `PRIOR_ART.md` (the 2026-08-27 sweep), which already credits
quizx, Tsim, Aer, de Colnet et al. and García for other rows; this screen adds what the sweep
did not have, and re-reads two of its entries against claims made after it.*

**Verdict words.** PRIOR ART: someone published a claim this record makes; the file that
must change is named. REFEREE: a tool we should grade against for correctness. BASELINE: a
number we must beat or concede to under matched conditions. NOT RELEVANT: read, does not touch
a view.

## 0. The three findings that change the QVM's standing

1. **The dispatch has direct prior art.** García, arXiv 2607.27075 (2026-07-29), eight weeks
   before ACUITY-2 was frozen (2026-09-23). §1.
2. **"Parity with stim on one core" is parity on a workload the field no longer argues about,
   and the field has moved past stim on it.** §2.
3. **The rank front's referee has moved more than seventy commits and the cell is still open; our two
   exclusions and our second six-copy class were never sent.** §4.

## 1. The dispatch ("picks the view unlabelled") — PRIOR ART

| | García 2607.27075 ("quipu-cut") | this record (`QVM_ACUITY2_RESULTS.md`) |
|---|---|---|
| views | four leaf backends: stabilizer frame, dense vector, Clifford tableau + bond-`χ` MPS, Bravyi–Gosset stabilizer rank | tableau, stabilizer-rank sum, certified MPS, dense |
| rule | "adaptively pick the cheapest backend whose predicted error fits" a per-leaf tolerance; a one-pass structural scan routes each leaf; a runtime check switches representation mid-computation | "runs every closure test, prices every closed view, and SELECTS the cheapest whose certificate reaches the acuity" |
| error | frame and dense exact; MPS "discarded SVD norm"; rank "binomial tail"; an additive bound over leaves stated as holding for any assignment of leaves to backends | tableau and dense exact; MPS `Σ√w`; the sum a certified remainder |
| also | a recursive spatial cut, so one circuit can use different backends on different parts; a cost model calibrated on the target machine | none: one view per circuit |
| measured | 571 cells, `n ≤ 32` for whole-circuit backends; against Aer statevector a 97.4 % win rate and `18.3 ×` geometric mean, 90 cells Aer did not finish; top-level picks frame 211, dense 170, MPS 53, rank never; intra-circuit mixing "never happens" at corpus scale and is exercised on built suites (19.3 % of 405; 93 % of the mismatched-halves cells at `n ∈ {36, 40, 44}`) | 80 instances, `n ≤ 20`; the picked view within `2 ×` of the hand-picked one on 77 of 80; certificate held 80 of 80; the search itself `112 ×` stim on the Clifford family |
| conditions | Apple M2 Pro for the calibration constants; single random trial per cell on the adversarial table; code "available at the project repository" (the link did not survive the HTML; **not found, licence not read**) | cores 21–27, load about 9.5 |

**Verdict: PRIOR ART** for the architecture — a dictionary of the same four representations,
selected per circuit by price under an error budget, published first, at larger `n`, with a
finer grain (per leaf) and a worst-case guarantee ("never asymptotically worse than direct
dense simulation") that this record does not have. `PRIOR_ART.md` already names the paper,
but only on the parallel-branch row ("our coordinator/merge design in prose ... watch it").
It was not read against the dispatch, and ACUITY-2's prereg says "That is what today's state
of the art does by hand". **That sentence was false when frozen.**

What is left to this record, stated narrowly: (i) the selection is by a closure TEST on the
circuit (a probe with a certificate), where García's is by structural predicates and a cost
model; (ii) the selection was graded against the hand-picked best on unlabelled instances,
with plants, which García's paper does not report (it reports wins against other
simulators, not how often the selector picked the best of its own four); (iii) the
certificate was checked against a referee on every instance. None of these is a claim to
the idea. And on (i) García's route is the cheaper one: a structural scan costs nothing next
to a probe that runs MPS before giving up, which is where the `112 ×` comes from.

Also bearing on the same row: **de Colnet et al., 2605.29944** (already credited in
`PRIOR_ART.md` for exact `ℤ[ζ₈]` amplitudes). Its second half is a mixed price inside one
algorithm: "Each subtree runs at the width price of its cut-ranks or at a magic price that
discharges the non-Clifford phases below it", polynomial on mixed families where pure
rank-width and pure T-count are both exponential. That is a per-subtree choice between a
tensor-like view and a stabilizer-rank view, with a proved bound `O(4^k poly(n))`. **PRIOR
ART for any sentence that says choosing between a width view and a magic view by price is
new.** (Kuyanov and Kissinger, 2026, on low-rank-width circuits is cited beside it in
2611.06119's references; title only, not read.)

**Owed:** a dated CORRECTION under ACUITY-2's prereg sentence and in `TIERS.md` §2's dispatch
row; `PRIOR_ART.md` gains a row for the dispatch crediting García and de Colnet; García's
code, if it can be found and its licence allows running, becomes the BASELINE for the
dispatch (§5, arm D).

## 2. The stabilizer tableau — BASELINES we have not met

What the record holds: single-shot tableau simulation of a rotated surface code against
`stim` 1.16.0's tableau simulator. One core at parity at `d = 221` (`31.08` s against
`35.78` s); eight cores `0.82 ×` one core, KILLED as staked; a re-aim at `0.41 ×` on a table
the load gate refused (`conformance/bigqvm/MESH_CLIFFORD_RESULTS.md`). **The record holds no
many-shot sampling number at all** (no shots/s anywhere in `BENCHMARKS.md`, `bigqvm/` or
`qasm/`), and many-shot sampling is the workload every paper below reports.

| item | claim, with its conditions | status | code licence | verdict |
|---|---|---|---|---|
| **SymFT**, 2607.28600 (Fang, Lou, Li; 2026-07-30) | pure-Clifford rotated surface-code memory, ONE logical processor of a Xeon Gold 5218R pinned with its sibling idle: `2.06 M` shots/s at `d = 7` (`2.52 ×` Stim, `22.05 ×` Clifft, `1.68 ×` Tsim); `899.20 k` shots/s at `d = 9` (`2.56 ×` Stim). Near-Clifford: `1.86–3.51 ×` Clifft on cultivation and distillation | preprint, self-reported; test circuits published (`haoliri0/SymFT_Test`) | predecessor `haoliri0/SOFT` Apache-2.0; SymFT's own repository not located | **BASELINE.** The first claim of beating Stim on Stim's own ground on one core. Our "parity" is against the simulator this paper says it beats `2.5 ×`, on a different task (single-shot tableau against compiled sampling). We have no sampler to enter |
| **QuaSARQ**, 2603.14641 (Osama, Thanos, Laarman; accepted in Quantum, 2026-10-01) | GPU tableau and Pauli-frame sampler on an **RTX 4090** (24 GB), random Clifford circuits, 1,000 to 180,000 qubits, depth to 1,000; completes 177 heavy-suite circuits in 72 h where Stim (EPYC 7282 nodes, 256 GB) completes 125 in 132 h; gate application `105 ×` Stim at 12,000 qubits and depth 100 (`8 ms` against `843 ms`: the gate kernel alone, not end to end); "over 80 %" energy reduction | peer-reviewed venue; self-reported numbers | **GPL-3.0**: may be RUN as an external referee, may not be linked or vendored | **BASELINE we likely lose to**, on our own card class (this box's GPU is an RTX 4090 Laptop). `d = 221` is about 98 thousand qubits, the middle of their range. `holon-gpu` has a fold for the rank sum and NO tableau path; the tableau row's GPU column is empty |
| **STABSim**, 2507.03092 (Garner et al., PNNL) | GPU tableau; "starts outperforming Stim at distance 11 in the planar code"; EPYC 7502 node against an H100 | preprint | in `pnnl/NWQ-Sim`, MIT | **BASELINE** at `d ≥ 11`; the same point as QuaSARQ on a larger card |
| **Clifft**, 2604.27058 (Chase, Labib; Unitary Foundation) | near-Clifford, compile once and sample many; cost set by the peak active dimension `k_max`, not `n` or `t`; "Stim retains a roughly `10 ×` throughput advantage" on pure Clifford; first exact end-to-end magic state cultivation with escape (463 qubits) over hundreds of billions of shots; 16 physical cores of an AWS c8i.8xlarge, median of 3 | preprint; active repository | Apache-2.0 | **BASELINE** for the rank-sum row on fault-tolerance circuits, and a **REFEREE** (exact, Stim-format input). Its regime (`n` in the hundreds, `k_max` modest) is one the sum view prices at `2^{0.396 t}` and Clifft prices at `2^{k_max}`: a different closed view of the same circuit that our dictionary does not contain |
| **Tsim**, 2604.01059 (Haenel, Luo, Zhao; QuEra) | ZX diagrams with parameterised Pauli channels, compiled for vectorised GPU sampling; linear in Clifford gates, exponential only in non-Clifford; `2.4 µs` a shot on `d = 3` cultivation on a GH200; cannot yet do `d = 5` cultivation | preprint; already credited in `PRIOR_ART.md` | Apache-2.0 (`rafaelha/tsim-bench` has NO licence: read only) | **BASELINE** for the GPU fold: the same design point (stabilizer terms folded on a card), with sampling |
| **T-by-T sampling**, 2611.06119 (Koch; 2026-11-04) | exact Clifford+T SAMPLING from `2t` amplitude evaluations, independent of the Clifford structure; cultivation with escape to distance 19 "in a couple of seconds" a sample, one core of a consumer laptop, on quizx's `α ≈ 0.396` decomposition | preprint | `mark-koch/t-by-t-sampling`: **no licence**, read only | **PRIOR ART to read before any sampling claim.** ACUITY-2 found "a marginal costs the sum `2^{|L|−4}` legs" and no marginal ever picked the sum. This paper removes the marginals. The algorithm is published and can be implemented from the paper; the code cannot be copied |

## 3. Benchmarks and referees

| item | what it is | licence | verdict |
|---|---|---|---|
| **ABSTRACTS**, 2608.24370 (Sutcliffe, van de Wetering; 2026-08-25), v0.1 | a fixed circuit set by class (random Clifford+T, Toffoli/CCZ-decomposed structured circuits, more planned), STRONG simulation only (one amplitude), run on fixed virtual hardware: AWS t3.small, **2 vCPU, 2 GiB**, Ubuntu 26.04, a 5-minute limit a circuit and 3 hours a simulator; records amplitude, wall, CPU time, peak memory; simulators in v0.1 include PyZX BSS, PyZX Magic Cat, quimb, Qiskit Aer statevector and a GPU parametric-ZX pipeline; a public dashboard; open to submissions; states it is "not ... a linear leaderboard" but a way to compare scaling | MIT (repository, circuits, results) | **the suite to benchmark against**, and a REFEREE: its amplitudes are a public answer key. It fits what our views compute (one amplitude). Its 2 GiB ceiling also fits today's box rule. A public submission is the lead's decision |
| **stabrank** (`unitaryfoundation/stabrank`) | the rank board and its verifier | Apache-2.0 | REFEREE already (pinned at `2ceb676`); see §4 |
| **quizx**, **PyZX**, **Stim**, **Qiskit Aer**, **cotengra** | the hand-picked methods | Apache-2.0 each | REFEREES and BASELINES, as before |
| **quimb** | tensor-network contraction | Apache-2.0 by its `LICENSE.txt` (GitHub's detector says "Other") | BASELINE for the MPS row; our MPS has only been compared with itself at `χ = 64` |
| **ArcTN**, 2610.31554; learning-to-rank contraction plans, 2608.05819 | contraction-order planners; ArcTN claims 61.32 % fewer FLOPs than cotengra's default on 30 networks | from a search summary only; **not read at source** | NOT RELEVANT until the dictionary has a general tensor-contraction view; it has an MPS only |
| **quEStab** (ICS'26, DOI 10.1145/3797905.3816723) | multi-GPU stabilizer simulation | **STILL UNREAD.** `PRIOR_ART.md` has said since 2026-08-27 that it "must be read before any adjacent claim"; a search on 2026-11-23 did not find it | the multi-node row of `PRIOR_ART.md` stays "contingent" and GPUFOLD-1 is an adjacent claim |

## 4. The stabilizer rank

| item | claim | status | verdict |
|---|---|---|---|
| **Bevins**, 2610.17500 (2026-10-12) | `γ_T ≤ 3/8` for the qubit FACE-CENTER magic state: two fifteen-term identities give 24 terms at twelve copies, then eight copies for eight terms each (`3·8^j` at `4 + 8j`). Says of itself: "does not determine the optimal exponent, establish a lower bound, or guarantee a particular simulation runtime. We also do not separate T-type and H-type orbit exponents." Results "emerged primarily from AI-assisted research" (Claude Opus 5.5 and a local Qwen), verified by the author | v1 preprint; no Lean; NOT on the stabrank board (its T-type cells stop at ten copies) | **Not a baseline for the sum view.** The sum splits on the `π/8` gate's state, the H-type orbit; this record already holds the trap exactly (`conformance/srank/verify.py`: a face-state identity is WRONG for the `π/8` state, fidelity 0.8249). `GANTT2.md` line 1522 lists it as "read before any adjacent claim"; this is that read. **Owed:** `PRIOR_ART.md`'s exponent row says "QPG 2021 α≤0.3963 ... the standing record" without naming the orbit. For the face orbit the record is now `0.375`, unverified; for the H-type orbit it is unchanged. One clause. And a cheap referee act: check the 24-term identity exactly in our ring at twelve copies, the same check `verify.py` made of Labib–Russo |
| **Sinha and Lovitz**, 2610.06819 (2026-10-05) | stabilizer rank `≤ k` implies extent `≤ 2^{O(√(k log(k+1)))}`; an `Ω(n²/log n)` lower bound on the APPROXIMATE stabilizer rank of tensor powers of any non-stabilizer qubit state | preprint; not in this repository until today | **PRIOR ART for the fold's pricing.** GF1 measured an extensive magic density and prices a box at `2^{M₂}`; this is a theorem relating the two measures the QVM prices by (rank for the exact sum, extent for sampling). To be cited where `TIERS.md` §1 prices the fold, after the paper's hypotheses are read |
| **Labib and Russo**, 2605.28586; **Kalra and Sinha**, Quantum 2026-07-29 (arXiv 2503.04101) | qutrit orbit bounds and a four-copy qubit T-type identity matching `0.396`; a rank-to-fidelity lower bound and the Barnes–Wall norm | the first is in `conformance/srank/MAGIC5.md`; the second is improved on by Sinha–Lovitz, by that paper's own account | no change |
| **Wan and Zhong**, magic state cultivation with few Clifford terms | named in Clifft's and Koch's references | **title only, not read** | to be read before the sum view is run on cultivation circuits |
| **The stabrank board since `2ceb676`** (2026-09-24 → `7d0cea2`, 2026-11-23) | more than seventy commits: `χ(|H⟩^⊗9) ≤ 17`, down from 18 (Kuyanov, a "Codex-assisted" block-symmetric search, 2026-11-15); `χ(cat₇) ≥ 6` by an exact certificate (2026-11-23); new orbits (`√T`, CS) with exact cells; Lean receipts for many bounds; PR #160 (2026-09-28) closes the twisted-cycle ansatz at `|H⟩^⊗7`; a daily literature and construction sweep by agents | the board's own verification receipts | **RANK-1's cell is unchanged: `6 ≤ χ(|H⟩^⊗7) ≤ 9`** (`bounds/qubit_H-m7-lower-6.json`, `-upper-9.json`). Nobody has posted a six-term witness or a rank-6 exclusion. But the board now excludes whole symmetric families at this cell by its own routes, and **whether those subsume our 3+3 and 2+4 split exclusions, or our second six-copy class, was not checked.** Both have been "still not sent" since 2026-09-26. Each week they sit they are worth less. `STABRANK_STATE.md` is two months stale and should be re-read at the new head before anything is sent |

## 5. AI-found circuits and the benchmark arenas

| item | claim | verdict |
|---|---|---|
| Gu, Kemeny, Yelin, Lukin, 2610.22367 | LLMs propose circuits, an independent evaluator scores them; Toffoli and qubit counts cut on seven of eight primitives | NOT RELEVANT to the views. Relevant as a workload: their circuits are Toffoli-heavy reversible arithmetic, exactly ABSTRACTS's structured class |
| Crichigno and Flammia, 2610.31352 | agents against a frozen exact verifier, built and validated on a solved case and on deliberately broken solutions "before any solution exists"; costs cut up to 73 % | NOT RELEVANT to the views. Their protocol is this record's plants-before-the-run, published; credit it where our method is described |
| Shirobokov et al., 2611.09822 | quantum programs synthesised against Lean specifications with verified cost | NOT RELEVANT to the views. It occupies "machine-checked" on the PROGRAM side; `PRIOR_ART.md`'s "machine-checked simulator kernel: VACANT" is about the simulator and still stands as far as this screen reached |
| QSim-Arena, 2611.15000 | agents reproduce published classical methods and cross-check them as baselines for quantum-advantage estimates | a second arena (dynamics, not Clifford+T). Not a baseline today |

## 6. Where the record's own sentences are weaker in this light

1. `QVM_ACUITY2_PREREG.md`, intro: "That is what today's state of the art does by hand" —
   false since 2026-07-29 (§1). A dated CORRECTION beside it (the prereg is frozen), and in
   `TIERS.md` §2's dispatch row, which calls the dispatch MEASURED without a credit.
2. `TIERS.md` §2, tableau row: "parity with stim on one core at `d = 221`" is true as measured
   and says less than it seems: it is single-shot tableau simulation, stim single-threaded,
   and SymFT claims `2.5 ×` Stim on one core while QuaSARQ and STABSim claim large factors on
   a GPU at this size. The row should name the task ("single-shot tableau, not sampling") and
   carry "no sampler; GPU baselines not met". `BENCHMARKS.md` line 44 names Stim as the one
   Clifford target; it is no longer the only one.
3. `PRIOR_ART.md`, exponent row: name the orbit; add Bevins as the face-orbit claim,
   unverified (§4).
4. `PRIOR_ART.md`, parallel-branch row: quEStab is still unread after three months, and
   GPUFOLD-1 was banked beside it.
5. The dispatch's `112 ×`: with García's structural scan published, a probe that costs a
   hundred times the view it picks is not a price to defend. It is a defect to fix or
   concede.

## 7. What could not be verified

- García's repository and its licence (the paper's link was lost in the HTML read).
- SymFT's own code repository (only its test-circuit repository and its predecessor's were seen).
- quEStab: not found.
- ArcTN, the learning-to-rank paper, Wan and Zhong, Kuyanov and Kissinger: not read at source.
- Whether the stabrank board's new exclusions at `|H⟩^⊗7` cover RANK-1's.
- Every performance number above: the sources' own, on the sources' machines. None is a
  record under `BENCHMARKS.md`'s rules until it is taken here with its conditions.
- Bevins's 24-term identity: not checked.

The benchmark that follows from this screen is drafted, not frozen, in
`conformance/qasm/QVM_BENCH2_PREREG_DRAFT.md`.
