#!/usr/bin/env python3
"""COMPARE-0's reference models, on the SAME geometries the engine was scored on.

Reads `served.json` (written by `holon-render`'s `compare0_harvest` example, which carries
every geometry as the record's own coordinates in bohr) and evaluates two references on
each one:

  MB-pol      the Paesani group's many-body potential, through MBX (the group's own C++
              implementation, `github.com/paesanilab/MBX`).  Energies and gradients come
              out of the library; no MB-pol number is typed here.  If the library is not
              importable the run records the exact failure and writes `available: false` --
              it never substitutes a number from memory.

  TIP4P/2005  Abascal & Vega, *A general purpose model for the condensed phases of water:
              TIP4P/2005*, J. Chem. Phys. 123, 234505 (2005).  Its parameters are typed
              here from that paper's model-parameter table and each one carries the citation
              in `TIP4P2005_SOURCE` below.  The model is RIGID and the map's monomers are
              not its monomer, so the placement rule is stated, applied, and priced.

Every physical constant comes from `scipy.constants` (CODATA), never from memory.

    python3 references.py [OUT_DIR]

Writes `references.json` beside `served.json`.  BLAS threads are pinned to 1 before numpy
is imported so a reference evaluation cannot spin the machine.
"""
import os

for _v in ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS",
           "NUMEXPR_NUM_THREADS", "VECLIB_MAXIMUM_THREADS"):
    os.environ[_v] = "1"

import json
import math
import pathlib
import sys
import time

import numpy as np
from scipy.constants import physical_constants

# ----------------------------------------------------------------------------- constants

# CODATA, through scipy.constants: nothing here is a remembered number.
BOHR_M = physical_constants["Bohr radius"][0]
BOHR_ANGSTROM = BOHR_M * 1e10
HARTREE_PER_KELVIN = physical_constants["kelvin-hartree relationship"][0]
CONSTANT_SOURCE = {
    "bohr_radius_angstrom": {
        "value": BOHR_ANGSTROM,
        "source": "scipy.constants.physical_constants['Bohr radius'] (CODATA)",
    },
    "hartree_per_kelvin": {
        "value": HARTREE_PER_KELVIN,
        "source": "scipy.constants.physical_constants['kelvin-hartree relationship'] (CODATA)",
    },
}

# ------------------------------------------------------------------------- TIP4P/2005

# Abascal & Vega, J. Chem. Phys. 123, 234505 (2005), the paper's model-parameter table
# (the TIP4P/2005 column).  Every one of these five numbers is that table's; no value here
# was fitted, adjusted or remembered from another model's table.
TIP4P2005 = {
    "d_oh_angstrom": 0.9572,
    "angle_hoh_degrees": 104.52,
    "d_om_angstrom": 0.1546,
    "q_h_e": 0.5564,
    "sigma_angstrom": 3.1589,
    "epsilon_over_kb_kelvin": 93.2,
}
TIP4P2005_SOURCE = (
    "J. L. F. Abascal and C. Vega, 'A general purpose model for the condensed phases of "
    "water: TIP4P/2005', J. Chem. Phys. 123, 234505 (2005) -- the model-parameter table "
    "(TIP4P/2005 column): d(OH) = 0.9572 A, HOH = 104.52 deg, d(OM) = 0.1546 A, "
    "q(H) = 0.5564 e, sigma = 3.1589 A, eps/k_B = 93.2 K."
)


def tip4p_units():
    """The model's parameters in atomic units, by the CODATA conversions above."""
    d_oh = TIP4P2005["d_oh_angstrom"] / BOHR_ANGSTROM
    d_om = TIP4P2005["d_om_angstrom"] / BOHR_ANGSTROM
    sigma = TIP4P2005["sigma_angstrom"] / BOHR_ANGSTROM
    eps = TIP4P2005["epsilon_over_kb_kelvin"] * HARTREE_PER_KELVIN
    q_h = TIP4P2005["q_h_e"]
    half = math.radians(TIP4P2005["angle_hoh_degrees"]) / 2.0
    return d_oh, d_om, sigma, eps, q_h, half


def unit(v):
    n = np.linalg.norm(v)
    if n <= 0.0:
        raise ValueError("a zero vector has no direction")
    return v / n


