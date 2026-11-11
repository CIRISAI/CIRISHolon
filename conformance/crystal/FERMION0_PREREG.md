# FERMION-0 — a fermion that is not an input: the toric code's bound charge and flux, put to the holon test on the stabilizer tier. PREREGISTRATION

*Frozen 2026-11-11, committed alone, at the lead's word of the same day ("yep, proceed") on a
draft whose stakes are the instrument author's and were not re-worded by the lead. At this
commit no FERMION-0 circuit generator exists, no exchange sign has been computed by any
instrument of this campaign, and no predictor has been built. What HAS been run: the
engine's `holon-qasm` release binary was built and one three-qubit circuit was put through
it to learn its output format, and stim 1.16.0 was installed in a scratch environment. The
draft's stakes S4 and S5 were re-designed before this freeze, for the reason given in
§"What changed from the draft"; no number was seen in between. Every piece of physics below
is the literature's and is credited; the direction of every stake is expected, and the
campaign is a RE-FINDING by design. What it tests is the instrument and the theory's
vocabulary, on a substrate where the answer is exact.*

## Why

`LEPTON_LADDER.md` says the electron is an INPUT to every tier and a TERMINAL of the grammar:
"no production makes one and no rewrite unmakes one". It prices six routes to a lepton on a
lattice and runs none. Before any of them, one prior question can be put exactly and cheaply:

> Is there, on a substrate this engine already hosts, a fermion that is a closed view of a
> vacuum holding no fermions, and that the tier above must carry?

If the answer is no on the simplest substrate where physics says it should be yes, the
theory's test for objects is wrong at the bottom of the ladder. If it is yes, "a fermion is
an input" becomes "a fermion need not be an input on this substrate", and nothing more.

## What is already known (credited; none of it is claimed)

- The toric code and its excitations: Kitaev, *Ann. Phys.* 303 (2003). Charges `e` on
  vertices, fluxes `m` on plaquettes, both bosons; a full braid of one around the other
  gives `−1`; the bound pair `ε = e·m` is a fermion, and turning it once about itself gives
  `−1` (its topological spin).
- Statistics from three hopping operators meeting at a point: Levin and Wen, *Phys. Rev. B*
  67 (2003).
- Fermions from bosonic models generally: Levin and Wen, string-net condensation (2005).
- That a stabilizer simulation carries all of it exactly: Gottesman–Knill; Aaronson and
  Gottesman (2004).
- This is a `ℤ/2` gauge theory. It has no chirality, no continuum limit taken here, no
  mass, and nothing of the weak interaction. The Nielsen–Ninomiya wall is untouched.

## Carrier and instrument

