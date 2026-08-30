# ELEMENTS-3 — results

*The record for `ELEMENTS3_PREREG.md` (frozen 2026-08-30, commit a0a6005) as amended by
AMENDMENT A1 (commit d409afb), A2 (d4fc687) and A3 (1b11ca4). Every gate below has a
verdict. Two of them are findings rather than passes and are written that way.*

---

## Scorecard

| gate | verdict | the number that decides it |
|---|---|---|
| **W1** — the mask widening costs nothing | **VERIFIED** (not implemented — see below) | 18 atoms + 40 pair points bit-identical; plant fires 1.12e-2 Ha at 36 orbitals, silent at 18 |
| **T1** — the transcription gates, generalized | **DISCHARGED** | worst ratio deviation 0.806x its own bound against a 4x threshold; 407 contractions pinned; 240/240 resolvable mutations fire |
| **R1** — atoms, dual-route and referee-pinned | **DISCHARGED ON THE RESCOPE** (A1.2, A3.1) | nine multiplicities exact; two sigma routes to 2.13e-14; referee 1.0e-11 / 5.3e-11 Ha on two of nine |
| **E1** — the emergent nobles | **DISCHARGED** | Kr and Xe one determinant; Kr2 and Xe2 unbound on the A3.2 grid |
| **E2** — the emergent column trend | **DISCHARGED** | 0.148293 > 0.145398 > 0.132360 Ha; Br2 binds at 0.079640 |
| **F1** — the relativistic fence, measured | **FIRED**, kept marked dead | deficit +0.0214 → −0.0013 → −0.0148 Ha: falls, does not grow |
| **P1** — the display tier | **DISCHARGED** on a substituted rule (A2.1) | valence radii reproduce period contraction and shell jumps |

### Plants

| plant | verdict | where |
|---|---|---|
| **(i)** transcription typo fires T1's ratio band | **CAUGHT** | this lane — 240/240 resolvable mutations, quietest 23x |
| **(ii)** a 32-bit truncation behind the widened path | **CAUGHT** | this lane — fires at 36 orbitals, silent at 18 |
| **(iii)** a DMRG-routed result presented as exact is REFUSED | **CAUGHT**, not by this lane | the shared provenance gate, demonstrated firing in the browser smoke test (codes 17 `DmrgClaimedExact`, 18 `DmrgUnvalidated`, 19 `UncertaintyMissing`). This lane's half is the route LABEL — every atom in the R1 record carries one, including `det (forced)` where the automatic route would have refused |
| **(A1.1)** a wrong d transform must miss the frozen counts | **CAUGHT** two-sided | this lane — all five counts, correct reproduces and planted misses |
| **(A1.3)** a density-derived radius presented as dimer-derived | **CAUGHT** | this lane — selenium, refusal fires, honest record checked not to trip it |

---

## W1 — the mask widening · **VERIFIED, NOT IMPLEMENTED**

The masks are `u64` and `MAX_ORB` is 64. What this lane did is **verify** that, not
implement it, and the distinction is the finding.

Commit `c03f282` ("expand determinant and string masks to 64/128 bits") landed between this
lane's baseline capture (`e7daece`) and its own widening commit (`8d98694`). Its diff is
this lane's prose verbatim — including the phrase *"precisely the defect the W1 plant
reproduces"*, naming a plant that existed nowhere but an uncommitted working tree. It swept
this lane's edits. There is therefore **one** widening, not two, and treating `c03f282` as
corroboration would have counted this lane's own work as its own witness. Recorded because
commit authorship is not authorship, and a swept commit is not a second source.

Verified anyway, and the verification is what stands:

* **bit-identity** — 18 atoms (H..Ar) and 40 pair points reproduce the pre-change baseline
  as IEEE-754 bit patterns, not to a tolerance. A widening reorders nothing, so the correct
  prediction is "the same f64"; any tolerance wide enough to write down is wide enough to
  hide a change to the model.
* **the packing needed to widen further.** `(ma as u64) | ((mb as u64) << n)` was correct
  only because `2 × 32` happened to be 64. `Mask` (u64) and `Det` (u128) are now separate
  types so the coincidence cannot return.
* **plant (ii) fires and is silent below.** At 36 orbitals a 32-bit mask gives 1024
  determinants against 1296 and an energy 1.12e-2 Ha adrift; at 18 orbitals the two widths
  are bit-identical.

The plant runs at one electron per spin rather than on a real species, deliberately: a
neutral >32-orbital species has no string a 32-bit mask can hold, so the truncated space is
EMPTY and the plant would fire by collapse. A gate that has only ever seen a collapse has
not shown that a narrow mask can return a *plausible wrong answer*, which is what the real
defect did.

`MAX_ORB = 64` is measured against need, not chosen: Xe2 is 54 orbitals, read from the
registry rather than written down — an earlier literal of 58 went stale the moment the
spherical projection landed and the assertion did not notice.

---

## The d-shell convention · **AMENDMENT A1.1**

Not a gate, and the largest single correction of the campaign.

The engine evaluated d shells as six Cartesian components. The freeze's own arithmetic is
derivable only under **five**: "Xe's atom is ONE determinant", "Br2 ~1.3e3", "HBr ~3.6e2",
HI ~784, "up to 54 spatial orbitals for Xe2". Under six the engine measures Kr at 361
determinants, Xe at 164,836, Br2 at 71,166,096, HBr at 36,100, HI at 16,483,600 and Xe2 at
58 orbitals — not one of which this freeze could have written.

