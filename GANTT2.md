# GANTT2 — the plan by tier (2026-09-06)

*`GANTT.md` is the build graph as it accreted, campaign by campaign, and it stays as the
record (the workbench cites its lines). This file is the plan: what each tier of the zoom
ladder has, in the ledger's own words, what production of the closure grammar it carries, and
the one campaign that moves it next, with its kill. Same laws as GANTT.md — a node with no
receipt-gate is a wish; no timelines; size is compute × scope — and one more from this
season: every tier is asked the same four questions (what is closed, what the ledger carries
between closures, what production rewrites a closure, what runs the tier's dynamics), and a
tier's row is complete when all four have a record or a named fence.*

## The vocabulary this plan is written in

| | plain name | physics name | kind | rate | what it does between two closed wholes |
|---|---|---|---|---|---|
| 1 | presence | field | Circumstances | R⁻¹ | what a whole simply is to others at a distance |
| 2 | accommodation | induction | Structure | R⁻⁴ | how each reshapes to make room for the other |
| 3 | attunement | pair dispersion | Process | R⁻⁶ | two flickering in step |
| 4 | concert | three-body dispersion | Rules | R⁻⁹ | what three do that no pair does |
| 5 | refusal | exchange | Identity (hard edge) | e^{−br} | this space is taken |
| 6 | sharing | charge transfer | Identity (soft edge) | e^{−cr} | electrons lent while both stay whole |

The closure grammar (Backpass V; the compiler-for-matter): terminals are atoms; a
**production** rewrites closures when their ledger satisfies a condition, and the merged form
is real when the dynamics never splits it (`Core/Closure.lean`) and its binding pays its rent
(`Core/Maintenance.lean`). Three productions are in play: **unit** (atoms → a molecule; the
closure reading), **bond** (two units → a pair; the seam ledger's condition), **rewrite** (a
unit changes composition; the identity boundary crossed). A tier's phase is the grammar's
fixed point: a spanning bond graph is a liquid.

## The ladder, bottom to top

### The fold below the atom (hadron) — see `GANTT.md` GF nodes and `OBJECT.md`; unchanged this season

### Atom — `TIERS.md`: the ladder climbed, node H decided (the bare many-body expansion does not terminate; the seam is the answer)

### Molecular — CERTIFIED (the water unit is a closed view: CIRISHolon FIELD-3, the closure test 893.8 fs; held under contention in the liquid: 128 units × 100,000 frames)

| question | record |
|---|---|
| closed | the unit production: an oxygen and the two hydrogens its O–H curve holds lowest (`assign_units`); its boundary at H···O 1.95 bohr where the reading hands a hydrogen over (FIELD-8) |
| the ledger between units | all six measured on the dimer, every coefficient from exact solves: presence (FIELD-1), refusal over 66 orientations (FIELD-6–9), sharing on a 64-node map (CT-1/2: 88 % of the remainder, follows the acceptor's plane normal), accommodation and attunement folded or read zero at this basis, concert priced and refused (the trimer: 2.9e9 determinants) |
| the bond production | measured as retention: the dimer holds its bond 55 % of frames at 293 K, 100 % at 150 K (CT-2 S3); its condition is the rent clause, not a threshold |
| the rewrite production | the identity boundary crossed under a stacked law (LIQUID-1 screen MAXBOND): a proton transfer the referee cannot hold — OH⁻ is unbound at this basis (fence I-5) |
| dynamics | the seam law, bounded (G-B0 passed first at CT-2), two forward predictions landed inside a quarter |

**Next, in order, each one freeze:**
- **CT-3 — the term as a table.** The 64-node exact-minus-closed-sector map served as the transfer term (a two-angle, one-distance table, the way the pair curves are served), no declared family; one new held-out node (~2 minutes on the product-start solver). Kill: the table misses the held-out node by more than the family did.
- **ION-1 — the cation half of the rewrite.** H₃O⁺ as a unit the reading may form; its presence from its own density; the seam H₃O⁺·H₂O on a dozen geometries (9.0M determinants, ~12 minutes each on the product-start solver — node C's compute fence, lifted); the hop production under the rent clause. Kills: the shared-proton geometry against Zundel/Eigen; the excess proton's diffusion against experiment (~5× water's). The anion half waits on a basis (I-5).
- **A basis lane (I-5).** One basis with virtual p on hydrogen and a diffuse function on oxygen: attunement stops reading zero and OH⁻ binds. Priced by the MPS route (channel 3/4 sizing: the closed sector 566 GiB at 20 orbitals on the determinant route; `price_mpo(21) = 1.76` GiB provisional).

### H-bond network — INSTRUMENTED, the first counted arm reading (LIQUID-1, verdict pending at frame ~90,000 of 100,000)

| question | record |
|---|---|
| closed | the unit production holds at liquid density: 128 units every pass (L0) |
| the ledger | the dimer's six channels under Ewald with a C² switch at 14 bohr (Amendment 2); the truncation priced at 1e-6 Ha per water |
| the bond production | 1.17 distinct bonds per molecule at 293 K (2.3 in experiment's both-ends convention against 3.5); the oxygen shell water-shaped (peak 3.0–3.1 Å, height in band, 3.6–4.0 neighbours in 3.3 Å against 4.5): the neighbours are there and pointed too loosely — the deficit is orientation, majority the law's, a quarter the criterion's (screens) |
| the phase | above the four-connected percolation threshold (~0.8 distinct bonds); the spanning-cluster fraction is the readout that names it and is not yet in the runner |
| dynamics | the price 0.23 s per force pass at 384 atoms on one core; the box found the engine's periodic three-body force bug (fixed, correction 4) |

**Next:** LIQUID-1's verdict, then **LIQUID-2** on CT-3's table term with **the spanning-cluster
fraction** as a readout and R2's band stated in the lens's own convention (a correction to
the freeze's letter: the band was typed in the both-ends convention); accommodation at the
box (FIELD-2's fixed point over 128 units) as its first pre-committed branch, dispersion from
the basis lane as its second. Kill bands unchanged.

### Fluid element — READ (FLUID-0): the single-species lattice gas cannot carry the liquid's two coefficients

| question | record |
|---|---|
| closed | node LG: the tier certifies as its own object; its block charts do not close except by conservation (the defect is the boundary fraction, exactly) |
| the ledger | transport: viscosity and tracer diffusion for all 4,608 REG+ laws; the Schmidt number 0.107–0.449 against water's 435 — no closure in the carrier, no water |
| the bond production | none: a single-species gas has nothing to trap a particle in |
| the phase | Sc ≫ 1 is the tier's closure ratio: particles trapped while stress passes |
| dynamics | integer-exact; the price 25.8 s per law at L = 256 |

**Next: FLUID-1 — the two-species carrier.** A lattice gas with bound pairs or a tracer species whose
diffusion is set apart from the momentum's; its Sc swept; the chart that maps LIQUID-1's D
onto it and a viscosity the engine does not yet measure (a pressure tensor and its
autocorrelation — a LIQUID-2 readout). Kill: no two-species law reaches Sc ≥ 100 either.

### The cube (1 km) — FENCED by name; goes live as the rungs beneath it certify (WORKBENCH_FSD §11.2)

## The page (the .io workbench) — WB-9, one pass after LIQUID-1 reads

The engine on the page is rebuilt from the deployed commit and carries everything above; the
page's manifest does not. One pass, under the FSD's own rules (no tier fakes; a band runs its
certified physics or wears its fence with owner and exit; 431 smoke checks and counting):

1. **The ledger by channel.** A panel of the six rows with their plain names, kind, reach and
   the row's live value (new exports `holon_channel_count / plain / kind / reach / value`), and
   the account's six kinds as the panel's columns where a record exists — the six-by-six on
   screen, cells citing their records.
2. **H-bond network**: from `fenced` to `measured` with LIQUID-1's readouts cited (the peak, the
   bond count in both conventions, the spanning-cluster fraction when LIQUID-2 has it); a small
   live box the wasm can afford at interactive rates (a 16-water cell), with the 128-water
   arm's numbers beside it and the price that keeps the large one off the page.
3. **Fluid element**: its finding on the face — the census's Schmidt range against water's,
   the closure-ratio reading, FLUID-1 as its exit.
4. **Molecular**: the six channels' status from the seam campaigns; the closure boundary;
   the two forward predictions.

Receipt-gate: the smoke passes with the new bands' cites resolving to the records above;
`pages.yml` deploys it; the page's manifest names this file's tier rows.

## Standing laws — as GANTT.md, plus

**The four questions.** A tier's row is complete when closed / ledger / production / dynamics
each have a record or a named fence with owner and exit.

**The screen law (2026-09-06).** A labelled screen (`--screen`, every file `dry: true`, the
knobs declared) may turn any knob to choose which hypotheses a freeze pre-commits as
branches; nothing a screen writes enters a gate or a claim.