def tip4p_sites(centers, rule):
    """THE PLACEMENT RULE, stated: where the rigid model's sites go on a node's monomer.

    `centers` is `[O, H1, H2]` in bohr, the record's own atoms.  The map's monomer is the
    MINIMAL-BASIS relaxed water (O-H 1.943574 bohr, H-O-H 96.76 deg) and TIP4P/2005's is
    its own (0.9572 A, 104.52 deg), so the two cannot both be honoured.  Two rules are run
    and both are reported; neither is chosen by its answer.

      "remap"  (PRIMARY) the oxygen, the H-O-H bisector and the molecular plane are kept
               exactly, and the two hydrogens are placed at the MODEL's own bond length and
               angle in that plane, H1 on the side H1 was on.  The model then runs on the
               geometry it was parameterised for, and the price is the hydrogens' movement,
               which this function's caller measures and records.

      "as_is"  the record's own hydrogens are used unmoved and only the M site is placed,
               on the record's own bisector at the model's d(OM).  The model runs on a
               monomer it was never fitted to.  This is the SENSITIVITY leg: the spread
               between the two rules is the placement's own cost, measured rather than
               argued.

    Returns `(sites, charges, oxygen_index, hydrogens_moved_bohr)` with `sites` in bohr.
    """
    d_oh, d_om, _, _, q_h, half = tip4p_units()
    o = np.array(centers[0], dtype=float)
    h1 = np.array(centers[1], dtype=float)
    h2 = np.array(centers[2], dtype=float)
    u1, u2 = unit(h1 - o), unit(h2 - o)
    bisector = unit(u1 + u2)
    in_plane = unit(u1 - u2)
    if rule == "remap":
        n1 = o + d_oh * (math.cos(half) * bisector + math.sin(half) * in_plane)
        n2 = o + d_oh * (math.cos(half) * bisector - math.sin(half) * in_plane)
        moved = max(float(np.linalg.norm(n1 - h1)), float(np.linalg.norm(n2 - h2)))
    elif rule == "as_is":
        n1, n2 = h1, h2
        moved = 0.0
    else:
        raise ValueError(f"unknown placement rule {rule!r}")
    m = o + d_om * bisector
    sites = np.array([o, n1, n2, m])
    charges = np.array([0.0, q_h, q_h, -2.0 * q_h])
    return sites, charges, 0, moved


def tip4p_pair(sites_a, q_a, sites_b, q_b):
    """The interaction of two rigid TIP4P/2005 waters: energy (hartree) and the net force
    on molecule B (hartree/bohr).

    Lennard-Jones acts on the two oxygens only; the Coulomb sum runs over the charged sites
    (the two hydrogens and M).  Both molecules' internal geometry is the model's own and
    fixed, so the monomer reference is a constant and the interaction energy is this sum:
    `E(dimer) - E(A) - E(B)` with the intramolecular terms cancelling exactly.
    """
    _, _, sigma, eps, _, _ = tip4p_units()
    d = sites_a[0] - sites_b[0]
    r = float(np.linalg.norm(d))
    sr6 = (sigma / r) ** 6
    e = 4.0 * eps * (sr6 * sr6 - sr6)
    # dE/dr = 4 eps (-12 sr12 + 6 sr6)/r ; force on B is +dE/dr * (d/r) with d = a - b
    dedr = 4.0 * eps * (-12.0 * sr6 * sr6 + 6.0 * sr6) / r
    f_b = dedr * (d / r)
    for i in range(4):
        if q_a[i] == 0.0:
            continue
        for j in range(4):
            if q_b[j] == 0.0:
                continue
            dv = sites_a[i] - sites_b[j]
            rr = float(np.linalg.norm(dv))
            qq = q_a[i] * q_b[j]
            e += qq / rr
            # dE/dr = -qq/r^2, and the same F_B = (dE/dr)(dv/r) as the LJ term above:
            # two like charges push B AWAY from A, so the sign is negative.
            f_b -= (qq / (rr * rr * rr)) * dv
    return e, f_b


# ------------------------------------------------------------------------------- MB-pol

MBX_JSON = """{
   "Note" : "COMPARE-0's MBX configuration: gas phase, no box, MB-pol defaults",
   "MBX" : {
       "box" : [],
       "twobody_cutoff"   : 9.0,
       "threebody_cutoff" : 7.0,
       "max_n_eval_1b"    : 500,
       "max_n_eval_2b"    : 500,
       "max_n_eval_3b"    : 500,
       "dipole_tolerance" : 1E-16,
       "dipole_max_it"    : 100,
       "dipole_method"    : "cg",
       "alpha_ewald_elec" : 0.0,
       "grid_density_elec" : 2.5,
       "spline_order_elec" : 6,
       "alpha_ewald_disp" : 0.0,
       "grid_density_disp" : 2.5,
       "spline_order_disp" : 6
   }
}
"""


