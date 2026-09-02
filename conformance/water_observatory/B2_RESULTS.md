# B2 — THE LONG-RANGE PAIR SUBSYSTEM, RESULTS

*Freeze: `B2_PREREG.md`, ADMITTED by `Audit/prereg_audit.py`, committed at `f2539f1` — one
commit BEFORE `longrange.rs` existed. Same ordering discipline as B1 and B1b, same proof:
`git log --oneline`, quoted in §8.*

**Subsystem:** `engine/crates/holon-render/src/longrange.rs`
**Instrument:** `engine/crates/holon-render/examples/b2_longrange.rs`
**Raw output, committed beside this document:** `b2_frames.log` (every frame),
`b2_frames_stride400.log`, `b2_engine_full.log`, `b2_engine_hh.log`, `b2_refusals.log`,
`b2_tests.log`, **`b2_engine_full_G9RED.log`** — the run in which G9's `f = 0.90` arm fired,
kept because a fired gate stays in the record with what fired it — and
**`b2_engine_postdoor.log`**, the same arm re-run against `scale_box`'s new door.

---

## 0. VERDICTS

| question | answer |
|---|---|
| Is the method Ewald? | **No, and the reason is a fact about this force law.** §1 |
| What was B1b's discard, really? | **G1: S-DOMINANT at S/T = 9.9e8. A RADIUS-BOOKKEEPING defect** — branch (a) |
| Is B1b's bill paid? | **G14: PASS, 0 of 8 seeds over the criterion**, residual 3.1e-8 – 1.3e-7 Ha against criteria of 4.6e-6 – 2.5e-5 |
| Does the tail model apply? | **G3: the one curve that matters ADOPTS**, `p_fit = 5.0049`; the other two are FENCED and nothing rests on them |
| Do the three conservation laws hold? | **G4, G5, G6 PASS**, complete and truncated — but **G4's arm is VOID under V2** because its staked plant cannot fire it |
| Fired gates | **G7** (coarsest staked step only) and **G8** (saturated by an unresolvable reference; filtered it still reads 1.19× the bar). Both kept fired, neither retuned |

**B1b's fired G1b stays fired**: 3 of 8 seeds, worst 2.496×. Nothing here erases it.

---

## 1. THE METHOD, AND WHY IT IS NOT EWALD

`Sim::compute_forces` carries no electrostatic term. Nuclear charge appears only as a species
label resolved into a bank slot (`holon-render/src/lib.rs:914`); the only `q_a q_b / r` in the
tree sits within the electronic-structure solver that GENERATES the curves
(`holon-chem/src/md.rs:588`). Ewald, PME and Wolf-style damped sums all exist to evaluate a
**conditionally convergent** `Σ 1/r` lattice sum — the reciprocal-space split, the
neutralising background and the surface term are there because that sum has no absolutely
convergent value and its answer depends on the order the images are added in. **That sum is
not present.**

B2 builds a split kernel whose far part is absolutely convergent on its own image lattice.
The licence — `p > d` — is MEASURED by G3 and REFUSED by R1, never assumed. The ionic `r⁻¹`
case is exactly where `p ≤ d` lands, in 2D and 3D alike, and R1 refuses it by name with Ewald
or PME stated as the exit. **Node C's ionic scenes are neither shown nor excluded by anything
here, and R1 is what stops a green B2 gate being read as coverage of them.**

---

## 2. G3 — THE TAIL EXPONENT, MEASURED FROM THE TABLE'S OWN KNOTS

Staked before the fit existed, with every band's meaning written down, because a
minimal-basis FCI curve may or may not have reached its asymptote inside its own support and
the author does not get to decide afterwards.

| slot | curve | `r_max` | `\|u(r_max)\|` | `p_fit` | fit residual | `exp_index` | band |
|---|---|---|---|---|---|---|---|
| 0 | H–H | 10.2400 | 6.272736e-9 | 20.6703 | 3.3123e-1 | 31.1423 | FENCED |
| 1 | O–H | 10.2400 | 1.645191e-10 | 30.6918 | 2.8526e-1 | 42.5440 | FENCED |
| 3 | **O–O** | **20.0000** | **6.641237e-7** | **5.0049** | **3.7181e-5** | **5.0038** | **ADOPTING** |

