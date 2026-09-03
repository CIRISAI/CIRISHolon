# GF2a — the exact arm reaches N = 10 on the device; the MPS arm is CONVICTED at χ = 64; nothing is cashed

*2026-09-02. Prereg `GF2A_QCD2_PREREG.md` (frozen alone). Instruments: `holon-chem::qcd2`
(two-string integrals AND colour lanes), `holon-gpu` `examples/qcd2_lanes` (the exact arm,
host and device), `q8-mps` `examples/qcd2_dmrg` (the MPS arm). Every number below is in the
committed logs `qcd2_lanes.log`, `qcd2_lanes_n10.log`, `qcd2_lanes_n10_resident.log`,
`qcd2_fci.log`, `qcd2_dmrg/*.json`.*

```
G0 (two solvers, one tensor, |E_FCI − E_DMRG(χ=64)| ≤ 1e-6 at N=8):  FIRED on the MPS arm
   N=8 x=4: B=0 +2.8e-2, B=1 +4.8e-3, B=2 +1.2e-4 ; x=9 B=2 +2.0e-4  (χ=64, above the exact)
   the χ=40 x=4 B=0 point reads −51.8346 under the absolute sweep tolerance (E3 timing run)
   and −51.3016 under the relative one: a penalty-sector METASTABLE state, not truncation.
   The MPS arm is retired for this rung; the ladder was stopped after N=8 (qcd2_dmrg_ladder.log).
G0 on the exact arm: PASS — colour lanes vs the two-string determinant route at N=4,6 to
   every printed digit (10 decimals), vs the dense Slater–Condon referee (plant i), host
   shards and device kernel bit-identical on every sector energy.
G1, G2, G3: NOT READ. N ≤ 10 is below the staked volume standard (N ≳ 20√x = 40 and 60), the
   mass is still descending like 1/N, and the frozen G1 ratio criterion (< 0.5 over three
   successive differences) is one that 1/N-convergent data cannot satisfy at these N
   (for a pure a/N tail the ratio at N = 6, 8, 10 is 0.60). That is a defect of the freeze,
   recorded here and NOT re-staked in this document. The reading stays OPEN.
plants (i)(ii)(iii): PASS on both routes (holon-chem tests qcd2_gauge, lanes_gauge)
```

## The exact arm (colour lanes, Cartan-neutral block; device, bit-identical to host)

| x | N | E₀(B=0) | E₀(B=1) | E₀(B=2) | M_B/g | U_BB |
|---|---|---|---|---|---|---|
| 4.0 | 4 | −24.5391166860 | −17.5847761876 | 0 (one det) | 1.738585 | +10.63044 |
| 4.0 | 6 | −38.2128396646 | −33.1848596866 | −19.1570928549 | 1.256995 | +8.999787 |
| 4.0 | 8 | −51.9229999638 | −47.9964825669 | −36.6401053164 | 0.981629 | +7.429860 |
| 4.0 | 10 | −65.6470061717 | −62.4301784810 | −52.9591823523 | 0.804207 | +6.254168 |
| 9.0 | 4 | −58.0230018617 | −41.7896555157 | 0 (one det) | 2.705558 | +25.55631 |
| 9.0 | 6 | −90.4871056656 | −78.7699472604 | −46.0059828234 | 1.952860 | +21.04681 |
| 9.0 | 8 | −123.0642401146 | −113.9136751337 | −87.5269948585 | 1.525094 | +17.23612 |
| 9.0 | 10 | −155.6819170182 | −148.1813486615 | −126.2212755359 | 1.250095 | +14.45950 |

Spaces: `C(N, n_q/3)³` — 216 / 8,000 / 343,000 / 16,003,008 at B=0. First run (host-driven
device sigma, subspace bound 12 because bound 48 was refused at the host door with the
ladder holding 20 GB): N=10 B=0 and B=1 exited `Stagnated` at residual 1.2–1.5e-10 in
396 s and 243 s. Re-run the same day on the DEVICE-RESIDENT solve (GANTT E13: the
Davidson's vectors on the card, the host holding the m×m eigenproblem, bound 48 priced
against VRAM): every N=10 sector `Converged` at residual ≤ 9.6e-11 in 79 / 47 / 6 s
(x=4) and 77 / 46 / 6 s (x=9), the same energies to the printed digit
(`qcd2_lanes_n10_resident.log`). The table above carries the resident run's values.

