# CT-3 — results

*Freeze `CT3_PREREG.md` (8114b3d, alone, before any new solve). Engine (this lane): `CtMode` and
the exclusive three shapes of channel 6, `seam::ct_coords`, `seam::CtTable` (the cubic
polyharmonic spline, its saddle solve, its analytic gradient on five atoms, the declared inward
fence), `bounded_ct`/`hole_ct`, the table served in `sim.rs::accumulate_seam`, checkpoint v14, the
door (`holon_ct_table_begin`/`knot`/`finish`, `holon_set_seam_ct_table`). Harvest
`holon-render/examples/ct3_harvest.rs`: `gate`, `predict`, `solve`, `read`. JSON under `ct3/`,
every file valid.*

## The verdict, first

**The map beats the family, on the geometry held out from it, by two and a half times — the kill
did not fire. Charge transfer served as a table is right where a fitted family was wrong: sixty-four
of sixty-four on the map's own leave-one-out against the family's forty-one, and 1.72 mHa against
4.29 mHa on a bond neither had seen. The contacts re-clamped against the deeper transfer make the
whole law bounded, so LIQUID-2 has its term. What the law does NOT yet do is the whole bond: the
full energy over-binds by 2.58 mHa on a bond of 2.75 mHa, S2 branch (b), and two thirds of that
miss is the transfer term's own.**

1. **S1 (a) — the table is nearer, and inside its own gauge.** At the held-out geometry the exact
   dimer's charge transfer is `−2.140507e-2` hartree. The table, filed before the solve, said
   `−2.312788e-2`: a miss of `1.723e-3`, 8.05 % over. CT-2's family on the same geometry said
   `−1.711144e-2`: a miss of `4.294e-3`, 20.06 % under. The table is **2.49×** nearer and inside
   the leave-one-out band of `2.277e-3` it staked on itself. The GANTT kill — *the table misses
   the held-out node by more than the family did* — did NOT fire.
2. **The four coordinates are the finding, and three were measured to be impossible.** Before any
   engine code was written, the plan's `(donor angle, acceptor angle, R)` was tested on the map's
   own records: it collides 24 of the 64 nodes onto 12 sites with spreads to `1.076e-2` hartree,
   larger than the values themselves — the tilt and twist families are two ORTHOGONAL great
   circles of the acceptor's orientation sphere and one acceptor angle cannot separate them.
   Adding the azimuth, written smoothly as `q = 2(u·n̂)² + (u·b̂)² − 1`, resolves eight of the
   twelve. The four that remain are genuine POLES where the azimuth is undefined; they are merged
   to their mean and the largest spread, `7.091e-4` hartree, is the table's own resolution floor.
3. **The serving rule is per-DIMER, and the alternative was measured, not argued.** `E_CT` is one
   number per node, already carrying transfer in either direction, so it is served ONCE per
   unordered pair of units at that pair's shortest cross-unit H···O. Serving it once per ORDERED
   pair — the obvious reading — double-counts by up to `2.505e-2` hartree on the twist family,
   larger than the node's own `E_CT`. The argmin's own discontinuity, measured over a full turn
   of the donor in 3,601 samples at four separations, is `1.418e-5` hartree.
