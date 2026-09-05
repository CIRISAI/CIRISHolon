# LIQUID-1 — AMENDMENT 1: the lattice formula corrected, the orientation measure declared, the dry run named as an instrument check

*Frozen 2026-09-05, committed alone, BEFORE any counted arm: at the time of this commit no
seam law has passed its boundedness gate (FIELD-9's and CT-1's laws are both REFUSED by
G-B0), so the campaign has not run and `conformance/water_observatory/liquid1/` holds only
the instruments' dry-run records, none of them a reading. Written because building the
instruments found three things the freeze's letter left wrong or unpinned.*

misfits: contacts **M-STALE-INSTRUMENT** (the freeze's own formula for the lattice spacing
was wrong by a factor of two while its number was right); **M-FIXED-POINT-TRAJECTORY** (the
generator and the orientation measure are pinned here, since a box built from an unpinned
measure is not one box); **M-VACUOUS-SUCCESS** (a dry run on a REFUSED law produces files
that look like an arm's; every one of them carries `law_refused_for_counted_arm: true` and
`is_a_reading: false`, and none may be quoted as a result); **M-PLANT-OBS** and
**M-PLANT-SECTOR** (the two plants are unchanged from the freeze; their shared carrier — the
fraction of first-shell O–O pairs whose minimum-image vector crosses a face, `≥ 5 %` on the
counted frames — is asserted nonzero in the sector each plant acts on, the wrapped
positions, before either plant is read); **M-VOLUME-SCALE** (contacted by keyword: the
lattice is the START only, and the cell edge is the state point's density, never a grid
count). Not contacted: the rest of the registry.

## The changes

1. **The nearest-site distance.** §0 wrote "nearest sites `6.41` bohr = 3.39 Å apart" and
   glossed it as `L·√3/4`. Corner to body centre in a `4 × 4 × 4` body-centred cubic lattice
   is `(√3/2)·(L/4) = L·√3/8 = 6.41` bohr; `L·√3/4` would be 12.81. The NUMBER in the freeze
   was right and the formula beside it was wrong; the builder's gate asserts `L·√3/8` to
   `1e-9` under the minimum image on every oxygen (EXACT).
   witness: none (a geometric assertion in `tests/liquid.rs`; nothing mechanized in Lean)
2. **The orientation measure, pinned.** §0 said "three LCG-drawn Euler angles (seed
   `0x4c49_5155_4944`)" and pinned neither the generator nor the measure. Pinned here: the
   64-bit linear congruential generator `x ← 6364136223846793005·x + 1442695040888963407`
   (Knuth's MMIX constants), the top 53 bits as a uniform in `[0, 1)`; Euler angles in the
   z-y-z convention with `α` and `γ` uniform on the circle and `cos β` uniform on `[−1, 1]` —
   the rotation-invariant measure. A uniform `β` would pile the molecular axes at the poles
   and give the box a direction the state point never gave it.
3. **The dry run is an instrument check and not the campaign.** `liquid1 --dry` builds the
   box, runs the door and the expectation, measures the price and steps 200 counted frames
   on whatever law the loader finds, INCLUDING a law the loader refuses for a counted arm,
   with the refusal recorded in every file it writes. Its records are the instruments'
   proof of function (the door's numbers, the price, the per-pass L0 rule firing), never
   readings of water. The campaign's own arm runs only on a law the loader admits, and its
   results file is the one that reads R1–R3.
4. **Two conversions stated.** The first-shell radius for the plants' carrier, "3.2 Å", is
   converted through the same `0.529177210903` Å per bohr the lens carries (6.047 bohr), not
   typed; the diffusion coefficient's conversion to cm²/s uses the engine's own atomic time
   unit (`sim.rs`'s `AU_TIME_S`), cross-checked at run time against the lens's `AU_TIME_FS`
   to `1e-9`, and the runner refuses if they disagree.

Gates, kill bands, the box's edge from the density, the settling and counted frame counts,
the readouts and their cadence: unchanged.
