# QVM-BENCH-2 — where each view of the QVM stands against the 2026 field: DRAFT, NOT FROZEN

*Drafted 2026-11-23 from `conformance/prior_art/QVM_FIELD_2026-11.md`, at the lead's word
("pursue the QVM ... benchmark against"). **Nothing here is frozen, and nothing has been built,
run or timed.** A stake written `«B·n»` is a blank the lead fills at the freeze; a proposed
value and its reason stand beside each. The draft goes to an adversarial review before any
freeze (§9 lists where to hit it). Rules it is written under: `conformance/BENCHMARKS.md` (a
record without its conditions is not a record; repeats and spread; a ratio or it did not
happen) and the misfit registry.*

## 0. The question, and what it is not

Each row of `TIERS.md` §2 carries a comparison this record chose for itself: stim's tableau
simulator on one core, a `χ = 64` MPS of our own, the full sum, a dense vector. The field of
2026 offers comparisons we did not choose. The question for each view: **under matched
conditions, on circuits someone else picked, is the view ahead of, level with, or behind the
best published tool for that job — and by what ratio, in which regime?**

This is not a campaign to win. Three of the five arms are expected to be lost (§5 says which
and why), and a loss written down with its regime is the product. It is not a claim about
the simulators named: every one of their numbers quoted in the screen was taken on their
machines, and none is used here as a bar. Every bar below is a ratio taken on THIS box, in
one session, both sides.

## 1. Arms

