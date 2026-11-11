# FERMION-0 — READ: branch (a) as amended. A fermion is not an input on this substrate; what the tier above must carry beyond the parts' counts and fluxes is one bit per bound pair, on the relationship. A re-finding, as staked.

*2026-11-11. Prereg `FERMION0_PREREG.md` (frozen alone, `7d8734b`), Amendment 1 (`9b8a40f`:
the parts predictor as frozen was a strawman), Amendment 2 (`9d00b20`, wording `4c9edfd`:
P-fine omitted enclosed particles). Instrument `fermion0/fermion0.py`, committed before the
read (`ce21318`). Records `fermion0/fermion0.log`, `fermion0/outcomes.tsv`. Carrier: the
toric code on a 12 × 12 torus, 288 link qubits, up to 1,090 qubits a circuit; the engine's
tableau tier with stim 1.16.0 as referee. Every sign here is Kitaev's (2003) and Levin and
Wen's (2003). Nothing below is a claim about a lepton.*

## 0. Verdict

| gate | staked | read | verdict |
|---|---|---|---|
| **G0** the vacuum | 288 stabilizers and 2 loops read `+1`, determined | 290 of 290 | **MET** |
| **S1** Gauss as the outflow law | interior product = boundary loop, 4,000 of 4,000 | 4,000 of 4,000, and each equals the tracked count inside | **MET** |
| **S2** no lone charge; the vacuum can pair | 0 odd counts in 2,000; at least 1,800 with a pair | 0 odd; 2,000 with a pair | **MET** |
| **S3** the statistics | `e–e` `+1`, `m–m` `+1`, `ε–ε` `−1`, `e` around `m` `−1`, twist `−1`; 500 of 500, none refused | 100 of 100 each, 0 refused | **MET** |
| **S4** the parts' coarse counts do not fix the sign | 500 of 500 pairs with identical coarse histories and opposite signs, at `c = 2` and `c = 3` | 500 of 500 identical and 500 of 500 opposite, at each | **MET** |
| **S5** (amended) one bit per pair is what is missing | P-whole 1,000; P-parts ≤ 550; P-parts = P-whole-bare on 1,000 | P-whole 1,000, P-parts 500, P-whole-bare 500, equal on 1,000, at each `c` | **MET** |
| **S6** the parts suffice at lattice resolution | P-fine right on 2,000 of 2,000 | 2,000 of 2,000 | **MET** |

**Plants.** P-ORDER fires (the engine prints its register last bit first; the reader named
the flipped bit). P-TRIVIAL fires: on a product state with no gauge structure, 0 signs
reported and 500 of 500 refused. P-OPEN fires: 50 of 50 unclosed words refused. P-ONE fires.
P-LABEL fires: with the label read as "boson", P-whole is right on 500 of 1,000. P-REFEREE
fires: the engine and stim agree on 6,061 of 6,061 circuits, 550 of which had an
undetermined ancilla and were read by Amendment 1 §4.

**Branch (a), in Amendment 1's words.** On this substrate a fermion is a closed view of a
vacuum that holds none, and it cannot be made alone. At an acuity that does not resolve its
binding, the conserved counts and fluxes of its parts give the exchange sign up to ONE BIT
per bound pair. That bit sits on the relationship between the two parts and on neither. The
label "fermion" is not something the tier above has to be told: it follows from the parts.

## 1. What is informative here and what is arithmetic

- **S4 is the measurement.** Two fine histories that agree, step by step and cell by cell,
  in every count of every part, and differ in a measured sign. That is `Object.NonFactoring`
  exhibited 1,000 times on an exact substrate: the coarse view of the conserved counts does
  not factor the quantity.
- **S5's scores are exact by design.** The four kinds of process are balanced 250 each, a
  predictor blind to the twist is wrong on exactly the twisted half, so `500` is arithmetic
  and not a statistic. What S5 adds to S4 is that one bit per pair is ENOUGH (P-whole, 1,000
  of 1,000), not merely that something is missing.
- **P-parts = P-whole-bare is true by derivation, and the reader computes both from the same
  permutation.** It is reported because the freeze had it wrong: the freeze staked that the
  whole needs a label the parts cannot give. They can.
- **S6 is close to a tautology,** as the freeze said: at lattice resolution the parts' paths
  are the operator word. It marks the acuity at which nothing is hidden.

## 2. What it means in the record's own vocabulary

- The bit is an attribute of an EDGE (the binding of an `e` to its `m`), owned by the lowest
  holon holding both ends. `ScenePopulation.edge_law_does_not_fix_wiring` says counts on
  edges do not fix the wiring; this is a measured case where the wiring's STATE is what the
  level above reads.
- The bit is memory, not a conserved quantity: it changes when the pair turns, with no
  count or flux changing anywhere. In the literature it is the composite's topological spin.
- `SceneCharge.vacuum_can_pair` and `no_lone_charge` held on every one of 2,000 states: the
  floor of the ladder is a floor for counts and not for charges.
- For the calibrated table in `OBJECT.md` (A′ against C), this is one exact substrate on
  which conserved counts and their fluxes are NOT the whole object at coarse acuity, by one
  bit. It is known physics re-found, and no weight is moved here; the table is the lead's.

## 3. Deviations, said plainly

1. **Two amendments between the freeze and the build,** both the author's own errors in the
   freeze, both committed alone before any circuit existed. Without the first, S5 would have
   passed by construction; without the second, S6 would have been killed by an omission in
   its own rule.
2. **The per-circuit record was not written at the first read.** The freeze says every
   circuit's outcome is committed. The first run printed only the summary. The outcome
   record (`outcomes.tsv`: per circuit its digest, its sign ancilla, the digest of every
   ancilla's reading, the number undetermined, and whether the engine agreed) was added
   afterwards and the campaign re-run from the same seeds; the summary lines of the two runs
   are identical apart from the timing line. Both logs are kept (`fermion0.read1.log`,
   `fermion0.log`).
3. **Four dead lines in the instrument as committed before the read** (a first P-ORDER probe
   whose result is never used); removed in the commit that adds the outcome record.
4. **Circuits are larger than the freeze's 577 qubits in two places:** G0 uses 579 (two loop
   ancillas) and S1 uses 1,088 (800 region ancillas on one state). Neither changes a stake.
5. **The memory-bit rule recognises two excursions:** one that encloses the partner's
   plaquette and one that encloses nothing. Any other raises an error; none occurred.
6. **S3's twist placements can put the second `ε` on the twisting path.** The operator is
   the same plaquette operator and the sign is the same; the picture of "one `e` circling
   one `m`" is exact only when the neighbour is elsewhere, as it is in S4 and S5.

## 4. What this does not say

Not a lepton, not chirality, not a mass, not a continuum limit, not a discovery, and not
evidence about any route in `LEPTON_LADDER.md`. `LEPTON_LADDER.md` §4's sentence ("the
electron is a TERMINAL") stands for the electron. What no longer stands as a general
statement is that a fermion must be an input: on one exact substrate this engine hosts, it
is not.

## 5. Next, named and not frozen

The fermion as a winding number in 1+1 dimensions on the Schwinger instrument; the axial
anomaly there (`LEPTON_LADDER.md` route R6); a handed mode on a wall in 2+1 dimensions with
the stake that the charge missing on the wall equals the flow in from the bulk.
