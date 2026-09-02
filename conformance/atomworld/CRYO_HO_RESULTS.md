# CRYO-H-O — results

*The record for `CRYO_HO_PREREG.md`, frozen 2026-09-02 at commit `fc7b6a0` and ADMITTED by
`Audit/prereg_audit.py` before any instrument in this campaign existed. Instruments at
`cc7d6bd`. Every number here is EXACT-IN-MODEL — STO-3G, full CI, Born–Oppenheimer,
classical nuclei, 2D scene. Physical values for hydrogen and oxygen appear as LABELLED
CONTEXT and nothing is scored against them.*

---

## VERDICT

| arm | verdict |
|---|---|
| **ARM 1 — liquid H₂** | **MEASURED NO WELL, with the number and the mechanism.** The model's exact H₂–H₂ interaction has no well deeper than the staked 1.0e-5 Ha anywhere at R ≥ 3 bohr. Its deepest attraction is **−5.592e-6 Ha = −1.77 K**, T-shaped, at R = 8.00 bohr — and it is **not dispersion**: the tail is `R^-5.00` with the classical quadrupole–quadrupole angular pattern, measured out to 100 bohr against a floor control. This model has no liquid hydrogen — and the quench ladder agrees: six H₂ molecules at every rung from 300 K down to 3 K, largest component ≤ 4 in 15 of 15 runs, nothing growing. |
| **ARM 2 — liquid O₂** | **One O12 aggregate at every rung from 300 K to 3 K**, fence exactly 220 every step, freezing (mobility 18.5 → 0.0) but never molecular — and it is NOT liquid oxygen: with no `(O,O,O)` surface the pair curve has no valence saturation. **The banked O–O curve IS a triplet curve at its well** (`⟨S²⟩ = 2.000000000`, multiplicity 3), so no paramagnetic fence is owed there — but the curve **changes spin state three times along its own length** (triplet → S = 0 → quintet → degenerate-and-UNRESOLVED), which no field in the banked artifact records — and the last of those **locates the banked curve's own `IterationCap`**: at dissociation the multiplet is degenerate, so there is no gap to converge against and more budget cannot buy one. The model cannot represent a molecular O₂ liquid at all: the `(O,O,O)` surface does not exist and the exact O₂–O₂ reference is priced out at 23,474,025 determinants. |
| **ARM 3 — the metallic-hydrogen fence** | **The fragment-local expansion never converges on this scene, at any density tested.** `|ΔMBE4| < |ΔMBE3| < |ΔMBE2|` fails at every rung including the loosest, because the three- and four-body terms have opposite signs and comparable size. The 1 mHa/atom crossing sits at **a = 5.50 bohr, n = 0.0661 atoms/bohr², P₂D = +1.254e-4 Ha/bohr²** — reported as a measurement, because its gate is VOID (below). |

**Two of the four pre-registered plants did not fire, and both voids stand.** That is the
loudest thing in this record and it is reported first rather than last: **G3 and G9 are
VOID**, their numbers are printed as measurements carrying no gate verdict, and the freeze
that wrote those plants is where the defect was.

**One instrument defect was caught by this campaign's own tie-to-banked rung and fixed
before any ARM 1 / ARM 2 quench verdict was read.** See § 0.

## SCORECARD

| gate | verdict | the number that decides it |
|---|---|---|
| **G1** — the model has no H₂–H₂ well | **HOLDS** | deepest `E_int = −5.592e-6 Ha` (−1.77 K) at R = 8.00 bohr, T; bar −1.0e-5 |
| **G2** — the pair level invents binding | **HOLDS** | MBE2 over-binds by −3.956e-1 Ha at R = 3.00, L; bar −1.0e-4 |
| **G3** — the three-body term corrects it | **VOID** (P2 did not fire) | criterion met at 1.2×, plant bar 10× |
| **G4** — no rung condenses | **KILLED**, on a clause this campaign mis-set | 5 of 15 breach an unsatisfiable `agg ≤ 1/6`; the `largest ≤ 4` clause held 15/15 |
| **G5** — which spin sector | **HOLDS** | `⟨S²⟩ = 2.000000000`, multiplicity 3 |
| **G6** — the sector plumbing, two-sided | **HOLDS** | quintet gap +4.332e-1 Ha; `M_s` degeneracy 3.13e-13 Ha |
| **G7** — one aggregate at every rung | **HOLDS** | O12 in 15 of 15; fence exactly 220 every step; zero free O |
| **G8** — `order` monotone as T falls | **VOID** (P4 did not fire) | criterion met; `order` starts at 0.996 and cannot rise |
| **G9** — the fence has a location | **VOID** (P2 did not fire) | all three clauses met; crossing at a = 5.50 bohr |
| **G10** — the ladder's convergence | **the expansion never converges** | `\|ΔMBE3\| > \|ΔMBE2\|` at 10 of 10 rungs |

| plant | | | void condition | |
|---|---|---|---|---|
| **P1** sign | **FIRES** | | **V1** instrument identity | **PASS**, both legs, both arms |
| **P2** three-body deletion | **DID NOT FIRE** | | **V2** ledger | 0 of 30 rung-seeds void |
| **P3** spin sector | **FIRES** | | **V3** classifier refusals | 5 of 15 (H), 6 of 15 (O) |
| **P4** label permutation | **DID NOT FIRE** | | **V4** solver exits | 0 of 10 exact solves void; 164 SCF-only sub-clusters, 0 IterationCap, 0 Stagnated |
| | | | **V5** basis dependence | never fired; the ladder ran to its end |
| | | | **V6** plant voids | **fired twice — G3, G8, G9** |

---

## 0. WHAT WENT WRONG IN THIS CAMPAIGN'S OWN INSTRUMENTS

Both were found by a check, both are fixed, and both are recorded because the check that
found them is more transferable than the fix.

