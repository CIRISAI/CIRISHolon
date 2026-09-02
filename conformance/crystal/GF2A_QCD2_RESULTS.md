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
