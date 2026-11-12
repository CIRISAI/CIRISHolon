# CONNECT-0 — results (2026-11-12)

Freeze `CONNECT0_PREREG.md` (committed alone at `da2cf7d`), review `CONNECT0_REVIEW.md`,
record before the parse `connect0/BEFORE_THE_PARSE.md`, script `connect0/connect0.py`,
log `connect0/run1.log`, numbers `connect0/results.json`. Raw files deleted after the run.

## One table of six was written

| unit | branch |
|---|---|
| ACS 2024, B28002 by state | **(a) written**, `tables/connect0/acs_b28002_states.json` |
| Eurostat countries | **(r)**: R2 reads misread by the frozen rule; not written |
| Eurostat regions | not written, because the country table is not |
| ITU K1, K3, K4 | not obtained |

## The United States, by state (the one table)

- **R3 met:** the identities among the counts hold for 52 of 52 areas.
- **S2 met, a reproduction:** the share of households with any access at home less the
  share with a paid cable, fibre or DSL line is over 5 points in 52 of 52.
- **S2b neither:** the 10th and 90th percentiles of that gap are 16.09 and 24.02 points,
  7.93 apart. The bar for "one number" was 5 and the kill was over 8. So the gap is not a
  constant across states, and it stops just short of the kill.
- **The range:** New Hampshire 12.76 (96.96 with access, 84.19 with a line); Puerto Rico
  33.13 (87.77, 54.63); Alaska 30.27; the median state 19.18.
- **The 50 states and DC together:** any access 95.46 %; any subscription 93.32 %; a line
  76.53 %; cellular plan and nothing else 12.60 % (by state 8.84 to 25.80); no access
  4.54 %.
- **For MESH-0:** about three households in four have a line that a host could sit on,
  as a ceiling. "Has internet" overstates that by 13 to 33 points depending on the state.
  This is one country, one survey, one year.

## Eurostat: refused by a bar I set too tight

- The file gives EU-27 2025 as 94.76. The frozen rule was "rounds to 94.7 at 1 decimal".
  94.76 rounds to 94.8. By the rule: misread, no table.
- **What the other checks said:** 27 of 27 members present; the EU value lies between the
  members' lowest and highest (87.91, 99.28); no neighbouring selection rounds to 94.7
  (the euro area reads 94.55); 2023 reads 93.09 where a summary had said 93.1.
- **My reading, not a verdict:** the selection is very probably right, and the page's
  94.7 was 94.76 cut short or an earlier revision. The draft's bar ("within 0.1") would
  have passed it; the reviewer's stricter form, which I adopted, did not. The rule was
  frozen and it fired. I have not written the table and will not on my own reading.
- **What would put it right:** the lead's word to accept the Eurostat tables under a
  dated amendment that says the bar was changed after the value was seen. R2b (the
  regional table against the country table) has not been run.

## ITU: not obtained

Its DataHub answers this machine with HTTP 403, licence page included, and no browser was
connected. Said in the record before the parse as a departure from the freeze's list of
reasons. P-KIND and P-K4 (plants for ITU files) and S1 and S3 were not run. No world
table of any kind exists from this campaign.

## What this does not license

- Nothing here is "a host that is on". A line says nothing of a public address, of
  uptime, or of a machine left running.
- The US gap is not to be carried to another country.
- Shares are from a sample survey; the margins in the table are for counts.
