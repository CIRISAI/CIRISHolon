# MIXTURES-1 referee

The 50-digit Python reference for the frozen campaign in `../MIXTURES1_PREREG.md`.
It owns the referee halves of R1 (second-row atoms), R2 (the staked exact pairs),
E1 (the emergent negatives) and E2 (the ordering), and it produces the D1
bridge's reference values.

It is a THIN LAYER on the ELEMENTS-1 referee, not a second implementation. The
integrals, the three FCI routes and the Temple certificates are that code, whose
arithmetic is already graded against the banked H2 referee; a second copy would
be a second thing that can drift. What lives here is this campaign's declared
model, its staked design, and the bindings that keep the two campaigns apart.

## What is here

| file | |
|---|---|
| `basis2.py` | MODEL DEFINITION: the STO-3G table for Z = 1..18. The first row is imported from ELEMENTS-1 by reference; the second row is 24 new exponents. |
| `species2.py` | the staked pairs, and the grid RULE — a function of one declared number per pair, so a grid can be regenerated rather than trusted |
| `m1core.py` | the two bindings (table, cache) and the three tag namespaces, checked at import |
| `build_atoms2.py` | gate R1: H..Ar, every Sz sector, dual route, ground spin derived twice |
| `test_basis_matches_engine.py` | the referee's model and the engine's, compared by PARSING `elements.rs` |
| `_cost_probe.py`, `_sio_stream.py` | what one geometry of each pair actually costs, measured |
| `_conditioning.py` | can the referee compute on the grid the rule produced |
| `env.sh` | one BLAS thread per process — source before any Python here |

## Three things about this lane that are not obvious

**The second row needs no d-orbital integrals, and adding them would have been
wrong.** The brief said they were needed. The frozen prereg proves otherwise on
its own: it states Ar2 is ONE determinant, which is true only if argon carries 9
basis functions — 1s 2s 2p 3s 3p, s and p and no more — since 36 electrons in 18
spatial orbitals leaves C(18,18)² = 1. Its Na2 figure agrees: about 1e9 is
C(18,11)² = 1.013e9, again 18 orbitals. The engine agrees a third time, in the
`second_row!` macro. So the second row is reached with the l ≤ 1 integrals
ELEMENTS-1 already validated on F2 and Ne2. Adding d functions "to be safe"
would not have been a safe superset — it would have been a DIFFERENT MODEL from
the engine's, and every R1 and R2 comparison would have failed in a way that
looked like an integral bug.

**The scope has an edge, and the edge raises.** `elements.rs` grew past argon
while this lane's first pass was being written — it now declares up to xenon,
most of those with a d shell. Na..Ar is untouched and still agrees exactly. But
this referee's declared model stops at argon, so for anything above it there IS
no referee, and a coverage gate reading "the engine matches the referee" would
be VACUOUSLY true for a pair it cannot grade. `basis2.shells_for` raises on an
out-of-scope Z rather than returning a smaller basis, because a smaller basis
does not fail: it reports a higher energy that looks perfectly converged.

**The two campaigns cannot read each other's cache, deliberately.** The basis
fingerprint hashes the whole table, so the Z = 1..18 table fingerprints
differently from ELEMENTS-1's Z = 1..10 one and a record under either is refused
by the other. That is right rather than inconvenient — they declare different
models — and it turns an inconvenience into a free check: the first-row atoms
must be recomputed here, and if they come back bit-identical then the table
extension provably touched nothing the first row uses. `build_atoms2.py` makes
that comparison and puts the result in the artifact.

## The staked design, and what is blind to what

Each pair declares ONE number, `R_ref`, the tabulated experimental separation.
It is CONTEXT under M-ONE-MODEL-DELTA and its only job is to set a grid's scale.
Both windows are computed from it by a fixed rule whose multipliers were fixed by
reading what ELEMENTS-1 had already staked across its seven bound species —
so the rule is calibrated on a CLOSED campaign's design choices, not on this
campaign's results. No energy this campaign computes enters a grid. Where a
minimum falls, how deep it is, and whether there is one at all are decided
afterwards, which is the whole content of gate E1.

Density is a compute decision and says so. The response to an expensive pair is
FEWER points, never cheaper ones: every emitted point is the same exact-in-model
full CI at the same 50 digits, dual-route and certified.

## Measured, not estimated

The determinant count is the number the prereg's feasibility map is stated in,
and for the mixed second row it understates the spread by an order of magnitude.
The cost is the number of NONZERO Hamiltonian elements, because the
working-precision matvec walks every one of them and `hp_cache` is off above
3000 determinants so nothing is stored.

| pair | ndet | nnz | notes |
|---|---|---|---|
| Ar2, NeAr | 1 | 1 | closed shells; the whole cost is integrals |
| HCl | 100 | 1.0e4 | |
| ClF | 196 | 3.8e4 | |
| Cl2 | 324 | 1.0e5 | |
| S2 | 23,409 | 3.1e7 | heavy, ordinary |
| NaH | 44,100 | 3.6e7 | heavy, ordinary |
| SiO | 132,496 | 2.0e8 | 22x N2's nonzeros — see the feasibility note |

The estimator is calibrated rather than trusted: it reproduces N2's 8,784,000
and HCl's 10,000 exactly. N2's measured working-precision matvec is 78.69 s for
8.78e6 nonzeros, which is the 9.0 us/element the projections above use.
