# FIELD-1 — AMENDMENT 3: the drift bound is the pair envelope, not the walls; plant (ii) reads the ledger's SHIFT against the honest run

*Frozen 2026-09-04, committed alone. AMENDMENT 2 moved G1 to the open box on the reading
that the wall stiffness set the 20-hartree drift bound. Run on the open box, the bound is
the same `20.15` hartree and the honest numbers are bit-identical to the walled run's: the
walls never engaged. A2's stated reason was WRONG and is corrected here; its re-staked gate
G1′ stands (the open box is the right scene for a conservation gate and G2 already lives
there). The bound is the engine's O(h²) integrator envelope, `DRIFT_SAFETY · ¼ · ω² dt² ·
e_ref`, with `ω²` from the stiffest reachable pair curvature and `e_ref` from the largest
pair energy seen — an honest envelope that is loose by four orders on a stiff water scene,
where the measured drift is `3e-6` hartree. A ledger gate that admits a `20` hartree
excursion cannot see an unposted `8e-4` hartree transition; the gate's LETTER cannot fire
on this plant on this carrier, and the record must say so rather than read a pass.*

misfits: contacts **M-VACUOUS-SUCCESS** (the envelope bound makes `drift ≤ bound` unable to
fail here; the gate that IS read below is a two-arm discrimination with its own carrier);
**M-NULL-MISSTAKE** (the plant is read on the quantity the transition moves — the ledger
`E − W_ext` — against the honest arm's measured excursion, never against the envelope);
**M-PLANT-OBS** and **M-PLANT-SECTOR** (the carrier — a transition of at least `1e-4`
hartree, asserted nonzero in the sector the plant acts on — was satisfied on both runs);
**M-STALE-INSTRUMENT** (alone, before the re-read); **M-EXIT-DISCRIMINATOR** (A2's wrong
reason is its own named outcome, corrected and kept); **M-BARE-CHARGE** (as before). Not
contacted: the rest of the registry.

## A3.1 The corrected reading of plant (ii)

- **plant (ii), re-staked.** Two arms of the same open-box scene from the same seed: HONEST
  (the transition posted) and PLANT (the transition applied unposted). Let `ΔE` be the
  transition's energy (the honest arm's `work.field` after enabling, `≥ 1e-4` hartree —
  the carrier). The plant FIRES iff the plant arm's `drift_peak ≥ ½ |ΔE|` while the honest
  arm's `drift_peak ≤ ⅒ |ΔE|`. Both arms report `drift_peak`, `drift_bound` and `ΔE`.
  witness: none (a two-arm discrimination on the ledger)

## A3.2 What is kept, and one engine fact for the fence ledger

G1′'s honest half stands as staked and is READ as what it is: the ledger closed to `3e-6`
hartree over 2,000 steps with the field on, the receipt columns summing to `w_ext`, the
enabling transition posted. Its `drift ≤ drift_bound` clause is TRUE and UNINFORMATIVE on
this scene, and that is an engine fact: **the drift bound's envelope is four orders loose
on stiff water scenes**, which is a fence with an owner (the ledger's author) and an exit
(a measured-excursion bound beside the envelope, or a tighter envelope from the actual
mode energies) — registered in `FENCES.md` by the results document, not fixed here.