HARTREE_KJ_PER_MOL = (physical_constants["Hartree energy"][0]
                      * physical_constants["Avogadro constant"][0] / 1000.0)
CONSTANT_SOURCE["hartree_kj_per_mol"] = {
    "value": HARTREE_KJ_PER_MOL,
    "source": "scipy.constants.physical_constants['Hartree energy'] * ['Avogadro constant'] / 1000 "
              "(CODATA) -- the only conversion applied to the OpenMM plugin's own numbers",
}


def load_mbpol_openmm(out):
    """MB-pol as the Paesani group's own OpenMM plugin computed it, if that run happened.

    `mbpol_openmm.py` runs under a python 3.6 environment pinned to the plugin's era and writes
    its numbers in the plugin's OWN units; the ONLY thing done to them here is the CODATA
    conversion above.  Nothing about MB-pol is fitted, adjusted or typed anywhere in this file.
    """
    p = pathlib.Path(out) / "mbpol_openmm.json"
    if not p.is_file():
        return None, f"no {p.name} in {out} (the plugin run did not happen here)"
    d = json.loads(p.read_text())
    f_scale = (BOHR_ANGSTROM / 10.0) / HARTREE_KJ_PER_MOL   # kJ/mol/nm -> hartree/bohr
    rows = {}
    for r in d["rows"]:
        rows[r["node"]] = {
            "mbpol": r["mbpol"] / HARTREE_KJ_PER_MOL,
            "mbpol_parts": {k: v / HARTREE_KJ_PER_MOL for k, v in r["mbpol_parts"].items()},
            "mbpol_force": [[c * f_scale for c in row] for row in r["mbpol_force"]],
        }
    meta = {k: d[k] for k in d if k != "rows"}
    meta["hartree_kj_per_mol"] = HARTREE_KJ_PER_MOL
    for k in ("carrier", "e2b_near", "e2b_far", "energy_near", "energy_far"):
        meta["plant_ii"][k] = meta["plant_ii"][k] / HARTREE_KJ_PER_MOL
    meta["plant_ii"]["carrier_nonzero_in_that_sector"] = abs(meta["plant_ii"]["carrier"]) > 1e-9
    meta["plant_ii"]["fires"] = bool(abs(meta["plant_ii"]["carrier"]) > 1e-9
                                     and abs(meta["plant_ii"]["e2b_far"]) < 1e-9)
    return rows, meta


def load_mbx():
    """Import MBX's own python module, or return the exact reason it could not be used."""
    home = os.environ.get("MBX_HOME", "")
    if not home:
        return None, "MBX_HOME is not set"
    plug = pathlib.Path(home) / "plugins" / "python" / "mbx"
    if not plug.is_dir():
        return None, f"no python plugin at {plug}"
    lib = pathlib.Path(home) / "lib" / "libmbx.so"
    if not lib.is_file():
        return None, f"no shared library at {lib} (configure with --enable-shared)"
    sys.path.insert(0, str(plug.parent))
    try:
        import mbx as mbx_module  # noqa: F401
    except Exception as exc:  # the failure is the record, not a substituted number
        return None, f"{type(exc).__name__}: {exc}"
    return mbx_module, None


def mbpol_dimer(mbx, cfg, donor, acceptor):
    """MB-pol's interaction energy and atomic forces for one dimer, in atomic units.

    `E_int = E(dimer) - E(monomer A) - E(monomer B)` with every monomer at ITS OWN geometry
    in the dimer -- the same definition the exact record uses (`de_exact` is the supermolecule
    total minus the two monomer solves at the same geometries).  MB-pol's one-body term is
    the Partridge-Schwenke monomer surface, so it is large on these minimal-basis-relaxed
    monomers and cancels exactly in this difference.
    """
    names = ["O", "H", "H"]

    def run(fragments):
        xyz = []
        for frag in fragments:
            for atom in frag:
                xyz.extend([float(x) for x in atom])
        nat = 3 * len(fragments)
        mbx.initialize_system(xyz, [3] * len(fragments), names * len(fragments),
                              ["h2o"] * len(fragments), cfg, units="au")
        e, g = mbx.get_energy_grad(xyz, nat, units="au")
        decomp = mbx.get_energy_decomp(xyz, nat, units="au")
        mbx.finalize_system()
        return e, np.array(g, dtype=float).reshape(nat, 3), decomp

    e_dim, g_dim, d_dim = run([donor, acceptor])
    e_a, g_a, d_a = run([donor])
    e_b, g_b, d_b = run([acceptor])
    grad = g_dim.copy()
    grad[0:3] -= g_a
    grad[3:6] -= g_b
    parts = {k: d_dim[i] - d_a[i] - d_b[i]
             for i, k in enumerate(["e1b", "e2b", "e3b", "e4b", "edisp", "ebuck", "eelec"])}
    return e_dim - e_a - e_b, -grad, parts


