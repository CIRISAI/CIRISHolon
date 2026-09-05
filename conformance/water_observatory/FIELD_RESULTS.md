# FIELD-1 — results

*Freeze `FIELD_PREREG.md` (0b6cb42, alone); amendments 1 (3f42eb9: the charged unit is the
engine's own bond verdict — the census carries pair rows), 2 (15415c2: G1 to the open box)
and 3 (648c800: the drift bound is the pair envelope, A2's reason corrected; plant (ii) as a
two-arm shift). Instrument `holon-render/src/field.rs` and the hooks in `sim.rs`,
`checkpoint.rs` (v6), `lib.rs`; gates `tests/field.rs`; S1 `examples/field_hbonds.rs` with
`examples/field_probe.rs` as its diagnostic.*

## The verdict, first

**The field is in the force law with its books closed: every gate green, every plant
fires.** The charge comes from the engine's own density (`q_H = 0.231380372`,
`q_O = −0.462760744`), the term's forces are its energy's derivative to `1.2e-9`, momentum
is conserved to `5e-14`, the ledger closes to `8e-6` hartree with the field's transitions
posted to their own column, and the wrapped box is refused with Ewald named. **S1 reads
BRANCH (b): with fixed dipole-exact charges, no hydrogen bond formed in either arm** — the
field as built pushes four waters apart at 293 K rather than binding them, and the next
freeze (FIELD-2) stakes what the fixed charges lack.

| gate | verdict | the number |
|---|---|---|
| G0 — identity, and pure hydrogen carries zero | **PASS** | checkpoint BYTES identical over 2,000 steps with the field enabled-then-disabled before the first step; `e_field = 0.0` exactly on a hydrogen scene at every step |
| G1 — the ledger closes with the field on (walls, as frozen) | **VOID as a gate, kept** (A2, A3) | `drift = 3.2e-6` against `drift_bound = 20.1` hartree: the O(h²) envelope cannot fail here (fence P20) |
| G1′ — the ledger closes, open box (A2) | **PASS** | `e_field = 1.74e-3` Ha after 2,000 steps, `work.field = −8.05e-4` (the enabling transition, posted), receipt columns summing to `w_ext`, honest `drift_peak = 8.3e-6` |
| G2 — momentum | **PASS** | internal forces sum to `5.6e-17` against a `6.4e-2` scale; residual `5.0e-14` under a `3.7e-10` bound |
| G3 — the force is the derivative | **PASS**, 12 atoms × 3 | worst `|F − (−∂E)| / |F| = 1.2e-9` (stake `1e-8`); `|F_field|` up to `1.2e-3` Ha/bohr |
| G4 — the charge is the record's | **PASS** | equal to `embed::fragment_charges` on the pinned monomer to `1e-12` |
| G5 — the wrapped box is refused | **PASS** | `FieldRefusal::PeriodicNeedsEwald`, state unchanged; the boundary door refuses wrapping while the field is on |
| plant (i) — the reaction dropped | **FIRES** | momentum residual over its bound; carrier `|F_field| ≥ 1e-6` |
| plant (ii) — the transition unposted (A3, two arms) | **FIRES** | plant arm `drift_peak = 8.17e-4` against the honest arm's `8.3e-6`, the transition `8.05e-4`; carrier one transition |
| plant (iii) — the sign | **FIRES** | the derivative disagrees by `2|F|` |
| S1 — hydrogen bonds appear | **BRANCH (b)** | OFF `0.0000`, ON `0.0000` of 20,000 frames with ≥ 1 inter-molecular H-bond (rung-1 lens); mean T 332 / 336 K; ON: 11 transitions, `work.field = 1.077e-03` |

## S1, read plainly, with its diagnostic

Neither arm formed a hydrogen bond in 22,000 frames. The OFF arm is rung 1's result again
(no electrostatics, molecules at distance and no alignment). The ON arm's diagnostic
(`field_probe.rs`, the same scene sampled every 500 steps) says why the field did not
change it: the four units were assigned at the first force pass (one transition), the
field's energy is POSITIVE — `+1e-3` to `+4e-3` hartree — and the nearest oxygen–oxygen
distance grows from 7.0 to 10.2 bohr over 6,000 steps. Fixed charges of `±0.23` on four
waters whose dipoles the scene set parallel repel, the thermostat at 293 K gives them a
`kT` comparable to the `~2e-3` hartree a favourable dipole pair would bind with, and they
drift to the walls. That is also why the ON arm's 22,000 steps ran in seconds where the OFF
arm's took minutes: separated molecules leave the pair loops nothing to do.

**What (b) says and does not say.** It does not say the field is wrong — every gate says
the term is the derivative of its energy and its books close. It says FIXED charges on a
rigid-monomer scene at this temperature do not produce hydrogen bonds in this engine, and
names what FIELD-2 must stake: the charges' geometry dependence and polarisation (channel
2, the fixed point the library already computes and the force law does not), the
orientation the scene starts in (a staked configuration, not a parallel-dipole square), and
the O···H pair table's role at hydrogen-bond distance (the table carries the covalent
curve's tail there, bare of any charge). The threshold-flicker branch A1 named did not
arise (one transition in the run).

## What this lands for the ladder

The engine's force law has an electrostatic term for the first time, derived from its own
density, with its own energy row, its own transfer column and a refusal where the far field
would need Ewald. G's hydrogen-bond network carrier can now be run WITH a field; whether
that field, as fixed charges, is enough is answered here: it is not, and the answer is a
measurement with a named successor rather than a guess.

## CORRECTIONS (FIELD-3, 2026-09-05)

Two findings of the FIELD-3 campaign land on this record and are entered here rather than
edited into the text above (rule 7: the record stays, marked).

1. **The unit rule was blind on this campaign's own scene from the first frame.** AMENDMENT 1's
   assignment (an oxygen with exactly two pair-verdict-bonded hydrogens, none shared) read a
   hydrogen of one water as BONDED to another water's oxygen at 5.7 bohr on the O–H curve's
   tail (`E_rel = −1.3e-4` inside the turning point) at step 16 of the four-water walled scene,
   so two of the four waters were uncharged there; the closure reading FIELD-3 installs (each
   hydrogen to the oxygen it is most bound to) finds all four. The two rules agree again by
   step 2,000, but the trajectories have diverged by then, and the channel receipt's `water4`
   block (nine lines: `e_kin`, `e_pair`, `e_three`, `e_field`, `energy`, `ledger`, `w_ext`,
   `work.field`, `drift`) was re-banked under FIELD-3's cause line. The S1 (b) reading above
   ("no hydrogen bond formed") stands as measured; its interpretation ("the field is net
   repulsive on the staked scene") was made on a scene where the assignment itself was
   flickering, and FIELD-2 then showed the assignment, not the field, to be the null's cause.
2. **The field's virial had the wrong sign.** `accumulate_field` posted `+E` to `w_virial`
   ("Σ r·F = E"); the engine's convention is `Σ r·dU/dr` (`pressure()` reads `(2K − W)/3V`),
   for which the Coulomb term is `−E`. Corrected under FIELD-3's cause line. No gate above,
   no ledger row and no receipt line reads the virial; `pressure()` with the field on was the
   only reading affected, and no campaign has read it.