### 0.1 The quench runner was not the banked protocol, and the curve check could not see it

`cryo_quench.rs` copied `waterquench.rs`'s scene, box, seeds, schedule and curve, and
omitted its **table set** — the H3 three-body surface above all. Run that way, the
hydrogen arm read a **7-to-12-atom aggregate at every rung including 300 K**, against
SATURATION-2's banked `44 × H2, 2 × H4, largest ≤ 4 in 8/8` on the **same seeds and the
same protocol**. With no H3 surface the scene is MBE2-only, the pair curve is a bonding
curve that knows nothing of valence saturation, and twelve hydrogens over-coordinate
exactly as twelve oxygens do.

**V1's curve-identity leg passed throughout**, because the curve was never what was wrong:
`R_e = 1.3887`, `D_e = 0.204142`, residual 8.7e-11, matching the bank to its printed
digits, on an instrument that was not the banked instrument. **A check that establishes
the CURVE is the banked one establishes nothing about the PROTOCOL** — one direction of a
two-directional question, reported as both.

The fix is the table set, copied verbatim. The gate is **V1 leg 2**, added after the fact
and stated as such: the 300 K rung is compared to the banked reading *mechanically*, not
by eye. The first-run log is kept at `engine/output/cryo/quench_hydrogen.NO_H3_TABLE.log`,
marked dead, because a record that deletes what it got wrong cannot be audited.

### 0.2 The unit-fence section reported a pass for a scene with no interactions in it

ARM 3's virial section read `Sim::pbc_ok` on an empty `Sim` and got
`list_cutoff = 0.000 → pbc_ok = true` at every density. That is a pass reported for a box
containing no forces. Loading the table was the obvious fix — and it **did not change the
reading**, which turned the defect into a finding about the engine's own guard (§ 3.4).

---

## 1. ARM 1 — LIQUID HYDROGEN

*Instrument `holon-chem/examples/cryo_h2_dimer.rs`; log `engine/output/cryo/arm1_dimer.log`.
431 FCI solves, largest 36 determinants. Every sub-cluster energy is a fresh exact solve,
so ARM 1 carries no interpolation error and a residual it reports is the expansion's.*

References, with their disclosure: `E(H) = −0.466581849557 Ha` (exit `Trivial`, 1
determinant); `E(H₂) = −1.137306051222 Ha` (exit `Converged`, residual 9.66e-17, 4
determinants, 2 basis functions, SCF converged, device `Cpu`). The implied
`D_e = 0.204142352108 Ha` reproduces the 50-digit referee's `0.204142352107591`.

### G1 — has the MODEL an H₂–H₂ well? · **HOLDS. No well.**

Staked: `E_int(R) > −1.0e-5 Ha` at every `R ≥ 3.0 bohr` in all three orientations.

    deepest E_int anywhere at R >= 3 :  -5.592087e-06 Ha  =  -1.77 K
    at                               :  R = 8.00 bohr, T (perpendicular)
    sign census at R >= 3            :  4 of 36 points attractive at all
                                        (all four are T at R >= 8)

The staked bar is cleared by a factor of 1.8, which is thin, so the well was refined on a
0.25 bohr grid (POST-DATA, changes no criterion): the T minimum is a real minimum at
`R = 8.00`, bracketed by `−5.074e-6` at 7.75 and `−5.481e-6` at 8.25.

### The mechanism, and the control that had to be run before believing it

The sweep returned an exact interaction at 12 bohr of `+4.27e-7 Ha` while the pair and
triple terms there are `6e-11` and `1.2e-10` — **four orders smaller**. Two opposite things
produce that shape: a genuine long-range electrostatic tail, or an instrument floor from
differencing two ~2.27 Ha numbers. A floor would have put G1's `−5.6e-6` only 13× above
noise and made the sign census meaningless.

`cryo_h2_tail.rs` settles it the only way that can: keep going out. A tail falls, a floor
does not.

| R (bohr) | E_int H | E_int T | E_int L | slope H | slope T | slope L |
|---|---|---|---|---|---|---|
| 8 | +3.716e-6 | −5.592e-6 | +2.979e-6 | | | |
| 12 | +4.266e-7 | −9.905e-7 | +7.395e-7 | −4.173 | **−5.006** | −2.558 |
| 20 | +4.297e-8 | −7.686e-8 | +9.641e-8 | −4.598 | **−5.003** | −4.274 |
| 40 | +1.573e-9 | −2.400e-9 | +3.905e-9 | −4.827 | **−5.001** | −4.725 |
| 60 | +2.171e-10 | −3.160e-10 | +5.534e-10 | | | |
| 100 | +1.749e-11 | −2.458e-11 | +4.546e-11 | | | |

**There is no floor.** The rows meant as a floor control continue the power law for four
more decades. The T slope is `−5.00` to four digits at every step, and the signs are the
classical quadrupole–quadrupole angular pattern exactly: **T attractive, side-by-side (H)
repulsive, collinear (L) repulsive.**

**One thing is measured and not explained, and is written down as such.** Only the T
channel's slope has converged: H and L climb monotonically from −4.17 and −2.56 toward −5
without reaching it by 40 bohr, and extrapolating each channel's own `R^-5` from 40 bohr
to 100 bohr reproduces T exactly (−2.458e-11 predicted, −2.458e-11 read) while missing H by
8% and L by 14%. So H and L carry a sub-leading component this campaign has not
characterised. It does not touch G1: the verdict rests on the T channel, which is both the
only attractive one and the cleanly converged one.

So the model's only H₂–H₂ attraction is **electrostatic** — H₂ has a permanent quadrupole
and a minimal basis carries one — and **not dispersion**, which a basis with no p function
on hydrogen cannot produce at any order. The freeze staked no-well from that argument and
the measurement returns the argument's own signature.

