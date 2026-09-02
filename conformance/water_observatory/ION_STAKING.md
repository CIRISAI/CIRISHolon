# The ion register — what charge at the solver seam does NOT yet buy, and who owns each piece

*Opened by the `ion-core` lane on landing GANTT node **C** (charged-fragments core). Read
it the way `DRY_RESIDUALS.md` is read: a row is a MEASUREMENT of something owed, not an
apology for it. Node C's own receipt is `engine/crates/holon-chem/tests/ion_core.rs`; every
row below is work that receipt deliberately does not discharge, with the gate that would.*

**No timelines anywhere in this file.** Size is compute × scope, and the only ordering is
dependency — GANTT.md's law, applied here.

---

## What node C landed, so the rows below are read against something

`holon-chem/src/ions.rs` — `solve_geometry_charged(species, centers, charge)`. Total
electrons are `sum(Z) − charge`; the `S_z` sector is the parity rule (even → singlet, odd
→ doublet), stated in the doc comment as a MODEL CHOICE with its caveat; everything
unstateable comes back as a named `ChargeRefusal` (`NegativeElectrons`, `ChargeTooLarge`,
`UnstatedSpinSector`) and never as a number. `charge == 0` is BIT-IDENTICAL to
`solve_geometry`, asserted on raw f64 bits, so the charged path is not a second copy of the
neutral one.

**No table, no dynamics, no census bookkeeping, no ion registry.** A charged fragment here
is a species list, a geometry and an integer. That is the whole node.

## The two measured readings that constrain every row below

Both at STAKED geometries — declared in advance, never relaxed — in STO-3G full CI, exact
in model on the determinant route.

| reading | value | verdict |
|---|---|---|
| `E(H2O) − E(H3O+)`, proton affinity, `E(H+) = 0` by convention | **+0.379432332077 Ha** | gate **PASSED** |
| `E(OH) − E(OH−)` at `r = 1.83` bohr, vertical | **−0.305545907904 Ha** | gate **FIRED** |

**The electron-affinity gate fired and that is the headline of node C.** OH− sits 0.3055
hartree ABOVE neutral OH in this model. It is kept in the record, marked, and pinned
two-sided rather than reversed into a green assertion. The discriminator is in the test:
the same measurement on H−/H — where the anion's CI space is ONE determinant, so no sector
rule and no electron count can be responsible — fires the same way at −0.308024094363 Ha,
while the CATION gate passes on the identical code path. The cause is the DECLARED BASIS
(STO-3G has no diffuse function for the extra electron), not the charged seam.

**What that licenses and what it forbids.** It licenses cation chemistry through this seam.
It forbids reading any ANION energy from this model as an affinity, an electron-detachment
energy, or a stability statement about the anion — including, and especially, inside any
table or scene built on the rows below. A row that needs an anion to be bound is blocked on
**I-5**, not on the machinery it looks like it is blocked on.

---

## Open rows

