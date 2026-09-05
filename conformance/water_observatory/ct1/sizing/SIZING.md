# Sizing channels 3 and 4 for WATER — what it would cost, and which instrument carries it

*Delegate sizing to `CT1_PREREG.md` §4, 2026-09-05. SCRATCH, not a campaign artifact and not
a harvest. Every determinant count is computed in the engine (`holon_chem::fci::Strings`,
cross-checked against an exact binomial); every price is `holon_chem::budget::price_determinant`
put to the live admission probe on this host; every energy is `solve_determinant` through
`density_embed`. Instrument: `engine/crates/holon-chem/examples/channel34_sizing.rs`, run on
4 cores (`taskset -c 24-27`, `LANE_THREADS=4`). Host: 31 GiB RAM, ~24 GiB available.*

---

## 0. The verdict, first

1. **The exact water trimer is out of reach and so is the embedding route to it.** The trimer
   is 2,944,581,696 determinants; its Davidson working set is 2,281.6 GiB and the door
   REFUSES it. The finding that matters is that **the density embedding does not help here**:
   `density_embed::subset_in_field(frags, dens, &[0,1,2], None)` on a three-fragment list has
   no partners left to embed in, so it *is* the exact trimer supermolecule. EMBED-3's
   instrument removes the FOURTH body, not the third. Channel 4 for water is refused at
   exactly the same door on both routes.
2. **What the embedding *can* measure for water today is channel 2's three-body part, not
   channel 4's.** Measured on the cyclic ring, 377 s on 4 cores: three-body induction
   `E_3^SCF = +8.864533e-05 Ha = +0.0886 mHa`, repulsive (anticooperative), `4.39e-3` of the
   ring's own embedded binding.
3. **Channel 3 is not a measurement problem at this basis, it is an absence.** FIELD-6 and
   FIELD-8 both read `C_6 = 0`: the remainder after charges, wall and contact decays with
   log-log slopes `-10.8, -12.0, -13.6`, which is overlap, not `R^-6`. A minimal basis has no
   virtual p on hydrogen, so there is almost nothing for dispersion to be made of. Adding one
   p shell puts the determinant route 24x to 800,000x over this host's RAM — **at the closed
   sector, not only at the full space**. The only route whose price fits at 20-26 orbitals is
   the labelled MPS one.

---

## 1. Channel 4 (three-body dispersion) — the exact route

### The counts, from the engine

`Strings::new(n_orb, n_e).masks.len()` squared, each cross-checked against an exact binomial
(`assert_eq!` in the example; all four verified cases agree).

