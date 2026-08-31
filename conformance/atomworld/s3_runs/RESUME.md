# SATURATION-3 (tables and physics) — detached runs

Lane: `saturation3-water`. Contract: `../SATURATION3_PREREG.md` (7bc8d50).
Sibling: `saturation3-mesh` owns G1 (sharded generation) and G2 (GPU).

| marker | producer | what it is | restart |
|---|---|---|---|
| `g0.DONE` | `holon-chem --example s3_g0` | the blocking cost gate: five staked triple types at their worst compact geometry | `engine/target/release/examples/s3_g0 > s3_runs/g0.log` |
| `oo_reexam.DONE` | `holon-chem --example s3_oo_reexam` | the O-O re-examination: SATURATION-2 P1's own 96-knot curve, production cap vs raised cap, scored at the derived bar | `s3-oo worktree: engine/target/release/examples/s3_oo_reexam 20000 > s3_runs/oo_reexam.log` |
| `bar_margin_after.log` | `holon-chem --example s3_bar_margin` | the nine staked curves re-read under the DERIVED bar (no marker; the run is seconds) | `s3-oo worktree: engine/target/release/examples/s3_bar_margin` |

## RUN FROM A PINNED WORKTREE, and why

Both runs above were built and run in `/home/emoore/holon-wt/s3-oo`, a detached
worktree pinned at `179db95` — NOT in the shared tree. An uncommitted refactor of
`fci.rs` (371 lines, the eigensolver made generic over a scalar tier) appeared in
the shared tree at 18:02 today, and a binary built there at 18:03 picked it up. A
measurement of the SHIPPED solver cannot be taken in a tree where the shipped
solver is being replaced. Check `grep -c "crate::tier" fci.rs` = 0 before trusting
a rebuild. Anything reproducing these numbers should use the same pin.

That refactor has since landed as `fe18572`, and both readings were re-taken
against it in the shared tree: `bar_margin_after.log` and `oo_trace.log` are
BIT-IDENTICAL to the pinned worktree's. So the numbers transfer to main, and the
f64 instantiation of the generic solver reproduces the crate's hardest solve —
the O-O iteration-capped knot — exactly, which is a check that lane did not have.

## G0 banked

Determinant counts exact, five of five, against the freeze's committed arithmetic.
All converged at 8.1e-11 to 9.8e-11. Nothing within 8x of the 10x kill.

    (H,H,Cl)     605   0.21 s      (H,Cl,Cl)   3,249   1.25 s
    (Cl,Cl,Cl) 9,477   6.47 s      (O,O,H)     9,075   0.38 s
    (O,O,O)  207,025  39.83 s

Two findings that shape the rest:

  * (Cl,Cl,Cl) is OVER the freeze's "<= 5 s/point" class at 1.29x. It does not
    re-scope; the class STATEMENT needs amending. The boundary is inside
    run-to-run variance -- two runs read 5.67 s and 6.47 s with identical
    residuals and iteration counts.
  * THE TWO COST CLASSES ARE BOUND BY DIFFERENT THINGS. (Cl,Cl,Cl) is 73%
    ASSEMBLE (27 orbitals, basis-bound); (O,O,O) is 99% CI (207,025 determinants,
    determinant-bound). The warm start and the GPU kernel both accelerate
    Davidson, so both can only pay on (O,O,O). Size by BASIS SIZE, not by
    determinant count.

## Staked geometry rule (fixed before any solve)

Each side at 0.75 x that pair's own located R_e. Located values:
H-H 1.3887, H-Cl 2.5369, Cl-Cl 4.0241, O-O 2.4421, O-H 1.9909 bohr.

`homonuclear_radius` must NOT be used for chlorine: it is declared only for
Z <= 10 and silently returns hydrogen's 0.694 bohr for Z = 17.

## THE TRAP, carried forward

`fci::solve` routes past MPS_ROUTE_THRESHOLD = 50,000 determinants into an MPO
builder that reaches six orbitals and HANGS rather than erroring. (O,O,O) at
207,025 over fifteen orbitals is the one staked combo that crosses it. Call
`solve_determinant` explicitly anywhere the space size is not statically obvious.

## Next, per the freeze's sequence

(2) generalize trimer.rs over species (symmetry axis per table; S3 only for the
two homonuclear types) with the warm-start locality sweep -- sized on (O,O,O) per
G0's split; (3) the angle-axis anomaly as a staked design obligation; (4) tables
in cost order; (5) P1-HCl; (6) P2-H2O.
