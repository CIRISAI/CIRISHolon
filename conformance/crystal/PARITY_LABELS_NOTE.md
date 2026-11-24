# Two-bit labels: "a holon is 00, a fermion is 01 and a baryon is 11" — one precise reading, recorded and NOT yet confirmed

*2026-11-23. A note: no claim, no compute, no freeze. The lead said the sentence in the
title and asked "does that make sense?". The lead's agent offered ONE precise reading, set
out below. The lead answered "Yes, that makes sense" to the message that carried it, which
also carried another proposal; whether THIS is the reading the lead had in mind has not been
confirmed, and the note ends with what needs confirming. Statements about the Standard Model
are from memory of textbook material and were not checked at a source.*

## The reading: (baryon parity, fermion parity)

Take the two bits to be, in the order written: the parity of baryon number, then the parity
of fermion number.

In the Standard Model the two are tied to lepton number. Every field carries
`(−1)^F = (−1)^{3B + L}`: quarks have `B = 1/3`, `L = 0` and are fermions; leptons have
`B = 0`, `L = 1` and are fermions; gauge and Higgs bosons have `B = L = 0`. For a
colour-singlet state `B` is an integer and `3B` has the parity of `B`, so

> a state is a fermion exactly when `B + L` is odd.

Caveats on that sentence: it rests on the Standard Model's assignments of `B` and `L` to its
fields; it is a statement about parities, so it survives processes that change `B` or `L` by
even amounts (a Majorana neutrino mass changes `L` by two) and it is the same relation that
defines matter parity, `(−1)^{3(B − L)}`, in extensions; it says nothing about a
hypothetical particle with other assignments.

With that, the four classes, and what the third bit one could have written (lepton parity)
must be:

| code | baryon parity | fermion? | lepton parity (forced) | examples |
|---|---|---|---|---|
| `00` | even | no | even | the vacuum, the photon, a pion, the deuteron, helium-4, an H₂ molecule |
| `01` | even | yes | odd | the electron, a neutrino |
| `11` | odd | yes | even | the proton, the neutron |
| `10` | odd | no | odd | the hydrogen atom |

- **Composition is XOR.** Both parities add modulo two when things are put together:
  `01 ⊕ 11 = 10` (electron and proton make hydrogen), `10 ⊕ 10 = 00` (two hydrogen atoms
  make H₂), `11 ⊕ 11 = 00` (proton and neutron make the deuteron).
- **The code the sentence did not name, `10`, is the atom:** a boson that carries baryon
  number, the first object of the tier above the nucleus and the electron.
- **`00` is what closes.** Everything in the `00` row is a composite whose two parities
  cancel, or the vacuum itself. That is the population law for charges already proved in
  `lean/CIRISHolon/SceneCharge.lean`: `vacuum_can_pair` (with a count that can cancel, an
  empty parent holds non-empty children: two odd children in ℤ/2), `no_lone_charge` (a
  charge under an empty parent has a partner under the same parent) and
  `odd_parent_has_odd_child` (parity is carried: a parent of odd parity has a child of odd
  parity). Each parity bit here is a count in ℤ/2 of exactly that kind, and "a holon is 00"
  then reads: a holon is a parent whose parities are both zero. That is a reading of the
  theorems, not a new one, and it is NOT a definition of a holon: plenty of closed views on
  this record carry a nonzero parity (a proton is `11` and is as closed as anything here).

## Where it does not match the record's own fermion

FERMION-0 (`conformance/crystal/FERMION0_RESULTS.md`, S3) read the toric code's four
sectors: the vacuum; the charge `e` and the flux `m`, each a boson (`e–e` `+1`, `m–m` `+1`),
with `e` around `m` giving `−1`; and their bound pair `ε`, a fermion (`ε–ε` `−1`). In two
bits, with charge parity first and flux parity second, that is vacuum `00`, `e` `10`, `m`
`01`, `ε` `11`.

Both labellings are the group ℤ₂ × ℤ₂ under XOR. They are not relabellings of one another.
In the parity reading above, "is a fermion" is the second bit: it is additive, and two of
the three nonzero classes are fermions (`01`, `11`) with their sum a boson (`10`). On the
toric code, "is a fermion" is the PRODUCT of the two bits: it is not additive, one of the
three nonzero classes is a fermion (`11`), and it is the sum of two bosons. No renaming of
the classes turns an additive rule with two fermions into a product rule with one. The
difference is physical: the toric code's sign comes from the mutual statistics of two
different bosons in two dimensions, which the Standard Model's particles in three do not
have.

So if the sentence meant the toric-code labels, the fermion is `11` and `01` is a boson. If
it meant the parity labels, FERMION-0's fermion is a different kind of `11` from a baryon.

## A second reading?

None is offered. "Bits = (static, organised)" was considered and cannot be made precise in
two sentences: on this record a tier's two portions are measured amounts, not parities, and
nothing makes a fermion "organised and not static".

## What the lead needs to confirm

1. Is (baryon parity, fermion parity) the reading meant, or the toric code's (charge, flux),
   or another?
2. If the first: is "a holon is 00" meant as "what closes has both parities zero" (the
   `SceneCharge` reading above), or as a definition? As a definition it would exclude the
   proton.
3. Whether `10` = the atom is wanted in `OBJECT.md` or on the ladder's captions. Nothing has
   been written anywhere but here.
