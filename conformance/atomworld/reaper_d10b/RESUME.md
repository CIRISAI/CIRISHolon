# D10b — the reaper's false-positive rate on a LIVE generation

**Status: RUNNING, observe-only.** Launched 2026-09-01 by the gpu-production lane.
Provenance in `LAUNCH_HEADER.txt` (binary sha256, repo HEAD, build exit status).

## What is running

`engine/target/release/examples/reap_observe` attached to PID **3117475**
(`target/release/examples/s2_ozone_table 32`, SATURATION-2's `(O,O,O)` ozone
tabulation, started Aug 31, 14,025 solves). It polls every 2 s and records what
the reaper WOULD have decided under four policies.

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
