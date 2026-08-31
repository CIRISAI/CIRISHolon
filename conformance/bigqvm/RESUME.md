# bigqvm lane — RESUME

## What this lane is

Massive virtualized quantum runs on the Clifford tier: scale the engine past
the bake-off's n = 4096 ceiling, then run a rotated surface code with FULL
adaptive syndrome extraction at the largest distance this box carries, and
compare against stim on the identical circuit.

## State

**Landed and committed** (`fb77e87`, plus the follow-up commit):

- `engine/crates/holon/examples/bigscale.rs` — the scaling probe. Reaches
  n = 131072; memory model exact (8.590 GB predicted, 8.617 GB measured).
- `engine/crates/holon/src/surface.rs` — the rotated surface code, derived
  from one parity rule and then self-checked (commutation, count, logicals,
  and a CONFLICT-FREE four-step schedule).
- `engine/crates/holon/src/coladaptive.rs` — the adaptive port: gates
  column-major, rowsums row-major, determinism scan as a contiguous column
  read, the single-term sign shortcut, a lazily-materialized row-major
  reference, and the mirror patch that keeps scans fast through collapses.
  11 conformance gates, including bit-identity with the row-major reference.
- `engine/crates/holon/examples/surface_flagship.rs` — the demo (`--mode qec`,
  7 verifications) and the matched benchmark (`--mode bench`, `--stim` emits
  the identical circuit).
- `conformance/qasm/surface_h2h.py` — the head-to-head harness.

**Measured, banked in BENCHMARKS.md:**

| d | n | measurements | wall | verifications |
|---|---|---|---|---|
| 141 | 39761 | 79520 | 25.8 s | 7/7 PASS |
| 181 | 65521 | 131040 | 49.5 s | 7/7 PASS |

## STATUS: both items CLOSED 2026-08-30

**d = 221 RAN AND PASSED.** The waiter was refused six times over 25 minutes,
then took a 17.9 GB window unattended: 97,681 qubits, 195,360 adaptive
measurements, 7/7 verifications, on seeds 1, 2 and 3. Against stim on the
identical circuit **we lead 1.33× (0.754)** — the one row in this lane whose
distributions do not overlap (our slowest run beat stim's fastest) and whose
min and median agree to one part in a thousand.

A defect the run exposed and that is now fixed: the memory guard UNDERSTATED
peak RSS by 50% (14.36 GB actual against a 9.54 GB model) because
`z_string_value` allocated a second full row-major tableau. It now reuses the
one buffer; peak RSS 9.571 GB against the 9.545 GB model, and wall fell from
111 s to 74 s.

**The head-to-head re-run is done** and moved the smaller-d verdict from
"stim leads 1.4–2.1×" to parity (0.82–1.26×).

Still owed, and neither blocks anything:

**(a) The QUIET-MACHINE repeat** — everything in this lane was taken at load
33–54, and per the standing ruling the ratios are not bankable against
CI-runner baselines until they are re-taken in a quiet window. A detached
waiter is armed for it:

    conformance/bigqvm/run_when_quiet.sh   (setsid)
    -> h2h_quiet.json      the citable table, written only if the window HELD
    -> quiet.DONE          success marker
    -> h2h_window_broke_*.json   kept, but explicitly NOT citable as quiet
    -> quiet.GAVEUP        no window inside MAX_WAIT

**CORRECTED 2026-08-30 after mesh-lane counter-evidence: it also PINS.** This
box is an i9-13900HX (P-cores 0-15, E-cores 16-31, scaling 57%), and with both
arms on the SAME core the d=101 verdict FLIPS — 0.822 unpinned, 1.201 on a
P-core, 0.989 on an E-core. That row is RETRACTED. Pinning also cut repetition
spread from 15-29% to 1.1-1.7% and HALVED both arms at d=221. A quiet window
fixes contention and does nothing about heterogeneity, so the quiet gate was
necessary and never sufficient; the sweep now runs BOTH core types and the
adversarial one is the number to report. See BENCHMARKS' correction to the
twenty-sixth entry.

It does not trust loadavg. It gates on a CALIBRATION RUN of known cost and
proceeds only when that job comes in within 1.15× of its record. The first
gate used d=45 and was REJECTED by measurement: at loadavg 46 it read 0.055 s
against a 0.052 s record (1.06×), because a 55 ms job gets an idle core even
on a saturated box — it could not tell quiet from loaded, which was its only
job. d=101 reads 1.79–2.02 s at loadavg 45 against a 1.319 s record
(1.36–1.53×) and does separate. The sweep also records loadavg at BOTH ends
and refuses to label a run "quiet" that started quiet and finished loaded.

A memory refusal at one size now SKIPS that size instead of aborting the
sweep — otherwise d=221 not fitting would throw away the other four sizes and
the window with them. That path is exercised, not assumed: the demo binary
carries `--force-refuse` (symmetric to `--no-guard`) so the harness's skip can
be tested on demand.

**(b) `SurfaceCode::new`'s O(stabilizers²) `verify_commuting`**, which costs
62–89 s of startup at d=221 and would be linear with a spatial index. It is
startup, not simulation, and it is separated from simulation time everywhere
it is reported.

## Historical — the two items as they stood while blocked

The box is under heavy memory pressure from siblings (a python3 process has
been holding 12–13 GB; MemAvailable has swung between 2.9 GB and 20 GB within
the hour). Both remaining items need a window and neither may be forced.

1. **d = 221 (n = 97681), the commissioned headline.** Working set is
   9.54 GB (column engine 4.77 + row-major reference 4.77), so it needs
   ~11.5 GB free with the 2 GB reserve. A detached waiter is polling:

       conformance/bigqvm/run_when_memory.sh   (setsid, running)
       -> conformance/bigqvm/d221.log          per-attempt log
       -> conformance/bigqvm/d221.DONE         written ONLY on success
       -> conformance/bigqvm/d221.FAILED       real failure (not memory)
       -> conformance/bigqvm/d221.GAVEUP       no window inside MAX_WAIT

   Restart with: `cd conformance/bigqvm && setsid nohup ./run_when_memory.sh
   > waiter.log 2>&1 &`. It runs the QEC demo, then the stim head-to-head.

   NOTE the marker discipline: an earlier version pre-checked memory in
   shell, lost the race to a falling MemAvailable, and would have written a
   DONE for a run the binary had REFUSED. The binary's exit code 2 is now the
   only authority; the marker is written only on rc = 0.

2. ~~**The head-to-head re-run.**~~ DONE 2026-08-30 — banked as the update to
   BENCHMARKS entry twenty-five. With the transpose blocking in, the ratios
   moved from 1.4–2.1× against us to **0.82–1.26×**, i.e. parity, with d=101
   reading 0.822 in our favour. Command, for the quiet-runner repeat that is
   still owed:

       stimvenv/bin/python conformance/qasm/surface_h2h.py \
           --d 21,45,101,141 --rounds 3 --reps 5

   stim 1.16.0 lives in a venv under this session's scratchpad; recreate with
   `python3 -m venv <dir> && <dir>/bin/pip install stim` if it is gone.

## The standing caveat on every ratio here

Every timing in this lane was taken on a box at load 33–41 with siblings
competing for memory and CPU. Spreads of 2–4× between repetitions of the SAME
measurement were observed at the large distances. The harness therefore
reports the MINIMUM as well as the median (interference can only add time),
and no ratio from this lane should be cited as a quiet-machine number. The
banked bake-offs used a quiet CI runner and this comparison deserves the
same before it is quoted anywhere.
