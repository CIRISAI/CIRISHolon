# THE CHANNEL LEDGER — the five channels as declarations in the engine, gated on bit-identity

*2026-09-05. Not a freeze and not a campaign: an engineering pass on the force law under
OBJECT.md design rule 10, at the operator's order, on the branch. It evaluates no new
physics. Its one gate is that the engine before it and the engine after it agree in every
byte on staked scenes, and that gate is a written receipt (`tests/data/channel_ledger.receipt`)
produced by the PRE-ledger library and reproduced by the post-ledger one. The reading that
motivated it is `CIRISOntology/PHILOLOGY_BACKPASS.md` Backpass III §7.*

## What was found (the assessment, §7 of the backpass, verified on the code)

The force law (`holon-render/src/sim.rs::compute_forces`) is organised by SECTOR — five
accumulators with five ledger rows: the near pair table, the far pair tail, the three-body
tables, the many-body clusters, the field. The physics of rule 10 is organised by CHANNEL —
five pushes with derived decay laws. These are two different partitions of one energy and
they map many-to-many: the pair table carries exchange and pair dispersion folded into one
number; the far tail is dispersion with its exponent FITTED where the ledger says it is
derived; channel 4's harvested `−C/R⁹` exists only in `EMBED2_RESULTS.md` and not in the
engine; induction has no row at all. "How far does this reach at this budget" existed in
three dialects: a closed form on a pure power, a bisection on an interpolant, and a declared
measured reach per class.

## What was built

| step | what | where | gate |
|---|---|---|---|
| 1 | The five channels as RECORDS: kind (the taxonomy's word), arity, derived rate, evaluation shape (sum / fixed point / solve), receipt column, prior art. The ledger ROWS as a table in the order `energy()` sums them, each row declaring which channels it carries and whether WHOLE or FOLDED. | `src/channel.rs` (`CHANNELS`, `Row`, `Row::carries`) | declarations only; bytes identical |
| 2 | ONE allocator, `reach_for_budget`, with three arms — `Power` (closed form), `Sampled` (the doubling walk + 80-step bisection), `Declared` (the registry's measured reach). `TailModel::radius_for_budget` and `Sim::derive_pair_cutoff` now call it; their arithmetic moved verbatim. | `src/channel.rs`, `src/longrange.rs`, `src/sim.rs` | every radius bit-identical to the dialect it replaced (`tests/channel_ledger.rs`, the old bisection kept in the test as the referee) |
| 3 | The tail exponent read as LAW vs FIT: `ExponentReading` per curve against its channel's derived power, `FarSector::exponent_readings`, and the OPT-IN refusal `FarSector::require_assigned_exponent` → `FarRefusal::ExponentDisagrees`. Nothing in the force law consults it. | `src/channel.rs`, `src/longrange.rs` | a pure `R⁻⁶` fixture agrees; `R⁻⁷` (inside B2's band) refuses by name when asked; sums unchanged |
| 4 | `Sim::energy()` derived from the row table: a left fold over `Row::ALL` starting from the first row, the exact operations of the hand-written chain. `Sim::row`, `Sim::channel_standing` (per-channel rows, values, reach, by which dialect). | `src/sim.rs` | fold == chain, bit for bit; standing reads only the rows it names |

**What was NOT built, by name.** Channel 4's far side (the harvested constant as the
handover past the three-body table's reach); channel 2's fixed point (FIELD-2); any
reordering of any sum; any refusal on by default. The first two are physics and need a
freeze committed alone; the third re-banks every digest and needs one cause line; the
fourth would change banked scenes.

## The receipt

`examples/channel_receipt.rs` prints, for FIELD-1's four-water scene with the field on
after 2,000 steps and for the staked quartet after 256: every ledger row, `energy`,
`ledger`, `w_ext`, `work.field`, `drift`, the physics digest, and the derived pair cutoff
at three floors, all as raw `f64` bits; plus the far sector's closed-form radii at three
budgets on two power fixtures. It was run against the PARENT commit's library (the
refactor stashed) to write `tests/data/channel_ledger.receipt`, then against the
refactored library by the test. `M-STALE-INSTRUMENT`: the receipt, the writer, the scenes
and the gate are committed together.

