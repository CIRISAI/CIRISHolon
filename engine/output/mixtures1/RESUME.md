# MIXTURES-1 engine lane — detached runs

Everything here was launched with `setsid nohup` and writes a `.DONE` marker on
completion, so a dead session kills narration and never computation. Logs are
line-flushed (`say!`), so a killed run keeps its partial results.

Campaign record: `conformance/atomworld/MIXTURES1_RESULTS.md`.
Frozen protocol: committed at `0851ccb`, **before** any arm ran.

## Complete

| marker | what it measured | verdict |
|---|---|---|
| `p1_hydrogen.DONE` | P1 control, 16 H, 8 seeds | H2 modal in 8 of 8; control PASSES |
| `p1_chlorine.DONE` | P1 control, 16 Cl, 8 seeds | Cl16 in 8 of 8; control PASSES |
| `p1_mixed.DONE` | P1 product, 8 H + 8 Cl, 8 seeds | **branch (b)**: HCl modal in 0 of 8 |
| `p1_diagnose.DONE` | post-hoc branch-(b) diagnostic, seed 1 | 74 bonded edges; 21 of 40 H–Cl within 0.5 a₀ of `R_e` |
| `p1.DONE` | all three P1 arms finished | — |
| `d1_overlap_H2.DONE` | D1 accuracy on H2, 16 points | worst 5.05e−13 Ha vs 1e−8 stake |

`mpo_cost_H2.log`, `mpo_cost_LiH.log`, `mpo_cost_HCl.log` carry the MPO-builder
cost measurement that decides D1: 0.00 s at 2 orbitals, 528.48 s at 6, did not
finish in over an hour at 10. `mpo_cost_HCl.log` was left running past its budget
and has no `.DONE`; the reading ("did not complete") is the result, and finishing
it would only sharpen a number that is already past the point of usefulness.

## Still running

| log | what | how to read it |
|---|---|---|
| `d1_overlap_LiH.log` | D1 accuracy on LiH, 12 grid points | one row per point; `d_chi64` is the column that matters. ~600–700 s per point, dominated by the MPO build. Ends with a `# WORST` line and `d1_overlap_LiH.DONE`. |
| `e2_ordering.log` | E2's well-depth ordering, 9 staked pairs | one row per pair. **S2 is the slow one** (18 basis functions, 23,409 determinants) and may take hours; NaH (44,100 determinants) likewise. Ends with `# MEASURED ORDER` and `e2_ordering.DONE`. |

### What to do when they land

* **`d1_overlap_LiH`**: fill the LiH row of the D1 table in `MIXTURES1_RESULTS.md`.
  It does NOT discharge D1 — the freeze stakes S2 and SiO, which this engine's
  MPO builder cannot reach — and the section already says so. Do not promote it.
* **`e2_ordering`**: fill the E2 table. One inversion is already visible and is
  the thing to look at: the stake says `ClF > Cl2`, and the measurement so far
  says **Cl2 0.064577385 > ClF 0.060622391**, a 6.6% adjacent swap. That is an
  inversion but not obviously a GROSS one, and E2's branch (b) is worded for gross
  inversions; say which it is rather than rounding the judgement either way.

### Re-running any of it

```
cd engine
./target/release/examples/mixquench <mixed|hydrogen|chlorine|cost|diagnose>
./target/release/examples/e2_ordering [PAIR ...]
D1_CHI=8,16,32,64 ./target/release/examples/d1_bridge <cost|probe|overlap|curve> <PAIR> [n]
```

Every parameter these accept is echoed in the run's own header, so a log says
which run it is. The P1 protocol's constants are NOT settable — they are `const`
in `mixquench.rs`, which is what makes a reported run re-runnable byte for byte.

## Not owed by this lane

`mpo_cost_HCl.log` has no `.DONE` by design (above). Nothing else here is
outstanding: R2 is blocked on the sibling lane's drop landing in
`engine/crates/holon-chem/tests/data/mixtures1/`, which is not a run.