The mechanism: the six Cartesian d functions do not span an `l = 2` space. They span the
five real solid harmonics **plus** `(x²+y²+z²)exp(−ar²)`, which is spherically symmetric and
therefore `l = 0`. In a MINIMAL basis that is a different model, not a larger one — it is
what turned single-determinant closed shells into 361-determinant problems with no chemistry
in the difference.

Measured after the projection, every freeze figure reproduces exactly: Xe 1, Br2 1296, HBr
361, HI 784, Xe2 54 orbitals.

**Size of what was removed:** krypton's spherical basis sits **1.001 hartree ABOVE** its
Cartesian one. The spurious `l = 0` function was worth a full hartree, not a rounding.

Gates on the transform, all demonstrated before trust:

* every element below Z = 21 bit-identical (no element below scandium has a d shell, so the
  projection is never built there);
* per-species dimensions asserted against what `build_basis` assembles;
* **the variational subspace ordering** — the five spherical functions span a strict
  subspace of the six Cartesian, so a full CI in the smaller space must sit *above* one in
  the larger, with no tolerance and no appeal. This is the only check here that a wrong
  orthonormal map onto the *wrong* five-dimensional subspace would fail;
* **the wrong-transform plant, two-sided**: the correct transform must REPRODUCE all five
  frozen counts and the planted one (sixth row retained) must MISS all five. It does, every
  time. A gate checking only the first half would pass on any self-consistent convention.

The convention is now a **declaration** in the registry header, because the failure was not
"we chose wrong" but "nobody wrote the choice down".

---

## T1 — the transcription gates · **DISCHARGED**

Z = 19..54 is **generated**, not typed: 36 elements, 130 shells, ~800 declared digits, emitted
by `elements3_transcribe.py` from a pinned Basis Set Exchange tabulation and a pinned NIST
mass table. At that volume a transcription error is not a risk to manage but a defect to
schedule, and the oxygen defect in the module header is what one costs.

**Three things the tabulation turned out to be**, none of which the old gates assumed:

1. **Shells are not listed in ascending principal quantum number.** Gallium's third listed
   shell is 4s4p and its fourth is 3s3p3d, because STO-3G groups each d function with the sp
   set sharing its exponents. A wrong `(n, l)` label relabels core as valence with every
   digit still right — invisible to any digit-level check. The assignment is derived from the
   coefficient triples, which identify the fit, and verified three independent ways before
   emission: no duplicate `(n,l)` per element, aufbau agreement up to STO-3G's unoccupied p
   partner, and leading exponent falling with `n` at fixed `l`. Two deliberate mislabellings
   produce 16 violations each, so the check discriminates.
2. **"One coefficient triple per shell type" is FALSE.** 3s, 3p, 4s and 4p were each fitted
   twice — once where the shell is valence, again where it has become core — and the
   tabulation carries both. A universality gate written to the tidier reading would have
   fired *honestly on correct data*.
3. **Mass is not monotone in Z.** The old gate said it was, because through argon it is.
   Under most-abundant-isotope mass there are **five** inversions, not the three the periodic
   table is known for: `40`Ar→`39`K, `59`Co→`58`Ni, `130`Te→`127`I, **and** `80`Se→`79`Br and
   `98`Mo→`97`Tc, because most-abundant-isotope mass is not standard atomic weight.
   Technetium is recorded as what it is — no stable isotope, so its `97`Tc is a
   representative choice and not an abundance at all.

**Three gate layers, catching different things:**

| layer | what it catches | measured |
|---|---|---|
| tabulation pin | hand-edits and drift, including last-place errors | 407 contractions, every declared number matched, every tabulated shell claimed |
| ratio band | errors in the source or in the `(n,l)` labelling | worst legitimate deviation **0.806x** its own derived bound, against a 4x threshold |
| coefficient universality + sp sharing | a shell wired to the wrong constant | 404 shells over 54 elements; 150 sp pairs; 30 d shells share an sp set, 20 carry their own |

The ratio band's tolerance is **derived per value**, not one constant: the declaration carries
eight decimals and the tabulation behind it ten significant digits, so xenon's leading 1s is
determined only to 5e-7 and its trailing decimals are padding. A single constant would fire
on correct heavy-row data or miss light-row typos.

### Plant (i) — CAUGHT, and its floor stated rather than tuned away

Every decimal position of all 33 xenon exponents, one unit each. **240 mutations at or above
100x the declaration's own rounding all fire, quietest at 23x** against a 4x threshold.

The honest part is the floor. A ratio band judges deviations against the rounding it is made
of, so a one-unit change in the last *determined* place is the same size as that noise and
**no ratio gate can see it** — 42 mutations sit there, and 33 more occupy a marginal decade
where 20 fire. That is the instrument's resolution, not a gap to be closed by moving a
threshold. **The tabulation pin is what covers it**: a planted last-digit change reaching only
2x the ratio band fires the pin immediately.

The historical oxygen defect itself (`130.70932140` → `130.70932000`) is planted separately
and **fires at 14.1x**.

---

## R1 — atoms · **DISCHARGED ON THE RESCOPE**

R1 was restated by A1.2 and corrected by A3.1. What was measured:

