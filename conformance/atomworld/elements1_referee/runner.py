"""
runner.py -- one species at one geometry, both routes, with the certificate.

Everything here is exact-in-model: the only inputs are the nuclear charges Z
(integers), the nuclear geometry, and the declared STO-3G table.  Energies,
bond lengths, well depths and which pairs bind at all are computed.
"""

import json
import os
import time

from mpmath import mp, mpf, nstr, sqrt

import elements_core as E
import fci as F

DPS = 60
REPORT = 50
CACHE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "cache")


def s(x, n=REPORT):
    return nstr(mpf(x), n, strip_zeros=False)


def _key(tag, dps):
    return "%s__dps%d" % (tag, dps)


def cache_get(tag, dps, table=None):
    """Read a cached record, refusing any computed under a different basis."""
    p = os.path.join(CACHE, _key(tag, dps) + ".json")
    if not os.path.exists(p):
        return None
    try:
        with open(p) as f:
            obj = json.load(f)
    except Exception:
        return None
    want = E.basis_fingerprint(table)
    got = obj.get("basis_fingerprint")
    if got != want:
        cache_get.refused += 1
        return None
    return obj


cache_get.refused = 0


def cache_put(tag, dps, obj, table=None):
    obj["basis_fingerprint"] = E.basis_fingerprint(table)
    os.makedirs(CACHE, exist_ok=True)
    p = os.path.join(CACHE, _key(tag, dps) + ".json")
    tmp = p + ".tmp%d" % os.getpid()
    with open(tmp, "w") as f:
        json.dump(obj, f)
    os.replace(tmp, p)


# ---------------------------------------------------------------------------
def geometry(spec):
    """spec: ('atom', Z) or ('diatomic', Z1, Z2, R).  Returns the atom list."""
    if spec[0] == "atom":
        return [(spec[1], (mpf(0), mpf(0), mpf(0)))]
    _, Z1, Z2, R = spec
    R = mpf(R)
    return [(Z1, (mpf(0), mpf(0), mpf(0))), (Z2, (mpf(0), mpf(0), R))]


def prepare(spec, table=None, screen=None):
    """AO integrals and both MO sets."""
    atoms = geometry(spec)
    mol = E.molecule(atoms, table=table, screen=screen)
    C, sev = F.lowdin_orbitals(mol["S"])
    hA, gA = F.mo_integrals(mol, C)
    Q = F.rotation_matrix(mol["nbf"])
    hB, gB = F.mo_integrals(mol, F.rotate_orbitals(C, Q))
    return mol, (hA, gA), (hB, gB), [float(x) for x in sev]


def solve_sector(mol, ints_A, ints_B, na, nb, want_B=True, max_outer=9,
                 tol_digits=None):
    if tol_digits is None:
        tol_digits = mp.dps - 8
    sp = F.DetSpace(mol["nbf"], na, nb)
    rA = F.solve_certified(F.RouteAOp(sp, *ints_A), tol_digits=tol_digits,
                           max_outer=max_outer)
    out = dict(ndet=sp.ndet, A=rA)
    if want_B:
        rB = F.solve_certified(F.RouteBOp(sp, *ints_B), tol_digits=tol_digits,
                               max_outer=max_outer)
        out["B"] = rB
    return out


ROUTE_C_BUDGET = 4.0e7