**The curve that matters is the one that adopts.** O–O is the only curve whose support
reaches past `c* = 15.0` — the mechanism B1b named — and its tail is a power law at index
**5.0049**, fit residual 3.7e-5, with the table's own exponential extrapolation index
agreeing to four digits. For a pure power law those two numbers are identically equal, so
their agreement is the check rather than a coincidence.

H–H and O–H are FENCED at slopes of 20.7 and 30.7 with fit residuals of 0.33 and 0.29 — an
exchange-dominated tail, measured. **Nothing rests on them**: their supports end inside the
cutoff, so no truncation this campaign is about reaches them, and at 20 bohr both are worth
under 1e-21 Ha by either model. R4 is live on the fenced pair.

**What index 5 is not.** A measured property of a committed table at a stated level of
theory, not a dispersion coefficient. FCI in a minimal basis (`n_basis = 10`, `n_det = 2025`)
underestimates dispersion badly, and the solve exits `IterationCap` at 5000 iterations with
worst residual 4.809e-6. Every constant fitted from it inherits that, and R5 refuses to emit
one without those fields beside it.

---

## 3. G1 — THE CHANNEL SPLIT, AND WHAT B1b'S NUMBER ACTUALLY WAS

Every frame of every admitted seed — 160,000 frames, all 8 seeds admitted against
`census_traj_manifest.sha256` by digest, 0 refusals.

**The instrument reproduces B1b bit for bit.** All eight `max|E_switch(c*)|` values and their
frame indices match `B1B_RESULTS.md` §2 exactly, including the worst: seed
`0x0000000053415424`, frame 10144, **1.150526e-5** Ha. `max|T|` on that seed reads 2.239050e-6
against B1b's `E_tail` max of 2.239049e-6.

At the worst frame:

```
E_switch  -1.150526e-5    S  -1.150526e-5    T  -1.166250e-14    signed sum  -1.150526e-5
S/T = 986,516,935
```

> **BRANCH (a) — S-DOMINANT. B1b's headline discard is a RADIUS-BOOKKEEPING defect.**

Essentially all of it is channel S: real tabulated interaction in `(15, 20]` bohr, thrown
away because the cell-list radius was set by a THREE-BODY table (`water::R_HI = 15.0`) while
the pair curve reaches 20.0. Channel T — the part past the table's edge, where a long-range
METHOD is the answer — is **nine orders smaller** at the frame that decides the verdict.

The freeze wrote this branch's meaning before the measurement, and it is honoured here:
**B2 does not claim to have solved a problem that was a radius.** The cure for channel S is
`R_s ≥ max r_max` and R3, which makes B1b's configuration unrepresentable. What remains for
the long-range method proper is the periodic case of §1.4 of the freeze — where no legal
single-image radius exists at all — and the tail model's honesty about an extrapolation that
was never computed.

Reported separately rather than as a signed sum, per M-LOOP-BLIND: max`|S|` over all frames
1.150526e-5 (seed …5424), max`|T|` 2.239050e-6 (same seed).

---

## 4. G2 — THE NEAR SECTOR'S COVERAGE

**Exact half: PASS.** Pairs in `(c*, r_max]` that a list built at `R_s = 20.0` would miss:
**0**, over 108,599 such pairs across 160,000 frames. EXACT, as staked.

**Energy half: PASS**, and non-vacuously. 512 atoms on a 3D cubic lattice in a box grown
until `Sim::route()` reports `Cells`, with the in-range pair population counted
independently: the cell decomposition and the complete enumeration return the same `e_pair`
to the bit, relative difference `0.000e0`.

That scene had to be built for the gate, and the reason is §7's first correction.

---

## 5. G14 — B1b'S BILL

The residual is a per-frame quantity maximised over every frame, not a difference of maxima
that fall on different frames. Two parts, both counted: pairs past `R_f`, which nothing
reaches, and the tail model's disagreement with the extrapolation it replaces on the pairs it
does carry — a pair carried wrongly has not been paid for either.

