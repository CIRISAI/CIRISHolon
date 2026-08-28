#!/usr/bin/env python3
"""selector_census.py — SELECTOR-MAP: exact census of candidate Ω-internal
selection principles on the TOE-NULL-1 model (6 states, 3 fibers, 48 closed
reversible dynamics).  Self-contained, exact (Fraction arithmetic), verifier
contract: prints a census and exits NONZERO on any internal inconsistency.

The question this instrument serves: TOE-NULL-1 established that Ω SEPARATES
the presented worlds but does not SELECT among them, and that any monotone
rent-minimizing selector picks the identity (the stasis theorem).  Every
candidate selector below is therefore scored on two axes at once: what it
selects, and whether it excludes the identity BY CRITERION.

Candidates computed here:
  S2  descent self-similarity, in FOUR formalizations (section / order /
      full-lattice / spectral), to expose the trivially-empty and
      trivially-satisfied failure modes by computation rather than by
      assertion.
  S3  observer bootstrap, over two act vocabularies (view-aligned and
      section-generated), separation decided EXACTLY by pair-BFS over all
      experiment words (no depth cut).
  S4  capacity-per-rent extremization, with an ACTIVITY-QUALIFIED capacity.
  S6  genericity / minimal accidental closure.
  S5  the no-selector null, given teeth: the exact resolving-power ceiling of
      ANY Ω-internal selector, computed as the orbit count of the view
      automorphism group acting by conjugation.

Credit lineage (house rule: convergence is a hit, not a strike): Wilson /
Kadanoff / Wegner (RG fixed points — S2's ancestor); Barnett & Seth
(dynamical independence — the descend operation); Kolchinsky & Wolpert
(semantic information — S1/S4's ancestor); Montévil & Mossio (closure of
constraints); Deutsch & Marletto (constructor theory — S3's act vocabulary);
Smolin (cosmological natural selection — the population face S1 lacks);
Wheeler (law without law — S3's participatory loop); Shalizi & Crutchfield
(causal states).  See conformance/omega/PRIOR_ART_CONVERGENCE.md.
"""
import sys
from fractions import Fraction as F
from itertools import permutations, product

FAIL = []


def check(cond, msg):
    if not cond:
        FAIL.append(msg)
    return cond


# ---------------------------------------------------------------- the model
S = list(range(6))
FIB = {0: 0, 1: 0, 2: 1, 3: 1, 4: 2, 5: 2}
mu = F(1, 6)


def all_partitions(seq):
    if not seq:
        yield []
    else:
        first, rest = seq[0], seq[1:]
        for part in all_partitions(rest):
            for i in range(len(part)):
                yield part[:i] + [[first] + part[i]] + part[i + 1:]
            yield [[first]] + part


PARTS = sorted({tuple(tuple(sorted(b)) for b in sorted(map(sorted, pt)))
                for pt in all_partitions(S)})
LAB = [{s: bi for bi, b in enumerate(pt) for s in b} for pt in PARTS]


def rent_gen(p, lab, nblocks, n):
    """Ω rent of dynamics p at the view with labelling lab: the mass the best
    deterministic coarse map fails to carry.  Uniform measure 1/n."""
    m1 = F(1, n)
    P = {}
    for s in range(n):
        k = (lab[s], lab[p[s]])
        P[k] = P.get(k, F(0)) + m1
    return 1 - sum(max(P.get((i, j), F(0)) for j in range(nblocks))
                   for i in range(nblocks))


def rent(p, pi):
    return rent_gen(p, LAB[pi], len(PARTS[pi]), 6)


def closed(p):
    img = {}
    for s in S:
        i, j = FIB[s], FIB[p[s]]
        if img.setdefault(i, j) != j:
            return False
    return len(set(img.values())) == 3


def macro_map(p):
    return {i: FIB[p[2 * i]] for i in range(3)}


def perm_order(sig, dom):
    x, k = tuple(dom), 0
    while True:
        x = tuple(sig[i] for i in x)
        k += 1
        if x == tuple(dom):
            return k


