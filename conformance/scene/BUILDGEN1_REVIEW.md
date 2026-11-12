# BUILDGEN-1, the adversarial review of the draft (2026-11-12)

One reviewer subagent read the draft, the parent campaigns and the survey's codebook.
Nothing was downloaded. **Verdict: do not freeze as is.** Ten must-fixes. Each is applied
in `BUILDGEN1_PREREG.md`; its sizes marked as general knowledge stay priors.

1. **The generator could emit impossible dwellings.** The survey's total energy includes
   electricity, so total is at least 3.412 thousand Btu per kWh; a copula over
   (electricity, total) breaks that. And fuel, drawn after the copula, was independent of
   energy given type and zone, which is strongly wrong. Now the copula carries
   electricity and NON-electric energy, fuel is drawn before it, and the marginals are by
   fuel group.
2. **Not-applicable was not structural, and storeys is not ordinal** (it has a
   "split-level" code, and is asked of houses alone). Now by type, in the type system.
3. **The tie convention and estimator were unstated,** and the obvious one fails D1 by
   construction on banded attributes (attenuating twice). Now mid-rank Spearman, with
   each latent correlation fitted so that the GENERATED Spearman equals FIT's.
4. **The rival was unfair** (it did not know type, so it lost on type alone). Now the
   rival is the generator with the identity matrix and no era.
5. **D1 pooled could pass with wrong within-type structure.** Now also within detached
   houses and within apartments in buildings of 5 or more.
6. **D2 and D3 were reproduction checks.** They are relabelled so, and new stakes reach
   quantities carried only by the dependence (energy by area tercile, era and fuel within
   detached houses; energy per floor area newest to oldest; joint tables).
7. **Thin cells had no rule, and tunable choices were left to after the FIT read.** Eras,
   neighbour zones, floors, drop order, the area column, knots and the normal are fixed
   in the freeze.
8. **The scored population and the bar's error terms were unstated.**
9. **The branch list had a hole and a contradiction.** A five-branch list, first match.
10. **"Known before the read" was inaccurate:** housing type and urban type HAVE been read
    on both halves by earlier campaigns; kWh has not.

Should-fix, applied: the tail stake redefined so it can resolve (top 5 % of both, within
detached, as a ratio); the gate in a separate process on FIT rows alone; the normal as a
committed integer table. Notes applied: knots capped at the 1st and 99th percentiles; the
energy is "billed and modelled", not metered.

The reviewer's table of dependences worth staking (energy per floor area falling with
later vintage; electric heating by zone and era; energy by heating fuel; wall material by
zone and era; type by urban type) is the list the revised stakes are built to detect.
