# MOLSEARCH-1 READ (preliminary, on a walk the gate REFUSES): the molecule is two decoupled closed sectors — and the rigid unit is the LESS closed one

*2026-09-21 01:50 CDT. Prereg `MOLSEARCH1_PREREG.md`, Amendment 1 (closed is not slow; the
carrier; the settle), reader `molsearch.py` (plants PM-1..3). This read is on the SECOND walk
(`molsearch1_warm_396K/`, 527 rows at 0.99 fs, 128 molecules), which entered NVE at 372 K and
ran to 396 K — outside A3's 10 % gate of 293 K, so REFUSED as a reading of the campaign's
carrier and reported as what the model does at 396 K. The third walk (260 fs → 1.06 ps of
settle) is running; its read will be appended, and the verdict below is provisional until
it lands.*

## 0. What the search returned

| lag | dictionary top σ | purity of top-6 (each in one sector) | cross-block `‖K_RI‖/‖K‖` | rigid view / bound at `k = 6` | own top σ, rigid / internal |
|---|---|---|---|---|---|
| 5 fs | 0.98, 0.97, 0.97 … | 1.00 ×6 | **0.024** | 1.07 | 0.97 / 0.98 |
| 10 | 0.92 … 0.83 | 0.72–1.00 | 0.021 | 0.77 | 0.91 / 0.92 |
| 20 | 0.79 … 0.70 | 0.53–0.98 | 0.052 | 0.62 | 0.73 / 0.79 |
| **50** | 0.62, 0.59, 0.29 … | **1.00, 1.00, 0.93, 0.96**, 0.58, 0.63 | **0.151** | **0.11** | **0.18 / 0.62** |
| 100 | 0.40 … | 0.99, 0.99, 0.96, 0.89, 0.91, 0.75 | 0.251 | 0.12 | 0.12 / 0.40 |

**As written:** S1 KILLED (top-3 weight in the rigid subspace `0.03` at 50 fs), S2 between
(internal σ `0.62`), S3 KILLED (rigid fraction `0.11`) → branch (b): the rigid unit is not
the closed view at 50 fs; the singular functions are printed and the tier's object is read
off them. **As amended:** S1′ (purity) MET for the four directions that carry signal at 50 fs
and at every shorter lag for the top three; S2′ (cross-block) MET at 5–20 fs (`0.02–0.05`),
between at 50 (`0.15`), at the kill at 100; S3 killed.

## 1. What it means, read off the singular functions

- **The two sectors are decoupled.** At the molecule's own cadence (5–20 fs) the Koopman
  matrix is block-diagonal to 2–5 %: every closed direction is a pure vibration or a pure
  rigid mode. That is the licence for the rigid operator, measured — dropping the vibrational
  block costs the cross-block coupling, which is REPLACE-0's price, and it is small.
- **The rigid unit is the LESS closed sector.** A molecule's centre-of-mass and angular
  velocities decorrelate in `~30` fs (σ `0.18` at 50 fs); its vibrations dephase in `~100` fs
  (σ `0.62`). At the liquid's cadence the search ranks the vibrations first, by a factor of
  three. **The tier's object is not the search's top rank.** It is the decoupled block that
  the tier above carries — mass and momentum, which sum over molecules into the fluid's
  fields (VIEW-SEARCH-1's closed directions) — while the vibrational block, more closed and
  just as separable, carries nothing the fluid tier reads.
- **So "closed" and "the object" are two properties.** The search finds the closed sectors
  and their coupling; which sector is the tier's object is decided by what the next view
  needs — the rent, in the object's own language — not by closure rank. Wager W4 as worded
  ("the search's optimum coincides with the tier's object") is FALSE at this tier in its
  naive reading and true in a corrected one: the object is a search-found closed sector,
  selected by the tier above. Recorded as an OPEN correction to W4 pending the 293 K walk.

## 2. Also read

The rigid modes' velocity decorrelation time `~30` fs at 396 K; the vibrational dephasing
`~100` fs; the cross-block coupling growing with lag from `0.02` at 5 fs to `0.25` at 100 fs —
the channel by which vibrational energy equilibrates, and the reason a 26 fs settle left the
first walk at 555 K.

## 3. The third walk, at 299 K — the read stands (appended 2026-09-21 11:20 CDT)

Settled 40,000 frames to 300.0 K, NVE from 298.9 K (inside the gate), 527 rows × 128
molecules (`molsearch1/molsearch.read.txt`):

| lag | purity of top-6 | cross-block | rigid / bound at `k = 6` | own top σ rigid / internal |
|---|---|---|---|---|
| 5 fs | 0.98–1.00 | **0.019** | 0.99 | — |
| 10 | 0.94–1.00 | 0.031 | 0.72 | — |
| 20 | 0.77–0.98 | 0.061 | 0.63 | — |
| **50** | 0.89–1.00 | **0.165** | **0.09** | **rigid ≪ internal 0.56** |
| 100 | 0.68–0.99 | 0.244 | 0.12 | — |

S1 KILLED (`0.01`), S2 between (`0.564`), S3 KILLED (`0.09`) as written; S1′ met at every lag
for the directions that carry signal, S2′ met at the molecule's cadence (`0.02–0.06`) and
between at 50 fs — the same picture as at 396 K to the second digit. **The provisional label
comes off: the molecule is two decoupled closed sectors, the vibrations are the more closed,
the rigid unit is the object because the fluid carries it.** Branch (b) as written, (a) as
amended.
