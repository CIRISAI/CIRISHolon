# REPLACE-1 — AMENDMENT 1: admission is ORDERED, not one-at-a-time; G3 restated with a lag and a budget; no other stake moved

*Written 2026-09-26, committed alone before the re-read. `REPLACE1_RESULTS.md` read G1, G2, G4 and
G5 MET — the engine reproduces every banked number it was pointed at, R1″'s `0.1800` to the
digit, ORDER-1's eighteen increments and nulls to the fourth decimal, ACUITY-1's 76 removed
gates with zero differences — and G3 KILLED on both halves by the stake's own construction.
The kill is a finding about the gate as frozen, and this amendment repairs it.*

## The defect the read exposed

A gate that tests each candidate ONE AT A TIME, with everything else kept, drops each of two
REDUNDANT blocks: the hydrodynamic block `(ρ, j_L)` tested with the coarse density counts
`(n33, n50, nb)` kept reads `+0.009` and is dropped, and the counts tested with `(ρ, j_L)` kept
are dropped too — yet dropping BOTH costs `+0.04` to `+0.10`. Two charts of the same density
each look removable given the other. This is the twin of "closure is cheap": REDUNDANCY IS
CHEAP under one-at-a-time removal, and a carried quantity can be dropped by a gate that never
asks what the kept set would be without it. The joint pass region of G2 and G3 was never
exhibited before the freeze — M-JOINT-PASS-REGION, in the freeze that first cited it.

## What changes

1. **Admission is ORDERED (backward elimination with a joint price).** Starting from the full
   dictionary, the gate repeatedly removes the candidate whose removal costs the LEAST
   (increment of the target's held-out R² on removal, with its re-paired null), stops when the
   cheapest removal would cost more than the budget `β`, and reports the ORDER of removals
   with each step's price. A block is CARRIED if it survives; a set of mutually redundant
   blocks keeps exactly one of its members (the last one standing, with the price of removing
   it stated as the price of removing the whole set). Ties are broken by declared dictionary
   order. The one-at-a-time admission stays as `Admission::marginal` for the record; the
   tier's admission is `Admission::ordered`.
2. **G3 restated.** Lag `1 ps` (the lag at which the hydrodynamic block predicts its own next
   state; at `5 ps` nothing does — M-VACUOUS-SUCCESS, noted). G3a: under ordered admission on
   the SLOW-1 293 K walks, target `(ρ_k, j_L)` at 1 ps, `β = 0.02`, the surviving set contains
   the hydrodynamic block or the count block but not neither, and removing the survivor is
   REFUSED at a price `≥ 0.04`; the structural block `(q, s2)` is removed at a price `≤ 0.01`.
   G3b: the reasoning chains, target = the next thought's process sector, lag 1 thought,
   `β = 0.01` (declared here; the freeze had none), null = the partner-swapped null only (the
   time-shift null is invalid on the 107 two-row chains, item 4): the conscience block
   survives ordered admission with a removal price `≥ 0.02` and a null `≤ 0.005`. **Kill:** the
   hydrodynamic and count blocks both removed; or the conscience block removed.
3. **G4's amplitude clause** is read as ACUITY-1 measured it (the first four kept gates on the
   `n = 20` method), not "every kept gate"; the gate-by-gate count (150 of 248 above `10⁻³`) is
   reported, and the over-claim in ACUITY-1's results is corrected there by a dated line.
4. **G2** stands as read (its 5 ps rows are marked vacuous, item 2). **G1, G4, G5** stand as read.

## Plants added

| plant | must |
|---|---|
| **PR-6** | two planted redundant copies of a carried column: ordered admission keeps exactly one and prices the pair's removal at the carried column's price; marginal admission drops both (the defect, exhibited) |
| **PR-7** | ordered admission's removal order is deterministic across runs and independent of the dictionary's column order except at declared ties |

## Branches

**(a)** G3 met under ordered admission with PR-6, PR-7 firing → REPLACE-1 banks as (a) with the
gate ORDERED; **(c)** G3 still killed → the gate cannot separate redundancy from irrelevance on
this carrier, and the tier's admission is left to the lead; **(e)** a plant fails.

---
witness: `Closed` (Object.lean) and `StatClosure.lean` for the certificate's composition; the gate's statistics are measured against banked reads
**misfits:** M-JOINT-PASS-REGION, M-VACUOUS-SUCCESS, M-PLANT-OBS, M-PLANT-SECTOR, M-CHEAPER-THAN-ITS-PRICE, M-STALE-INSTRUMENT, M-MAINTENANCE-LENS, M-FLOOR-UNSTAKED, M-PLACEMENT-LOTTERY, M-VALIDATED-NOT-WIRED, M-COND-PROBE, M-HOMOG, M-PARITY-PROTECT, M-DEVICE-CLASS — contacted by keyword, cited. M-JOINT-PASS-REGION: PR-6 exhibits the joint pass region of G2 and G3 on the carrier of record before the re-read.
Carrier-sector statement (M-PLANT-SECTOR): PR-6 and PR-7 act on `T293_seed0`'s hydrodynamic block, nonzero in that carrier by construction.