4. **The contacts had to be re-clamped, and re-clamping them admits the law (C1 (a), G-B0W).** The
   served boundedness walk on CT-2's own contact terms with the table in place REFUSES: the H–O
   class falls `2.2 kT` below its fit floor with its minimum at 2.48 bohr, INSIDE the table's own
   knot range, which is why no inward fence reaches it and why the re-fit is in this freeze rather
   than the next one. Re-fit under the served walk, `P_HO = 17.643681` at `c = 2.36` (clamped by
   the gate, down 20 % from CT-2's 22.174044) and `P_HH = 0.745116` at `c = 2.34` (not clamped),
   54 of 64 within and the line at 2.7/2.9/3.1 Å three of three. **G-B0W returns `None`**: the
   whole served law is bounded in every class and positive at contact. LIQUID-2's admission gate
   is met.
5. **S2 (b) — the whole bond is not yet right, and the transfer term owns two thirds of the
   miss.** Filed before the solve: `ΔE_exact = −5.330669e-3` hartree. Measured: `−2.748015e-3`.
   The miss `2.583e-3` is inside the re-fit law's own band (`2.661e-3`) by 3 % and outside CT-2's
   comparative bar (`6.870e-4`) by nearly four times — branch (b). `1.723e-3` of that miss is
   S1's own, so the law over-binds this bond by 94 % of its depth and two thirds of the cause is
   the transfer term reading 8 % too deep, on a geometry where it is the largest term by an order.

| gate | verdict | the number |
|---|---|---|
| G-T0 — the interpolant is an interpolant | **PASS** | worst `9.992e-16` hartree at 60 sites (bar `1e-12`); the statement is about the 65×65 saddle solve's arithmetic and nothing else |
| G-T1 — every node of the map | **PASS** | worst `9.992e-16` at the 56 unmerged nodes; the 8 nodes in the 4 merged groups read their site's mean, worst `3.545e-4` = half the largest pole spread, per node with its spread beside it |
| G-B3 — the force is the derivative of the served term | **PASS** | worst relative `5.229e-11` at `h = 1e-5` over 5 atoms × 3 coordinates × 64 nodes (bar `1e-8`), worst at `twist_R2.7_t30`; the gradients sum to `4.770e-18` |
| G-C1 — the engine serves what the table says | **PASS** | worst `9.298e-16` (bar `1e-10`) on 64 nodes, two units on every one; the double count the serving rule refuses measured beside it at `2.505e-2` worst (`twist_R2.7_t120`) |
| G-A0 — the argmin's own seam | **measured, under its floor** | worst jump `1.418e-5` hartree, the donor turned 90.10° about its own oxygen at `R_OO = 2.7` Å, the contact passing between its two hydrogens; the refusal floor is `5e-4` |
| G-B0 — bounded, on the law AS SERVED | **REFUSED, one clause** | the SERVED walk (grid 41/axis, coarse+fine, `(p, q)` on the reachable set `\|q\| ≤ 1 − p²`): H–O falls to `−5.3611e-2` at 2.63 bohr, `2.2 kT` below its floor `−5.2315e-2`, minimum `−5.4336e-2` at **2.48 bohr**. It does NOT fall without bound and IS positive at contact. STRICT leg (shape `2.9158` carried inward): `−6.9029e-1` at 0.5 bohr, 665 kT, negative at contact — a term the law does not serve. NEAR leg (shape `1.1582` at or inside `r_min_oh`): `None`. LINEAR leg: `None` |
| G-B0F — the declared inward fence | **RUN and REFUSED HARDER** | `r_clamp = 2.402796` bohr: minimum `−1.1595e-1` at 0.5 bohr, 68.6 kT, negative at contact. The fence makes the walk WORSE because the spline's own r-extrapolation makes the shape SMALLER inward, so holding it at the knot floor keeps it higher (`−4.013e-1` served against `−8.736e-1` fenced at 0.5 bohr). Its other price, the force discontinuity at the clamp, `1.141e-2` hartree per bohr. And it could not have helped either way: G-B0's violation is at 2.48 bohr, ABOVE the innermost knot at 2.402796 |
| C1 — the contacts re-fit under the served gate | **BRANCH (a)** | constrained `P_HO = 17.643681` at `c_HO = 2.36` (clamped: yes), `P_HH = 0.745116` at `c_HH = 2.34` (clamped: no), 9 rounds to the fixed point, weighted residual `1.080329`, neither exponent at a grid edge; unconstrained `52.338389/2.74` and `0.003054/0.50` (`c_HH` at the grid's FLOOR — the grid's number, §5), weighted residual `0.652188`; ratio `1.656469` (under the 2 that would name the shape); 54 of 64 within `max(0.25·\|ΔE_exact\|, 5e-4)`, line 3 of 3; worst `1.164e-2` at `linear_R2.3`; 3,459,631 boundedness walks over 49 memoised radii in 15.8 s |
| G-B0W — the WHOLE served law | **PASS — `None`** | table + the re-fit contacts + the three walls + the charges, bounded in every class and positive at contact; the monotone walk names a dip at 2.95 bohr (`−5.1639e-2 < −5.1245e-2`), a bounded well, as CT-2's did. LIQUID-2's admission gate MET |
| G-R0 — the reach | **measured** | table `17.211` bohr at the `1e-10` budget, the law's `18.578` (the H–H contact's soft exponent) still the larger; LIQUID-1 Amendment 2's C² switch at `r_cut = 14.0` bohr unchanged |
| plant (i) — one knot moved by its own tolerance | **FIRES** | carrier `2.334195e-3` at `linear_R2.9`, served value `−9.336780e-3 → −7.002585e-3`, miss from the analytic reach `3.912e-16` (bar `1e-12`) |
| plant (ii) — the sign of the transfer term | **FIRES** | `1.867356e-2` against the analytic `2·\|table\| = 1.867356e-2`, miss `1.874e-16`; carrier `9.336780e-3 ≥ 1e-6` |
| S1 — the held-out bond's transfer term, forward | **BRANCH (a)** | measured `−2.140507e-2`; table (filed) `−2.312788e-2`, miss `1.723e-3` (8.05 %); family `−1.711144e-2`, miss `4.294e-3` (20.06 %); band `2.277e-3`; the table is nearer AND inside the band. Both are inside the CT tolerance `5.351e-3` |
| S2 — the held-out bond's FULL energy, forward | **BRANCH (b)** | measured `ΔE_exact = −2.748015e-3`; filed `−5.330669e-3` (field `−3.455329e-3`, seam `−1.875340e-3`); miss `2.583e-3`, inside the band `2.661e-3` by `7.87e-5` and outside CT-2's comparative bar `6.870e-4` |
| the solve | **priced, converged, inside the identity** | exact 17 iterations, residual `6.366e-9`, `2,376` core-s, 146 s wall; sector 14 iterations, residual `3.785e-9`, dimension `194,481` EXACT, metric's smallest eigenvalue `0.9227`, `2,080` core-s; `4,456` core-s in total against the freeze's priced band `3,337–5,151`; two units throughout; the order `E_exact ≤ E_noCT ≤ E_HL` holds |