What the exact arm says on its own, unread as a gate: `M_B` falls with `N` at every
volume in both columns, and `U_BB` is positive and falls with `N` (ratios 0.85, 0.83,
0.84 at x=4; 0.82, 0.82, 0.84 at x=9 — above G2's 0.75 at these volumes). Neither is a
reading; both are what the successor instrument must extend past N = 10.

## The MPS arm at N = 8 (χ = 40 → 64 warm ladder, penalty sector, JW modes)

| x | B | χ=40 | Δ vs exact | χ=64 | Δ vs exact | max discarded (χ=64) |
|---|---|---|---|---|---|---|
| 4.0 | 0 | −51.3015858292 | +6.2e-1 | −51.8945707005 | +2.8e-2 | 6.4e-6 |
| 4.0 | 1 | −47.7574593186 | +2.4e-1 | −47.9916420916 | +4.8e-3 | 3.9e-6 |
| 4.0 | 2 | −36.4916958360 | +1.5e-1 | −36.6399863323 | +1.2e-4 | 7.2e-7 |
| 9.0 | 2 | −87.4178595438 | +1.1e-1 | −87.5267971899 | +2.0e-4 | 5.2e-7 |

A discarded weight of 6e-6 does not buy a 3e-2 energy error; the arm is stuck, not
truncated. Successor (GANTT E7): U(1)³ colour-number blocks in the MPS, which remove the
penalty and its metastability and make χ count only in-sector states — the same
conserved-integer-lane structure the exact arm runs on.

## Provenance

The exact arm is `holon-chem/src/lanes.rs` + `holon-gpu/src/lanes.rs` +
`kernels/lanes_sigma.cu` (GANTT E11), gated in `holon-chem/tests/lanes_gauge.rs`
(two lanes vs the string solver on random integrals, colour lanes vs the two-string route,
shards bit-identical, plants) and `holon-gpu/tests/lanes_sigma.rs` (device vs host
bit-identical: every sigma entry, the diagonal, the Davidson energies). Binaries:
- `qcd2_lanes` sha256 1409feb89c7e752ec4947df719d91cc05be095f6f9308a5c339b83dd50d0afd3
- `qcd2_dmrg` sha256 a0bcdef8a395cc0dc757f82c4b36497d01455b81e785b847a3a9a042e1dc55cc
- `qcd2` sha256 c4c5b9a4741a39669304575a8d7d9437e915e6476ff495d678165c23fbf330e2
- rustc rustc 1.95.0 (59807616e 2026-04-14), nvcc cuda_12.0.r12.0/compiler.32267302_0

---

## E7 — G0′ on the symmetric arm: **FIRED** at x = 4, B = 1 (2026-09-03, appended as the rungs land)

*Read against `GF2A_AMENDMENT_1.md` §A1.3, which stakes: at N = 8, both x, all three
sectors, `|E₀(exact) − E₀(MPS-sym, χ)| ≤ 1e-6` at some χ on the warm ladder 64 → 128 → 256,
and "a sector that meets none of the ladder's χ fires G0′ for the arm". The exact arm's
column above is the referee; the symmetric arm's own convergence test (energy change
≤ 1e-10·max(1,|E|), discarded weight ≤ 1e-8, at least four sweeps) is printed per rung.*

| x | B | n_q | states | χ | E₀(MPS-sym) | miss vs exact | ≤ 1e-6 | arm's own verdict | discarded | sweeps |
|---|---|---|---|---|---|---|---|---|---|---|
| 4 | 2 | 18 | 21,952 | 64 | −36.639986332414 | +1.190e-4 | no | not converged | 7.23e-7 | 60 |
| 4 | 2 | 18 | 21,952 | 128 | −36.640105311856 | +4.544e-9 | **MEETS** | converged | 3.33e-11 | 4 |
| 4 | 2 | 18 | 21,952 | 256 | −36.640105316416 | −1.599e-11 | **MEETS** | converged | 1.96e-18 | 4 |
| 9 | 2 | 18 | 21,952 | 64 | −87.526797317669 | +1.975e-4 | no | not converged | 5.19e-7 | 60 |
| 9 | 2 | 18 | 21,952 | 128 | −87.526994855406 | +3.094e-9 | **MEETS** | converged | 1.01e-11 | 4 |
| 9 | 2 | 18 | 21,952 | 256 | −87.526994858459 | +4.100e-11 | **MEETS** | converged | 1.48e-19 | 4 |
| **4** | **1** | **15** | **175,616** | **64** | −47.994211358802 | **+2.271e-3** | **no** | not converged | 7.52e-6 | 60 |
| **4** | **1** | **15** | **175,616** | **128** | −47.996352367153 | **+1.302e-4** | **no** | not converged | 1.89e-7 | 60 |
| **4** | **1** | **15** | **175,616** | **256** | −47.996416223802 | **+6.634e-5** | **no** | **converged** | 2.98e-9 | **4** |

