# Prior art for the two water builds — the far field and the cluster seam

*Searched 2026-09-04, before either build started, at the operator's order. What each
group did, what they reported, and what it changes about our design. Runtimes are quoted
only where a paper states one; none of these do.*

## 1. The far field for water and ice (build 1)

**The embedded many-body expansion for ice — Gillan, Alfè, Bygrave, Taylor and Manby, 2013**
(arXiv:1307.3767; *Energy benchmarks for water clusters and ice structures from an embedded
many-body expansion*). Ice Ih, II and VIII. Their expansion is EMBEDDED: every monomer,
dimer and trimer is computed inside an approximate representation of the rest of the
crystal's Coulomb and exchange-repulsion field. Reported: at MP2, the embedded expansion
truncated at the TWO-body level reproduces the correlation energy of standard periodic
methods to better than 0.1 mEh per monomer, and MP2 near the basis-set limit reproduces the
experimental lattice energies well; coupled-cluster is called essential for the many-body
(non-additive dispersion) part.

**The bare expansion for ice** (OSTI 1924510, *A Formulation of the Many-Body Expansion for
ice*): the unembedded expansion converges at the four-body term, with the total four-body
contribution to the lattice energy between 0.5 % and 5.7 % across seven ice phases; five-body
terms were not evaluated because they run into numerical instability.

**MB-pol — Paesani and co-workers.** A many-body potential built entirely from CCSD(T)
reference data: explicit one-, two- and three-body terms, with everything above three body
carried by a CLASSICAL POLARISATION model rather than by higher terms of the expansion. It
reproduces the vapour–liquid equilibrium of water (J. Chem. Phys. 154, 211103, 2021;
arXiv:2103.06978) and is the reference potential for the model's phase diagram. That
polarisation model IS a far field: it is how the expansion is truncated at three-body and
still closes.

**Periodic fragment methods** (PMC3517951, *Fragment-based quantum mechanical methods for
periodic systems with Ewald summation and mean image charge convention*): the far field of a
fragment expansion in a periodic box is handled by Ewald summation of the embedding charges
with a mean-image convention, which is the standard where the fragments carry charges.

### What this changes for build 1

Our engine carries no electrostatic term at all (`B2_PREREG.md` §1.1), and B2 built the
far tail as a declared power-law model on tabulated pair curves, absolutely convergent by
construction, with Ewald named as the exit for `p ≤ d`. The literature does not do it that
way for water: every method above carries the far field as an EMBEDDING FIELD — point
charges, or a polarisation model — around each fragment, not as a tail on a pair table. The
difference matters for ice specifically, because a hydrogen-bond network's long-range order
is an electrostatic fact and a pair-tail model has no charges to be ordered by. Build 1
should therefore be the embedding field, with the pair-tail model kept as the control that
says how much of the answer the embedding adds. Ewald is prior art from 1921 and PME from
1993 and neither needs re-deriving; what is ours is the honesty machinery around them.

## 2. The cluster seam — exact cores over a far field (build 2)

**Density matrix embedding theory — Knizia and Chan, 2012** (Phys. Rev. Lett. 109, 186404;
arXiv:1204.5783 and 1212.2679). A fragment of a large system is solved EXACTLY (any
high-level solver, DMRG included) inside a small, rigorously constructed quantum bath that
reproduces the fragment's entanglement with its environment; exact in the non-interacting
and atomic limits; a strong-coupling alternative to dynamical mean-field theory. This is the
seam's shape, with the far field being a mean-field bath rather than an electrostatic one,
and it is the standard construction for "exact core, approximate surroundings" today.

**The electrostatically embedded many-body expansion — Dahlke and Truhlar, 2007**
(J. Chem. Theory Comput. 3, 46 and 4, 1). Dimers and trimers of fragments are computed in
the field of point charges representing every other fragment; the embedded three-body
expansion reproduces full water-cluster energies to 0.03 kcal/mol on average. This is the
cheapest form of the same idea and the one most directly comparable to our ladder.

**Fragment methods with correlated cores in a periodic Hartree–Fock field** (J. Chem.
Theory Comput. 16, 7100, 2020): a fragment defined as localised occupied and virtual
orbitals of a converged periodic HF solution, treated post-HF inside the Coulomb and
exchange potential of the rest of the crystal.

### What this changes for build 2, and it is the important finding

Our five-body audit (`DE5_RESULTS.md`) found the ladder does NOT terminate at four: 24 of
24 compact clusters over bound, the worst by 1,572×. Gillan et al. find the ice expansion
converges at TWO-body. The two results are not in tension: **theirs is embedded and ours is
bare**. Every fragment in their expansion sits in the field of the rest of the crystal, so
the higher-body terms are absorbed into the embedding and the series closes; every cluster
in ours sits in vacuum, so the higher-body terms carry everything the field would have and
the series does not close. The seam node's premise — "exact solves of compact cores over a
many-body far field" — is right in shape and wrong in one word: the far field must be an
EMBEDDING the cores are solved inside, not a sum added to them afterwards. That is the
difference between DMET or EE-MBE and what node MPS was going to build.