## 1. The held-out bond, decomposed

The geometry: `twisted(2.7 Å, 20°)` then tilted `85°`, the donor unbent — coordinates
`r = 3.158687` bohr, `cos θ_d = −1`, `u·b̂ = −0.087156`, `q = +0.760226`. Three quarters of the way
from the twist sheet toward the acceptor's plane normal, on an azimuth and a twist angle NO node
of the map carries, `0.127562` in the scaled box from its nearest site (`tilt_R2.7_t90`), just
above the band floor `0.121140`.

| | mHa |
|---|---|
| `E_HL − monomers` (electrostatics + undeformed exchange) | +20.494 |
| closed-sector gain (polarisation and correlation, no electron crossing) | −1.837 |
| `E_CT` measured (`E_exact − E_noCT`) | −21.405 |
| **ΔE_exact** | **−2.748** |
| E_CT as a share of the remainder `E_CT/(E_exact − E_HL)` | **92.1 %** |

That share is the highest in the programme's table: CT-1 read 75–91 % over twelve geometries, and
this bond — the acceptor turned most of the way onto its plane normal at the map's shortest
separation — is 92 %. It is also why the node discriminates: the transfer term is the whole story
here, and the two candidate transfer terms disagreed by `6.016e-3` hartree, more than twice the
band, before the solve.

## 2. What S1 says, and what S2 says

S1 is the campaign's third landed forward prediction and its first head-to-head. The two
predictions were filed together, they straddled the truth, and the gap between them was larger
than twice the band — so the solve could not have failed to separate them, and it separated them
in the table's favour: `1.723e-3` against `4.294e-3`, and the table inside the gauge it staked on
itself before it knew the answer. The direction is the one the map predicts: the family, which has
no azimuth, under-binds a geometry pointed at the acceptor's plane normal by 20 %; the table,
which has the azimuth but no knot at this one, over-binds by 8 %.

S2 is the honest counterweight and it is a branch (b) by three percent of its own band. The whole
law over-binds this bond by `2.583e-3` on a measured `2.748e-3` — 94 % of the bond's depth. The
decomposition is not ambiguous: `1.723e-3` of it, two thirds, is S1's own miss on the transfer
term, and `0.860e-3` is everything else the law does at this geometry. A term that is 92 % of the
remainder and 8 % too deep produces most of a doubled bond by itself. Rule 6 counts S1's landing
as support for the TABLE where its data are; it does not count S2's (b) as support for the law,
and the law is what a liquid would run on.

## 3. The three legs of G-B0, and what the fence taught

The freeze's ruling was that the boundedness walk must be run on the law AS SERVED, not on an
outer knot's shape carried inward, and the three legs beside the gate are what made the verdict
readable. The strict leg refuses at 665 kT and ends at `−0.690` hartree at contact; the served
walk refuses at 2.2 kT, does not fall without bound, and is positive at contact. Those are not the
same verdict wearing one word, and only the second describes anything the dynamics would
integrate.

The declared fence was built exactly as specified, run, and **failed harder than no fence at
all** — because the spline's own extrapolation makes the shape SMALLER inward, so holding it at
the innermost knot keeps it higher than letting it run. That is a fact about this table that no
argument would have produced. And it was irrelevant either way: the violation sits at 2.48 bohr,
above the innermost knot at 2.402796, INSIDE the table's measured range, so it is a statement
about the map's own data and no inward rule touches it. The lever was the contact clamp, C1 found
it, and G-B0W closed it.

## 4. Bookkeeping, declared

- The served walk on CT-2's contact terms was RUN and REFUSED before the freeze was written, and
  the freeze says so in §0 in those words. C1 is in this freeze for that reason and not because a
  postmortem wanted it.
