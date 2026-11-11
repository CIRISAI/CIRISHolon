# FSD: The world model — a generated world, universe to vacuum, that says where it holds

**Status**: DRAFT for the lead's lock. Sections tagged **[today]** are built and gated;
sections tagged **[proposed]** name a producer, a consumer and a check, or they do not belong here.
**Date**: 2026-11-11.
**Repo**: CIRISHolon. This FSD names *what* is built; [`MISSION.md`](MISSION.md) names *why*.
**Reads against**: CIRISHolon `0f63e4a` (`lean/CIRISHolon/{SceneCert,ScenePopulation,Ladder}.lean`,
`engine/crates/holon-scene`); issue #3; the CEWP twin at `twin/globe` `6075f3d` (read-only);
[`../TIERS.md`](../TIERS.md) of 2026-09-27.
**The lead's purpose line** (2026-11-11): *"goal is to enhance the holon engine with our worked
example to become a better world model."*

---

## §0 The one-sentence problem

The engine certifies a coarse view against a finer **motion**, tier by tier from the vacuum
up; it has nothing to say about a world that is **generated** from the top down to stand in
for a real one nobody holds in full — and the first consumer, the CEWP twin, generates every
street, building, household and device below a 14 km cell with nothing real beside them.

## §1 Findings this FSD stands on **[today]**

1. **The certificate for a generated scene is two objects** (`SceneCert.lean`). An exact one:
   the generator is a section of the coarse view, `v (gen c s) = c` for every seed. A
   statistical one: a transport cost between the generated ensemble and a reference sample,
   read per observable and per consumer. Neither discharges the other
   (`section_does_not_fix_fibre`), and matching every observable's own distribution does not
   certify a consumer that reads two together (`marginals_match_joint_differs`).
2. **Every generated object is a holon with a population** (`ScenePopulation.lean`). A
   parent's reading is the sum of its children's counts; it composes along containment and
   conserves the total. Relationships are the same law on pairs. The coarse edge does not fix
   the wiring (`edge_law_does_not_fix_wiring`). An object with two parents needs both to hold
   the same total, or no scene exists (`no_generator_of_totals_ne`).
3. **The ladder** (`Ladder.lean`). Generators of any depth collapse to one; zoom is
   consistent; a generated scene under a certified tier moves as its parent does
   (`meets_tier`, `meets_stack`); what leaves a parent is the sum of what leaves its children
   (`outflow_commutes`); an empty parent generates the empty scene.
4. **The exact half runs** (`holon-scene`, 39 tests, zero dependencies, integers only,
   builds for wasm32): apportionment, seeds with random access, the tree, the edge quotient,
   the verdict, exact one-dimensional W1, and a checker for sums a consumer emits.
5. **The twin's inventory** (its lead's answer of 2026-11-11). Real reference data: people
   and one land-cover class per cell, to 1/8 degree. Below that, and for every relationship
   at every tier, nothing real. No street object, no intersection graph, no stored link from
   a building to its street. Device counts are per-person constants, the same everywhere.
   The exact chain holds from people to households and stops there.
6. **The physical tower** (`TIERS.md`): six tiers measured bottom-up, the fluid element in
   the engine, the nucleus a declared input, the fold below the atom measured in 1+1D only,
   the cosmological rung a wager. Nothing links the device to the continuum.

## §2 What the world model is

A **world model**, in this FSD, is four things together and nothing less:

| part | what it is | exact or statistical | status |
|---|---|---|---|
| **The ladder** | a tree of holons, universe to vacuum; each generated from its parent's population, class and seed | exact (section law, edge law) | **[today]** as theory and as `holon-scene`; no physical rung below the device is generated yet |
| **The references** | per kind of object and per class, a sample of reality held as statistics | — | **[proposed]**; STREETREF-1 is the first, drafted and held |
| **The generators** | one per kind of object, fitted to its reference on a training split | statistical | **[proposed]** |
| **The certificates** | per (generator, observable, consumer, class): certified, refused or unresolved | both | the order and the arithmetic **[today]**; no certificate issued yet |

