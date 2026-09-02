# What the dE3 subtraction costs, measured before the emitter bakes

Instrument: `holon-chem/examples/s3_pair_cost.rs`. Value-only pair solves (constant centres,
no dual propagation), because the emitter is fenced from emitting derivatives of the
subtracted quantity until `Surface::subtract` can carry them — so there is nothing to spend
duals on.

## The reading, 3 reps, loadavg 70.7 at start / 66.7 at end

| pair | min ms | med ms | max ms | spread | dE2 (Ha) |
|---|---:|---:|---:|---:|---:|
| H-H | 18.8 | 134.5 | 189.0 | 10.04× | −0.20414235 |
| O-H | 74.3 | 103.5 | 165.1 | 2.22× | −0.12290107 |
| H-Cl | 277.7 | 279.0 | 306.4 | 1.10× | −0.14829318 |
| O-O | 397.8 | 413.1 | 432.3 | 1.09× | −0.14762114 |
| **Cl-Cl** | **1903.0** | **23598.2** | **43620.1** | **22.92×** | −0.06457738 |

**Read the minima, not the medians**, and read nothing here as a floor: this is a loaded box
and the conditions are stamped at both ends per the fleet standard. Cl-Cl's 22.9× spread is
its own finding and is unexplained — the other four pairs sit between 1.09× and 10×.

## How this instrument started wrong, kept because it is the campaign's own lesson

The first version quoted **one wall-clock timing per pair** and reported H-Cl at 19,970 ms.
The very next run of the same call read **216 ms** — 93× apart. A single timing on a
contended box is a DRAW, not a measurement. That is this campaign's most-repeated lesson,
walked into by the lane that has spent the day applying it to other people's numbers, inside
an hour of doing so. The repaired instrument takes repeats, reports min/med/max and spread,
and stamps loadavg at both ends.

## The design consequence: the subtraction is NOT free, and its cost is table-dependent

Per node the emitter needs three pair energies. Two are **axis-separable** and cache to
`nx + ny` solves for a whole table; only the third side `r_23 = sqrt(x² + y² − 2xyu)` varies
with all three indices and costs one solve per node.

| table | trimer/node (G0) | cached apex legs | per-node third side | subtraction vs trimer |
|---|---|---|---|---|
| (O,O,O) | 39.8 s | O-O, ~0.4 s × (nx+ny) | O-O ~0.4 s | **~1%, negligible** |
| (H,H,Cl) | 0.21 s | H-Cl, ~0.28 s × (nx+ny) | H-H 0.019–0.19 s | **~9–90%, comparable** |
| (Cl,Cl,Cl) | 6.5 s | Cl-Cl, 1.9–43 s × (nx+ny) | **Cl-Cl 1.9–43 s** | **30%–7×, DOMINANT** |

The pattern follows water's own G0 split: pair cost scales with **basis size**, and the
tables whose trimer is basis-bound rather than determinant-bound are exactly the ones where
a diatomic is a large fraction of the trimer. (Cl,Cl,Cl) is the bad case — G0 measured it
73% assemble — and it is the one where the third-side leg cannot be cached away.

**The axis cache is therefore not an optimisation, it is a requirement**: without it the cost
is three per-node solves instead of one plus two cached axes, which triples the worst case.

## Owed before this is acted on

* **Cl-Cl's 22.9× spread needs explaining**, not averaging. Until it is, any (Cl,Cl,Cl)
  budget built on these numbers inherits a factor of 23.
* These are minima on a loaded box, not quiet-box costs. Per the fleet standard they may not
  serve as a baseline without their conditions, which are recorded above.
* No decision is taken here. The finding is that **the dE3 ruling has a compute price that
  varies by two orders across the four tables**, and the lead should see it before the
  emitter bakes it in.