*[LABELLED CONTEXT, never compared against: the real H₂–H₂ van der Waals well is ~34 K and
is dispersion-dominated. The model's 1.77 K electrostatic well is a different physical
mechanism at 1/19 the depth. Nothing here is scored against that.]*

### G2 — does the ENGINE's pair level invent binding? · **HOLDS, and it is large**

    worst MBE2 over-binding in R in [3, 6] : -3.955864e-1 Ha
    at                                     : R = 3.00 bohr, L (collinear)
    exact +9.143646e-02   vs   MBE2 -3.041500e-01

The pair term is a covalent bonding curve applied to every H–H distance in the box,
including the four cross distances between two already-saturated molecules. Where the
model says two H₂ molecules **repel** by 0.091 Ha, the pair level says they **attract** by
0.304 Ha. **SATURATION-2's two banked H4 components in its hydrogen control are explained
by this, not anomalous** — and § 0.1's accidental MBE2-only run is the same mechanism at
full strength across a whole box.

### G3 — does the three-body term correct it? · **VOID per V6**

    at R = 3.00, L:   |MBE2 residual| 3.955864e-1  ->  |MBE3 residual| 3.388549e-1   (1.2x)

The staked criterion (MBE3 closer than MBE2) is met. **The gate is VOID anyway**: the
plant that guards it, P2, did not clear its stated bar (§ 4). The measurement stands as a
measurement and carries no verdict.

### The finding neither gate asked for: the engine's dynamics has no long-range H₂–H₂ interaction

At `R = 12 bohr`, H orientation:

    exact       +4.266283e-07 Ha
    MBE2        -5.994227e-11 Ha
    MBE3        +6.121770e-11 Ha
    dE4         +4.265671e-07 Ha        <- the whole interaction

**The model's entire long-range H₂–H₂ interaction lives in the four-body term of the
atom-based expansion.** A molecular quadrupole is not a sum of atom-pair terms, so the
pair and triple sectors are blind to it and decay exponentially (2.46 bohr⁻¹, § 0.3 of the
freeze) where the model's interaction decays as `R^-5`. The engine integrates MBE3. **It
therefore has no long-range H₂–H₂ interaction at all** — three to four orders below the
model's own — and no amount of cooling can condense a gas through a force the integrator
cannot see.

### G4 — does any rung condense? · **KILLED as staked, on a clause this campaign mis-set**

*Instrument `holon-render/examples/cryo_quench.rs`; log `engine/output/cryo/quench_hydrogen.log`.
15 runs of 20,000 × 64 steps. **V1 leg 2 PASS**: the 300 K rung reads largest 2, zero free
H, in 3 of 3 seeds — SATURATION-2's banked hydrogen control reproduced, so this is the
banked protocol and the ladder is the only variable.*

| T_target | largest, per seed | aggregated fraction, per seed |
|---|---|---|
| 300 K | 2, 2, 2 | 0.000, 0.000, 0.000 |
| 100 K | 2, **4**, 2 | 0.000, **0.333**, 0.000 |
| 30 K | 2, **3**, 2 | 0.000, **0.250**, 0.000 |
| 10 K | 2, 2, **3** | 0.000, 0.000, **0.250** |
| 3 K | 2, **3**, **3** | 0.000, **0.250**, **0.250** |

Ledger: 0 of 15 rung-seeds VOID; worst drift/bound 1.188e-2, worst |p|/bound 1.207e-4.

**The gate is KILLED: 5 of 15 rung-seeds breach.** Reported as plainly as a survival would
be. And then the arithmetic, which is this campaign's fault and not the model's:

**G4's aggregated-fraction bar was unsatisfiable.** The bar is `agg ≤ 1/6 = 0.1667` on a
12-atom scene, and the smallest possible non-zero aggregated fraction is one 3-atom
component, `3/12 = 0.250`. **The bar therefore admits exactly zero components larger than
two** — the freeze's own note beside it ("admits the banked 300 K artifact rate ... without
admitting a second H4 in one seed") is arithmetically false, since one H4 alone is
`4/12 = 0.333`. **SATURATION-2's banked hydrogen control would fail this bar on 2 of its 8
seeds**, the two carrying an H4. A bar the reference control fails is a bar, not a finding.

The gate's OTHER clause is the one that carries information, and it **held 15 of 15**:
`largest ≤ 4` everywhere, at every temperature down to 3 K. The named prong is the
aggregated-fraction prong; the size prong never fired.

**What the numbers say, with the mis-set clause set aside and said as a measurement rather
than a verdict:** the scene is six H₂ molecules at every rung, with an occasional
transient H3 or H4 — the artifact SATURATION-1 named and SATURATION-2 reproduced, where
two H₂ molecules whose cross pair momentarily reads `bonded` are one component of four.
Nothing grows. **This model does not condense hydrogen at any temperature this protocol
reaches, down to 3 K**, and G1 says independently why: there is no well to condense it,
and the only attraction the model has (1.77 K, electrostatic) is one the engine's MBE3
dynamics cannot see at all.

*[LABELLED CONTEXT, never scored against: hydrogen boils at 20.3 K and freezes at 14.0 K.
The ladder was chosen as a logarithmic ladder before those numbers were looked up.]*

### The classifier on ARM 1 — reported verbatim, unscored, and the pre-staked fence confirmed

The freeze staked, before any run, that the blind classifier's VAPOR clause counts atoms
in **singleton** components and so **cannot fire on a molecular gas**: six H₂ molecules
have `free_fraction ≈ 0` however dilute they are. Confirmed, and the mechanism turns out
to be sharper than the freeze knew. `cryo_order_probe.rs` on the highest-order rung
(3 K, seed `…5422`):

    verdict LIQUID   order 0.8477   mobility 0.4172   free_fraction 0.0950
    frames_read 200   interior_atoms 1   interior_samples 20
                                       (STAKE_MIN_INTERIOR_ATOMS = 2)

