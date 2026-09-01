# THE DRY-RESIDUAL REGISTER

*Opened 2026-09-01 by the closure-census lane, under WB-8.7 (`WORKBENCH_FSD.md` §10):
"every special case, hardcoded branch, or per-composition carve-out the build FORCES us to
write is a measurement against the claim — a witness pair at the architecture level."*

**How to read this file.** It is not a to-do list and it is not a shame ledger. Each row is
a place where the recursive object did NOT fold, recorded with the reason and with the
fold that would absorb it. The register's **growth rate against domain size** is the live
falsifier: short and closing as the domain widens means the fold is winning; growing in
step with the domain means the DRY claim is wrong, said quantitatively.

So every row carries the **domain it was forced by**, and the register is counted per
domain rather than in total. A row with no fold named is not admissible: "we could not
think of one" is itself a finding and must be written as the fold being unknown.

| id | residual | domain that forced it | the fold that would absorb it | why not folded now |
|---|---|---|---|---|
| **R-LENS-DIM** | `q_tetrahedral` and `steinhardt_q` refuse 2D; `hexatic_psi6` refuses 3D. Three lenses where the object has one question: how orientationally ordered is the neighbourhood. | H/O dynamics, 2 dimensions and 3 | Bond-orientational order as harmonics on `S^(d−1)`, with `psi6` the `d = 2` instance of the same construction that gives `q6` at `d = 3`. | Written as two hardcoded cases because the 2D and 3D scenes arrived from different tiers and only the 2D one had data. The fold is standard mathematics, not research; this is a debt, not a wall. |
| **R-SHELL-COUNT** | `want = if dims == 2 { 6 } else { 12 }` in the classifier: the size of a "first shell" is a hardcoded pair of integers. | same | The first coordination number derived from the scene's own radial distribution, not from a table of lattices. | A 12-atom scene has too few atoms for a radial distribution function to have a first minimum. The fold needs the T3 scale-up, which is exactly the axis WB-8.7 wants measured. |
| **R-Z-SYMBOL** | `partition::symbol` maps nuclear charge to `H`/`C`/`N`/`O` and everything else to `X`. A per-species branch, which WB-8.7 §(2) names by name. | H/O census reporting | `holon_chem::elements`, which already holds the periodic table this duplicates. | `holon-lens` is deliberately zero-dependency so that it stays testable while `holon-render` is mid-refactor — and it WAS written during such a window. The isolation bought a real thing; the cost is this table. Fold when the census moves in-tree, or when `elements` is split into a `no_std` leaf the way `holon-device` was. |
| **R-DUMP-CAP** | The trajectory format caps scenes at 16 atoms, because `C(16,2) = 120` pair bits fit a `u128`. | trajectory persistence | A variable-length bitset sized by the scene, as the engine's own pair sector already is. | The engine's scene size stopped being capped at T3 (`DEFAULT_SCENE_ATOMS` replaced `MAX_ATOMS`), so the ARTIFACT now carries a limit the OBJECT does not. That is the register's own shape: a representation constraining a thing that has outgrown it. This one will be forced by the first scene above 16 atoms and should be folded before then. |
| **R-CENTROID-MASS** | `census::carrier_motion` takes a GEOMETRIC centroid where the physics wants a mass-weighted one. | closure census, Leg A | Carry mass in the trajectory header beside `Z`. | The dump carries nuclear charge because that is what the census's composition reading needs. The affected quantity is the split between translation and internal motion; the second motion statistic (the intra-block separation excursion) is centroid-free and settles a frozen carrier on its own, so the residual is bounded rather than load-bearing. Stated in the function's own doc comment as well as here. |

## Count, by domain

| domain | residuals open | closed |
|---|---|---|
| H/O chemical dynamics, 2D and 3D | 5 | 0 |

The register opens at five and nothing is closed yet, which is the honest starting point
rather than a result. The number to watch is what this row does when a SECOND domain
arrives — a third element, a 3D scene, a scene past sixteen atoms. If the count roughly
doubles with the domain, the fold is not winning and WB-8.7's falsifier has fired.

Two of the five (**R-SHELL-COUNT**, **R-DUMP-CAP**) are already known to be forced by the
T3 scale-up, so that work package will move this table one way or the other on its own.