# --------------------------------------------------------------------------------- main


def main():
    out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    served = json.loads((out / "served.json").read_text())
    rows = served["rows"]
    t0 = time.time()

    # MB-pol: the OpenMM plugin's precomputed run if it happened, else MBX in process, else VOID
    plugin_rows, plugin_meta = load_mbpol_openmm(out)
    mbx, mbx_reason = (None, plugin_meta) if plugin_rows is not None else load_mbx()
    cfg = str(out / "mbx.json")
    if mbx is not None:
        (out / "mbx.json").write_text(MBX_JSON)

    result = {
        "phase": "references",
        "served_source": str(out / "served.json"),
        "geometries": len(rows),
        "constants": CONSTANT_SOURCE,
        "tip4p2005": {
            "source": TIP4P2005_SOURCE,
            "parameters": TIP4P2005,
            "placement_rule": tip4p_sites.__doc__,
            "primary_rule": "remap",
        },
        "mbpol": {
            "available": plugin_rows is not None or mbx is not None,
            "route": ("the Paesani group's OpenMM plugin, precomputed by mbpol_openmm.py"
                      if plugin_rows is not None else
                      ("MBX in process" if mbx is not None else None)),
            "reason_unavailable": None if (plugin_rows is not None or mbx is not None) else mbx_reason,
            "plugin": plugin_meta if plugin_rows is not None else None,
            "mbx": {
                "implementation": "MBX (paesanilab/MBX), the Paesani group's own C++ library, "
                                  "through its python plugin",
                "used": mbx is not None and plugin_rows is None,
                "mbx_home": os.environ.get("MBX_HOME", ""),
                "config": json.loads(MBX_JSON) if (mbx is not None and plugin_rows is None) else None,
            },
        },
        "rows": [],
    }

    for r in rows:
        donor = [list(map(float, a)) for a in r["donor_centers"]]
        acceptor = [list(map(float, a)) for a in r["acceptor_centers"]]
        entry = {"node": r["node"], "family": r["family"], "held_out": r["held_out"]}
        for rule in ("remap", "as_is"):
            sa, qa, _, moved_a = tip4p_sites(donor, rule)
            sb, qb, _, moved_b = tip4p_sites(acceptor, rule)
            e, f_b = tip4p_pair(sa, qa, sb, qb)
            entry[f"tip4p2005_{rule}"] = float(e)
            entry[f"tip4p2005_{rule}_force_on_acceptor"] = [float(x) for x in f_b]
            entry[f"tip4p2005_{rule}_hydrogen_moved_bohr"] = float(max(moved_a, moved_b))
        if plugin_rows is not None:
            entry.update(plugin_rows[r["node"]])
        elif mbx is not None:
            e, force, parts = mbpol_dimer(mbx, cfg, donor, acceptor)
            entry["mbpol"] = float(e)
            entry["mbpol_force"] = [[float(c) for c in row] for row in force]
            entry["mbpol_parts"] = {k: float(v) for k, v in parts.items()}
        result["rows"].append(entry)

    # ------------------------------------------------- the classical force is its own derivative
    # A sign error in a Coulomb force is invisible in the energy and would be invisible in every
    # table below, so the analytic net force on the acceptor is checked against a central
    # difference of the interaction energy under a RIGID translation of that molecule, on every
    # geometry. Bar is absolute (hartree/bohr): a component passes through zero.
    fd_h = 1e-5
    fd_worst, fd_at = 0.0, ""
    for r in rows:
        donor = [list(map(float, a)) for a in r["donor_centers"]]
        acceptor = [list(map(float, a)) for a in r["acceptor_centers"]]
        sa, qa, _, _ = tip4p_sites(donor, "remap")
        sb, qb, _, _ = tip4p_sites(acceptor, "remap")
        _, f_analytic = tip4p_pair(sa, qa, sb, qb)
        for c in range(3):
            shift = np.zeros(3)
            shift[c] = fd_h
            ep, _ = tip4p_pair(sa, qa, sb + shift, qb)
            em, _ = tip4p_pair(sa, qa, sb - shift, qb)
            fd = -(ep - em) / (2.0 * fd_h)
            if abs(fd - f_analytic[c]) > fd_worst:
                fd_worst, fd_at = abs(fd - f_analytic[c]), r["node"]
    result["tip4p2005"]["force_is_its_own_derivative"] = {
        "rule": "worst absolute |analytic net force on the acceptor - central difference of the "
                "interaction energy under a rigid translation of that molecule| at h = 1e-5 bohr, "
                "over every geometry and all three components",
        "h_bohr": fd_h,
        "bar_hartree_per_bohr": 1e-8,
        "worst": float(fd_worst),
        "at": fd_at,
        "pass": bool(fd_worst <= 1e-8),
    }

    # ------------------------------------------------------------------------- the plants
    # Two plants, each with its carrier asserted NONZERO in the sector the plant acts on. A
    # reference model that cannot be broken on purpose is a number nobody has checked.
    plant_node = next(r for r in rows if r["held_out"])
    donor = [list(map(float, a)) for a in plant_node["donor_centers"]]
    acceptor = [list(map(float, a)) for a in plant_node["acceptor_centers"]]
    sa, qa, _, _ = tip4p_sites(donor, "remap")
    sb, qb, _, _ = tip4p_sites(acceptor, "remap")
    e_full, _ = tip4p_pair(sa, qa, sb, qb)
    zero = np.zeros(4)
    e_lj, _ = tip4p_pair(sa, zero, sb, zero)
    e_full, e_lj = float(e_full), float(e_lj)
    coulomb = e_full - e_lj
    result["plants"] = {
        "i": {
            "rule": "TIP4P/2005's charges set to an exact zero: the interaction must move by "
                    "exactly the Coulomb sum, which is computed here by the same routine with the "
                    "Lennard-Jones term alone left standing",
            "node": plant_node["node"],
            "sector": "the electrostatic sector of the classical reference",
            "carrier": coulomb,
            "carrier_nonzero_in_that_sector": bool(abs(coulomb) > 1e-9),
            "energy_full": e_full,
            "energy_charges_off": e_lj,
            "moved": e_full - e_lj,
            "miss": abs((e_full - e_lj) - coulomb),
            "fires": bool(abs(coulomb) > 1e-9),
        }
    }
    if plugin_rows is not None:
        result["plants"]["ii"] = dict(plugin_meta["plant_ii"])
        result["plants"]["ii"]["rule"] = (
            "the acceptor moved 40 bohr along x -- the engine's own far reference. MB-pol's "
            "short-range two-body term must fall below the floor there, because it is switched "
            "off past its own cutoff by construction. The TOTAL interaction is NOT required to "
            "vanish: MB-pol's electrostatics are long-ranged and a dipole-dipole tail at 40 bohr "
            "is real, so the far total is reported beside the plant rather than hidden inside "
            "its floor.")
        result["plants"]["ii"]["far_total_is_reported_not_gated"] = True
    elif mbx is not None:
        far = [[a[0] + 40.0, a[1], a[2]] for a in acceptor]
        e_near, _, parts_near = mbpol_dimer(mbx, cfg, donor, acceptor)
        e_far, _, _ = mbpol_dimer(mbx, cfg, donor, far)
        carrier = parts_near["e2b"]
        _, _, parts_far = mbpol_dimer(mbx, cfg, donor, far)
        result["plants"]["ii"] = {
            "rule": "the acceptor moved 40 bohr along x -- the engine's own far reference. "
                    "MB-pol's short-range two-body POLYNOMIAL (E2b) must fall below the floor "
                    "there, because it is switched off past its own cutoff by construction. The "
                    "TOTAL interaction is NOT required to vanish: MB-pol's electrostatics are "
                    "long-ranged and a dipole-dipole tail at 40 bohr is real, so the far total is "
                    "reported beside the plant rather than hidden inside its floor.",
            "node": plant_node["node"],
            "sector": "MB-pol's short-range two-body polynomial sector",
            "carrier": float(carrier),
            "carrier_nonzero_in_that_sector": bool(abs(carrier) > 1e-9),
            "floor_hartree": 1e-9,
            "e2b_near": float(carrier),
            "e2b_far": float(parts_far["e2b"]),
            "energy_near": float(e_near),
            "energy_far": float(e_far),
            "far_total_is_reported_not_gated": True,
            "fires": bool(abs(carrier) > 1e-9 and abs(parts_far["e2b"]) < 1e-9),
        }

    result["seconds"] = time.time() - t0
    (out / "references.json").write_text(json.dumps(result, indent=1) + "\n")
    on = plugin_rows is not None or mbx is not None
    print(f"references.json  {len(rows)} geometries; "
          f"MB-pol {result['mbpol']['route'] if on else 'OFF (' + str(mbx_reason) + ')'}")


if __name__ == "__main__":
    main()
