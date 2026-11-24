# COSMOS-BASE and COSMOS-CMB-CHANNEL — adversarial review, 2026-11-23: no zero-parameter base or rate law is forced; one real prediction was buried and one inherited law is mislabelled

*An independent reader, given both notes and scripts (`aa1995b`, `8a98cf3`) and told that the
lead expected the base idea to work and to attack both ways. It wrote nothing in either
repository; this is its report as the lead's agent recorded it, shortened. Values marked
(memory) were not checked at source.*

## 1. The rate law the lead meant is in the note, and its crossing is amplitude-free

`d ρ_org/dt = S − 3H(1 + w_dil) ρ_org`, with `S` the cosmic star-formation history
(Madau and Dickinson), gives DESI's sign pattern: below a constant early, above it today.
The crossing sits where the writing rate equals the dilution rate, and for a linear law it
does not depend on the amplitude: `0.811 / 0.813 / 0.818 / 0.826 / 0.837` at shares
`10⁻⁴ / 0.05 / 0.186 / 0.46 / 0.9` (the drift is the background moving). `COSMOS_BASE.md`
reports `0.818` as a table entry and never says it is amplitude-free.

**But it lands high, and it is not parameter-free.**

- The four published DR2 pairs cross at `z = 0.496, 0.354, 0.440, 0.405` (68 % ranges
  `0.29–0.63`, correlation `−0.9` assumed). Like for like, the law's own two-parameter
  projection crosses at `0.652`: high by one to three of DR2's widths.
- Choices, not zero: the star-formation history's form (`0.60` to `0.99` for modest changes;
  a later fit's constants give `0.93` (memory)); how the record dilutes (`w_dil = 0`: `0.82`;
  `−1/3`: `0.66`; `−0.6`: `0.45`; `−1`: none); the writer (existing stellar mass: `0.69`).
  Landing on DR2 needs `w_dil ≈ −0.55` to `−0.7`, which is a fit.
- **The inherited law is mislabelled.** `cosmos_two_portion.py` line 86, copied from the
  sibling's `DE_LEDGER_MODEL.py` line 121, weights the comoving star-formation rate by `a³`.
  That is not "what stars have written, diluting as matter". The plain version (formed
  stellar mass) crosses at `z = 1.95`, reads `+34.5` above a constant at a share of `0.186`,
  and tolerates a share of at most about `0.03`. The sibling record owes this label; the
  two notes here inherit it.

## 2. No zero-parameter rate law gets both the share and the crossing

Enumerated before computing, at the share `Ω_b/Ω_dm`, on the record's fourteen numbers
(a constant reads `10.603`): sources proportional to `H·ρ_m`, to the growth rate, to growth
times amplitude squared: no crossing; `+0.5` to `+71`. A collapsed-fraction rate crosses at
`3.46 / 1.82 / 0.50 / 0.08` for mass thresholds of rising size; at about `10¹⁴` solar masses
it is the best fixed-share reading found (`−2.10`), but the threshold is a free parameter in
disguise and the sibling found this first. With their OWN amplitudes the shares are tiny:
formed stellar mass times `Ω_b/Ω_dm` is `1.7 × 10⁻³`. A share in the DR2 window comes only
from setting the stock by hand.

## 3. "Relabelling" hides a signed prediction, and it is already excluded at that share

Matter-like late and absent at recombination: late `Ω_m = 0.443` at `Ω_b/Ω_dm`, an excess of
`+0.128` over the early value. Published late values (memory) are `0.30` to `0.36`. The
allowed excess caps the share at about `0.03–0.06`. The record's fourteen numbers carry no
early-universe matter-density prior; adding one changes the gains but leaves the windows
wide, so "the data do not fix the share" stands.

## 4. COSMOS-CMB-CHANNEL: correct

The sign and size reproduce (`A_ISW = 0.2006` at a product of `+0.10`): a portion that
follows matter WEAKENS the void cold spot, and the sibling stance's sentence joins two
opposite things. With a published all-sky amplitude of `0.96 ± 0.30` (Krolewski and Ferraro,
arXiv 2110.13959) the bound is `−0.070 < P < +0.080`; a matter-following portion at a share
of `0.2` is `4.0` to `6.4` standard errors out on every value tried. The bias ceiling is
right: a weight `ρⁿ` has large-scale bias `n`; a threshold tracer reaches 100 only at an
abundance of `10⁻³⁸`. One caveat the note lacks (memory): for a real clustering fluid the
perturbation carries a factor `(1 + w)`, so a phantom-phase portion anticorrelates with
matter; the note's quasi-static form is an ansatz.

## 5. By tier

The numbers match the record. In the one common unit, energy fraction, only the molecule has
a reading: `0.108 ± 0.033`, which sits 1.1 standard errors from `Ω_b/Ω_DE = 0.072` and 2.4
from `Ω_b/Ω_dm`. One tier is not a rule.

## Verdict

No zero-free-parameter base or rate law is forced. The nearest thing: share `Ω_b/Ω_dm`,
crossing `0.82`, decided by DR3, but resting on an `a³` convention nobody chose, excluded on
the perturbation channel unless it sits away from the stars that write it, and 28 to 47
times all stellar rest mass. The physically plain rate law gives a share of `10⁻³` to
`10⁻²`: allowed and invisible. `COSMOS_BASE.md` was fair on "nothing forced", too kind to
its reading B, and under-served the lead by listing the rate reading as not done.

Not verified: Planck's own table value; the late-time matter densities and several
constants (memory); the supernova-side matter density uses a crude shape fit.
