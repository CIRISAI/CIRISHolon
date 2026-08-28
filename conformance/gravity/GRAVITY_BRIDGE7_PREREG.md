# Pre-registration — GRAVITY-BRIDGE-7: the live carrier and the pendant plaquette

*Frozen 2026-08-28, committed ALONE before its instrument, and the FIRST
freeze required to pass `Audit/prereg_audit.py` (CI gate 32). Successor to
BRIDGE-6 (VOID with G0/R1/B3 earned). Two owed claims, one campaign.*

misfits: contacts M-BARE-CHARGE (the pump conditions on the Wilson-dressed
channel, invariance re-verified as G0), M-PLANT-OBS and M-PLANT-SECTOR
(every plant below states its carrier and the sector it must be nonzero
in), M-HOMOG (R3 is re-posed on a structurally inhomogeneous graph, not a
term substitution), M-LOOP-BLIND (R2's distant reading is the loop-class
set, the same observable BRIDGE-6 validated as movable), M-GAUGE-LAUNDER
and M-PARITY-PROTECT (the pendant plaquette's holonomy content differs by
CONSTRUCTION, not by a parity-sector choice), M-COND-PROBE (all dynamics
remain inside T, inherited from BRIDGE-6's R1 which stays a gate here).

## The two repairs

**R2' — charged sourcing, on a LIVE off-channel carrier.** BRIDGE-6's R2
fired because its off-channel state was built from a seed Gauss
annihilates. The diagnostic showed the off-channel sector is LARGE (~860k
of ~982k amplitudes). Here the carrier is constructed correctly: take a
generic Gauss-projected state, project OUT the dressed channel, and the
instrument asserts the result is nonzero IN THE OFF-CHANNEL SECTOR before
any gate is scored.
- **R2' gate**: the loop-class sets of the dressed carrier and the
  off-channel carrier must DIFFER within 3 steps of T (EXACT set
  comparison, both graphs). witness: none (measured gate; no theorem
  staked — the staked theorem-level fact is G0's invariance)

**R3' — spatial locality, on a structurally inhomogeneous graph.** BRIDGE-6
measured near == far EXACTLY and diagnosed the fan disk's parity
homogeneity as robust to term substitution (M-HOMOG). So the graph changes:
the PENDANT graph is the base fan disk PLUS a pendant plaquette attached at
the rim by a bridge edge — its holonomy is built from edges the fan's
plaquettes do not touch, so its class content differs from p0's by
construction, and "distant" is now a structural fact rather than a
symmetry hope.
- **R3' gate**: conditioning the in-step geometry term on p0 must MOVE the
  matter channel reading (near ≠ base, EXACT integers), and conditioning
  the same term on the pendant plaquette must leave it EXACTLY unchanged
  (far == base). witness: none (measured gate)
- **R3' honesty clause**: if far ≠ base too, the claim "locality" dies and
  is kept marked; if near == base, the geometry term is inert on this graph
  and R3' VOIDs (cannot pose).

## Carried gates

- **G0**: the dressed projector commutes with joint Gauss on the pendant
  graph too, checked FIRST; nothing else is reported if it fires. EXACT.
  witness: none (instrument-checked invariance; the construction's
  derivation is in BRIDGE6's prereg)
- **R1**: the channel reading moves under iterating T alone — re-asserted
  on the pendant graph. EXACT (set of ≥ 2 distinct values in 3 steps).
  witness: none (measured gate)
- **B3**: joint Gauss holds on every registry state at every step. EXACT.
  witness: none (measured gate)

## plants (each with its carrier and sector, per M-PLANT-SECTOR)

- **(i) wrong-side gauge action** must fire B3. Carrier: the dressed
  vacuum; sector: total amplitude, asserted nonzero before scoring.
- **(ii) non-central pump (L_r for central r²)** must fire B3 at e*'s
  endpoints. Carrier: the OFF-CHANNEL state (the pump acts only there);
  sector: the off-channel component, asserted nonzero IN THAT SECTOR
  before scoring — the exact assertion whose absence voided BRIDGE-6.
- A missed plant VOIDs the campaign.

## Meaning

All gates → "charged matter (Wilson-dressed, gauge-invariant) sources
geometry endogenously, and the geometry term's influence on matter is
spatially local: a structurally distant plaquette's condition leaves the
matter reading exactly unchanged." Fires → the named claim dies, kept
marked. Still a finite-group BF toy; SU(2)-via-2T and the GR phase-space
channel remain the successors, unclaimed.