| arm | our view (instrument) | their tool, version pinned at the freeze | circuits | task |
|---|---|---|---|---|
| **A** amplitudes | the dispatch and each view alone (`holon::views`, `holon::magic5`, `holon::qasm`) | PyZX Magic Cat, PyZX BSS, quimb, Qiskit Aer statevector (the suite's own four), and quizx | **ABSTRACTS v0.1**, all 395 circuits: `random_clifford_t` 223, `random_fixed_t30` 103, `random_fixed_q20` 69 (MIT; `mjsutcliffe99/ABSTRACTS` at a commit pinned at the freeze) | `⟨0…0|C|0…0⟩`, one amplitude |
| **B1** tableau, single shot | `holon::coladaptive` at `S = 1` (`surface_flagship`) | Stim's tableau simulator; QuaSARQ single-shot (GPL-3.0, run as an external process, never linked) | rotated surface-code memory, `d ∈ {11, 21, 45, 141, 221}`, 3 rounds (MESH-CLIFFORD-1's four, and `d = 11`, STABSim's stated crossover), exported once to Stim format and to QASM; and 6 circuits from QuaSARQ's own heavy suite spanning 1,000 to 180,000 qubits, if its generator's licence allows their use (to be read) | the measurement record of one shot, bit-identical where the tool exposes its random stream, else statistically (§3) |
| **B2** tableau, many shots | **none exists.** The arm is entered as a declared ABSENCE | Stim's frame sampler; SymFT if its code can be found and run; QuaSARQ's sampler | the same surface-code circuits with circuit-level noise at `p = 10⁻³`, `10⁶` shots | detector samples per second |
| **C** near-Clifford | the stabilizer-rank sum on CPU and on the GPU fold (`holon::acuity`, `holon_gpu::acuity`) | Clifft (Apache-2.0), Tsim (Apache-2.0) | the published non-Clifford Stim-format circuits both papers use: `d = 3` and `d = 5` cultivation, `d = 3` distillation, a `d = 7` surface code with a coherent rotation (sources and licences read and pinned at the freeze; `rafaelha/tsim-bench` has no licence and is NOT a source) | one amplitude or a 4-qubit marginal at the acuity of ACUITY-1; no sampling claim |
| **D** the dispatch | the search of ACUITY-2 as amended, with its search wall COUNTED | the best of arm A's tools per circuit, by hand; García's quipu-cut if its repository is found and its licence allows running | arm A's 395, unlabelled to the search; plus the 80 instances of ACUITY-2 unchanged | which view is picked, and the wall of search plus view |
| **E** the rank referee | `holon::stabrank` and the exact ring | `unitaryfoundation/stabrank` at a head pinned at the freeze | Bevins's 24-term identity at twelve copies of the face state (arXiv 2610.17500); the board's `qubit_H` cells at `m = 7` | accept or reject, exactly. No timing |

Arms A, D and E need no card and stay under 2 GiB by construction (the suite's own ceiling).
B1 at `d = 221` needs about 14 GB (`MESH_CLIFFORD_RESULTS.md`: 14.36 GB actual) and the GPU.

## 2. Conditions (every timed number, both sides)

- Stamped at both ends by `conformance/lib/run_conditions.sh`: one-minute load at start and
  end, the core class, the clock as a fraction of advertised. A table whose load at either end
  exceeds `«B·load»` (proposed: `3.0`; MESH-CLIFFORD-1's citable table opened at `2.5` and its
  refused one ended at `9.7`, under a harness ceiling of 8 that proved too loose) is written as NOT CITABLE and is not read.
- Placement declared before the run and checked on the running PID: CPU arms pinned with
  `taskset` to one core class, never mixed; one core for every one-core comparison, with its
  SMT sibling idle; multi-core comparisons state both core counts in the row.
- Repeats: `«B·reps»` (proposed: 7) per cell; reported as median, min, max; a ratio is read
  only when the two spreads do not overlap, else the cell reads "level".
- Versions: every external tool pinned by release and by the sha256 of what was installed;
  our side by commit, with the binary's sha256 printed by the harness beside the commit it
  was built from, and a failed build aborts the run (the provenance rule).
- Timing scope stated per row: engine only, with parse and compile reported beside it for
  both sides; for compile-once tools the compile is amortised over the shot count in the row
  and ALSO shown unamortised.
- The GPU rows carry the device, driver and class; a CPU result is refused as a GPU result.
- Arm A is run twice: on this box under the rules above, and under the suite's own protocol
  (2 vCPU, 2 GiB, 5 minutes a circuit, 3 hours in all) emulated with a memory and CPU-count
  limit. The emulation is NOT the suite's hardware and is labelled so; an official number
  exists only if the lead decides to submit (§8).

## 3. Correctness before any clock

No cell is timed until its answer is accepted.

- **A, C:** our amplitude against the suite's published amplitudes and against quizx's exact
  value: equal in the ring where the referee is exact; else `|ours − theirs| ≤ 10⁻¹⁰`
  absolute, with the referee's own stated precision recorded. A circuit where two referees
  disagree with each other beyond that is set aside and listed, not scored.
- **B1:** the final tableau's stabilizer group equal to Stim's (canonical form), and where
  the random streams cannot be aligned, 1,024 shots each side and every detector's rate
  agreeing at `«B·z»` (proposed: `|z| ≤ 4` after a Bonferroni count over detectors).
- **D:** the certificate of the picked view checked against the referee on every instance,
  as ACUITY-2's S3: one violation kills the arm.
- **E:** exact acceptance in `Cyc`, and the board's own `stabrank_verify.py` on the same
  witness; the two must agree.

## 4. Plants

plants:

| plant | carrier, and the sector it must be nonzero in | must |
|---|---|---|
| **PB-1** a wrong amplitude | arm A, ten circuits of `random_fixed_q20`, our value multiplied by `1 + 10⁻⁶`; nonzero in the amplitude's modulus | the correctness gate REFUSES all ten before timing |
| **PB-2** a slowed side | arm A and B1, one cell each, our binary run under a `2 ×` CPU quota; nonzero in the wall | the ratio moves by `2 ×` within the spread; a harness that hides it is not an instrument |
| **PB-3** a loaded box | arm B1 at `d = 45`, a pinned busy loop on the sibling core; nonzero in the load stamp | the table is written NOT CITABLE |
| **PB-4** a mislabelled class | arm C, the CPU twin presented as the GPU; nonzero in the device class | refused at the door, as GPUFOLD-1's G2 |
| **PB-5** a face-state identity offered for the `π/8` state | arm E, Labib–Russo's three-term four-copy identity; nonzero in fidelity (`0.8249`, `conformance/srank/verify.py`) | REJECTED exactly; and accepted for the face state |
| **PB-6** a search that is free | arm D, the search wall zeroed in the harness; nonzero in the procedure's wall | the procedure ratio returns to ACUITY-2's view-only ratio on its 80 instances (C `0.55`, within the spread); else the harness is not counting what it says |

Each plant is run on the carrier of record before the freeze and its firing recorded; a
plant that does not fire there is redesigned before anything is frozen.

## 5. Stakes — ratios with their regimes

Ratio = our wall / theirs, same box, same session; `< 1` is ahead. "Level" = spreads overlap.
Cell edges are stated; a value on an edge takes the WORSE branch.

| stake | regime (a constant is a price in a regime) | bar | expectation, written before any run |
|---|---|---|---|
| **A1** coverage | all 395, suite protocol emulated | solved within the limits: `≥ «B·A1»` (proposed: the largest count any of the suite's four published tools solves, read from the suite's own results files at the freeze and written into the frozen text as a number) | level or ahead: the suite's published tools are Python; ours is compiled. Being ahead of Python is not a finding and is not claimed as one |
| **A2** the sum view against quizx | circuits with reduced T-count `t ≥ 20`, one core | median ratio `≤ «B·A2»` (proposed: `2.0`; kill `> 5.0`) | **behind or level.** Same decomposition (Magic5FromCat, theirs, credited); quizx has ZX simplification before the split and ours has `holon::simplify`, never compared |
| **A3** scaling | `random_fixed_q20` and `random_fixed_t30` | the fitted exponent of wall in `t` at fixed `n`, and in `n` at fixed `t`, each with its interval, beside each tool's | reported, not staked: the suite's stated purpose |
| **B1a** one core | `d ∈ {45, 141, 221}`, Stim | ratio at each `d`; MET if `≤ 1.10` at all three (the banked reading is `0.87` at `d = 221`) | level |
| **B1b** the card | `d ∈ {141, 221}`, QuaSARQ on this RTX 4090 Laptop GPU against our best CPU arm with its core count stated | ratio `≥ «B·B1b»` reads CONCEDED (proposed: `2.0`, QuaSARQ ahead by that or more with disjoint spreads) | **conceded.** We have no GPU tableau. The number that matters is by how much |
| **B2** sampling | `d ∈ {7, 9}`, `10⁶` shots, one core | none can be met: the view does not exist | **CONCEDED before the run, by declaration.** The arm exists so that Stim's and (if runnable) SymFT's shots/s on this box are in the record beside the empty cell |
| **C1** the sum on fault-tolerance circuits | `d = 3` cultivation, one amplitude | our sum's wall against Clifft's time to its first exact amplitude-equivalent (`«B·C1»`: the matched task is NOT settled, §9 item 3) | **behind**, by orders: their cost is `2^{k_max}`, ours `2^{0.396 t}` |
| **C2** the GPU fold against Tsim | terms folded per second at equal term counts, same card | ratio `≤ «B·C2»` (proposed: `1.0`; kill `> 4.0`) | unknown. The one arm with no expectation; say so |
| **D1** the pick | arm A's 395, unlabelled | picked view within `2 ×` of the best by hand on `≥ «B·D1»` of the scored circuits (proposed: `90 %`; kill `< 75 %`) | met on the view, as ACUITY-2 |
| **D2** the procedure | the same, search wall counted | search + view within `2 ×` of the best by hand on `≥ «B·D2»` (proposed: `75 %`; kill `< 50 %`) | **not met** below the size where a view costs more than the probes; ACUITY-2 read `112 ×` on Clifford at `n ≤ 20`. The stake that decides whether the dispatch is a product or a demonstration |
| **D3** against García | every circuit both complete | ratio, and the count of circuits where the two pick different representations | if runnable: behind on search cost. If not runnable: the cell says "not run: no code" and the prior-art credit stands on the paper alone |
| **E1** Bevins's identity | twelve copies, the face state | accepted or rejected, EXACT | accepted. A rejection is a finding to send to the author |
| **E2** the board's cell | `m = 7`, H-type | `6 ≤ χ ≤ 9` confirmed at the pinned head, EXACT; RANK-1's two exclusions checked against the board's closed families | unchanged |

## 6. Branches — exactly one fires

Decided in this order; the first that applies is the reading.

- **(v) VOID:** any plant of §4 fails on the carrier of record, or fewer than `«B·void»`
  (proposed: 300) of arm A's 395 pass the correctness gate on our side. Nothing is read.
- **(k) KILLED on correctness:** not void, and any certificate violation in D, or any
  accepted wrong answer in A, C or E. The timing arms are not read.
- **(a) AHEAD somewhere real:** neither of the above, and at least one of A2, B1a, C2, D2 is
  met with disjoint spreads against a compiled tool.
- **(b) LEVEL, with the gaps named:** none of the above, and B1a is level, and no kill bar
  (A2 `> 5.0`, C2 `> 4.0`, D1 `< 75 %`, D2 `< 50 %`) is crossed.
- **(c) BEHIND:** none of the above (some kill bar is crossed).

B2, B1b and C1 do not enter the branch: they are conceded or expected lost, and are
reported in every branch.

## 7. What "concede" looks like in the record

For each conceded or killed row, in the same commit as the results: (1) `TIERS.md` §2's row
gains the ratio, its regime and the tool's name, in the status cell; (2) `PRIOR_ART.md` gains
or corrects the row, by a dated CORRECTION where a sentence was wrong; (3)
`conformance/BENCHMARKS.md` gets the entry with its conditions; (4) the view is marked one
of: KEPT AS REFEREE (exact, slow, used to check others), KEPT FOR ITS CERTIFICATE (the
certificate or the bit-identity is the product, not the speed), or RETIRED IN FAVOUR OF the
named tool behind the dispatch, with its licence. No row is left reading "parity" or
"MEASURED" above a lost comparison.

## 8. Compute — tasks, not durations; NOTHING RUNS until the box is free and the lead says so

Barred today: anything over about 3 GB, cargo builds, any timing (a timing on a loaded box is
not a record). In order:

1. Zero compute, can be done now: pin every external tool's version and licence in the
   frozen text; read García's repository if it can be found; read quEStab (owed since
   2026-08-27); re-read `STABRANK_STATE.md` at the board's head; fetch ABSTRACTS's circuits
   and published results into a scratch folder (MIT; pinned by commit and sha256, not
   committed here unless the lead wants the 395 files in the repository).
2. The adversarial review of this draft, then the lead's blanks, then the freeze, alone.
3. Small, under 2 GiB, no card: arm E; the plants PB-1, PB-5, PB-6; arm A's correctness gate.
4. Needs a quiet window on one core class: arms A and D timed.
5. Needs the card and about 14 GB: B1 at `d ≥ 141`, B1b, C2. QuaSARQ built from source in a
   scratch folder as its own process.
6. Only on the lead's word: a public submission of a `methods/holon/` entry to ABSTRACTS (a
   pull request to someone else's repository, run on their hardware, results published on
   their dashboard).