| seed | B1b's max `\|E_switch\|` | `0.10·D_s` | B1b ratio | residual after B2 | beyond `R_f` | model gap | **B2 ratio** | paid |
|---|---|---|---|---|---|---|---|---|
| …5421 | 6.811667e-6 | 1.500e-5 | 0.454 | 9.800065e-8 | 0 | −9.800065e-8 | **0.0065** | yes |
| …5422 | 9.112721e-6 | 4.800e-6 | **1.898** | 1.317522e-7 | 0 | −1.317522e-7 | **0.0274** | yes |
| …5423 | 8.846165e-6 | 5.620e-6 | **1.574** | 9.987434e-8 | 0 | −9.987434e-8 | **0.0178** | yes |
| …5424 | 1.150526e-5 | 4.610e-6 | **2.496** | 1.338921e-7 | 0 | −1.338921e-7 | **0.0290** | yes |
| …5425 | 7.635210e-6 | 2.450e-5 | 0.312 | 3.363819e-8 | 0 | −3.363819e-8 | **0.0014** | yes |
| …5426 | 1.109996e-5 | 1.130e-5 | 0.982 | 3.051820e-8 | 0 | −3.051820e-8 | **0.0027** | yes |
| …5427 | 1.001624e-5 | 2.260e-5 | 0.443 | 6.568698e-8 | 0 | −6.568698e-8 | **0.0029** | yes |
| …5428 | 7.611945e-6 | 1.280e-5 | 0.595 | 5.278043e-8 | 0 | −5.278043e-8 | **0.0041** | yes |

> **G14 PASS: 0 of 8 seeds over the criterion.** The three seeds B1b failed — 1.898, 1.574,
> 2.496 — come in at 0.0274, 0.0178 and 0.0290, between 58× and 86× under the criterion they
> broke.

**`beyond R_f` is EXACTLY ZERO on every seed, and that is a fact about this box rather than a
result.** `R_f = 73.2697` bohr at the declared budget of 1e-9 Ha, and the census box's
diagonal is 40.37 bohr, so no pair is ever outside the far sum's reach. **What the subsystem
leaves behind in this box is not a discard at all**; it is the tail model's disagreement with
the exponential it replaces.

**Which makes the residual a BRACKET, exactly as R4 requires**, and the table's two zero
columns are its two ends: at the exponential end the subsystem differs from the complete
table sum by **0**, and at the power-law end by `|model gap|`. Both ends are under the
criterion, which is why the verdict is not itself bracketed.

**What G14 does NOT say.** It does not say the far sector is unnecessary — it says that at
these eight seeds, in this box, at this budget, what B2 leaves behind is 1–3% of the incurred
drift where B1b left 25%. A box large enough to put pairs past `R_f` would move `beyond R_f`
off zero, and the N-scaling curve (§6) is the only thing measured about larger boxes.

---

## 6. THE CONSERVATION BATTERY, AND ITS PLANTS

One gate per law, each staked on the quantity its own law constrains. Numbers from
`b2_engine_full.log` unless marked.

| gate | complete near sector | truncated near sector (the O(N) route) |
|---|---|---|
| **G4** energy | drift peak 5.587362e-5 vs bound 6.242874e-3, ratio 0.0089; `work_columns_ok` | ratio 0.0109 |
| **G5** momentum | 1.387608e-13 vs bound 8.118668e-10; isolated pair `\|F_i + F_j\|` **EXACT 0** | 1.310899e-13 |
| **G6** angular | 3.166929e-12 vs bound 2.123218e-8 | 2.779653e-12 |
| **G12** work count | 20000 steps, 267,581 far contributions, 23 `R_s` crossings | 20000 steps |

