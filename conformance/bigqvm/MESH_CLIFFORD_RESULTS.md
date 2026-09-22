# MESH-CLIFFORD-1 — results: «TITLE TO BE FILLED WITH THE VERDICT WHEN THE TABLE IS TAKEN»

*Freeze `MESH_CLIFFORD_PREREG.md` (5c259f9, alone, before any code). Engine
`engine/crates/holon/examples/surface_flagship.rs` + `holon::coladaptive`
(delegate; `--shards S`, the S echo, the shard-crossing count and the
measurement-record hash). Harness `conformance/bigqvm/mesh_h2h.py` (G3, G4 and
G1's cross-check), waiter `conformance/bigqvm/mesh_quiet.sh` (the quiet window,
both core classes). Records under `conformance/bigqvm/`:
`mesh_h2h_pcore.json`, `mesh_h2h_ecore.json`, `mesh_quiet.log`,
`mesh_quiet.DONE`; and `mesh_h2h_dryrun_S1.json`, which is the protocol
validation and NOT a reading. Cores: P 8–15, E 16–23, never the cores another
campaign holds. «WALL AND CORE-SECONDS TO BE FILLED».*

## The verdict, first

**«THE ONE-PARAGRAPH VERDICT TO BE FILLED: which gates, which branch of §4,
and the one number a reader should carry away.»**

## §0 — the gates

| gate | stake | verdict | the number |
|---|---|---|---|
| **G1** — bit-identity across shard counts | the record and the final tableau identical to the bit to `S = 1`, at `d ∈ {21, 45, 141, 221}`, seeds 1–3, and the bake-off's random Cliffords | | |
| **G1 (harness cross-check)** — the record hash at every `(d, S)` equals `S = 1`'s, checked BEFORE any arm is timed | no arm is timed whose hash differs | | |
| **G2** — the crossing fraction declared and `< 0.25` at `S = 8`, `d ≥ 141` | printed per `(d, S)` | | |
| **G3** — the head-to-head | at `d = 221`, `S = 8` on P-cores: wall `≤ 0.5 ×` `S = 1` AND `≤ 0.5 ×` stim, spreads not overlapping. KILL: `S = 8` slower than `0.8 ×` `S = 1` | | |
| **G4** — memory | peak RSS at `S = 8` within `1.25 ×` of `S = 1` | | |

| plant | must | verdict | the number |
|---|---|---|---|
| **P1** | a corrupted shard column is CONVICTED by G1, by name | | |
| **P2** | a scrambled fold order leaves the phase bit unchanged | | |
| **P3** | the random stream consumed out of order makes the record DIFFER | | |
| **P4** | `S = 1` through the sharded path reproduces unsharded `ColAdaptive` bit for bit | | |

Branch taken: «(a) / (b) / (c) / (e) — TO BE FILLED».

## §1 — the protocol as run

The G3/G4 instrument is `conformance/bigqvm/mesh_h2h.py`. One invocation per
core class, launched by `mesh_quiet.sh` when the box is quiet:

    conformance/bigqvm/mesh_h2h.py <flagship binary> \
        --d 45,141,221 --shards 1,2,4,8 --reps 5 --rounds 3 --seed 1 \
        --cores 8-15 --max-load 8 --out conformance/bigqvm/mesh_h2h_pcore.json

What it does, per distance:

1. **The circuit, once.** `surface_flagship --mode bench --stim PATH` emits the
   same four-step rotated-surface-code extraction cycle it runs, in stim's
   format. Both arms therefore run the IDENTICAL circuit and this compares
   engines, not circuit generators (the bake-off's rule, BENCHMARKS entries
   six/eight/ten).
2. **A probe pass before any timing.** Each `S` runs once; the harness reads
   back the S the binary actually used, the shard-crossing fraction and the
   measurement-record hash, and compares the hash to `S = 1`'s. An arm whose
   hash differs is REFUSED, not timed and caveated — that is G1 standing inside
   the timing harness, and §4's branch (e) applied where it bites. An arm whose
   S the binary does not echo back is refused too; see §on-building (1). The
   probe doubles as the warm-up, whose timing is discarded either way.
3. **A timed pass, interleaved.** Every surviving arm — each `S`, and stim —
   runs once per repetition, and the arm ORDER ROTATES by repetition, so a
   drifting machine hits every arm equally rather than whichever ran last.
