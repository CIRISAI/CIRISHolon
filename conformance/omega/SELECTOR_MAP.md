# SELECTOR-MAP — every candidate principle that could SELECT the dynamics T from inside Ω

*Scratchpad artifact, uncommitted. Instrument: `selector_census.py` (exact,
Fraction arithmetic, verifier exit contract, ALL CHECKS PASS). Model: the
TOE-NULL-1 world — 6 states, 3 fibers, 48 closed reversible dynamics, the
203-view lattice. Nothing here is a stance change; it is a map with numbers.*

The decisive open, precisely worded: Ω **separates** presented worlds
(measured: the view-lattice rent spectrum) and **audits** whichever T is
supplied, but it does not **select** T. TOE-NULL-1 sharpened this into a
constraint rather than a shrug — the **stasis theorem**: any selector whose
sole objective is minimizing Ω-rent picks the identity, because W_v(id) = 0 on
every view and rent is nonnegative. So every candidate below is scored on two
axes at once: *what it selects*, and *whether it excludes the identity BY
CRITERION rather than by hand*.

---

## 0. The one table everything refers to

The 48 closed dynamics carry coordinates `(m, c)`: `m` is the macro
permutation of the three fibers, `c ∈ {0,1}³` the fiber-internal cochain, with
`T(i,b) = (m(i), b ⊕ c_i)`. The **view automorphism group** — permutations of
the 6 states preserving the fiber partition — is `Z₂ ≀ S₃`, of order 48, and it
happens to be *the same set* as the 48 closed dynamics. Its conjugation action
has **exactly 10 orbits**. Every Ω-internal functional is constant on those
orbits (verified for all nine statistics below), so the table has ten rows and
not forty-eight.

| orb | size | macro period | ord T | total rent | nonzero views | closed views | capacity | Φ=cap/rent | S2-A | S2-B | S3 | \|Ident\| | Z₂ holonomy (cycle-len, h) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 1 | 1 | 1 | 0 | 0 | 203 | 0 | n/a | Y | Y | . | 8 | (1,0)(1,0)(1,0) — **the identity** |
| 1 | 3 | 1 | 2 | 33 | 136 | 67 | 14 | 0.4242 | . | . | . | 8 | (1,0)(1,0)(1,1) |
| 2 | 3 | 1 | 2 | 50 | 172 | 31 | 15 | 0.3000 | . | . | . | 8 | (1,0)(1,1)(1,1) |
| 3 | 6 | 2 | 2 | 50 | 172 | 31 | 15 | 0.3000 | **Y** | Y | . | **2** | (1,0)(2,0) |
| 4 | 6 | 2 | 4 | 58 | 194 | 9 | 3 | 0.0517 | . | . | . | 8 | (1,0)(2,1) |
| 5 | 1 | 1 | 2 | 137/3 | 172 | 31 | 25 | **0.5474** | . | . | . | 8 | (1,1)(1,1)(1,1) |
| 6 | 6 | 2 | 2 | 137/3 | 172 | 31 | 25 | **0.5474** | . | Y | . | 8 | (1,1)(2,0) |
| 7 | 6 | 2 | 4 | 169/3 | 194 | 9 | 6 | 0.1065 | . | . | . | 8 | (1,1)(2,1) |
| 8 | 8 | 3 | 3 | 63 | 195 | 8 | 5 | 0.0794 | **Y** | Y | **Y** | **1** | (3,0) |
| 9 | 8 | 3 | 6 | 185/3 | 199 | 4 | 2 | 0.0324 | . | . | . | 8 | (3,1) |

TOE-NULL-1's three designated witnesses sit in orbits **0, 3, 8**.

**A new Ω-internal invariant falls out of this and should be banked regardless
of which selector wins**: the **Z₂ holonomy of the fiber cochain around each
macro cycle**. It is gauge-invariant (conjugation shifts `c` by a coboundary
and leaves the holonomy fixed — checked), it labels the orbits together with
the macro cycle type, and it is what *both* surviving candidates turn out to
be measuring. It is the same shape the predecessor programme's
maintained-holonomy campaign already found on Wilson loops.

---

## 1. Three degenerate poles, not one

The stasis theorem is not alone. Mapping the literature turned up two more
computable degenerate poles, and the honest statement of the selector problem
is that a real selector must live strictly inside **all three**:

