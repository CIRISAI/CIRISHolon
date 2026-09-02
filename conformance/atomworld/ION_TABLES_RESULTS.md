# ION TABLES — results

*Every gate of `ION_TABLES_PREREG.md` (frozen 2026-09-01 at `cce77f6`, ADMITTED by
`Audit/prereg_audit.py`, committed ALONE before the generator existed). Instrument:
`engine/crates/holon-chem/src/ion_table.rs`; gates
`engine/crates/holon-chem/tests/ion_tables.rs`; emitter
`engine/crates/holon-chem/examples/emit_ion_tables.rs`; bank
`docs/atoms/tables/ions/`. All four land in the commit that carries this file, so the
readings below and the code that produced them are the same artifact.*

## Verdict

**Eleven gates green, five plants firing, nothing re-staked.** Charged species tabulate
through the same generic door the neutral ones go through, with charge and spin sector in
the key; the neutral path did not move by one bit; H3O⁺ is tabulated and anchored to node
C's certified point; OH⁻ is REFUSED under fence I-5, which stays fired.

**Two findings the gates produced rather than confirmed:**

1. **The H3O⁺ single-bond stretch dissociates to H₂O⁺ + H, not to H₂O + H⁺** — the radical
   channel is **0.159653 Ha BELOW** the naive proton channel in this model. A table that
   enumerated only the obvious channel would publish a well **0.1597 Ha too deep**, and
   nothing in the curve itself would say so. That is now the asymptote rule (the MINIMUM
   over stated channels), and plant P3 measures the damage it prevents.
2. **(H₃O⁺·H₂O), row I-2's headline ionic pair, is priced out and the price is now
   measured**: 15 orbitals, **9,018,009 determinants**, past `MPS_ROUTE_THRESHOLD`
   (50,000) and past the MPS route's 9-orbital reach. It is a COMPUTE-PRICED fence with an
   exit (GANTT node F, or the DMRG cluster seam), not a modelling gap — and the refusal
   carries both numbers rather than saying "no".

## The table

96 knots on `q ∈ [1.0, 12.0]` bohr, uniform in `q^{-1/4}`; 8 basis functions, 3136
determinants, determinant route, exit `Converged` everywhere, device class `cpu`, Davidson
budget 5000, worst residual 9.989e-11.

