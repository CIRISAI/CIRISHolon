# NODE LG — RESUME

**Owner:** lattice-tier lane. **Worktree:** `/tmp/claude-1000/lg-wt`, branch `lattice-tier`.
**Never push.** Pathspec commits only; cirisontology-b4 integrates.

## State: BANKED

| | |
|---|---|
| prereg | `conformance/mesh/LG_PREREG.md`, **ADMITTED**, frozen `ce392f3`, post-freeze annexe appended |
| instrument | `engine/crates/holon-lattice/`, `c61ddbd`; 32 tests green; CI gate in `engine/ci-gates.sh` |
| results | `conformance/mesh/LG_RESULTS.md`, banked `048a5c2` |
| logs | `lg_full.log` (sha256 `7180b50a…`), `lg_tests.log` (sha256 `9dccd645…`) |
| verdict | every gate PASSED, 1,454,433 checks, `STEPS_COMPLETED`, no kill fired |

Nothing is owed to workbench-engine — the door spec and the landed artifacts are both sent.

## The one loose end, named rather than dropped

**The Zanetti literature check is OWED.** `ref_invariants.py` measures that FHP-I on this
lattice has exactly three linear invariants (gauged: identity → `6L`, HPP-4 → its textbook
`2L+1`), so no staggered linear invariant exists *on this configuration*. But this session's
web-search budget was exhausted before the citation could be read, so nobody in this lane has
seen Zanetti's statement, its model variant, or its scope. **No document here may say the
result contradicts him.** Prereg annexe A4 and `LG_RESULTS.md` §8.3 both carry the boundary.

## Unfinished, post-freeze, gates nothing

`ref_invariants_sweep.py` — the 4608-law version of the invariant measurement, launched via
`sweep.sh`, marker `invariant_sweep.DONE`, output `invariant_sweep.log`. It is an extension,
not a gate; if it never finishes, nothing in the bank changes.

## What must not drift

1. **The first law.** Not a view of the molecular dynamics; never composed through
   `closed_comp`. The seam takes no status from this node, in any branch.
2. **No Navier–Stokes limit.** Only the necessary lattice condition is measured. The exit is
   viscosity, semi-detailed balance, and the `g(ρ) ≠ 1` defect, and it stays named.
3. **`b = L` is the VACUOUS end of the curve**, never its success.
4. **No band state.** This certificate confers none (FSD `b374773`); §12 binds research
   content only.
5. **`field_lg` and this tier are not one object.** Theirs is a chart on the molecular tier.

## If a number is ever quoted from here

Read it out of `lg_full.log`, not out of prose. Two defects in this node were exactly this
failure: a diagnostic that printed a wall length it was not running at, and a results row
drafted beside a live run that carried a *killed* run's figures. Every figure in
`LG_RESULTS.md` §2 was cross-checked against the banked log before it was committed.
