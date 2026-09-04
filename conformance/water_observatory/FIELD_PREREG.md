# Pre-registration — FIELD-1: the embedding field enters the force law — fixed derived charges on census molecules, one ledger column, the wrapped box refused by name

*Frozen 2026-09-04, committed ALONE, before `field.rs` existed. Built by the lead. Rung 1
measured water molecules at hydrogen-bond distance with almost no hydrogen bonds and B2
named the cause: `Sim::compute_forces` carries no electrostatic term. EMBED-1 built the
field as a library and EMBED-2/3 characterised it. This freeze puts its simplest honest form
INTO the force law: point charges on every census-certified water molecule, derived from the
engine's own density (the dipole-exact charge of the monomer at EMBED-1's pin), the Coulomb
term between molecules with analytic forces, its energy in the ledger, its charge-assignment
transitions posted as scene events, and the periodic box REFUSED by name until Ewald exists.
Polarisation and the charges' geometry dependence are named as FIELD-2; nothing here claims
them.*

misfits: contacts **M-VACUOUS-SUCCESS** (every gate states what it counted; a scene with no
charged rows carries an EXACT zero field energy and cannot pass a field gate);
**M-NULL-MISSTAKE** (each conservation gate is staked on the quantity its law constrains —
the ledger on `E − W_ext`, momentum on the internal-force sum — never on a proxy);
**M-PLANT-OBS** and **M-PLANT-SECTOR** (three plants, each with its carrier asserted nonzero
in the sector the plant acts on — §4); **M-STALE-INSTRUMENT** (this freeze alone; code,
tests and the results document together); **M-EXIT-DISCRIMINATOR** (every gate's fail is a
named branch; the identity gate compares BYTES, never a digest of positions);
**M-FIXED-POINT-TRAJECTORY** (the identity gate runs on a scene whose atoms MOVE, and the
momentum gate's plant must fire on it); **M-BARE-CHARGE** and **M-HOMOG** (the words
"charge", "local" appear; classical point charges, nothing homogeneous assumed);
**M-VOLUME-SCALE** (the periodic box with charges is REFUSED, Ewald the named exit; no volume
limit is taken); **M-DEVICE-CLASS** (native `f64` and the shipped wasm are one class by the
law-probe gate, unchanged here); **M-COND-PROBE** ("inside the field" appears; a force term,
not a post-step operator). Not contacted: the rest of the registry.

## 0. The term

For every live census row that is a water molecule (three members: one oxygen, two
hydrogens), charges `q_O = −2q`, `q_H = +q`, with `q` the dipole-exact charge of the isolated
monomer at EMBED-1's pin (`r = 1.9435738400` bohr, `θ = 1.6887434037` rad), computed ONCE from
`embed::monomer` when the field is enabled and carried in the checkpoint. Every other atom
carries zero. Between atoms of DIFFERENT charged rows:

```
E_field = Σ_{i<j, row(i)≠row(j)} q_i q_j / r_ij        F_i = Σ_j q_i q_j r_ij / r_ij³
```

Forces go into the internal accumulator (they cancel pairwise from the momentum sum); the
virial gains `E_field`; `energy()` gains `e_field`; NOTHING is posted to `w_ext` for the
conservative term. When the charge assignment CHANGES between steps (a row forms or dies),
the energy jump is posted to `w_ext` and to a new receipt column `work.field` as one
ledgered event — the ACUITY-B pattern. With the field disabled every line above is skipped
and the code path is the current engine's.

## 1. Gates

- **G0 — identity, EXACT.** Two scenes stepped 2,000 steps, one never touching the field
  and one with the field enabled then disabled before the first step: checkpoint BYTES
  identical. And a pure-hydrogen scene with the field enabled: `e_field` is exactly `0.0`
  at every step (no water row, no charge). 2 scenes.
  witness: `closed_iff_fiber_invariant`
- **G1 — the ledger closes with the field on.** A four-water scene under walls at 293 K,
  2,000 steps: `drift() ≤ drift_bound()` at every grain boundary AND `work_columns_ok()`;
  every charge-assignment transition posted to `work.field`. 1 scene.
  witness: none (an engine ledger gate)
- **G2 — momentum.** The same scene under `Boundary::Open`: `momentum_residual ≤ bound` —
  the field's forces are pairwise antisymmetric and cancel from the sum to roundoff. 1 scene.
  witness: none (an engine conservation gate)
- **G3 — the force is the derivative.** On a staked configuration, the analytic field force
  on every charged atom equals the central difference of `E_field` (`h = 1e-5` bohr) to
  `≤ 1e-8` relative. 12 atoms.
  witness: none (arithmetic)
- **G4 — the charge is the record's.** `q` equals `embed::fragment_charges(DipoleExact)` on
  the pinned water monomer to `1e-12` — the same number EMBED-1 read. 1 check.
  witness: none (an identity between two callers of one function)
- **G5 — the wrapped box is refused.** Enabling the field under `Boundary::Periodic`, or
  wrapping a box with the field on, returns a refusal naming Ewald as the exit; the state
  is unchanged. 2 checks.
  witness: none (a refusal gate)
- **S1 — hydrogen bonds appear (the payoff, staked).** A four-water scene, walls, 293 K,
  20,000 frames after 2,000 of settling, same seeds, two arms: field OFF and field ON. The
  fraction of frames carrying at least one inter-molecular hydrogen bond by the rung-1
  lens's criterion. **(a)** ON ≥ 10 × OFF and ON ≥ 0.10 ⇒ the field is what rung 1 was
  missing, and G's H-bond network carrier gets it. **(b)** otherwise ⇒ reported with both
  fractions; the next freeze stakes polarisation. 2 arms.
  witness: none (a measured population)

## 2. What each outcome means

G0–G5 green is the term landed with its books closed. S1(a) is the first time the engine's
water has bonded to water for a reason it derived; S1(b) is the measurement that says fixed
charges are not enough, which is FIELD-2's brief. A refusal under G5 is a feature.

## 3. Plants

- **(i) The reaction dropped.** The force on `j` omitted while `i`'s is applied. G2 must fire.
  Carrier: `|F_field| ≥ 1e-6` hartree/bohr on some atom of the scene, asserted nonzero in the
  sector the plant acts on.
- **(ii) The transition unposted.** A charge-assignment change applied without posting its
  energy. G1's `work_columns_ok` or drift must fire. Carrier: at least one transition occurs
  in the run (a row dies or forms), asserted nonzero in the sector the plant acts on; a run
  with none is VOID for this plant and says so.
- **(iii) The sign.** `E_field` negated. G3 must fire by `2|F|`. Carrier: as (i).

## 4. Discipline

Code `holon-render/src/field.rs` and the hooks in `sim.rs`, `checkpoint.rs` (version 6:
`work.field` and `field_q` join the ledger block), `lib.rs` (`holon_set_field`,
`holon_field_energy`, `holon_field_charge`, `holon_work_field`); tests `tests/field.rs`
carry G0–G5 and the plants; S1 in `examples/field_hbonds.rs` with `FIELD_RESULTS.md`. No
number enters from outside the engine.