| | |
|---|---|
| species | H₃O⁺ — charge **+1**, 10 electrons, `(n_α, n_β) = (5,5)`, `sz2 = 0`, class `{1,1,1,8}` |
| cut | node C's staked C3v pyramid, `r(O–H) = 1.85` bohr, `∠(H–O–H) = 113°`, one O–H bond stretched, the other two frozen, nothing relaxed |
| asymptote | **−75.169612229307 Ha**, from channel **H₂O⁺ + H** |
| other channel | H₂O + H⁺ at −75.009959677828 Ha (**+0.159653 Ha higher**) |
| well | `q_e` = 1.911844 bohr, `D_e` = **0.223535 Ha**, `E(q_e)` = −75.393147462129 Ha, `k_e` = 0.556149 Ha/bohr² |
| solver uncertainty | 9.989e-11 Ha (worst Davidson residual) **at budget 5000** |
| grid uncertainty | **1.0040e-7 Ha** over 285 held-out points (the full sweep; the gate's staked subset read 6.9088e-8 over 24) |
| boundary systematic | 1.8474e-13 Ha at `q_max` |

`D_e` is measured against fragments FROZEN at their in-complex geometry, so it OVERSTATES
the relaxed depth. Every number is exact-in-model STO-3G FCI and none is compared to
experiment.

## Gate by gate

| gate | reading | verdict |
|---|---|---|
| **G1** neutral path unmoved | 192 raw-bit comparisons (2 species × 4 columns × 24 knots), H2 (singlet) and OH (doublet), all equal — grid, energy, force, curvature | **PASS** |
| **G2** charge and sector in the key | `IonKey` has private fields and one constructor taking the charge; the artifact carries `charge`, `sz2`, `n_electrons`, `class_z` | **PASS** |
| **G3** unstated charge refuses | `"O H H H"` → `UnstatedCharge`; `"O H H H +1"` served | **PASS**, both legs |
| **G4** anions refused under I-5 | OH⁻ → `AnionFenced{fence:"I-5"}` naming the cause and the exit; OH (asym −74.270732095) and OH⁺ (asym −73.910165796) served on the same door | **PASS**, all three legs |
| **G5** asymptote is the minimum over channels | both channels measured; `E(q_max) − E_asym` = **−1.847e-13 Ha**, inside the staked 1e-3 and nonzero | **PASS**, two-sided |
| **G6** held-out interpolant | 24 staked points (8 intervals × 3 offsets, none on a node): worst `|ΔE|` **6.9088e-8 Ha**, worst `|ΔF|` 1.0148e-5 Ha/bohr, against the 1e-4 kill. Full sweep, 285 points: **1.0040e-7 Ha** | **PASS**, two-sided |
| **G7** boundary systematic | **1.8474e-13 Ha** at `q_max` = 12 bohr, against 1.9e-4 Ha at the knot nearest 6 bohr — a factor **1.13e9**, far inside the ≥4 decay leg | **PASS**, both legs |
| **G8** disclosure | 14 required fields present; `solver_exit`, `uncertainty_hartree` and `solver_budget_iterations` adjacent on the row source; `to_json` REFUSES a table whose interpolant error was never measured | **PASS** |
| **G9** the anchor | proton affinity through the table's own path: **+0.379432332077 Ha**, **8.338e-14 Ha** from `ion_core.rs`'s pin, against a staked 1e-12 | **PASS** |
| **G10** priced-out refusal | (H₃O⁺·H₂O): 15 orbitals, 9,018,009 determinants, refused with both numbers, on a cut whose geometry would have crashed any solve — so the refusal provably preceded the spend | **PASS** |
| **G11** charge conservation | a channel summing to 0 against a declared +1 refuses by integer identity; overlapping and gapped partitions refuse | **PASS** |

## Plants — every one fired, none silent

| plant | carrier and sector | measured effect |
|---|---|---|
| **P1** one ULP on the energy | the energy column, nonzero on every knot | 24/24 knots moved; worst \|ΔE\| 2.220e-16 Ha (H2), 1.421e-14 Ha (OH) — **G1 fires** |
| **P2** charge off by one | the cluster energy, in the charge sector (10 vs 9 electrons) | 24/24 knots moved, worst \|ΔE\| 0.8615 Ha (H2), 0.4530 Ha (OH); at G9 the affinity reads −0.458218 Ha, **0.838 Ha** from the pin — **G1 and G9 fire** |
| **P3** lowest channel dropped | the declared asymptote, in the channel-sum sector | asymptote moves to −75.009960 (H₂O + H⁺); G5's residual becomes **0.1597 Ha = 160× the staked bound**, and the well it would have published is `D_e` 0.383188 against the true 0.223535 — **+0.1597 Ha of manufactured depth**. **G5 fires** |
| **P4** charge left unstated | the parse result, in the refusal sector | `"O H H H"` → `UnstatedCharge` — **G3 fires** |
| **P5** spin sector shifted (`n_α+1, n_β−1`) | the energy, in the `S_z` sector the plant acts on | 24/24 knots moved, worst \|ΔE\| 1.664 Ha (H2), 1.057 Ha (OH) — **G1 fires** |

P5 is deliberately the sector SHIFT and not the α/β swap: the swap is degenerate by spin
symmetry and would have left the energy unmoved, which is an unobservable mutation dressed
as a test. Three of seven plants in a sibling campaign stayed silent for numerical reasons,
so every plant above was checked to move its carrier before its gate's verdict was read.

## Two smaller readings, recorded because they are cheap to lose

* **`E(H⁺) = +0.0 Ha` is COMPUTED here, not a convention.** `ion_core.rs` takes it as one;
  the charged door actually solves it — zero electrons, one determinant, exit `Trivial`,
  no nuclear repulsion — and returns exactly `0x0000000000000000`, at the origin and
  displaced. The convention and the computation agree, which is worth knowing rather than
  assuming.
* **The far tail is fast for a BASIS reason, and that is not a statement about ionic long
  range.** The residual interaction falls from 1.9e-4 Ha at 6 bohr to 1.8e-13 Ha at 12: the
  lower channel is cation-plus-NEUTRAL, so there is no `r^-1` term in it at all, and the
  charge-induced-dipole term that would remain is zero because a hydrogen atom in STO-3G is
  a single `s` function with no polarizability. **G7 passing says nothing whatever about
  the ionic `r^-1` tail**, which is GANTT node B2's, and this table neither serves it nor
  claims it.

## What G5 can and cannot catch, stated before someone else states it

The channel enumeration is CALLER-SUPPLIED, so the obvious attack is: omit the true lowest
channel and the gate cannot know what it was never told. It mostly cannot succeed, and the
reason is worth writing down because it is the reason the two gates are not redundant.

A curve whose declared asymptote is too HIGH approaches the true one from below, so
`E(q_max) − E_asym` goes large and negative and **G5 fires** — that is exactly what plant
P3 measures, at 160× the bound. The escape is a domain too short for the curve to have
reached any asymptote, where the residual could be small for the wrong reason; that is
what **G7's decay leg** is for, since a curve still moving at `q_max` does not show a tail
falling by a factor of four over the last half of its domain.

What survives both: a missing channel that lies ABOVE the declared one. Nothing here
detects that, and nothing needs to — the declared asymptote is still the minimum over the
channels that exist, and an omitted higher channel changes no published number. And a
missing channel below the declared one on a curve whose domain is BOTH too short and
coincidentally flat would pass both. No such case is exhibited; it is recorded as the hole
rather than argued away.

## What this does NOT discharge

* **The ionic three-body surfaces** (I-2's remaining half). They need a rule assigning
  charge to MBE fragments, which is **I-1**'s ambiguity for the census and is not answered
  here. What this lane added is the ENFORCEMENT: `Channel::validate` refuses any fragment
  partition whose charges do not sum to the cluster's, by exact integer identity.
* **Anion chemistry** (**I-5**, unowned). The fence fired in node C and now refuses at a
  second door. Nothing was tuned to get past it.
* **Multiplicity** (**I-4**). The parity rule fixes `S_z`, never `S`. The key names the
  sector solved in and claims nothing about the total spin found.
* **The variational margin** (**I-6**). `PointSolution` still does not carry it, so no gate
  here certifies that the reported state is the LOWEST one in its sector — only that the
  solve converged.
* **Cut shapes beyond the single-bond stretch.** One staked cut, one ion. Angle scans and
  symmetric stretches are a successor's, and the generator is generic over composition and
  charge but not yet over cut shape — stated so nobody reads more genericity into it than
  it has.
* **Nature.** STO-3G full CI is exact-in-model and this document never compares it to
  experiment.
