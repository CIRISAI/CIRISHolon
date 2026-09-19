# Fast and slow — why the exact tiers fill in minutes and the fluid tier in core-days, and how the fast side renders the slow side

*2026-09-19, the day GF1's whole ladder read in 31 minutes on two cores while RESPONSE-1's
eight arms took a day of the machine for one fluid-element reading. Written to the owner's
question: what is the nature of the difference, and can the fast side render the slow side
by omissions or negations? The answer is yes, in three named moves, two of which the
program has already made without naming them, and one lemma is owed.*

## 1. The nature of the difference: whether the Record is compressible

Every tier's object is a lossy view, and what the view forgets is the Record — the
forgotten Identities. The commuting square (coarsen then evolve, evolve then coarsen; the
0:1:0:1 walk) closes when the Record does not come back through the view.

**On the fast side the Record is a gapped, finite fibre and is summed EXACTLY.** The
Schwinger vacuum on 48 sites sits in a matrix product state of bond dimension 8 to a
variance of `10⁻¹²` per site; every observable is one contraction; the plants are identities
(brute enumeration to `10⁻¹⁵`, Clifford invariance to the bit). A reading is one number,
deterministic, seconds. Closure holds exactly or fails with a finite witness. And theorems
act on it at zero compute: Clifford invariance retired the prereg's S3, parity retired the
amendment's — the fibre's symmetry held the staked quantity fixed before any run.

**On the slow side the Record is thermal and can only be AVERAGED.** Each of the 2,592
rigid degrees of freedom in a 432-water box carries `kT` and is forgotten by the density
and momentum chart; the state is as mixed as its energy shell allows and compresses to
nothing but the chart itself. The square closes only in expectation. On any one trajectory
its holonomy is the Record leaking through a cell's faces at shot noise `1/√N_cell`, so a
reading costs `√(samples)` per digit and every sample is the full propagation of the box
(`2,000` core-seconds per picosecond). Closure is `D ≤ β`, statistical; so is non-closure —
there is no witness, only a `D` above the floor.

That is the whole of it: **fast is a compressible Record, slow is an incompressible one.**
The two are one thing in two regimes, and the boundary moves: at weak coupling the Schwinger
vacuum's bond dimension outgrew the exact instrument and the fast side became slow (`x = 16`
refused past `N = 12`), for the same reason the liquid is slow — the fibre stopped fitting.

## 2. Three moves by which the fast side renders the slow side

**Negation — a symmetry or an arithmetic says a reading is identically zero or one, so it
is a NULL, not a stake, and is never measured.** This is what turned the 3,000 core-hour
fluid-element fence into 24: continuity at equilibrium reads `D_cont = 1` by arithmetic (a
cell's coherent drift is a fifth of its thermal crossings), so the equilibrium read was
measuring nothing; the transverse kick has `∇·v = 0`, so continuity must see nothing (R1′);
the undriven quadrature carries nothing (R4′); the scrambled partition carries nothing
(R4). Nulls are cheap because "nothing" needs a noise floor, not a signal, and a null that
fires convicts the instrument at the price of one floor. **Rule: before a stake is run, test
it on the fast side for being a theorem's constant.** If it is, it is a plant or a null.
The program's recurring fault — a constant taken as a price measured in a regime, nine
instances this month — is precisely the omission of this test.

**Omission — drop a part of the Record and MEASURE the price.** REPLACE-0 dropped the three
vibrational degrees of freedom per water: `14.6×` cheaper, `+0.325 ± 0.099 kT` per water less
bound, dynamics unresolved past 40 fs. The next omissions are rotations (point particles),
then positions (fields). Each omission is a tier, each price a number, and the cube is the
table of omissions with their prices. The fast side's part: an omitted part's price can
often be DERIVED — a harmonic mode's free energy is `kT ln(ħω/kT)`, a rigid rotor's
partition function is closed-form — so the measured price checks a derived one, and the
derivation says in advance which omissions are free at the scale that matters. The scout's
result that the density chart is shot-noise-limited at every liquid this machine can run
was such a derivation, made after the run instead of before.

**Composition — measure the residue ONCE on the smallest cell that holds it, and let the
theorem carry it.** What is neither a symmetry's zero nor an omission's price is the
residue: a memory kernel, local in time (the shear kernel decays in ~100 fs, the density
kernel in ~2 ps) and local in space (the far field is a coarse halo). RESPONSE-1 measures
it on 432 waters for 24 core-hours a seed. `viewClosed_comp` says closed views compose: the
certificate for one cell under coarse neighbours is the certificate for the lattice, and the
price of the lattice is the price of one cell (`FLUID_ELEMENT_RESPONSE.md` §4). Read
through Mori–Zwanzig: coarse dynamics = a closed part (derived) + a memory (measured once,
local) + the noise (the Record, at its floor). The fence's 3,000 core-hours were the price
of measuring the noise; the residue costs one percent of that.

## 3. What this does not buy, said now

- **Composition is proved for exact closure.** For statistical closure (`D ≤ β`) the
  certificates compose only if the cells' noises are independent or the budgets add; that
  is a lemma OWED, not a theorem held, and RESPONSE-1's read on one cell is not yet a read
  on a lattice of them.
- **The residue is not a zero of anything.** A viscosity or a diffusion constant must be
  measured once; negation removes what surrounds it, not it. And at the wavevectors a
  432-water box admits the transverse mode is near water's onset of propagating shear
  waves, so the first residue read may be a `k`-dependent kernel, not a hydrodynamic
  constant — the arms will say.
- **The fast side has its own wall**, and it is the same wall: when the fibre stops
  compressing, the exact instrument refuses. GF1's weak-coupling refusal and the liquid's
  shot-noise floor are one statement about an incompressible Record. The moves above push
  the boundary; they do not remove it.

## 4. The rule, in one line

**Derive the zeros, price the omissions, measure the residue once, compose.** A campaign
that runs before its zeros are derived pays for noise; one that measures every cell pays
for the same residue `N` times.