## 9. Open to the reviewer — the weakest points

1. **B2 and C1 are losses entered on purpose.** A reviewer may say an arm that cannot be met
   is theatre. The defence is that the empty cell is today invisible in `TIERS.md`; the
   attack is that a sentence would do and no run is needed.
2. **A1's bar is taken from the suite's own published results**, which are Python tools on
   2 vCPU. Meeting it says little. A2 (quizx, compiled, same decomposition) is the real bar
   and its `2.0` is a guess with no banked number behind it.
3. **C1's matched task is not defined.** Clifft and Tsim sample; our sum gives one amplitude.
   Comparing them needs either a sampler on our side (T-by-T sampling, arXiv 2611.06119,
   implementable from the paper; its code has no licence) or a strong-simulation mode on
   theirs. As drafted C1 compares unlike things and should be cut or rebuilt.
4. **B1's bit-identity gate may be unreachable** against QuaSARQ (a different random stream);
   the statistical fallback at `|z| ≤ 4` has no power analysis.
5. **Emulating the suite's hardware** by quota is not the suite's hardware; clock and cache
   differ. The emulated numbers may not be placed beside the dashboard's.
6. **D2's `75 %` and D1's `90 %` are proposed from ACUITY-2's reading** of the same
   instrument. Unless the lead sets them from something else, that is a bar from the read.
