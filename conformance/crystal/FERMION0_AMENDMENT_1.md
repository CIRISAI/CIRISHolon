# FERMION-0 — Amendment 1: the parts predictor as frozen was a strawman; the coarse history gets a clock; how a random outcome is read

*Frozen 2026-11-11, committed alone, AFTER the preregistration (`7d8734b`) and BEFORE any
circuit of this campaign exists or any sign has been computed. Found by the instrument's
author while specifying the predictors for the build. Stakes G0, S1, S2, S3, S4 and S6 do
not move. S5's parts clause is CORRECTED, and with it the sentence branch (a) may say.*

## 1. The error in the freeze: P-parts was defined to lose

The freeze says of P-parts: "An `e` and its `m` move through the same cells at the same
steps, so no braid of one around the other is resolved, and it predicts `+1`." That looks
only at each `e` and its OWN partner. It ignores the other pair.

When two bound pairs exchange, the first pair's `e` goes half-way around the second pair's
`m`, and the second's `e` half-way around the first's `m`. Those two are in different cells
throughout, so the coarse tier resolves them, and two half-braids make one full braid: `−1`.
A predictor that knows only what the parts are (two bosons and their mutual `−1`) therefore
gets the exchange sign from the coarse history. This is how the literature derives that the
bound pair is a fermion in the first place.

As frozen, S5's clause "P-parts is right on at most 550 of 1,000" would have been met by a
predictor built to ignore half its evidence, and its kill ("the composite is not needed")
could never have fired. That is a stake that passes by construction (M-VACUOUS-SUCCESS, and
M-NULL-MISSTAKE in the shape of the control).

## 2. The correction

- **P-parts (amended).** Knows what the parts are: `e` a boson, `m` a boson, a full braid of
  an `e` around an `m` gives `−1`. From the coarse history it follows each `e` and each `m`
  from cell to cell, counts the braids of every `e` around every `m` that the cells resolve,
  and predicts `−1` raised to that count. On this campaign's processes, where each pair
  moves as one and no cell ever holds two particles, the resolved count is the parity of
  the permutation of the pairs; the reader computes it that way and says so.
- **P-whole-bare** is unchanged. By §1 it now predicts exactly what P-parts predicts. That
  equality is reported as a result, not hidden: **the label "fermion" is not something the
  tier above has to be told. It follows from the parts.**
- **P-whole** is unchanged: P-whole-bare and one bit of memory per bound pair. The bit is
  computed from the fine history by a rule that does not know which processes were built
  with a twist: for each excursion of an `e` that leaves its site and returns without
  crossing a face, the bit flips if the closed path encloses its partner's plaquette.
- **S5 (amended).** For each `c` in `{2, 3}`, on the 1,000 processes:
  - P-whole is right on 1,000 of 1,000;
  - P-parts (amended) is right on at most 550 of 1,000;
  - P-parts and P-whole-bare agree on 1,000 of 1,000.

  *Kill (the bit is not enough):* P-whole misses 1 or more.
  *Kill (nothing beyond the parts' counts and fluxes is needed):* P-parts (amended) is right
  on 950 or more.
- **What branch (a) may now say.** Not "the tier above must carry the whole with a label".
  This: at an acuity that does not resolve the binding, the conserved counts and fluxes of
  the parts give the exchange sign up to ONE BIT per bound pair, and that bit sits on the
  relationship between the two parts, not on either. In this record's vocabulary it is an
  attribute of an edge, owned by the lowest holon holding both ends.

## 3. The coarse history gets a clock, so the pairs of S4 are honestly identical

The freeze let the coarse tier see counts and face crossings "per step". A twist is four
fine steps with no crossing, so a process with a twist is four steps longer than one
without, and a coarse tier with a clock would tell them apart by length alone.

- Every process WITHOUT a twist now carries a NULL EXCURSION of the same four steps: the
  `e` steps to the neighbouring vertex in its cell and back, twice. It encloses nothing and
  its sign is `+1`.
- The coarse history is, for every fine step, the count of `e` and of `m` in every cell.
  S4's pairs `(A, B)` are `A` with the null excursion and `B` with the twist, and their
  coarse histories are compared step by step, cell by cell, before any sign is read.
- S5's four kinds are (one exchange or two) × (null excursion or twist), 250 each.

## 4. Notes on building: how a non-deterministic outcome is read

The engine prints the exact distribution of the whole classical register. A state with many
undetermined stabilizers (P-TRIVIAL has 143 independent ones) would have a distribution of
`2^143` entries. So:

- The referee is asked first. stim gives each ancilla's exact expectation: `+1`, `−1`, or
  `0` for undetermined.
- If every ancilla is determined, the engine runs the full 577-qubit circuit, and its one
  printed outcome must equal stim's, bit for bit.
- If any ancilla is undetermined, the engine runs the same circuit measuring only the sign
  ancilla and at most 3 of the ancillas stim calls undetermined, and must print each of
  those with probability one half. The process is then REFUSED.

The reporting rule is unchanged: a sign is reported only when the sign ancilla and every
stabilizer are determined and the stabilizers equal the claimed configuration.

---
witness: `CIRISHolon.ScenePopulation.edge_law_does_not_fix_wiring` (the bit is on the relationship, not in the counts)
witness: none (the amended S5 is measured)
**misfits:** M-VACUOUS-SUCCESS, M-NULL-MISSTAKE, M-PARITY-PROTECT, M-PLANT-OBS, M-PLANT-SECTOR, M-BASE-RATE-OMITTED. These are the registered ids this text contacts.

- M-VACUOUS-SUCCESS and M-NULL-MISSTAKE: §1 is an instance of each, caught before a number.
- M-PARITY-PROTECT: the permutation count P-parts reads is a mod-2 quantity; S2's plant P-ONE is unchanged.
- M-PLANT-OBS: P-LABEL is re-read against the amended predictors: with the label read as "boson", P-whole must be right on at most 550 of 1,000.
- M-BASE-RATE-OMITTED: P-parts, P-whole-bare and P-whole are printed side by side.

Carrier-sector statement (M-PLANT-SECTOR): the sector the plant acts on is nonzero in its carrier; P-LABEL acts on the 500 processes of odd exchange, fixed by the design of §3.