- The C1 fit's UNCONSTRAINED companion puts `c_HH` at `0.50`, the grid's FLOOR. The freeze's rule
  is that a parameter at a grid edge is reported as the grid's number and not the physics'; it is
  reported here. The CONSTRAINED fit — the law — has neither exponent at an edge.
- S2's two bars swapped order between filing and reading, and the freeze's tighter-first rule is
  what kept the branches coherent. At filing the band `2.661e-3` was looser than the comparative
  bar `1.333e-3` computed on the PREDICTED total; at reading the bar is recomputed on the MEASURED
  total and tightens to `6.870e-4`, so the band stayed the looser of the two and branch (b) means
  what it says. Had the ordering been fixed by assumption in either direction, one branch would
  have been unreachable.
- A reporting bug of this lane's, found and fixed before any prediction was filed: the C1 block's
  worst-miss variable shadowed G-C1's, so `gate.json` briefly recorded `g_c1.worst` as C1's
  `1.164e-2` for a gate whose true worst is `9.298e-16`. The VERDICT was computed before the
  shadow and was never wrong; the number was. Renamed and re-run.
- `CT2_RESULTS.md`'s gate table prints the H–O contact amplitude as `22.17048` where
  `wall_ct2.json` carries `22.174044` — a transposition in the prose. Every number here is the
  record's.
- The engine's `bounded`/`hole` are unchanged in letter; `bounded_ct`/`hole_ct` take the transfer
  row from the caller and `bounded`/`hole` delegate to them with `charge_transfer`, so every
  caller of record reads the same walk it always did (unit-tested on CT-1's and CT-2's laws).
- `ct_table_on` is `false` in every record written before this campaign, so every one of them
  still reads `Pair` or `Angular`, bit for bit; checkpoint v14's seam-off identity in
  checkpoint bytes is gated and passes.
- `solve` refuses to run without `prediction.json` on disk. It ran with it, detached, on cores
  0–23, and wrote its own marker.
- No number enters from outside the engine, its own solver and the map's own records. The one
  coefficient the table does not hold as a knot is `c₀ = c_ct = 1.400000` per bohr from
  `ct2/wall_ct2.json`, with its path in `ct_table.json`.

## 5. What this reads for the next freeze

The transfer term is measured and served, and the law around it is bounded — that is what
LIQUID-2 was waiting on, and `ct3/wall_ct3.json` is the law it would run. What S2 (b) says is that
a term right to 8 % on a geometry where it is 92 % of the remainder is not yet a bond right to a
quarter, and the two places to look are named by this campaign's own numbers rather than guessed:
the map has NO knot between its two azimuth sheets at high tilt, which is exactly where the
held-out node sat and where the table's 8 % lives; and the map has no node with a pair donating in
BOTH directions at once, which a liquid has and the serving rule cannot represent. Neither is a
defect of the interpolant. Both are gaps in the map, and both are answered by nodes, not by shapes.


## Correction (2026-09-07): the per-dimer serving rule is not dynamics-grade, measured in LIQUID-2's instrument phase

CT-3 served one transfer reading per unordered pair of units at the pair's SHORTEST cross-unit
H···O contact, an argmin, and G-A0 measured the argmin's own discontinuity on one dimer at
`1.418e-5` hartree per handover, under the `5e-4` floor. In the 128-water box (LIQUID-2's
labelled screen, `liquid2/DRIFT_NOTE.md`, every file `dry: true`) the served table costs three
orders of magnitude of energy conservation: drift peak `6.585e-3` hartree at the tables' step
against `4.720e-6` with channel 6 off on the same box, non-monotone in the step. The cause is
CONFIRMED as the handover: 3,803 handovers in 2,000 frames, mean absolute jump `9.03e-6` (the
dimer's order), worst single jump `2.01e-3` (142× CT-3's whole-dimer worst), and the running
extremum of the SIGNED jump sum `6.361e-3` against the measured drift peak `6.585e-3` — 96.6 %
of the drift, the same statistic the ledger accumulates; the residual 3.4 % is not claimed and
names the two untested suspects (the switch on the contact distance while the table's coordinates
carry `r`; the below-knot tail's gradient). The table's VALUES stand (S1 (a) is a reading on a
dimer, where no contact changes hands); what does not stand for a liquid is the argmin. The owed
rule, specified in `DRIFT_NOTE.md` and not built: a partition of unity over the four cross-unit
contacts, `w_k = e^{−βr_k}/Σ_j e^{−βr_j}`, `β` derived from the map's own shortest-to-second
contact separations and the table's resolution floor, the gradient analytic on every atom of
every contact, gated on the served law's drift at the tables' step within 2× of the channel-6-off
control. LIQUID-2 does not freeze until that gate passes.