7. **Branch (a) is cheap to reach**: one met stake among four. A stricter (a) would require
   A2 or D2 specifically.
8. **The joint pass region of D1 and D2** has not been exhibited: on small circuits D1 can be
   met only where D2 fails. The reviewer should ask for the circuits where both can hold.
9. **395 circuits of three random classes** are one population someone else chose; no
   structured class yet (the suite lists them as planned). The result is about random
   Clifford+T at small `n`, and the write-up must say so.
10. **Everything in the "expectation" column was written by someone who has read the
    competitors' papers and our own results.** They are not blind predictions and must not
    be scored as such.

witness: none for A1–A3, B1a, B1b, B2, C1, C2, D1–D3 (measured gates); `Closed` (Object.lean) and `tableau_closed_under_hadamard` (Stabilizer.lean) for the tableau view's closure in arm D, as ACUITY-2; none for E1, E2 (exact arithmetic checks against an external verifier, no Lean statement of ours)
**misfits:** M-BAR-FROM-THE-READ (§9 item 6: D1 and D2 as proposed), M-JOINT-PASS-REGION (§9 item 8), M-POPULATION-CHOICE (§9 item 9), M-PLACEMENT-LOTTERY (§2: one core class, pinned, median and spread, never the minimum alone), M-DEVICE-CLASS (PB-4), M-IDLE-CALIBRATED-TIMEOUT (§2: the load stamp read at both ends, in the regime it runs in), M-PROVENANCE-OVERREACH (§2: the binary's sha256 beside its commit, a failed build aborts), M-CHEAPER-THAN-ITS-PRICE (PB-6 and D2: a procedure cheaper than its own search is not that procedure), M-PLANT-OBS and M-PLANT-SECTOR (§4: each plant fired on the carrier of record before the freeze, its nonzero sector named), M-STALE-INSTRUMENT (the harness committed before the freeze), M-VALIDATED-NOT-WIRED (arm C runs the GPU fold through the same door as GPUFOLD-1, not its unit test), M-VACUOUS-SUCCESS (branch (v): a gate that saw too few circuits is void, not passed), M-ONE-MODEL-DELTA (every "behind" names the one tool it is behind); M-PARITY-PROTECT and M-LOOP-BLIND are contacted by keyword only ("parity" with stim as a timing word; a busy "loop" in PB-3) and neither sense applies — cited as the audit requires. Admitted by `Audit/prereg_audit.py` as drafted on 2026-11-23; to be re-run at the freeze.
