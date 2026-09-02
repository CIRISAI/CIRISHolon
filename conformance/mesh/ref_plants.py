"""LG_PREREG §7 — the three isolating conservation plants.

Each moves exactly ONE conserved quantity, so a plant that fires two gates is
itself a finding. All three act on the SAME carrier, local state 9 (the head-on
pair), which is nonzero in the (N=2, P=0) sector the plants act on.
"""
from lattice_common import lab

SRC = 9

if __name__ == "__main__":
    n0, x0, y0 = lab(SRC)
    print(f"carrier {SRC} -> N={n0} P=({x0},{y0})")
    for name, pred in (
        ("mass only",       lambda t: lab(t)[0] != n0 and lab(t)[1:] == (x0, y0)),
        ("momentum-x only", lambda t: lab(t)[0] == n0 and lab(t)[1] != x0 and lab(t)[2] == y0),
        ("momentum-y only", lambda t: lab(t)[0] == n0 and lab(t)[1] == x0 and lab(t)[2] != y0),
    ):
        tgt = [t for t in range(64) if pred(t)]
        print(f"  {name:16s} targets: {tgt[:6]}")
    for t in (0, 34, 5):
        print(f"  plant target {t}: bits {[k for k in range(6) if t >> k & 1]} -> {lab(t)}")