def run_point(spec, tag, want_C=True, want_B=True, dps=DPS, table=None,
              force=False, sectors=None, max_outer=9):
    """Compute one geometry.  Returns a JSON-safe dict; caches to disk."""
    cached = None if force else cache_get(tag, dps, table)
    if cached is not None:
        return cached
    mp.dps = dps
    t0 = time.time()
    mol, iA, iB, sev = prepare(spec, table=table)
    t_int = time.time() - t0
    nelec, norb = mol["nelec"], mol["nbf"]

    import species as SP
    if sectors is None:
        sectors = SP.sz_sectors(nelec, norb)
        if spec[0] != "atom":
            sectors = [t for t in sectors if t[0] == (nelec % 2)]

    res = dict(tag=tag, dps=dps, nbf=norb, nelec=nelec,
               E_nuc=s(mol["E_nuc"]), t_integrals=t_int,
               overlap_eigenvalues=sev, sectors={})
    for (two_sz, na, nb) in sectors:
        t1 = time.time()
        r = solve_sector(mol, iA, iB, na, nb, want_B=want_B,
                         max_outer=max_outer)
        EA = r["A"]["energy"] + mol["E_nuc"]
        sp_ = F.DetSpace(mol["nbf"], na, nb)
        vec = r["A"].get("vector")
        if vec is None:
            raise RuntimeError("solver returned no vector; the spin check "
                               "cannot be skipped silently")
        s2 = F.spin_squared(sp_, vec)
        twoS, s2dev = F.spin_from_s2(s2) if s2 is not None else (None, None)
        ent = dict(ndet=r["ndet"], E_A=s(EA),
                   S2_expectation=nstr(s2, 12) if s2 is not None else None,
                   two_S_from_S2=twoS,
                   S2_deviation_from_exact=nstr(s2dev, 6)
                   if s2dev is not None else None,
                   resid_A=nstr(r["A"]["resid"], 6),
                   bound_A=nstr(r["A"]["bound_temple"], 6)
                   if r["A"]["bound_temple"] is not None else None,
                   outer_A=r["A"]["outer"], seed_A=r["A"]["seed"],
                   gap_A=r["A"]["gap"], t_sector=time.time() - t1)
        if want_B:
            EB = r["B"]["energy"] + mol["E_nuc"]
            ent.update(E_B=s(EB), resid_B=nstr(r["B"]["resid"], 6),
                       outer_B=r["B"]["outer"],
                       dev_AB=nstr(abs(EA - EB), 6))
        res["sectors"][str(two_sz)] = ent

    if want_C:
        nd, cost = F.route_c_cost(norb, nelec)
        res["route_C_fock_dim"] = nd
        res["route_C_cost"] = cost
        if cost <= ROUTE_C_BUDGET or want_C == "force":
            t1 = time.time()
            ec, asym, nfock = F.route_c_energy(norb, nelec, *iA)
            EC = ec + mol["E_nuc"]
            best = min(mpf(v["E_A"]) for v in res["sectors"].values())
            res["E_C"] = s(EC)
            res["dev_AC"] = nstr(abs(EC - best), 6)
            res["route_C_asym"] = nstr(asym, 6)
            res["t_route_C"] = time.time() - t1
        else:
            res["E_C"] = None
            res["route_C_skipped"] = "cost %.1e above budget %.1e" % (
                cost, ROUTE_C_BUDGET)
    res["t_total"] = time.time() - t0
    cache_put(tag, dps, res, table)
    return res


