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
| generated_utc | 2026-09-02T16:29:29Z | MEASURED |
| binary | `engine/target/release/examples/periodic_availability` | MEASURED |
| binary_sha256 | `8af18600cbc7e4d705be76e1eea2abedee0d8b8757302a5d7eb1e23e6edc72ee` | MEASURED |
| repo_HEAD | `fde999d1cee520bfbfdbc6a5a1f062f7dec4bde3` | MEASURED |
| build_exit | `0` (`cargo build --release -p holon-chem --example periodic_availability`) | MEASURED |
| tree_dirty_rows | 80 (the caps-to-leases change set — `holon-chem/src/budget.rs`, the retired constants, this document — uncommitted at generation and landed in the commit that carries this file; so the binary was built from HEAD PLUS that change set, and the door it exercises is the one that commit ships) | MEASURED |
| run_exit | `0` | MEASURED |
| machine | 31 GiB RAM, 18 GiB `MemAvailable` at generation (`free -g`); the door's verdicts are THIS machine's | MEASURED |

The build's exit status is recorded beside the hash per M-PROVENANCE-OVERREACH
(`conformance/gravity/MISFITS.md`): a true sha256 printed beside a HEAD it was not built
from is more confidently wrong than a timestamp. `build_exit = 0` is what licenses reading
the HEAD line as describing these bytes; the inference from hash to HEAD is an INFERENCE
and is labelled one here.

## What this table is, and the sentence that bounds it

**Route classification is a door's verdict on this machine, not a certification.** A row
reading `FCI-DIRECT` says one thing: the neutral atom's determinant space, counted from
the registry's declared shells, was PRICED (`n_det × 104 vectors × 8 bytes`, the Davidson
working set) and the resource door ADMITTED that price here — so `fci::try_solve` would
take the determinant route on this machine. Another machine gets another column. It does not
say that solve has been run, that it converged, that it agrees with a second route, or
that anything about the species is fit for use. Certification remains per-species campaign
work and this document is not it — ELEMENTS-3 priced the difference when indium came back
at a 3.98e-1 residual, nine orders above `pair::CONVERGED_RESIDUAL`, in a row otherwise
indistinguishable from a measurement.

