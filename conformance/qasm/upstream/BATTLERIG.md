# BATTLERIG -- holon-qasm against the field's own tools

Every number here was measured on this machine by `battlerig.py` and is
rendered straight out of `battlerig_results.json`. Nothing is quoted from a
paper, a README, or another machine. Where a tool failed or exceeded the
120 s cap the cell says so; no failure is rendered as a number.

Machine: 13th Gen Intel(R) Core(TM) i9-13900HX, 32 logical cores.
Generated 2026-08-27T09:22:55; total rig time 46.3 min.
Cap 120 s per point; every entry is the median of 3 reps.

| component | version |
|---|---|
| qiskit | 2.5.1 |
| stim | 1.16.0 |
| qiskit-aer | 0.17.2 |
| holon-qasm | CIRISHolon @ 3346c64, binary built 2026-08-27T07:33:35 |

## What is being timed

One gate sequence per point is generated from a seed, then RENDERED into
each tool's native input -- QASM for ours, `stim.Circuit` for stim,
`QuantumCircuit` for qiskit and Aer. No tool is fed another tool's file
format, so no parser is on trial.

Two numbers per contender, both in the JSON:

- **sim_s** -- the simulation call only. Ours: the engine's self-reported
  seconds, which exclude QASM parse and JSON printing. Theirs: the
  evolve/run call, excluding circuit construction and imports. This is the
  head-to-head number, and the one tabulated below.
- **wall_s** -- the whole process, including startup and (for the python
  contenders) the qiskit/stim import. Disclosed because ours is a process
  launch and theirs is an in-process library call, and that gap is real.

---

## Conformance -- does it compute the right thing?

Speed without this table is meaningless.

| check | result |
|---|---|
| Clifford exact distribution, n=6 d=60, ours vs qiskit | max abs prob error **0.0e+00** over 16 outcomes; unitarity defect 0.0e+00 |
| Deterministic Clifford (echo), n=8: ours / stim / qiskit must agree | `00000000` / `00000000` / `00000000` -- **AGREE** |
| Our 7-T CCZ gadget vs qiskit's native `ccz` | max abs prob error **0.0e+00** |
| Clifford+T exact distribution, n=10 d=80 (23 T): ours vs qiskit / vs Aer | **0.0e+00** / **4.9e-17** |
| `amp` all-zeros amplitude vs qiskit, 5 circuits at n=6 t=6 | max abs error **0.0e+00**; smallest reference probability in the set **0.0156** (non-vacuous: YES) |
| Hidden shift n=6 t=14, shift `101011`: true un-shift must read 1, one-bit-corrupted un-shift must read 0 | ours **1.0** / **0.0**; qiskit reference 1.0 -- **TWO-SIDED PASS** |

The last row is the load-bearing one, because it is *two-sided*: a
simulator that returned 1.0 for everything would pass the positive leg and
fail the corrupted leg. Ours passes both. The `amp` row was rewritten after
its first version compared our 0.0 against a reference that was also 0.0 --
a pass that a simulator returning zero for everything would also earn -- so
its circuits are now searched for a reference probability large enough to
resolve, and that floor is reported.

---

## Lane 1 -- Clifford (the tableau lane)

Random Clifford circuits, depth 20n, then every one of the n qubits is
measured. All three contenders do the same work: evolve, then measure all.

| n | depth | ours | stim | qiskit `StabilizerState` | ours / stim | qiskit / ours |
|---|---|---|---|---|---|---|
| 64 | 1280 | 0.0043 | 0.0002 | 4.6818 | 28x | 1,093x |
| 256 | 5120 | 0.1212 | 0.0016 | TIMEOUT >120s | 76x | -- |
| 1024 | 20480 | 8.2788 | 0.0260 | TIMEOUT >120s | 318x | -- |

Seconds, lower is better. Every ratio column is literally its heading:
numerator divided by denominator. `ours / stim` above 1 means our run took
that many times as long as stim's; `qiskit / ours` above 1 means qiskit took
that many times as long as ours; a value BELOW 1 means the numerator was the
faster of the two. The same convention holds in every table below.

## Lane 2 -- Statevector (Clifford+T, dense)

Random Clifford+T, depth 8n. Ours measures ONE qubit rather than all n: the
2^n evolution is identical either way, but printing a 2^24-entry
distribution would time the JSON writer instead of the simulator.

