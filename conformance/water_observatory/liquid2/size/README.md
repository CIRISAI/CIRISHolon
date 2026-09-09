# LIQUID-2's size check — and why one box has a log and no record

Four boxes, one process each, at the state point's density on an `n^3` body-centred lattice
carrying `2 n^3` waters. One process per box on purpose: the largest one is killed by the
operating system, and run together it would have taken the other three records with it.

| file | waters | verdict |
|---|---|---|
| `cells3.json` | 54 | door REFUSED by the image rule: the half-edge `11.0976` bohr is under the law's reach `14.0000` by `2.9024` |
| `cells4.json` | 128 | ADMITTED — the campaign's own box, measured here again so every other size is read against a number this phase took itself |
| `cells5.json` | 250 | ADMITTED and priced |
| `cells6.log` | 432 | **no record, and that is the reading**: the process is killed with exit `137` (SIGKILL, the OOM killer) while building the box, BEFORE its door runs, on three independent attempts with 25 GB free |

The 432 fence is measured and not estimated. Peak resident set, by `/usr/bin/time -v` on this
host: `0.929` GiB at 384 atoms and `6.845` GiB at 750 — a factor of `7.37` for an atom ratio
of `1.953`, whose cube is `7.45`. The periodic three-body enumeration is cubic in the atom
count, confirmed to 1 % at two points; carried to 1,296 atoms that is `38.4 x 0.929 = 35.7`
GiB against this host's `31.8`. It is the ENUMERATION that is cubic, not the physics, so the
fence's owner is whoever gives the seam a three-body neighbour list and its exit is a 432-water
box that builds, here or on a larger host.

`LIQUID2_PREREG.md` section 3a carries all of this with the campaign's decision: 128 waters is
the smallest periodic box CT-3's law admits at all, and 250 waters is pre-committed as a
one-seed size check inside the price ceiling.

## The fence is DISCHARGED, and the paragraph above named the wrong owner (2026-09-08)

The second review's build order (`GANTT2.md`, step 2) asked for the cost to be INSTRUMENTED
before anything was changed. It was, and the instrument refutes the attribution above while
confirming its arithmetic. `cost_before.json` and `cost_after.json` carry the readings, one
box and one arm per process (`VmHWM` is monotone, so two of either in one process would read
the first one's peak as the second one's); `cost_{arm}_cells{n}.json` are the fragments they
gather, each written by its own run of `liquid2 cost`.

**The cubic term was the PLACEHOLDER, not the enumeration.** `quartet::scene` opened every
scene with `Sim::reset(n)`, which lays a configuration nobody asked for — a ring or shell of
radius **6 bohr whatever the atom count** — and evaluates the forces on it before the real
coordinates arrive. The three-body enumeration has been cutoff-local since T3; it only
degenerates to the complete `C(N, 3)` when every atom is inside every other's cutoff, and 750
atoms on a 6-bohr shell is exactly that. The stage table says so without ambiguity: at 750
atoms the peak stands at `0.079` GiB when `reset` is entered and at `6.842` GiB when it
returns, and NOTHING after it — the real coordinates, the field, the seam, the boundary, the
force passes — adds another `0.01` GiB.

| | 128 waters (384 atoms) | 250 waters (750) | 432 waters (1,296) |
|---|---|---|---|
| peak RSS, `reset` first | `0.953` GiB | `6.852` GiB | killed by the OOM killer |
| peak RSS, geometry first | `0.192` GiB | `0.445` GiB | **`0.947` GiB** |
| construction seconds, `reset` first | `2.69` | `17.17` | — |
| construction seconds, geometry first | `0.81` | `2.32` | `3.39` |
| `fenced_triples`, enumerated | `0.0461` s | `0.304` s | `0.999` s |
| `fenced_triples`, census | `1.3e-5` s | `2.0e-5` s | `2.0e-5` s |
| core-seconds per picosecond | `2184` | `4275` | `4337` |
| the same with the enumerated fence | `2405` | `5732` | `9127` |

**So the 432-water box builds.** `cells6` was re-benchmarked after the change and its peak is
`0.947` GiB, against the `35.7` GiB the paragraph above projected and against three killed
attempts with 25 GB free. The fence's exit is discharged, and by the opposite repair to the
one it named: not a three-body neighbour list for the seam — the seam already had one — but a
constructor that installs the coordinates it was handed before it evaluates a force
(`Sim::reset_with`, `quartet::scene_placed`). The `reset` placeholder is untouched and still
serves the callers that open on it.