def macro_order(p):
    return perm_order(macro_map(p), range(3))


# CP in toe_null1.py's own enumeration order, so the representatives match.
CP = [p for p in permutations(S) if closed(p)]
IDP = tuple(S)
census = {}
for p in CP:
    census[macro_order(p)] = census.get(macro_order(p), 0) + 1
check(len(CP) == 48, "closed-dynamics count != 48")
check(census == {1: 8, 2: 24, 3: 16}, "macro census != 8/24/16")

# ------------------------------------------------- (m, c) coordinates of T
# state 2i+b;  T(i,b) = (m(i), b XOR c_i).  c is the fiber-internal cochain;
# its Z2 holonomy around each cycle of m is the obstruction S2 measures.
def mc(p):
    m = macro_map(p)
    c = tuple((p[2 * i] & 1) for i in range(3))
    for i in range(3):                      # the (m, c) form must be exact
        for b in (0, 1):
            check(p[2 * i + b] == 2 * m[i] + (b ^ c[i]), "(m,c) coordinates fail")
    return m, c


def cycles(m):
    seen, cyc = set(), []
    for i in range(3):
        if i in seen:
            continue
        cy, j = [], i
        while j not in seen:
            seen.add(j)
            cy.append(j)
            j = m[j]
        cyc.append(cy)
    return cyc


def holonomy(p):
    """The Z2 holonomy of c around each cycle of the macro map, sorted by
    cycle length — the gauge-invariant content of the fiber-internal data."""
    m, c = mc(p)
    return tuple(sorted((len(cy), sum(c[i] for i in cy) % 2) for cy in cyc_of(m)))


def cyc_of(m):
    return cycles(m)


# ------------------------------------------------- the view-lattice spectra
SPEC, RTOT, NZ, CLOSEDV, ORD = {}, {}, {}, {}, {}
for p in CP:
    sp = [rent(p, pi) for pi in range(len(PARTS))]
    SPEC[p] = tuple(sp)
    RTOT[p] = sum(sp)
    NZ[p] = sum(1 for r in sp if r != 0)
    CLOSEDV[p] = [pi for pi, r in enumerate(sp) if r == 0]
    ORD[p] = perm_order({s: p[s] for s in S}, S)

reps = {o: next(p for p in CP if macro_order(p) == o) for o in (1, 2, 3)}
check((RTOT[reps[1]], RTOT[reps[2]], RTOT[reps[3]]) == (0, 50, 63),
      "TOE-NULL-1 rep spectra not reproduced")
check((NZ[reps[1]], NZ[reps[2]], NZ[reps[3]]) == (0, 172, 195),
      "TOE-NULL-1 rep nonzero counts not reproduced")
check(len(PARTS) == 203, "view lattice != 203")

# the stasis theorem instance, re-verified in-instrument (SELECTOR-1 gate N0)
zero_everywhere = [p for p in permutations(S)
                   if all(rent(p, pi) == 0 for pi in range(len(PARTS)))]
check(zero_everywhere == [IDP], "stasis theorem instance failed")


# ------------------------------------------ induced coarse map at a closed view
def induced(p, pi):
    lab = LAB[pi]
    return {lab[s]: lab[p[s]] for s in S}


# ================================================================== S2 family
# S2-A  SECTION form: a T-invariant transversal of the fibers exists — the
#       macro law is not merely induced, it is IMPLEMENTED by a micro
#       subsystem (one state per fiber, dynamically closed).
def sections(p):
    out = []
    for x in product((0, 1), repeat=3):
        X = frozenset(2 * i + x[i] for i in range(3))
        if {p[s] for s in X} == set(X):
            out.append(X)
    return out


S2A = {p: len(sections(p)) > 0 for p in CP}

# S2-B  ORDER form: the descended law has the same period as the law.
S2B = {p: ORD[p] == macro_order(p) for p in CP}