and, joined to it from below, **the tiers**: each rung's dynamics certified against the finer
motion. The scene comes from above, the motion from below, and `meets_tier` is where they join.

**What a consumer sees.** One call:

```
children(holon id, population, class, seed)
  -> for each child: id, population, attributes
     the relationships among the children, and the ports to the neighbours
     for each declared observable: certified | refused | unresolved, with the reason
```

A consumer supplies its root populations and declares which observables its own model
reads. It does not hold a dataset, fit a generator or compute a certificate.

## §3 The rules of the model

1. **Population first.** Geometry, attributes and relationships are consequences of a
   holon's population and class. Nothing is placed that the population does not pay for.
2. **Exact where it can be.** Counts are integers and children sum to parents for every
   seed. Kinds are apportioned in order, so people follow households and devices follow
   people. **[today]** in `holon-scene::tree`.
3. **Relationships are owned by the lowest holon holding both ends,** handed down as ports,
   and summed back up. A child depends on its parent's reading, its seed and its ports, never
   on a sibling's interior. **[today]** as law; **[proposed]** as a generator.
4. **A certificate is owed only for what a consumer reads.** Everything else is decoration
   and carries a label. Budgets belong to an (observable, consumer) pair.
5. **Three verdicts, in order.** A scene that does not aggregate is refused. Outside the
   envelope: refused by name. Inside but below the resolution floor: unresolved. Only then is
   the match consulted. **[today]** in `holon-scene::verdict`.
6. **A reference is frozen before any generator is fitted to it,** its places drawn by a
   declared rule, its licence read and recorded, and only statistics kept.
7. **The reference is a sample.** A certificate is against the sample, and says so.
8. **Random access and determinism.** Any holon from its ancestors' splits alone, without
   generating any sibling's contents; the same holon from the same seed on every platform. Integers only makes the second hold by construction.

## §4 The ladder, rung by rung

| rung | population | generated or measured | reference | joins below by |
|---|---|---|---|---|
| universe | — | a wager (`TIERS.md` §4) | none | — |
| planet, region, cell | people; land class | real to 1/8 degree | GHS-POP, land cover | — |
| district, street | households | generated | STREETREF-1, held | — |
| building | households | generated | none yet | — |
| household | people, devices | generated | none yet | — |
| device | kind, links | generated | none yet | **nothing: the open join** |
| continuum, fluid element | mass, momentum; fluxes on faces | measured; in the engine | the fine walks | `Tier.stack` |
| molecule | the rigid unit | measured | all-atom walks | `Tier.stack` |
| atom, chemistry | derived force law | measured | full-CI cores | the seam |
| nucleus | Z, mass, spin | declared | none | — |
| the fold, vacuum | conserved charges; zero at the floor | measured in 1+1D | — | `Vacuum.lean` |

**The open join.** Above the device the model generates and has no dynamics certified;
below the continuum it has dynamics certified and generates nothing. A device is made of
matter the lower rungs describe, and no rung says so. Closing it is a rung of its own
(materials under a building or a device), not a consequence of anything here.

## §5 What is built next **[proposed]**

Each item names its producer, its consumer and its check.

| # | item | producer | consumer | check |
|---|---|---|---|---|
| 1 | The reference store: per dataset a manifest (source, licence sentence, digest, date), a reducer, a statistics table keyed by class | CIRISHolon `conformance/scene/` | every generator | one freeze per reference; the audit; raw files never committed |
| 2 | STREETREF-1 frozen and fetched | CIRISHolon | the street generator; the twin's page | its own stakes; held for the twin's final class rules |
| 3 | A graph generator: children plus the relationships among them, with the edge law as a tested property | `holon-scene` | the twin's street and mesh tiers | `sums::check` on crossings from both sides; the wiring fence as a plant |
| 4 | Generators fitted per kind (street, building, household, device) | `holon-scene` | the twin | a scene certificate each, per consumer |
| 5 | The twin calls the model instead of its own layout | the twin | its mesh simulation and page | the twin's summing test emits lines; `check_sums` exits 0 |
| 6 | SCENE-1 on water: the same certificate where the fine tier exists | CIRISHolon | the fluid element | drafted, on hold |
| 7 | The open join named as a rung | the lead | — | a freeze, when a target is chosen |