| n | depth | T | ours | qiskit `Statevector` | Aer statevector | qiskit / ours | ours / Aer |
|---|---|---|---|---|---|---|---|
| 16 | 128 | 38 | 0.0222 | 1.5222 | 0.7418 | 69x | 0.03x |
| 20 | 160 | 46 | 0.3427 | 8.9737 | 0.7855 | 26x | 0.44x |
| 24 | 192 | 47 | 9.0764 | 87.1579 | 1.6621 | 9.60x | 5.46x |

Aer is multithreaded C++ over all cores; ours is single-threaded scalar
Rust. That is the honest reading of the last column.

## Lane 3a -- Hidden shift (the differentiated lane)

The standard extended-stabilizer benchmark. Construction
(Maiorana-McFarland, pi = identity): on n = 2m qubits with x = q[0..m-1]
and y = q[m..2m-1],

    f(x,y) = x.y (+) g(y),    dual  f~(a,b) = g(a) (+) a.b

is bent for any g, and then

    H^n . O_f~ . H^n . O_{f(. (+) s)} . H^n |0^n>  =  |s>

exactly, for any shift s. A final X layer maps |s> to |0..0>, so our `amp` --
which only ever reads the all-zeros amplitude -- must read exactly 1.0.
**That one reading is simultaneously the timing and the correctness check.**

### Why t is a multiple of 14 here, not the briefed {6, 12, 18}

O_f is a +-1 phase oracle, so it is generated by {Z, CZ, CCZ}. Degree <= 2 is
Clifford, so the cheapest non-Clifford term is one CCZ, whose T-count is
exactly 7 (the 4-T construction needs mid-circuit measurement, which this
QASM subset does not have). And g appears in BOTH O_f and its dual, so each
cubic monomial costs 2 CCZ = 14 T. t in {0, 14, 28} is the entire ladder this
family admits. The briefed t in {6, 12, 18} is served exactly by lane 3b.

| n | cubic terms | t | gates | ours `amp` | p (must be 1.0) | Aer ext-stab | Aer's top outcome | Aer / ours |
|---|---|---|---|---|---|---|---|---|
| 20 | 0 | 0 | 141 | 0.0001 | 1.0 | 0.0140 | `00000000..` 100% | 116x |
| 20 | 1 | 14 | 170 | 2.0176 | 1.0 | TIMEOUT >120s | -- | -- |
| 20 | 2 | 28 | 199 | TIMEOUT >120s | -- | TIMEOUT >120s | -- | -- |
| 40 | 0 | 0 | 300 | 0.0005 | 1.0 | 0.0195 | `00000000..` 100% | 38x |
| 40 | 1 | 14 | 323 | 5.9966 | 1.0 | 47.3886 | `10101101..` 1% | 7.90x |
| 40 | 2 | 28 | 343 | TIMEOUT >120s | -- | TIMEOUT >120s | -- | -- |
| 60 | 1 | 14 | 470 | 14.9095 | 1.0 | 55.8559 | `10000101..` 1% | 3.75x |

**Apples-to-oranges, stated plainly.** Our column is an EXACT amplitude:
2^t stabilizer branches, poly(n) work each, no 2^n anywhere. Aer's column is
APPROXIMATE sampling -- 100 shots through a randomised stabilizer-rank
decomposition with the parameters recorded in the JSON. They are not the
same computation, so the ratio is context and not a verdict. What the two
columns do share is the answer: both must name the hidden shift.

### Aer's extended stabilizer does not recover the shift at n >= 40

At t = 0 Aer is exact: one distinct outcome in 100 shots, the right one.
At t = 14 and n >= 40 it is not.

| n | t | Aer's top outcome | its share of 100 shots | distinct outcomes |
|---|---|---|---|---|
| 20 | 0 | `0000000000..` | 100% | 1 |
| 40 | 0 | `0000000000..` | 100% | 1 |
| 40 | 14 | `1010110110..` | 1% | 100 |
| 60 | 14 | `1000010110..` | 1% | 100 |

100 distinct strings in 100 shots is not a disagreement with us -- it is
the sampler returning noise. Aer is APPROXIMATE by design and its accuracy
is tunable, so before recording that as a failure it was given its knobs,
on the same n=40 circuit, at a raised 400 s cap:

