# MESH-CLIFFORD-1 — the flagship Clifford tier cut across cores: PREREGISTRATION

*Frozen 2026-09-21, committed alone before any code. The QVM's two head-to-heads against stim
(`conformance/qasm/BAKEOFF_PREREG.md`; `conformance/bigqvm/RESUME.md`, d = 221 at parity) run
one tableau on one core, as stim does. `holon::mesh` is measured and bit-identical on every
fold-shaped workload and has never been applied to a tableau, which is one dense state, not
a sum. This freezes the cut, its gates, its plants, its head-to-head and its kill.*

## 1. The cut

`ColAdaptive` keeps gates column-major (one qubit = `2n/64` contiguous words) and the
destabilizer product row-major. **The shard is a contiguous range of qubit columns.** With
`S` shards over `n` qubits, shard `s` owns columns `[s·n/S, (s+1)·n/S)`.

- Single-qubit gates (H, S, S†, X, Z) act on one column: shard-local, no communication.
- CX(c, t) acts on two columns. Within a shard: local. Across shards: the two columns
  are exchanged for the duration of the gate (a copy of `2n/64` words each way), then
  written back. The surface code's four-step schedule fixes which pairs cross; the shard
  boundaries are chosen on the code's row structure so that crossing pairs are a minority,
  and their count is REPORTED per round.
- Measurement of `Z_q`. The determinism scan is a read of column `q`: shard-local. The
  rowsum, when the outcome is random, is a row operation across ALL columns: each shard
  applies its column range of the rowsum in parallel, and the phase bit is a FOLD over
  shards of each shard's partial product, in shard order, under `holon::merge`'s law. The
  random outcome bit comes from ONE declared stream, consumed in circuit order,
  independent of `S`.

Nothing else changes: the same `PackedTableau` reference, the same mirror patch, the same
`scan_fast`/`scan_fallback` counters.

## 2. Gates, each with its kill

- **G1 — bit-identity across shard counts.** For `S ∈ {1, 2, 4, 8}`, the same seed and
  circuit: the measurement record (every outcome bit, in order) and the final tableau are
  identical to the bit to the `S = 1` engine, which is itself gated bit-identical to the
  row-major reference (coladaptive's existing gate). Circuits: the surface code at `d ∈ {21,
  45, 141, 221}`, `--mode bench`, rounds 3, seeds 1–3; and the bake-off's random Clifford
  circuits at `n ∈ {256, 1024, 4096}`, five seeds. **Kill:** one differing bit at any `S`.
- **G2 — the crossing count is declared and bounded.** The fraction of CX gates that cross a
  shard boundary at each `(d, S)` is printed and is `< 0.25` at `S = 8` for `d ≥ 141`.
  **Kill:** a cut that crosses more than a quarter of its gates has the wrong boundaries;
  the cut is re-derived before any timing is read.
- **G3 — the head-to-head.** The quiet-window harness (`surface_h2h.py`, loadavg printed,
  both core types pinned, five repetitions, medians and spreads) at `d ∈ {45, 141, 221}`,
  `S ∈ {1, 2, 4, 8}` on P-cores `8–15` and E-cores `16–23`, against stim on the identical
  circuit. **Staked:** at `d = 221`, `S = 8` on P-cores, wall `≤ 0.5 ×` the `S = 1` engine's
  and `≤ 0.5 ×` stim's, with the spreads not overlapping. **Kill:** `S = 8` slower than
  `0.8 ×` of `S = 1` — the cut does not pay and the mesh's memory-bandwidth wall is the
  finding, banked as such.
- **G4 — memory.** Peak RSS at `S = 8` within `1.25 ×` the `S = 1` model (the column
  exchange buffers are the only addition). **Kill:** above it.

## 3. Plants, each of which must fire before any timing is read

| plant | must |
|---|---|
| **P1** | a shard whose column range is corrupted by one bit after the gate phase: G1 CONVICTS it by name (which shard, which column) |
| **P2** | shards applied in a scrambled order for the rowsum fold: the phase bit is unchanged (the fold is commutative under the declared order, and the record is identical) |
| **P3** | the random stream consumed out of circuit order on purpose: the record DIFFERS, so the stream-order rule is load-bearing and not decorative |
| **P4** | `S = 1` through the sharded code path reproduces the unsharded `ColAdaptive` bit for bit on every gate-phase and measurement-phase test the crate already carries |

## 4. Branches

- **(a)** G1–G4 met → the first QVM tier ahead of stim by construction rather than by cache;
  BENCHMARKS.md, the stance's measured section and the fence ledger's compute-priced rows
  the mesh discharges are updated; the GPU cut (`holon-gpu`, gathers not scatters) is the
  next freeze.
- **(b)** G3 killed with G1 met → the cut is correct and does not pay on this box; the
  bandwidth wall is measured (`S` against wall) and banked; the GPU cut is still the next
  freeze, since a warp schedule is a different bandwidth.
- **(c)** G2 killed → the boundaries are re-derived from the schedule; nothing timed.
- **(e)** G1 or a plant fails → nothing is read.

## 5. Cost

None new to derive: the engine, the reference, the harness and the quiet-window waiter
exist. Core placement is declared (P-cores 8–15 or E-cores 16–23, never the fluid arms'
cores while they run).

## 6. What this does not test

Circuits other than Clifford; the magic tier's branch sums (already meshed); the GPU.

---
witness: none (an engineering campaign; its gates are bit-identity and measured wall, its plants convict a corrupted shard)
**misfits:** M-PLACEMENT-LOTTERY, M-CHEAPER-THAN-ITS-PRICE, M-PLANT-OBS, M-PLANT-SECTOR, M-PARITY-PROTECT, M-HOMOG, M-DEVICE-CLASS, M-IDLE-CALIBRATED-TIMEOUT — contacted by keyword, cited.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier (a shard, a fold order, a random stream, the unsharded engine), and the sector the plant acts on is nonzero in that carrier by construction.

---
### Amendment 1, on building the harness (2026-09-21, before any sharded timing; stakes unchanged, one placement corrected)

1. **The staked P-core placement was wrong by a factor of two.** Cores 8–15 are FOUR physical
   P-cores with their SMT siblings (`thread_siblings_list`: 8/9, 10/11, 12/13, 14/15); the
   lead knew this from the arms' own layout and wrote `S = 8` on them anyway — the
   twenty-first instance. G3's P-core arm runs on eight PHYSICAL P-cores `0,2,4,6,8,10,12,14`
   once the fluid arms release them; `8–15` is kept as a labelled SMT control. The E-core arm
   (`16–23`, eight distinct cores) stands.
2. **G3's comparison class is declared.** A sharded arm against single-threaded stim is a
   wall-clock claim with eight cores against one; the banked table's row carries that class,
   and the one-core-vs-one-core row (`S = 1`) is beside it as before.
3. **A flag the binary ignores must refuse, not pass.** The harness refuses any `S ≠ 1` the
   binary has not echoed back, and G1 is read only on echoed runs — a pre-`--shards` binary
   would otherwise pass G1 trivially and time one engine four times.
4. **G4 is measurement against measurement** (VmHWM at every `(d, S)` against `S = 1` at the
   same `d`, two sources cross-checked), not against the analytic model, which understated
   the d = 221 peak by 50 % once. The worst ratio anywhere is reported beside the endpoint.
5. §5's "the harness exists" was false in the sense that mattered: no shard axis, no RSS
   ratio, a single-core pin; it was written new. `mesh_quiet.sh` is not launched by the
   agent; the lead launches it.
