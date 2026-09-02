"""LG_PREREG §5.3 — the exact k=1 block-chart closure defect, from geometry alone.

W(b) = 1 - max(0, b-2)^2 / b^2 for b < L, and W(L) = 0. Derived, not fitted: the
collision is a bijection fixing (N,P), streaming moves each particle exactly one
cell, so a cell whose six neighbours all lie in its own block cannot produce a
witness. The enumeration below says that bound is SATURATED.
"""
from ref_defect_allmodels import rate
from lattice_common import fhp_i

if __name__ == "__main__":
    C, L = fhp_i(), 64
    print(f"{'b':>4} {'exact rate k=1':>16} {'geometric bound':>16}")
    for b in (1, 2, 4, 8, 16, 32, 64):
        bound = 1.0 - (max(0, b - 2) ** 2) / b ** 2 if b < L else 0.0
        print(f"{b:>4} {rate(b, L, C):>16.10f} {bound:>16.10f}")
    print("\nL-independence at b=8:", [round(rate(8, n, C), 10) for n in (64, 128, 256)])
