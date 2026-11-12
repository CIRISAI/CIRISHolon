# PRESENCEGEN-2 — adversarial review of the draft, 2026-11-12: FREEZE WITH NAMED FIXES

*A second reader attacked `PRESENCEGEN2_PREREG_DRAFT.md`. No survey data fetched or opened;
no fitted table opened; no cargo. Read: the draft, the predecessor's freeze, results,
`diagnosis.json`, `results.json`, `run1.log`, `presence.rs`, `presencegen1.py`. Two synthetic
scripts, single-threaded, numpy, seconds each, in a scratch directory outside the repo
(`/tmp/tmp.MuIGi8UD8K/sim.py`, `sim2.py`). Every number below marked "synthetic" comes from
populations made up for the purpose (40,000 single-outing days; wake tied to the hour of
leaving; 35 % stay home and wake late) and says nothing about the diaries.*

**Verdict.** The keyed fill does what the diagnosis asks of the hourly shares. The freeze
around it does not: no inherited stake can see what G4 changes, one new plant fails as
drafted, the new gate clause is an in-sample reproduction, and the "seen" paragraph is not
accurate. Ten findings, each with a fix.

## Findings

1. **No stake reads what G4 changes; the stakes cannot judge it.** All four changes act on
   asleep-against-awake (at home, on return, within the frame, at 4 a.m. elsewhere). S1 and
   S2 are computed from `S > 1` and `S <= 1` alone (`spells`, `stats` in `presencegen1.py`);
   S4 reads states 0 and 1 together. They return the same numbers for G3 and G4 up to a
   second-order effect of the sixth inside state. Only S3 (Saturday home awake) sees the
   split. The draft's line "the stakes on 2024 are what judge it" is therefore wrong: what
   judges the redesign is the gate on FIT, which the draft itself calls weak.
   **Fix (S6, a stake on the split).** On 2024, quantities the fill does not hold as a row:
   (i) the share at home asleep in the hour before first leaving (see finding 2); (ii) the
   quartiles of hours asleep per day; (iii) home asleep at clock 7 among days whose first
   leaving is at 12 or later. Budget `0.03` / one hour, G3 scored beside. State now that G4
   is *judged* on S3 and S6, and that S1, S2, S4 judge the frame.

