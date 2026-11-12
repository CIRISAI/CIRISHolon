# TIES-1 — adversarial review of the draft, 2026-11-11: freeze only after named fixes

*A second reader attacked `TIES1_PREREG_DRAFT.md`. It read the questionnaire and the data
file's first line; no data row, topline or crosstab. One synthetic script
(`ties1/review_synthetic.py`). The author's condensation; the draft is kept as written.*

**Verdict.** Three faults would have been frozen.

1. **Who is missing from "closest ties" is measurable in the file and the draft did not
   measure it.** "Discuss important matters" and "most significant" under-name the spouse
   and almost never bring up a child at home. The file carries the number of adults and
   children in the household and marital status, so the share of co-residents actually
   named can be read. As drafted, the household's share of contact was a FLOOR offered as
   the number. **Fix:** a coverage reading, and the household rows printed as named (floor)
   and with unnamed co-residents added (ceiling).
2. **The answer scale decided the household stake, and summed means are not messages.**
   With "several times a day" counted as 1, 3, 10 or 30 a day the household's share of
   electronic contact walks from 0.25 to 0.44 in a synthetic with no change in behaviour.
   A call, an email and a text were added as equals. **Fix:** days with contact as the
   primary; acts at four scales; a stake met only if it holds across them; written means
   and voice apart.
3. **The exclusive kind order answered a question the twin does not ask, and "not asked =
   zero" is confounded with age.** This instrument samples CLOSE ties; "co-worker" here
   is a close tie who is also a co-worker, not the work group. **Fix:** two keys (same
   house or not; relation) with co-worker, neighbour, group member and online as
   overlapping flags. And every table also for respondents who hold the means.

**Also.**
- **Take the household out of the "messages are nearer than ties" stake**: with it in, the
  stake mixes "near ties are contacted more" with "a spouse is called daily".
- **A known-answer check that costs nothing:** the file carries the publisher's own
  per-respondent counts; the reader must reproduce them exactly.
- **Stakes the draft lacked, each with an unknown outcome:** coverage; whether the near
  excess of contact grows with adoption of means (is the 2008 bridge a floor or a ceiling
  for today); and, connecting to TIES-0, the share of close ties within 50 miles that are
  on the same block or under a mile, where a 1/distance law on uniform population puts
  about 2 %.
- **The comparison with the twin's "65 % of acts self or family" is not like for like;**
  print it as two bounds.
- Truncation at five names, fatigue by name position and respondents with no names are
  to be printed. "Don't know" is missing, not zero; an empty name slot is not a tie.
- The header may carry a county code; if so the urban stake becomes a slope on density.

**What the author took.** Each fix is applied in `TIES1_PREREG.md`.
