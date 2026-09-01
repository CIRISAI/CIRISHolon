# `SATURATION3/trimer-table/v1` — proposed shipped artifact for heteronuclear three-body surfaces

*RULED AND BUILDING (lead, 2026-08-30): this lane owns the schema and the emitter;
`render-3d` owns the wasm loader and the fence lift, working against this document as the
interface. Written by `saturation3-mesh`.*

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

  // THE AXIS RULE AND THE COORDINATES THEMSELVES. Spans plus counts do NOT determine node
  // positions -- they say nothing about interior spacing -- so a consumer handed only
  // corners must GUESS uniform. That guess is WRONG for this build's H3 table, which places
  // r by `trimer::r_of_tau` (STRETCH_A = 2.0) and its angle axis by `node_c`; a loader
  // assuming uniform on a stretched grid interpolates smoothly, plausibly, and wrongly
  // everywhere except the boundary.
  //
  // THIS generator is uniform-linear, which is NOT the H3 rule despite node counts that can
  // match it. So the rule is NAMED and the coordinates SHIP -- 79 floats against 14,157
  // energies -- and the coordinates are authoritative where they disagree with the name.
  // render-3d asked for this and was right to; it blocks interpolation without it.
  "axis_rule": {"x": "uniform-linear", "y": "uniform-linear", "u": "uniform-linear",
                "note": "NOT trimer.rs's tau-stretch; the coordinates below are authoritative"},
  "x_nodes": [], "y_nodes": [], "u_nodes": [],
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

  // THE SEAM RECORD (water's state-crossing ruling, adopted by the lead 2026-08-30).
  // A seam is a STATE CROSSING -- a slope discontinuity where a reactive channel opens.
  // A cubic interpolant across one has an error floor set by the jump, so uniform
  // refinement CANNOT beat it: paying for resolution past the seam buys nothing. Every
  // trimer type here has a reactive channel ((H,H,Cl) and (H,Cl,Cl) most obviously), so a
  // shipped surface must say where its corners are or that it knowingly accepted a floor.
  // EXACTLY ONE of `loci` or `accepted_floor` must be present and non-empty.
  "seams": {
    "scanned": true,
    "instrument": "s3_angle_slice",
    "loci": [
      {"axis": "u", "at": 1.37, "kind": "state-crossing",
       "grid_line_placed_on_it": true,   // the domain is split into smooth patches here
       "jump_hartree": 0.0}
    ],
    // ...OR, when the seam is located and the floor is taken deliberately:
    "accepted_floor": null
    // {"hartree": 1e-4, "why": "seam sits inside the last cell; splitting costs more grid
    //  than the feature is worth", "located_by": "s3_angle_slice, slice u=…"}
  },

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
| `axis_rule` or the `*_nodes` arrays absent, or an array whose length disagrees with its `grid` count | spans and counts do not determine spacing; a loader would have to guess uniform, and on a stretched grid that guess is wrong everywhere but the boundary |
| `grid.region` absent | the table's identity is incomplete and two different artifacts could collide on one digest |
| `seams` absent, or carrying **neither** `loci` nor `accepted_floor` | a shipped surface with a **hidden corner** is exactly what this gate exists to see. Uniform refinement cannot beat a state crossing, so a table that has not located its seams is claiming a smoothness it has not checked |
| `seams.scanned: false` with a non-empty `loci` | the loci came from somewhere other than a scan and the source is unstated |

## Owner split, as I read it

| piece | owner |
|---|---|
| the schema and the emitter (this document, and `s3_tables` writing JSON instead of `.tbl`) | **mine — ruled 2026-08-30, building now** |
| a loader so the wasm consumes shipped surfaces | **`render-3d`'s — ruled, assigned, working against this spec** |
| `holon_trimer_h_only()` returning false once a heteronuclear surface is loaded — **the thing that actually lifts the fence** | **`render-3d`'s** |

The interface between us is this document at its commit; nothing waits on all four tables,
since the loader can land against the schema with the first one.

## Status

Nothing here is implemented. `s3_tables` currently writes a plain-text `.tbl` with a comment
header — adequate for a campaign artifact, **not** adequate for a shipped one the page's
provenance gate reads. Converting the emitter is small and is mine the moment the shape is
ruled.

---

## The coordinate convention, stated (render-3d's last blocker, 2026-09-01)

Read from `holon-tables/src/surface.rs::TrimerSurface::realise`, which is the code that
turns a node's three coordinates into atomic centres. It is authoritative; this section
restates it and adds nothing.

    centres = [ [0, 0, 0],  [x, 0, 0],  [y*u, y*s, 0] ]      s = sqrt(1 - u^2)

**`u` IS THE COSINE of the angle between the two short sides. It is not the angle, and it
is not `sqrt(1 - cos theta)`.** The axis is named `u` rather than `cos_theta` for brevity
and that was a mistake worth naming, because a neighbouring lane's domain sweep
parameterises the same angle as `c = sqrt(1 - cos theta)` over [0.05, 1.4142]. Those are
different variables on the same axis. A domain handed over in `c` and consumed as `u`
would place every grid line in the wrong spot — smooth, plausible, and wrong everywhere.
The emitter therefore ships `axis_rule` and the explicit `u_nodes` array, and the
coordinates are authoritative wherever a name and an array disagree.

**Which species x and y are measured from.** `species[0]` sits at the origin and is the
APEX — the atom the table's symmetry does not exchange. `x` is the apex-to-`species[1]`
side; `y` is the apex-to-`species[2]` side; `u` is the cosine of the angle between them,
at the apex. So for `--species Cl,H,H`, Cl is the apex and both axes are Cl-H distances,
which matches the neighbouring lane's own apex convention.

**The third side is not stored and must be derived** by the law of cosines, which the
centres above satisfy exactly:

    r_23 = sqrt(x^2 + y^2 - 2*x*y*u)

A consumer that wants three side lengths computes that one; it is not in the artifact
because storing a derived quantity invites the two copies to disagree.

**Units** are bohr for `x`, `y` and `r_23`; `u` is dimensionless on [-1, 1]; energies are
hartree.

**Grids are ARBITRARY and uniform-linear.** `nx`, `ny`, `nu` come from the domain and are
not the engine's `trimer::NR/NU`, and the spacing rule is uniform-linear rather than
`trimer.rs`'s tau-stretch. A consumer must size its storage from the artifact and
interpolate on the shipped coordinates. `holon_chem::trimer::TrimerTable::eval` is NOT a
valid evaluator for these surfaces on two independent counts — it assumes tau-stretched
axes and it takes three side lengths where these axes are `(x, y, u)`.
