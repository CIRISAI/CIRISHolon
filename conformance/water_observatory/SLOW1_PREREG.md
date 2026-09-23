# SLOW-1 — the search on an OPEN question: what is the slow variable of a sluggish liquid? PREREGISTRATION

*Frozen 2026-09-23, committed alone before any arm runs and before the reader exists. The
stakes are the lead's (2026-09-23), written here as given; where the carrier could not hold a
stake exactly as worded, §"What could not be built as written" says what and why, and the
stake's number is not moved.*

**Why this is a kill test for the programme.** Every search so far (`MOLSEARCH1`, `VIEW_SEARCH`,
`TSCAN_HBOND_SEARCH`, `OBJECT.md` "The procedure, as arithmetic") returned something already
known: the rigid unit, the long-wavelength density and current modes, a bond network the
momentum law does not carry. None was asked a question whose answer is not in a textbook.
The slow variable of a supercooled liquid — the structural quantity that closes at 1–20 ps
AND carries mobility (dynamic heterogeneity) — has candidates in the literature (tetrahedral
order `q`, local density or Voronoi volume, H-bond count, cage-escape displacement, a local
proxy of the non-Gaussian parameter, local potential energy) and no agreed answer. If the
search-then-select procedure can only re-find, that is a finding about the procedure; this
campaign is built so that either outcome is read.

