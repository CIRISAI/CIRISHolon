# SEAM-1 — AMENDMENT 1: the monotonicity clause fired on an unstaked floor; the floor is measured and the clause re-staked

*Frozen 2026-09-04, committed alone, AFTER SEAM-1's eight nodes were read and BEFORE
the floor measurement it stakes was run. It changes no number already read. The
analysis that forced it, the registered misfit it cites, and the successor gate it
stakes are below; the results document carries S1 as fired and S1′ as read.*

misfits: contacts **M-FLOOR-UNSTAKED** (registered from this campaign the same day:
the monotonicity clause read a quantity, the embedded residual, that the freeze
gave no resolution floor, and fired on values within an order of magnitude of the
arithmetic floor of a seven-solve difference); **M-NULL-MISSTAKE** (the successor
clause is staked on the quantity it constrains, with the floor of THAT quantity);
**M-STALE-INSTRUMENT** (this amendment alone, then the floor instrument's run, then
the results document with both); **M-EXIT-DISCRIMINATOR** (every outcome of the
floor measurement is named below and none is the default); **M-VACUOUS-SUCCESS**
(node counts stated; a clause with fewer posable pairs than staked is VOID, not
passed); **M-CHEAPER-THAN-ITS-PRICE** (the floor run is three trimer solves at G0's
measured 265 wall-seconds each on the same machine; a run arriving under half that is
VOID); **M-DEVICE-CLASS** (host `f64`, `solve_determinant`, one class);
**M-MAX-OVER-SUCCESSES** (S1′ is a for-all over posable pairs); **M-HOMOG** and
**M-BARE-CHARGE** (the words "local" and "charge" appear; nothing homogeneous or
gauge-charged is meant); **M-PLANT-SECTOR** and **M-PLANT-OBS** (the word "sector"
appears for the far sector; NO deliberate defect is switched on in this amendment —
the null is a calibration of the instrument, not a verdict, and a switched-on defect
inside a floor measurement would fabricate the floor it measures — so the carrier
statement those two require, a carrier asserted nonzero in the sector the defect acts
on, has no referent here and is discharged by absence, stated); **M-COND-PROBE** (the
phrase "inside the field" contacts its keyword; the field is the Hamiltonian's own
external-charge term, applied before the solve, never an operator applied after a
step). Not contacted: the rest of the registry.

## A1.0 What was read, and stays read

SEAM-1's S1, by its frozen letter, reads **BRANCH (b)**: the fraction clause
`κ ≤ 0.25` holds at every posable far node by a factor of 300 or more
(`κ = 5.07e-4, 7.81e-4, 6.78e-4` at 5, 6, 8 Å), and the monotonicity clause fires
at the 5 → 6 Å pair (`κ` rises 5.07e-4 → 7.81e-4). That reading is kept, marked
fired, and is not rewritten by this amendment (rule 7 of the seed's discipline).

## A1.1 The analysis that forced this amendment

The residuals behind the fired pair are `r_emb = +2.27e-9` and `+1.15e-9` hartree.
Each is `E_ABC − E_EE-PA`, a difference of seven exact solves of energies near
−100 to −300 hartree, each carrying roundoff of order `1e-13` relative; the
difference's arithmetic floor is therefore of order `1e-10`. The freeze staked a
resolution floor of `1e-7` for the DENOMINATOR (the bare three-body term) and none
for the NUMERATOR, so the clause compared a physical quantity to one whose bottom
decade may be arithmetic. The 6 → 8 Å pair, where the residual reads `1.75e-10`,
is unreadable for the same reason — and its exponent (`6.55`, against the bare
term's `6.06`) would have read the OPPOSITE way. Whether the 5 → 6 Å rise is physics
(induction the point charges mis-carry — the freeze's own branch-(b) reading) or
arithmetic is exactly what the frozen clause could not tell, and it is what this
amendment measures instead of arguing.

## A1.2 The floor instrument (`holon-chem/examples/seam_floor.rs`)

At each far node (5, 6, 8 Å): the chain is moved by the fixed off-axis vector
`(0.37, 0.21, 0.5)` bohr, every one of the seven solves is repeated on the moved
geometry with the self-consistent charges re-derived there, and

```
floor(R) = | r_emb(R) − r_emb(R, moved) |
```

is the node's measured floor — the physics is translation-invariant, the
arithmetic is not, and the difference is the instrument's own resolution of the
quantity the clause reads. The floors of `E_ABC` and of the bare three-body term
are recorded beside it. 3 nodes, one run, resumable per node.

## A1.3 The successor clause, S1′

- **posable, per node:** `|r_emb(R)| ≥ 10 · floor(R)`. A node below that is VOID for
  the monotonicity clause, named, and still counts for the fraction clause (whose
  stake, `0.25 · |ΔE_3^bare|`, is four orders above any floor here).
- **S1′(a):** the fraction clause holds at every far node (already read: it does)
  AND `κ` is non-increasing across every consecutive pair of POSABLE far nodes.
- **S1′(b):** a posable pair with `κ` rising ⇒ the rise is physics at this
  resolution: the embedded residual decays more slowly than the bare three-body
  term, the freeze's branch-(b) reading stands on measured ground, and the next
  freeze stakes polarisable embedding.
- **S1′(void-monotone):** fewer than ONE posable pair ⇒ the monotonicity clause is
  unreadable on this carrier at this resolution; the campaign's reading is then the
  fraction clause ALONE — "the embedding carries the three-body term to the
  instrument's resolution across the far sector" — which is a weaker statement than
  (a) and is written as such, never as (a).
  witness: none (a measured floor and a measured ratio; no Lean object)

## A1.4 What each outcome licenses

S1′(a) licenses what SEAM-1's (a) would have: the seam's premise on this carrier,
and the water cores go inside the field under the next freeze. S1′(b) licenses the
same next step with a named missing term (polarisation) staked beside it. The
void-monotone reading licenses the water freeze on the fraction alone, with the
monotonicity question carried forward to a carrier whose three-body term is larger
than this one's — a shorter chain spacing or a more polar monomer — named there.

## A1.5 Discipline

Runner `seam_floor.rs`; JSON `seam/floor_R*.json`; the results document
`SEAM_RESULTS.md` carries S1 (fired, kept), the floors, and S1′ as read, and is
committed with both runners. Both clocks reported. No number enters from outside
the engine.
