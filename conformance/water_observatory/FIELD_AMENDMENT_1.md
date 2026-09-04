# FIELD-1 — AMENDMENT 1: the charged unit is an oxygen with its two bonded hydrogens by the engine's own bond verdict, because the census carries PAIR rows

*Frozen 2026-09-04, committed alone, BEFORE any gate of FIELD-1 was read with a charge
assigned. The freeze defined the charged unit as "a live census row that is a water
molecule (three members: one oxygen, two hydrogens)". The engine's census has no such row:
`HolonLayer` forms rows from BONDED PAIRS only (`member_count = 2`, one row per atom, a
dwell of `DWELL_K = 3` grain boundaries and a closure-defect bar at formation), so a water
molecule is one O–H row and a second hydrogen the row cannot take. Under that definition
no atom was ever charged, `E_field` read an exact zero, and the first run of the gates was
VACUOUS (G1–G3 failed on their own carrier checks: no field force anywhere — the
M-VACUOUS-SUCCESS shape caught by the plants' carriers, as designed). This amendment
corrects the DEFINITION and nothing else.*

misfits: contacts **M-VACUOUS-SUCCESS** (the amendment exists because the carriers fired on
an empty field); **M-STALE-INSTRUMENT** (alone, before the corrected gates are read);
**M-EXIT-DISCRIMINATOR** (a flickering assignment is named as its own branch below);
**M-PLANT-OBS** and **M-PLANT-SECTOR** (the three plants' carrier sentences are unchanged;
each carrier is asserted nonzero in the sector the plant acts on, and this amendment is
what makes the sector non-empty); **M-BARE-CHARGE** (classical charges, as before); **M-COND-PROBE** ("inside the outer turning
point" contacts its keyword; the bond verdict is read from the pair list the force pass
already produced, not applied after a step). Not contacted: the rest of the registry.

## A1.1 The corrected definition

A **water unit** is an oxygen together with exactly the hydrogens the engine's own PAIR
BOND VERDICT bonds to it — the same criterion the page draws bonds with (`E_rel < 0` and
inside the outer turning point, read from the pair list, never a distance threshold of the
field's own) — when that count is exactly two and neither hydrogen is bonded to any other
oxygen. Its charges are `q_O = −2q`, `q_H = +q` with `q` as frozen (the dipole-exact charge
at EMBED-1's pin, G4 unchanged). Every other atom carries zero. Everything else in the
freeze — the term, the forces, the energy row, the transition posting to `work.field`,
the wrapped-box refusal, the gates, the plants, S1 — is unchanged in letter and in stake.
witness: none (a definitional amendment; the gates keep the freeze's own witnesses)

## A1.2 What the correction changes and what it does not

It changes which atoms are charged (from none to the water units) and therefore makes G1,
G2, G3 and the plants posable. It does not change any number already read: G0 (identity
and the pure-hydrogen zero), G4 (the charge) and G5 (the refusal) passed on the first run
and are unaffected, because none of them depends on the assignment.

**A named branch the census's dwell used to hide.** The pair verdict has no hysteresis of
its own, so a hydrogen at the bond threshold can enter and leave a water unit at frame
rate; each such change is a transition, posted, and the ledger stays closed by
construction — but a run whose `work.field` column is dominated by threshold flicker is
reported as such (transitions per frame written beside the energy), and FIELD-2 may adopt
the census's `DWELL_K` for the unit if the count is material. Not staked here; reported.
