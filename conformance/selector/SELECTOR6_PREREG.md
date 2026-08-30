# Pre-registration — SELECTOR-6: the landscape, done properly

*Frozen 2026-08-30, committed ALONE, on the floor SELECTOR-5's refutation
built (SELECTOR5_REFUTATION.md). Every requirement below exists because a
specific defect was demonstrated there, and the mapping is stated so the
next reader knows which scar each rule is.*

misfits: contacts M-TAG-AS-PROPERTY (the tag-shuffle plant is gate I1's
audit half — the refuted C4 is this misfit's founding case),
M-BASE-RATE-OMITTED (gate E1 exists in the exact shape this misfit
demands: eligible-pool rate plus permutation null, never raw precision),
M-PRESENTATION-VERDICT (gate I1 and plant (ii); A4/Delta(12) is the
mandatory regression pair), M-POPULATION-CHOICE (the population section:
declared rule, A000001 completeness gate, exclusions named),
M-CONJUNCTION-MONOTONE (no thinning claim of any kind is staked or may
be reported as a finding), M-GAUGE-LAUNDER (verdicts are functions of
the group alone), M-HOMOG (the enrichment statistic's null), 
M-NULL-MISSTAKE (branch (b) is a RESULT and is staked as one),
M-PLANT-OBS and M-PLANT-SECTOR (plants below, carriers asserted),
M-FINAL-VIEW-COLLISIONS (identity sets by exact closure, budget
exhaustion VOIDs loudly, per the SELECTOR-4 instrument), M-PARITY-PROTECT
(not contacted beyond the standing spin-audit convention),
M-STALE-INSTRUMENT (instrument, census, results committed together),
M-NONBIJECTIVE-STEP, M-FIXED-POINT-TRAJECTORY, M-PROBE-EIGENSTATE,
M-LOOP-BLIND, M-BARE-CHARGE, M-COND-PROBE, M-ELECTRIC-BASIS,
M-RING-MIXING, M-GAUGE-UNIFORM-MOMENTUM, M-KINEMATIC-NONLOCAL,
M-VOLUME-SCALE (not otherwise contacted).

## The criterion — inherited, not invented

The selection criterion is the SELECTOR-4 bootstrap gauntlet EXACTLY as
frozen (SELECTOR4_PREREG.md; instrument selector4.py): the acuity ladder
of cumulative coarsenings, the invariant-partial-section act vocabulary,
exact rho-closure separation with a declared budget, selected = passing a
proper nonempty subset test at the finest decided rung. NO new gate may
be added, weighted, or thresholded in this campaign. Generalization to an
arbitrary finite group is by MULTIPLICATION TABLE ONLY, and every
group-specific construction choice (the step, the coarsening chain) must
be made by a rule that is a class function — verdicts aggregated over
conjugacy-class representatives of the choice, so that isomorphic
presentations provably receive identical verdicts (gate I1). The
refutation's A4-vs-Delta(12) pair is the permanent regression.

## Population — declared, complete, and checked against the world

- PRIMARY: every isomorphism type of order 1..63. Completeness is GATED,
  not claimed: for each order n the census count must EQUAL A000001(n)
  (the published number-of-groups sequence, pinned as a data file with
  its own provenance line); any order whose count disagrees is VOID for
  every claim, loudly, and no landscape statement covers it.
- Deduplication: invariant fingerprint (order profile, class sizes,
  abelianization, center, derived length) followed by explicit
  isomorphism search on every collision. The three duplicate pairs the
  refutation found are the demonstration case: the census must count
  them once.
- HELD-OUT, declared now and not touched until H1: (a) all isomorphism
  types of orders 65..71; (b) the named 2-swamp panel the refutation
  said the old population was protecting itself from: both extraspecial
  groups of order 32, both of order 64, and the generalized quaternion
  groups Q64 and Q128. No number from any held-out group may be computed
  before the primary record and the H1 prediction are committed.
- Orders 64 and >71 (beyond the panel) are EXCLUDED and the exclusion is
  a stated limitation of this campaign, not a secret of its result.

## Labelling — RULE-B only

SM(G) iff G embeds in SU(3) x SU(2) x U(1), decided exactly by the
refutation's RULE-B (refute_lib.py: character-table witness — a degree-3
det-1 representation, a degree-2 det-1 representation, and a linear
character with trivially-intersecting kernels), frozen at the sha the
refutation pinned. Construction tags are FORBIDDEN as inputs to anything.

## Gates

- **I1 — isomorphism invariance, demonstrated**: for >= 8 staked pairs of
  distinct presentations of one group (A4/Delta(12) mandatory), verdicts
  identical, and the criterion's inputs are audited to be the
  multiplication table alone (the tag-blindness plant below). witness:
  none (measured + audit)
- **S1 — census completeness**: per-order count == A000001(n) for every
  primary order, else that order VOIDs. witness: none (exact count)
- **T1 — theorem consistency, predictions not filters**: every abelian
  group must select NOTHING (the stasis theorem's prediction) and every
  ambivalent group's oriented sector must be empty (FROB-ORIENT-1's).
  A violation is an INSTRUMENT bug and VOIDs the run for diagnosis —
  these are the banked theorems auditing the code. witness:
  frob_not_ambivalent (the orientation half); the stasis half
  is instrument-checked, its dedicated brick being this campaign's owed
  formalization, named here rather than implied
- **E1 — THE STAKE: enrichment over the base rate**: the RULE-B fraction
  among selected groups versus the eligible population, with an exact
  permutation null (>= 10^4 draws of equal-size subsets, seeded, seed
  staked). Branch (a): observed enrichment exceeds the null's 99th
  percentile. Branch (b): it does not, and the landscape's answer is
  "the bootstrap criterion does not preferentially select SM-embeddable
  structure at these orders" — reported with exactly the prominence a
  branch (a) would have received. NO third option, NO added criteria,
  NO relabelling after this line. witness: none (measured)
- **H1 — the forward test**: after the primary record is committed, the
  branch verdict and the per-group predictions for the HELD-OUT set are
  committed BEFORE any held-out computation runs; the held-out run then
  scores them. Agreement is rule-6-grade support; disagreement is
  reported as the finding it is. witness: none (measured, sequenced by
  commits)
- **B1 — budget honesty**: per-group compute budget declared in the
  instrument header; exhaustion -> that group VOIDs loudly and appears
  in every table as VOID, never as a silent absence. witness: none
  (contract gate)

## plants (carrier and sector per M-PLANT-SECTOR)

Each plant's carrier is asserted nonzero in the sector the plant acts on
before the plant is scored; a plant on an empty sector VOIDs.

- **(i) tag blindness**: shuffling every construction tag and group name
  must change NO verdict (carrier: the verdict vector, asserted
  nonempty). The refuted C4 is why this plant exists.
- **(ii) the isomorphic-pair plant**: a deliberately re-presented group
  (permuted multiplication table) must receive the identical verdict,
  and a deliberately WRONG table (one entry corrupted, checked to break
  associativity or an inverse) must be REFUSED by the census validator,
  not scored (carrier: both outcomes demonstrated).
- **(iii) the null-integrity plant**: the permutation null run against a
  label vector that is ITSELF a uniform random relabelling must report
  no enrichment at the staked threshold in >= 99% of trials (the null
  must be able to say nothing when there is nothing).

## Meaning

Branch (a) => "the banked bootstrap criterion, run uniformly over the
complete small-group landscape, selects SM-embeddable structure beyond
its base rate, and predicted held-out behaviour in advance." Branch (b)
=> "it does not, at these orders" — an Omega-internal null with teeth,
and the honest state of rung 7 either way. NOT claimed under any branch:
that these orders are nature's, that embeddability is physics, that any
group IS the Standard Model, or anything about orders this census does
not cover.

---

## AMENDMENT A1 — 2026-08-30, at design re-audit, before any census or run

*Three items, each decided at the design review (SELECTOR6_DESIGN.md,
commit 1d5b23b) and none after any number existed.*

1. **The I1 mechanism clause is corrected by proof.** The freeze suggested
   aggregating verdicts over conjugacy-class representatives of the step
   choice. The design traced the criterion and showed (a) the step itself
   is canonical — written in the group operation, commuting with every
   isomorphism — and the only non-canonical construction was the family's
   representative choice; and (b) representative aggregation is NOT a
   class function (changing one class's representative conjugates one
   family member independently, so no gauge element relates the families).
   The mechanism that ships instead is stronger: the FULL family
   F(G) = {GAUGE[g] ∘ step^d : g in G, d | ord(step)}, deduplicated as
   permutations — no choice exists, an isomorphism carries F(G) onto
   F(G'), and I1 holds by construction. The representatives' economy is
   recovered as a proved gauge-orbit decomposition, with orbit constancy
   re-checked at a second random representative per group and any
   disagreement a loud VOID. The gate I1 is unchanged; only the freeze's
   suggested mechanism is superseded, and this note is its record.
2. **T1's stasis half is pinned to the criterion's own quantity:**
   "SELECT(G) = False for every abelian G", with (k*, |sel|, |F|) printed
   per group. The design's trace shows abelian groups fail selection in
   TWO modes — sel empty at coarse views, sel = F(G) (everything) at the
   discrete view — and both are non-selection since neither is a proper
   nonempty subset. A literal "sel is empty" assertion would VOID the
   abelian half of the census on the second mode; the pinned reading
   keeps the theorem's prediction exact while putting the select-
   everything behaviour on the page. The results doc must report the two
   failure modes' counts separately.
3. **A000001's provenance is a three-legged pin:** the primary-source
   b-file fetched by the lead and committed as A000001.pin (sha256
   54358f9b…1ad5a0f3), cross-audited by the census's two internal
   theorems (exact abelian counts from partition products; Holder's
   formula on squarefree orders). A typed-from-memory sequence is
   exactly the unchecked constant this programme distrusts, and the
   completeness gate S1 now checks an argument against an external
   record, not a hope against a builder list.

---

## AMENDMENT A2 — 2026-08-30, at design v2 re-audit, before any census or run

*Two items from the design's second pass (SELECTOR6_DESIGN.md v2, commit
be7092f), the first being the campaign's most important pre-run catch.*

1. **The decided-rung rule, corrected before it could confound E1.** The
   v1 definition ("k* = the finest rung with zero VOIDs, A0 always
   decided") composed with two facts of the predecessor's own log —
   coarse rungs select nothing BY CONSTRUCTION on every world, and the
   largest world VOIDs its fine rungs on budget — into a silent
   disaster: budget exhaustion would fall back to a coarse rung and be
   recorded as SELECT = False, with the expense scaling in
   |F| = |G| x #divisors(ord step), a structural property plausibly
   correlated with the label. E1 would have measured the budget and
   called it physics. The rule that ships: a VOID at rung k removes k
   AND every finer rung; if the finest survivor is coarser than A3, the
   group VOIDs — it never scores False. VOID counts are reported by
   order and by |F| so a structured refusal pattern is visible on the
   page. This is B1's "exhaustion VOIDs loudly" made airtight against
   the one composition that would have laundered it.
2. **The criterion is imported by proved extraction, because importing
   it directly would have destroyed the predecessor's evidence.**
   selector4.py opens its committed run log at module scope in mode
   "w"; a bare `import selector4` TRUNCATES conformance/omega/
   selector4.log — the banked SELECTOR-4 record. The extractor
   (make_s4core.py) reads the PINNED BLOB (never the working tree),
   neutralises exactly that one line, and proves it did no more: refuses
   unless the source hashes to the pin, unless exactly one line matches,
   unless the diff is exactly that line, unless the line count is
   unchanged; and asserts the log's byte size after import. The log is
   verified byte-identical. Recorded as the general lesson: importing an
   instrument executes its side effects, and a predecessor's record must
   be reachable only through a proof-carrying door.

Budgets declared per the measured cost model (worst real shape F_57 at
~30,408 closure calls — |F| tracks |G| x #divisors(ord step), so a
budget derived from |G| alone would have been wrong): BUDGET = 2000
(inherited), GROUP_BFS_BUDGET = 300,000 (10x measured worst),
GROUP_WALL = 3600 s (20x scaled worst), CENSUS_WALL = 24 h.