**The scene has no bulk** — one distinct atom ever closed a complete six-neighbour shell,
against the classifier's own declared minimum of two — and the verdict is still a positive
phase label. The classifier's branch order is
`VAPOR → (refuse if no bulk AND mobility < 0.10) → ICE → else LIQUID`. The ICE branch is
correctly guarded by `has_bulk`; the refusal is **conjunctive with a low mobility**; and
LIQUID is the `else`. So a scene with no complete neighbour shell anywhere and a mobility
above 0.10 is labelled **LIQUID by fall-through**, on no positive evidence. That is
documented behaviour, not a bug — the module's own comment says a scene with no shell "can
still be asked whether it is flowing" — but on a twelve-atom molecular gas the two clauses
that could refuse are both out of reach, and the label carries no phase information.

**Every classifier verdict in ARM 1 is therefore reported and not scored**, exactly as the
freeze required. The exit is named: a classifier whose free clause counts MOLECULAR
components rather than singleton atoms.


### Localization clause

The expansion's error is largest at `R = 2.50, T`: `dE4 = −5.134e-1 Ha`, at a shortest
cross distance of 1.935 bohr — i.e. the error concentrates where a cross pair has come
inside a bond length, not where the molecules are far apart. All 39 four-atom solves and
all sub-cluster solves exited converged.

---

## 2. ARM 2 — LIQUID OXYGEN

*Instrument `holon-chem/examples/cryo_o2_spin.rs`; log `engine/output/cryo/o2_spin.log`.*

**Inherited disclosure, carried on every number in this arm:** the banked O–O curve exits
`IterationCap` at `solver_budget_iterations = 5000` with `worst_residual = 4.809e-6 Ha`,
`n_det = 2025`, `n_basis = 10`. A capped residual is not monotone in effort.

### G5 — which spin sector did the banked curve return? · **HOLDS. A triplet.**

At the banked `R_e = 2.4421 bohr`, `S_z = 0`, `n_det = 2025`, exit `Converged`, residual
8.24e-11, 36 Davidson iterations, `variational_margin +1.700e-1`, device `Cpu`:

    <S^2> = 2.000000000        multiplicity 3        S = 1

**The banked O–O curve IS a triplet curve at its well.** Physical O₂ is ³Σg⁻ and liquid O₂
is paramagnetic; the engine's minimal-|S_z| rule found the triplet without being told to,
which is what its warrant claims. **No paramagnetic model fence is owed at the well**, and
the pre-committed KILL branch (an S = 0 curve) did not fire.

### G6 — is the sector plumbing two-sided? · **HOLDS**

    E(S_z=2) - E(S_z=0)   = +4.331627e-01 Ha        bar +1.0e-4   PASS
    |E(S_z=1) - E(S_z=0)| =  3.126388e-13 Ha        bar  1.0e-6   PASS

The two `M_s` components of one triplet are degenerate to 3e-13 Ha, which is the
degeneracy a spin-free Hamiltonian must produce and is four orders below the inherited
residual. The quintet sits 0.433 Ha above. A **control on a molecule whose answer is known
independently** — H₂ at `R_e` through the same code path — returns `⟨S²⟩ = 0.000000`,
multiplicity 1: the instrument distinguishes the two molecules and does not merely always
say "triplet".

### POST-DATA: the scope G5's single point does not cover

G5 is staked at one geometry; the banked curve is 96 knots over a range, and multiplicity
is a property of a geometry.

| R (bohr) | ⟨S²⟩ | multiplicity | E(S_z=1) − E(S_z=0) | exit | residual |
|---|---|---|---|---|---|
| 1.800 | 2.000000 | 3 — triplet | +5.7e-14 | Converged | 8.2e-11 |
| 2.200 | 2.000000 | 3 — triplet | −1.4e-12 | Converged | 8.6e-11 |
| **2.442** | **2.000000** | **3 — triplet** | +3.1e-13 | Converged | 8.2e-11 |
| 2.800 | 2.000000 | 3 — triplet | −4.5e-13 | Converged | 8.0e-11 |
| 3.500 | **0.000000** | **1** | +2.922e-3 | Converged | 7.5e-11 |
| 4.500 | **0.000000** | **1** | +1.682e-3 | Converged | 9.7e-11 |
| 6.000 | **6.000000** | **5 — quintet** | +1.4e-13 | Converged | 8.6e-11 |
| 8.000 | **2.743455** | **UNRESOLVED** | −1.5e-9 | **IterationCap** | **9.3e-8** |

**The banked O–O curve changes spin state three times along its own length**, and **all
three of G5's pre-committed branches occur somewhere on it**: triplet through the well,
S = 0 from about 3.5 bohr, quintet by 6 bohr, and UNRESOLVED at 8. The minimal-|S_z| rule
follows the lowest state across every crossing, which is exactly what it is designed to do
— and it means **the curve is not a single-multiplet curve, and no field in the banked
artifact records that**. G5's verdict is about the well, where the bonding is; the
crossings are scope G5 did not claim and are now measured rather than owed.

The sequence is the two oxygen atoms' multiplet structure resolving as they separate: at
8 bohr they are two dissociated ³P atoms, whose S = 0, 1 and 2 states are degenerate, so
the eigensolver returns an arbitrary mixture of degenerate components. `⟨S²⟩ = 2.743455`
is `S(S+1)` for no half-integer S at all, and `multiplicity` refuses it rather than
rounding — the UNRESOLVED branch working as written. SATURATION-2's own spin audit
described exactly this shape on the (O,H,H) surface ("56 are degenerate — 2S = 0, 1 and 2
all occur, and none is asserted on").

**And it locates the banked curve's `IterationCap`.** That row is the only one that did not
converge: exit `IterationCap`, residual 9.3e-8 against 8e-11 everywhere else — three orders
worse — and `E(S_z=1)` sitting 1.5e-9 Ha *below* `E(S_z=0)`, which is degeneracy, not an
ordering. The banked O–O curve's inherited disclosure (`IterationCap`, budget 5000,
`worst_residual 4.809e-6`) has been carried on every number in this arm as a fact without a
cause. **The cause is the dissociation tail: where the spin multiplet is degenerate there is
no gap for Davidson to converge against, and more budget cannot buy one.** That is a
different fact from a budget being too small, and it is the distinction
**M-EXIT-DISCRIMINATOR** exists to preserve.

