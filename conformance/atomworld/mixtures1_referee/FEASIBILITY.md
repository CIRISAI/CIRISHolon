# What the staked pairs actually cost — measured, and what it does to gate D1

Written before any curve exists, because the answer decides whether one of the
staked pairs is a grid or a campaign.

## The quantity that matters is not the determinant count

The prereg's feasibility map is stated in determinants, which is the right unit
for "is this space enumerable" and the wrong one for "what will this cost". The
cost is the number of NONZERO Hamiltonian elements: `hp_cache` is off above 3000
determinants, so the working-precision matvec re-walks every one of them on
every iteration and stores none.

The estimator below is calibrated, not trusted. It predicts N2's nonzeros as
8,784,000 against the 8.8e6 the crate's own comment records, HCl's as 10,000
against 10,000 measured, and SiO's as 196,889,056 against **196,889,056
measured**. Three exact hits, so the projections built on it are as good as the
timings.

| pair | nbf | ndet | nonzeros | AO integrals | one working-precision matvec |
|---|---|---|---|---|---|
| Ar2 | 18 | 1 | 1 | 152 s | 0.00 s |
| NeAr | 14 | 1 | 1 | 59 s | 0.00 s |
| HCl | 10 | 100 | 10,000 | 15 s | 0.03 s |
| ClF | 14 | 196 | 38,416 | 58 s | 0.10 s |
| Cl2 | 18 | 324 | 104,976 | 154 s | 0.27 s |
| S2 | 18 | 23,409 | 31,110,561 | ~154 s | ~380 s |
| NaH | 10 | 44,100 | 35,500,500 | ~15 s | ~430 s |
| **SiO** | 14 | 132,496 | **196,889,056** | 65 s | **2378 s** |
| (N2, the calibration) | 10 | 14,400 | 8,784,000 | 19 s | 78.69 s |

SiO's number is a direct measurement, not a projection: `_sio_stream.py` walks
the generator doing exactly the multiply-add the matvec does, storing nothing,
so it reads the true nonzero count and the true per-element rate at flat memory
(0.10 GB) on a machine that could not have held the matrix.

## What that means for SiO

12.08 µs per element. One route-A working-precision matvec is **39.6 minutes**.

N2's real dual-route grid points, from its own cache records, took 2354 s to
10893 s, and its average matvec is 93.6 s across the two routes — so a certified
point costs **25 to 116 matvecs**. At SiO's 2830 s average that is **20 to 91
hours per geometry**, times 15 staked knots, and the f64 CSR alone needs about
5.6 GB at peak per worker on a 31 GB machine shared with four campaigns, which
caps concurrency at four. The grid ALONE is 3 to 14 days of wall time, before
the spin audit, the stencils, the Hermite probes or the beyond-grid probes.

Call it 1000 to 4000 core-hours. That is a campaign, not a stage.

## Two remedies tested, both negative

**Hoisting the generator's redundant work** (`_fast_elements.py`). The generator
rebuilds, for every (alpha, beta) string pair, four things that depend on only
one of them — including one `_excite` call per nonzero in the mixed
alpha/beta block, which is 73% of SiO's work. Hoisting all four gives a stream
verified IDENTICAL element for element and value for value on H2, HCl and ClF
— and a speedup of 1.34x, 1.19x and **0.93x**. The excitation bookkeeping is not
the cost; the cost is yielding 2e8 Python tuples and doing 2e8 mpf multiply-adds,
and no rearrangement inside this design touches that.

**Route B**, the spin-summed generator formulation, which contracts over string
couplings rather than enumerating nonzeros and might therefore scale
differently. Measured B/A on the same operators: 17.3x at ndet 100, 45.2x at
324, 1.38x at 14,400. It closes with size but never wins.

So the cost is structural. Making SiO affordable needs a different ALGORITHM —
a string-driven sigma that never enumerates a matrix element — which is a
rewrite of the arithmetic that produces every referee number in two campaigns.

## The consequence for gate D1, stated as a choice and not taken

D1 stakes S2 **and** SiO as the two species where exact FCI and q8-mps overlap.
S2 is fine: 3.1e7 nonzeros, about 4x N2, ordinary. SiO is the one that does not
fit.

- **(a) Run it as staked.** Weeks on a dedicated allocation.
- **(b) Rewrite the sigma.** Real speedup available; real risk to validated
  arithmetic, mid-campaign, in two live lanes.
- **(c) Amend D1's second overlap species.** **NaH** is the natural
  substitute: 44,100 determinants, the LARGEST feasible space in the staked set
  — larger than S2's 23,409 — at 3.6e7 nonzeros and roughly 430 s per matvec, so
  3 to 14 hours a geometry and 15 knots in days. Its referee table is being
  built anyway for R2, so D1 would get its second overlap species at zero extra
  cost, and SiO would stay owed in R2 rather than blocking the bridge.

A frozen stake is not this lane's to change. The measurement is the deliverable;
the choice is the lead's, and if it is (c) it is a prereg amendment and should
be recorded as one.
