# HOUSEHOLD-0 — adversarial review of the draft, 2026-11-11: freeze only after named fixes

*A second reader attacked `HOUSEHOLD0_PREREG_DRAFT.md`. No data was fetched, no data service
called and no documentation page opened; survey facts are from `ACS_DOCS.md` or the
reviewer's memory, and the ones from memory are marked. One single-threaded synthetic script
(`household0/review_synthetic.py`, kept as run). The author's condensation; the draft is
kept as written.*

**Verdict.** The right campaign. Three faults would have been frozen.

1. **The sampling-error floor was inverted.** "A distance smaller than twice its own error
   is unresolved" makes the best generator there can be (distance near zero) unresolved, and
   lets a verdict through only when the miss is large. It contradicted the draft's own plant
   P-EQUAL, which must read `certified`. The floor a certificate needs compares the
   reference's error with the BUDGET. At these pool sizes it would not bind anyway (pooled
   error of order `0.002` in a share), and the split-half scatter is the better error.
2. **The budget measured the wrong thing.** Real-against-real scatter of a pooled class
   mean is a few thousandths of a share, far tighter than anything a consumer reads.
   - The twin's rule, simulated: one-person share `0.181`, against a national share near
     `0.29` (memory). S1's "at least 3 β₀" was slack by an order of magnitude; it could not
     fail.
   - S2 held out by STATE and set its budget from random COUNTY splits. On a toy model
     (county spread `0.04`, state spread `0.015` in the one-person share, both guesses) a
     held-out half of other states exceeds the 95 % random-split gap in 35 % of draws at
     1,500 counties a side, 6 % at 300, 1 % at 60. A coin flip on half-point differences.
   - **Fix:** budgets from the consumer, stated before any file is read. The twin fixes
     households at people over 2.5 with one always-on router each, so the MEAN moves a
     street's node count linearly: 5 % of nodes is `0.12` persons. The one-person share
     matters to who is home; `0.03` there. Two consumers, two verdicts. The forecast under
     them: the twin's rule is certified on the mean and refused on composition, which one
     pooled W1 hides.
3. **The open top band's derived mean is unstable.** It divides a small difference of large
   numbers by about 1.5 % of households. A 1 % disagreement between household population
   and household counts turns a true top mean of `7.8` into `9.45` or `6.15`. The reviewer
   believes the two are weighted differently in the survey (memory; NOT checked). P-TOP
   recovered `9.0` only because its carrier was self-consistent. **Fix:** place the band at
   one declared value on both sides of every distance, and print the residual as a reading
   with its sensitivity.

**Also.**
- **The unit.** A tract's class matches a 0.9 km tile's only near 4,000 per km². The table
  describes households in tracts whose AVERAGE density is X. **Fix:** produce every table
  under two keys (tract, block group) and stake whether the key transfers.
- **The envelope hole.** "Tract density class" is not a coordinate a twin leaf has, and
  the envelope is tested first (`verdict_certified_iff`). For the twin's own use every
  verdict here is outside the envelope until the key is shown to transfer. What is issued
  is a verdict on the rule as a description of US households pooled by the reference's unit.
- **Resolved.** "30 counties with one tract" counts a county with one such tract the same
  as Los Angeles. **Fix:** effective count by households, `(Σw)² / Σw²`. The top class is
  about two thirds New York City and is likely unresolved.
- **The generator.** The apportionment changes the distribution: mean `2.539` to `2.500`,
  W1 `0.051` persons between raw and apportioned, and households of 7 beyond the clamp.
  A port's details matter at that scale. **Fix:** score the twin's own emitted histogram,
  committed before the survey is read; the port is a cross-check.
- **S3** compared the ends of what is probably a hump (suburban tracts hold the largest
  households). **Fix:** the largest difference between any two resolved classes.
- **S4** was slack (`0.30` against about `0.7`, memory) and misnamed: the table merges
  detached and attached single houses.
- **Plants missing:** the file's "not computed" codes; a tract with zero land; the top-band
  mismatch; the port against the twin's file with the apportionment removed.
- **Ignored by the draft:** group quarters are 2 to 3 % of people and the twin makes each a
  household member; vacant units; tenure; Puerto Rico; suppression of the internet
  variables in 2023 and 2024; Connecticut's counties replaced by planning regions in this
  window (memory); the 2020 year at two thirds of the usual responses.
- **Misfit gloss.** M-FOREIGN-DOMAIN-CORROBORATION is about crediting a result to code it
  did not exercise. The contact is the port of the twin's rule, not other countries.

**What will not carry to a European grid country:** margins of error, the tract-against-leaf
caveat (a grid cell is a tile), county blocks and state hold-out, the 7-or-more band. What
replaces them is disclosure control: small cells are suppressed or perturbed, and those are
the low-density cells.

**What the author took and did not take.** Every fix above is applied in
`HOUSEHOLD0_PREREG.md`. The internet share is dropped to a later freeze. Structure type and
tenure are kept as readings with no bar. S2 is kept, rewritten at the county, against the
reviewer's suggestion that it could wait: it is the one stake here whose outcome nobody has
forecast.