### What ARM 2 cannot reach, priced

The exact O₂–O₂ reference that ARM 1 gets for free needs FCI on four oxygens: 20 orbitals,
32 electrons, `C(20,16)² = 23,474,025` determinants — **11.7× past
`HARD_DETERMINANT_CAP = 2,000,000`** and 469× past `MPS_ROUTE_THRESHOLD = 50,000`. Exit:
`FENCES.md` C5/C6, GANTT node F. **ARM 2 cannot decide whether this model has a molecular
O₂ liquid.**

And the `(O,O,O)` three-body surface does not exist (`FENCES.md` P1, `ozone.rs:412`), so
the oxygen scene runs **MBE2-only** with every one of `C(12,3) = 220` triples refused per
force evaluation. The model has no valence-saturation term for oxygen at all. Whatever it
condenses is a pair-only aggregate, and calling it "liquid oxygen" would be a category
error the engine itself refuses to make.

### G7 — one aggregate at every rung? · **HOLDS, 15 of 15**

*Instrument `holon-render/examples/cryo_quench.rs`; log `engine/output/cryo/quench_oxygen.log`.*

**V1 leg 1 PASS**, and it reproduces the banked disclosure exactly, not approximately:

    O-O 96 knots, R_e = 2.4421 bohr, D_e = 0.147621 Ha,
    exit IterationCap, budget 5000, worst_residual 4.809e-6, n_det 2025, n_basis 10,
    device Cpu                                                        [841.5 s]

**V1 leg 2 PASS**: at 300 K the scene reads `largest 12, free O 0, fence 220` in 3 of 3
seeds — SATURATION-2's banked oxygen control reproduced.

| T_target | largest, per seed | fence | free O | classifier |
|---|---|---|---|---|
| 300 K | 12, 12, 12 | 220, 220, 220 | 0, 0, 0 | LIQUID ×3 |
| 100 K | 12, 12, 12 | 220, 220, 220 | 0, 0, 0 | LIQUID ×3 |
| 30 K | 12, 12, 12 | 220, 220, 220 | 0, 0, 0 | REFUSED ×3 |
| 10 K | 12, 12, 12 | 220, 220, 220 | 0, 0, 0 | REFUSED ×3 |
| 3 K | 12, 12, 12 | 220, 220, 220 | 0, 0, 0 | ICE ×3 |

Ledger: 0 of 15 rung-seeds VOID; worst drift/bound 2.064e-2, worst |p|/bound 1.472e-4.
The fence is **exactly `C(12,3) = 220` at every force evaluation of every run**: every
triple in the box refused for want of an `(O,O,O)` surface. This arm is MBE2 throughout
and the count says so mechanically.

**One aggregate, at every temperature, including 300 K.** *[LABELLED CONTEXT, never scored
against: oxygen boils at 90.2 K. The model's oxygen is already condensed 210 K above
that, and the ladder was chosen before the number was looked up.]* The reason is named in
the freeze and is not a temperature effect at all: with no three-body term the pair curve
has no valence saturation, so twelve oxygens over-coordinate into one droplet the way
SATURATION-1's pair-only hydrogens did. **This is not liquid oxygen and the campaign does
not call it that.** Liquid O₂ is a molecular liquid of O₂ units held by weak forces; the
model has no mechanism to stop an oxygen collecting neighbours, and the reference that
could settle whether the model has a molecular O₂ liquid is priced out at 23,474,025
determinants.

**G7 carries no plant.** The freeze assigned P1→G1, P2→G3/G9, P3→G5/G6 and P4→G4/G8, and
left G7 unguarded. What stands behind it instead is mechanical and stated so the reader can
weigh it: V1 leg 2's reproduction of the banked control, and the fence count landing on
`C(12,3)` exactly rather than near it.

### G8 — is `order` monotone as the scene is cooled? · **VOID (its plant did not fire)**

    T   300 K : order 0.9964   mobility 18.53   LIQUID x3
    T   100 K : order 0.9861   mobility  3.14   LIQUID x3
    T    30 K : REFUSED x3 — no scorable seed
    T    10 K : REFUSED x3 — no scorable seed
    T     3 K : order 0.9997   mobility  0.00   ICE x3

The staked criterion is met (no fall exceeds the 0.05 tolerance). **The gate is VOID**
because P4 did not fire (§ 4). And the criterion was close to uninformative in any case:
`order` starts at 0.996 and has nowhere to rise to, which is **M-UNTESTED-GAP** in a form
the freeze did not anticipate — a monotonicity stake on a quantity already at its ceiling.

What the numbers do show, as a measurement without a verdict, is a **droplet that freezes**:
mobility falls by four orders across the ladder, 18.5 → 3.1 → ~0.0, and by 3 K it is
exactly zero with the classifier reading ICE in 3 of 3 seeds. The two middle rungs are
**REFUSED** by the classifier's own bulk gate — 0 or 1 atoms ever closed a complete
six-neighbour shell, against its declared minimum of 2 — and it names the gate that would
lift the refusal: a scene with a bulk. **A twelve-atom droplet in 2D is almost all
surface.** That the same classifier reaches ICE at 3 K and refuses at 10 K and 30 K is the
droplet settling into a structure that finally has an interior, not a phase transition,
and this record does not present it as one.

