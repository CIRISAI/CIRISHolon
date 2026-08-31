# `SATURATION3/trimer-table/v1` — proposed shipped artifact for heteronuclear three-body surfaces

*A PROPOSAL, not a shipped schema. Placement and ownership are the lead's to rule; this
exists so the ruling is made against a concrete shape rather than a description. Written by
`saturation3-mesh`, 2026-08-30.*

## Why a new class rather than a row in the pair manifest

`docs/atoms/tables/manifest.json` is `MIXTURES1/pair-table/v1` and its provenance gate knows
one shape: two species, one `R` axis, 192 knots. A three-body surface is a different object —
three species, a `(x, y, u)` box, a region shape that is part of the table's identity — and
feeding it to that gate would hand a validator a shape it cannot check while it reports
success. Separate manifest, separate schema, same discipline.

**The page is already waiting for this.** `docs/unified/app.js:257` computes H3 in the browser
(9 determinants, affordable) and, while `holon_trimer_h_only()` is true, prints a fence saying
heteronuclear trimer surfaces are a named successor. The comment at `:261` states the fence is
read from the engine precisely so it lifts itself when such a surface lands. (H,H,Cl) at 605
determinants and (Cl,Cl,Cl) at 9,477 are not browser-affordable, so shipping is the route.

## The one thing this schema deliberately does NOT copy from the pair schema

`HCl.json` carries `converged: true` and a convergence block whose deepest field is
`worst_davidson_residual`. **That is the shape M-EXIT-DISCRIMINATOR names.** Measured this
campaign: every heavy all-electron solve exits `Stagnated`, never `Converged`, at a residual
just under 1e-10 — because the Davidson's expansion floor is scale-free. A bare `converged`
boolean derived from a residual is a threshold masquerading as an outcome, and the same
defect was corrected in G0 this afternoon.

So this schema carries **`exit_reason` per node** and a histogram at the top level. There is
no top-level `converged` boolean, because for these systems it would be false everywhere and
mean nothing.

## Proposed shape

```jsonc
{
  "schema": "SATURATION3/trimer-table/v1",
  "provenance": "engine-computed STO-3G FCI (determinant, Knowles-Handy), f64, generated through holon-tables' leased mesh",
  "solver_route": "determinant",
  "exact_in_model": true,
  "model": "(H,H,Cl)/STO-3G/FCI",
  "species": [ {"symbol": "H", "Z": 1}, {"symbol": "H", "Z": 1}, {"symbol": "Cl", "Z": 17} ],
  "symmetry_axis": "two-identical-one-distinct",   // or "S3"
  "n_determinants": 605,

  // THE DOMAIN IS A PHYSICS CLAIM AND CITES ITS SOURCE. A table whose domain names no
  // curve is refused: "derived from the pair curve" without naming the file is a claim
  // nobody can re-check.
  "domain": {
    "x_bohr": [2.0, 6.0], "y_bohr": [2.2, 6.4], "u": [-1.0, 0.6],
    "derivation": "A1 second-smallest-side logic with each pair's measured tail",
    "cited_curves": [
      {"file": "docs/atoms/tables/HCl.json", "sha256": "..."},
      {"file": "docs/atoms/tables/Cl2.json", "sha256": "..."}
    ]
  },

  // REGION SHAPE IS PART OF THE TABLE'S IDENTITY, not a scheduling note: it fixes the warm
  // chains and therefore the trailing bits. Two tables differing only here are different
  // artifacts and the digest says so.
  "grid": {"nx": 33, "ny": 33, "nu": 13, "region": [3, 3, 13], "n_nodes": 14157},
  "warm_policy": "canonical-chain",

  // THE CERTIFICATE, so a consumer verifies rather than trusts.
  "digest": "1504da0f…",              // merge digest over index+energy+derivatives+status
  "digest_covers": ["node", "energy", "d1", "d2", "status"],

  // M-EXIT-DISCRIMINATOR: reasons, not a boolean.
  "exit_histogram": {"converged": 0, "stagnated": 14157, "iteration_cap": 0, "trivial": 0},
  "worst_davidson_residual": 9.8e-11,
  "expansion_floor": 1e-10,           // the tier edge these solves stop at, named
  "uncertainty_hartree": 1.0e-10,     // WEIGHED: refused if >= the feature it would describe

  // M-BUDGET-LAUNDER: VOIDs are listed, never averaged away.
  "voided": {"count": 0, "nodes": [], "reasons": {}},

  "units": "Hartree atomic units: x,y in bohr, u dimensionless, dE3 in hartree",
  "generation": {
    "workers": 32, "wall_s": 0.0, "cold_solves": 0, "warm_solves": 0,
    "binary_sha256": "…", "repo_head": "…", "build_exit_status": 0, "tree_dirty": false
  },

  "node": [0, 1, 2, "…"],             // canonical index
  "dE3_hartree": [],                  // the three-body term
  "d1": [], "d2": []
}
```

## What a validator should REFUSE

| condition | why |
|---|---|
| `domain.cited_curves` empty or a file that does not exist | a domain with no cited curve is a physics claim nobody can re-check |
| `uncertainty_hartree` ≥ the feature depth it would describe | the pair manifest already weighs this; same rule |
| `digest` absent, or not reproducing over the arrays | the certificate is the point |
| a top-level `converged: true` | see above — this schema has no such field, and one appearing means somebody derived an outcome from a threshold |
| `voided.count > 0` with an empty `nodes` list | a VOID that is counted but not named has been averaged away |
| `grid.region` absent | the table's identity is incomplete and two different artifacts could collide on one digest |

## Owner split, as I read it

| piece | owner |
|---|---|
| the schema and the emitter (this document, and `s3_tables` writing JSON instead of `.tbl`) | mine |
| a loader so the wasm consumes shipped surfaces | `holon-render`'s |
| `holon_trimer_h_only()` returning false once a heteronuclear surface is loaded — **the thing that actually lifts the fence** | `holon-render`'s |

I have built none of the second or third and will not without a ruling.

## Status

Nothing here is implemented. `s3_tables` currently writes a plain-text `.tbl` with a comment
header — adequate for a campaign artifact, **not** adequate for a shipped one the page's
provenance gate reads. Converting the emitter is small and is mine the moment the shape is
ruled.