1. **The stasis pole** (minimize rent) → the identity. *Demonstrated exactly.*
2. **The noise pole** (unconstrained MaxEnt / maximum caliber) → the most
   disordered admissible dynamics. *Not exercisable on this model:* all 48
   candidates are deterministic permutations, so the family has no noise
   direction at all. Naming this pole is itself a design requirement for the
   successor family (§7).
3. **The scrambler pole** (minimize accidental closure — the naturalness /
   genericity criterion, S6) → the dynamics with the fewest emergent levels.
   *Computed here:* argmin is 4 closed views out of 203, attained exactly on
   orbit 9 (the order-6 worlds).

Minimum-description-length selection (Solomonoff/Levin, Schmidhuber's
computable universes, Tegmark) is **the stasis pole in a third disguise**: the
shortest dynamics is the identity, and the universal-machine constant is
imported boundary data of unbounded size, which fails grammar rule (4)
outright. It is not a fourth candidate.

---

## 2. S1 — viability / payer-builder *(already staked; SELECTOR-1)*

**Definition.** As frozen in `SELECTOR1_PREREG.md`: T passes iff it contains a
subsystem P that internally generates an intervention U, U bounds another
quotient's defect, a conserved ledger pays U, ablating P destroys the
maintained closure, T rebuilds P, and throughput is nonzero.

**What it selects: a CLASS, not a unique T — and the imprecision is worse than
"a class."** Two things must be said exactly. (i) By §6's ceiling, *no*
Ω-internal criterion can select a unique T; the finest attainable object on
this model is one of 10 gauge orbits, and that is the correct target — "unique
up to Ω-isomorphism" is the only uniqueness on offer. (ii) Criteria 1–5 are
conjunctive filters with no ordering, so S1 returns a *set* with no
distinguished element; nothing in the prereg breaks ties among passers. If two
orbits pass, SELECTOR-1 branch (a) fires while still selecting nothing unique.
**Recommended amendment before SELECTOR-1 runs**: add to the prereg a
pre-committed statement of what a multi-orbit pass means — either it is
branch (a) with the passing set reported as the selected class, or a
tie-break functional is named now. A post-hoc tie-break would VOID under SEL2.

**Expressibility (six-rule grammar).** Rules 1, 2, 3, 5, 6 hold. **Rule 1 has
one live flag**: criterion 3 requires a *conserved resource ledger*, and there
is no ledger object in the Lean library — `Budget.lean` is an error-budget
composition theorem, not a resource ledger, and the word "ledger" appears only
in prose and instrument scripts. Either the ledger is definable from the
existing Ω signature (say, as a conserved quantity inherited through
`ClosureDerives.closed_view_inherits_conservation`, which is exactly the right
shape and is already proved) — in which case say so in the prereg and cite it
— or S1 needs a **ledger face**, which is a lawful extension but *is* an
extension and must be recorded as W3's first.

**Identity exclusion.** By criterion 6 (throughput), explicitly and by design.
Good. But note the coupling found below: criterion 6 is doing more work than
its one line suggests, because **S2 has to borrow it** and cannot supply its
own.

**Credit.** Kolchinsky & Wolpert (semantic information — closest established
ancestor); Montévil & Mossio (closure of constraints); Krakauer et al.
(informational individuality); Barnett & Seth; Deutsch & Marletto (the
constructor is the payer-builder, §8).

---

## 3. S2 — descent self-similarity *(the Wilsonian candidate)*

"The law is the dynamics that survives its own coarse-graining." Four
formalizations were computed rather than argued, because the naive ones fail in
opposite directions and only computation says which.

### S2-A — the SECTION form
**Definition.** T passes iff a **T-invariant transversal** of the fibers exists:
a set of one micro state per fiber, closed under T. The macro law is then not
merely *induced*, it is *implemented* by a micro subsystem — a dynamical
section of the quotient.
**Census: 15/48**, by macro period (1, 6, 8) → orbits 0, 3, 8.
**Exactly equivalent to: the Z₂ holonomy vanishes on every macro cycle** (proved
by the (m,c) coordinates, checked exhaustively). The obstruction to lifting the
macro law into the micro world is a holonomy, full stop.