| space | n_orb | n_a=n_b | strings | determinants | Davidson working set | the door |
|---|---|---|---|---|---|---|
| water monomer, STO-3G | 7 | 5 | 21 | 441 | 0.36 MB | ADMITTED |
| water DIMER, STO-3G (of record) | 14 | 10 | 1,001 | 1,002,001 | 0.78 GiB | ADMITTED |
| HF trimer (EMBED-2/3's carrier) | 18 | 15 | 816 | 665,856 | 0.52 GiB | ADMITTED |
| **water TRIMER, STO-3G** | **21** | **15** | **54,264** | **2,944,581,696** | **2,281.6 GiB** | **REFUSED** |
| water dimer + 1 p-shell on 2 H | 20 | 10 | 184,756 | 34,134,779,536 | 26,449.7 GiB | REFUSED |
| water dimer + 1 p-shell on 4 H | 26 | 10 | 5,311,735 | 28,214,528,710,225 | 21,862,320 GiB | REFUSED |

The refusal is the engine's own words:

```
REFUSED at the admission door: determinant route, 2944581696 determinants x 104 vectors
(subspace bound 48) would hold 2449891971072 bytes of RAM and the probe said "the ask
exceeds the kernel's available RAM" (price provenance: computed from tier.rs's Davidson
allocations (2·max_sub + 8 vectors)).
```

**There is no longer a determinant "cap" to compare against.** The hard cap of 2,000,000 and
the MPS routing threshold of 50,000 were deleted on 2026-09-02 (`fci.rs:1002`, `budget.rs`
header) because each was a price measured in one regime and consumed as physics. The gate is
the price at the door. Stated against the deleted cap anyway, for scale: the trimer is
**1,472x** the old cap, and **2,939x** the largest space this programme has actually solved
(the 1,002,001-determinant water dimer).

### The Heitler–London product of three monomers — also out of reach, and by memory first

`heitler_london_undeformed` builds the product state as a per-spin-lane expansion
`T (n_dimer_strings x n_monomer_string_products)` and then takes ONE sigma. For three water
monomers:

| item | size |
|---|---|
| expansion matrix `T`, per spin lane: 54,264 x 21³ = 54,264 x 9,261 doubles | 3.74 GiB |
| both spin lanes | 7.49 GiB |
| the product vector `v` and the sigma written from it: 2 x 2,944,581,696 x 8 B | 43.87 GiB |
| **floor for ONE evaluation of `<v|H|v>`** | **51.4 GiB** |
| the `det T[Q,P]` determinants alone, per lane | ~3.4e12 flops |

51.4 GiB against 31 GiB of RAM on this host, and that is the floor before the Hamiltonian's
own tables. **One energy evaluation of the three-monomer product state does not fit**, so the
undeformed-referee route (FIELD-6's instrument) does not extend to the trimer either.

### The named instrument for channel 4 on water

The MPS route. `budget::price_mpo(21) = 1.76 GiB` (PROVISIONAL — the bond dimension is fitted
to one measurement). This is EMBED-2's own closing line, unchanged by this sizing: *"the water
triple (21 orbitals) is the labelled-MPS base's job."* Nothing else on this host has a price
that fits.

---

## 2. The measurement: the ring, and what the embedding gives without a trimer solve

### The geometry (`water_ring`, in the example)

Three monomers at EMBED-1's pin (`r = 1.9435738400`, `theta = 1.6887434037`), oxygens on an
equilateral triangle of side 2.9 Å in the xy-plane, each donating one O–H straight at the next
oxygen around the ring, the free O–H tilted outward by the monomer angle. Weights `[-2, 1, 1]`.

| quantity | bohr | Å |
|---|---|---|
| O–O, all three (C₃ by construction) | 5.480206 | 2.9000 |
| **shortest cross-unit H···O** (the three donor bonds) | **3.536632** | **1.8715** |
| next H···O (donor to the far oxygen) | 4.812375 | 2.5466 |
| shortest cross-unit H···H | 3.067823 | 1.6234 |

**The over-binding warning applies.** The shortest H···O is 1.87 Å, far inside 2.9 Å; this is
a CONTACT geometry, where FIELD-4 showed the density embedding is not a field and over-binds.
It is not inside FIELD-8's closure boundary: the donor hydrogen sits at 1.81x the 1.95 bohr
where a hydrogen changes hands, so the three units are still three units and the seam law's
premise holds.

### What was measured (377 s wall on 4 cores)

Composition is the engine's own, generalised from `embed3_campaign.rs::run_water`
(`E_A[B] + E_B[A] − E_cl(A,B)`) to any subset:

```
E_emb(S) = Σ_{i∈S} E_i[fixed-point field of the others in S] − Σ_{i<j∈S} E_cl(i,j)
ΔE(S)   = E_emb(S) − Σ_{i∈S} E_i(isolated)
E_3^SCF = ΔE(ABC) − ΔE(AB) − ΔE(AC) − ΔE(BC)
```

| quantity | value (Ha) | mHa | sweeps | both starts |
|---|---|---|---|---|
| E_i(isolated), all three | −7.491107035863e+01 | | | |
| E_emb(ABC) | −2.247534093555e+02 | | 7 | Δρ = 0.0 |
| E_i[field of the other two], all three | −7.492461986008e+01 | | | |
| ΔE(ABC) | −2.019828e-02 | −20.1983 | | |
| E_emb(AB) = E_emb(AC) = E_emb(BC) | −1.498289030256e+02 | | 6 | Δρ = 0.0 |
| ΔE(pair), each | −6.762308e-03 | −6.7623 | | |
| **E_3^SCF** | **+8.864533e-05** | **+0.0886** | | |
| E_3^SCF / ΔE(ABC) | −4.3888e-03 | | | |

The three pairs agree to every printed digit and the two density starts converge to
bit-identical densities on all four fixed points — the ring's C₃ symmetry and EMBED-3's G1,
reproduced. Energies are written to `scf3.json` at `{:.17e}` (M-FORMAT-FLOOR).

**What this number IS.** The non-additivity of the self-consistent polarisation: channel 2
(induction), arity 3. It costs only monomer solves (441 determinants each) and is therefore
the one three-body quantity for water that is cheap today.

**What this number is NOT.** It is not channel 4. Three-body dispersion is a correlation
effect that lives in the trimer wavefunction; the density embedding's fixed point is a mean
field and carries none of it. EMBED-2 got channel 4 for the HF chain *because* it could solve
the exact trimer (665,856 determinants); it read `r_ρ = −C/R⁹, C = 8.52 Ha·bohr⁹` as the
residual of the two-body embedded view against that exact solve. Water has no such solve.

Two further cautions on the number, stated rather than buried. STO-3G carries no polarisation
functions, so induction is structurally underestimated at this basis; and the embedding is
Coulomb-only frozen density (Wesolowski–Warshel's Coulomb part, no Pauli term), which at 1.87 Å
contact is the regime FIELD-4 convicted. `+0.0886 mHa` is a sizing reading with its
instrument named, not a physical claim about the water trimer.

### The pairwise leg, priced

`rho_pa_subset(frags, dens, &[0,1,2], None)` needs, beyond the monomer solves:

- three **dimer-in-field** solves at **1,002,001 determinants** each (14 orbitals, 10α10β,
  in the third monomer's frozen density).
- From the record: EMBED-3 System B measured an isolated water dimer at 574.5–950 wall-seconds
  on 32 threads = **14,508 processor-seconds**; FIELD-6 and FIELD-8's CONTACT solves cost
  **23,507** and **23,660 core-seconds** at 217 and 235 Davidson iterations. This ring's
  dimers are contact dimers.
- On 4 cores that is **1.0–1.6 h per pair, 3–5 h for the three**. Launched detached.
- The trimer term `E_ABC` that would complete `r_3` is REFUSED, above. **So the pairwise leg
  cannot be cashed into a three-body residual for water even when it finishes.**

**Phase 4, partial (detached, still running at the time of this report).** The three-fragment
density fixed point converged in **7 sweeps, 78.5 s** on 4 cores;
`E_i[field of the other two] = −7.492461986008e+01 Ha` for all three, identical to phase 2's.
Then the exact-trimer term was priced and refused in the run's own words, which is the whole
of the finding:

```
  subset_in_field(frags, dens, [0,1,2]) IS the exact trimer supermolecule.
  REFUSED at the admission door: determinant route, 2944581696 determinants x 104 vectors
  (subspace bound 48) would hold 2449891971072 bytes of RAM and the probe said "the ask
  exceeds the kernel's available RAM" ... The escalation is a lease on a bigger box or the
  next tier, never an edit to a constant.
```

The first of the three 1,002,001-determinant dimer-in-field solves is running. Its wall,
Davidson iteration count and residual land in `pa_partial.json` as each completes.

---

## 3. Channel 3 (pair dispersion)

### The record already answers the physics

FIELD-6 (six linear nodes) and FIELD-8 (66 orientations) both report **`C_6 = 0`, not
transferred**. FIELD-6's remainder after charges, wall and contact has log-log slopes
`-10.79, -12.04, -13.59` against dispersion's band `[-8, -4]`. That is an overlap-driven
exponential, and at the hydrogen-bond minimum it is `-10.4` mHa — the quantity CT-1 is now
splitting into closed-sector polarisation-plus-correlation and charge transfer. **At STO-3G
there is no `-C_6/R^6` tail to isolate**, because a minimal basis gives hydrogen no virtual p
function and oxygen no virtual shell above 2p: the instantaneous dipole–dipole excitations
dispersion is made of have almost nowhere to go.

### The instrument that would isolate it

The inter-fragment correlation energy of the CLOSED sector, from CT-1's instrument:

```
E_disp,inter(R) = E_noCT(R) − E_HL,undeformed(R) − E_ind(R)
```

- `E_noCT` — the closed sector's ground state. **This instrument now exists in the working
  tree**, landed by the lead while this sizing ran, and CT-1's own gate has already moved it:
  `fci_no_charge_transfer_orthogonalised` (the freeze's masked orthogonalised sector) is
  **REFUTED by CT1_AMENDMENT_1** — that sector is a sector of DEFORMED monomers, the
  undeformed product does not lie in it, and on a hydrogen-molecule pair its ground state
  sits ABOVE the product. The closed sector proper is **`fci_block_localised`**, whose
  `sector_dim = n_det(A)·n_det(B)` and which carries the sector metric's smallest eigenvalue
  as its conditioning reading.
- `E_HL,undeformed` — `heitler_london_undeformed`, already built and gated (FIELD-6 G-U0), and
  already measured on the six linear nodes.
- `E_ind` — the density-embedding fixed point's induction, the same machinery phase 2 above
  used, so no new solve class is needed.

CT-1's prereg already names `E_noCT − E_HL(undeformed)` as "induction plus the inter-fragment
correlation this basis carries (channels 2 and 3 together)". Subtracting a separately measured
`E_ind` is what splits the pair, and that subtraction is the whole of channel 3's instrument:
no new solver, one more arithmetic line on readings CT-1 and FIELD-6 already produce.

### The cost, at this basis

The closed sector, engine-counted. `fci_block_localised` defines it as
`sector_dim = n_det(A)·n_det(B)`, which is `[C(n_A,5)·C(n_B,5)]²`; at STO-3G that is
`441² = 194,481`, CT-1's stated count exactly.

| basis | fragments | sector = [C(n_A,5)·C(n_B,5)]² | working set | the door |
|---|---|---|---|---|
| STO-3G (CT-1 as frozen) | 7 + 7 | **194,481** | 0.16 GiB | **ADMITTED** |
| +1 p-shell on one H of each monomer | 10 + 10 | 4,032,758,016 | 3,124.8 GiB | REFUSED |
| +1 p-shell on both H of one monomer | 13 + 7 | 730,458,729 | 566.0 GiB | REFUSED |
| +1 p-shell on every H | 13 + 13 | 2,743,558,264,161 | 2,125,875 GiB | REFUSED |

**Number of sector solves.** CT-1 takes twelve (plus the 40-bohr limit and one held-out
prediction). A `-C_6/R^6` tail needs an exponent read between consecutive far nodes, and
CT-1's twelve geometries already span 2.3–3.4 Å plus 40 bohr, so **the solve COUNT is not the
binding constraint** — twelve is enough, and at STO-3G they are affordable (0.16 GiB each,
priced in the freeze at 55–105 core-seconds per Hamiltonian application times the iteration
count). What is missing is not solves; it is basis.

**Basis growth, and where it breaks.** For dispersion to exist at all each fragment needs at
least one polarisation shell:

| growth | full space | Davidson working set | closed sector | sector working set |
|---|---|---|---|---|
| p on two H (n_orb 20) | 3.4135e10 | 26,450 GiB | 7.30e8 (13+7) | 566 GiB |
| p on all four H (n_orb 26) | 2.8215e13 | 21,862,320 GiB | 2.744e12 (13+13) | 2,125,875 GiB |

The **cheapest** polarised object in that table — the 13+7 closed sector at 730,458,729
determinants — still asks 566 GiB, **24x this host's total RAM**. The determinant route does
not reach a resolvable dispersion tail for the water dimer at any polarised basis, on this box
or on a plausibly larger one (the balanced 10+10 sector asks 3.1 TiB).

The MPS route's prices at those orbital counts are `price_mpo(20) = 1.38 GiB` and
`price_mpo(26) = 5.12 GiB`, both PROVISIONAL. As with channel 4, it is the only route whose
price fits.

---

## 4. In reach today, and not

**In reach.**
- The three-body INDUCTION of a water ring, from monomer solves alone: measured above,
  +0.0886 mHa, 377 s on 4 cores.
- CT-1's twelve closed-sector solves at STO-3G (194,481 determinants, admitted), and with
  `E_HL,undeformed` of record plus an embedding `E_ind`, a three-way split of FIELD-6's
  `-10.4` mHa into induction, inter-fragment correlation and charge transfer. That is
  channel 3's SIZE at this basis — expected small, and its smallness is the finding, not a
  failure.
- The pairwise-additive embedded sum for the ring: 3 dimer-in-field solves, 3–5 h on 4 cores.

**Not in reach, with the reason.**
- The exact water trimer: 2.94e9 determinants, 2,281.6 GiB, refused. 2,939x the largest space
  this programme has solved.
- The embedding's three-body residual for water: refused at the SAME door, because the
  embedding removes the fourth body and the third body is still an exact trimer solve.
- The three-monomer Heitler–London product: 51.4 GiB for one energy evaluation, against 31 GiB.
- A `-C_6/R^6` tail: needs a polarised basis; the cheapest polarised closed sector asks 24x
  this host's RAM.

**The two named successors**, neither of them a determinant solve: the labelled MPS base at
21 orbitals (`price_mpo` 1.76 GiB, PROVISIONAL) for channel 4, and the same route at 20–26
orbitals for channel 3.

---

## 5. Files

- instrument: `/home/emoore/CIRISHolon/engine/crates/holon-chem/examples/channel34_sizing.rs`
- counts: `.../scratchpad/ch34/counts.json`
- the ring measurement: `.../scratchpad/ch34/scf3.json`
- the pairwise leg (detached, partial): `.../scratchpad/ch34/pa.log`, `pa_partial.json`
