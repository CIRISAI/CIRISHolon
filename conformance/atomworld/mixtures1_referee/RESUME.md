# MIXTURES-1 REFEREE — resume state

Scratchpad lane: `/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/mixtures_referee`
Contract:        `conformance/atomworld/MIXTURES1_PREREG.md` (frozen, admitted)
Sibling lane:    `elements1/` — a symlink to `../elements_referee`; this lane
                 RUNS that code rather than copying it.
Source before any Python here: `. ./env.sh` (one BLAS thread per process).

## Gate R1: DONE

H through Ar, 50 digits, dual route, every Sz sector. Worst |A−B| across the
eighteen: 3.1e-55. Ground multiplicity derived TWICE — from the Sz degeneracy
pattern and from ⟨S²⟩ of the converged vector — agreeing on all eighteen, which
is the only check that can see a subspace method converging cleanly onto a
spin-EXCITED state. What came out is Hund's rules down both rows (C/Si triplets,
N/P quartets, O/S triplets, closed shells at He/Ne/Ar) from Z and a basis table.

**Ten of ten first-row energies are BIT-IDENTICAL to ELEMENTS-1's** at all 50
digits, so extending the table to Z = 18 provably touched nothing the first row
uses. Product: `mixtures_atoms.json`.

## Built and green, before any energy exists

| | checks |
|---|---|
| `test_basis_matches_engine.py` | 23, 0 FAIL — the declared model vs the engine's, by PARSING `elements.rs` |
| `test_species_shim.py` | 30, 0 FAIL — the stand-in module's surface, the table binding, the cache separation, the grid rule |

Both were written and run BEFORE the first energy, which is the only time a
model-agreement check is worth anything.

## The three findings that shaped the lane

**The second row needs no d-orbital integrals.** The brief said it did. The
frozen prereg proves otherwise on its own: it states Ar2 is ONE determinant,
which is true only if argon carries 9 basis functions — 1s 2s 2p 3s 3p — since
C(18,18)² = 1. Its Na2 figure agrees (C(18,11)² = 1.013e9). The engine's
`second_row!` macro declares exactly those five shells and no species uses
`ShellKind::D3`. Adding d functions "to be safe" would have been a DIFFERENT
MODEL from the engine's, and every R1/R2 comparison would have failed looking
like an integral bug.

**The engine's scope now exceeds the referee's.** `elements.rs` grew past argon
during this lane's first pass — 36 elements up to xenon, most with a d shell.
Na..Ar is untouched and still agrees exactly. But a gate reading "the engine
matches the referee" is VACUOUSLY true for any pair the referee cannot compute,
so `basis2.shells_for` raises on an out-of-scope Z and the manifest states the
bound as a fact.

**SiO is NOT FEASIBLE as staked, measured and with both obvious remedies
tested.** Full writeup in `FEASIBILITY.md`. Measured, not projected: nnz =
196,889,056 (the estimator predicted exactly that, as it did N2's 8,784,000 and
HCl's 10,000), 12.08 µs/element, ONE working-precision matvec = 2378 s. N2's
real dual-route points cost 25–116 matvecs, putting SiO at 20–91 hours per
geometry × 15 knots, ~5.6 GB peak per worker. Roughly 1000–4000 core-hours.

Both remedies came back negative. HOISTING the generator's redundant work
(`_fast_elements.py`, stream verified IDENTICAL element-for-element on H2, HCl,
ClF) gives 1.34× / 1.19× / **0.93×** — the excitation bookkeeping is not the
cost, the 2e8 Python tuple yields and mpf multiply-adds are. ROUTE B
(`_routeb_cost.py`) measures B/A = 17.3× at ndet 100, 45.2× at 324, 1.38× at
14,400 — it closes with size but never wins. The cost is structural to the
design; only a string-driven sigma that never enumerates an element would move
it, and that is a rewrite of arithmetic two live campaigns depend on.

Reported to the lead with three options; recommended amending D1's second
overlap species to **NaH** (44,100 determinants — the largest FEASIBLE space in
the staked set, larger than S2's 23,409 — whose table is being built anyway for
R2, so D1 gets its second species at zero extra cost). A frozen stake is not
this lane's to change.

## How this lane runs ELEMENTS-1's machinery without forking it

`build_curves.py` does `import species as SP`. This directory ahead of
`elements1/` on `sys.path` makes that resolve to the shim, and the whole
pipeline — run lock, pool guard, merge-not-narrow, grid_provenance
regeneration, the assembler, the gates — runs on this campaign's species with no
edit to code a live campaign is executing. `curves2.py` arranges the four
things that would otherwise be wrong:

1. `sys.path` — or it would run ELEMENTS-1's nine species and succeed at it;
2. `build_curves.HERE` — or it would take ELEMENTS-1's run locks and OVERWRITE
   its assembled potential file. Wrong here is damage, not a wrong number;
3. the table and cache bindings (via `m1core`) — `table=None` is rebound to mean
   Z = 1..18, because the shared code calls `runner` without a table and a
   first-row-only chlorine would report a higher energy, perfectly converged for
   the wrong model, with no exception;
4. the model string — `build_curves.py` stamps `ELEMENTS1/STO-3G/FCI` as a
   literal; `restamp_model()` corrects it after the assemble and keeps the
   original.

Two filenames are INHERITED rather than fought: `elements_atoms.json` (a symlink
to `mixtures_atoms.json`) and `elements_potential_partial.json`. Both are
checked by CONTENT — the model string — before anything reads them.

## Live

- `run_pairs.sh` (detached, markers in `markers/`, log `run_pairs.log`) — waits
  for `mixtures_atoms.json`, then the five cheap pairs together, then S2 and
  NaH one at a time. **SiO is deliberately not in it.** `NPROC_LIGHT=4
  NPROC_HEAVY=3`: ELEMENTS-1 is priority one on this machine and holds twelve.
- `build_atoms2.py` — gate R1, DONE (2877 s, 4 workers).
- `_sio_stream.py` — the SiO feasibility measurement, DONE.

Cheapest-first is not impatience: it means the engine lane has real referee
tables long before the expensive tail, and any defect in the shared pipeline
shows up on a pair that costs an hour to redo rather than one that costs a week.

## The staked design

| pair | knots | ndet | nnz | rule |
|---|---|---|---|---|
| HCl | 176 | 100 | 1.0e4 | BOUND |
| ClF | 160 | 196 | 3.8e4 | BOUND |
| Cl2 | 83 | 324 | 1.0e5 | BOUND |
| Ar2 | 80 | 1 | 1 | NEGATIVE |
| NeAr | 80 | 1 | 1 | NEGATIVE |
| S2 | 15 | 23,409 | 3.1e7 | BOUND, sparse |
| NaH | 15 | 44,100 | 3.6e7 | BOUND, sparse |
| SiO | 15 | 132,496 | 2.0e8 | BOUND, sparse |

624 geometries. Every window is a function of one declared `R_ref` and a fixed
rule, so a grid regenerates rather than being promised — checked per pair in
`test_species_shim.py`. Conditioning at every staked inner edge is measured and
admissible; the worst is Ar2 at 1.42 bohr with s_min = 0.0096, against the
0.0062 ELEMENTS-1 ran Li2 on.

## Standing rules for this lane

- Never trade digits for knots: a sparse exact referee is a referee.
- Measure the cost, do not infer it from the determinant count.
- A guard must demonstrate a failing case AND be shown to be connected.
- Two campaigns, two tables, two caches, two fingerprints. "Refused" and
  "never offered" are different states and this lane wants both.
- A frozen stake is not this lane's to change. Measure it and hand the number up.