| Aer setting | result |
|---|---|
| defaults (lane 3a above) | 47.4s, 100 distinct outcomes -- noise |
| approximation_error=0.01 | no answer within 400 s |
| mixing_time=100000 | no answer within 400 s |
| norm_estimation sampler | no answer within 400 s |

So the honest statement is not *Aer is wrong*. It is: **on this circuit Aer
at defaults is fast and wrong, and every setting that might make it right
does not return within 400 s** -- while the exact amplitude is 6.0 s. Note
also that Aer IS correct on the n=6 hidden shift in the conformance table
above, so this is a failure that appears with scale, not a broken method.

## Lane 3b -- Random Clifford+T at the briefed t

A random Clifford body of depth 4n with EXACTLY t T-gates spliced in, so
t in {6, 12, 18} is hit on the nose.

| n | t | gates | ours `amp` | Aer ext-stab (100 shots) | Aer / ours |
|---|---|---|---|---|---|
| 20 | 6 | 86 | 0.0006 | 12.6622 | 21,534x |
| 20 | 12 | 92 | 0.0278 | 19.0586 | 686x |
| 20 | 18 | 98 | 1.5826 | TIMEOUT >120s | -- |
| 40 | 6 | 166 | 0.0059 | 10.3847 | 1,755x |
| 40 | 12 | 172 | 0.1456 | 25.0647 | 172x |
| 40 | 18 | 178 | 3.7498 | 108.5518 | 29x |

On these circuits the all-zeros amplitude is about 2^-n, so our `p` prints
as 0.0 at the CLI's 12 decimals and Aer's 100 shots never land on that string.
Neither column is a correctness check here -- lane 3a and the `amp` conformance
row carry that -- so this lane is timing only, and is labelled as such.

---

## Caveats, and what the rig turned up about our own engine

1. **The T-count cap is not enforced on the `amp` path.** `run_magic` asserts `t_count <= 24`, but `magic_amplitude` has no such guard, so the t=28 hidden-shift points (2 of them) did not refuse by name -- they began enumerating 2^28 branches and hit the cap as TIMEOUT. Refusing by name is this engine's stated discipline, so this is a defect worth fixing, not a benchmark result.

2. **Aer's extended stabilizer cost is randomised, so its median is soft.** Its work is dominated by a randomised stabilizer-rank setup and norm estimation rather than by the sampling: an ad-hoc probe on one 6-qubit t=14 circuit took 58.4 s for 1 shot, 2.3 s for 10 shots and 59.6 s for 100 shots -- nearly independent of shot count, and swinging 25x on what is essentially the same job. Within this rig, where the simulator seed is varied per rep, the worst rep-to-rep spread was 3.4x (n=20, t=0). `sim_s_all`, `sim_s_min` and `sim_s_max` are in the JSON for every point. Read this lane's medians as order-of-magnitude.

3. **Our engine is single-threaded scalar Rust**; Aer's statevector is multithreaded C++ across all cores, and stim is vectorised. Lane 2's Aer column and lane 1's stim column should be read with that in mind -- they are a fair measure of what a user gets, and an unfair measure of the algorithm.

4. **`amp` prints 12 decimals**, which is why lane 3b's `p` column reads 0.0: the true amplitudes there are around 2^-n. That is a CLI formatting limit, not a precision limit of the computation, and it is why the `amp` conformance row had to search for circuits with a resolvable reference probability.

5. **Lane 1 measures every qubit** in all three contenders, so the tableau measurement cost is inside every number. Splitting qiskit's n=64 number shows where its time goes: evolving takes **0.0060s** and measuring all 64 qubits takes a further **1.2307s** -- the measurement is 99.5% of its cost.

   This qualifies lane 1's headline and should be read with it. On gate
   evolution alone qiskit is not slow: 0.0060s at n=64, which is the
   same order as our 0.0043s for evolve AND measure combined.
   Our large lead over `StabilizerState` is a lead in the MEASUREMENT path,
   not in Clifford gate handling, and it would shrink sharply on a workload
   that measured one qubit instead of all n.

   Caveat on the caveat: this diagnostic read 1.24s total on the same
   circuit lane 1 recorded at 4.68s.
   `StabilizerState.measure()` collapses random outcomes with an unseeded RNG,
   and how many outcomes come out random changes the work done, so qiskit's
   numbers here carry real run-to-run spread. Read its ratio columns as
   order-of-magnitude.