4. **Pinned, both arms.** Each engine run is `taskset -c <--cores>`; the
   harness sets its own affinity to the same set so the in-process stim arm is
   pinned identically. The d=101 verdict FLIPPED between placements on this box
   (0.822 unpinned, 1.201 P-core, 0.989 E-core) — see BENCHMARKS'
   twenty-seventh entry.
5. **Median, min and spread.** The median carries contention; the minimum is
   the robust estimator (interference can only ADD time); the spread is
   max/min. The stake is graded on the medians with the `[min, max]` intervals
   required not to overlap.
6. **Peak RSS twice.** The binary's own `VmHWM` (its `peak_rss_bytes`) is the
   primary; `/usr/bin/time -v` on the child is the independent check. A
   disagreement over 5% is FLAGGED in the row, never averaged away. G4 is the
   ratio of the `S = 8` peak to the `S = 1` peak at the same distance.
7. **Conditions at both ends.** Loadavg, core class derived from SMT siblings,
   the pinned cores' clock as a fraction of advertised, and MemAvailable, at
   launch and at exit — the fleet rule of 2026-09-01
   (`conformance/lib/run_conditions.sh`).
8. **The load gate.** If the 1-minute loadavg exceeds `--max-load` (default 8)
   at either end, the JSON is NOT written to `--out`; the data is kept beside
   it as `*.not-citable-loadNN.json` and the harness exits 3. A mislabelled
   quiet number is worse than an honestly-labelled loaded one.

The waiter, `mesh_quiet.sh`, gates on loadavg AND at least 14 GB MemAvailable
(d=221's 9.54 GB working set plus stim's own ~2.4 GB tableau plus the reserve),
then on TWO calibration runs: a single-core `d = 101` against this box's own
banked quiet records (0.763 s P, 1.371 s E, the min columns of
`h2h_quiet_pcore.json` / `h2h_quiet_ecore.json`), and a CONCURRENT one, one
copy per core of the set, gated on the slowest — because a sweep that will use
eight cores is only as quiet as its worst one, and `run_when_quiet.sh`'s
one-core gate cannot see that. The binary's own exit code 2 remains the
authority on memory; the shell polls only to avoid hammering.

## §2 — the protocol validation (NOT a reading)

`mesh_h2h_dryrun_S1.json`, taken 2026-09-21 on cores 21–27 against the
PRE-`--shards` binary, `--d 21,45 --shards 1 --reps 3`. It exists to prove the
instrument runs end to end — engine arm, stim arm on the identical circuit,
both RSS sources, the conditions stamp, the grader and the JSON — and it is
stamped `citable: false`, `protocol_validation: true`. It was taken at loadavg
12 on a box running seven other lanes, its load gate correctly REFUSED, and no
number in it may be cited.

What it read, for the record and for nothing else:

| d | n | arm | wall median | min | spread | peak RSS (VmHWM / time -v) |
|---|---|---|---|---|---|---|
| 21 | 881 | ours S=1 | 0.002034 s | 0.002016 s | 1.012× | 3.215 MB / 3.092 MB (3.8%) |
| 21 | 881 | stim 1.16.0 | 0.001649 s | 0.001536 s | 1.079× | — |
| 45 | 4049 | ours S=1 | 0.043492 s | 0.041776 s | 1.067× | 19.743 MB / 19.620 MB (0.6%) |
| 45 | 4049 | stim 1.16.0 | 0.027546 s | 0.027077 s | 1.120× | — |

ours/stim 1.23× at d=21 and 1.58× at d=45 on medians — consistent in direction
and magnitude with the banked quiet table's small-d rows (1.37×/1.46× on a
P-core, 1.32×/1.49× on an E-core), which is the only claim made for it: the
instrument is reading the same machine the last one read. The two RSS sources
agreed to 3.8% at d=21 and 0.6% at d=45, both inside the 5% flag.

