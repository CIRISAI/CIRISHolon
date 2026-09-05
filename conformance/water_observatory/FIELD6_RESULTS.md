# FIELD-6 — results

*Freeze `FIELD6_PREREG.md` (f242e51, alone; the instrument `heitler_london_undeformed` built
and unit-tested before it). Harvest `examples/field6_harvest.rs` (delegate, to the freeze);
the arms by `field3_hbonds.rs` reading `wall6.json`; FIELD-4's engine unchanged. JSON under
`field6/`.*

## The verdict, first

**The undeformed referee gives a physical wall, the wall is harvested, and the seam law
built from it repels at the hydrogen bond — so the ledger's remaining residual is now
measured, named and sized.** With the monomers left in their own orbitals the exchange
reads `+12.1` mHa at the hydrogen-bond minimum (the deformed referee said `+40`), a single
exponential fits the five shortest nodes within the derived tolerance (`A = 1586` Ha,
`b = 2.158` per bohr, `R_x = 3.4` Å — **S1 (b)**), every gate on the instrument passes
(ordering exact ≤ undeformed ≤ orthogonalised at all six nodes; exchange `8e-12` and the
electrostatics accounted at 40 bohr — FIELD-5's letter-failure closed), and G-C1 holds at
`1.5e-16`. But the budget at 2.9 Å is `E_q −4.13 + p −3.09 + wall +12.10 = +4.9` mHa against
the exact `−5.48`: the seam law is `10.4` mHa too repulsive at the minimum, the expectation
rule wrote "break", and **S3 reads (c)**: `f = 0.0000` on the dimer at both temperatures,
`0.002–0.003` on the ring. The missing `−10.6` mHa is the remainder after charges, wall and
the density field's contact term; its shape is exponential (log-log slopes `−10.8`, `−12.0`,
`−13.6`, far steeper than dispersion's `−6`), which is why channel 3's `R⁻⁶` could not take it
(`C₆ = 0`, not transferred). That remainder is charge transfer and inter-fragment
correlation, which the Heitler–London state excludes by construction and the exact dimer
includes; it lives on the contact and decays like overlap. Plant (ii) DID NOT FIRE: the
derived tolerance scales with `E_exch`, so the deformed referee's four-times-larger wall got
four times the slack and fit its own exponential too — the discrimination between the
referees is in G-U0's ordering and H1's ratio (`0.30`), not in the fit. S2 reads (b): the 45°-bent bond at 3.1 Å binds by `−5.43` mHa on the exact solver against a filed `+0.97` — the seam law misses by `6.4` mHa, but the wall's value there (`5.13`) is within `0.65` mHa of the undeformed exchange on that geometry (`5.77`): the WALL transfers to a bent bond, the miss is the remainder's.
FIELD-7 is named: the remainder transferred into the channel whose shape it has — one
attractive exponential on the contact carrying penetration, induction, charge transfer and
correlation together, fit on the linear nodes over the charges and the harvested wall —
and the hydrogen bond asked again.

| gate | verdict | the number |
|---|---|---|
| G-U0 — the undeformed state is what it says | **PASS** | overlap `⟨v|v⟩` 0.9693 (2.5 Å) → 0.9998 (3.7 Å); `e_super ≤ E_HL(undeformed) ≤ E_HL(orth)` at all six (gaps `+3.2e-2 → +7.5e-4` and `+1.1e-1 → +1.2e-3`); at 40 bohr `|E_exch| = 8.4e-12` and `|E_HL − E_A0 − E_B0 − E_es| = 8.4e-12` (stake 1e-8) |
| H1 — exchange is a wall | **PASS** | `+58.3`, `+26.9`, `+12.1`, `+5.31`, `+1.48`, `+0.39` mHa at 2.5–3.7 Å, non-increasing; at 2.9 Å the undeformed is `0.302` of the deformed (stake < 0.5) |
| S1 — what the wall is | **BRANCH (b)** | k = 6 misses `1.22` of its derived tolerance; k = 5 fits at `0.84`: `A = 1.58645e3` Ha, `b = 2.15816` /bohr, `R_x = 3.4` Å; the excluded 3.7 Å node misses by `+5.2e-5` Ha (`0.24` of its own tolerance); the `E_exch` arm of the tolerance binds at 2.5–2.9 Å, the `ΔE_exact` arm at 3.1–3.7 Å |
| the penetration term (FIELD-5's, reused) | — | `P = 16.040`, `c = 2.42` |
| dispersion | **not transferred** | remainder `ΔE_exact − E_q − p − wall` on the outer nodes: log-log slopes `−10.79`, `−12.04`, `−13.59` (band `[−8, −4]`); `C₆ = 0` |
| G-C1 — the harvest is the engine's arithmetic | **PASS** | worst `1.46e-16` (stake 1e-10), one reference both sides |
| plant (i) — the sign of the penetration term | **FIRES** | `6.173650e-3` against `2·|p_HO| = 6.173650e-3`, to `9.5e-17`; carrier `3.09e-3` |
| plant (ii) — the deformed referee | **DOES NOT FIRE, read** | carrier present (`E_exch^{orth} − E_exch = 2.80e-2` at 2.9 Å, ≥ 1e-3), but the deformed exchange ALSO fits five nodes within the derived tolerance (`0.97` of it; `A = 1886`, `b = 1.972`): the tolerance's `0.05·E_exch` arm grants the larger wall proportionally more slack (8.3 and 4.1 mHa at 2.5 and 2.7 Å). The freeze's reasoning held only at 2.9 Å. A plant that does not fire is recorded as such; the referees are told apart by G-U0 and H1 |
| §1 — the expectation | **"break" at both temperatures** | dimer binding `+4.08e-3` (field `−4.08e-3`, seam `+8.16e-3`), ring `+2.07e-2` (field `−1.20e-2`, seam `+3.27e-2`); units 2 / 4 |
| S2 — a bent hydrogen bond, predicted forward | **BRANCH (b)** | filed before the solve on the 45°-tilted dimer at 3.1 Å: `E_pred = +9.686e-4` (`E_q(R) − E_q(40) = −2.919e-3`, `p_HO = −1.238e-3`, wall `+5.125e-3`, dispersion `0`); the solve: `ΔE_exact = −5.4345e-3`, `Converged`, 217 iterations, residual `8.8e-11`, 23,507 core-seconds; miss `6.40e-3` (118 %) against `1.36e-3` — (a) fails; `|wall − E_exch(tilted)| = 6.5e-4` against `1.36e-3` — (b): the wall transfers, the miss is the other terms'. The bent bond binds MORE than the line at 3.1 Å (`−5.43` against `−4.76`) |
| S3 — retention under the seam law | **BRANCH (c)** | dimer `f_SEAM` 0.0000 / 0.0000 (293 / 150 K), ring `0.0019` / `0.0027`; `f_OFF` 0.0000 on every arm (FIELD-2's, reproduced); books closed and momentum conserved on all eight; the switch posts `−1.29e-2` (dimer) and `−2.55e-1` (ring) to `work.seam` |

## 1. The readings, and the budget

| R_OO (Å) | ΔE_exact | E_q (charges) | p (contact) | E_exch (undeformed) | wall fit | E_q + p + wall | remainder |
|---|---|---|---|---|---|---|---|
| 2.5 | `+2.507` | `−7.069` | `−19.22` | `+58.33` | in fit | `+32.0` | `−29.5` |
| 2.7 | `−4.320` | `−5.337` | `−7.703` | `+26.89` | in fit | `+13.8` | `−18.2` |
| 2.9 | `−5.480` | `−4.134` | `−3.087` | `+12.10` | in fit | `+4.9` | `−10.4` |
| 3.1 | `−4.757` | `−3.270` | `−1.237` | `+5.307` | in fit | `+0.80` | `−5.6` |
| 3.4 | `−3.217` | `−2.377` | `−0.314` | `+1.477` | in fit | `−1.21` | `−2.0` |
| 3.7 | `−2.141` | `−1.784` | `−0.080` | `+0.392` | outside | `−1.47` | `−0.67` |

(all mHa; `p` is FIELD-5's contact term evaluated on each geometry, the 2.5 and 2.7 Å values
outside its fit range). Three things are measured here for the first time. The first-order
exchange of the water dimer at this basis level is `12.1` mHa at the minimum, decaying with
`b = 2.16` per bohr. The classical electrostatics of the isolated densities is `−6.93` mHa
there (FIELD-5's `E_es`), `1.7` times the point charges. And what the exact dimer has beyond
first order — charge transfer and inter-fragment correlation, everything the product state
excludes — is `−10.4` mHa at the minimum, comparable to the exchange it sits behind, and it
decays like overlap, not like `R⁻⁶`. In a larger basis that remainder would split into
induction, dispersion and charge transfer; in this one it is a single exponential on the
contact, and that is the shape the ledger transfers it in.

## 2. What (c) on the arms says

The expectation rule predicted it before the arms ran: with the wall in and the remainder
out, the seam law is repulsive at every start, and the molecules leave. It is the reverse
of FIELD-3's diagnostic (where a wall fit to two nodes held the dimer at 150 K because it
was too steep to matter at the start): a wall harvested from a real referee, without the
attraction that belongs behind it, is a law that repels. The arms measured exactly that,
with the books closed. Nothing about the wall is wrong; the seam law is one channel short,
and the harvest says which and how much.

## 3. S2 — the bent bond, and the free readings

The 45°-bent bond at 3.1 Å is `−5.43` mHa exact, more bound than the line at the same
separation. The seam law predicted `+0.97`: charges `−2.92`, contact `−1.24`, wall `+5.13`.
The wall's part is right — the undeformed exchange on that geometry is `+5.77`, the O–O wall
within `0.65` mHa of it — so the `6.4` mHa miss is entirely the remainder the law does not
carry, and it is LARGER on the bent bond than on the line. Two free readings beside the wall:
the 30°-bent bond at 2.9 Å (FIELD-5's) has `E_exch = 12.62` against the wall's `11.59`
(`−1.0`, 8 %); the flipped dimer at 3.4 Å (FIELD-4's) has `E_exch = 8.62` against the wall's
`1.51` (`−7.1`, a factor of `5.7`). At one oxygen–oxygen separation the undeformed exchange
moves by a factor of six with orientation, the same ratio the deformed referee gave in
FIELD-5: exchange lives on the contacts, and FIELD-7 places the wall on the atom-pair
classes.

## 4. FIELD-7, named

The remainder transferred into the channel whose shape it has: the whole non-charge,
non-wall residual `ΔE_exact − [E_q(R) − E_q(40)] − wall(R)` fit as ONE attractive
exponential on the cross-unit H–O contacts (the placement the density field's term already
uses; the freeze names what it folds — penetration, induction, charge transfer, correlation
— as folded channels in the ledger's table), over the six linear nodes with the wall held at
FIELD-6's harvest; then G-C1, a new bent bond predicted forward, and the arms. If the fit is
within a derived tolerance the seam law reproduces the linear dimer by construction and the
bent bond and the dynamics are the tests; if it is not, the residual's shape says what a
second term must be.

## 5. Bookkeeping, declared

- The arms ran before S2 (they do not depend on the held-out solve); the freeze orders
  neither before the other. The seam gates re-run with the harvested coefficients loaded
  (`A = 1586`, `b = 2.158`, `P = 16.04`, `c = 2.42`): derivative `9.8e-9` over every atom,
  net internal force `7.6e-18` against `2.9e-2`, columns closed with the switch posting
  `−1.29e-2` on the dimer — 7/7.
- The freeze's plant (ii) was pre-derived from FIELD-5's failures under the TENTH, not under
  the derived tolerance, and did not fire under the latter: M-PLANT-OBS names this shape (a
  plant must be pre-checked to fire for THIS instrument); recorded, not softened.
- The delegate's `predict` phase waits on FIELD-5's tilted solve before launching FIELD-6's,
  to keep both inside the price window on 24 threads.
- No number enters from outside the engine and its own solver.
