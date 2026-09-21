# MOLSEARCH-1 — the variational search at the molecule tier: PREREGISTRATION

*Written 2026-09-20, committed alone before the walk is run. Wager W4 (`STANCE.md`) says the
search's optimum coincides with the tier's object on every tier. VIEW-SEARCH-1 measured it
once, at the fluid tier. This is the second tier: does the search, run on the fine model's
own all-atom trajectory with no unit named, return the MOLECULE — the rigid water unit that
REPLACE-0 projected by hand — as the closed view at the molecule tier's cadence?*

**Carrier.** 16 waters (`n_cells = 2`, 48 atoms) on the fine (flexible) model at 293 K,
1,000 settle frames under the thermostat, then 80,000 NVE frames (`~2.1` ps) with every
atom's position and velocity banked every 38 frames (`1.0` fs): `replace0 atomwalk`.

**Dictionary, per molecule (the molecules are the ensemble; 16 × 2,000 samples).** Internal
coordinates: the two O–H lengths, the H–O–H angle, and their rates (6). Rigid modes: the
centre-of-mass velocity (3) and the angular velocity about the centre of mass (3). Twelve
features; the search is over the 12-dimensional dictionary, held out by molecule (fit on
12, evaluate on 4, rotated).

**Views scored.** The rigid view (6); the internal view (6); each alone against the bound at
its dimension. **Lags:** `5, 10, 20, 50, 100` fs.

**Stakes.** S1: at `τ = 50` fs the dictionary's top-3 singular functions have weight `≥ 0.9`
(R²) in the rigid subspace — the slow closed directions of the model at the molecule tier
are the rigid unit's. **Kill:** weight `< 0.7`. S2: the internal view's own top `σ` at 50 fs
is under `0.5` — the vibrations are not closed at that cadence, so the molecule as a rigid
unit is the tier's object and REPLACE-0's projection is the search's answer. **Kill:** an
internal `σ ≥ 0.8` at 50 fs, which would say a vibration is as slow as a rigid mode on this
model and the rigid operator's omission is not free at the molecule's own cadence. S3: the
rigid view's held-out fraction of the bound at `k = 6` is `≥ 0.8`. **Kill:** `< 0.6`.

**Plants.** PM-1: a synthetic ensemble of rigid rotors plus fast harmonic bond stretches at
known frequencies — the search returns the rotor subspace at weight `≥ 0.95` and the
stretch `σ` at its known `cos(ωτ)`-average. PM-2: time-shuffled rows — every score under
`0.05` of the bound. PM-3: the angular velocity computed on a rigid synthetic molecule
matches the imposed one to `10⁻⁹`.

**Branches.** (a) all met: W4 measured on a second tier; the molecule is found, not named.
(b) S1 or S3 fails: the rigid unit is not the closed view at 50 fs; the singular functions
are printed and the tier's object is read off them. (c) S2 fails: a slow vibration exists;
REPLACE-0's price is re-read at the molecule's cadence. (e) a plant fails: nothing read.

---
*Audit footer, added 2026-09-21 for `Audit/prereg_audit.py` after CI read red since 2026-09-19; no stake, gate, plant or number above moved.*
witness: none (a measured campaign: its gates are numeric and its closure algebra is `Closed` and `StatClosure.lean`, cited in words above; no gate is a Lean theorem of its own)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR — the registered ids this text contacts by keyword, cited at the audit's demand.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier in its own row, and the sector the plant acts on is nonzero in that carrier by construction of the plant.