### Hund's rules come out rather than in

Ground-state multiplicity **derived** from `⟨S²⟩` on the converged vector, against the
periodic table:

| Z | | dets | E (hartree) | ⟨S²⟩ | 2S+1 | table |
|---|---|---|---|---|---|---|
| 32 | Ge | 23,409 | −2051.649918027 | 2.000000 | 3 | 3P |
| 33 | As | 2,754 | −2209.263793608 | 3.750000 | 4 | 4S |
| 34 | Se | 324 | −2373.527374279 | 2.000000 | 3 | 3P |
| 35 | Br | 18 | −2544.636780644 | 0.750000 | 2 | 2P |
| 36 | Kr | 1 | −2722.705999503 | 0.000000 | 1 | 1S |
| 51 | Sb | 9,477 | −6251.362514225 | 3.750000 | 4 | 4S |
| 52 | Te | 729 | −6547.122368855 | 2.000000 | 3 | 3P |
| 53 | I | 27 | −6850.676245633 | 0.750000 | 2 | 2P |
| 54 | Xe | 1 | −7162.104208224 | 0.000000 | 1 | 1S |

What makes this a result rather than a tautology is the **sector**: the solver works in the
MINIMAL `S_z` sector on purpose, because that sector contains every state of every
multiplicity, so a wrong guess about spin cannot be baked in. The gate asserts the sector is
minimal *first*. Nothing here is told a term symbol; the declared inputs are Z, the masses
and the basis.

### Two independent sigma routes

`sigma` (Knowles–Handy string factorisation) against `sigma_reference` (Slater–Condon
enumeration), on a probe vector rather than an eigenvalue — because eigenvalues are blind to
the failure this crate actually had, an interleaved spin-orbital ordering giving the same
Hamiltonian conjugated by a diagonal sign matrix. **Worst relative disagreement 2.13e-14
over eight atoms.**

### The 50-digit referee reaches TWO of the nine

| | engine | referee (50 digits) | residual |
|---|---|---|---|
| Kr | −2722.705999502536 | −2722.705999502525703297529011387121500966 | 1.0e-11 Ha |
| Xe | −7162.104208223682 | −7162.104208223629284562335948652300326813 | 5.3e-11 Ha |

Compared in exact decimal against the referee's digits, never by parsing the referee to f64
— that would round the reference to the precision of the thing being graded.

Kr and Xe are reachable for a **structural** reason, not a size one: every orbital doubly
occupied, so the determinant is unique up to a phase, its energy is invariant under orbital
rotation, and it is a closed expression in the AO integrals with `D = 2S⁻¹` — no eigensolve
and no SCF. The two atoms E1 asserts "exactly" are exactly the two a referee can reach
exactly.

**The other seven are OWED**, not delivered. A 50-digit FCI needs an eigensolve over the
determinant space in mpmath, and germanium's 23,409 is far past it. R1's 3e4 threshold was
staked result-blind and is a fine threshold; the arithmetic does not follow it that far.

### The referee had to be built, and its first answer was wrong instructively

The ELEMENTS-1 referee was s/p only: `CART` had no `l = 2` key, and `_self_overlap` returned
the `l = 1` formula for every non-zero `l` — so a d shell would have been normalised by the
p rule and returned a **plausible number rather than refusing**. That failure mode was
inside the instrument we grade with.

Two edits to the shared core, both **measured inert** rather than argued inert: all **424
second-row values bit-identical** through the extension.

The instructive part: the first version built the heavy table from the same pinned tabulation
and the same generator the Rust registry was emitted from, at the same rounding, and reasoned
it must therefore agree. **Krypton came back 6.3e-7 Ha off — two hundred times f64 noise.**
One rounding tie caused it: the 2p coefficient is `0.155916275`, exactly half way; the
generator rounded half-even to `...28` and the registry declares `...27`. Both defensible,
and they are **different bases**. The table is now PARSED from `elements.rs`. *A referee
grades the declared model; regenerating the same numbers beside it is how a referee comes to
disagree with its subject about something neither got wrong.*

### Route scope — and the constant these verdicts turn out not to rest on

Three different criteria are in play and this campaign has confused them once already
(A3.1), so they are separated here.

| class | criterion | Z | determinants |
|---|---|---|---|
| **has an automatic route** | `n_det ≤ MPS_ROUTE_THRESHOLD` (5e4) | 32–36, 51–54 | 1 – 23,409 |
| no automatic route; solved here via `solve_determinant` | inside this record's 1.2e6 budget | 19, 20, 30, 31, 50 | 8.2e4 – 1.2e5 |
| no automatic route; **REFUSED, did not converge** | inside the budget but Davidson stopped at its cap | **49 (In)** | 1,026,675 |
| no automatic route; over this record's **budget** | a spending cap, **not** a claim of no route | 28, 29, 37, 38, 48 | 2.4e6 – 1.1e7 |
| no automatic route; **A1.2's sixteen** | past 2.6e7, where reachability becomes the open question | 21–27, 39–47 | 2.6e7 – 1.97e12 |

The nine with an automatic route are exactly the nine A1.2 named referee-eligible — the two
criteria coincide here, which is a fact about this range rather than a definition.