**The truncated arm is the configuration B2 exists to make safe.** The window is derived at a
floor of `|u(r_max)|`, which puts its inner edge exactly on the curve's support — `(20.0000,
22.0000)` bohr against `R_s = 20.0000`, the tightest legal handover, read off the curve rather
than chosen. All three laws hold there too.

**G10 image convergence: PASS** at 3 shells with the difference emitted as
`uncertainty_hartree`. **G9 stale-cache: PASS bit-identical in BOTH directions** (`f = 0.90`
and `f = 1.10`), with **P1 still firing** at a carrier of 7.332040e-7 Ha, so the gate keeps
its power. **G13 N-scaling: exponent 2.123**, monotone in N over 12–192 at fixed
number density, 3 repetitions, CPU time, pinned to an E-core. That is `O(N²)`, which is what a
complete far sum is; **the far sector as built does not buy an `O(N)` far route, and nothing
here claims it does.** **G11: 10 of 10 refusals fire**, including the negative control — the
periodic size refusal staying silent on a box that clears `2 R_s`, with the shells genuinely
converging at 2. **G-LEDGER** (added after the freeze; see §7) closes to `0.000000e0` on both
box moves with `w_ext` matching `work.hand`.

### 6.1 The plants

| plant | carrier | sector | verdict |
|---|---|---|---|
| **P1** stale image lattice | `\|E_far(stale) − E_far(fresh)\|` = 7.332040e-7 Ha | periodic image sector | **FIRED** |
| **P2** one-sided far force | `\|P − P₀ − J\|` = 1.150051e-5 | far pair force | **FIRED** (G5 FAIL) |
| **P3** non-central far force | `\|L − L₀\|` = 4.704846e-7 | angular | **FIRED, and G5 STAYED GREEN** |
| P4 zero-point step at `R_s` | 1.0e-6 Ha per crossing, 23 crossings | energy ledger | **did NOT fire** — §7 |
| **P5** gradient mismatch | see §7 | far force | fires on H–H; **cannot discriminate on the full arm** |
| **P6** truncated far sum | omitted band 4.074994e-8 Ha | far pair sector | **FIRED** |
| **P7** omitted virial | `Σ r du/dr` = 2.171990e-7 Ha | virial | **FIRED, and G4/G5 STAYED GREEN** |

**P3 is the result this battery was built for.** A pairwise force that is equal and opposite
but NOT central conserves linear momentum exactly and destroys angular momentum. P3 fires G6
while G5 stays green — the demonstration, not the assertion, that the two gates are
independent, and the reason they are separate rows. P7 does the same for the virial against
both conservation gates: a channel can be perfectly conservative and still be missing from
the pressure.

---

## 7. THE FIRED GATES AND THE VOID ARMS, KEPT AS THEY READ

Reported as plainly as the passes. **None retuned.**

### G7 — fires on its coarsest staked step, and only there

`h = 1e-3` reads 3.8381e-5 against a criterion of 1e-6; `h = 1e-5` reads **3.8386e-9**, and
the Richardson extrapolation equals the virial. The fired prong is the central difference's
own `O(h²)` truncation error rather than a missing virial term, and P7 discriminates cleanly
at the finest step (6.0686e-5 planted against 3.8386e-9 clean, four orders). **The freeze
gates all three steps, so the gate is fired.** The successor's fix is the criterion's SHAPE —
a tolerance that scales with `h²`, staked with its convergence argument — never its value.

### G8 — fires on a statistic that is measuring the reference, not the gradient

Worst relative error **1.0000e0** against 1e-7 over 100 configurations. The diagnostic printed
beside it identifies the cause without moving the verdict:

* the worst component's analytic force is `2.442e-23` Ha/bohr and the NUMERIC reference is
  exactly `0` — the finite difference underflowed, because `|F|·2h ≈ 5e-28` inside a sum of
  magnitude `|E_far| ≈ 4e-8` is 1e-20 relative, twenty orders under double precision;
* a max over per-component RELATIVE errors pins at exactly 1.0 the moment one component does
  that, and **no plant can move a saturated maximum** — which is why P5's carrier equals the
  clean run's value on this arm. P5 discriminates normally on the H–H arm (9.9900e-4 planted
  against 4.2616e-6 clean);
* **the far force is not missing.** The largest absolute disagreement anywhere is 1.087e-17
  Ha/bohr against a largest far force of 1.148e-8 — **9.470e-10 of the force scale**;
* filtering to the components whose disagreement the reference can actually resolve (a
  disagreement above `4·eps·|E_far| / 2h`, its own noise) removes **1909 of 2096** components
  on the full arm. **Over the 187 that remain, the worst relative error is 1.1870e-7 —
  still OVER the staked 1e-7, by 19%.** On the H–H arm the same filter gives 3.0392e-8, which
  is inside.

**So the filtered statistic does not rescue this gate, and it is not offered as if it did.**
The saturation at 1.0 is an artifact of the reference; the residual 1.19× overshoot on the
full curve set is not, and it is a real reading against a criterion the freeze staked. What
the diagnostic buys is that the two are now separable, and that the far force is bounded from
the other side: the largest absolute disagreement anywhere is 9.470e-10 of the largest far
force.

**The gate is fired on its staked criterion.** The successor's fix is a criterion that is
well-posed for a quantity formed as a difference of two nearly equal functions — the far term
is exactly that by construction — staked in advance with its resolvability condition. Nothing
here proposes moving the number.

### G4 — PASSES, and its arm is VOID under V2

P4's 1.0e-6 Ha zero-point step moves the drift peak by exactly the planted amount but the
derived bound is 6.24e-3, so the energy gate cannot resolve it. The **power certificate** — a
sweep that is a measurement OF the gate and never a new criterion — says G4 resolves a step at
**1.0e-2 Ha**, a factor of **10⁴** above the staked plant. Under the freeze's own V2 that
VOIDs G4's arm until a successor stakes a plant this gate can see. G4's readings are printed
above because they are real; they are not scored.

### The periodic arm is VOID by construction on the hydrogen curve set

At `p_fit = 20.67` the far sum reaches `R_f = 11.19` bohr while the smallest legal wrapping
box is `2 R_s = 20.48`, so no legal periodic box can place an image in range. **A property of
the tail, not of the instrument**: a kernel that steep has no long-range content for an image
sum to carry. On the full curve set `R_f = 73.27` against `2 R_s = 40.00` and the arm scores.

### G9 — now PASSES both directions, after two stacked defects it exposed

`f = 0.90` and `f = 1.10` both pass bit-identical (`-1.565574861693e-6` and
`-6.147355490814e-7`, scaled against fresh), and P1 still fires at 7.332040e-7 Ha, so the gate
kept its power through the fix. `f = 0.90` FAILED on the first full run, and the cause was two
defects stacked:

1. **The instrument swallowed a refusal.** `0.90 × 41.0 = 36.9` bohr is below the far sector's
   `2 R_s = 40` legality floor, so the fresh sector correctly REFUSED — and the gate scored
   that refusal's empty energy as a disagreement, because the call site read
   `let _ = ffs.resolve_shells(...)`.
2. **The far sector rebuilt its lattice at a STALE SHELL COUNT.** A shrunk box needs MORE
   shells to reach `R_f`; rebuilding the offsets at the old count reaches only `f` times as
   far, which reads as a silently truncated far sum rather than as an error.

Both are fixed at `4d25135`: `accumulate` re-resolves the shell count when the box key
changes and refuses when a scaled box falls below the legality floor, surfaced through
`Sim::far_ok`; the instrument no longer swallows the error at any of its four call sites; and
the periodic arm's box is sized at `2.05 R_s / 0.90` = 45.5556 bohr so it stays legal on both
sides of the move.

**The engine-level finding underneath it was worth more than the gate, and it is now
closed.** `Sim::scale_box` shrank the box affinely and **re-checked no legality condition
afterwards** — not this sector's floor, and **not `Sim::pbc_ok` either, which was consulted
only by `Sim::set_pair_cutoff`** (`sim.rs:1781`). A barostat move could therefore carry a
periodic scene from `pbc_ok` true to false with no complaint anywhere, breaking the
minimum-image convention silently: the pair reduction would drop an image force, a wrong
number rather than an error.

**The door was built at `44ac404`** by the integrating lane, after B2 routed the finding to
it: `scale_box` now refuses any factor that would put the list cutoff past half the shortest
scaled edge on a wrapping boundary (`ScaleRefusal::BreaksPeriodicImages`), mutation-tested,
with the numbers behind the refusal readable at `Sim::pbc_margin`. **The two halves live in
two places on purpose** — the pair sector's `pbc_ok` condition is the door's, and the far
sector's own `min_edge ≥ 2 R_s` is not, so it stays a per-pass check exactly as designed.

Three of this instrument's probes walk through that door — G9 scales by 0.90 and 1.10, P1 by
0.90 — and each call was an `.expect(...)`, so a refusal would have surfaced as a PANIC
rather than a reading. They now print the refusal and mark the arm VOID, **and they print the
margin**: this arm's box clears the door by 2.5% (`list_cutoff` 20.0 against a scaled
half-edge of 20.5). **DEMONSTRATED, not predicted** — `b2_engine_postdoor.log` re-runs the
whole arm against the live door and no probe hits it: G9 passes bit-identically in both
directions at the same values as the pre-door run (−1.565574861693e-6 and
−6.147355490814e-7), P1 still fires at 7.332040e-7, G10 still converges at 3 shells. The 2.5%
is additionally pinned by a test with a negative control — a box just under the floor IS
refused — so the margin fails loudly if either constant moves. The two floors coincide here ONLY because no pair truncation is declared,
so `list_cutoff == R_s`; a scene that declared one would push `list_cutoff` past `R_s` and
`pbc_ok` would bind first. A margin nobody prints is a margin nobody notices moving.

---

## 8. THE ORDERING PROOF, AND THE SUITE

The freeze landed before the subsystem existed; `git log --oneline` on `b2-ewald` is the
check, and `Audit/prereg_audit.py` returns `ADMITTED B2_PREREG.md`.

`cargo test --release -p holon-render`: **21 binaries, exit 0**, `t3_replay` included — so
every replay fingerprint banked before B2 still validates. That is the receipt for `e_far`
being an EXACT 0.0 when no far sector is declared, and for the far sector's declaration
entering `physics_digest` in a way that writes nothing when there is none.

`tests/b2_longrange.rs` adds 21 tests of the sector's arithmetic and refusals at suite cost,
the last of which is a mutation check: every invariant the file asserts is re-run under the
plant that should break it, and the plant must move the quantity the invariant reads.

---

## 9. WHAT THIS BANKS, AND WHAT IT DOES NOT

**Banked:**

1. **B1b's discard was a radius, not a tail.** G1 reads S/T = 9.9e8 at the deciding frame.
   The cure is a list radius that respects every loaded pair curve's support, and R3 makes the
   old configuration unrepresentable.
2. **The bill is paid.** G14 passes on all 8 seeds, the three that failed B1b now 58–86× under
   the criterion they broke, with the residual a bracket whose ends are 0 and `|model gap|`.
3. **The O–O tail is a power law at index 5.0049**, measured from the table's own knots with
   its exponential extrapolation index agreeing to four digits.
4. **Three conservation laws hold independently**, in both the complete and the truncated
   configuration, with plants demonstrating that each gate can fail without the others.
5. **The engine now has an angular-momentum ledger** it did not have, and it returns `None`
   rather than `true` where the box does not conserve `L`.
6. **The far sector costs `O(N²)`** at the sizes measured — exponent 2.123.

**Not banked:**

1. **Nothing about ionic scenes.** Every nucleus here is neutral H or O. R1 refuses the `r⁻¹`
   case by name; node C's scenes are neither shown nor excluded.
2. **Nothing about an `O(N)` far route.** G13 measures `O(N²)`, which is what a complete far
   sum is. Making the far sector cheap is a different piece of work.
3. **G4 is VOID under V2**, G7 and G8 are FIRED, and the periodic arm is VOID on the hydrogen
   set. Four of the fourteen staked gates did not deliver a clean scored pass.
4. **Nothing about a physical dispersion coefficient.** The constant is a model quantity from
   a minimal basis whose solve was budget-limited; the diffuse-basis exit (`ION_STAKING.md`
   I-5) is named and UNOWNED.
5. **Nothing above the scene sizes G13 measures**, and nothing about a device class other than
   the one it ran on.
6. **`G-LEDGER` is not one of the fourteen staked gates.** It was added after the freeze in
   response to the brief's ledger-closure requirement. A gate added afterwards only raises the
   bar, but it earns no credit as a pre-registered result.