### x = 4, B = 0 landed next, and it takes plant (iv) down with it

| x | B | n_q | states | χ | E₀(MPS-sym) | miss vs exact | ≤ 1e-6 | arm's own verdict | discarded | sweeps |
|---|---|---|---|---|---|---|---|---|---|---|
| **4** | **0** | **12** | **343,000** | **64** | −51.913392934209 | **+9.607e-3** | **no** | not converged | 1.01e-5 | 60 |
| **4** | **0** | **12** | **343,000** | **128** | −51.916887808623 | **+6.112e-3** | **no** | not converged | 3.82e-7 | 60 |
| **4** | **0** | **12** | **343,000** | **256** | −51.917057523964 | **+5.942e-3** | **no** | **converged** | 6.92e-9 | **4** |

**The miss is monotone in the size of the sector, and only the smallest one is solved.**
At x = 4: 21,952 states → 1.6e-11 (exact to the referee's own digits); 175,616 → 6.6e-5;
343,000 → 5.9e-3. Two orders of sector size cost eight orders of accuracy at the same χ.

**PLANT (iv) IS VOID ON THIS SECTOR — it no longer discriminates, and it fails on the side
that matters.** §A1.5 stakes it as a two-sided test at N = 8, x = 4, B = 0, χ = 64: *the
mutant must land more than 1e-3 from the exact arm's energy, and the successor must not.*
Measured, both halves now in hand:

| arm | χ = 64 | miss vs exact −51.9229999638 |
|---|---|---|
| mutant (labels ignored, `mutant_x4.0_N8_B0.json`) | −51.917869990835 | +5.13e-3 |
| **successor (the shipped symmetric arm)** | −51.913392934209 | **+9.61e-3** |
| successor at its BEST rung (χ = 256) | −51.917057523964 | **+5.94e-3** |

The mutant's half passes; **the successor's half fails**, at χ = 64 as staked and at every
richer rung besides. At its best the correct arm is FURTHER from the exact answer than the
deliberately broken one. When the earlier record said plant (iv) "fires as designed" only
the mutant had been run; with both halves measured the plant is a detector that cannot tell
its two causes apart on this sector, so it says LOOK and it does not say the labels are
load-bearing. What the passing B = 2 sectors do still say is narrower and worth keeping: at
21,952 states the labelled machinery reproduces the referee to 1e-11, so the label
bookkeeping is not broken — it is out of reach at eight and sixteen times that size.

**G0′ FIRES.** `x = 4, B = 1` meets none of the ladder's χ — its best rung is 66× outside
the stake — and by §A1.3's own words that fires the gate for the arm. The ladder is not
extended past 256 without a further amendment, and by §A1.6 **nothing downstream is read:
G1′, G2 and G3 stay unread on the symmetric arm.** Both `x = 9` non-trivial sectors are still running and are appended when they land; they
cannot un-fire this, and `x = 4, B = 0` above has already made it worse.

**The discarded weight cannot buy the error, which is the same conviction that retired the
penalised arm (§A1.1).** On the sector that passes, the two quantities track: at `x = 4,
B = 2, χ = 128` a discarded weight of 3.3e-11 comes with a 4.5e-9 miss, a ratio of ~140. On
the sector that fires, `χ = 256` reports a discarded weight of 3.0e-9 with a 6.6e-5 miss —
a ratio of ~22,000, two orders worse. A truncation that small cannot be responsible for an
error that large, so the state is not truncated; it is **converged inside the wrong
variational manifold**.

**Two measured facts point at the ladder itself, and neither is a mechanism yet.**

1. **The lower rungs never converged.** χ = 64 and χ = 128 both ran the full 60 sweeps with
   `converged = false`, so the warm ladder carried a state that had not converged at
   χ = 128 into χ = 256.
2. **The top rung then converged in the MINIMUM four sweeps.** That is the fixed-point
   symptom this instrument has already been caught by once (§A1.8.1): once χ is large
   enough that truncation stops moving the state, the sweep-to-sweep energy change falls
   under the test's leg (a) whether or not the state is right. The passing sector shows the
   same four-sweep signature at 1e-11, so the signature alone does not discriminate — the
   miss does.

**The named suspect, stated as the next test rather than as a finding.** §A1.8.2 established
on this instrument that a charge sector absent from both neighbouring bonds can never
reappear, which is why the ladder starts at 64 and rescues blocks in the split. The
hypothesis is that at N = 8, B = 1 — eight times the states of the sector that passes — the
χ = 64 rung's block content is already deficient and every warmer rung refines inside that
deficiency. **It is untested here.** The test that would separate it from ordinary
metastability is a COLD χ = 256 run of that sector from a fresh seeded labelled start: if a
cold 256 reaches the referee, the ladder's inheritance is the mechanism and the gate's
warm-ladder design is what fired; if a cold 256 lands in the same place, the arm's ansatz or
its label set is short at this sector size and that is a deeper finding. Either way it needs
an amendment, because the frozen ladder is 64 → 128 → 256 warm and this document does not
get to change it after reading it.

---

## E7's misfit reading, and the retro-refusal of the amendment that governs it (2026-09-03)

*Run after G0′ fired, by putting the registry (`conformance/gravity/MISFITS.md`) and the
audit (`Audit/prereg_audit.py`) against this campaign's own documents and its own JSONs.
Three findings, in the order they hurt.*

### 1. The arm's record could not tell success from stagnation — and the registry already had that name

Every rung's `worst_residual`, across every sector and every χ:

| sector | χ | miss vs exact | worst Lanczos residual | max discarded | `converged` |
|---|---|---|---|---|---|
| x=4 B=2 | 256 | **−1.6e-11** | 9.97e-12 | 1.96e-18 | true |
| x=4 B=1 | 256 | **+6.6e-5** | 9.99e-12 | 2.98e-9 | true |
| x=4 B=0 | 256 | **+5.9e-3** | 9.98e-12 | 6.92e-9 | true |

The local residual is **pinned at its own stopping tolerance in all twelve rungs** — 9.8e-12
to 1.0e-11 — identical where the answer is right to eleven decimals and where it is wrong by
six milli-hartree. It is a fact about when the solver stopped, not about what it found, and
it cannot discriminate. The discarded weight does vary, and it is no better as an error bar:
the ratio (miss ÷ discarded) runs 1.4e2, 2.2e4, 8.6e5 across the three sectors — four orders
of drift in the quantity leg (b) of the convergence test reads. And `converged` is `true` on
all three, including the two that are wrong.

**M-EXIT-DISCRIMINATOR was registered before this campaign was frozen** and says it in
advance: *a solve record keeping only the residual makes iteration-cap and
subspace-stagnation indistinguishable … the discriminator field must be READ, not merely
carried.* `GF2A_AMENDMENT_1.md`'s `misfits:` line does not cite it. Registered today from
the second half of the same reading: **M-TRUNCATION-AS-ERRORBAR**, that a variational
method's own truncation measure is a self-consistency reading which degrades exactly where
the ansatz fails.

**Neither could have been caught.** Both ids were UNARMED — no contact keyword — so a freeze
could contact their shapes freely, and 22 of the registry's 42 ids still are. Both are armed
now (`iteration cap`, `sweep cap`, `stagnation`; `discarded weight`, `truncation error`),
narrowly, and verified to newly refuse **zero** of the 57 freezes and amendments in the tree.

### 2. The audit never looked at this document at all

`ci-gates.sh` globbed `conformance/*/*_PREREG.md`. **Amendments were not in the glob**, and
that hole was nowhere stated — unlike the `engine/*_PREREG.md` hole, which the script names
with an owner and an exit. So the document that retired one instrument, admitted its
successor and staked G0′, G1′ and two plants was never audited. The glob now takes
`*AMENDMENT*.md`; the gate sees 57 documents where it saw 55.

### 3. And when it is audited, it REFUSES

> `REFUSED GF2A_AMENDMENT_1.md — witness does not resolve in lean/CIRISHolon: macro_law_forced`

G1′ names `macro_law_forced` as its witness. That theorem is **CIRISOntology's**
(`Core/Closure.lean`), not this repo's, and the amendment's own §A1.4 says so in words. The
audit has no way to express a cross-repo witness, so the honest form was
`witness: none (macro_law_forced is CIRISOntology's)`. The amendment is frozen and cannot be
edited to comply, so the remedy is the one the seven gravity freezes took: **this paragraph
is the retro-refusal, on the record**, and `GF2A_AMENDMENT_1.md` carries a CI exemption
naming the crystal lane as owner and this record as its exit.

**What this does NOT change.** G0′ still fired, on the numbers in §E7 above and for the
reasons given there. The misfit reading does not rescue the arm — it explains why the arm's
own record said everything was fine while it was wrong by 5.9e-3, and it says the successor
amendment must stake a criterion that an independent referee can fail, not one the solver
can satisfy by stopping.