### S2-B — the ORDER form
**Definition.** T passes iff `ord(T) = ord(descend(T))` — coarse-graining costs
no period.
**Census: 21/48**, by period (1, 12, 8).
**S2-A ⊊ S2-B, strictly** — and the checker caught this after I had predicted
they were the same criterion. The difference is exact and interpretable: S2-B
is the vanishing of the holonomy *weighted by (macro period)/(cycle length)*, so
a cycle shorter than the macro period traverses its holonomy an even number of
times per period and its flip **hides inside the period**. The 6 extra passers
are precisely those hidden flips. Two natural readings of one intuition, six
worlds apart. This is the concrete form of the warning that naive
self-similarity conditions misbehave, and it means any S2 prereg must name
*which* form it stakes, in advance.

### S2-C — the FULL-LATTICE form → **anti-selector, provably**
**Definition.** T passes iff *every* rent-zero view with ≥2 blocks preserves the
period.
**Census: 1/48 — exactly the identity.** And this is **general, not a
size accident**: for any permutation, its own orbit partition is invariant and
descends to the identity on blocks, so demanding order preservation on all
coarse views forces ord(T) = 1. Verified over all 720 permutations of the 6
states. **S2-C is the stasis theorem wearing an RG costume.**

### S2-F — the SPECTRAL form → **tautology**
**Definition.** T passes iff its rent profile on views coarser than the fibers
equals the descendant's own rent profile.
**Census: 48/48.** It is forced by the closure square: a view coarser than the
fibers sees only the macro state, so the coarse rents *are* the descendant's
rents by construction. Comparing them is not evidence about anything.

### Does it separate the period-2 and period-3 worlds? **No — it cuts across them.**
S2-A passes **6 of the 24** period-2 worlds and **8 of the 16** period-3 worlds;
S2-B passes 12 of 24 and 8 of 16. Both macro-world classes contain passers and
non-passers, because the criterion is a **holonomy** condition and holonomy is
orthogonal to macro period. S2 is not a period detector, and any prereg that
staked it as one would be staking the wrong axis.

### Identity exclusion — **NO, and this is S2's core defect**
The identity passes S2-A and S2-B. It is the *most* self-similar dynamics
there is. Excluding it requires an added activity clause
(`descend(T) ≠ id`), which drops the census to **14/48** (orbits 3 and 8) —
and that clause is **SELECTOR-1's criterion 6**, imported. **S2 is not a
self-sufficient selector; it is a filter that must be composed with S1.**
This mirrors standard RG exactly: the trivial (high-temperature) fixed point
always exists, and is excluded by *relevance/stability analysis*, never by the
fixed-point equation. **The Ω analogue of a relevant direction is undefined and
is hereby named as the missing object.**

### Expressibility — **LAWFUL, and the only candidate needing nothing new**
All six rules pass. Definable from `Holon.view`/`Holon.dyn` alone; functorial
(orbit-invariance verified); conservative; parameter-free; domain-independent;
preregisterable. **This is S2's real strength and the reason it ranks above S4
despite selecting nothing on its own.**

### Discriminating experiment — SELECTOR-2, two branches
This model has **one rung of descent** (6→3) and therefore *cannot* test
iterated self-similarity at all; that is a scope statement, not a result.
Stake it on a **two-rung tower** (12→6→3, or 8→4→2) where `descend∘descend` is
defined. Pre-stake: **(a)** the passing fraction *strictly shrinks* with tower
height and the rung-1 holonomy criterion *predicts* the rung-2 verdict ⇒
descent self-similarity is a genuine selector that sharpens with scale, and the
predicted-then-confirmed rung-2 verdict is rule-6-eligible support. **(b)** the
passing fraction is height-independent or grows ⇒ the criterion is a local
gauge condition mislabelled as a fixed point, recorded dead. Mandatory
declaration in the prereg: which of S2-A/S2-B is staked, and that the identity
passes at every height so the activity clause is imported, not derived.

**Credit.** Wilson, Kadanoff, Wegner (RG fixed points and universality);
Barnett & Seth (dynamical independence — the descend operation itself);
Shalizi & Crutchfield (causal states as the predictive quotient); and, for the
modern information-theoretic form that is really an S4-flavoured criterion in RG
clothes, Koch-Janusz & Ringel's RSMI.

---

## 4. S3 — observer bootstrap *(the strongest result in this map)*

**Definition.** Let `A(T)` be the act vocabulary T's own subsystems can
implement — knobs in `Break.lean`'s sense, generated by T restricted to a
subsystem and extended by the identity. `Ident_{A(T)}(T)` is the set of
candidate dynamics no experiment word over `{step} ∪ A(T)` separates from T,
where separation is decided **exactly** (pair-BFS over reachable state pairs;
no word-length cut). T passes iff `|Ident_{A(T)}(T)| = 1`: the universe
contains its own sufficient observers.