**Indium is the one refusal.** At 1,026,675 determinants Davidson stopped at its
1200-iteration cap with a residual of **3.98e-1** — nine orders above the crate's declared
`CONVERGED_RESIDUAL` — and returned **2S+1 = 4.216**, which is not a multiplicity: the
Hamiltonian is spin-free, so a converged eigenvector is a spin eigenstate and 2S+1 is an
integer. Both its energy and its multiplicity are meaningless and the row is refused rather
than reported. See defect 5 — the record printed the residual and checked nothing, so on the
first run this appeared as a row indistinguishable from a measurement.

Its status is **"did not converge at the production cap"**, not a number. `DAVIDSON_MAX_ITER`
is `#[doc(hidden)]` and its own doc reserves it for `tests/front_door.rs`, stating production
never touches it — so raising it to force convergence would be reaching around a contract to
obtain a result.

The multiplicities and energies for 51–54 in the table above come from
`tests/elements3_atoms.rs`, which recomputes them, not from the record.

**The third state, named exactly:** these sixteen are **unblocked in principle, unmeasured in
fact.** `pair::MPS_MAX_ORBITALS = 6` was measured against the OLD MPO builder (LiH at six
orbitals took 528 s; HCl at ten never finished). That builder has been replaced —
channel-based, and SiO's MPO build went from "did not finish in 12 hours" to **0.07 s** —
so the constant no longer describes the engine. mixtures-engine is running gate D1 to
re-derive it properly rather than inferring it from one species, which is the right order.

**Correction to this section's own earlier claim.** It said every route verdict above rests
on that superseded measurement. It does not, and the reason matters. `automatic_route` tests
determinant count *first*:

```rust
if n_det <= MPS_ROUTE_THRESHOLD  { Determinant }      // 50,000
else if n_orb <= MPS_MAX_ORBITALS { Mps }             // 6
else                              { NoneAvailable }
```

Those two conditions cannot both hold. Six orbitals admit at most C(6,3)² = **400**
determinants, and the threshold is **50,000** — so a space inside `MPS_MAX_ORBITALS` is
always already inside the determinant threshold, and **the `Mps` arm is unreachable.** No
test constructs it; `holon-render` maps it to a viewer route code that can never be
produced. Every refusal in the table above is therefore a verdict of the *determinant
threshold*, and the superseded constant is inert — it has no effect on any verdict this
record contains.

That changes what re-deriving it can do here, in both directions. The arm first becomes
reachable at **10 orbitals**, where the maximum is 63,504 determinants; below ten, no
orbital count can exceed the threshold at all. And at ten the live window is narrow — n_det
between 50,000 and 63,504, which is half-filling and nothing else. So the *first* spaces a
raised constant would ever route automatically to MPS are ten-orbital half-filled ones,
which is the exact corner where the ladder's one mid-filled ten-orbital rung, NaH at 44,100
determinants, came back BUDGET five orders short of the stake. Raised to 18 instead, the
constant would flip this record's 18-orbital refusals (Sc–Cu) to "automatic route available"
in one step, on the strength of a rung measured at germanium's filling.

**But the successor cannot lift them, and that is knowable before it arrives.** The ladder
being run to derive it is seven two-centre pairs topping out at S2, which is 18 orbitals —
so the 22- and 27-orbital species, fourteen of the sixteen, are above anything it can
return. And its one 18-orbital rung is at the wrong filling: S2 is 32 electrons in 18
orbitals, C(18,16)² = 23,409 determinants, which is **germanium's FCI space exactly** — a row
this record already solves by determinant. The refused rows share the orbital count and
nothing else. Scandium is 1,392,554,592 determinants at the same 18 orbitals, **59,500×
larger**; the block spans four and a half orders of magnitude between its cheapest and most
expensive member. A new constant will therefore license the MPO *build* at some orbital
count; it will not license the *verdict* at these electron counts. The ladder's own rule is
that a reach without a budget is not a measurement, and that rule binds on the electron-count
axis too.

**And while this section was being written, the ladder measured exactly that.** Its fourth
rung is NaH: 10 orbitals — the *same* orbital count as HCl, which reached the stake in 55.9 s
— but 12 electrons instead of 18, so 44,100 determinants instead of 100. NaH spent its entire
300 s budget and stopped **4.391e-3 from the stake, five orders short**. Same orbital count,
441× apart in determinants, opposite verdicts.

I have to correct myself here rather than quietly strengthen: the paragraph above originally
said that no measurement bore on the filling axis in either direction, and that was true when
written and false forty minutes later. It is now measured, once, and it points the way the
argument above only conjectured. That does not upgrade the conjecture to a result — one rung
is one rung, and NaH's failure is a *time* limit at a declared budget rather than a
demonstrated limit of the method. What it does establish is narrower and harder: **orbital
count alone does not determine the verdict**, so a routing door keyed on orbital count alone
cannot be sound, whatever number is put in it.

Fixed in advance, so the rescope cannot be tuned to whatever lands: when the number arrives,
route labels move from "no automatic route" to "automatic route available" for species at or
below it, and **nothing else moves**. No refused row becomes a measured row. If the number
comes back below 18, this table does not change at all.

