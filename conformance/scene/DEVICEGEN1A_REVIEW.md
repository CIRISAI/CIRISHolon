# DEVICEGEN-1A, the adversarial review of the draft (2026-11-12)

One reviewer subagent read the draft, the parent, its script and results, and the shipped
code. Nothing was downloaded. **Verdict: do not freeze as is.** Ten must-fixes. The draft
is NOT revised yet; this file records what a revision owes.

1. **The "gap" was redefined** as the FIT-to-SCORE difference. DEVICEGEN-1's gap is two
   jackknife standard errors. As drafted the candidate could not fail A1.
2. **A1's bar is loose at sizes 3, 4 and 5 or more** (it passes a none-share from 0.010 to
   0.064 at size 4). Sharper: 2·sqrt(se_FIT² + se_SCORE²), no added constant, with a
   neither zone at 4 of 5; and stakes on quantities the switch is not fitted to (none by
   size and age, by size and building; households with more phones than persons).
3. **The switch ignores age,** so an adult over 64 living alone gets the same chance of
   none as a younger one. That is the group MESH-0 flagged. The switch needs the senior
   flag, with fallbacks.
4. **The per-person estimator was not stated and the obvious one is biased upward** by
   truncating twice: about +8 % in the mean for two adults over 64, +23 % with no
   connection (a synthetic simulation). Use the zero-truncated fit, cap the redraws, and
   plant it.
5. **The rival loses by construction as written** (fitted on holders and redrawn, it has
   no household with none). It must be fitted on all households with no redraw.
6. **The branch list does the wrong thing in some outcomes** (a rival killed on the mean
   blocks a good candidate; two branches each hold two outcomes). A five-branch list is
   proposed.
7. **P-STREAM cannot hold as described.** Steps 3 to 5 share one stream; the amended path
   must replay the old step-3 picks on the main stream and draw the new ones from a
   named child stream clear of the one MESH-0 already uses.
8. **A4 was vacuous:** the have-nothing ratio does not depend on phones. Replace with the
   clustering of "no smartphone" with "no computer", and mobile-data-only households
   with no smartphone.
9. **The key packing collides** if a size band goes in the natural slot. New table ids
   for the switch and for the banded shares; the old table untouched.
10. **No gate, and the script loads SCORE with FIT.** A gate on FIT rows alone first.

Should-fix: bound the second read of SCORE ("reproduced on a half read twice", and no
third read); the person-level share IS bounded by the survey (one-person households are
persons; per-household lower and upper bounds), so stake it; replace the unmeasured
"0.050 or so" row; name the new entry point; the verdict for "adults with no smartphone"
stays `unresolved` with "2020, households" in its reason whatever the stakes read.