**P4b fired here.** On the oxygen droplet the same geometry jitter that left the hydrogen
scene at 0.5956 drove `order` from 0.9997 to **0.0598**. The contrast is the point: a dense
12-atom droplet has real bond-orientational structure to destroy, and a 12-atom molecular
gas does not.


---

## 3. ARM 3 — THE METALLIC-HYDROGEN FENCE

*Instrument `holon-render/examples/cryo_h_compress.rs`; log `engine/output/cryo/compress.log`.
1,570 FCI solves, largest 4,900 determinants. Four H₂ molecules on a 2 × 2 planar lattice,
bond frozen at the referee's `R_e`, nearest-neighbour centre separation `a` stepped down.*

**NO PHASE IS CLAIMED.** Metallization is electron delocalization across many centres and
this engine's whole picture is fragment-local. It cannot exhibit a metal. What follows is
where its own picture stops working.

### The ladder

| a (bohr) | n (bohr⁻²) | E_exact (Ha) | ΔMBE2/atom | ΔMBE3/atom | ΔMBE4/atom | Σ dE3 | Σ dE4 | P₂D (Ha/bohr²) |
|---|---|---|---|---|---|---|---|---|
| 8.00 | 0.03125 | −4.54921314 | 2.136e-5 | 2.618e-5 | 1.622e-5 | +3.80e-4 | −3.39e-4 | +3.146e-7 |
| 6.50 | 0.04734 | −4.54890932 | 5.027e-4 | 7.102e-4 | 5.377e-4 | +9.70e-3 | −9.98e-3 | +1.382e-5 |
| **5.50** | **0.06612** | −4.54653625 | 3.430e-3 | **5.194e-3** | 4.507e-3 | +6.90e-2 | −7.76e-2 | **+1.254e-4** |
| 4.50 | 0.09877 | −4.53047389 | 1.960e-2 | 3.224e-2 | 3.238e-2 | +4.15e-1 | −5.17e-1 | +9.694e-4 |
| 4.00 | 0.12500 | −4.50270631 | 4.249e-2 | 7.479e-2 | 8.325e-2 | +9.38e-1 | −1.26e0 | +2.602e-3 |
| 3.50 | 0.16327 | −4.43689425 | 8.583e-2 | 1.657e-1 | 2.085e-1 | +2.01e0 | −2.99e0 | +7.038e-3 |
| 3.00 | 0.22222 | −4.27847425 | 1.633e-1 | 3.524e-1 | 5.011e-1 | +4.13e0 | −6.83e0 | +2.018e-2 |
| 2.60 | 0.29586 | −3.98582481 | 2.621e-1 | 6.227e-1 | 9.609e-1 | +7.08e0 | −1.27e1 | +5.149e-2 |
| 2.20 | 0.41322 | −3.29036126 | 3.989e-1 | 1.052e0 | 1.721e0 | +1.16e1 | −2.22e1 | +1.541e-1 |
| 1.90 | 0.55402 | −2.02386631 | 5.091e-1 | 1.515e0 | 2.509e0 | +1.62e1 | −3.22e1 | +4.536e-1 |

All ten exact solves exited `Converged` with residuals 3.7e-11 to 9.0e-11. **V5 never
fired**: the basis held to the end of the ladder, smallest overlap eigenvalue 2.1e-2 at
`a = 1.90`.

### G9 — the fence's location · **numerically met; VOID per V6**

    (i)   MBE3 error at a = 8.00 is 2.6179e-5 Ha/atom, below the 1.0e-3 bar   -> met
    (ii)  monotone non-decreasing as a falls                                  -> met
    (iii) crosses 1.0e-3 Ha/atom at a = 5.50 bohr                             -> met

**THE FENCE IS AT `a = 5.50 bohr`, `n = 0.06612 atoms/bohr²`, `P₂D = +1.254e-4 Ha/bohr²`.**

The gate is **VOID** because plant P2 did not fire (§ 4). The three numbers above are
reported as a measurement — they are differences against a converged exact reference and
nothing about them is in doubt — but they carry no gate verdict, and this record does not
launder a void into a pass.

### G10 — the ladder's own convergence · **it never converges, at any density**

`|ΔMBE4| < |ΔMBE3| < |ΔMBE2|` is the statement that the expansion is a converging series.
It is **false at every rung, including the loosest**:

* `ΔMBE3 > ΔMBE2` at all ten rungs — adding the three-body term makes the answer *worse*;
* `ΔMBE4 < ΔMBE3` at `a ≥ 5.50` and `ΔMBE4 > ΔMBE3` at `a ≤ 4.50` — the fourth order
  helps at low density and hurts at high.

The reason is in the table: **`Σ dE3` and `Σ dE4` have opposite signs and comparable
magnitude at every rung** (+3.80e-4 vs −3.39e-4 at `a = 8`; +16.2 vs −32.2 at `a = 1.90`).
The series alternates and overshoots. Chemically this is not surprising and it is the
sharper statement of ARM 3's whole point: **the atoms are not the right fragments.** An
atom-based expansion on a molecular solid has to build each covalent bond out of pair
terms and then undo the over-counting at every higher order, and on this scene it never
gets ahead.

**G9's crossing (a = 5.50) and G10's break (a = 8.00, the first rung) DISAGREE, and the
disagreement is the finding**: there is no density at which this expansion is converging,
only densities at which its truncation error is small enough not to matter yet.

### Localization clause

| a | largest single expansion term | at separation |
|---|---|---|
| 8.00 → 4.00 | a **pair**, −2.0414e-1 Ha | 1.389 bohr (the intramolecular bond) |
| 3.50 | a **quadruple**, −2.191e-1 Ha | 4.889 bohr |
| 3.00 | a **quadruple**, −3.389e-1 Ha | 4.389 bohr |
| 2.60 | a **quadruple**, −4.441e-1 Ha | 3.989 bohr |
| 2.20 | a **quadruple**, −6.715e-1 Ha | 2.345 bohr |
| 1.90 | a **quadruple**, −9.606e-1 Ha | 1.968 bohr |