## §6 Non-goals

- **Not a learned model of dynamics.** Nothing here predicts a next frame from pixels or
  plans in a latent space. Dynamics enter only through tiers certified elsewhere in this repo.
- **Not a map.** No real road, building or home is drawn or stored; references are statistics.
- **Not a claim about any real place.** A certified scene is statistically like the places
  its reference sampled, for the observables named, and nothing more.
- **Not complete.** `unresolved` is a first-class answer, and most of the ladder returns it today.

## §7 What the model cannot do, stated against its own theorems

- A section is not a match; a count is not a wiring; marginals are not a joint.
- Closed is not carried: a sector can obey every law here and be something the tier above
  does not read.
- Population is the diagonal. Where coherence begins a scene is not a function of its
  populations (`Object.diag_not_closed_under_coherence`); below that line the ladder carries
  conserved charges only.
- No law is shared between rungs. What repeats is the procedure.

## §8 Position against the literature **[today]**

*From a prior-art screen run on 2026-11-11 against primary pages where they could be read.
House rule: convergent art is a hit, not a strike. "Not found" below means not found in that
screen, and is not a claim of novelty. Citations the screen confirmed only through secondary
sources are marked (2ry) and are to be read before they are cited anywhere else.*

**Is this a "world model"?** Not in the machine-learning literature's sense. There the term
means a LEARNED predictive model of how an environment responds over time, usually
conditioned on actions and used for planning or imagined rollouts (Ha & Schmidhuber 2018,
arXiv:1803.10122; Hafner et al., Dreamer, arXiv:1912.01603; Bruce et al., Genie,
arXiv:2402.15391; the survey of Ding et al. 2024, arXiv:2411.14499). This model learns no
dynamics and predicts under no action. The name is kept in this repository in the sense §2
defines, and outside it the accurate description is longer: a certified, multiscale,
mass-preserving generator of a synthetic world, over a hierarchy of lumpable abstractions
for its dynamics. Calling it a world model in public would invite comparison on axes where
it does not compete and hide the ones where it does.

**Part by part.**

| part of the model | status | occupier, credited |
|---|---|---|
| counts by kind per spatial unit | **OCCUPIED** | census histograms and synthetic populations (SPEW, Gallagher et al. 2017, arXiv:1701.02383; PopGen and iterative proportional updating) |
| children sum exactly to the parent | **OCCUPIED — cite, never claim** | Tobler's pycnophylactic property (JASA 1979); the US Census TopDown Algorithm, which is integer, non-negative, exact, hierarchical and top-down (Abowd et al. 2022, arXiv:2204.08986) and is the strongest single precedent; hard-constrained downscaling (Harder et al., JMLR 24, 2023, arXiv:2208.05424); microcanonical cascades, exact for every draw; the multiscale Poisson and multinomial factorisation (Kolaczyk & Nowak, Ann. Statist. 2004, arXiv:math/0406424), which is the statistical home of "child given parent" |
| the coarse edge is the sum of the fine edges, top-down | **OCCUPIED** | HiGen (Karami, ICLR 2024, arXiv:2305.19337), including the parent's self-loop as the internal total; ownership by the lowest common ancestor is Clauset, Moore & Newman (Nature 2008, arXiv:0811.0484); the nested block model (Peixoto 2014, arXiv:1310.4377) |
| seeded random access | **OCCUPIED as a capability** | counter-based generators (Salmon et al. 2011); a learned generator claiming seed consistency and constant-time access (Terrain Diffusion, Goslin 2025, arXiv:2512.08309) |
| the commuting square, exact and within a budget | **OCCUPIED, textbook** | lumpability (Kemeny & Snell, 2ry); bisimulation and MDP homomorphisms (Givan, Dean & Greig 2003; Ravindran & Barto; Li, Walsh & Littman 2006); approximate bisimulation (Girard & Pappas 2007); causal abstraction (Rubenstein et al. 2017, arXiv:1707.00819; Beckers & Halpern 2019) |
| the certificate's ingredients | **OCCUPIED, one by one** | one-dimensional Wasserstein as a validation metric is the area metric (Ferson, Oberkampf & Ginzburg 2008); budgets set by context of use are ASME V&V 40; a declared envelope is the operational design domain (ISO 34503); three-valued outcomes exist in equivalence testing and in metrology |
| street-network statistics | **OCCUPIED** | Boeing 2019 (arXiv:1808.00600) for the observables; Neural Turtle Graphics (Chu et al., ICCV 2019, arXiv:1910.02055) for scoring generated networks against city statistics, with no validity domain |
| exact integer sums for every seed TOGETHER with random access down a deep tree | **not found** | nearest: microcanonical cascades; top-down iterative proportional updating |
| the edge law as the population law on pairs, with the two-parent condition | **not found** | — |
| the certificate as ONE object: per-observable and per-consumer budgets, a class envelope, and `unresolved` when the reference holds too few independent samples | **not found** | nearest: an unreviewed preprint with an "inconclusive" outcome for empty subgroups |
| a generated scene under a certified coarse view moves as its parent does, stacked across tiers | **not found; likely near exact lumpability, which the screen did not read** | read before any claim |
| machine-checked proofs about a multiscale generator | **not found, in any prover** | nearest: Isabelle's Markov-chain and bisimulation libraries; an experimental Lean package with a lumpability module |
| the whole assembly | **not found** | — |