| id | what is owed | owner | gated on | the receipt-gate (what "done" means) |
|---|---|---|---|---|
| **I-1** | **Census charge bookkeeping.** The census names molecules by CONNECTED-COMPONENT NAMING (`CENSUS_PREREG.md`, opening section) and has no charge at all: a component is a set of atoms, so H₃O⁺ and H₂O+H are the same reading. Charge assignment needs a stated rule — which component carries the excess proton, and what the census reports when the rule is ambiguous. | census-lens (the census's own lane); ion-core supplies the seam only | node C (done). NOT gated on A or B2. | A charge-assignment rule written down BEFORE it is run, with its ambiguous case named; the census's per-frame charge column summing to the scene's declared total charge on every frame of a reference trajectory (an exact integer identity, not a tolerance); and a REFUSAL — not a guess — on any frame where the rule cannot assign. The C-4 naming-artifact control of `CENSUS_PREREG.md` re-run with charge present, still rejecting. |
| **I-2** | **Ion pair and triple TABLES.** **PARTLY DISCHARGED 2026-09-01 by the ion-tables lane** (freeze `conformance/atomworld/ION_TABLES_PREREG.md`, ADMITTED and committed alone before the generator existed; receipts `conformance/atomworld/ION_TABLES_RESULTS.md`, code `holon-chem/src/ion_table.rs`, gates `holon-chem/tests/ion_tables.rs`, bank `docs/atoms/tables/ions/`). **What landed:** the charged table path, keyed by composition AND charge AND spin sector — `IonKey` has private fields and one constructor that takes the charge, so a row that does not name its charge and sector cannot be constructed; the neutral curve is the `charge == 0` instance of the same generator and reproduces `generate_pair_table` on raw f64 bits (192 comparisons, H2 and OH, six plants all firing); H3O+ tabulated on a staked cut through node C's certified pyramid, with the proton affinity recovered to 8.3e-14 Ha of the pin. **The finding:** the single-bond stretch dissociates to **H2O+ + H**, 0.1597 Ha BELOW the naive H2O + H+ channel, so the asymptote is the MINIMUM over stated channels and a lane enumerating only the obvious one would have published a well 0.1597 Ha too deep — planted and caught. **What remains:** the ionic PAIR and TRIPLE surfaces this row opened with. (H3O+ . H2O) is priced out and the number is now measured rather than guessed — 15 orbitals, 9,018,009 determinants, past `MPS_ROUTE_THRESHOLD` (50,000) and past the MPS route's 9-orbital reach — so it is a COMPUTE-PRICED fence whose exit is GANTT node F or the DMRG cluster seam, not a modelling gap. The ionic three-body surface additionally needs a rule assigning charge to MBE fragments, which is **I-1**'s ambiguity and is not answered here; what this lane added is the enforcement, `Channel::validate`, refusing any fragment partition whose charges do not sum to the cluster's by exact integer identity. Also owed: cut shapes beyond the single-bond stretch (angle scans, symmetric stretch), and a second tabulated cation once one has a green core. | mbe-generic (node A owns the machinery); ion-tables instantiated it | **GANTT node A** (landed 4966658) | The charged surfaces produced through A's generic Z-tuple door with NO ion-specific branch — DONE, charge is data and nothing branches on it. Held-out interpolation error and a domain-boundary systematic staked before measuring — DONE (G6 and G7 of the freeze). Provenance carrying the charge and the solver route on every table — DONE, with the residual and its BUDGET and the interpolant error carried as separate fields, because a capped residual is not monotone in solver effort. **The I-5 constraint is honoured MORE STRICTLY than this row originally wrote it:** the row permitted an anion surface to be tabulated so long as its depth relative to the neutral went unpublished; the lane lead ruled REFUSE, and `ion_table.rs` refuses every net-negative charge at the table door with the fence and its cause named. The seam is deliberate — a solve is a MEASUREMENT and `ions.rs` must keep making it, a table is a PUBLICATION and I-5 forbids that one. |
| **I-3** | **The Grotthuss chain** — autoionisation and H₃O⁺/OH⁻ proton-hop wires, `WORKBENCH_FSD.md` **WB-8.5**'s exotic showcase entry. | not yet owned; a dynamics lane, downstream of the two above | **I-1 AND I-2 AND GANTT node B2.** B2 is the hard one and it is now near-certain rather than conditional: B1 is measuring what cutoff-locality discards for NEUTRAL scenes, and an ionic scene's discarded term falls off as 1/r rather than as a neutral multipole. GANTT already records C as making B2 near-certain for ionic scenes; this row is where that comes due. | B1's verdict re-run on an IONIC scene class (the neutral verdict does not transfer and must not be cited as if it did); the energy ledger closed under B2's new term, with a PLANTED violation caught (`one-gate-per-conservation-law`); a hop exhibited as a census-visible charge REASSIGNMENT between named components across consecutive frames, not as a picture; and the whole chain refusing rather than serving wherever a fence is undischarged. |
| **I-4** | **The spin sector as a MEASUREMENT rather than a model choice.** The parity rule solves in minimal \|S_z\|, which cannot MISS the ground state but does not certify which total spin it found. | ion-core, or whoever first needs an open-shell ion | node C (done). Independent of A, B2 and I-1. | A variational sweep: solve `S_z = 0, 1, 2, …` (or `1/2, 3/2, …`) at one geometry, take the lowest, and REPORT the winning sector on the solution rather than assuming it. Done means the sweep agrees with the parity rule on every closed-shell case certified in `ion_core.rs` (so it is a superset, not a replacement), and disagrees somewhere — a case where the swept sector beats the parity one — or the sweep is recorded as having found no such case over a stated set, which is itself the result. `fci::s_squared` reports the multiplicity that goes with it. |
| **I-5** | **A basis in which anions are bound.** The fired gate's only honest discharge. | not this node; a basis lane. `holon-chem` declares exactly one basis (`sto3g.rs`) and adding a second is a crate-shaped decision, not a test fix. | nothing technical — it is unowned, not blocked | The SAME two gates in `ion_core.rs`, unchanged and un-retuned, re-run against a basis carrying diffuse functions: `E(OH−) < E(OH)` and `E(H−) < E(H)`. The STO-3G readings stay in the record beside the new ones, marked as the fired pair, because the point of the pin is that the two bases can be compared. Until then, `OH_ELECTRON_AFFINITY_MEASURED` is a property of the declared basis and is cited as one. |
| **I-6** | **The variational margin is not visible through the charged door.** `fci::Solution` carries `variational_margin` — `min_i H_ii − E`, the one cheap check that catches a solve converged cleanly onto the WRONG eigenvector, which no residual can — and `PointSolution` does not re-export it. So `ion_core.rs` certifies convergence (SCF, exit reason, residual) but NOT that the reported state is the lowest one, on either side of either gate. | whoever next moves `PointSolution`; NOT taken here | nothing technical. Deliberately not done in this commit: `PointSolution` is a shared struct with callers in several lanes' live files, and widening it inside a node-C commit is the cross-lane sweep the house rules forbid. | `PointSolution` carrying the margin from the routes that compute it and `None` from the ones that do not (the MPS path), so a caller must decide what a missing check means rather than read a default as a pass — the shape `Solution` already uses. `ion_core.rs` then asserts it non-negative on all six solves, and a PLANTED wrong warm start is caught by it. |

## Fence I-5 now has a SECOND enforcement site

`FENCES.md` row M10 names `ion_core.rs:288` as the place I-5 is carried. As of 2026-09-01
it is carried in two places: that gate, which MEASURES the fired reading, and
`ion_table.rs`'s `AnionFenced` refusal, which REFUSES to publish one. The ledger is
`bank-fences`' file and this lane has not edited it; the second site is recorded here so
the fence's owner can pick it up. The two sites are not redundant — one is an instrument
and the other is a door, and a fence that lived only on the instrument would leave the
door open.

## Rows that are NOT owed, recorded so they are not re-opened

* **An ion species registry.** There is no `H3O_PLUS` constant and there must not be. A
  charged fragment is a species list, a geometry and an integer; a registry of "the ions we
  support" would be the table-lookup shape the crate header already refuses for which pairs
  bind.
* **A charge-aware `solve_geometry`.** The neutral entry point is unchanged and the
  bit-identity gate is what keeps it that way. Charge enters through its own door.
* **Anything in `sim.rs`.** Node C touched no dynamics. Four lanes are in that file
  (`DRY_RESIDUALS.md` WO-R-4, WO-R-5), and the migration is I-3's problem when it has an
  owner.

## How to cite a row from this register

`ION-I-3`. There are already two registers in this tree numbering from R-1 (`WO-R-*` and
`GRAV-R-*`, see `DRY_RESIDUALS.md` WO-R-15); this one starts prefixed so it never joins
that problem.