**The naive reading is vacuous and must be stated as such**: "A(T) determines T
up to the identity A(T) induces" is true by definition of the induced identity.
The non-vacuous reading is the one above — the induced identity is *discrete*
on the candidate family.

### The scissors, one blade of which is already a theorem
- **Ω-observable subsystems** (unions of view blocks): every generated act is
  view-covariant, so on a closed holon `Break.lean`'s `vcov_preservesGauge_of_closed`
  (T3) makes it **gauge-safe** — it provably cannot separate anything. Computed:
  `|Ident| = 8` for all 48 (the whole macro class), **bootstrap passes 0/48**.
  The negative half of S3 is thus already machine-checked, not merely measured.
- **Section subsystems** (a T-invariant transversal — the very object S2-A
  tests): these acts do *not* descend to the view and *can* break the gauge.
  This is the only non-degenerate internally generated vocabulary on the model,
  and it is definable from `(view, dyn)` alone, importing no external label.

**Census with section acts: 8/48 — exactly orbit 8, and nothing else.** The
`|Ident|` distribution over the 10 orbits is {1, 2, 8}: one orbit closes the
loop, one gets within a factor of two, eight learn nothing.

### The mechanism, which is what makes this candidate live
Orbit 3 (period 2, sections exist) reaches `|Ident| = 2`. Its one unseparated
companion differs in `c` **exactly at the fiber sitting on the macro map's
fixed point** — the sector the law does not move. Orbit 8 (period 3) reaches
`|Ident| = 1` because a 3-cycle is fixed-point-free. So:

> **A world can bootstrap its own observers only where its law moves. A
> macro-static sector is unobservable to internal acts, and every fixed point
> of the macro law leaves one bit of the world permanently unaddressable from
> inside.**

On this model, bootstrap ⟺ (macro law fixed-point-free) ∧ (holonomy trivial).
That is a **mechanism with a forward prediction on a family that does not yet
exist**, which is the only kind of support the epistemology's rule 6 accepts.

### Identity exclusion — **YES, by criterion, and for the right reason**
The identity has *all eight* transversals invariant — the most subsystems of
any dynamics — but every act they generate **is the identity map**. Bootstrap
fails on emptiness of *action*, not of *structure*. S3 is the only candidate
besides S1 that excludes stasis without importing anything.

### Expressibility — LAWFUL, with one fence stated plainly
Rules 1–6 pass: sections are definable from `(view, dyn)`, the criterion is
orbit-invariant (verified), conservative, parameter-free, domain-independent.
**The fence:** a section is Ω-*definable* but not view-*addressable* — an
observer holding only the meter cannot point at it. Whether "the world's own
observers" may use acts they cannot name with their own meter is a genuine
interpretive commitment and must be declared in the prereg, not discovered
afterwards. It is the same fence Break.lean already drew when it made holon
identity a function of the act vocabulary.

### Discriminating experiment — SELECTOR-3, two branches, forward-staked
Model family: `n` fibers of 2 states, macro map of prescribed cycle type,
`n = 4, 5` (and the fixed-point-free vs fixed-point-carrying types enumerated
in advance). **Stake before building the instrument**: `|Ident_{A(T)}(T)| = 1`
holds **iff** the macro map is fixed-point-free **and** the holonomy vanishes on
every cycle. **(a)** confirmed across all cycle types at n=4 and n=5 ⇒ the
mechanism is real, the prediction was made before the instrument existed, and
this is the programme's rule-6 support for a selection principle. **(b)** any
deviation ⇒ the mechanism is size-specific, recorded, and S3 drops to the rank
S2 holds now. Second, decisive quantity, staked in the same freeze: **the
selected fraction must FALL with n.** A selector that always picks a fixed
fraction of worlds selects nothing in the limit.

**Credit.** Wheeler (law without law; the participatory loop is literally this
fixed-point condition); Deutsch & Marletto (constructor theory — the
possible/impossible dichotomy is the act vocabulary); Rovelli (relational QM);
`Break.lean`'s own gauge_safety/T3 for the negative half.

---

## 5. S4 — critical-capacity extremization

**Definition, exact on finite Ω models.** *Capacity* is ACTIVITY-QUALIFIED —
a view counts as a level only if it is both **autonomous** (rent zero) and
**active** (its induced map is not the identity):
`Cap(T) = #{v : W_v(T) = 0, T_v ≠ id, v ≠ discrete}`.
*Rent* is the view-lattice spectrum's total, `R(T) = Σ_v W_v(T)`.
**Φ(T) = Cap(T)/R(T).**