With one reading rule, stated now because NaH forces it. The ladder reports two numbers: the
largest orbital count that *reached* the stake, and the smallest that *did not*. Its closing
line nominates the first as the new constant — but `pair.rs` uses the constant as a door that
admits **everything at or below** it, and NaH has already put a failure at 10 orbitals while
HCl reached at 10. A maximum taken over a set that contains a failure is not a bound on that
set. This lane will therefore read the constant as the **wall minus one** — the largest count
at which every tested rung reached — and if the ladder's two numbers disagree, that
disagreement is reported here rather than resolved in this lane's favour.

A3.1 corrected this lane's own over-claim here: A1.2 said "no route at all", which read
`automatic_route`'s refusal as a statement about reachability. SiO at 132,496 determinants
solves in 33.9 s through `solve_determinant`. The sixteen span 2.6e7 to 1.97e12 and do not
share a verdict — yttrium's 1.97e12 determinants need a sixteen-terabyte CI vector, which
is a storage argument and sound; cobalt's reachability is a COST argument and determinants
are the wrong axis for it. mixtures-referee measured that from the other end: the working-
precision cost tracks NONZERO Hamiltonian elements, not determinants, and their calibrated
estimator puts germanium's 23,409 determinants at roughly 3.1e7 nonzeros. The crossing for
the As/Sb/Ge tier will be predicted by nonzeros; `mixtures1_referee/FEASIBILITY.md` carries
the method. The boundary stays **unmeasured**, and this record no longer implies
determinants are the axis to measure it on.

**The named successor is shared.** `mixtures1_referee/FEASIBILITY.md` records the same wall
from the other side (SiO's 196,889,056 nonzeros re-walked per matvec, measured not
projected). Its string-driven sigma rewrite is what both the seven owed referee atoms and
R2's SiO need — one successor, two campaigns, discharging without a re-freeze.

---

## E1 — the emergent nobles · **DISCHARGED**

**Krypton and xenon are single-determinant closed shells**, asserted as a determinant count
because that is the exact statement: one determinant means every orbital the basis provides
is doubly occupied, with no room for a single excitation anywhere.

* Kr — 18 basis functions, 36 electrons, **1 determinant**
* Xe — 27 basis functions, 54 electrons, **1 determinant**

The carrier is checked rather than assumed: **bromine (18 dets) and iodine (27 dets) are NOT
one determinant**, so "one" distinguishes closed shells rather than describing the heavy
registry.

**Neither noble dimer binds:**

| | orbitals | dets | range sampled | knots | well |
|---|---|---|---|---|---|
| Kr2 | 36 | 1 | 3.278 – 10.240 bohr | 24 | **none** |
| Xe2 | 54 | 1 | 3.773 – 12.800 bohr | 24 | **none** |

Judged against the schema's own `WELL_MIN_DEPTH = 1e-4` Ha — the same threshold under which
He2 and Ne2 report "repulsive only" — rather than one invented for the occasion, and on the
grid AMENDMENT A3.2 declares.

---

## E2 — the emergent column trend · **DISCHARGED**

In-model `D_e` falls down the halide column exactly as nature's does:

| | R_e (bohr) | D_e (Ha) | D_e (eV) | k_e | dets | route |
|---|---|---|---|---|---|---|
| HCl | 2.536888 | **0.148293** | 4.0353 | 0.34327 | 100 | determinant |
| HBr | 2.726538 | **0.145398** | 3.9565 | 0.29798 | 361 | determinant |
| HI | 3.079982 | **0.132360** | 3.6017 | 0.23192 | 784 | determinant |
| Br2 | 4.417139 | 0.079640 | 2.1671 | 0.20263 | 1296 | determinant |

All four exact in model, on the determinant route. Given Z, the masses and a basis, and
nothing else. The three hydrogen halides reproduced identically across two independent
process runs.

---

## F1 — the relativistic fence · **FIRED**, and the rescue also fails

F1 staked that the in-model deficit against experimental `D_e` **grows** down the column,
reasoning that relativity and core correlation are absent and increasingly missed. The prereg
made it two-sided: if it does not grow, the claim as stated dies.

**It does not grow. It falls, and changes sign.**

| | model (eV) | experiment (eV) | deficit (Ha) | of experiment |
|---|---|---|---|---|
| HCl | 4.0353 | 4.618 | **+0.021415** | +12.6% |
| HBr | 3.9565 | 3.922 | **−0.001268** | −0.9% |
| HI | 3.6017 | 3.198 | **−0.014836** | −12.6% |

Experimental values are Huber & Herzberg (1979) `D_e`, not `D_0` — they differ by the
zero-point energy, about 0.18 eV for HCl, which is five times the margin the middle row turns
on. HCl's value was cross-checked against NIST's spectroscopic constants (ω_e = 2990.9463
cm⁻¹, ω_e x_e = 52.8186 cm⁻¹): ZPE 0.18377 eV on D_0 = 4.4336 gives D_e = 4.617 against the
4.618 used.

**The gate asserts the fall and the two OUTER signs, and says nothing about bromine's.**
−0.9% is 0.03 eV, the same size as the uncertainty on the number it is measured against.
Locating the crossing would be reporting the error bar of a table this lane did not measure.

### Branch (b): the counterpoise question, and it CLOSES rather than rescues

The obvious competing cause is not relativity. A dissociation energy computed as
`E(A) + E(B) − E(AB)` in a finite basis carries basis-set superposition error: at equilibrium
each atom borrows its partner's functions, so the molecule sits in a larger effective basis
than the atoms do. The borrowing scales with what the partner brings — chlorine offers nine
functions, iodine twenty-seven — so BSSE grows down *precisely the column F1 walks*, and
pushes the deficit the opposite way from the missing relativity.