## 3. Metallic hydrogen, since the second build was asked about it

**Coupled electron–ion Monte Carlo — Pierleoni, Ceperley, Morales** (Phys. Rev. Lett. 93,
146402, 2004; arXiv:physics/0405056; the equation of state in later work). Path integrals
for the protons at finite temperature coupled to reptation quantum Monte Carlo for the
ground-state electrons, applied to hydrogen beyond molecular dissociation; the proton
crystal's melting and the liquid structure differ materially from Car–Parrinello results.
Ceperley and Alder's jellium phase diagram is the electron-gas anchor beneath it. **This is
how metallic hydrogen is actually computed: stochastic, delocalised, with the electrons
never fragmented.** No fragment expansion, embedded or bare, is used for it by anyone, and
our cryo arm 3 measured why (the expansion never converges at any density). DMRG and its
tensor-network relatives are, per the pedagogical literature (arXiv:2304.13395), effective
in one and quasi-one dimension and an open research problem in three. The honest statement
stands: build 2 helps ice and does not reach metallic hydrogen.

## 4. Compute time

None of the papers above reports wall-clock time or core-hours. No comparison of speed is
made here, in either direction, and none may be made until one is measured on both sides.

## 5. Prior art for EMBED-2 and EMBED-3 — the field as monomer densities, the residual as dispersion, the precision floor (searched 2026-09-04, while EMBED-3 ran)

**The fragment molecular orbital method — Kitaura, Fedorov and co-workers.** FMO divides a
system into fragments and computes each fragment and each pair INSIDE the electrostatic
potential of all the other fragments' nuclei and electron densities, the monomer densities
iterated to self-consistency (Kitaura et al. 1999; Fedorov and Kitaura, *J. Chem. Phys.* 120,
6832, 2004, "The importance of three-body terms in the fragment molecular orbital method";
Fedorov and Kitaura, *Chem. Phys. Lett.* 433, 182, 2006, the three-body FMO3 formulation;
review Fedorov, *WIREs Comput. Mol. Sci.* 2017). **EMBED-2 is FMO2's embedding scheme in the
EE-PA form**: the Coulomb potential of self-consistent partner densities and nuclei, no
exchange, and the pairwise sum with the monomer field terms cancelling. FMO3 is the
three-body correction — the seam's own shape. FMO's practice also settles a design point we
had left open: FMO uses the ESP of the densities, not of point charges, for exactly the
penetration reason, and reports that three-body terms matter most for water clusters and
solvated ions, which is our carrier.

**The electrostatically embedded many-body expansion — Dahlke and Truhlar.** *J. Chem. Theory
Comput.* 3, 46 (2007); 3, 1342 (2007); 4, 1 (2008): point-charge embedding lowers EE-PA errors
on water clusters by up to a factor of ten against the bare pairwise approximation, and the
embedded three-body expansion reaches mean unsigned errors of 0.05 kcal/mol across nine
levels of theory. This is the charge-field arm of EMBED-1 and SEAM-1, and its numbers are the
water target EMBED-3's System B is priced against in our own basis.

**The residual as three-body dispersion.** Otero-de-la-Roza, LeBlanc and Johnson, "What is
'many-body' dispersion and should I worry about it?", *Phys. Chem. Chem. Phys.* 22, 8266
(2020): the leading ATOMIC many-body dispersion term is Axilrod–Teller–Muto, `R⁻⁹` on a triple,
distinct from the electronic many-body effects the phrase also names. EMBED-2's reading — that
once the electrostatics is exact the far-field residual of an embedded two-body expansion is
the ATM term — is therefore standard physics; what is ours is that it came out of exact solves
as a harvested residual with a measured floor, as a one-constant law, beside the charge-field
residual that partly cancels it.

**Precision in the many-body expansion — Richard, Lao and Herbert.** *J. Chem. Phys.* 141,
014108 (2014), "Understanding the many-body expansion for large systems. I. Precision
considerations": the MBE loses precision through the factorial growth of subsystem terms as a
function of system size and truncation order; parts II (2016) and III (2017) add accuracy,
counterpoise and the necessity of four-body terms for (H₂O)₃₇. Our case is seven terms, not
thousands, and the loss we found was a RECORD's printed precision, not the arithmetic's
(M-FORMAT-FLOOR) — but the shape is theirs: a difference of large terms is only as good as
the least precise term it is built from.

**Compute time.** None of these reports a runtime comparable to ours; none is compared.
