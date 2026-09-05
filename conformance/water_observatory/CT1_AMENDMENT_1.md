# CT-1 — AMENDMENT 1: the closed sector is the monomers' own orbitals, not the orthogonalised ones

*Frozen 2026-09-05, committed alone, BEFORE any water reading of the instrument exists: at
the time of this commit `conformance/water_observatory/ct1/` does not exist. Written
because the instrument as the freeze defined it — "a full CI restricted to the sector of
the dimer's determinant space over the orthogonalised monomer orbitals where each fragment
holds its own electron count" — refuted itself on its unit test: on a hydrogen-molecule
pair at 3 bohr the sector's ground state came out at `−1.8601` hartree, ABOVE the undeformed
Heitler–London product's `−2.1579` and only a little below the orthogonalised product's
`−1.8066`. The freeze's own T0 order (`E_exact ≤ E_noCT ≤ E_HL(undeformed)`) fired, on the
instrument, before a water node was touched. The cause is FIELD-5's lesson read again:
the sector of determinants over the ORTHOGONALISED orbitals `C' = C·M^{−1/2}` is a sector
of deformed monomers, and the undeformed product state — the vector that lies in the
physical closed sector — is spread over the whole `C'` space by the orthogonalisation
(FIELD-7's W0 measured exactly that: nonzero on all `1,002,001` determinants). A mask on
`C'` determinants cuts through the physical sector rather than around it.*

misfits: contacts **M-STALE-INSTRUMENT** (this amendment alone); **M-VACUOUS-SUCCESS** (the
new solve asserts its residual and its subspace count before its energy is read);
**M-EXIT-DISCRIMINATOR** (the new Davidson records its exit reason and iteration count; an
iteration cap is VOID); **M-CHEAPER-THAN-ITS-PRICE** (the price is re-stated below);
**M-PLANT-OBS** and **M-PLANT-SECTOR** (plant (i) is re-derived for the new solve: its
carrier `|E_CT(2.9 Å)| ≥ 1e-3` hartree is asserted nonzero in the sector the plant acts on —
the closed sector's restriction — exactly as the freeze stated it); **M-HOMOG** (the word
"localised" appears; the block-localised space is a subspace, nothing homogeneous).

## The change

The closed sector is the span of the products of the two monomers' determinants in their
OWN (mutually non-orthogonal) orbitals: `B = span{ |D_s^A ⊗ D_t^B⟩ }`, `441 × 441` states on
the water dimer — the block-localised space of Mo, Gao and Peyerimhoff, taken at full CI
within each fragment. Expressed in the orthonormalised basis, each product string of one
spin is a column of the expansion matrix `T` the undeformed product already uses
(`heitler_london.rs`, the minors of `M^{1/2}`), so a vector of `B` with coefficients `C`
(`441 × 441`) is `v = T_α · C · T_βᵀ`, and the sector's metric is `S = S_α ⊗ S_β` with
`S_α = T_αᵀ T_α` (`441 × 441`, the overlaps of the product strings). The sector's ground
state is the lowest eigenvalue of the generalised problem `H_B C = E S C`, solved as the
symmetric problem in `D = S^{1/2} C`: the operator `A D = S_α^{−1/2} · T_αᵀ H (T_α S_α^{−1/2}
D S_β^{−1/2} T_βᵀ) T_β · S_β^{−1/2}`, one full-space Hamiltonian application per step (the
same `sigma` as everything else) plus four contractions. The Davidson runs in the `194,481`-
dimensional `D` space, started from the product state (`D₀ = S_α^{1/2} c_A c_Bᵀ S_β^{1/2}`,
which IS the undeformed Heitler–London vector), preconditioned by the orthogonalised
sector's diagonal (the same combinatorial index set; a preconditioner, not the operator),
with the residual bar `1e-8` on the residual norm and a cap of `300` iterations. `E_noCT` is
the converged Rayleigh quotient; `E_CT = E_exact − E_noCT` as frozen.

Gates unchanged: T0's order now reads on THIS solve (and is expected to hold by
construction: the product state is in `B`, so the sector's minimum is at or below it); the
in-sector norm leg becomes the residual of the generalised problem (`≤ 1e-8`), since every
vector of the new solve is in the sector by construction; plant (i) (the sector restriction
removed: the same Davidson on the full space from the same start, reproducing `E_exact` to
`1e-8`) stands. The instrument's unit test on the hydrogen pair — order, plant, far limit —
must pass before a water node is read.

- **T0 (amended) — the sector is what it says.** The generalised residual `≤ 1e-8`, exit
  `Converged`, the subspace dimension `194,481` (EXACT); `E_exact ≤ E_noCT ≤
  E_HL(undeformed)` within `1e-10`; at 40 bohr `|E_CT| ≤ 1e-8`.
  witness: none (a residual, a count, an order, a limit)

Price: one `sigma` (FIELD-5's `55–105` core-seconds) plus contractions of order `10⁹` flops
per iteration; `50–300` iterations per node; the twelve nodes and the 40-bohr reading
priced at that, recorded per node, refused under a tenth of the iteration count times the
floor.