Counterpoise (Boys–Bernardi, ghost nuclei), at the record's own `R_e`:

| | D_e raw | D_e counterpoise | BSSE | deficit raw | deficit corrected |
|---|---|---|---|---|---|
| HCl | 0.148293 | 0.132164 | 0.016129 | +0.021415 | **+0.037545** |
| HBr | 0.145398 | 0.122989 | 0.022409 | −0.001268 | **+0.021142** |
| HI | 0.132360 | 0.112306 | 0.020054 | −0.014836 | **+0.005218** |

**Neither observable grows.** The corrected deficit falls too: +0.0375 → +0.0211 → +0.0052.

So the contaminated-observable rescue **fails**, and F1 is dead on both observables. The
question is closed rather than transferred to a successor.

Two things the correction *does* establish, worth keeping:

* **the sign change was a BSSE artifact.** Corrected, the model underbinds all three. "The
  model overbinds HBr and HI" was an artifact of the uncorrected observable;
* **the FALL is real and survives correction.** That is the part F1 got wrong, and it is not
  explained by basis-set superposition.

BSSE is 0.016–0.022 Ha and is *not* monotone (it rises HCl→HBr then falls slightly at HI), so
it is not a clean function of partner size either.

---

## P1 — the display tier · **DISCHARGED** on a substituted rule

A1.3 approved an r-expectation over the whole electron density for species whose homonuclear
dimer has no automatic route. **It was built first, measured, and does not do the job:**
averaged over every electron it is dominated by the tight core, comes out flat at about one
bohr from hydrogen to xenon, and is not even monotone — **xenon reads 1.026 against
hydrogen's 1.396.** The correct value of a quantity that does not mean what a drawn radius
has to mean.

A2.1 declares what shipped: the same expectation over the **outermost occupied orbital**. It
reproduces two facts about the periodic table that are not inputs to it:

| | Na | Al | Cl | Ar | | K | Se | Br | Kr |
|---|---|---|---|---|---|---|---|---|---|
| valence RMS radius (bohr) | 2.261 | — | — | 1.640 | | 3.499 | 2.169 | — | 1.901 |

Size **falls across a period** and **jumps when a new shell opens** (Na > Ne, K > Ar).

The failed rule is **kept** in the crate under a gate asserting it still has the defect it was
rejected for, so the next person to propose it finds the measurement rather than repeating
the work.

No density matrix was needed: `Σᵢ rᵢ²` is a one-electron operator, so its expectation is
`c·(Oc)` and feeding it through the existing sigma with the two-electron integrals **zeroed**
gives it exactly.

Two caveats travel in the declaration, not in a commit message: the orbital is the SCF's, so
"which orbital is outermost" is a property of that reference rather than of the correlated
state — adequate for a drawn radius, **not a physical observable**; and the quantity is not on
the same axis as the other two rules, which is why every surface carries `radius_from_dimer`.

**Plant (iii) — CAUGHT.** Presenting a density-derived radius as dimer-derived is refused.
Selenium is the victim, named rather than searched (dimer 396,900 determinants, atom 324), and
the honest record is checked NOT to trip the same condition or the check would refuse
everything. The type system enforced the rider before the plant was written: adding the third
`RadiusRule` variant broke `emit_palette`'s match at compile time.

---

# FINDINGS, NOT PASSES

## 1. The 3d/4s high-spin anomaly — real, and with no explanation

**Gallium comes out a quartet where the periodic table says doublet** — and zinc a quintet
against a singlet, on a solve that misses the declared convergence bar and so corroborates
rather than carries (below). Germanium onward is correct again, so the anomaly is confined to
the two elements immediately after the 3d shell fills.

**It is a property of the model, not a solver failure.** Two explanations had identical
signatures — a genuine high-spin ground state, or Davidson converging to an excited state on
a 665,856-determinant space — so a discriminator was built rather than a choice made. The
smallest DIAGONAL element of H is the best single determinant in the same orbital basis and a
variational upper bound on the true ground state; above it means the solver missed, below
means the answer is real.

| | dets | E_FCI | best single determinant | gap | 2S+1 | residual |
|---|---|---|---|---|---|---|
| Zn | 665,856 | −1757.457601385 | −1757.416112914 | **4.149e-2 below** | 5 | **2.72e-10** ⚠ |
| Ga | 124,848 | −1900.978972651 | −1900.941032919 | **3.794e-2 below** | 4 | 7.04e-11 ✓ |
| Ge *(control)* | 23,409 | −2051.649918027 | −2051.609656646 | 4.026e-2 below | 3 ✓ | 8.85e-11 ✓ |
| Ca *(control)* | 81,796 | −670.021880540 | −669.988870398 | 3.301e-2 below | 1 ✓ | 8.51e-11 ✓ |

Both controls are below theirs too, so the check discriminates rather than always passing.

### The finding rests on GALLIUM, because zinc's solve does not meet the declared bar

Added after the convergence verdict of defect 5 was applied to the record — **it caught
zinc as well as indium, and zinc was already published in this document.**