The crossover is at `a ≈ 3.5–4.0`: below it the largest term in the whole expansion stops
being the covalent bond and becomes a four-body term spanning the lattice. That is the
fragment-local picture failing, localized.

### V4 — the solve record, attributed

    exact solves not converged                                        : 0 of 10
    sub-cluster solves not converged, over the ladder                 : 164
      of which SCF-only 164,  Stagnated 0,  IterationCap 0
      a 8.00: 84 of 154   a 6.50: 56   a 5.50: 20   a 4.50: 4   below: 0

All 164 are **SCF-only**, and an SCF-only non-convergence does not move a full-CI energy:
a full CI is invariant under any unitary rotation of its orbitals, so the SCF here only
chooses the basis the Davidson runs in. They cluster at the LOOSE end, where sub-clusters
contain widely separated atoms and the mean-field has near-degenerate solutions — the
classic restricted-Hartree–Fock dissociation problem, appearing exactly where it should.
Zero `IterationCap` and zero `Stagnated`: no solve here was budget-limited, which is a
different fact from "converged" and is counted separately for that reason.

### 3.4 THE UNIT FENCE — and an engine finding this arm did not go looking for

Both readings, as the freeze demands:

* **the model's own 2D pressure**, `P₂D = −dE_exact/dA`, in the table above, **Ha/bohr²**;
* **the engine's virial pressure**, on the same configuration at zero velocity:

```
  the H-H pair table's own support: r_min 0.393 to r_max 10.240 bohr

  a  8.00: box 16.00 x 16.00 x depth 24.0 | WALLS pressure_defined = false
           PERIODIC list_cutoff 0.000 vs half-edge 8.000 -> pbc_ok = true
           P = -2.3408e-07 Ha/bohr^3 = -6.887e6 Pa
  a  4.50: box  9.00 x  9.00 x depth 24.0 | WALLS pressure_defined = false
           PERIODIC list_cutoff 0.000 vs half-edge 4.500 -> pbc_ok = true
           P = -2.0984e-04 Ha/bohr^3 = -6.174e9 Pa
  a  1.90: box  3.80 x  3.80 x depth 24.0 | WALLS pressure_defined = false
           PERIODIC list_cutoff 0.000 vs half-edge 1.900 -> pbc_ok = true
           P = +1.6649e-03 Ha/bohr^3 = +4.898e10 Pa
```

**The stated unit fence holds and is confirmed.** `Sim::pressure` computes
`(2K − Σ virial) / 3V` with `V = width · height · depth` and `depth = 24.0 bohr` by default
on a 2D scene, so the pascal numbers above are three-dimensional pressures on a slab of
assumed thickness, with the 3D virial factor 3 where a 2D scene wants 2. `P₂D` in Ha/bohr²
is the primary reading. **No GPa comparison against the ~500 GPa metallization pressure of
the literature is performed**: it would be a number with an invented thickness in it.

**And a second thing, which this arm found by trying to use the guard.**
`Sim::pbc_ok` is `list_cutoff() ≤ half the shortest edge`, and `Sim::list_cutoff` is
`max(three_body_cutoff, four_body_cutoff, far.r_s, pair_switch.r_cut)` — **the pair
table's own support is not among them**, because an undeclared pair sector is a complete
sum with no cutoff to report. On a scene with no three-body surface, no far sector and no
declared truncation window, `list_cutoff()` is exactly **zero**, and `pbc_ok` returns
**true for every box size**. Measured above: the H–H table reaches **10.240 bohr** and the
guard passed periodic boxes with half-edges of 8.00, 4.50 and **1.90 bohr** — the last
one 5.4× inside the table's reach. The engine will hand back a virial pressure for a box
in which each atom sees the minimum image of a partner whose interaction reaches five box
widths.

This is **not** a defect this campaign introduced and it is **not** ARM 3's verdict; it is
why ARM 3's engine-side pressure column is not usable and why `P₂D` is the primary
reading. It is handed to the engine lane as a measured finding, with the numbers above as
its evidence. It looks like the same shape as `FENCES.md` finding **F-9** (a
one-directional gate), and the exit would be for `list_cutoff` to include the loaded pair
tables' `r_max` — a decision for that lane, not this one.

### The exit, as the fence law requires

Past this fence **this engine stops being able to speak**, and the exit is named:
**delocalized / periodic electronic structure** — a band or plane-wave solver with k-point
sampling. A different solver class, out of scope for this campaign and for this crate.

---

## 4. THE PLANTS — two of four fired, and the two voids stand

| plant | carrier, and its sector | verdict |
|---|---|---|
| **P1** — negate `E_int` (ARM 1 G1) | the intermolecular channel; **nonzero in** it at 21 of 21 points in `R ∈ [3,6]` with `\|E_int\| > 1.0e-4 Ha` | **FIRES.** Planted deepest reading −9.144e-2 Ha against a −1.0e-4 bar; G1's no-well verdict inverts. 36 points negated. |
| **P2** — delete every three-body term (ARM 1 G3, ARM 3 G9) | the three-body channel; **nonzero in** it — `Σ dE3 = +7.344e-1 Ha` at ARM 1's point, `+3.803e-4 Ha` at ARM 3's loosest rung | **DID NOT FIRE.** ARM 1: 1.2× against a 10× bar. ARM 3: **0.82×** — deleting the term makes the residual *smaller*. **G3 and G9 are VOID.** |
| **P3** — force the solve into `S_z = 2` (ARM 2 G5/G6) | the quintet block, 210 determinants, containing no `M_s = 0` component at all, so disjoint from the honest solve's space by construction | **FIRES.** +4.332e-1 Ha against a 1.0e-4 bar. |
| **P4** — permute positions across atoms (ARM 1 G4, ARM 2 G8) | the bond-orientational order channel; **nonzero in** it — unscrambled 0.8477 (H arm), 0.9997 (O arm) | **DID NOT FIRE** on either arm (order unchanged to four decimals), and it could never have. **G8 is VOID.** It also never guarded G4 at all — see below. |

