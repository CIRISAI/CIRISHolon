# The first true dE4 run — momentum-clean

| marker | producer | restart |
|---|---|---|
| `rerun.DONE` | `holon-render --example waterquench -- mixed` | `engine/target/release/examples/waterquench mixed >> p2_de4_seeds/rerun_momentum_clean.log` |

**This is not a reproduction of the banked seeds.** Those ran with the four-body
force divided by mass twice (`e6cbb13`), so the sector was three to four orders
too weak — effectively MBE3 physics plus a small biased perturbation. The water
observed on seed `0x53415422` was NOT observed under a working four-body term.
This run is the experiment those were mistaken for, and whether OH2 survives a
sector that actually pushes is open. Do not read a disagreement between the two
as irreproducibility; they are different physics.

**Stdout is redirected to a file.** The banked arm's was not — it went to a TTY,
which is why its per-seed lines existed only in a harness buffer and my own
"no artifact exists" search reported a scope as a conclusion. Every future arm
redirects.

The two old-binary seeds (`0x...21`, `0x...25`, PIDs 3213745 / 3431805) are still
running under the DEFECTIVE build and are kept deliberately as records of what a
momentum-non-conserving build did. Do not kill them; do not merge their lines
with this run's.

Provenance is in the log header: fix commit, gate, binary sha256, HEAD, launch
time. Expect it to be SLOWER than the banked arm — a sector that was inert now
does real work, so more solves per frame.