`fenced_triples` was separately cubic in TIME on the seam's scenes, and is now arithmetic on
the species-and-unit census (the drop rule is a fact about membership, not position), held
against the enumeration it replaced — kept as `Sim::fenced_triples_enumerated` — by
`tests/fence_census.rs` on nine small scenes and on this box. At 1,296 atoms it was costing
more than the force pass it rode on.

**What did NOT move.** The two constructors build the same scene bit for bit, machine-checked
in every `cost` run (`arm_disagreements`, `production_disagreements`, both `0` at every size,
over coordinates, velocities, forces, energies, `l0`, `p0`, `l0_ang` and the step). The `door`
blocks of `cells4.json` and `cells5.json` re-run field for field identical. The L0 scene's
cross-unit rows are identical to `gate_provisional_8x/expectation.json` to every recorded
digit — `field_part` `1.235166962e-2`, `seam_part` `3.047021222e-1`, `per_water`
`2.476982749e-3`. The engine's own `tests/data/channel_ledger.receipt` reproduces bit for bit,
which is WHY `quartet::scene` keeps its placeholder: the receipt's `quartet` block is stepped
without a `rebase`, so it reads the placeholder's `l0` and moving it is a decision about a
banked record rather than about the cost of construction.

`seconds_per_pass` in `cells4.json` and `cells5.json` is NOT re-banked: those runs' door
blocks reproduce exactly and only the timing differs, and a re-run here read `0.2408` s
against the banked `0.3400` at 128 waters and `0.4872` against `0.6094` at 250 — on a host
carrying three other lanes, so the improvement is a lower bound and the banked numbers stay as
the campaign took them.

## The core-seconds per picosecond above are SUPERSEDED (2026-09-08): they were taken at the withdrawn `8x` step

**Every `core-seconds per picosecond` in this directory's table and in `cost_after.json`,
`cost_before.json` and the `cost_{arm}_cells{n}.json` fragments was measured at
`STEP_MULT = 8.0`.** That step is withdrawn: under the smooth serving rule the drift is
quadratic in the step again and `ct3/smooth/nve.json` selects `1x`, the tables' own
(`"step_chosen": 1`). A core-second per picosecond is `seconds_per_pass / picoseconds_per_pass`,
and `picoseconds_per_pass` is the STEP — so the `8x` numbers divide the same seconds by eight
times the physical time.

**The direction, stated plainly, because the review that ordered this re-price had it
backwards.** The stale numbers do not overstate the cost of a picosecond; they **UNDERSTATE**
it, by the step ratio. At a fixed cost per force pass, `1x` costs exactly `8x` more per
picosecond than `8x` does, because a pass buys an eighth of the time.

**RE-MEASURED at the selected configuration** (`cost_geometry_first_1x_cells{4,5}.json`:
`CtServe::Blend` at the derived beta, `ThermostatKind::StochasticRescaling`, `1x` step, every
one of them read back out of the objects and written into the record), **with an `8x` CONTROL
RE-RUN IN THE SAME MIX** (`cost_geometry_first_8xctl_cells{4,5}.json`). The control is the
point: seconds per pass move with placement by more than the term being measured
(M-PLACEMENT-LOTTERY), so a `1x` reading held against a record taken on a differently loaded
host measures the host as much as the step. Held against a control in its own mix, the step is
the only thing that moved.

| | s/pass, `1x` | s/pass, `8x` control | ratio | core-s/ps, `1x` | core-s/ps, `8x` control | ratio | core-s/ps, BANKED `8x` | `1x` over banked |
|---|---|---|---|---|---|---|---|---|
| **128 waters** | `0.2607` | `0.2471` | `1.055` | `10003.6` | `1185.1` | `8.441` | `2183.8` | `4.581` |
| **250 waters** | `0.5294` | `0.5155` | `1.027` | `20312.1` | `2472.6` | `8.215` | `4274.8` | `4.752` |

The `1x` / `8x`-control ratio on core-seconds per picosecond is the step ratio times whatever
the blend costs over the argmin at the same step; the `s/pass` ratio in the same row is that
second factor alone, measured. **The banked column is kept and is not corrected in place**: it
is what that phase measured at the step it ran, the records stay bit-identical, and this
section is the conversion. Nothing in `cost_after.json`, `cost_before.json` or the
`cells{3,4,5,6}.json` size records is edited.

The `1x` records are written by `liquid2 cost --tag _1x`; the control by
`liquid2 cost --tag _8xctl --selection banked`. Both flags exist so that a re-measurement can
never land on a banked record's name.
