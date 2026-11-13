# BUILDGEN-2, the adversarial review of the draft (2026-11-12)

One reviewer subagent read the draft, BUILDGEN-1's files and the shipped carrier, and ran
simulations on synthetic tables. Nothing was downloaded. **Verdict: do not freeze as is.**
Seven must-fixes. The draft is NOT revised; this records what a revision owes, and why
the campaign is split in two.

1. **L1 was biased toward the bigger pool and its kill fired under a pure null.** With no
   structure at all, a 4-state division against a 12-state zone reads "division nearer"
   in 35 % of pairs. Pools must be equalised in donor states and households.
2. **L2's "2 times the resampling floor" could not see differences the size of D3's kill
   bar** (the floor of a 35-cell table at 200 households is about 0.10). A permutation
   test of state labels within the division replaces it.
3. **At a floor of 100 "keyed by STATE" is mostly nominal:** about 6 of 1,020 type by
   fuel group by state cells would reach it. Drop order, floors by table, and a report
   of how many households are served at each level.
4. **D3's "fuel by STATE" bar sat at the noise floor** of a perfect model.
5. **L1 and L2 used SCORE households and then changed the generator that SCORE scores.**
   They must be computed first, on FIT alone, and fix the ladder.
6. **The Student copula as drafted:** matrices must be refitted for each candidate; the
   tail stake was close to foregone; symmetric tails on all pairs are an untested
   prediction (a bottom-corner stake, or scaling the size and energy scores alone); the
   integer tables cannot reuse the normal's clip.
7. **The shipped key cannot hold a state,** and a call-time ladder needs a lookup that
   returns "absent". New functions beside the old, the old pinned by a digest test.

Should-fix: D1 to D3 on random halves of the same states are split-half checks of form,
not prediction, and several are near-foregone after BUILDGEN-1; the split must name its
hash and column (`DOEID`); states alone in a pool or spanning zones; what "regional
norms" can honestly mean (the division is contiguous geography and proxies climate, the
gas grid, prices and age; a degree-day rival and a data-driven grouping fixed before the
read); and that the twin holds neither the survey's zone nor its urban type.

**What follows from it.** The question of WHICH key carries the joint (finding 1, 2, 5,
10, 11) is a reading of the survey that needs no generator, and its answer decides the
generator's ladder. It is taken first, alone, as REGIONNORM-1, with the review's wording.
The generator (findings 3, 4, 6, 7) follows as its own campaign once that is read.
