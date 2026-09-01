# The dE4 reruns — which one is the experiment

**THE REAL RUN IS `engine/output/p2_de4_full/`** (per-seed logs, binary built from
`21e6be3` in a clean worktree, sha256 7790cf3d…, conditions at both ends,
done-markers). That build carries `quaternary::ohhh_fci_grad`: the EXACT Cartesian
gradient from nine seeded dual solves, with the oxygen row by translation
invariance so ΣF is zero to the last bit. It is the first run of the four-body
term at design strength.

## Three arms, and they are three different physics — do not merge their censuses

| arm | gradient | status |
|---|---|---|
| the banked seeds (`seed_0x*.log` here) | radial-only AND divided by mass twice | records of a defective build; the sector was 3–4 orders too weak, so effectively MBE3 plus a small biased perturbation |
| `rerun_momentum_clean.VOID_superseded.log` | radial-only, mass fixed | **VOID.** Killed mid-run. Momentum-clean but still HALF a gradient — no H-H force inside the correction ever reached a trajectory, and it fails force-is-the-gradient at order one |
| `engine/output/p2_de4_full/` | exact Cartesian | THE EXPERIMENT |

The water observed on seed `0x53415422` belongs to row one. It was **not**
observed under a working four-body term, and a disagreement between rows is two
different physics rather than irreproducibility. Whether OH2 survives a sector
that actually pushes is open.

The two old-binary seeds still running (PIDs 3213745 / 3431805) are row one and
are kept deliberately. Do not kill them; do not merge their lines with row three.

## Why the middle row exists at all

It was launched believing the mass-division fix was the whole defect. The audit
that followed priced the path that actually runs and found the gradient was
radial in the three O-H distances only — half the degrees of freedom — so the
force was not the gradient of the energy the ledger sums. A momentum-clean force
that is not a gradient is still wrong; conserving the wrong thing exactly is not
a smaller error than conserving it approximately. The log is kept voided rather
than deleted, because a deleted run is a gap someone re-derives.

## Stdout goes to a FILE, always

Row one's went to a TTY, which is why its per-seed lines existed only in a harness
buffer and a search of the repo tree reported a scope as a conclusion. Every arm
since redirects.
