# Periodic-table availability — PROBED

*GANTT node D's receipt. Every number below is read off the elements registry by a
program, not asserted from memory. Regenerate it and diff; that is the point.*

## How to regenerate

```
cargo run --release -p holon-chem --example periodic_availability
```

The document body below is that command's stdout, verbatim. The generator is
`engine/crates/holon-chem/examples/periodic_availability.rs`; the row-count gate is
`the_generator_runs_and_covers_every_registered_species` in
`engine/crates/holon-chem/tests/periodic_availability.rs`.

## Provenance of the run that produced this body

| field | value | |
|---|---|---|
| generated_utc | 2026-09-02T00:25:07Z | MEASURED |
| binary | `.target-bf/release/examples/periodic_availability` | MEASURED |
| binary_sha256 | `784a5d1bcd22503b24244c33e1038c1e0a53ff6b5d8401537decf74a65895a0b` | MEASURED |
| repo_HEAD | `7c2b87a6792fafa1a470dca7c69952975ddf8245` | MEASURED |
| build_exit | `0` (`cargo build --release -p holon-chem --example periodic_availability`) | MEASURED |
| tree_dirty_rows | 5 (this lane's four new files, untracked, plus the private `CARGO_TARGET_DIR`; no tracked file modified) | MEASURED |
| run_exit | `0` | MEASURED |

The build's exit status is recorded beside the hash per M-PROVENANCE-OVERREACH
(`conformance/gravity/MISFITS.md`): a true sha256 printed beside a HEAD it was not built
from is more confidently wrong than a timestamp. `build_exit = 0` is what licenses reading
the HEAD line as describing these bytes; the inference from hash to HEAD is an INFERENCE
and is labelled one here.

## What this table is, and the sentence that bounds it

**Route classification is arithmetic, not a certification.** A row reading `FCI-DIRECT`
says one thing: the neutral atom's determinant space, counted from the registry's declared
shells, is small enough that `fci::solve` would take the determinant route. It does not
say that solve has been run, that it converged, that it agrees with a second route, or
that anything about the species is fit for use. Certification remains per-species campaign
work and this document is not it — ELEMENTS-3 priced the difference when indium came back
at a 3.98e-1 residual, nine orders above `pair::CONVERGED_RESIDUAL`, in a row otherwise
indistinguishable from a measurement.

Everything here is counting: `n_basis` sums the declared shells' `n_functions()`, the
electron split is `pair::electron_counts` (the solver's own function), and
`n_det = C(n_orb, n_alpha) * C(n_orb, n_beta)` is the same arithmetic
`pair::automatic_route` spends at its door. No solve runs. The thresholds are read live
out of the crate and printed beside their values in the body, so the comparison can be
checked rather than trusted.

Two of the numbers are independent reproductions of figures already pinned in the source,
which is the check that the probe reads the registry the way the engine does:
xenon's atom at **one** determinant and eighteen orbitals half-filled at **23,409** are
both stated in `elements.rs`'s and `pair.rs`'s own headers, and both fall out of this
arithmetic without being told to.

## The relativistic fence, staked at Z = 36

**Rows with Z > 36 (past krypton) carry NON-RELATIVISTIC-MODEL-FENCE whatever their
route.** The reasoning, in one sentence: this engine's Hamiltonian is the non-relativistic
electronic one — `md.rs` and `fci.rs` build kinetic, nuclear-attraction and two-electron
Coulomb integrals over real contracted Gaussians and contain no mass–velocity, Darwin or
spin–orbit term anywhere — while the leading correction scales as (Zα)², which is already
about 7% at krypton and grows through the 5th and 6th rows into the regime where
relativity stops being a correction to the chemistry and becomes a determinant of it.

Where the line falls is a STAKE, and a deliberately CONSERVATIVE one rather than a
measurement. Two things about it should be said plainly, because the first invites an
obvious objection and the second is what the fence actually claims:

**Z = 36 is not where the error becomes non-negligible — it is later than that.** On the
bare (Zα)² scaling the correction is already ~2% at argon, in the 3rd row, so a stricter
line could defensibly sit lower. The stake is placed at 36 because it is a row boundary
the registry already has, it sits below the 5th row where relativistic effects are
uncontroversially decisive, and it was written down before any heavy-element campaign ran
so that a later result cannot move it.

**So the fence does NOT say that answers below it are relativistically converged.** It
says this model declines to offer answers above it. A Z = 20 row reading FCI-DIRECT
carries the same missing Hamiltonian terms as a Z = 50 row; what differs is only that
their size is small enough to be somebody's stated error budget rather than the answer.
Nothing here measures that budget, and quantifying it is exactly the measurement this
engine cannot currently perform — it would need the same species solved both ways, and it
has only the one way.

**This is a MODEL fence and its exit is named: the relativistic solver rung** (a
scalar-relativistic one-electron correction first — ZORA or Douglas–Kroll–Hess — which
needs no new two-electron machinery; spin–orbit second, which does). It is far, and it is
not permanent, and the difference between those two words is the whole of this document's
discipline. It is NOT a compute fence: no amount of GPU makes a Hamiltonian
relativistic.

## The second availability axis: scene radius

`scene r` reads `NONE` for 44 of the 54 registered species. `Species::homonuclear_radius`
is a MEASURED quantity — half the located `R_e` of the homonuclear pair — and the palette
carries ten. A row with an electronic route and no radius has a solvable atom that cannot
be placed in a scene, which is a different kind of unavailable from a determinant count,
and it is why heavy-element scenes do not follow from this table alone. Extending it means
measuring 44 homonuclear equilibria, not writing 44 more match arms.

## Body — the generator's output, verbatim

```
# PERIODIC-TABLE AVAILABILITY — probed from holon_chem::elements::ALL_ELEMENTS
#
# Constants READ LIVE from the crate, printed so the comparison can be checked:
#   fci::MPS_ROUTE_THRESHOLD  = 50000   (n_det above this: fci::solve routes to MPS/DMRG)
#   fci::HARD_DETERMINANT_CAP = 2000000   (solve_determinant refuses outright above this)
#   fci::MAX_ORB              = 64        (the string machinery's orbital ceiling)
#   pair::MPS_MAX_DETERMINANTS= 1024      (MEASURED reach of the DMRG sweeps)
#   pair::MPS_MAX_ORBITALS    = 9        (MEASURED orbital wall of the MPO build)
#   elements::MAX_Z           = 54       (heaviest registered nuclear charge)
#   RELATIVISTIC_FENCE_Z      = 36       (STAKED here; rows past it carry the model fence)
#
# MPS-ROUTE IS A BAND, NOT AN AVAILABLE ROUTE: pair::MPS_MAX_DETERMINANTS (1024) <= fci::MPS_ROUTE_THRESHOLD
# (50000), so a space large enough to be routed to MPS is necessarily larger than the sweeps' measured
# reach. `AutomaticRoute::Mps` is unreachable at these constants. An MPS-ROUTE row's real production
# status is 'determinant, BY HAND ONLY (solve_determinant)' — and the hard-cap column reads REFUSED
# on the rows where even that by-hand route refuses.
#
# ROUTE CLASSIFICATION IS ARITHMETIC, NOT A CERTIFICATION. FCI-DIRECT says a space is small
# enough for the determinant route; it says nothing about whether that solve converges, agrees
# with a second route, or is fit for any use. Certification is per-species campaign work.
#
# 'scene r' is a SECOND availability axis: Species::homonuclear_radius, measured for ten species
# only. A row with an electronic route and no radius cannot be placed in a scene.
#
  Z  sym   nbas   n_elec    na    nb                  n_det  route        hard-cap  relativity                     scene r
  1  H        1        1     1     0                      1  FCI-DIRECT   -         -                             yes   
  2  He       1        2     1     1                      1  FCI-DIRECT   -         -                             yes   
  3  Li       5        3     2     1                     50  FCI-DIRECT   -         -                             yes   
  4  Be       5        4     2     2                    100  FCI-DIRECT   -         -                             yes   
  5  B        5        5     3     2                    100  FCI-DIRECT   -         -                             yes   
  6  C        5        6     3     3                    100  FCI-DIRECT   -         -                             yes   
  7  N        5        7     4     3                     50  FCI-DIRECT   -         -                             yes   
  8  O        5        8     4     4                     25  FCI-DIRECT   -         -                             yes   
  9  F        5        9     5     4                      5  FCI-DIRECT   -         -                             yes   
 10  Ne       5       10     5     5                      1  FCI-DIRECT   -         -                             yes   
 11  Na       9       11     6     5                  10584  FCI-DIRECT   -         -                             NONE  
 12  Mg       9       12     6     6                   7056  FCI-DIRECT   -         -                             NONE  
 13  Al       9       13     7     6                   3024  FCI-DIRECT   -         -                             NONE  
 14  Si       9       14     7     7                   1296  FCI-DIRECT   -         -                             NONE  
 15  P        9       15     8     7                    324  FCI-DIRECT   -         -                             NONE  
 16  S        9       16     8     8                     81  FCI-DIRECT   -         -                             NONE  
 17  Cl       9       17     9     8                      9  FCI-DIRECT   -         -                             NONE  
 18  Ar       9       18     9     9                      1  FCI-DIRECT   -         -                             NONE  
 19  K       13       19    10     9                 204490  MPS-ROUTE    -         -                             NONE  
 20  Ca      13       20    10    10                  81796  MPS-ROUTE    -         -                             NONE  
 21  Sc      18       21    11    10             1392554592  MPS-ROUTE    REFUSED   -                             NONE  
 22  Ti      18       22    11    11             1012766976  MPS-ROUTE    REFUSED   -                             NONE  
 23  V       18       23    12    11              590780736  MPS-ROUTE    REFUSED   -                             NONE  
 24  Cr      18       24    12    12              344622096  MPS-ROUTE    REFUSED   -                             NONE  
 25  Mn      18       25    13    12              159056352  MPS-ROUTE    REFUSED   -                             NONE  
 26  Fe      18       26    13    13               73410624  MPS-ROUTE    REFUSED   -                             NONE  
 27  Co      18       27    14    13               26218080  MPS-ROUTE    REFUSED   -                             NONE  
 28  Ni      18       28    14    14                9363600  MPS-ROUTE    REFUSED   -                             NONE  
 29  Cu      18       29    15    14                2496960  MPS-ROUTE    REFUSED   -                             NONE  
 30  Zn      18       30    15    15                 665856  MPS-ROUTE    -         -                             NONE  
 31  Ga      18       31    16    15                 124848  MPS-ROUTE    -         -                             NONE  
 32  Ge      18       32    16    16                  23409  FCI-DIRECT   -         -                             NONE  
 33  As      18       33    17    16                   2754  FCI-DIRECT   -         -                             NONE  
 34  Se      18       34    17    17                    324  FCI-DIRECT   -         -                             NONE  
 35  Br      18       35    18    17                     18  FCI-DIRECT   -         -                             NONE  
 36  Kr      18       36    18    18                      1  FCI-DIRECT   -         -                             NONE  
 37  Rb      22       37    19    18               11265100  MPS-ROUTE    REFUSED   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 38  Sr      22       38    19    19                2371600  MPS-ROUTE    REFUSED   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 39  Y       27       39    20    19          1971493202250  MPS-ROUTE    REFUSED   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 40  Zr      27       40    20    20           788597280900  MPS-ROUTE    REFUSED   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 41  Nb      27       41    21    20           262865760300  MPS-ROUTE    REFUSED   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 42  Mo      27       42    21    21            87621920100  MPS-ROUTE    REFUSED   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 43  Tc      27       43    22    21            23896887300  MPS-ROUTE    REFUSED   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 44  Ru      27       44    22    22             6517332900  MPS-ROUTE    REFUSED   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 45  Rh      27       45    23    22             1416811500  MPS-ROUTE    REFUSED   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 46  Pd      27       46    23    23              308002500  MPS-ROUTE    REFUSED   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 47  Ag      27       47    24    23               51333750  MPS-ROUTE    REFUSED   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 48  Cd      27       48    24    24                8555625  MPS-ROUTE    REFUSED   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 49  In      27       49    25    24                1026675  MPS-ROUTE    -         NON-RELATIVISTIC-MODEL-FENCE  NONE  
 50  Sn      27       50    25    25                 123201  MPS-ROUTE    -         NON-RELATIVISTIC-MODEL-FENCE  NONE  
 51  Sb      27       51    26    25                   9477  FCI-DIRECT   -         NON-RELATIVISTIC-MODEL-FENCE  NONE  
 52  Te      27       52    26    26                    729  FCI-DIRECT   -         NON-RELATIVISTIC-MODEL-FENCE  NONE  
 53  I       27       53    27    26                     27  FCI-DIRECT   -         NON-RELATIVISTIC-MODEL-FENCE  NONE  
 54  Xe      27       54    27    27                      1  FCI-DIRECT   -         NON-RELATIVISTIC-MODEL-FENCE  NONE  

# TOTALS over 54 registered species
#   FCI-DIRECT                    27
#   MPS-ROUTE                     27
#   UNAVAILABLE                   0
#   ---
#   past fci::HARD_DETERMINANT_CAP 21
#   NON-RELATIVISTIC-MODEL-FENCE  18   (Z > 36)
#   no measured homonuclear radius 44
# rows 54
```

## Reading the totals

Twenty-seven species take the determinant route and twenty-seven do not, which reads as a
half-and-half table and is not one: the determinant count is not monotone in Z. A minimal
basis makes a closed shell a ONE-determinant problem, so krypton and xenon are the two
cheapest atoms in their rows while the mid-shell transition metals cost hundreds of
millions. The expensive band is the middle of each row, and it is the middle of each row
that chemistry is mostly made of.

The `hard-cap` column is the fence that actually bites. Twenty-one species sit past
`fci::HARD_DETERMINANT_CAP`, where `solve_determinant` — the by-hand route an MPS-ROUTE row
falls back to — refuses outright rather than letting a caller wait on a space that large
without having said so deliberately. For those twenty-one there is today no route at all,
automatic or by hand, without deliberately raising a declared constant.

Zero species read UNAVAILABLE: every registered element carries finite, non-zero primitive
data for every declared shell, enough orbitals to hold its own electrons, and an orbital
count inside `fci::MAX_ORB`. The registry is complete for what it declares. What bounds it
is `elements::MAX_Z` = 54, which is a fence on the REGISTRY and is carried in `FENCES.md`
(row M1) rather than here, because a species that is not registered cannot appear as a row
in a table generated by walking the registry — and a document that silently omits its own
boundary is the failure this programme keeps meeting.

A further constraint this table does NOT show, because it is about charge rather than
species: **anions are unbound in this basis.** OH⁻ sits 0.3055 Ha above neutral OH, and
H⁻/H shows the same sign with a one-determinant CI space, so the cause is the absence of
diffuse functions and not the charged seam (cations pass the identical path). Every row
above is a NEUTRAL atom and is unaffected; but any charged use of these species inherits
`FENCES.md` row M10 and `ION_STAKING.md` I-5.