# S2-A is the vanishing of the Z2 holonomy on EVERY macro cycle.
S2H = {p: all(h == 0 for _, h in holonomy(p)) for p in CP}
check(all(S2A[p] == S2H[p] for p in CP),
      "S2-A is not the trivial-holonomy condition")

# S2-B is the vanishing of the holonomy WEIGHTED by n/len(cycle): a cycle
# shorter than the macro period traverses its holonomy an even number of
# times per macro period, so its flip hides inside the period.  Hence
# S2-A is STRICTLY STRONGER than S2-B — the two natural formalizations of
# "the law survives its own coarse-graining" are NOT the same criterion.
def s2b_pred(p):
    m, c = mc(p)
    n = macro_order(p)
    return all(((n // len(cy)) * sum(c[i] for i in cy)) % 2 == 0
               for cy in cyc_of(m))
check(all(S2B[p] == s2b_pred(p) for p in CP),
      "S2-B is not the weighted-holonomy condition")
check(all((not S2A[p]) or S2B[p] for p in CP), "S2-A is not contained in S2-B")
check(sum(S2A.values()) < sum(S2B.values()), "S2-A/S2-B containment is not strict")

# S2-C  FULL-LATTICE form: every rent-zero view with >=2 blocks preserves the
#       period.  (Anti-selective by a general argument: the orbit partition of
#       any permutation is invariant and descends to the identity.)
def s2c(p):
    for pi in CLOSEDV[p]:
        if len(PARTS[pi]) < 2:
            continue
        if perm_order(induced(p, pi), range(len(PARTS[pi]))) != ORD[p]:
            return False
    return True


S2C = {p: s2c(p) for p in CP}
check([p for p in CP if S2C[p]] == [IDP],
      "S2-C does not select exactly the identity")

# S2-F  SPECTRAL form: the rent profile on views COARSER than the fibers
#       equals the descendant's own rent profile.  Tautological by the closure
#       square — computed here so the trap is a measurement, not a claim.
MPARTS = sorted({tuple(tuple(sorted(b)) for b in sorted(map(sorted, pt)))
                 for pt in all_partitions([0, 1, 2])})
COARSE = []            # (index into PARTS, index into MPARTS)
for mi, mpt in enumerate(MPARTS):
    blocks = [tuple(sorted(s for i in b for s in (2 * i, 2 * i + 1))) for b in mpt]
    key = tuple(sorted(blocks))
    COARSE.append((PARTS.index(key), mi))
check(len(COARSE) == 5, "coarse sub-lattice != 5 views")

S2F = {}
for p in CP:
    m = macro_map(p)
    ok = True
    for pi, mi in COARSE:
        lab = {i: bi for bi, b in enumerate(MPARTS[mi]) for i in b}
        rm = rent_gen([m[i] for i in range(3)], lab, len(MPARTS[mi]), 3)
        if rent(p, pi) != rm:
            ok = False
    S2F[p] = ok
check(all(S2F.values()), "S2-F is not tautological — coarse rents differ")


# ================================================================== S3 family
def knobs_view_aligned(p):
    """Acts generated by T on Ω-OBSERVABLE subsystems (unions of view blocks).
    Every such act is view-covariant, hence gauge-safe by Break.lean's T3."""
    out = []
    for B in product((0, 1), repeat=3):
        out.append(tuple(p[s] if B[FIB[s]] else s for s in S))
    return out


def knobs_sections(p):
    """Acts generated by T on its own invariant SECTIONS — definable from
    (view, T) alone, but not view-addressable.  These are the only internally
    generated acts that fail to descend to the view."""
    return [tuple(p[s] if s in X else s for s in S) for X in sections(p)]


def separates(home_knobs, p, q):
    """EXACT over all experiment words: BFS on state pairs.  Letters are
    `step` (p on the left, q on the right) and each fixed knob (same map both
    sides).  Separated iff a reachable pair is split by the view."""
    seen = set((s, s) for s in S)
    stack = list(seen)
    while stack:
        x, y = stack.pop()
        if FIB[x] != FIB[y]:
            return True
        for nxt in [(p[x], q[y])] + [(a[x], a[y]) for a in home_knobs]:
            if nxt not in seen:
                seen.add(nxt)
                stack.append(nxt)
    return False


IDENT_V1, IDENT_V2 = {}, {}
for p in CP:
    k1, k2 = knobs_view_aligned(p), knobs_view_aligned(p) + knobs_sections(p)
    IDENT_V1[p] = [q for q in CP if not separates(k1, p, q)]
    IDENT_V2[p] = [q for q in CP if not separates(k2, p, q)]

check(all(len(IDENT_V1[p]) == 8 for p in CP),
      "view-aligned acts separate something they provably cannot")
check(all(p in IDENT_V1[p] and p in IDENT_V2[p] for p in CP),
      "separation is not reflexive-safe")

S3_V1 = {p: len(IDENT_V1[p]) == 1 for p in CP}
S3_V2 = {p: len(IDENT_V2[p]) == 1 for p in CP}
check(not S3_V2[IDP], "S3 fails to exclude the identity")


# ================================================================== S4 and S6
# Capacity, ACTIVITY-QUALIFIED: a view counts as a level only if it is both
# autonomous (rent zero) and active (its induced map is not the identity).
def cap(p, drop_discrete):
    n = 0
    for pi in CLOSEDV[p]:
        if drop_discrete and len(PARTS[pi]) == 6:
            continue
        ind = induced(p, pi)
        if any(ind[b] != b for b in range(len(PARTS[pi]))):
            n += 1
    return n


CAPE = {p: cap(p, True) for p in CP}          # emergent levels (no discrete view)
CAPA = {p: cap(p, False) for p in CP}         # all active closed views
NCL = {p: len(CLOSEDV[p]) for p in CP}

check(CAPE[IDP] == 0 and CAPA[IDP] == 0, "identity has nonzero active capacity")
check(RTOT[IDP] == 0 and NCL[IDP] == 203, "identity is not the stasis point")
check(all(RTOT[p] > 0 for p in CP if p != IDP), "a non-identity dynamics has zero rent")
check(max(NCL.values()) == NCL[IDP], "identity is not the closure argmax")

PHI = {p: F(CAPE[p], 1) / RTOT[p] for p in CP if p != IDP}


# ============================================ S5: the resolving-power ceiling
# The view automorphism group is exactly the set of closed dynamics (order 48,
# Z2 wr S3).  Any Ω-internal functional is invariant under its conjugation
# action, so the orbit count is a HARD CEILING on any selector's resolution.
def conj(g, p):
    gi = [0] * 6
    for s in S:
        gi[g[s]] = s
    return tuple(g[p[gi[s]]] for s in S)


orbit_of, ORBITS = {}, []
for p in CP:
    if p in orbit_of:
        continue
    orb = frozenset(conj(g, p) for g in CP)
    ORBITS.append(orb)
    for q in orb:
        orbit_of[q] = len(ORBITS) - 1
check(sum(len(o) for o in ORBITS) == 48, "orbits do not partition the 48")
check(len(ORBITS) == 10, f"orbit count is {len(ORBITS)}, expected 10")
check(sorted(len(o) for o in ORBITS) == [1, 1, 3, 3, 6, 6, 6, 6, 8, 8],
      "orbit sizes are not the Z2 wr S3 class sizes")
check(len([o for o in ORBITS if IDP in o][0]) == 1, "the identity is not its own orbit")

# every Ω-internal statistic MUST be constant on orbits — the gauge check that
# certifies each candidate really is Ω-internal.
for name, stat in (("rent spectrum", SPEC), ("total rent", RTOT),
                   ("nonzero views", NZ), ("closed views", NCL),
                   ("capacity", CAPE), ("order", ORD), ("S2-A", S2A),
                   ("S2-C", S2C), ("S3-V2", S3_V2)):
    for orb in ORBITS:
        vals = {stat[q] if name != "rent spectrum" else tuple(sorted(stat[q]))
                for q in orb}
        check(len(vals) == 1, f"{name} is not orbit-invariant — not Ω-internal")

# is the rent spectrum a COMPLETE invariant for the orbits?
spec_classes = len({tuple(sorted(SPEC[p])) for p in CP})
tot_classes = len({RTOT[p] for p in CP})

# view-INDEXED spectrum up to the gauge group: p ~ q iff some conjugate of p
# has the identical view-by-view profile.  Strictly finer than the multiset.
prof_cls, seen_p = [], set()
for p in CP:
    if p in seen_p:
        continue
    cl = {q for q in CP if any(SPEC[conj(g, p)] == SPEC[q] for g in CP)}
    prof_cls.append(cl)
    seen_p |= cl
check(sum(len(c) for c in prof_cls) == 48, "profile classes do not partition")

# the multiset collisions, named
mset = {}
for k, orb in enumerate(ORBITS):
    mset.setdefault(tuple(sorted(SPEC[sorted(orb)[0]])), []).append(k)
COLLIDE = [v for v in mset.values() if len(v) > 1]


# ==================================================================== report
def by_class(pred):
    return tuple(sum(1 for p in CP if macro_order(p) == o and pred(p))
                 for o in (1, 2, 3))


print("=" * 78)
print("SELECTOR CENSUS — TOE-NULL-1 model, 48 closed dynamics, 203-view lattice")
print("=" * 78)
print(f"macro-world census (period 1/2/3): {census[1]}/{census[2]}/{census[3]}")
print(f"rep spectra reproduced: {RTOT[reps[1]]}/{RTOT[reps[2]]}/{RTOT[reps[3]]}"
      f"  nonzero {NZ[reps[1]]}/{NZ[reps[2]]}/{NZ[reps[3]]}")
print(f"stasis theorem: identity is the unique lattice-wide zero among 720  OK")
print()
print("-- S2  descent self-similarity ------------------------------------------")
print(f"S2-A section form   passes {sum(S2A.values()):2d}/48   by period {by_class(lambda p: S2A[p])}"
      f"   identity passes: {S2A[IDP]}")
print(f"S2-B order form     passes {sum(S2B.values()):2d}/48   by period {by_class(lambda p: S2B[p])}"
      f"   identity passes: {S2B[IDP]}")
print(f"    S2-A = trivial Z2 holonomy on EVERY macro cycle (checked)")
print(f"    S2-B = trivial holonomy WEIGHTED by period/cycle-length (checked):"
      f" strictly weaker,")
print(f"    the {sum(S2B.values()) - sum(S2A.values())} extra passers are short-cycle flips that hide inside the macro period.")
print(f"S2-C lattice form   passes {sum(S2C.values()):2d}/48   = {{identity}} exactly -> ANTI-SELECTOR")
print(f"S2-F spectral form  passes {sum(S2F.values()):2d}/48   tautological -> VACUOUS")
print(f"S2-A + activity (descend(T) != id): passes "
      f"{sum(1 for p in CP if S2A[p] and macro_order(p) > 1):2d}/48"
      f"   by period {by_class(lambda p: S2A[p] and macro_order(p) > 1)}")
print()
print("-- S3  observer bootstrap -----------------------------------------------")
print(f"V1 view-aligned acts:  |Ident| = 8 for all 48 (the macro class) -> bootstrap "
      f"passes {sum(S3_V1.values())}/48")
print(f"V2 section acts added: bootstrap passes {sum(S3_V2.values()):2d}/48"
      f"   by period {by_class(lambda p: S3_V2[p])}   identity passes: {S3_V2[IDP]}")
print(f"    |Ident_V2| distribution: "
      f"{sorted({len(IDENT_V2[p]) for p in CP})}")
print(f"    S3-V2 == S2-A-and-active ? "
      f"{all(S3_V2[p] == (S2A[p] and macro_order(p) > 1) for p in CP)}")
print()
print("-- S4  capacity per rent ------------------------------------------------")
best = max(PHI.values())
argmax = [p for p in CP if p != IDP and PHI[p] == best]
print(f"capacity(identity) = 0 -> identity EXCLUDED BY CRITERION")
print(f"capacity range over the 47 live worlds: {min(CAPE[p] for p in CP if p != IDP)}"
      f"..{max(CAPE.values())}")
print(f"argmax Phi = Cap/Rent : Phi = {best} ({float(best):.4f}), "
      f"{len(argmax)} dynamics, periods {sorted({macro_order(p) for p in argmax})}")
for o in (1, 2, 3):
    vals = [PHI[p] for p in CP if p != IDP and macro_order(p) == o]
    if vals:
        print(f"    period {o}: Phi in [{float(min(vals)):.4f}, {float(max(vals)):.4f}]"
              f"   rent in [{min(RTOT[p] for p in CP if p!=IDP and macro_order(p)==o)},"
              f" {max(RTOT[p] for p in CP if macro_order(p)==o)}]"
              f"   cap in [{min(CAPE[p] for p in CP if p!=IDP and macro_order(p)==o)},"
              f" {max(CAPE[p] for p in CP if macro_order(p)==o)}]")
print(f"    argmax orbits: {sorted({orbit_of[p] for p in argmax})}"
      f"   S2-A holds on argmax: {sorted({S2A[p] for p in argmax})}")

# S4 is only a genuine BALANCE if the rent denominator changes the argmax.
# Swept over defensible numerators and denominators; the answer is a finding,
# not a gate — but the identity must be excluded in EVERY variant.
S4VAR = {
    "Cap_emergent / total rent      ": lambda q: F(CAPE[q], 1) / RTOT[q],
    "Cap_active   / total rent      ": lambda q: F(CAPA[q], 1) / RTOT[q],
    "Cap_emergent / nonzero-view cnt": lambda q: F(CAPE[q], NZ[q]),
    "Cap_active   / nonzero-view cnt": lambda q: F(CAPA[q], NZ[q]),
    "Cap_emergent / mean rent/view  ": lambda q: F(CAPE[q], 1) / (RTOT[q] / NZ[q]),
    "Cap_emergent / (rent x levels) ": lambda q: F(CAPE[q], 1) / (RTOT[q] * NCL[q]),
    "Cap_emergent ALONE (no rent)   ": lambda q: F(CAPE[q], 1),
}
LIVE = [q for q in CP if q != IDP]
print("    definitional sensitivity of the argmax:")
S4ARG = {}
for nm, fn in S4VAR.items():
    b = max(fn(q) for q in LIVE)
    S4ARG[nm] = frozenset(orbit_of[q] for q in LIVE if fn(q) == b)
    print(f"      {nm} -> orbits {sorted(S4ARG[nm])}")
    check(CAPE[IDP] == 0, "identity not excluded by capacity in a S4 variant")
noRent = S4ARG["Cap_emergent ALONE (no rent)   "]
degen = all(v == noRent for v in S4ARG.values())
print(f"    every rent-bearing variant agrees with capacity ALONE: {degen}"
      f"  -> the rent denominator does NOT move the argmax on this model:")
print(f"       S4 DEGENERATES TO CAPACITY MAXIMIZATION here; the 'critical balance'"
      f" it was posed to test is NOT exercised by this model family.")
print()
print("-- S6  genericity / minimal accidental closure --------------------------")
lo = min(NCL.values())
argmin = [p for p in CP if NCL[p] == lo]
print(f"closed-view count range {lo}..{max(NCL.values())} (identity = {NCL[IDP]}, the argmax)")
print(f"argmin |closed views| = {lo}: {len(argmin)} dynamics, "
      f"periods {sorted({macro_order(p) for p in argmin})}, "
      f"orders {sorted({ORD[p] for p in argmin})}, S2-A {sorted({S2A[p] for p in argmin})}")
print()
print("-- S5  the ceiling on ANY Ω-internal selector ---------------------------")
print(f"view automorphism group = the 48 closed maps (Z2 wr S3); conjugation orbits = "
      f"{len(ORBITS)}")
print(f"orbit sizes: {sorted(len(o) for o in ORBITS)}")
print(f"NO Ω-internal selector can resolve finer than {len(ORBITS)} classes; unique "
      f"selection is impossible on this model (orbits of size > 1 exist).")
print(f"rent spectrum, SORTED MULTISET: {spec_classes} distinct vs {len(ORBITS)} orbits"
      f"  -> {'complete' if spec_classes == len(ORBITS) else 'INCOMPLETE'}")
for cl in COLLIDE:
    ps = [sorted(ORBITS[k])[0] for k in cl]
    print(f"    COLLISION: orbits {cl}, macro periods "
          f"{[macro_order(q) for q in ps]}, orders {[ORD[q] for q in ps]} "
          f"— same rent multiset, different world")
print(f"rent spectrum, VIEW-INDEXED up to gauge: {len(prof_cls)} classes vs "
      f"{len(ORBITS)} orbits -> "
      f"{'COMPLETE' if len(prof_cls) == len(ORBITS) else 'INCOMPLETE'}")
print(f"total rent alone: {tot_classes} distinct values")
print()
print(f"TOE-NULL-1's three designated witnesses sit in orbits "
      f"{ {o: orbit_of[reps[o]] for o in (1, 2, 3)} } — its period-2 witness is in "
      f"orbit {orbit_of[reps[2]]}, one half of a multiset collision.")
print()
print("-- per-orbit table ------------------------------------------------------")
print(f"{'orb':>3} {'size':>4} {'period':>6} {'ord':>4} {'rent':>7} {'nz':>4} "
      f"{'closed':>6} {'cap':>4} {'Phi':>8} {'S2A':>4} {'S2B':>4} {'S3':>3} "
      f"{'|Id|':>4} {'holonomy (len,h)':>22}")
for k, orb in enumerate(ORBITS):
    p = sorted(orb)[0]
    print(f"{k:>3} {len(orb):>4} {macro_order(p):>6} {ORD[p]:>4} {str(RTOT[p]):>7} "
          f"{NZ[p]:>4} {NCL[p]:>6} {CAPE[p]:>4} "
          f"{(('%.4f' % float(PHI[p])) if p != IDP else 'n/a'):>8} "
          f"{'Y' if S2A[p] else '.':>4} {'Y' if S2B[p] else '.':>4} "
          f"{'Y' if S3_V2[p] else '.':>3} {len(IDENT_V2[sorted(orb)[0]]):>4} "
          f"{str(holonomy(p)):>22}")
print()

SEL_S3 = {p for p in CP if S3_V2[p]}
SEL_S4 = set(argmax)
SEL_S6 = set(argmin)
SEL_S2 = {p for p in CP if S2A[p] and macro_order(p) > 1}
print("-- disagreement among the live candidates -------------------------------")
for na, a in (("S2-A+active", SEL_S2), ("S3", SEL_S3), ("S4", SEL_S4), ("S6", SEL_S6)):
    print(f"    {na:>12}: {len(a):>2} dynamics, orbits {sorted({orbit_of[p] for p in a})}, "
          f"periods {sorted({macro_order(p) for p in a})}")
print(f"    S3 subset of S2-A+active: {SEL_S3 <= SEL_S2}")
print(f"    S4 disjoint from S3: {not (SEL_S4 & SEL_S3)}   S4 disjoint from S6: "
      f"{not (SEL_S4 & SEL_S6)}   S3 disjoint from S6: {not (SEL_S3 & SEL_S6)}")
print(f"    S6 disjoint from S2-A+active: {not (SEL_S6 & SEL_S2)}")
print(f"    union of all four covers {len(SEL_S2 | SEL_S3 | SEL_S4 | SEL_S6)}/48; "
      f"intersection {len(SEL_S2 & SEL_S3 & SEL_S4 & SEL_S6)}/48")
print()

if FAIL:
    print("INTERNAL INCONSISTENCY:")
    for f in FAIL:
        print("  -", f)
    sys.exit(1)
print("ALL CHECKS PASS")