## Results

All gates GREEN, 2026-09-05, on the branch, release profile.

| gate | verdict | the number |
|---|---|---|
| the receipt — every row, both sums, `w_ext`, `work.field`, `drift`, the physics digest, three derived cutoffs, two scenes; six far radii on two power fixtures | **PASS, bit for bit** | 46 of 46 receipt lines identical; `water4.energy = bff12fe7bd4a6bbc`, `water4.physics_digest = 37d350af22f2f2c6`, `water4.e_field = 3f5ef1987a10a314` (live, nonzero), `quartet.e_three` live |
| the fold is the chain | PASS | `energy()` and `ledger()` identical to the hand-written chain on the four-water scene after 2,000 steps; every row reads its field |
| the sampled arm is the old bisection | PASS | 4 floors posed (1e-6, 1e-8, 1e-10, 3e-7), `r_in` and `r_cut` identical to the bit against the old loop kept in the test; `2.5e-9` and `1e-12` both refused identically |
| the power arm is the old closed form | PASS | 3 exponents × 5 budgets identical to the bit, through `TailModel` and through the allocator directly |
| the standing report | PASS | five channels; every named row's value identical to `Sim::row`; the field WHOLE at `Reach::Scene`; induction has no own row, `Reach::Absent`; three-body `Reach::Radius` from the tables' declared reach |
| the exponent reading | PASS | pure `R⁻⁶` fixture agrees (deviation 0); `R⁻⁷` (inside B2's band) builds and sums as before and the opt-in refusal names it: `REFUSED (channel ledger, exponent)` |
| the declarations | PASS | kinds Circumstances · Structure · Process · Rules · Identity in rate order; powers 1, 4, 6, 9, exponential; shapes sum · fixed point · sum · sum · solve; only channel 1 posts a receipt (`work.field`) |

The existing gates the refactor touches, re-run on the refactored library, dev profile:
`tests/field.rs` 7/7 (FIELD-1's own gates, 1,466 s), `tests/ledger.rs` 12/12,
`tests/b2_longrange.rs` 22/22. The crate's whole test suite in release: RUNNING at this commit (release profile, every test target of `holon-render`); its count is appended to this section by the commit that reads it, never edited into this one. The wasm target (`wasm32-unknown-unknown`) builds with the module in.

### The full suite's first reading, and a defect it found that is not the ledger's

The first full run (release, every `holon-render` target, fail-fast) stopped at
`tests/hadron_band.rs::the_evolution_is_unitary_and_conserves_energy`: 9 targets green
before it, that one 5/6. The test PASSES alone and passed 3/3 on re-run, and `hadron.rs`
imports nothing from `sim`, `longrange` or `channel`, so the ledger cannot have moved it.
Root cause, read from the file rather than called a flake: the sub-atom bands are ONE
process-global object (`hadron.rs::BANDS`, three slots) and five of the file's six gates
solve, step and grab slot 1; Rust runs a file's tests on parallel threads, so under a
loaded box two gates stepped one band at once and the energy-drift gate read the other's
steps as its own drift. `M-STALE-INSTRUMENT`'s third variant one level down — a shared
object read as if it were private. Fix, test-only: the six gates take the band in turn
through a file-local lock (`SERIAL`), poison-tolerant, no physics touched; 3/3 stable
after. The remaining targets are re-run with `--no-fail-fast` and their count is
appended below by the commit that reads it.

## What this does and does not do for the ladder

It changes no number, so it certifies nothing and un-fences nothing. Rung 1 (the H-bond
network) still waits on FIELD-1's S1, which is unread. Rung 2 (the fluid element) is
fenced with numbers (`RUNG2_RESULTS.md` §0, §6: the occupancy/transport scissor; ≥ 400
atoms, 3D, and a format v2 for any successor carrier), and **the 1 km face does not flip
and is not owed** by anything here. What the ledger gives a future carrier is the shape of
the question it will have to answer five times: how far does each channel reach at the
budget the band asks for, asked once, in one format, with the reach a MEASURED or DERIVED
number and never a guess. `Sim::channel_standing` is that question's current answer sheet.