- **The substrate.** The toric code on an `L × L` torus, `L = 12`: 288 qubits on the links,
  144 vertex operators `A_v` (`X` on a vertex's 4 links) and 144 plaquette operators `B_p`
  (`Z` on a plaquette's 4 links). The vacuum is prepared from `|0…0⟩` by a unitary Clifford
  circuit: for each vertex but the last, a Hadamard on one of its links and three CNOTs
  from it to the other links of its star.
- **Excitations.** An `e` at vertex `v` is `A_v = −1`; an `m` at plaquette `p` is
  `B_p = −1`. A `Z` on a link moves an `e` along it; an `X` on a link moves an `m` across
  it. An `ε` at site `v` is an `e` at `v` and an `m` at the plaquette to its north-east,
  and it hops as one: the `Z` on the link it travels and the `X` on the link its plaquette
  crosses. The two never fall on the same link, so every hop is a Pauli string squaring to
  `+1`.
- **A process** is an ordered list of hops and twists that returns the set of occupied
  sites to itself. A TWIST of an `ε` is its `e` carried once around its own `m`: the four
  `Z` of that plaquette.
- **The sign, as a measurement.** An ancilla is prepared in `|+⟩`, every Pauli of the
  process is applied controlled on it in order, and the ancilla is read in `X`. The
  outcome's exact distribution is taken from the simulator, not sampled.
- **The syndrome.** After the process, every `A_v` and `B_p` is read through its own
  ancilla. 577 qubits a circuit.
- **The reporting rule.** A sign is REPORTED only if the sign ancilla is deterministic and
  every stabilizer is deterministic and equals the configuration the process claims.
  Otherwise the process is REFUSED and no sign is printed.
- **The exchange.** A junction: a centre and three arms, each `c` links long. Particles
  start at the ends of two arms. One exchange is six arm-moves: second particle to the
  centre, on to the empty arm; first to the centre, on to the second's arm; the parked one
  back to the centre, on to the first's arm.
- **The engine's instrument.** Each process is one OPENQASM 2 circuit, Clifford only, run
  as `holon-qasm run --tier tableau`, which prints the exact distribution of the classical
  register.
- **The referee.** The same gate list on stim 1.16.0's tableau simulator, each ancilla read
  by its exact expectation. A disagreement on any circuit is a kill of the instrument and
  nothing is read.
- **Cost.** About 7,000 Clifford circuits of at most 577 qubits. Not measured, so no figure
  is claimed: the first 10 are timed and the extrapolation printed before the rest run.
  CPU only, pinned with `taskset` to one core class, niced; no wall clock is compared.

## The coarse tier and the three predictors

The coarse tier reads the torus in `c × c` cells (`c = 2` and `c = 3`; a vertex and the
plaquette to its north-east share a cell). It sees, per step, how many `e` and how many `m`
each cell holds and how many crossed each face. A twist stays inside one cell and crosses no
face, so the coarse tier does not see it happen.

Each junction is placed with its centre on a cell corner and arms of length `c`, so the two
particles, the centre and the parked position always lie in four different cells and no
cell ever holds two.

- **P-parts.** Knows what the parts are: `e` is a boson, `m` is a boson, and a full braid
  of an `e` around an `m` gives `−1`. From the coarse history it follows each `e` and each
  `m` from cell to cell. An `e` and its `m` move through the same cells at the same steps,
  so no braid of one around the other is resolved, and it predicts `+1`.
- **P-whole-bare.** Treats each bound pair as one object with one label, "fermion". From
  the coarse history it follows each `ε` from cell to cell and predicts `−1` raised to the
  parity of the permutation.
- **P-whole.** P-whole-bare, and one bit of memory per object, flipped each time that
  object is twisted. It predicts P-whole-bare's sign times `−1` raised to the twists.
- **P-fine** (lattice resolution, no cells). Knows only what P-parts knows, and is given
  the parts' full paths, link by link. It predicts the product, over every `Z` step and
  every later `X` step on the same link, of `−1`. It is computed in plain integer
  arithmetic and does not call either simulator.

## Stakes

- **G0 (the vacuum).** On the prepared state all 288 stabilizers and both `Z` loops around
  the torus read `+1`, deterministically, on both simulators. EXACT. If not, nothing runs.
- **S1 (Gauss's law is the outflow law, in `ℤ/2`).** For 200 random rectangular regions on
  each of 10 states reached by random hopping, the product of the `A_v` inside equals the
  `X` loop on the boundary, and the same for `B_p` and the `Z` loop. EXACT: 4,000 of 4,000.
  *Kill:* 1 disagreement. Witness `SceneCharge.gpop_total`.
- **S2 (no lone charge; the vacuum can pair).** Over 2,000 random Pauli products applied to
  the vacuum (each qubit non-identity with probability 0.05), the number of `e` is even and
  the number of `m` is even every time, and at least 1,800 of them create at least 1 pair.
  EXACT on the parity. *Kill:* 1 odd count. Witnesses `SceneCharge.no_lone_charge`,
  `vacuum_can_pair`.
- **S3 (the statistics).** Reported signs: `e` exchanged with `e`, `+1`; `m` with `m`, `+1`;
  `ε` with `ε`, `−1`; a full braid of `e` around `m`, `−1`; a twist of `ε`, `−1`. Each on
  100 placements: 500 of 500, EXACT, none refused. *Kill:* 1 other sign.
- **S4 (counts and fluxes of the parts do not determine the sign).** For each `c` in
  `{2, 3}`: 500 pairs of processes `(A, B)`, where `B` is `A` with one twist added. The two
  have the IDENTICAL coarse history of `e` and of `m`, checked field by field before any
  sign is read. Stake: the reported signs differ in 500 of 500 pairs at each `c`. EXACT.
  This is `Object.NonFactoring` measured: two histories agreeing under every coarse view of
  the parts, split in a quantity. *Kill:* 1 pair with equal signs.
- **S5 (the tier above must carry the whole, with its label and its memory).** For each `c`
  in `{2, 3}`: 1,000 processes, 250 in each cell of (one exchange or two) × (no twist or
  one). Stake, at each `c`:
  - P-whole is right on 1,000 of 1,000;
  - P-parts is right on at most 550 of 1,000;
  - P-whole-bare is right on at most 550 of 1,000.

  *Kill (the composite is not the object):* P-whole misses 1 or more. Then the bound pair
  with a fermion label and one bit is not a sufficient statistic at this acuity.
  *Kill (the composite is not needed):* P-parts is right on 950 or more. Then the parts'
  coarse counts already carry the sign, and by this record's own rule the composite is a
  closed pattern and NOT a thing.
- **S6 (the object appears where its parts stop being resolved).** On the same 2,000
  processes, P-fine is right on 2,000 of 2,000. EXACT. At lattice resolution the parts and
  their one mutual label suffice and no composite is needed; at `c = 2` and `c = 3` it is.
  *Kill:* P-fine misses 1 or more — then the mutual-braid rule as written here is not the
  parts' whole story and S5's contrast is not about resolution.

## What would make a stake vacuous

- S3 is vacuous if the circuits are not the processes they are named for. The reporting
  rule, P-TRIVIAL and P-OPEN are its guards.
- S5 is vacuous if the four kinds of process are not balanced; the design fixes 250 each.
- S5 says the composite is carried FOR THIS CONSUMER, the exchange sign. A consumer that
  reads only energy never needs it. "Carried" is relative to the target named.
- S6 could be read as trivial, since the full paths are the operator word. That is the
  point of it: it marks the resolution at which nothing is hidden. It carries no claim
  beyond that.

## Plants

Each runs before any stake is read and prints ahead of every stake line.

- **P-ORDER** (carrier: a 3-qubit circuit with one qubit flipped; sector: the bit order of
  the engine's printed register). The reader must name the flipped bit. 1 of 1.
- **P-TRIVIAL** (carrier: 288 qubits each in `|+⟩`, no gauge structure; sector: the
  reported signs of S3's 500 processes). There the plaquette operators have no definite
  value, so every process must be REFUSED by the reporting rule. 0 signs may be reported.
- **P-OPEN** (carrier: the toric code vacuum; sector: 50 hopping words that do not close,
  leaving 1 excitation displaced). Every one must be REFUSED. 50 of 50.
- **P-ONE** (carrier: a syndrome record of S2 with 1 bit flipped by hand; sector: the
  parity checker). The checker must report 1 odd state of 1.
- **P-REFEREE** (carrier: every circuit of G0 and S1–S5; sector: its ancilla
  distributions). The engine and stim agree on every ancilla of every circuit, 100 %.
- **P-LABEL** (carrier: S5's 1,000 processes at `c = 2`, with the label read as "boson";
  sector: P-whole's rule). P-whole must then be right on at most 550 of 1,000.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier: P-ORDER's flipped bit is set by an `X` gate; P-TRIVIAL's state has random
plaquette values by construction (each `B_p` anticommutes with the `X` stabilizers of its
links); P-OPEN's displaced excitation exists because the word's two ends differ, checked on
the claimed configuration before the run; P-ONE's flipped bit is read back before the
check; P-LABEL acts on the 500 processes of odd exchange, fixed by design.

## Branches (evaluated in this order)

- **(e)** G0 fails, a plant fails, or the referee disagrees. Nothing is read.
- **(k)** S1, S2, S3 or S4 is killed. The instrument or the model is wrong about known
  physics; the failure is the finding and no claim about objects is made.
- **(a)** S1–S6 met. On this substrate a fermion is a closed view of a vacuum that holds
  none; it cannot be made alone; and at an acuity that does not resolve its binding the
  tier above must carry the whole, with a label and one bit of memory that the conserved
  counts and fluxes of its parts do not give. `LEPTON_LADDER.md` §4's sentence is then true
  of the electron and not of fermions as such.
- **(b)** S1–S4 met, S5 killed. Stated with the kill that fired: the composite is not the
  object, or it is not needed.
- **(c)** S1–S5 met, S6 killed. The contrast in S5 is real and is not a matter of acuity.

## Order

1. This file, committed alone.
2. The circuit generator, the reader and the four predictors are built. The first 10
   circuits are timed.
3. G0, then P-ORDER, P-TRIVIAL, P-OPEN, P-ONE.
4. S1, S2, S3.
5. The processes of S4 and S5 are drawn (seed 20261111); the coarse histories of each pair
   are compared; P-LABEL; then S4, S5, S6.
6. P-REFEREE is reported over everything that ran.

If step 2 cannot be built as written, a dated note is appended here under "Notes on
building" and committed alone before step 3.

## What changed from the draft, and why (before any number)

The draft's S4 compared a parts predictor and a whole predictor on bound pairs "whose
binding lies inside one cell", and its S5 staked that at cell size 1 the parts suffice.
Working the geometry before the freeze showed two faults. First, two particles one hop
apart share a cell, so the coarse tier could not see them exchange at all, and the whole
predictor had nothing to read; the junction's arms are now as long as the cell. Second, a
bound pair turned once about itself changes the sign while crossing no face, so a whole
predictor with a label alone is wrong half the time; the object the tier above carries has
one bit of memory as well, which is the composite's spin in the literature's terms. The
stake that the parts' coarse view does not fix the sign is now an explicit pair of
histories (S4), not an assertion about a predictor. The lattice grew from 6 to 12 so that
cells of 3 leave four cells a side.

## What this does not claim

- **Not a lepton.** A `ℤ/2` charge bound to a `ℤ/2` flux. No electric charge, no spin
  one-half as a representation of rotations, no generation, no mass.
- **Not chirality.** Nothing here is left-handed. The wall of `LEPTON_LADDER.md` §1 stands
  exactly where it stood.
- **Not a discovery.** Every sign is Kitaev's and Levin and Wen's. The campaign re-finds.
- **Not a continuum statement.** One lattice size, no limit.
- **Not the Standard Model's fermions from a vacuum.** Whether a chiral fermion with the
  electron's quantum numbers can emerge from any bosonic model is open in the literature,
  and nothing here bears on it.

---
witness: `CIRISHolon.SceneCharge.gpop_total` (S1)
witness: `CIRISHolon.SceneCharge.no_lone_charge` (S2)
witness: `CIRISHolon.SceneCharge.vacuum_can_pair` (S2)
witness: `CIRISHolon.Ladder.outflow_commutes` (S1, on counts; the charge form is measured)
witness: `CIRISHolon.Object.NonFactoring` (S4)
witness: none (S3, S5 and S6 are measured; no gate here is a Lean theorem of its own)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-LOOP-BLIND, M-PARITY-PROTECT, M-BARE-CHARGE, M-VOLUME-SCALE, M-VACUOUS-SUCCESS, M-BASE-RATE-OMITTED, M-FOREIGN-DOMAIN-CORROBORATION, M-HOMOG, M-PLACEMENT-LOTTERY, M-CHEAPER-THAN-ITS-PRICE, M-STALE-INSTRUMENT. These are the registered ids this text contacts.

- M-PLANT-OBS: every plant is built for this instrument and runs before a stake is read.
- M-LOOP-BLIND: the signs are read from closed processes, and P-OPEN is the plant for one that does not close.
- M-PARITY-PROTECT: S2 is a parity statement, and P-ONE flips the one bit parity forbids.
- M-BARE-CHARGE: S2 is the bare-charge annihilation read as a count; the dressed pair is what survives.
- M-VOLUME-SCALE: one lattice size, `L = 12`. Nothing is claimed about any other.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous" names the ways a pass could mean nothing.
- M-BASE-RATE-OMITTED: all three coarse predictors' scores are printed side by side, with the count of each kind of process.
- M-FOREIGN-DOMAIN-CORROBORATION: a pass here exercises the tableau tier and four small predictors. It is not evidence about any lepton route, the chemistry tier's electron, or the cosmological rung.
- M-HOMOG: "cell" means a `c × c` block of this torus. Nothing is claimed about spatial structure elsewhere.
- M-PLACEMENT-LOTTERY: CPU work is pinned to one core class and no wall clock is compared.
- M-CHEAPER-THAN-ITS-PRICE: no cost figure is claimed before the first 10 circuits are timed.
- M-STALE-INSTRUMENT: stim lives in a session-keyed scratch environment; its version is recorded here and the generator, reader and every circuit's outcome are committed.
