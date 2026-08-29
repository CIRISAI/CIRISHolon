# ELEMENTS-1 referee

The 50-digit Python reference implementation for the frozen campaign in
`../ELEMENTS1_PREREG.md`. Exact-in-model full CI in the declared STO-3G minimal
basis, s and p shells, McMurchie–Davidson integrals, no chemistry library.

The published product of this lane is the drop the engine's R2 gate reads:
`engine/crates/holon-chem/tests/data/elements1/`. This directory is the source
that produced it, committed so the drop can be re-derived rather than trusted.

## Run it

    python3 verify_elements.py          # full: recomputes from scratch where it can
    python3 verify_elements.py --quick  # skips the slow from-scratch legs

Exit status is nonzero on any failure. It needs `elements_potential*.json` and
`elements_atoms.json` (both here) and imports the banked `h2_core.py` from the
parent directory — deliberately, because two copies of a bank is how a bank
stops being one.

    bash run_final.sh                   # tests, guards, spin audit, assemble, emit, verify

## What is here

| file | |
|---|---|
| `elements_core.py` | integrals: Hermite E/R recursions, Boys, shells; normalisation kept OUTSIDE the stored coefficients so the s-only path matches `h2_core` term for term |
| `fci.py` | three independent FCI routes (Slater–Condon, spin-summed generators in a rotated basis, Fock-space ladders) and the certified eigensolve with Temple bounds |
| `curve.py` | exact-decimal grids, 8th-order stencils, Newton minima, cubic Hermite and the envelope |
| `species.py` | the staked nine, the atoms, and `sparse_subset` — the subset rule for N2 and CO, in staked parameters only |
| `runner.py` | the point cache, keyed by exact geometry string and stamped with the basis fingerprint |
| `build_curves.py` | the stages, the run lock, and the pool guard |
| `emit_engine.py` | the drop, with every refusal that keeps a misleading file from being written |
| `verify_elements.py` | the standalone re-check (V1–V10) |
| `plants.py` | the two mutation plants and the empty-sector control |
| `test_*.py` | the suites, including one test per refusal and per guard |

## Two things about this code that are not obvious

**Every guard here has a demonstrated failing case.** `test_pmap_safety.py`
first proves the unguarded pool HANGS, then that the guarded one raises;
`test_emit_refusals.py` fires all nine emitter refusals on purpose;
`test_runlock.py` fires the duplicate-run refusal against a live process. A
check that has never fired is indistinguishable from a check that cannot, and
this campaign shipped one of those: the pool guard was correct, tested, and
called by nothing for its entire life.

**The audits are part of the product.** `_dead_guard_audit.py` lists every
function defined against every function referenced, and V9 fails on anything
reachable only from its own test. `_inert_audit.py` splits every emitted key
into read / guarded-at-write / inert, and V9 fails on any inert field not named
in `prose_fields.txt`. The allowlist is a separate file because naming a field
inside the verifier made the audit think it was consumed.

## A limitation, stated rather than left to be discovered

Cache records are stamped with the BASIS fingerprint and refused when it
changes. They are not stamped with a fingerprint of the SOLVER, and the solver
did change during this campaign (cluster deflation for near-degenerate levels,
a gated stall-stop, a one-determinant space returning its vector). So a curve
can be assembled from records computed by different versions of `fci.py`.

What makes that safe rather than merely unnoticed, and where the argument stops:

* every record was checked by TWO independent routes at the moment it was
  written, and the third route agrees to 5e-48 or better wherever it ran;
* `--recertify` re-runs any geometry whose stored bound does not cover the
  50th significant digit, which is what caught a stale over-optimistic bound of
  1.30e-17 on HF;
* the emitter refuses a file whose declared uncertainty does not cover the
  digits it prints;
* V2 checks H2 against the banked `h2_core.py` run LIVE, so a solver change
  that moved both routes together would have to move the bank too.

The gap that leaves is a change making both routes wrong in the same direction
by less than the declared bound. Two independent implementations make that
unlikely; nothing here makes it impossible. A solver fingerprint on each record
would close it and would also invalidate about 45 MB of exact solves, so it is
recorded as a known limitation rather than paid for mid-campaign.

## Not committed

The point cache (about 45 MB, tens of thousands of CPU-hours of exact solves)
lives outside the repo. Every record is regenerable from this source, and the
cache is keyed by the exact geometry string and the basis fingerprint, so a
stale-basis record is refused rather than reused.
