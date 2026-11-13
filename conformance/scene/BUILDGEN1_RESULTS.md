# BUILDGEN-1 — results (2026-11-12): killed. The joint is real, and it does not travel between states on a climate zone alone

Freeze `BUILDGEN1_PREREG.md` (committed alone at `9fa6b43`), review `BUILDGEN1_REVIEW.md`,
record before the parse `buildgen1/BEFORE_THE_PARSE.md`, script `buildgen1/buildgen1.py`,
gate `buildgen1/gate.json`, numbers `buildgen1/results.json`. The survey file carried the
recorded digest and was deleted after the run. SCORE was read once.

## Branch (k): the generator is killed on D1 and D3. Nothing ships.

- **The gate passed on the fitting states.** 9 of 9 plants; the marginals; the section
  law; and the generated rank correlations within 0.002 of the fitting states' own.
- **D1, the dependences: KILL.** Pooled, 6 of 10 pairs within the bar (the kill was 6 or
  fewer). Within detached houses and large apartment buildings, 9 of 16 (the kill was 10
  or fewer).
- **D3, materials and fuel: KILL.** Distance between generated and held-out joint tables:
  wall by roof 0.195, fuel by zone 0.167, fuel by era 0.099, type by urban type 0.035
  (met). A perfect generator at that sample size shows about 0.02.
- **D2, power through the joint: neither.** 14 of 18 cells certified, 1 refused (total
  energy of detached houses heated by propane, oil or wood: 108,801 generated against
  87,907 thousand Btu). Energy per unit of floor area, newest era over oldest: 0.716
  generated against 0.761, inside its bar.
- **S4, the rival: the joint IS needed, and the stake reads neither.** The rival with no
  dependence is outside the bar on 6 of 6 pairs within detached houses (it gives −0.05
  for floor area against electricity where the survey has +0.29). By its letter S4 is
  met only if D1 is, so it reads neither.
- **S5, the tail: KILL of the normal copula's tail.** Houses in the top 5 % of both floor
  area and total energy are 1.6 % of detached houses in the survey and 0.8 % generated.
  The largest homes with the largest use coincide twice as often as a normal copula
  allows.
- **S6, era: not shown** for wall or fuel.

## Why it was killed: the two halves of the country do not share one joint

The generator reproduced the fitting states. The held-out states are different, by more
than sampling error. For detached houses:

| rank correlation | fitting states | held-out states (gap) | generated |
|---|---|---|---|
| vintage with storeys | −0.02 | +0.11 (0.03) | −0.05 |
| storeys with electricity | +0.02 | +0.09 (0.03) | +0.02 |
| floor area with electricity | +0.22 | +0.29 (0.03) | +0.23 |

And the same climate zone means different things in different states:

| share heated by electricity | held-out states | generated from the fitting states |
|---|---|---|
| Hot-Dry | 0.20 | 0.42 |
| Mixed-Dry | 0.09 | 0.42 |
| Hot-Humid | 0.64 | 0.55 |
| Cold | 0.23 | 0.16 |

- A climate zone and an urban type do not carry what the lead called regional norms.
  "Hot-Dry" is one fuel mix in one state and another in the next. The split was by state
  at random, so each zone's states fell unevenly on the two sides, and the generator had
  no key for which side of that it was on.
- **This is the finding, and it is the kind the campaign was built to make:** the
  dependence among a dwelling's attributes is real (the rival fails everywhere), it is
  heavier in the tail than a normal copula, and it is REGIONAL below the level of
  climate. A generator keyed by climate would draw Arizona's houses in California.

## Readings that came with it (no verdict)

- Energy per unit of floor area falls with later vintage in the survey's detached
  houses, from 46.4 thousand Btu per square foot before 1950 to 31.1 for 2016 to 2020,
  and the generator follows it (54.8 to 31.3), overstating the oldest band.
- Mean electricity by type is near (detached 12,500 kWh in the survey, 13,088
  generated). Mean total energy is overstated for attached houses and small apartment
  buildings (79,664 against 54,830; 67,282 against 39,239): their cells are thin and
  were pooled across fuel groups, which the floor rule allowed and should not have.
- Homes with no fuel but electricity: 0.77 of Hot-Humid homes in the held-out states,
  0.42 generated.

## What exists and what does not

- `holon_scene::buildings::generate_with` exists, is tested and builds for wasm32. NO
  tables ship with it. There is no `buildings::generate`, and a caller has nothing to
  call. The roof-tone mapping stays an assumption on paper.
- The survey's columns named in the freeze have now been read on SCORE. A second
  campaign on this split would be a second read and must say so.

## What follows (not started)

- A generator keyed by REGION (the survey's census division, 10 of them, or the state),
  with climate inside it, and a held-out design that asks the right question: do states
  held out WITHIN a division share its joint? That needs a new split and a new freeze.
- A copula with a heavier upper tail for size against energy.
- A pooling rule that never lets go of the fuel group.
- For the twin: nothing to draw from this yet. Its constants stay, marked as its own.
