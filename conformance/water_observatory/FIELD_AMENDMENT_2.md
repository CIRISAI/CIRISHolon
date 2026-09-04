# FIELD-1 — AMENDMENT 2: G1 moves to the open box, because under walls the drift bound is 20 hartree and the ledger gate cannot fire on its own plant

*Frozen 2026-09-04, committed alone, after G1 was RUN under walls and BEFORE plant (ii) was
read anywhere it could fire. The freeze staked G1 — "the ledger closes with the field on" —
on a four-water scene UNDER WALLS. Run as staked, the gate reads closed (drift/bound 0.000)
and plant (ii), the unposted transition, does NOT fire: the transition moved the ledger by
`8.16e-4` hartree against a drift bound of `20.15` hartree. The bound is the integrator's own
error envelope, and with the walls engaged it is set by the wall stiffness (`K_WALL / m_H`),
not by the field or the bonds; under it the ledger gate cannot fail on any term. The engine's
own conformance rule says what that is: a gate that cannot fire on a plant is refused
(`OBJECT.md`, conformance obligations). G1-under-walls is therefore VOID as a gate — reported
here with its numbers, never read as a pass — and G1 is re-staked on the open box, where the
same scene's bound is the pair curvature's and the plant has room to fire.*

misfits: contacts **M-VACUOUS-SUCCESS** (the amendment exists because the gate passed
without being able to fail); **M-PLANT-OBS** and **M-PLANT-SECTOR** (plant (ii)'s carrier —
a transition occurred, nonzero in the sector the plant acts on — was satisfied; the gate,
not the carrier, was blind); **M-STALE-INSTRUMENT** (alone, before the re-staked gate is
read); **M-EXIT-DISCRIMINATOR** (the vacuous run is its own named outcome);
**M-NULL-MISSTAKE** (the ledger gate is still staked on `E − W_ext`; only its scene moves);
**M-BARE-CHARGE** (classical charges, as before). Not contacted: the rest of the registry.

## A2.1 The re-staked gate

- **G1′ — the ledger closes with the field on, OPEN BOX.** The four-water scene of the freeze
  under `Boundary::Open`, 2,000 steps after the field is enabled: `drift() ≤ drift_bound()`
  at every grain boundary AND `work_columns_ok()`, with the enabling transition posted to
  `work.field`. 1 scene. The plant (ii) reading moves with it: the unposted transition must
  open the gate here. Carrier unchanged.
  witness: none (an engine ledger gate)

## A2.2 What is kept

G1 under walls: closed, `drift/bound = 0.000`, bound `20.15` hartree — VOID as a gate,
kept in the record with the reason. G2 (momentum, already staked on the open box), G3
(the derivative), G0, G4, G5: unchanged and read. S1 stays under walls: it is a
measurement of a population, not a conservation gate, and the walls are the scene the
freeze chose for it.