def spin_only(spec, tag, dps=DPS, table=None, na=None, nb=None, force=False):
    """<S^2> of the converged route-A vector at one geometry.

    H commutes with S^2, so a subspace method never leaves the spin sector of
    its starting vector: it can converge with a small residual, a tight bound
    and two routes agreeing, onto a spin-excited state, and nothing it reports
    about itself will show it.  This is the one quantity that does, and it has
    to be checked at EVERY geometry rather than at spot ones -- the place it
    would slip is the dissociation tail, where the singlet and the triplet come
    together and the two are separated by less than the grid's own spacing in
    energy.  The engine lane's planted defect fired on F2, a species neither of
    us had spin-tested, not on the carbon case we both knew about.
    """
    cached = None if force else cache_get(tag, dps, table)
    if cached is not None:
        return cached
    mp.dps = dps
    t0 = time.time()
    mol = E.molecule(geometry(spec), table=table)
    C, _ = F.lowdin_orbitals(mol["S"])
    hA, gA = F.mo_integrals(mol, C)
    if na is None:
        nel = mol["nelec"]
        na, nb = (nel + nel % 2) // 2, nel // 2
    sp = F.DetSpace(mol["nbf"], na, nb)
    r = F.solve_certified(F.RouteAOp(sp, hA, gA), tol_digits=dps - 8,
                          max_outer=9)
    vec = r.get("vector")
    if vec is None:
        raise RuntimeError("solver returned no vector; the spin check cannot "
                           "be skipped silently")
    s2 = F.spin_squared(sp, vec)
    twoS, dev = F.spin_from_s2(s2)
    E0 = r["energy"] + mol["E_nuc"]
    lvl = F.ground_level_spins(F.RouteAOp(sp, hA, gA), sp)

    # THE DEFECT TEST, as distinct from the spin READING.
    #
    # A curve changing multiplicity along R is physics, not a bug: for two
    # open-shell atoms the two-centre exchange integral favours the HIGH-spin
    # coupling at long range -- Hund's rule between centres -- while the bonding
    # term favours the singlet at short range, so the two cross.  F2 does this
    # near 4 bohr in this model.  Refusing a curve for changing spin would
    # refuse the physics.
    #
    # What IS a defect is returning a state that is not the lowest in the
    # sector.  The Sz+1 sector contains every multiplet the Sz sector does
    # EXCEPT the lowest one (a spin-S multiplet appears in every sector with
    # |Sz| <= S), so E_min(Sz) <= E_min(Sz+1) must hold.  If the answer sits
    # ABOVE the next sector's minimum, the solver missed something -- and that
    # inequality holds whichever spin happens to win, so it needs no assumption
    # about the answer.
    up = None
    if na + 1 <= mol["nbf"] and nb - 1 >= 0:
        spu = F.DetSpace(mol["nbf"], na + 1, nb - 1)
        ru = F.solve_certified(F.RouteAOp(spu, hA, gA), tol_digits=dps - 8,
                               max_outer=9)
        up = ru["energy"] + mol["E_nuc"]
    obj = dict(tag=tag, dps=dps, ndet=sp.ndet,
               E=nstr(E0, 30), S2=nstr(s2, 20), two_S=twoS,
               S2_dev=nstr(dev, 6),
               level_size=lvl["level_size"],
               level_two_S=lvl["two_S_in_level"],
               level_resolved=lvl["resolved"],
               level_gap_to_next=lvl["gap_to_next"],
               E_next_Sz_sector=nstr(up, 30) if up is not None else None,
               below_next_sector=bool(up is None or E0 <= up + mpf(10) ** -40),
               margin_to_next_sector=nstr(up - E0, 8) if up is not None
               else None,
               t=time.time() - t0)
    cache_put(tag, dps, obj, table)
    return obj


def energy_only(spec, tag, dps=DPS, table=None, na=None, nb=None,
                max_outer=12, force=False):
    """Route A energy at one geometry -- the workhorse for FD stencils."""
    cached = None if force else cache_get(tag, dps, table)
    if cached is not None:
        return mpf(cached["E"]), cached
    mp.dps = dps
    t0 = time.time()
    atoms = geometry(spec)
    mol = E.molecule(atoms, table=table)
    C, _ = F.lowdin_orbitals(mol["S"])
    hA, gA = F.mo_integrals(mol, C)
    if na is None:
        nelec = mol["nelec"]
        na, nb = (nelec + nelec % 2) // 2, nelec // 2
    sp = F.DetSpace(mol["nbf"], na, nb)
    r = F.solve_certified(F.RouteAOp(sp, hA, gA), tol_digits=dps - 8,
                          max_outer=max_outer)
    Etot = r["energy"] + mol["E_nuc"]
    obj = dict(tag=tag, dps=dps, E=nstr(Etot, dps - 2, strip_zeros=False),
               resid=nstr(r["resid"], 6), outer=r["outer"],
               t=time.time() - t0)
    cache_put(tag, dps, obj, table)
    return mpf(obj["E"]), obj