2. **The band key removes the fault in the hourly shares and leaves it in the joint.**
   The selection is forward-looking: at a slot inside the band the fitted population is
   those who have not yet left, and those who leave next hour are awake. The band fill
   gives all of them one waking rate. Synthetic check, generated minus planted:

   | fill | awake share of the day | worst hour, home asleep | asleep in the hour before leaving (planted 0.000) |
   |---|---|---|---|
   | one chain (G3's) | +0.037 to +0.046 | 0.10 to 0.16 | 0.33 to 0.48 |
   | four-hour band (G4 as drafted) | +0.0002 to +0.0016 | 0.007 to 0.018 | 0.04 to 0.17 |
   | hours until leaving, capped at 6 | +0.0001 to +0.0004 | 0.005 to 0.008 | 0.000 |
   | hours until leaving, capped at 3 | +0.0003 to +0.036 | 0.006 to 0.095 | 0.000 |

   So the band passes P-SELECT and G0b with room, and still sends 4 to 17 % of synthetic
   leavers from asleep straight out of the door. A countdown key (also 6 before-groups, so
   it fits the same 8 values) holds the joint, but a cap shorter than the wake-to-leave lag
   is worse than the band; my cap-6 result is exact by my own construction (wake within
   5 hours of leaving). A dedicated row for the last hour before leaving made it worse
   (0.25 against 0.17, synthetic): do not patch that way.
   **Fix.** Build both keys (band; hours until leaving capped at 6), freeze the rule that
   picks one on FIT across years (finding 4), and stake the joint as S6(i). If only the
   band is built, declare "asleep in the hour before leaving" unresolved in the call.

3. **After the frame: the hourly shares are safe, the joint is not, and it is undeclared.**
   The after-group grows, and each newcomer's state is drawn, so the marginals hold by
   induction (synthetic: asleep share of after-hours `0.4267` planted, `0.4266` generated,
   with sleep onset tied to return). The joint does not: planted "awake for two hours
   after a late return" came back as `0.52` to `0.56` asleep one hour after returning
   awake. Also unstated: whether the return table is keyed by employment, and what is
   drawn when the frame runs to the day's end.
   **Fix.** Say so under "vacuous"; add `time_asleep_after_return: unresolved` or key the
   after chain on "returned in the last two hours". State the return table's full key.

4. **G0b is a bar from the read, in sample, and vacuous for G1.** `0.02` and 22 of 24 were
   set after seeing G3's nine missed hours. A first-order chain applied to the population
   it was fitted on reproduces hourly shares by construction; G1 does (so the clause "G1
   fails G0b" is dead), and G4 does except inside a band (finding 2: worst synthetic hour
   `0.018`, under the bar). It also reads only home asleep, though the diagnosis shows
   elsewhere asleep `0.049` low at slot 0 and home awake `0.054` high at slot 4.
   **Fix.** Keep G0b labelled a reproduction check, and add **G0c, out of sample without
   2024**: tables fitted on 2022, scored on 2023's hourly shares, and the reverse; states
   0, 1 and 4 each within `0.02` in 22 of 24 hours, both directions. Do not use 2024's
   hourly shares for a gate: they are S3's and S4's targets.

5. **P-FIRST fails as drafted, for a reason the draft keeps.** The hour first left stays in
   two-hour bins with an even draw (`first = lb*2 + below(2)`). People already away at 4
   are a point mass at slot 0; the draw sends half of them to slot 1 and writes slot 0 as
   home. Synthetic: `0.099` planted away in the first hour, `0.049` generated. Keying the
   first state on "starts at slot 0" cannot repair a frame that does not start there. (The
   only positive hour in the diagnosis's home-asleep row is slot 0, which is what this
   smear would produce; I did not fit to it.)
   **Fix.** Slot 0 gets its own cell in the first-left table (13 cells become 14; the away
   table's bin index still fits 32). Give P-FIRST a G3 leg (expected low) so it contrasts.

6. **The other plants: two legs true by inspection, two thresholds loose.**
   - P-SELECT can fail and discriminates (one chain `+0.04`, band `+0.001`, synthetic). It
     reads only the day's mean. **Fix:** add the leg "asleep in the hour before leaving
     within `0.03` of planted" — the band fails it as drafted, which is the point.
   - P-RETURN: "G3 has none asleep on return" is true by code (`h = 1`), not a test. G4's
     leg reproduces its own row; `0.03` hides a bin shifted by one when the planted share
     is flat over hours. **Fix:** plant none before 20:00 and two thirds after; tolerance
     `0.015` per bin.
   - P-NAP: "G3 zero" is true by code (no such state). `0.005` absolute against a planted
     share near `0.008` of person-hours passes a fill that makes a third of it. **Fix:**
     within a tenth of the planted share, and the share of *days* with such sleep within
     `0.02` (a chain scatters naps over days).
   - Add **P-S3-START**: see finding 8.

7. **The key does not hold a ninth group, and the fallback borrows the faulty row.**
   `key()` gives `x` 8 values; eight groups fill it, and `x = 8` carries into the employed
   bit and collides with another row. The "row over all groups" needs its own table id.
   Worse, that pooled row *is* G3's selected chain: a thin group borrows the fault this
   successor exists to remove. Structure: 736 nominal rows, 472 not empty by construction
   (a band has no before-hours past its last slot). Synthetic, 16,700 days with naps and
   late leavers: 391 rows seen, 58 under 30 diaries, 29 of them in the last band (first
   leaving after midnight) and 15 in "after" at early hours; `0.026` of days touch one,
   `0.0015` of home transitions. So "a tenth of days" will likely not fire and names no
   consequence; the damage sits in one or two groups.
   **Fix.** New table ids for keyed rows; pooled rows keep the old ids. Fallback order for
   a before-group: the next earlier band, then the no-outing group, pooled last. Print
   borrowed transitions per group; alarm if any group with over 2 % of days borrows more
   than a fifth of its transitions, and say what that does to the branch.

8. **S3 and S4 need rewording, and S3's code did not match its freeze.** `fit()` takes the
   frame's 4 a.m. state (`T_HOME_INIT`) from `within` (Monday to Thursday), not from
   `kindsel` (Saturday); the freeze says Saturday's. It never ran. With G4, "within-kind
   tables" must be listed: which of the keyed home chain, the return table, the keyed 4 a.m.
   state and the keyed first state come from Monday–Thursday or 2015–2019, and which from
   the target. Saturday × 8 groups × employment for the 4 a.m. state will be thin.
   **Fix.** A table in the freeze: each G4 table against {S3 source, S4 source}; a plant
   (P-S3-START) with a carrier whose Saturday 4 a.m. state differs, failing on today's code.

9. **Once the gate passes, branch (a) is reached almost by construction.** S2's quartiles
   of hours away and home at 9, 14, 18 are fixed by the frame's own tables (first left,
   hours until last home, one-hour cells) up to home hours inside; with a one-hour
   tolerance they are a reproduction check for G4 and a test only of G1. S1's eight-hour
   line is the away table read back on single-outing days since the CORRECTION (and P-LATE
   is now trivial for the frame). The real line is S1's two-hour one: it depends on the
   inside chain's breaks at home. "S1 or S2 met → ships" lets S2 ship G4 alone.
   Sorted: **real tests** — S1 two-hour line, S3, S4 (of the frame, not the fill), S6 if
   added; **reproduction for G4** — S1 eight-hour line, S2 quartiles, most of S2's three
   hours; **print** — S5.
   **Fix.** Branch (a) requires S1 met (both lines, as frozen) or S2 met *and* G4 within on
   the two-hour line; and S6 not failed for the call to carry sleep states as resolved.
   Label S2's quartiles R for G4 in the text.

10. **"What has been seen" is not accurate, and the successor's standing rests on it.**
    - `run()` calls `load(raw, [2022, 2023, 2024])`: every 2024 diary was parsed and its
      sampled hours built in memory before the gate. `results.json` holds
      `diaries_score: 7669`. "No diary of 2024 has been read by any script" is false as
      written; true is "parsed; no statistic but the count computed".
    - Also seen on FIT and not listed: kind shares (`0.184 / 0.301 / 0.032 / 0.484`), the
      six state shares, row counts (315 chain rows, 253 inside rows by slot), and three
      by-slot rows of the diagnosis (home awake and elsewhere asleep as well as home asleep).
    - PRESENCE-0's table pools 2022–2024 by state, hour and day type, and prints at-home
      9-to-17 for 2015–2019 against 2022–2024. So S3's and S4's diary-side targets are
      known in pooled form; say it in those words.
    - The author's forecast omits S2 and G0b.
    **Legitimacy.** Proceeding is legitimate: no 2024 spell, quartile or joint was
    computed, the fault was found on FIT, and iterating a fill against FIT is fitting. It
    becomes illegitimate if (a) any 2024 stake quantity is computed before G0/G0b/G0c and
    all plants pass; (b) a third fill follows a failed or disappointing 2024 read with 2024
    still the scoring year; (c) a budget, G0's `0.01`, or a kill moves; (d) G0b's bar is
    revised after a G4 number exists.
    **Fix.** Rewrite the paragraph with the list above. The script loads 2022–2023 only
    until the gate and plants pass (as `diagnose` already does), then 2024. One sentence in
    the freeze: this is the last successor scored on 2024; a further one waits for 2025 or
    scores on a declared held-out half.

**Smaller.** The coercion "the frame's last hour is away" must cover the new home-asleep
inside state, and its rate should be printed (it turns home hours into elsewhere awake).
`INSIDE_OF` maps state 0 to index 4 and must change with the sixth state. A drawn odd
first slot with a duration row fitted on the bin can overrun the day and is clipped; print
how often.

## Forecasts (the reviewer's, labelled as such; not derived from the diagnosis numbers)

- **Plants as drafted:** P-FIRST fails (finding 5). P-SELECT, P-RETURN, P-NAP pass.
- **Gate:** G0 passes for G4 and G1. G0b as drafted passes for both (it nearly must). G0c,
  if adopted, passes for G4 with one or two hours to spare, not more.
- **S1:** G4 within `0.03` on the eight-hour line (reproduction); the two-hour line is the
  open one, a little better than even that G4 is within. G1 outside on at least one line:
  likely. S1 met: slightly under even.
- **S2:** G4 within; met. This alone carries branch (a) as drafted.
- **S3:** met, 18 or more hours, with the misses in the morning. Modest confidence.
- **S4:** no better than a coin; if it misses, it misses in the working hours.
- **S6(i), if added:** the band fill fails it; a countdown key passes.
