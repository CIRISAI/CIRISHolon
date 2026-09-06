# LEPTON_LADDER — the chiral wall named, and the routes through it PRICED before any freeze

*The file `GANTT.md` row L6 and `OBJECT.md` Fold III cite. A pricing note, not a freeze: it
carries no claim, runs no compute, and types no number from outside the crate except exact
integers of the lattice (doubler counts, operator dimensions) and the credited theorems. The
electron is an INPUT to every tier of this ladder (`OBJECT.md` "what the fold does not
claim"; `TIERS.md` §fold; `GANTT2.md` §Leptons: beside the fold, not below it). Nothing here
changes that; this note says what it would cost to put a lepton ON the lattice, so that a
lepton row can one day carry something, and what would convict each route.*

## 0. Two questions, priced apart

The ladder's electron and the Standard Model's electron are different jobs, and the wall
stands in front of only one of them.

1. **The atom tier's electron is vector-like.** QED couples both chiralities alike, so the
   electron the chemistry tier takes as given needs NO chiral route: a Wilson or a staggered
   fermion carries it, at the price each has (§2, R1–R2). Today the tier does not put it on a
   lattice at all — it is a constituent in the chemistry basis (`holon-chem`), and the seam
   campaigns ran on that. The fold's quarks are the same case: QCD is vector-like, and
   `OBJECT.md` Fold III says "staggered quarks suffice" for that reason.
2. **The weak interaction's electron is chiral** — only the left-handed lepton couples — and
   THAT is what the wall forbids on a naive lattice. No campaign on this ladder asks for it.
   It is priced here (§2, R3–R6) so that when the ladder reaches the question the routes are
   already costed and each has a kill.

## 1. The wall

**Nielsen–Ninomiya (1981).** In an even number of dimensions no lattice fermion action can be
all of: local, translation-invariant, Hermitian, with the right continuum limit, and chiral
without doublers. The naive discretisation of the Dirac operator pays with doublers: `2^d`
poles in the Brillouin zone — 4 in 1+1D, 16 in 3+1D — of which half carry each chirality, so
the lattice theory is vector-like whatever the continuum one was. Every route below gives up
one of the theorem's legs and is priced by which. Credit: Nielsen and Ninomiya, *Nucl. Phys.
B* 185 (1981) and 193 (1981); Karsten and Smit (1981); the review by Kaplan, *Chiral
symmetry and lattice fermions* (Les Houches 2009), which this note follows for the routes.

## 2. The routes, each with its cost on this engine and its kill

The engine carries fermions two ways: as conserved-integer LANES of determinant strings
(E11, `Lane solver is the engine`), and as a labelled MPS (E14). Both scale in the DIRAC
OPERATOR'S DIMENSION PER SITE — the number of one-particle orbitals a site contributes —
so that is the unit every route is priced in. `V` is the site count of the box.

| route | what it gives up | orbitals per site (3+1D, one lepton) | what it costs beyond that | the kill that convicts it |
|---|---|---|---|---|
| **R1 Wilson** (1974) | chiral symmetry, explicitly: an `r·a·∇²/2` term lifts 15 of the 16 doublers to the cutoff | 4 | an additive mass renormalisation (the bare mass is TUNED to the chiral point); `O(a)` errors unless improved | the free propagator's pole count — EXACTLY ONE light pole in the zone, computable on the lanes to machine precision; and the tuned pion-mass-squared failing to vanish linearly at the tuned point |
| **R2 staggered / Kogut–Susskind** (1975) | most of the spinor: one component per site, the rest spread over the hypercube; `2^{d/2}` TASTES remain (2 in 1+1D, 4 in 3+1D) with a remnant `U(1)_ε` chiral symmetry | 1 | one lepton flavour needs the FOURTH ROOT of the determinant (rooting), whose locality is the open question in the literature (credit Sharpe, *Lattice 2006* review; Bernard–Golterman–Shamir); taste splitting at finite `a` | taste splitting not closing as `a²`; the rooted theory's non-locality showing in a measured correlator. This is the route the fold's quarks and the crystal's Schwinger instruments already run (`SCHWINGER3_RESULTS.md`: `M_V/g` to 2 % on staggered fermions in 1+1D) |
| **R3 domain wall** (Kaplan 1992; Shamir 1993; Furman–Shamir 1995) | locality in a FIFTH dimension of extent `L_s`: a Wilson fermion in 4+1D with a mass domain wall, the chiral mode bound to each wall | `4·L_s` | `L_s` copies of a Wilson lattice; the residual chiral breaking `m_res` falls exponentially in `L_s` and must be MEASURED, never typed | `m_res` not falling with `L_s`; the two walls' modes mixing at the box's `L_s` |
| **R4 overlap** (Neuberger 1998; Ginsparg–Wilson 1982; Lüscher 1998) | nothing of the theorem's letter — it has exact lattice chiral symmetry — at the price of a NON-ULTRALOCAL operator `D = 1 + γ₅·sign(H_W)` that is exponentially local | 4, with the sign function of a `4V × 4V` Hermitian Wilson operator applied at every use | a polynomial or rational approximation of `sign(H_W)` whose degree is set by the condition number of `H_W` (the literature reports one to two orders of magnitude over Wilson per application; credit Kennedy, *Algorithms for dynamical fermions*, 2006; to be MEASURED here, never quoted) | the Ginsparg–Wilson relation `γ₅D + Dγ₅ = a·Dγ₅D` — an exact algebraic identity, checkable on the lanes to machine precision — read beyond the approximation's own residual |
| **R5 SLAC** (Drell–Weinstein–Yankielowicz 1976) | locality: a non-local lattice derivative with no doublers | 4 | a DENSE derivative, `V²` couplings per component, and a locality the object contract's own budget refuses (`OBJECT.md` Fold II: `Core/Locality.lean`, `Budget.lean`) | REFUSED before pricing: the contract's locality leg is the kill, and it fires at construction |
| **R6 the 1+1D chiral rehearsal on the lanes** | nothing new: the Schwinger model on the EXISTING instrument (`conformance/crystal/instrument/dmrg_schwinger.py`, banked at SCHWINGER-3) with a chiral observable added | the instrument's own (staggered, 1 per site) | one new observable and one prereg; NO new instrument. The 1+1D axial anomaly is exact and elementary — the axial charge is not conserved, its rate fixed by the electric field with coefficient `e/π` (Schwinger 1962; the lattice statement in Banks–Kogut–Susskind 1976) — and the staggered lattice's remnant `U(1)_ε` makes it the cheapest measured statement about chirality this ladder can make | the anomaly coefficient missed on the continuum extrapolation, read exactly as SCHWINGER-3 read `1/√π` (band and posability rules inherited unchanged) |

A second rehearsal of R3 at the same low price: a mass domain wall on a 2+1D Wilson lattice
binds a 1+1D chiral mode to the wall (Jackiw–Rebbi 1976; Callan–Harvey 1985) — the
`4·L_s`-orbital cost of R3 rehearsed as a 2D lattice of `2·L_s` orbitals per site on the
lattice crate's own torus, its kill the bound mode's chirality read from its dispersion.

## 3. The order, sized by compute

1. **Nothing** for the atom tier: its electron is vector-like and is today a constituent in
   the chemistry basis. No row waits on this note.
2. **R6 first** — a prereg owed, no instrument owed; priced at SCHWINGER-3's own ladder (its
   `N` and `χ` rungs and posability rule), one observable added. The first measured
   statement about chirality on the engine; a kill of the ladder's own arithmetic, not of the
   world.
3. **R3's 2+1D rehearsal** on `holon-lattice`'s torus, after R6: `2·L_s` orbitals per site,
   `L_s` swept until the measured `m_res` is below the box's own resolution.
4. **3+1D routes only after E10** (the 3D finite-group box), as `GANTT.md` row L6's
   dependencies say: R1/R2 for a vector-like lepton in a box, R3 or R4 for a chiral one,
   R4's sign function priced by the measured condition number of `H_W` on that box.

## 4. The electron in the closure grammar

In the grammar the ladder is written in (`GANTT2.md`: productions unit / bond / rewrite,
a phase a spanning bond cluster) the electron is a TERMINAL: no production makes one and no
rewrite unmakes one, on any tier. Whether it stays a terminal is not a question this
ladder can pose until a route above is run, and the note carries no wager on it.

## 5. What this note does not do

It does not claim a lepton, a chirality, an anomaly or a mass; it does not choose a route;
it types no cost — every "one to two orders" above is the literature's, credited, and is to
be replaced by a measured `price.json` the first time a route runs. When a route runs, its
freeze cites this note and this note is not edited to agree with the result.
