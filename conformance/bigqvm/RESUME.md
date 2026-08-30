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

## OWED — the two open items, both blocked on the same thing

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