**What this stance claims, at that strength.** No part. The assembly as one instrument,
with its exact half proved and its verdicts typed, and only until someone shows it elsewhere.

**What the literature says against the model's assumptions. Each is adopted as a risk.**

1. **Exactness passes the parent's error down.** Harder et al. report that a hard constraint
   hurts where the coarse input is itself biased. Here every child inherits its cell's
   population error exactly, and the twin's 1/8 degree cells carry up to about 7 % coding
   error. *Adopted:* the parent's own error is a budget term in every certificate under it.
2. **Exact non-negative integer apportionment is biased at small counts** (the Census
   TopDown literature). *Adopted:* the apportionment's bias at counts under ten is a named
   observable, with a plant.
3. **Conserving at every branch bounds the extremes** and under-represents autocorrelation
   (Serinaldi 2010; Müller-Thomy 2020, on cascades). *Adopted:* tail quantiles and a spatial
   autocorrelation reading are in every scene certificate, not only the bulk.
4. **"Without generating its siblings" needs saying carefully.** An exact sum fixes the whole
   sibling SPLIT at the parent. What is not generated is the siblings' contents. Rule 8 of §3
   is to be read that way.
5. **Orientation entropy is not enough for streets.** Barthelemy & Boeing (2025,
   arXiv:2509.21931) argue most street models fail on the heterogeneity of betweenness.
   *Adopted:* a betweenness reading is owed in STREETREF-1's observables before it is frozen.
6. **A commuting square checked on a limited set of views can hide differences** (Beckers &
   Halpern). This is the record's own closed-is-not-carried finding from another side.

**Vocabulary taken from the literature, with credit:** "context of use" and "applicability"
(V&V 40) for per-consumer budgets and the envelope; equivalence testing with a margin fixed
before the data for the three verdicts; value equivalence (Grimm et al. 2020,
arXiv:2011.03506) for certification relative to a named set of consumers; the lowest-pedigree
input rule (NASA-STD-7009) for the reference behind a certificate. A zero distance is "no
evidence of disagreement", never "agreement" (Ferson & Oberkampf).

## §9 Acceptance

The world model is accepted, rung by rung, when for that rung:

1. the section law holds on emitted sums for every seed tested (`check_sums` exits 0);
2. a reference is frozen, fetched and banked with its licence reading;
3. a generator's certificate is issued for every observable a consumer declares, and each
   verdict is one of the three with its reason;
4. at least one plant per certificate fires: real against real certifies, and a generator
   that matches every marginal and breaks the joint is refused;
5. the consumer's own result is reported beside the verdicts of the scene it ran on.

No rung is accepted on the exact half alone.
