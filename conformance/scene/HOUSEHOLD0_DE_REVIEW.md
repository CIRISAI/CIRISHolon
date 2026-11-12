# HOUSEHOLD-0 (Germany) — adversarial review of the draft, 2026-11-11: freeze only after named fixes

*A second reader attacked `HOUSEHOLD0_DE_PREREG_DRAFT.md`. No data file was fetched or
opened, no service called, no documentation page opened; data facts are from
`EUROPE_SCOUT.md`. One single-threaded synthetic script (`household0_de/review_synthetic.py`)
at cell sizes 20, 40, 80 and 150 households, not the draft's. No US result number was used
to set a German bar. The author's condensation; the draft is kept as written.*

**Verdict.** Three faults would have been frozen.

1. **The disclosure check was blind to the bias it was there to catch.** Whether pooled
   shares are biased depends on one unpublished fact: whether an original 1 or 2 goes to 0
   or 3 with its expectation kept. If it does not, the tail is badly off while the ratio of
   parts to total stays inside 1 %:

   | cell households | rule for a 1 | parts / total | 6+ share (true 0.015) | one-person (true 0.360) |
   |---|---|---|---|---|
   | 40 | 0 or 3, even | 1.006 | 0.0177 | 0.358 |
   | 40 | always 0 | 0.978 | 0.0057 | 0.368 |
   | 20 | always 0 | 0.940 | 0.0033 | 0.383 |

   The floor of 10 households excluded no cell in any run and does not touch a per-column
   bias. **Fix:** the 10 km file is the referee. Each of its columns is perturbed once on a
   large true count, so the sum of 1 km cells against it measures the bias column by column.
   The draft read one line of that file; it must read all six columns.
2. **S5 tested almost nothing about the US.** It took US values for three classes the US
   results call unresolved. Restricted to classes resolved in both countries the US
   increments are a few hundredths, so it asked only whether two German classes sit near
   the German national value. "Class minus national" also moves with each country's mix of
   classes, and the US key is a tract where the German key is a tile. **Fix:** a reading,
   three transfers printed, no stake.
3. **The budget carried a US-regime constant.** `0.12` was 5 % of the twin's 2.5. At German
   household sizes 5 % is nearer `0.10`. **Fix:** decide before the read whether the budget
   is relative or absolute, and say which.

**Also.**
- **The split floor was asymmetric.** As drafted a miss of 0.119 with a gap of 0.05
  certifies at 0.12. **Fix:** certified if miss + gap ≤ budget; refused if miss − gap >
  budget; otherwise unresolved. In a full count the gap measures how different places are,
  not how well the reference is known. P-FLOOR-SPLIT named no budget and was right only at
  0.03.
- **The hold-out could not see regional structure.** Interleaved 100 km squares put east
  and west in both halves. **Fix:** contiguous bands; the interleaved result beside it.
- **Persons per household:** weight the mean-size file by the published total column (the
  count the publisher divided by), not by the sum of parts. Add the residual mean of the
  6+ band as a check; in one full count it should read between 6 and 8.
- **The building top band:** numerator and denominator from the same pair of files and the
  same category. A class value under 13 dwellings per building is impossible and is an alarm.
- **P-CKM at 300 households never exercised the small-count rule; P-CKM-SMALL could not
  fail.**
- **S1 was a confirmation.** An unknown version: is the twin's FAMILY wrong or only its two
  numbers? Fit the rule's mean and spread to the national values, then score the rest of
  the shape.
- Cells are classed by a population count that includes communal residents and is itself
  perturbed; border cells hold only their German part.

**What the author took and did not take.** Each fix is applied in `HOUSEHOLD0_DE_PREREG.md`.
The buildings reading is kept, against the "smallest honest version", as a reading with its
alarm. The family fit uses this campaign's port of the rule, which matched the twin's own
file to 0.0014 at the twin's parameters; it is labelled a port.