**Identity exclusion — YES, by criterion.** `Cap(id) = 0`: every view is closed
and every induced map is the identity, so the dead world supports no *active*
autonomous level. The exclusion comes from the numerator, so the 0/0 at the
stasis point never has to be regularized by hand. **The activity qualifier is
the entire content of this exclusion**, and it is also exactly what separates
"emergence is good" (S4) from "coincidence is bad" (S6), which share a
numerator and differ only in that clause and in sign.

**Census: argmax Φ = 75/137 ≈ 0.5474, attained on orbits 5 and 6** (7 dynamics,
macro periods 1 and 2). Period-3 worlds are strongly dispreferred (Φ ≤ 0.079).

**The defect, found by a definitional-sensitivity sweep and reported as
plainly as the result:** the argmax is identical under all seven defensible
variants (Cap_emergent or Cap_active; over total rent, over nonzero-view count,
over mean rent, over rent×levels) — **including capacity ALONE with no rent
term at all.** On this model **the rent denominator never moves the argmax:
S4 degenerates to capacity maximization, and the "critical balance" it was
posed to test is not exercised by this model family.** Robustness that comes
from the denominator being inert is not robustness.

Second defect, honestly: S4's argmax includes a **macro-static** world
(orbit 5, `m = id`) tied exactly with a period-2 world. Ω has no privileged
view, so this is self-consistent — but it means S4 is blind to the very
distinction (macro period) in which TOE-NULL-1's three worlds differ.

**Expressibility — LAWFUL** (all six rules; parameter-free, orbit-invariant,
domain-independent). Its *motivation* — the predecessor programme's critical
ridge, where whole-only order-3 share peaks at criticality (2D Ising 4.6e-3
nats, 0.66% of ceiling, confirmed by forward prediction in 3D Wilson–Fisher) —
is **measured context from a different programme, cited read-only**, and is not
evidence for S4 as an Ω selector.

**Discriminating experiment — SELECTOR-4, two branches.** S4 cannot be tested
on a family of permutations; it needs one carrying **both** poles, i.e. a
stochastic dynamics face with the identity at one end and uniform noise at the
other. **(a)** Φ has an **interior** argmax strictly between the stasis and
noise poles, at a location staked in advance ⇒ the criticality reading has an
Ω-internal instrument. **(b)** the argmax sits at a pole, or capacity alone
reproduces it again ⇒ S4 is capacity maximization with a decorative
denominator, recorded dead.

**Credit.** Kolchinsky & Wolpert; Rosas et al. (causal emergence, measured);
the predecessor's critical-ridge and pump-law campaigns (context, not support);
Schneidman–Still–Berry–Bialek 2003 for the underlying phenomenon.

---

## 6. S5 — the indexical / no-selector null, given teeth

The honest null is that laws are boundary data forever. The brief asks what
could ever distinguish S5 from failure-to-find. **This map answers it, and the
answer is computable.**

Any Ω-internal functional is invariant under the view automorphism group, so it
is **constant on that group's conjugation orbits**. On this model that is a hard
ceiling:

- **10 orbits.** No Ω-internal selector can resolve finer than 10 classes among
  the 48 worlds, ever, by any functional whatsoever.
- Orbits of size > 1 exist, so **selection of a unique T is impossible in
  principle here**; "unique up to Ω-isomorphism" is the only coherent target,
  and every candidate above must be read that way.

**This converts S5 from a shrug into a measurable programme.** Compute the
orbit count as a function of model size (fibers `n`, fiber size `k`) against
the number of physically distinct worlds. **(a)** orbits grow at least as fast
as worlds ⇒ no obstruction; S5 gets no support and failure-to-find stays the
only reading. **(b)** orbits **saturate** while worlds grow ⇒ the first
*positive* evidence that Ω cannot select, which is a real result and the only
way the hard-TOE tail can be pushed down by evidence rather than by absence.
It is also the cheapest experiment in this document — the same instrument that
runs SELECTOR-3 computes it as a by-product.