Everything here is counting: `n_basis` sums the declared shells' `n_functions()`, the
electron split is `pair::electron_counts` (the solver's own function), and
`n_det = C(n_orb, n_alpha) * C(n_orb, n_beta)` is the same arithmetic
`pair::route_for` spends at its door. No solve runs. There are no route thresholds to
print: the caps that used to decide this column (`MPS_ROUTE_THRESHOLD`,
`HARD_DETERMINANT_CAP`, `MPS_MAX_DETERMINANTS`, `MPS_MAX_ORBITALS`) were retired on
2026-09-02 for prices put to a probe (`holon_chem::budget`), and the `det price` column
prints what each row asked the door for, so the verdict can be re-derived on any machine
from its own headroom.

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
# The route column is decided by THIS MACHINE's resource door, not by a constant:
#   FCI-DIRECT       the determinant working set (n_det x 104 vectors x 8 bytes) was ADMITTED
#   MPS-ROUTE        the determinant set was refused and the MPS route's PROVISIONAL MPO price admitted
#   REFUSED-BY-DOOR  both refused here; the 'det price' column says what was asked for
#   fci::MAX_ORB              = 64        (the string machinery's orbital ceiling)
#   elements::MAX_Z           = 54       (heaviest registered nuclear charge)
#   RELATIVISTIC_FENCE_Z      = 36       (STAKED here; rows past it carry the model fence)
#   MPS reach provenance: MPS reach, superseded record: chi=32, 300 s wall-clock on a loaded box; reached 225 determinants (LiH), not 23,409 (S2); MPO driven at 9 orbitals, not 10. Provenance: pair-route sweeps 2026-08-3x; re-measured in work units by the MPS seam node.
#
# ROUTE CLASSIFICATION IS ARITHMETIC, NOT A CERTIFICATION. FCI-DIRECT says a space is small
# enough for the determinant route; it says nothing about whether that solve converges, agrees
# with a second route, or is fit for any use. Certification is per-species campaign work.
#
# 'scene r' is a SECOND availability axis: Species::homonuclear_radius, measured for ten species
# only. A row with an electronic route and no radius cannot be placed in a scene.
#
  Z  sym   nbas   n_elec    na    nb                  n_det  route        det price relativity                     scene r
  1  H        1        1     1     0                      1  FCI-DIRECT   8.00e1B   -                             yes   
  2  He       1        2     1     1                      1  FCI-DIRECT   8.00e1B   -                             yes   
  3  Li       5        3     2     1                     50  FCI-DIRECT   4.16e4B   -                             yes   
  4  Be       5        4     2     2                    100  FCI-DIRECT   8.32e4B   -                             yes   
  5  B        5        5     3     2                    100  FCI-DIRECT   8.32e4B   -                             yes   
  6  C        5        6     3     3                    100  FCI-DIRECT   8.32e4B   -                             yes   
  7  N        5        7     4     3                     50  FCI-DIRECT   4.16e4B   -                             yes   
  8  O        5        8     4     4                     25  FCI-DIRECT   1.16e4B   -                             yes   
  9  F        5        9     5     4                      5  FCI-DIRECT   7.20e2B   -                             yes   
 10  Ne       5       10     5     5                      1  FCI-DIRECT   8.00e1B   -                             yes   
 11  Na       9       11     6     5                  10584  FCI-DIRECT   8.81e6B   -                             NONE  
 12  Mg       9       12     6     6                   7056  FCI-DIRECT   5.87e6B   -                             NONE  
 13  Al       9       13     7     6                   3024  FCI-DIRECT   2.52e6B   -                             NONE  
 14  Si       9       14     7     7                   1296  FCI-DIRECT   1.08e6B   -                             NONE  
 15  P        9       15     8     7                    324  FCI-DIRECT   2.70e5B   -                             NONE  
 16  S        9       16     8     8                     81  FCI-DIRECT   6.74e4B   -                             NONE  
 17  Cl       9       17     9     8                      9  FCI-DIRECT   1.87e3B   -                             NONE  
 18  Ar       9       18     9     9                      1  FCI-DIRECT   8.00e1B   -                             NONE  
 19  K       13       19    10     9                 204490  FCI-DIRECT   1.70e8B   -                             NONE  
 20  Ca      13       20    10    10                  81796  FCI-DIRECT   6.81e7B   -                             NONE  
 21  Sc      18       21    11    10             1392554592  MPS-ROUTE    1.16e12B  -                             NONE  
 22  Ti      18       22    11    11             1012766976  MPS-ROUTE    8.43e11B  -                             NONE  
 23  V       18       23    12    11              590780736  MPS-ROUTE    4.92e11B  -                             NONE  
 24  Cr      18       24    12    12              344622096  MPS-ROUTE    2.87e11B  -                             NONE  
 25  Mn      18       25    13    12              159056352  MPS-ROUTE    1.32e11B  -                             NONE  
 26  Fe      18       26    13    13               73410624  MPS-ROUTE    6.11e10B  -                             NONE  
 27  Co      18       27    14    13               26218080  MPS-ROUTE    2.18e10B  -                             NONE  
 28  Ni      18       28    14    14                9363600  FCI-DIRECT   7.79e9B   -                             NONE  
 29  Cu      18       29    15    14                2496960  FCI-DIRECT   2.08e9B   -                             NONE  
 30  Zn      18       30    15    15                 665856  FCI-DIRECT   5.54e8B   -                             NONE  
 31  Ga      18       31    16    15                 124848  FCI-DIRECT   1.04e8B   -                             NONE  
 32  Ge      18       32    16    16                  23409  FCI-DIRECT   1.95e7B   -                             NONE  
 33  As      18       33    17    16                   2754  FCI-DIRECT   2.29e6B   -                             NONE  
 34  Se      18       34    17    17                    324  FCI-DIRECT   2.70e5B   -                             NONE  
 35  Br      18       35    18    17                     18  FCI-DIRECT   6.34e3B   -                             NONE  
 36  Kr      18       36    18    18                      1  FCI-DIRECT   8.00e1B   -                             NONE  
 37  Rb      22       37    19    18               11265100  FCI-DIRECT   9.37e9B   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 38  Sr      22       38    19    19                2371600  FCI-DIRECT   1.97e9B   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 39  Y       27       39    20    19          1971493202250  MPS-ROUTE    1.64e15B  NON-RELATIVISTIC-MODEL-FENCE  NONE  
 40  Zr      27       40    20    20           788597280900  MPS-ROUTE    6.56e14B  NON-RELATIVISTIC-MODEL-FENCE  NONE  
 41  Nb      27       41    21    20           262865760300  MPS-ROUTE    2.19e14B  NON-RELATIVISTIC-MODEL-FENCE  NONE  
 42  Mo      27       42    21    21            87621920100  MPS-ROUTE    7.29e13B  NON-RELATIVISTIC-MODEL-FENCE  NONE  
 43  Tc      27       43    22    21            23896887300  MPS-ROUTE    1.99e13B  NON-RELATIVISTIC-MODEL-FENCE  NONE  
 44  Ru      27       44    22    22             6517332900  MPS-ROUTE    5.42e12B  NON-RELATIVISTIC-MODEL-FENCE  NONE  
 45  Rh      27       45    23    22             1416811500  MPS-ROUTE    1.18e12B  NON-RELATIVISTIC-MODEL-FENCE  NONE  
 46  Pd      27       46    23    23              308002500  MPS-ROUTE    2.56e11B  NON-RELATIVISTIC-MODEL-FENCE  NONE  
 47  Ag      27       47    24    23               51333750  MPS-ROUTE    4.27e10B  NON-RELATIVISTIC-MODEL-FENCE  NONE  
 48  Cd      27       48    24    24                8555625  FCI-DIRECT   7.12e9B   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 49  In      27       49    25    24                1026675  FCI-DIRECT   8.54e8B   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 50  Sn      27       50    25    25                 123201  FCI-DIRECT   1.03e8B   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 51  Sb      27       51    26    25                   9477  FCI-DIRECT   7.88e6B   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 52  Te      27       52    26    26                    729  FCI-DIRECT   6.07e5B   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 53  I       27       53    27    26                     27  FCI-DIRECT   1.34e4B   NON-RELATIVISTIC-MODEL-FENCE  NONE  
 54  Xe      27       54    27    27                      1  FCI-DIRECT   8.00e1B   NON-RELATIVISTIC-MODEL-FENCE  NONE  

# TOTALS over 54 registered species
#   FCI-DIRECT                    38
#   MPS-ROUTE                     16
#   UNAVAILABLE                   0
#   ---
#   REFUSED-BY-DOOR (this machine) 0
#   NON-RELATIVISTIC-MODEL-FENCE  18   (Z > 36)
#   no measured homonuclear radius 44
# rows 54
```

## Reading the totals

Thirty-eight species are admitted to the determinant route on this machine and sixteen are
not, and the split is not monotone in Z. A minimal
basis makes a closed shell a ONE-determinant problem, so krypton and xenon are the two
cheapest atoms in their rows while the mid-shell transition metals cost hundreds of
millions. The expensive band is the middle of each row, and it is the middle of each row
that chemistry is mostly made of.

The `det price` column is what actually decides. The sixteen MPS-ROUTE rows are the
species whose Davidson working set exceeded this machine's 18 GiB of headroom — iron
asks 6.1e10 bytes — and whose MPO price (PROVISIONAL: `n_orb × (2 + 3.8 n_orb²)² × 32`
bytes) the door then admitted. Zero rows read REFUSED-BY-DOOR here; a smaller machine
regenerates this table and gets some. Two things that row does NOT say, stated so the
column cannot be over-read: an admitted MPS price is a working set that fits, not a solve
that converges — the MPS reach record printed in the header (225 determinants reached, 9
orbitals driven, measured in wall-clock on a loaded box) is the method's measured face and
is far short of iron; and the price itself is provisional, re-measured in work units by
the MPS seam node. Before 2026-09-02 this paragraph counted twenty-one species past a
declared constant with "no route at all"; that constant was a price wearing constant
costume, and the honest form is the one above — priced, probed, and re-derivable.

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
