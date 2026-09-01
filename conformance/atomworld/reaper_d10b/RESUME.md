# D10b — the reaper's false-positive rate on a LIVE generation

**Status: RUNNING, observe-only.** Launched 2026-09-01 by the gpu-production lane.
Provenance in `LAUNCH_HEADER.txt` (binary sha256, repo HEAD, build exit status).

## What is running

`engine/target/release/examples/reap_observe` attached to PID **3117475**
(`target/release/examples/s2_ozone_table 32`, SATURATION-2's `(O,O,O)` ozone
tabulation, started Aug 31, 14,025 solves). It polls every 2 s and records what
the reaper WOULD have decided under FIVE policies.

**P5 is the load-bearing one and was added after the first reading**, which is why
it is called out here rather than left to the code. The first four did not
discriminate: P2, P3 and P4 all read zero, but at a 2 s poll against a
CPU-saturated holder the CPU tick ALWAYS advances, so rung 2 fires and MASKS rung
1 — P4's zero said nothing about the grace rule. P5 removes rung 2 and keeps the
own-step grace, so nothing but rung 1's rule separates it from P1. It reads **0
against P1's 2,809 of 3,390 polls**, which isolates §5's claim on real work: the
grace being sized by the holder's own step is what fixes it, not anything in rung
2.

**Current reading (partial, run continuing).** 3,390 polls, 130 nodes solved:

| policy | rung 1 grace | rung 2 | FALSE reaps |
|---|---|---|---:|
| P1 rung-2 absent | flat 10 s | absent | **2,809** |
| P2 CPU tick | flat 10 s | since last poll | 0 |
| P3 debounced | flat 10 s | over 3 polls | 0 |
| P4 own-step | 3x own step | since last poll | 0 |
| P5 own-step ALONE | 3x own step | **absent** | **0** |

The holder's own worst step GREW from 128 s to 364 s as the generation reached
slower nodes, so the flat 10 s grace went from 0.08x of it to 0.03x — three times
more wrong without anyone touching it. That is the case against a flat constant:
the quantity it bounds is not stationary, so no constant chosen from any window
survives the next one.

**The reaper is OFF and the books prove it.** Only `Reaper::judge` is called —
`judge` does not take the arena and cannot convict. At exit the observer asserts
its own arena ledger shows `convicted == 0` and `reaped == 0`. Nothing signals,
kills, or writes to the observed process. The only write is rung 3's own
dot-file probe into `engine/output/`, written and unlinked each poll.

## Outputs

| file | what |
|---|---|
| `d10b_ozone.jsonl` | one record per poll: receipts, silence, own-step, per-policy verdict |
| `d10b_ozone.log` | the running summary (re-printed every 30 polls) |
| `d10b_ozone.summary.txt` | the final table, written at exit |
| `d10b_ozone.DONE` | `exit=N` — written by the launcher when the observer stops |

## How it ends

The observer stops when the observed PID disappears (the generation finishes or
is stopped) and then writes its summary. **D10b asks for a FULL generation**;
if this reading covers only part of one, that is what it says and the reading is
partial. Do not promote the reaper on a partial reading.

## To resume / restart

```bash
setsid nohup bash -c '
  D=/home/emoore/CIRISHolon/conformance/atomworld/reaper_d10b
  /home/emoore/CIRISHolon/engine/target/release/examples/reap_observe \
    --pid <PID> --log /home/emoore/CIRISHolon/engine/output/s2_ozone_table_progress.log \
    --probe-dir /home/emoore/CIRISHolon/engine/output \
    --poll-ms 2000 --flat-grace-s 10 --k 3 --debounce 3 \
    --out "$D/d10b_ozone" > "$D/d10b_ozone.log" 2>&1
  echo "exit=$?" > "$D/d10b_ozone.DONE"' &
```

`--flat-grace-s` is a **declared** constant, not a derived one: it is the shape of
the constant that convicted 1115 live holders, and the summary prints its ratio to
the holder's own measured step so the reader can see the gap rather than be told it.