Which paths of the harness were exercised, and how: the engine arm and the stim
arm on the identical circuit (above); the S-echo refusal (`--shards 1,2` at
d=21 against the pre-`--shards` binary — S=2 refused by name, never timed); the
per-size memory skip (`--exercise-refusal`, which passes the binary's own
`--force-refuse` test hook — both distances skipped, the sweep continuing); the
load gate's refusal branch and its `--dry-run` override; and both branches of
the waiter's window check (see `mesh_quiet.sh`'s header). Not exercised, because
no binary yet emits them: the record-hash cross-check and the crossing-fraction
read. Their parsers are unit-visible in `extract_shard_fields()` and the format
they expect is in the harness's header and was sent to the engine agent.

The grader's own arithmetic was exercised separately, on three synthetic tables
(`mesh_h2h.py --selftest`): the stake met, the kill fired, and the
neither-case with G4 breached. Synthetic numbers, never written to any record.

## §3 — the tables

«THE P-CORE TABLE, THE E-CORE TABLE, AND THE S-AGAINST-WALL CURVE IF BRANCH (b)
— TO BE FILLED FROM `mesh_h2h_pcore.json` AND `mesh_h2h_ecore.json`.»

## §4 — on building the harness: what the freeze got wrong

The freeze is not amended by any of this and no stake, gate, plant or number is
moved. These are the things the harness found that the freeze could not have
known, recorded where the next freeze in this lane will read them.

1. **G1 as written cannot catch a flag that does nothing, and that is the most
   likely failure of the whole campaign.** `surface_flagship`'s `arg()` helper
   finds a flag by searching the argument vector and returns a default when it
   is absent — it does not reject unknown flags. So a binary built before
   `--shards` landed, handed `--shards 8`, runs `S = 1` in silence. G1 then
   PASSES trivially (the `S = 1` engine is bit-identical to itself), G2 reports
   nothing, and G3 times the same engine four times and reads the four numbers
   as a scaling curve. The harness closes it: any `S != 1` whose S the binary
   has not ECHOED BACK — metadata `shards.count`, or the stderr line — is
   refused by name and never timed. If a future freeze in this lane stakes a
   behaviour on a new flag, the echo of that flag belongs in the gate.

2. **P-cores 8–15 is FOUR physical cores, not eight.** This box is an
   i9-13900HX; `thread_siblings_list` says cpus 8/9, 10/11, 12/13 and 14/15 are
   SMT pairs. The freeze stakes G3 at `S = 8` on cpus 8–15, which puts two
   shards on each of four physical cores, sharing execution units, L1 and L2.
   E-cores 16–23 are eight DISTINCT physical cores with no siblings. So the two
   staked placements are not the same experiment at `S = 8`: only the E sweep
   gives the cut eight real cores, and the staked P sweep asks for a 2× wall
   reduction from four cores plus hyperthreading on a workload whose own
   kill-clause names memory bandwidth as the suspected wall. If the stake fails
   on P and passes on E, the freeze's letter kills a cut its own reasoning
   supports. The waiter takes `PCORES` from the environment for exactly this:
   `PCORES=0,2,4,6,8,10,12,14` is eight physical P-cores, one thread each, and
   is the control worth taking when cores 0–7 are free.

3. **G3 compares a multi-core arm against single-threaded stim.**
   `stim.TableauSimulator` is one thread. "`S = 8` wall `≤ 0.5 ×` stim's" is a
   WALL-CLOCK claim on eight cores against one, which is a different comparison
   class from every Clifford row already in BENCHMARKS (all one core against
   one core). It is a legitimate claim — wall is what a user waits — but the
   row must carry its class, or it will be read as "the QVM is 2× stim" when
   the honest sentence is "with eight cores the QVM is 2× one-core stim". The
   harness prints the `S = 1` column beside it precisely so the single-core
   comparison stays visible in the same table.

4. **G4 says "the `S = 1` MODEL"; the harness grades against the `S = 1`
   MEASUREMENT.** The only peak RSS this lane can measure is `VmHWM`, and the
   analytic model in `working_set_bytes()` already understated the true peak by
   50% once (the d=221 defect: 14.36 GB actual against 9.54 GB modelled,
   because `z_string_value` allocated a second row-major tableau). Grading a
   1.25× ceiling against a model that has been wrong by 50% would be a gate on
   the model. The harness grades measurement against measurement at the same
   distance and cross-checks two independent sources of it. If the freeze meant
   the analytic model, that is a weaker gate and someone should say so before
   the table is banked.

