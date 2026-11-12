# KINK-0 — READ: branch (a), as a CONFIRMATION and not a blind read. At half a unit of background a unit charge moves freely in the bulk at finite positive cost; at none it stays at the end. A re-finding of Coleman (1976).

*2026-11-11. Prereg `KINK0_PREREG.md` (frozen alone, `5786412`, after two adversarial reviews);
instrument `kink0/kink0.py` (completed and committed before the freeze, `064fc70`); record
`kink0/campaign.log`; pilot `kink0/pilot.log`. Carrier: the massive Schwinger model in the
engine's spin form, `x = 3`, `m/g = 0.8` and `0.15`, exact diagonalisation in sectors of
total charge at `N = 12, 14, 16, 18`; 48 states, none void, largest residual `1.2 × 10⁻¹³`.*

**How this result may be cited.** The campaign's point was never computed before the run.
But the second review computed a grid that brackets it on both axes, and every reading is
smooth across that grid, so these numbers could have been interpolated to a few per cent
beforehand. The stakes were met as expected. This is a confirmation that the instrument and
the physics agree at one more point, with the plants firing. It is not evidence obtained
blind.

## 0. Verdict

| gate | staked | read | verdict |
|---|---|---|---|
| **G0** massless referee | `10⁻⁶` | `2.6 × 10⁻¹¹` | **MET** |
| **G0b** mass, background, charged sectors against a Kronecker referee | `10⁻¹⁰`; `> 1` with the sign reversed | `8.5 × 10⁻¹⁴`; `2.05` | **MET** |
| **S1** total charge (a constant of the sector) | `10⁻¹⁰` | `1.3 × 10⁻¹⁴` | **MET** (cannot fail) |
| **S2** glued at none, loose at half a unit | end-distance change `< 0.05` at `α = 0`; growth `≥ 1.5` at `α = ½` | `0.0000`; `2.337` | **MET** |
| **S3** a mobile object | heavy ratio `≤ 0.65`; contrast `≥ 0.15`; exponent rising heavy, falling light | `0.539`; `+0.222`; `+0.094`, `−0.035` | **MET** |
| **S4** finite positive cost | heavy `ΔE₋(18) > 0`, differences shrinking; light `< 0` | `+2.364`, shrinking; `−0.509` | **MET** |
| **S5** the field at the centre | heavy within `0.05` of `½`; light lower by `≥ 0.10` | `0.473`; lower by `0.214` | **MET** |

**Plants.** P-MASS-SIGN fires (the energy moves by `3.03`). P-BACKGROUND fires (the
reflection identity holds to `1.1 × 10⁻¹⁴`, and a mislabelled sector would differ by
`5.33`). P-PINNED fires, signed: raising the background on links 0 to 8 moves the charge
`4.55` sites left, lowering it moves it `1.55` right.

## 1. The readings

| | `α = 0`, heavy | `α = ½`, heavy | `α = ½`, light |
|---|---|---|---|
| distance of the `−1` charge from the right end, `N = 12 → 18` | `0.411 → 0.411` | `3.44 → 5.78` | `2.94 → 4.18` |
| gap above the lowest state in the `−1` sector | `2.896` at every size | `0.662 → 0.357` | `1.645 → 1.252` |
| cost of the charge, `ΔE₋` | `+3.439` | `+2.447 → +2.364` | `−0.287 → −0.509` |
| field at the centre of the neutral state, `N = 18` | `0.000` | `0.473` | `0.260` |

- At no background the charge is `0.41` sites from the end at every size, to five digits,
  and the sector's gap does not move: nothing there is free.
- At half a unit and the heavy mass the charge sits about a third of the way along a chain
  of any length, its gap closes with a rising exponent as a level in a box does, and it
  costs a finite positive energy that is still falling slowly (`M ≈ 2.2` to `2.3` by the two
  unstaked fits).
- At half a unit and the light mass the charge still moves outward as the chain grows, only
  less; its gap closes with a falling exponent; and its sector lies BELOW the neutral one,
  which the freeze stakes as a property of this open chain's end and not of the vacuum.

## 2. What it means, at the strength it carries

- In this record's vocabulary: on this lattice a `U(1)` charge is its field's step
  (`SceneCharge`'s law, trivially), and whether that step is a free object in the bulk
  depends on a property of the vacuum on either side of it, not of the charge. That is
  Coleman's result and it is re-found, not found.
- It is the second substrate, after FERMION-0's `ℤ/2` one, on which a charged object exists
  in the bulk only by virtue of the field configuration around it. Neither is a lepton.
- No continuum statement: `N ≤ 18` at `x = 3` is below the registry's own floor for one.
- Two masses do not locate the transition; the review's grid puts the finite-size crossover
  near `m/g ≈ 0.22` to `0.26` at these sizes, and that is the review's reading, not a stake.

## 3. Deviations and process, said plainly

1. **Three texts.** The first draft's parameter point was contaminated by the author's own
   brief to the first review, and two of its stakes and two of its plants were wrong. The
   second draft moved the point and rebuilt them. The freeze applied the second review.
2. **Not blind.** As above. The author asked the reviewer where the transition sits, and the
   honest answer required a bracketing grid.
3. **A light mass chosen before the grid was kept after it.** The review offered a point
   with twice the contrast; it was declined so that the point was not chosen for its effect.
4. **A disturbance to the machine.** One batch of the second review ran 14 unpinned
   processes and briefly drove the load average above 300. No timing is staked here, but any
   timing-sensitive job on this machine around that time may have been disturbed.
5. **`step_monotone`** is still computed by the reader and no longer read: it was true at
   every pilot row and discriminated nothing.

## 4. What this does not say

Not a lepton; not chirality; not the location of the transition; not a continuum limit; not
3+1 dimensions; not a discovery; and not a blind test.

## 5. Next, named and not frozen

An instrument campaign that gates the engine's real-time evolver and a current reader on
this model against the dense propagator; then the plasma frequency after a field quench
(PLASMA-0). Both are in `KINK0_ANOMALY0_REVIEW.md`. The two-static-charges form of this
campaign (energy against separation) needs a mass term and a non-integer background in the
engine's own Schwinger model, added once and gated against this instrument.