**The regime, from the record.** `TSCAN1_RESULTS.md`: the rigid operator diffuses at
`D = 5.4–7.7 × 10⁻¹⁰` m²/s at 312–317 K (a quarter to a third of water's), Arrhenius with
`E_a = 19.0` kJ/mol; at 400–412 K `D = 4.5–4.8 × 10⁻⁹`. `TSCAN_HBOND_SEARCH_RESULTS.md` S3:
the Stokes–Einstein product `Dη/T` in units of water's is `0.09` at 312 K and `0.44` at 406 K
(`0.72` at 520 K) — decoupled `~10×` at 293 K target, nearer to holding at 400 K. So 293 K is
the sluggish regime and 400 K is the control where heterogeneity should have faded.

## Carrier

- The rigid operator, `replace0 scout` from the main checkout's committed binary
  (`engine/target/release/examples/replace0`, built 2026-09-20 10:22, the binary TSCAN-1 ran;
  not rebuilt), `--cells 5` = **250 waters** (`2 n³`; `--cells 4` is 128, not 64 — the brief's
  "64" is TSCAN-1's prose, the TSCAN-1 records say 128), the banked stiffness envelope
  (`replace0/transport_seed0/stiffness.json`), imposed liquid density, `--settle 1000
  --settle-rigid 1000` (rigid settle by criterion, floor 1,000), `--readouts 2500
  --readout-fs 20` = **50 ps NVE production**, `--workers 1`.
- Arms: **293 K seeds 0, 1, 2** (the sluggish regime) and **400 K seed 0** (the control).
  Records in `replace0/slow1/T{293,400}_seed{S}/` (`rigid.walk`, `rigid.vwalk`, `scout.*`,
  `run.done`), `replace0/slow1/slow1.DONE` when all four end.
- Placement: one arm per core, pinned with `taskset -c` to **E-cores 16, 17, 18, 19** (the
  same core class for every arm, and the class TSCAN-1's prices were measured on); core 20
  is left for the launcher's price watcher. Prices across arms are compared only within that
  core class (M-PLACEMENT-LOTTERY); no wall clock on another class is quoted against them.
- Temperature: the rigid modes are read against the target as TSCAN-1 did; TSCAN-1 ran 6–8 %
  warm on every arm. A point outside 10 % of its target is refused and reported (the 293 K
  arms are expected near 310–317 K; that is the regime, read at the temperature reached).

## Cost (written before launch)

From the banked scout prices on E-cores: 128 waters `1,373–1,434` core-s/ps (TSCAN-1),
432 waters `2,874–4,066` (RESPONSE-1). The price grows sublinearly in `N`; 250 waters is
estimated at **`2,000–2,600` core-s/ps**, so 50 ps is **`28–36` h per arm on one core**, plus
a fine and rigid settle of `2–4` h: **about 1.5 days**, well under the 5-day ceiling. The
launcher MEASURES the price on the first 100 readouts (the scout's own `core-s/ps` line at
readout 100) and writes the ETA to `replace0/slow1/slow1.eta`. **If the measured price puts
50 ps beyond 5 days on one core, the arms are stopped at readout 1,250 (25 ps) and read
there** — the walk is appended per readout, so this needs no relaunch — and the `### On
launching` note says so before any read. Nothing else about the stakes changes with the length.

## The dictionary, declared per molecule per readout

The rigid walk carries the **oxygens' unwrapped positions and velocities only** (`rigid.walk`,
`rigid.vwalk`, 3 numbers per water per readout; `replace0.rs` `append_walk_rows`). Structure is
computed from the wrapped oxygen positions under the minimum image.

| block | columns | definition |
|---|---|---|
| **STRUCT** | `q` | tetrahedral order of Errington & Debenedetti (Nature 409, 318, 2001; after Chau & Hardwick 1998): `q = 1 − (3/8) Σ_{j<k} (cos ψ_jk + 1/3)²` over the four nearest oxygens |
| | `n33`, `n50` | local density: oxygens within `3.3 Å` and within `5.0 Å` |
| | `nb` | bond count: oxygens within `3.5 Å`, the O–O leg of the HBOND reader's rule (see below) |
| **HIST** | `h1, h2, h5, h10` | cage-escape displacement `|r(t) − r(t − Δ)|`, `Δ ∈ {1, 2, 5, 10}` ps — BACKWARD-looking, known at `t` |
| **MOM** | `vx, vy, vz` | the oxygen's velocity (the control that must NOT be the slow sector) |
| **MODE** (reported) | `cos, sin(2π x_a / L)`, `a = x, y, z` | the molecule's phase in the first-harmonic density modes — the hydrodynamic directions |

Local potential energy per molecule: **omitted — the rigid walk does not carry it** (the scout
writes oxygen positions and velocities; the potential is summed over the box in the log only).

**Lags searched:** `0.1, 0.5, 1, 2, 5, 10, 20` ps (5 to 1,000 readouts). Transitions are pooled
over molecules and time origins; the dictionary is standardised globally and NOT centred per
molecule (a per-molecule persistent offset is part of what "slow" means; the time-shuffle
plant PS-4 bounds what such offsets alone contribute). Held out by molecule in four folds
(`i mod 4`). The VAMP core is `conformance/reasoning/reason_search0b.py` (`vamp`, `blocks`,
`heldout`), the one core `hbond_search.py` already imports — `OBJECT.md` "The procedure, as
arithmetic", adopted from Wu & Noé 2020.

**The sector.** At each lag the STRUCT block's own VAMP: `σ₁ ≥ σ₂ ≥ …` and its top singular
function `f(t) = Φ_STRUCT(t) · W₀ U[:, 0]` (standardised coordinates). The **sector lag** `τ*`
for S2 and S3 is the lag in `{1, 2, 5}` ps where the STRUCT `σ₁` is largest on that seed —
chosen on the closure alone, before any prediction is computed.

## Stakes

- **S1 (closure).** At some lag `≥ 1` ps the STRUCT block's own top `σ₁ ≥ 0.5`, AND at that
  lag the cross-block `‖K_{STRUCT,MOM}‖/‖K‖ < 0.2` (distinct from the momentum directions).
  MET if this holds on every 293 K seed. **Kill:** `σ₁ < 0.2` at every lag `≥ 1` ps on every
  293 K seed — nothing structural closes beyond the hydrodynamic modes. Otherwise between.
  Reported beside it, no band: the cross-block of STRUCT with MODE and with HIST, and the
  held-out score of the one-dimensional view `f`.
- **S2 (carried — THE STAKE THAT MATTERS).** Target: each molecule's displacement over the
  NEXT 5 ps, `y(t) = |r(t + 5 ps) − r(t)|`. Baseline `Z` = the current velocity
  `(vx, vy, vz, |v|²)`. For each known candidate `c` taken individually — `q`; density
  `(n33, n50)`; bond count `nb`; and each cage-escape displacement `h1, h2, h5, h10` — the
  increment `ΔR²_c = R²(Z + c + f) − R²(Z + c)`, held out by molecule (four folds, ridge
  `10⁻³`, the HBOND S3 regression). The S2 statistic is `min_c ΔR²_c` at `τ*`.
  MET: `≥ 0.05` on every 293 K seed AND the 400 K statistic is LESS than the 293 K seeds'
  mean (heterogeneity fades). **Kill:** the 293 K seeds' mean `< 0.02`, OR the 400 K
  statistic `≥` the 293 K mean (no contrast). Otherwise between. Reported beside it: the
  increment beyond `Z` alone (`R²(Z + f) − R²(Z)`, "the carry") and every `ΔR²_c`.
  *Construction, not a moved stake:* a candidate on which `f` loads `≥ 0.8` (S3) is `f`
  itself, and "beyond itself" is zero by construction, so that candidate is excluded from
  the minimum; in branch (b) S2 is then read as that candidate's own carry beyond `Z` and
  beyond the others. This is said here, before the read, because otherwise a re-finding
  would read as S2 killed by arithmetic rather than by the liquid.
- **S3 (a direction, not a re-finding).** The loading of `f` at `τ*` on each candidate
  `c ∈ {q, density (n33, n50), bond count (nb)}` is the `R²` of `f` regressed on `c`'s columns.
  RE-FINDING if one candidate's loading is `≥ 0.8` → branch (b); DIRECTION if no candidate is
  above `0.6` → branch (a), with the weights of `f` on the four STRUCT columns as the result;
  between `0.6` and `0.8`: reported as a partial re-finding, neither branch. On an
  oxygen-only walk `n33` and `nb` differ by a 0.2 Å shell and are nearly collinear: if
  density AND bond count both load `≥ 0.8`, the re-finding is named "the first-shell count"
  and the two are NOT claimed separable.
  **The independent check owed in branch (a):** the non-Gaussian parameter
  `α₂(t) = 3⟨Δr⁴⟩/(5⟨Δr²⟩²) − 1` over all oxygens; its peak time `t_α` against `τ*`: consistent
  if within a factor of 2. Computed on every arm and reported in every branch.

## Plants

Each plant names its carrier; the sector the plant acts on is nonzero in that carrier by its
construction, and each is run on this reader (on the TSCAN-1 walks and the synthetic) before any 50 ps walk is read.

- **PS-1** (carrier: every real walk; sector: MOM) the momentum block is NOT the slow sector:
  MOM's own `σ₁` at 1 ps `< 0.2`.
- **PS-2** (carrier: every real walk; sector: STRUCT paired to another molecule's future) the
  re-paired null — STRUCT (hence `f`) and the candidates from molecule `π(i)`, a fixed
  derangement, `Z` and `y` from molecule `i` — gives the increment beyond `Z` `≤ 0.01`.
- **PS-3** (carrier: a synthetic walk generated by the reader, 128 particles × 2,500 readouts
  at 20 fs; sector: a planted STRUCT column) a per-molecule log-mobility `m_i(t)`, an
  Ornstein–Uhlenbeck process with **5 ps** memory, scales each particle's random-walk step
  by `e^{m}` and is written into STRUCT as a fifth column with noise of equal variance; the
  other columns are computed from the synthetic positions. PASS: `f` at `τ*` loads `≥ 0.9` on
  the planted column (and its carry is reported).
- **PS-4** (carrier: every real walk and the synthetic; sector: STRUCT) shuffling each
  molecule's readouts in time destroys S1: STRUCT `σ₁ < 0.2` at every lag `≥ 1` ps after the
  shuffle. A failure here names per-molecule offsets over the run as the source of `σ₁`, and
  S1 is then not read.
- Lag 0 (the HBOND PH-2 check): `max |σ − 1| < 10⁻²` on STRUCT at lag 0, reported.

## Branches

- **(a) DIRECTION** — S1 and S2 MET, S3 a mixture (no candidate above 0.6): the slow variable
  of this operator's sluggish liquid is the declared mixture, weights reported, the `α₂` check
  owed and read.
- **(b) RE-FINDING** — S1 and S2 MET, S3 `≥ 0.8` on one candidate: the literature's candidate,
  confirmed on this operator; the procedure re-found, it did not direct.
- **(c) S1 killed** — no structural slow variable at these lags on this operator.
- **(d) S2 killed** — closed but not carried, as the H-bond network was.
- **(e) a plant fails** — nothing is read; the failure is the finding about the reader.

## Referee-free

No experimental comparison is claimed. The liquid is the rigid operator's; its slow variable,
if one is found, is the model's, and a sentence carrying it to real supercooled water owes a
separate campaign on a carrier that has earned that name.

## What could not be built as written (said before the read)

1. **Bond count donated/accepted** needs hydrogen positions; the rigid walk carries oxygens
   only and the binary is not rebuilt. `nb` is the O–O distance leg of the HBOND rule
   (`< 3.5 Å`), undirected; donated and accepted are not separated.
2. **Local potential energy** — not on the walk; omitted.
3. **"64 waters"** at `--cells 4` is 128 on the record; `--cells 5` gives 250, within the
   brief's 125–250 window, and is used.
4. **The momentum** is the oxygen's velocity, the walk's; the centre-of-mass velocity is not
   written by the rigid scout.
5. **S2's "beyond the candidates individually"** excludes the candidate `f` IS (S3 `≥ 0.8`);
   stated above as a construction.

---
witness: none (a measured campaign: its gates are numeric and its closure algebra is `Closed` and `StatClosure.lean`, cited in words above; no gate is a Lean theorem of its own)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-HOMOG, M-PLACEMENT-LOTTERY — the registered ids this text contacts by keyword, cited at the audit's demand. M-PLANT-OBS: every plant was re-derived for THIS reader and is run on it before the read. M-HOMOG: "local" here is a molecule's neighbourhood count, not a spatial-locality claim about a graph family. M-PLACEMENT-LOTTERY: all arms on one core class, prices compared only within it.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier in its own row, and the sector the plant acts on is nonzero in that carrier by construction of the plant.