5. **§5 "Cost: none new to derive — the engine, the reference, the harness and
   the quiet-window waiter exist" is false about the harness and the waiter.**
   `surface_h2h.py` has no shard axis, no per-`(d, S)` row, no RSS ratio, and
   its `--pin` takes a single integer — a sharded arm cannot run under it at
   all. `run_when_quiet.sh` calibrates ONE core, which was the right gate for a
   one-thread engine and is the wrong gate for an eight-shard sweep: it cannot
   tell "my core is free" from "eight cores are free". Both were written new
   (`mesh_h2h.py`, `mesh_quiet.sh`). No new DERIVATION was needed, which is
   what §5 probably meant; the instrument was not free.

6. **G3 names no seed, and the record hash is only comparable within one.** The
   harness holds `--seed 1` across every `S` and stamps it into the JSON. G1
   names seeds 1–3; if the intent is that G3 also sweeps seeds, the arm count
   triples and the window needed with it.

7. **G4 assumes the exchange buffers are the only addition, so it stakes only
   `S = 8`.** If the sharded engine allocates per-shard scratch proportional to
   `n`, RSS grows with `S` and the interesting number is the SLOPE, not the
   endpoint. The harness reports peak RSS for every `(d, S)` and the worst ratio
   anywhere, not only the staked one, so the S-dependence is visible rather
   than assumed.

8. **A record taken in under five seconds has one loadavg, not two.** The
   kernel updates `/proc/loadavg` every five seconds, so the "conditions at both
   ends" stamp on a short run reads the same number twice — visible in the dry
   run's JSON. The real G3 sweeps run for many minutes and the stamp does its
   job there. A reader of a short record should not mistake the repetition for
   a copy, and should not read it as evidence that the window held.

9. **§4 of the freeze has branches (a), (b), (c) and (e), and no (d).** A
   reader looking for branch (d) will not find one; nothing depends on it, and
   it is noted only so that the gap is known to be a skip and not a loss.

---
witness: none (an engineering campaign; its gates are bit-identity and measured
wall, its plants convict a corrupted shard)
**misfits:** M-PLACEMENT-LOTTERY (every arm pinned, both core classes run, and
the P/E asymmetry at S=8 named in §4.2), M-CHEAPER-THAN-ITS-PRICE (§4.5: the
freeze's "no new cost" is corrected against what the instrument actually took),
M-PLANT-OBS, M-PLANT-SECTOR, M-PARITY-PROTECT, M-HOMOG, M-DEVICE-CLASS (§4.2,
§4.3: core class and thread count are declared variables, not background),
M-IDLE-CALIBRATED-TIMEOUT (the waiter's gate is a calibration run against a
measured record, and the concurrent calibration is named as a judgement until
the first quiet window measures it) — contacted by keyword, cited.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier (a
shard, a fold order, a random stream, the unsharded engine), and the sector the
plant acts on is nonzero in that carrier by construction.


## Read so far (2026-09-22, before any quiet-window timing)

G1 PASS at `S = 1, 2, 4, 8` (record, digest and final tableau identical to the unsharded engine
and the row-major reference: surface code `d = 21, 45` three seeds; random Clifford `n = 256,
1024` five seeds; `d = 141` the same hash on all paths). G2 KILLED on the flagship's qubit
numbering (`0.98–1.00` of CX gates cross at every `S`) and MET after branch (c) was carried
out on the numbering: banded, `0.025` at `d = 141, S = 8`. Plants P1–P4 fire. First look on
seven E-cores at `d = 141`: `6.21 → 5.80` s (unsharded → `S = 1`), `4.38` s at `S = 8`, `0.74 ×`,
with `S = 4` and `S = 8` tied — because the cross-shard rowsum the cut parallelises fires on
`69` of `9,940` cascades: the section is one percent of the work. **G3's quiet window is
DEFERRED** rather than run: it would measure that the cut is aimed at the wrong section. The
next step is the profile (gate streaming, scans, rowsums, rebuilds, exchanges at `S = 1` and
`8`), then a re-aimed cut with the same bit-identity plants, then the window with the
comparison class declared (an adaptive-circuit claim, eight cores against one). The QVM's
claim itself — cheap part located, hard part priced, only enough of it done — is
`conformance/qasm/QVM_ACUITY1_PREREG.md`, where the mesh's home is the branch sum.
