# Pre-registration — ION TABLES: charge and spin sector belong IN the table key

*Frozen 2026-09-01, committed ALONE, before `ion_table.rs` exists. GANTT node **C**'s
remaining half and `ION_STAKING.md` row **I-2**: the charged analogues of the banked
surfaces, produced through node A's generic Z-tuple door with NO ion-specific branch. The
whole campaign is one sentence — **a table row that does not name its charge and its spin
sector is unusable, and must be impossible to construct** — and every gate below is that
sentence made falsifiable.*

*Node A (4966658) landed the species-generic machinery; node C's core (af9791a) landed
charge at the solver seam with `H3O+` CERTIFIED and `OH−`'s electron-affinity gate FIRED.
This campaign builds nothing new in the solver. It builds the DOOR between them.*

misfits: contacts **M-CACHE-KIND** (the whole campaign: kind belongs in the key — here
the kind is the charge and the sector, and a manifest keyed only by Z lets existence stand
in for certification), **M-BARE-CHARGE** (every energy below is a charged species solved in
a stated `S_z` sector; the fragment channels state their charge assignment rather than
inheriting one), **M-PLANT-OBS** and **M-PLANT-SECTOR** (five plants below, each naming its
carrier and the sector the plant acts on, each pre-checked to fire on THIS instrument),
**M-EXIT-DISCRIMINATOR** (`solver_exit` is emitted and READ, not merely carried — G8 and
the emit refusal), **M-BUDGET-LAUNDER** (a non-converged exit VOIDs the emission loudly and
never degrades to a published table), **M-VACUOUS-SUCCESS** (every gate asserts its WORK
COUNT, not only its failures), **M-UNTESTED-GAP** (the priced-out species G10 names is
measured, not extrapolated from the affordable one), **M-ONE-MODEL-DELTA** (every number is
exact-in-model STO-3G FCI and is never compared to experiment), **M-DEVICE-CLASS** (the
device class is carried per knot and a mixed-class curve is refused), **M-PROVENANCE-OVERREACH**
(the provenance line names the route that ran, never a route inferred alongside it),
**M-STALE-INSTRUMENT** (this freeze, the generator and the emitted artifacts are separate
commits in that order, and the results document names the instrument's commit),
**M-PARITY-PROTECT** (the parity rule that names the sector is a MODEL CHOICE carried in
the key, not a fact the table may hide), **M-MAX-OVER-SUCCESSES** (the feasibility door
takes the refusal's own numbers, never the largest species that happened to work).
Armed by keyword and NOT otherwise contacted: M-HOMOG, M-LOOP-BLIND, M-VOLUME-SCALE,
M-COND-PROBE, M-GAUGE-LAUNDER, M-MAINTENANCE-LENS, M-PLACEMENT-LOTTERY,
M-CHEAPER-THAN-ITS-PRICE, M-IDLE-CALIBRATED-TIMEOUT, M-PROBE-THE-RESOURCE,
M-TAG-AS-PROPERTY, M-NULL-MISSTAKE.

---

## DISCLOSED — seen before this freeze

A feasibility probe (`examples/ion_probe.rs`, untracked, deleted before the generator
lands) was run on the staked C3v pyramid to price the work. These are PRIORS, not results,
and no gate below is staked on a quantity they contain:

* One H3O⁺ knot costs **0.4–0.6 s**: 8 orbitals, 3136 determinants, determinant route,
  Davidson exit `Converged` at residual ~7e-11, 18–38 iterations.
* `E(H3O⁺)` at `r(O–H) = 1.85` bohr, the certified geometry: **−75.392010513557 Ha**.
* The two dissociation channels of the single-bond stretch, fragments frozen at their
  in-complex geometry: **channel A** (H₂O + H⁺, the leaving hydrogen takes no electrons)
  = −75.009959677828 Ha; **channel B** (H₂O⁺ + H, the leaving hydrogen takes one electron)
  = −75.169612229307 Ha. **Channel B is 0.1597 Ha LOWER.** The naive channel is the wrong
  one, and a table that enumerated only channel A would publish a well 0.16 Ha too deep.
* At `q = 10` bohr the computed curve sits 1.4e-9 Ha below channel B — the residual
  interaction is already at solver noise there, which is a statement about THIS BASIS
  (a hydrogen atom in STO-3G is one `s` function and has no polarizability, so the
  charge-induced-dipole tail is absent by construction), not about ionic long range.

## Scope, stated before the gates

**One ion is tabulated: H₃O⁺**, the only charged species whose core certification is green
(`ion_core.rs`, proton affinity +0.379432332077 Ha, gate PASSED). Charge **+1**, 10
electrons, `S_z = 0` by the parity rule, class `{1,1,1,8}`.

**OH⁻ IS NOT TABULATED. It is REFUSED at the table door under fence I-5**, which fired in
node C and stays fired: STO-3G carries no diffuse function, OH⁻ sits +0.3055 Ha ABOVE
neutral OH, and the one-determinant H⁻/H control fires the same way, so the cause is the
DECLARED BASIS. No basis is tuned here to get past it — basis extension is its own node.
The refusal is placed on the TABLE and not on the SOLVER, and that seam is the point: a
solve is a measurement (node C must keep making it, and the fired reading is a measured
model fact worth keeping), while a table is a PUBLICATION, and I-5 forbids publishing an
anion energy from this model as chemistry. **This is stricter than `ION_STAKING.md`'s I-2
row**, which permits an anion surface to be tabulated so long as its depth relative to the
neutral is not published; the lane lead ruled REFUSE, and the divergence is recorded here
rather than left for a reader to discover in the code.

**The cut.** One staked one-dimensional cut through the cluster: O at the origin, two
hydrogens frozen on the C3v cone at `r = 1.85` bohr and `∠(H–O–H) = 113°` (node C's staked
geometry, unchanged and unrelaxed), the third hydrogen moving out along its own cone
direction at distance `q` from the oxygen. **Nothing is relaxed anywhere**, so every energy
is an energy AT A POINT, and the fragments in every channel are frozen at their in-complex
geometry. A frozen fragment sits above its relaxed self, so the depth this table reports
OVERSTATES the relaxed depth; that direction is stated so the number cannot be read the
other way.

**Domain**: `q ∈ [1.0, 12.0]` bohr, **96 knots**, placed uniform in `q^{-1/4}` by
`table::grid_point` — the spacing that equidistributes the cubic Hermite error against a
`1/q` nuclear repulsion (derived in `table.rs`, not chosen here). Outside the domain the
table says nothing and must refuse rather than extrapolate.

**Explicitly NOT in scope, and named so the seam is visible rather than crossed:**

* **The ionic `r^-1` long-range tail belongs to GANTT node B2**, not here. This cut's lower
  channel is cation-plus-NEUTRAL, so no `r^-1` term exists in it at all, and the
  charge-induced-dipole term that would exist is zero in this basis. G7 measures what THIS
  table discards at ITS boundary and claims nothing whatever about ionic long range.
* **The MBE decomposition of a charged cluster.** Splitting a charged cluster into charged
  fragments needs a rule for which fragment carries the excess charge, which is exactly the
  ambiguity `ION_STAKING.md` row I-1 opens for the census. This campaign tabulates the
  EXACT cluster energy and states its channels; it does not assign charge to MBE fragments,
  and a three-body ionic surface is I-2's remaining half.
* **Multiplicity.** The parity rule fixes `S_z`, never `S` (node C's stated caveat, I-4).
  The key names the sector solved in and does not claim the total spin of what was found.

## Gates

Every gate is EXACT-IN-MODEL. A gate that fires is the result and is reported as one.

- **G1 — the neutral path does not move by one bit (EXACT).** The charge-0 instance of the
  ion generator, run on the grid `generate_pair_table` derives for itself, reproduces that
  function's `R`, `E`, `F` and `d²E/dR²` columns with `assert_eq!(a.to_bits(), b.to_bits())`
  on EVERY knot, for H₂ (even electron count, singlet) and OH (odd count, doublet — the
  branch of the parity rule the existing charge-0 regression never covered). Kill: any
  differing bit — the ion door would be a second implementation of the neutral path.
  Carrier: the energy column, which is nonzero on every knot in both cases.
  witness: none (measured gate, mechanized in `tests/ion_tables.rs`)
- **G2 — charge and sector are IN the key (EXACT).** `IonKey` has private fields and no
  constructor that omits the charge, so a keyless row cannot be built; the compiler is the
  check. Mechanized leg: every emitted table file and every manifest row carries `charge`,
  `sz2`, `n_electrons` and the class's sorted Z-tuple, asserted field by field, with the
  asserted field COUNT reported so a passing test cannot be a test that checked nothing.
  Kill: any emitted row missing any of the four.
  witness: none (measured gate; the type-level half is the compiler)
- **G3 — an unstated charge REFUSES, loudly (EXACT).** The spec door parses `"H3O:+1"` and
  refuses `"H3O"` with a named `UnstatedCharge` refusal carrying the offending spec. Two
  legs, because a refusal that passes for the wrong reason proves nothing: the refused form
  must refuse AND the stated form must be served.
  witness: none (measured gate)
- **G4 — OH⁻ is REFUSED under fence I-5, not generated (EXACT).** Every net-negative charge
  refuses at the table door with a refusal naming the fence id `I-5` and the cause (no
  diffuse functions in STO-3G). Sanity legs, so the refusal is about the anion and not
  about a broken door: the same species at charge 0 is served, and a cation at +1 is served.
  Kill: an anion table is produced, or a refusal that does not name the fence.
  witness: none (measured gate)
- **G5 — the asymptote is the MINIMUM over enumerated channels, measured not assumed.**
  Every table declares its dissociation channels with each fragment's charge stated; the
  declared asymptote is the smallest channel sum; and the curve's outer-end value agrees
  with it: `|E(q_max) − E_asym| ≤ 1e-3` Ha. Two-sided: the difference must be NONZERO (an
  exact zero would mean the curve and the channel sum are the same computation, and the
  check tested nothing — VOID, not pass). Kill: outside the bound, which is what an
  incomplete channel enumeration looks like from inside.
  witness: none (measured gate)
- **G6 — held-out interpolant fidelity.** The table's cubic Hermite interpolant against a
  direct charged FCI solve at 24 held-out points: 8 staked intervals (indices 0, 12, 24,
  36, 48, 60, 72, 84) × 3 staked positions `t ∈ {½ − 1/(2√3), ½, ½ + 1/(2√3)}`, none on a
  node. The offsets are DERIVED, not convenient: the Hermite value error peaks near the
  midpoint while the slope error peaks at `½ ± 1/(2√3)`, and a campaign in this tree read
  its own bound 1.5% low by sampling `{¼, ½, ¾}`. Max `|ΔE|` REPORTED; kill if `> 1e-4` Ha.
  Two-sided: an exact zero means the draw hit nodes and the check tested nothing — VOID.
  witness: none (measured gate)
- **G7 — the domain-boundary systematic, and its decay.** `|E(q_max) − E_asym|` REPORTED as
  the interaction this table's domain truncation discards; kill if `> 1e-3` Ha. Decay leg:
  the residual at `q_max` is at most a quarter of the residual at the knot nearest 6 bohr,
  so a NON-decaying tail fires the gate. The bound is deliberately loose — it exists to
  catch a tail that does not fall off, and the disclosed probe says this one falls off fast
  for a BASIS reason that this gate does not certify and does not claim.
  witness: none (measured gate)
- **G8 — the manifest discloses exit, uncertainty and budget together (EXACT).** Every
  emitted row source carries `solver_exit` beside `uncertainty_hartree` beside
  `solver_budget_iterations`, and beside them `interpolant_uncertainty_hartree` as a
  SEPARATE field. The two uncertainties are different quantities and the pair-table schema
  conflates them: the residual describes the solve, the interpolant error describes the
  grid. A capped residual is not monotone in solver effort, so quoting one without its
  budget is quoting a number whose meaning is missing. Kill: any field absent from any row.
  witness: none (measured gate)
- **G9 — the anchor: the certified core point is ON the cut.** At `q = 1.85` bohr the cut IS
  node C's staked C3v pyramid, so the proton affinity recomputed through the TABLE's own
  generic path must reproduce `ion_core.rs`'s pinned `PROTON_AFFINITY_MEASURED` to
  `≤ 1e-12` Ha (not bit-identity: the two paths associate the same geometry's products in a
  different order). Kill: outside the bound — the table's cut is then not the certified
  species. This is the gate that ties a new artifact to an old receipt.
  witness: none (measured gate)
- **G10 — the feasibility door refuses with NUMBERS.** `(H₃O⁺·H₂O)`, I-2's headline ionic
  pair, is priced before it is attempted: the door computes `n_orb` and
  `n_det = C(n_orb, n_α)·C(n_orb, n_β)` from the registry and refuses if the determinant
  count is past `fci::MPS_ROUTE_THRESHOLD` and the orbital count past `MPS_MAX_ORBITALS`,
  carrying both numbers in the refusal. Kill: a door that starts a solve it cannot finish,
  or a refusal that reports no count. The measured count is the receipt that prices the
  successor node rather than guessing at it.
  witness: none (measured gate)
- **G11 — charge conservation across a channel partition is an EXACT INTEGER identity.**
  A channel whose fragment charges do not sum to the cluster's declared total charge is
  refused, by integer comparison and not by tolerance; every slot appears in exactly one
  fragment, also checked. Kill: a channel that violates either and is still scored. This is
  the same identity `ION_STAKING.md` I-1 stakes for the census, enforced here at the door
  it can first be enforced at.
  witness: none (measured gate)

## Plants — each names its carrier and the sector it must be nonzero in

Plants drive the generator through its own code path (`Plant`, a named defect kind the
production entry point passes as `Plant::None`); a plant that bypasses the code it is
planted in tests nothing. Each is PRE-CHECKED to fire on this instrument before it is
believed, because three of seven plants in a sibling campaign stayed silent for numerical
reasons and the mutation, not the gate, was the fault.

- **P1 — one ULP on the energy column.** Carrier: the neutral H₂ energy column, nonzero in
  the sector the plant acts on (every knot's total energy, order −1 Ha). Must fire **G1**.
- **P2 — charge off by one.** The solve runs at `charge + 1` while the key still says what
  the caller stated. Carrier: the cluster energy, nonzero in the charge sector the plant
  acts on (10 vs 9 electrons). Must fire **G1** and **G9**.
- **P3 — a channel dropped.** The channel enumeration loses its lowest member. Carrier: the
  declared asymptote, nonzero in the channel-sum sector the plant acts on — the disclosed
  probe puts the two channels 0.1597 Ha apart, so the residual must exceed G5's bound by at
  least 100×. Must fire **G5**.
- **P4 — the charge left unstated.** The spec `"H3O"` with no charge suffix. Carrier: the
  parse result, nonzero in the refusal sector the plant acts on. Must fire **G3**.
- **P5 — the spin sector shifted.** `(n_α, n_β) → (n_α + 1, n_β − 1)`: a genuinely higher
  `S_z` sector, NOT the α/β swap, which is degenerate by spin symmetry and would leave the
  energy unmoved — an unobservable mutation dressed as a test. Carrier: the OH doublet's
  energy, nonzero in the `S_z` sector the plant acts on. Must fire **G1**.

## What each outcome MEANS, written before any of it is seen

* **All gates green.** The charged table path exists, keyed by composition AND charge AND
  sector, with the neutral path bit-unmoved. That is I-2's machinery half discharged and
  nothing more: one ion, one cut, one basis. It says nothing about ionic long range (B2),
  nothing about anions (I-5), nothing about multiplicity (I-4), and nothing about nature.
* **G1 fires.** The ion door is a second implementation of the neutral path. The campaign
  stops; nothing is emitted; the divergence is the result.
* **G5 or G9 fires.** The cut or the channel enumeration is wrong, and the table is not of
  the species it says. Report the reading, do not re-stake the channel set to fit it.
* **G6 fires.** The grid is too coarse for the declared domain. The measured error becomes
  the successor's stake and the knot count moves — the DOMAIN does not, because shrinking a
  domain to pass an interpolation gate is fitting the ruler to the answer.
* **G7 fires.** The domain truncation discards real interaction. That is a finding for B2
  and is reported as one; this campaign does not add a long-range term to rescue it.
* **G4 fails to fire on an anion.** The fence has been quietly lifted. That is the most
  serious failure available here, because it publishes as chemistry a reading whose own
  gate is fired.
* **Any plant stays silent.** Suspect the mutation first, then the gate. A gate that cannot
  see its own planted defect is not a gate, and neither the gate nor the plant may be
  reported until one of the two is repaired.

## The law this freeze lives under

`epistemology.md` (CIRISOntology) rules 1, 2, 6 and 7: method and the meaning of every
answer written down first; kills staked first and separable; a residual is never support;
and the fired kill is reported as plainly as the survival, with the dead claim kept in the
record and marked dead. Node C's fired electron-affinity gate is that last rule already
working, and this campaign's first duty is not to quietly undo it.