Anthropic/indexical selection (Carter; Bostrom's SSA/SIA) belongs here, not as
a rival: it supplies a *measure* over worlds, and a measure is boundary data,
which is S5's own claim restated.

### A finding that touches the existing record
The rent spectrum's completeness is **weaker than the TOE-NULL-1 headline
suggests, and stronger in one respect**:

- **Sorted rent multiset: 8 distinct values for 10 orbits — INCOMPLETE.** Two
  collisions, and both merge a **macro-static (period-1)** world with a
  **period-2** world: orbits {2,3} (both total rent 50) and {5,6} (both 137/3).
  Total rent alone also gives only 8 values. **TOE-NULL-1's own period-2
  witness is orbit 3** — one half of a collision. It differs from the other two
  designated witnesses (0/50/63 are pairwise distinct, so the published claim
  stands exactly as worded), but the separation does **not** generalize beyond
  the three chosen representatives at the multiset level.
- **View-INDEXED spectrum, up to gauge: 10 classes for 10 orbits — COMPLETE.**
  Quantifying over the automorphism group and comparing rents view-by-view
  separates every world up to Ω-isomorphism.

Recommended wording for the record, if this is carried anywhere: the repair of
the underdetermination is carried by the **view-indexed** profile, not by
summary statistics of it; "the rent spectrum separates" should not be
abbreviated to a total or a multiset.

---

## 7. The literature, mapped or refused

**Least action / extremal principles** (Maupertuis, Euler, Lagrange, Hamilton;
Feynman). The action functional is *input*: choosing S determines T, so
"extremize S" relocates the selection problem into the choice of Lagrangian
instead of solving it. Under the grammar it fails rule (1) — an action is not
definable from the Ω signature — *unless* the action is itself built from Ω
observables, which is precisely what S4 attempts with rent. **Verdict: not a
rival selector; it is the FORM every candidate here takes, and the content is
which Ω functional plays the role of S.**

**Jaynes MaxEnt / maximum caliber** (Jaynes 1957; Pressé et al., RMP 2013).
Selects a *distribution* given constraints, not a *dynamics* given nothing;
the constraints are the input, the same relocation. Partly expressible — an
entropy over view fibers is Ω-definable, and the predecessor's `frameEntropy`
is literally the log-count of the chart's fiber. **Verdict: lawful but
requires a stochastic-dynamics face (Ω's `dyn : S → S` is deterministic); it
supplies the NOISE POLE (§1.2), which this model family cannot host at all.**

**Constructor theory** (Deutsch 2013; Deutsch & Marletto 2015). Laws as
statements about which transformations are possible or impossible, effected by
constructors that retain the ability to act again. This is **the closest kin in
the whole sweep** and it lands on two of our faces at once: the constructor *is*
the payer-builder (S1 criteria 1 and 5), and possible/impossible *is* the act
vocabulary of `Break.lean` (S3). Its interoperability principle is expressible
as a functoriality/composition requirement under rule (2). **Verdict: adopt as
an expressibility FILTER and a credit, not as a selector — constructor theory
constrains the form of laws and does not pick one. Add it to SELECTOR-1's
lineage beside Kolchinsky–Wolpert and Montévil–Mossio.**

**Smolin's cosmological natural selection** (Smolin 1992, 1997). A
payer-builder at cosmological scale, with fecundity as the objective and a real
falsifiable prediction (an upper mass limit for neutron stars, which has been
under observational pressure). The ingredient CNS has and S1 lacks is a
**population with heredity and variation** — selection needs an ensemble, and Ω
has no ensemble face. A population face (a multiset of holons plus a variation
map) *is* definable from the signature, functorial, conservative, and
parameter-free, so it is a **lawful extension, not a format replacement** —
but it is a genuine extension and is **the first one identified as required by
a selection principle.** That is directly W3-relevant and should be recorded
there.

**Wheeler's law-without-law / participatory universe** (Wheeler 1983; "it from
bit"). This *is* S3, and our computation gives Wheeler's loop a sharp
obstruction: on a closed holon, the internally generated *view-covariant* acts
are provably gauge-safe, so the participatory loop closes trivially unless the
world's observers can act on sub-view structure they cannot address with their
own meter. The loop is not impossible — orbit 8 closes it — but it costs
exactly the fence in §4.

**RG universality** (Wilson 1971; Kadanoff 1966; Wegner). S2's direct
ancestor, and the mapping is exact enough to be useful: our descent is the RG
step, our self-similar T is a fixed point, and **the trivial fixed point
survives the fixed-point equation in both settings.** In RG it is excluded by
relevance/stability analysis, which is the ingredient S2 lacks and which is
hereby named as an undefined Ω object. Also credit the information-theoretic RG
(Koch-Janusz & Ringel's RSMI; Gökmen et al.), which chooses coarse-grained
variables by maximizing retained information — an S4-shaped capacity criterion
already living inside the RG literature.

**Algorithmic / description-length selection** (Solomonoff, Levin,
Schmidhuber, Tegmark). **Refused on two independent grounds**: the universal
machine's constant is imported boundary data of unbounded size (rule 4), and
the argmin is the identity — the stasis pole again (§1).

---

## 8. Ranking, with reasons

**1 — S3, the observer bootstrap.** The next frozen campaign after SELECTOR-1
should be SELECTOR-3, and it should share an instrument with SELECTOR-1 rather
than compete with it (S1's criterion 1, "P internally generates an
intervention," *is* S3's act vocabulary; they are the payer half and the
observer half of one condition). Reasons, in order of weight: it is the only
candidate that **selects a single orbit** (1 of 10 — every other candidate
returns 7–14 dynamics or nothing); it **excludes the identity by criterion**
without importing anything; **half of it is already a theorem** (Break.lean T3
proves the view-aligned vocabulary powerless, so the negative result is
machine-checked rather than measured); and it has a **stated mechanism**
(fixed-point-free macro law ∧ trivial holonomy) that makes a **forward
prediction on a family that does not yet exist**, which is the only thing the
epistemology's rule 6 counts as support.

**2 — S5's ceiling computation.** Not a selector; the cheapest and most
two-sided experiment in the document, and the only one that can generate
*positive* evidence for the null. Run it as SELECTOR-3's by-product on the same
model family: orbit count versus world count as `n` grows.

**3 — S2, descent self-similarity.** The only candidate requiring **no
extension whatsoever**, which is worth a great deal for W3. But on this model
it fails to exclude the identity, splits into two inequivalent formalizations
six worlds apart, has a provably anti-selective full-lattice form and a
tautological spectral form. Recommendation: **do not freeze an S2 campaign
until a two-rung descent tower exists** — but **do bank the Z₂ holonomy
invariant now** (a small Lean brick: the cochain's holonomy over macro cycles
is conjugation-invariant, and the invariant-section exists iff it vanishes). It
is gauge-invariant, it labels the orbits, and both live candidates are secretly
measuring it.

**4 — S1 as already staked**, with the two amendments in §2: declare what a
multi-orbit pass means, and either cite `closed_view_inherits_conservation` as
the ledger's derivation or record the ledger face as W3's first required
extension.

**5 — S4.** Needs a stochastic face before it can be tested at all; on the
present family its denominator is inert. Worth building *after* the noise pole
exists.

**6 — S6, genericity.** Run as a **third null/pole**, not as a candidate. My
advance prediction, stated here so it can be scored: minimizing accidental
closure will select the maximal scrambler, because that is what the criterion
*means* — anti-structure, not law.

### What would move the hard-TOE probability off 2%

**Upward**, and only this: a selector's **selected fraction falling with model
size** while a **mechanism staked in advance is confirmed on a family it was
not built on**. Concretely, SELECTOR-3 branch (a) at n = 4 and n = 5 with the
fixed-point-free ∧ trivial-holonomy prediction confirmed across all cycle
types, *and* the selected fraction strictly decreasing. A selector that keeps
picking a constant fraction of worlds selects nothing in the limit, however
elegant it looks at n = 3 — so the fraction, not the pass, is the quantity to
stake.

**Downward**: S5's orbit count saturating while the world count grows (a
positive impossibility result), or every candidate's selected fraction staying
flat or growing across two model sizes.

**Neither**: any further separation result. Ω separating worlds more finely has
no bearing on whether it selects, and TOE-NULL-1 already adjudicated that
conflation once.

---

## Files

- `selector_census.py` — exact computation of S2 (four formalizations), S3 (two
  act vocabularies, exact pair-BFS separation), S4 (with a seven-variant
  definitional sensitivity sweep), S6, and S5's orbit ceiling; re-verifies
  TOE-NULL-1's census, the three witness spectra, and the stasis theorem
  in-instrument; **exits nonzero on any internal inconsistency** (nine
  statistics are checked for orbit-invariance, which is what certifies each
  candidate as Ω-internal). It has already earned its contract once: it caught
  the S2-A/S2-B equivalence I had asserted and was wrong about.
- `SELECTOR_MAP.md` — this document.