Zinc's residual is **2.72e-10 against `CONVERGED_RESIDUAL` = 1e-10**. Not the nine orders
indium missed by, but above the bar, so the record now refuses the row. It is reproducible:
two independent runs give 2.72e-10 to the digit.

What still supports the quintet, and it is not nothing: 2S+1 comes back **5.000, exactly
integral**, so the vector IS a spin eigenstate; and its energy sits 4.149e-2 Ha below the
best single determinant in the same orbital basis, so the variational bound holds. But
"probably converged, and the supporting evidence is strong" is not "converged", and the
declared bar is the declared bar.

**Gallium carries the finding.** Its residual is 7.04e-11, comfortably inside, its 2S+1 is
4.000, and it sits 3.794e-2 below its best determinant. The high-spin anomaly at the 3d/4s
boundary therefore stands on a fully converged solve, with zinc as corroboration that
carries a convergence caveat rather than as a second independent leg.

The 4p counterfactual is unaffected: it was run on gallium.

### The mechanism was proposed, tested by its own counterfactual, and REJECTED

A mechanism was available and comfortable: STO-3G gives 4p the *same exponents* as 4s, making
4p artificially compact and low, so Hund exchange over-stabilises a high-spin `4s¹4p³` over
the closed `4s²`. A story that explains a result is not evidence for itself, so it was staked
with a **prediction committed before the run**: as the 4p exponents are scaled down by λ, the
ground state should revert — gallium 4 → 2, zinc 5 → 1.

| λ | E (hartree) | ⟨S²⟩ | 2S+1 |
|---|---|---|---|
| 1.00 | −1900.978972651 | 3.750000 | 4 *(control, reproduces the registry exactly)* |
| 0.70 | −1900.974063484 | 3.750000 | 4 |
| 0.50 | −1900.899658862 | 3.750000 | 4 |
| 0.25 | −1900.707970083 | 3.750000 | 4 |
| 0.12 | −1899.999959075 | 8.750000 | **6** |

**It never fell.** Across a factor of eight in exponent and 271 millihartree of energy cost
the quartet did not move, and at the extreme it went **up** to a sextet. The shared-exponent
mechanism is **falsified on its own pre-registered counterfactual.**

**The λ = 0.12 sextet is a datum, not merely the failure of the prediction.** The
perturbation drove multiplicity the *wrong way*, which constrains any successor mechanism
more than "unchanged" would have: whatever explains the anomaly must also explain why
diffusing 4p raises the spin rather than lowering it.

**Untested candidates, left unasserted:** the 3d shell (which for gallium shares its
exponents with 3s3p, not with 4sp), and the 4s/4p near-degeneracy independent of how diffuse
either is. Both are testable the same way — scale one shell, leave the rest declared, watch
`⟨S²⟩`.

**The fence:** neither element is in the multiplicity gate. Zinc alone is minutes of suite
time, and gating a number whose mechanism is a *reading* would freeze it where it stops being
asked about. What the gate has instead is a `KNOWN_DISAGREEMENTS` list that **refuses** to let
either be added to the expected-agreement list, so the next person who notices the clean sweep
of nine and helpfully extends it gets a clear failure rather than a confusing one.

## 2. F1's fired fence, and a closed question

Covered above. The short form: **a gauge that fires is the campaign working, not failing.**
F1's stake is dead on the raw observable and dead on the counterpoise-corrected one, so the
question closes rather than passing to a successor. What survives is a sharper statement than
the stake: the model's deficit against experiment falls down the halide column, and basis-set
superposition error is not why.

---

# DEFECT LEDGER

Four defects of this lane's own. **All four are the same shape** — a check that could only see
what it expected — and each is recorded with the instrument it produced.

| # | defect | why it survived | instrument now |
|---|---|---|---|
| 1 | `Species::n_basis()` kept summing CARTESIAN components after the projection landed: the registry said xenon was 29 functions while the engine assembled 27, so `pair::automatic_route` overstated every d-bearing species | the only covering test ran Z ≤ 18, which has **no d shells** — it established one direction on a set where the two conventions cannot differ | every element's declared `n_basis` asserted equal to what `build_basis` assembles, for all 54 |
| 2 | AMENDMENT A1.2 read `feasibility`'s refusal — a statement about the AUTOMATIC route — as "unreachable" | the refusal is correct about what it says; the over-reading was in the prose around it | A3.1, plus corrected doc comments on `RadiusRule::ValenceDensity` and `homonuclear_size` |
| 3 | the 4p counterfactual's verdict column tested only `mult < base − 0.5` and printed **"unchanged"** against the row where the multiplicity had risen from 4 to 6 | the check was written to detect the direction the prediction staked | two-sided verdict; the log carries a header recording the wrong label rather than being regenerated clean |
| 4 | in the f-projection review, a **support-only** row pin (blind to a sign flip) with a `\|\| p != SPHERICAL_F` fallback in its own plant, which made "caught" trivially true for any mutation | the plant was written to guard against exactly this and reproduced it one level down | value pin; the plant calls the same function the pin test calls, so it cannot pass by testing something weaker |
| 5 | the R1 record printed `sol.residual` and **checked nothing**, so indium — Davidson stopped at its 1200-iteration cap, residual **3.98e-1** against ~1e-10 elsewhere — appeared as a row indistinguishable from a measurement. Applying the fix then caught **zinc** too, at 2.72e-10, a number this document had already published | the crate had ALREADY written `pair::CONVERGED_RESIDUAL` for exactly this, with a doc comment describing it word for word; I did not reach for it | the row is REFUSED on two independent checks — the declared residual bar, **and** 2S+1 integrality, which is free and catches the failure for a reader who never looks at the residual column |