### Why P2's failure is a result and not an accident

P2 was written to prove that ARM 3 measures the expansion rather than a constant offset.
It failed to prove it, and **the reason it failed is G10**: because the expansion
alternates, deleting the three-body term moves the reported residual *toward* the exact
answer, not away. A plant whose premise is "removing this term makes things much worse"
cannot fire on a series where that term overshoots.

Something else does establish that ARM 3 measures the expansion — the residuals span nearly
five orders of magnitude across the ladder, from 2.6e-5 to 1.5 Ha/atom, which no constant
offset does. **That reasoning is post-hoc and does not discharge the gate.** V6 says a
plant that does not fire voids the gate it guards, and G3 and G9 stay void.

### Why P4 could never have fired, which is worse

P4 permutes position **labels** across atoms. `ψ6` is computed from each atom's nearest
neighbours and is therefore a function of the **point set**, which a relabelling does not
touch. **P4's carrier lies exactly in the null space of the statistic it was meant to
move** — it could not have fired on any scene whatever, and the vacuity was written into
the freeze's own wording ("leaving the per-frame position multiset untouched"). That is
**M-PLANT-OBS** in its pure form: the plant was not re-derived for this instrument. Its
work count is honest and useless: 200 frames permuted, 2,226 atom positions moved, `order`
unchanged at 0.8477 to four decimals.

**And it was assigned to a gate it does not touch.** The freeze put P4 on ARM 1's G4 as
well as ARM 2's G8. G4's criterion is component composition, read from the engine's bonded
bitset — which P4 *explicitly preserves*. So P4 could not have guarded G4 either, on a
different mechanism from the one that made it vacuous for G8. **G4's real guard turned out
to be V1 leg 2**, the mechanical comparison against SATURATION-2's banked 300 K reading,
which passed — and which is also what caught § 0.1. The lesson is not that the plant was
weak; it is that a plant assigned to a gate must be checked against *that gate's own
statistic*, and neither of P4's two assignments was.

**P4b** (POST-DATA, and it does not cure the void) displaces every atom by an independent
uniform vector of up to half the frame's own mean nearest-neighbour distance, which is
what the order parameter actually reads. It read `order = 0.5956`, still above the 0.45
bar — so it did not fire either, and that left two opposite explanations open: an
under-powered amplitude, or a degenerate statistic.

`cryo_order_probe.rs` (POST-DATA) separates them by sweeping the amplitude two decades:

| jitter, × mean nn | amplitude (bohr) | order | interior_atoms | verdict |
|---|---|---|---|---|
| 0.05 | 0.126 | 0.8562 | 1 | LIQUID |
| 0.25 | 0.630 | 0.7943 | 1 | LIQUID |
| **0.50** (P4b) | **1.259** | **0.5956** | 1 | LIQUID |
| 1.00 | 2.519 | 0.2427 | 1 | LIQUID |
| 2.00 | 5.037 | 0.1394 | 1 | LIQUID |
| 5.00 | 12.593 | 0.1013 | 5 | LIQUID |

**The order channel IS movable — P4b was simply under-powered on the hydrogen scene.** It
crosses the 0.45 bar between one half and one full nearest-neighbour distance, and P4b sat
just under the crossing. On the OXYGEN droplet the same P4b, at the same fraction, drove
`order` from 0.9997 to 0.0598 and fired comfortably — a dense droplet has structure to
destroy where a molecular gas has none. That is the benign answer to the question the probe
asked, and it is arm-dependent.

The unbenign answer is in the last column but one: **`interior_atoms = 1` at every
amplitude below a five-fold displacement.** The order number, moved or unmoved, is one
atom's local hexagon counted across twenty correlated frames — the exact failure mode
`STAKE_MIN_INTERIOR_ATOMS` was added to catch, present here and not caught, because the
refusal that would catch it is conjunctive with a low mobility this scene does not have.
So `order` never gates anything on a twelve-atom molecular scene whether it moves or not,
and no amplitude choice would have made P4 or P4b informative about a phase.

A post-data plant can never discharge a pre-registered one. **P4's void stands.**

---

## 5. WHAT THIS CAMPAIGN CANNOT DECIDE — the list from the freeze, unchanged

1. Whether the **substances** liquid H₂ and liquid O₂ behave as this model does. Three
   standing fences on every number: 2D scene, classical nuclei, STO-3G minimal basis. For
   hydrogen the nuclear-quantum fence is severe: real liquid H₂ has a de Boer parameter
   near 1.7 and is the most quantum molecular liquid there is, while every nucleus here is
   Newtonian. Node E's ring-polymer route exists and is not coupled.
2. Whether the model has a **molecular O₂ liquid** — priced out at 23,474,025 determinants.
3. **Where the metallization pressure of hydrogen is.** ARM 3 locates a fence in the
   model's own picture and refuses the pressure comparison for the unit reason in § 3.4.
4. Whether ARM 3's fence location survives **N** or survives **disorder**. Eight atoms, one
   geometry, two dimensions, an ordered lattice.

## 6. CREDITS AND CONVERGENCES

The `R^-5` quadrupole–quadrupole form of the H₂–H₂ long-range interaction, and its
angular pattern (T attractive, side-by-side and collinear repulsive), are textbook
electrostatics of two linear quadrupoles and are not a discovery here. What is measured
here is that **this model, in this basis, reproduces that form to a slope of −5.00 over
four decades** — and that its atom-based many-body expansion is blind to it.
