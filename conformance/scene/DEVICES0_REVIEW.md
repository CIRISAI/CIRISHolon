# DEVICES-0 — adversarial review of the draft, 2026-11-11: freeze only after named fixes

*A second reader attacked `DEVICES0_PREREG_DRAFT.md`. No data fetched or opened, no page
opened, no computation run; the standard errors are first-principles arithmetic on assumed
values, stated with each. The author's condensation; the draft is kept as written.*

**Verdict.** Two structural faults.

1. **The budgets were below what the sample can resolve, so the main stake was decided
   before the read.** With about 18,500 households, a design effect of 1.5 and counts whose
   spread is 0.4 to 1.7 times their mean, the relative standard error of a mean count is
   0.7 to 2.9 % for one-person households and 1.5 to 6.3 % for five-person ones. Under a
   half-against-half hold-out the gap is four times that. Against a 5 % budget seven of
   the ten kinds could not certify at any household size, and the "share with none"
   verdict could never certify. The 5 % was HOUSEHOLD-0's number, carried to a regime it
   was not priced in: for the energy delta a count multiplies a device power known to
   perhaps ±50 %. **Fix:** 15 % on the count and `0.05` on the share with none; no state
   hold-out; cells pooled upward until their error is at most half the budget.
2. **"On all day" is not what the delta needs.** A set-top box and a smart speaker are on
   all day and cannot run anyone else's software. What decides "devices added" is whether
   a household has something on all day that could HOST. **Fix:** two declared attributes
   per kind (power state; can host). A gateway is declared present wherever there is
   fixed broadband. Homes with cellular data only are a third state.

**Also.**
- **Per person is the wrong quantity.** Five-person households are mostly children, so
  "phones per person fall with size" was foregone. **Fix:** three rates (per adult 18 to
  64, per adult over 64, per child) and a stake on whether they predict the count at each
  household size. The top band (7 or more) is left out of every per-person stake.
- **The year comparison could not tell drift from a pandemic year and a first release,**
  and was foregone (smart speakers rose and set-top boxes fell by far more than 5 %).
  **Fix:** use the later year as the hold-out: a table fitted on 2020, scored on 2024.
  The 2015 survey as a third point, if its codebook carries the same counts.
- **Urban type is settlement size, not density.** Housing type is the nearer stand-in and
  the twin generates building types.
- **Errors:** compute each difference inside each replicate.
- **Unknown outcomes the draft missed:** whether have-nots cluster (homes with no
  computer and no fixed broadband, against the product of the separate shares), which
  decides whether kinds may be drawn independently; and whether counts are narrower or
  wider than a Poisson, which picks the generator's family.
- **To read as well:** how many weekdays someone is home, and television hours.
- **Scope:** the twin's 2.25 and 1.0 are world figures; a verdict here is about the US.

**What the author took and did not take.** Each fix is applied in `DEVICES0_PREREG.md`.
Both years are read (the lead approved both); 2015 is read only if its codebook has the
same counts, and then as a reading.