The fifth is the sharpest: **the fix already existed and was walked past.** The constant, the
bar and the doc comment naming this exact failure were all in the crate before this campaign
started. Writing a new instrument is not the hard part; reaching for the one already there is.

The fourth is the other one worth keeping: **the plant caught the author before it caught
anything else.** The best statement of the family is the one it produced — *a check that can only see
the direction it expects reports every other direction as nothing happening, which is worse
than not checking at all.*

A record-hygiene note that goes with #3: the counterfactual log keeps its wrong-label header
rather than being regenerated clean. **A regenerated-clean log is a record that lies about its
own history.**

---

# CROSS-LANE

Two items this lane resolved that were not its gates.

**The wasm stack trap was misattributed to W1.** A browser artifact built from current source
trapped at the wasm-ld default 1 MiB stack while every native suite was green. The bisect
point predated both W1 and another lane's uncommitted f-integral constants, so it could not
separate them. Two-sided rebuild: clean HEAD (with the full widening) **green**; clean HEAD
plus *only* the four f-shell constants **traps**, same function indices. `MAX_ORB` sizes no
stack array anywhere — it is an assertion bound and a default argument.

The usage that overflowed was `RTensor::work`, through a construct that *looks* like a heap
allocation: `Box::new(<array literal>)` materialises the literal on the STACK before moving
it, 685,464 bytes at RMAX = 12 against 157,464 at RMAX = 8. Fixed by building a boxed **slice**
through `vec!` — 52,728 bytes peak — and `smoke.mjs` passes at the **default** stack, so no
`-zstack-size` bump was needed and the 1 MiB default survives as the only stack-growth
detector the artifact has. **`Box::new(<big array literal>)` is a stack allocation wearing a
heap allocation's clothes, and it is invisible in review precisely because the type says
`Box`.**

**The f projection is correct; its gates pin the subspace, not the rows.** `SPHERICAL_F` is
orthonormal in an independently re-derived Cartesian Gram to 2.2e-16 and annihilates all three
`l = 1` contaminants to 4e-16. But the space orthogonal to those contaminants is exactly
seven-dimensional, so ANY orthonormal basis of it passes both conditions — demonstrated: a row
swap, a sign flip and a 40° rotation all pass with identical residuals. Harmless for total
energies (full CI is invariant under orbital rotation); **not** harmless for anything
orbital-resolved, and this crate has one — `atomic_valence_rms_radius` picks the outermost
occupied orbital by column index. A row pin and a two-sided plant now close it. No
determinant-count plant exists for f and that is recorded rather than silent: ELEMENTS-3
stakes no f-bearing species, so the plant has **no carrier** and a plant on an empty sector
VOIDs.

---

# WHAT THE CAMPAIGN SAYS, IN ONE PAGE

The periodic table through xenon emerges from **Z, the masses, and a basis** — nothing else,
no fitted parameter, no table of chemical results.

**What was shown.** Transcription is machine-guarded at three independent layers, with the
gate's own resolution floor stated rather than tuned away. Every route is labelled and every
refusal named. The nobles close themselves: krypton and xenon are single determinants and
neither dimer binds. A trend of nature is reproduced in kind — `D_e` falls HCl > HBr > HI from
the declared inputs alone. Hund's rules come out of `⟨S²⟩` for nine heavy atoms with nothing
about spin supplied, and two of those atoms are pinned against a 50-digit referee at 1e-11 Ha.
Atomic size, derived from the atom's own valence orbital, contracts across a period and jumps
at each new shell.

**What was NOT shown, and is written down as such.** Lanthanides or anything needing f
functions — `MAX_Z = 54` and the refusal is in force. Relativistic fidelity: F1 measured that
edge and the fence claim **fired**, on both the raw and the counterpoise-corrected observable.
Quantitative thermochemistry. Mid-row exactness: sixteen atoms have no automatic route, their
determinant-route reachability is **unmeasured**. The constant those refusals were attributed
to turns out not to enter them at all — the route arm it guards is unreachable at the current
thresholds — and its successor's ladder could not reach these species anyway, topping out at
germanium's FCI space wearing a different name. Seven of nine referee-eligible atoms have no 50-digit reference.

**The one thing the campaign found that it cannot explain.** At the 3d/4s boundary the model
prefers a high-spin ground state — gallium a quartet on a fully converged solve, zinc a
quintet corroborating from one that misses the bar — and this is real by a discriminator that
clears the solver, while the mechanism proposed for it was falsified by its own pre-registered
counterfactual. It stands with **no explanation**, which is a better state
than a wrong one, and the sextet at λ = 0.12 is a constraint any successor mechanism must
meet.

**And the correction that mattered most was not a gate at all.** The d-shell component
convention was never written down; the engine carried six Cartesian components and the freeze's
own arithmetic had been written against five. The difference was a full hartree in krypton and
the difference between a closed shell and a 361-determinant problem. It is now a declaration,
and so is the f convention — written before any f element exists rather than after one goes
wrong, because the lesson was never "we chose wrong" but "nobody wrote the choice down".
