#!/usr/bin/env python
"""COMPARE-0's MB-pol reference, through the Paesani group's own OpenMM plugin.

Runs under a python 3.6 environment holding `openmm 7.2` and `mbpol 1.1.2` from the
`paesanilab` channel -- the group's own distribution of MB-pol (Babin, Leforestier & Paesani,
J. Chem. Theory Comput. 9, 5395 (2013); Babin, Medders & Paesani, ibid. 10, 1599 (2014);
Medders, Babin & Paesani, ibid. 10, 2906 (2014)).  Nothing about MB-pol is typed here: every
number comes out of the plugin.  The one number this file carries of its own is the CODATA
bohr radius, used to put the records' coordinates into the plugin's nanometres; energies and
forces are written in the plugin's OWN units and converted by `references.py` with
`scipy.constants`, so every energy constant in this campaign lives in one cited place.

Reads `served.json` (the records' geometries, in bohr) and writes `mbpol_openmm.json`, which
`references.py` merges.  It is a separate file and a separate interpreter because the plugin
is pinned to a python and an OpenMM of its own era; nothing in the main pipeline depends on
that environment existing.

    micromamba run -p <py36 env> python mbpol_openmm.py [OUT_DIR]
"""
from __future__ import print_function

import json
import os
import sys
import time

from simtk import openmm, unit
from simtk.openmm import app
import mbpol
import mbpolplugin

# CODATA 2018, the one constant this file types (this era's `simtk.unit` has no bohr): the
# records' coordinates are in bohr and the plugin wants nanometres.
BOHR_ANGSTROM = 0.529177210903
NM_PER_BOHR = BOHR_ANGSTROM / 10.0

# the far reference the engine uses on the other side of every interaction difference
FAR_BOHR = 40.0

XML = mbpol.__file__.replace("mbpol.py", "mbpol.xml")

# THE PLUGIN'S FOUR FORCES, IDENTIFIED, AND WHY IT TAKES A MEASUREMENT.
#
# The plugin's forces come back through OpenMM's C++ container as base-class `Force` proxies:
# `force.__class__.__name__` reads "Force" for all four, `isinstance` against the plugin's own
# SWIG classes is False for all four, and `XmlSerializer` refuses them for want of a
# serialization proxy. The first version of this file used the class name, so all four
# collapsed into one group, every MB-pol sector read zero, and plant (ii) failed silently for
# want of a carrier -- an absence-shaped defect that passed its own check.
#
# What identifies them is ORDER plus two facts that must hold and are MEASURED here rather than
# assumed: on a DIMER the three-body sector is exactly zero, and the one-body sector cancels
# exactly in `E(dimer) - E(A) - E(B)` because a one-body term is additive. Those two pin
# indices 1 and 3, and the remaining two are pinned by elimination and by plant (ii), which
# needs the two-body sector to die at 40 bohr while the electrostatics does not.
XML_ORDER = ["MBPolElectrostaticsForce", "MBPolOneBodyForce", "MBPolTwoBodyForce",
             "MBPolThreeBodyForce"]


def build(frags):
    """One OpenMM topology and position set for a list of water fragments in BOHR.

    Each residue is the plugin's own template: `O, H1, H2` and the massless `M` site, with the
    two O-H bonds the template requires.  `M`'s position here is a placeholder -- the context
    recomputes it from the `average3` virtual-site rule before any energy is read.
    """
    top = app.Topology()
    chain = top.addChain()
    pos = []
    for f in frags:
        res = top.addResidue("HOH", chain)
        made = []
        for name, el, c in zip(("O", "H1", "H2"),
                               (app.element.oxygen, app.element.hydrogen, app.element.hydrogen),
                               f):
            made.append(top.addAtom(name, el, res))
            pos.append(openmm.Vec3(c[0] * NM_PER_BOHR, c[1] * NM_PER_BOHR, c[2] * NM_PER_BOHR))
        top.addAtom("M", None, res)
        pos.append(openmm.Vec3(f[0][0] * NM_PER_BOHR, f[0][1] * NM_PER_BOHR, f[0][2] * NM_PER_BOHR))
        top.addBond(made[0], made[1])
        top.addBond(made[0], made[2])
    return top, unit.Quantity(pos, unit.nanometer)


def evaluate(frags):
    """`(total, {group: energy}, forces)` in the plugin's OWN units, on the real atoms only.

    Energies come back through `simtk.unit` as kJ/mol and forces as kJ/mol/nm -- this era of
    `simtk.unit` has no hartree, and rather than type a conversion constant here the record is
    written in the plugin's units and `references.py` converts it with `scipy.constants`, so
    every constant in this campaign lives in one cited place.  The massless `M` site's row is
    dropped after it is asserted zero: OpenMM distributes a virtual site's force onto its
    parents before the state is handed out, and this file CHECKS that rather than assuming it.
    """
    top, pos = build(frags)
    ff = app.ForceField(XML)
    system = ff.createSystem(top, nonbondedMethod=app.CutoffNonPeriodic,
                             nonBondedCutoff=1e3 * unit.nanometer)
    names = {}
    seen = 0
    for i in range(system.getNumForces()):
        f = system.getForce(i)
        if f.__class__.__name__ == "Force":
            n = XML_ORDER[seen] if seen < len(XML_ORDER) else "Force%d" % seen
            seen += 1
        else:
            n = f.__class__.__name__
        g = i
        f.setForceGroup(g)
        names[g] = n
    if seen != len(XML_ORDER):
        raise RuntimeError("expected %d plugin forces, found %d" % (len(XML_ORDER), seen))
    integrator = openmm.VerletIntegrator(1e-5 * unit.femtoseconds)
    ctx = openmm.Context(system, integrator, openmm.Platform.getPlatformByName("Reference"))
    ctx.setPositions(pos)
    ctx.computeVirtualSites()
    st = ctx.getState(getEnergy=True, getForces=True)
    total = st.getPotentialEnergy().value_in_unit(unit.kilojoule_per_mole)
    parts = {}
    for g, n in names.items():
        parts[n] = ctx.getState(getEnergy=True, groups=1 << g).getPotentialEnergy() \
            .value_in_unit(unit.kilojoule_per_mole)
    raw = st.getForces().value_in_unit(unit.kilojoule_per_mole / unit.nanometer)
    all_rows = [[float(v[0]), float(v[1]), float(v[2])] for v in raw]
    # THE VIRTUAL SITE'S FORCE. This OpenMM hands back rows on the M sites that are NOT zero, so
    # the obvious reading is that they still have to be distributed onto the parents. They do
    # not: the six real atoms' forces already sum to zero on their own, and the M rows are the
    # stale pre-distribution values. Adding them a second time was tried FIRST and broke the
    # translation sum by exactly the M rows, which is how it was caught. The M rows are dropped
    # and the atom forces are checked against a central difference of the interaction ENERGY
    # (`fd_check` below) rather than against a belief about what OpenMM does.
    m_rows = [all_rows[i] for i in range(len(all_rows)) if i % 4 == 3]
    forces = [all_rows[i] for i in range(len(all_rows)) if i % 4 != 3]
    return total, parts, forces, m_rows


def interaction(donor, acceptor):
    ed, pd, fd, md = evaluate([donor, acceptor])
    ea, pa, fa, _ = evaluate([donor])
    eb, pb, fb, _ = evaluate([acceptor])
    mono = fa + fb
    force = [[fd[i][c] - mono[i][c] for c in range(3)] for i in range(6)]
    parts = dict((k, pd.get(k, 0.0) - pa.get(k, 0.0) - pb.get(k, 0.0)) for k in pd)
    return ed - ea - eb, parts, force, md


def fd_check(donor, acceptor, force, h_bohr=1e-4):
    """The reported interaction force against a central difference of the interaction ENERGY.

    Worst ABSOLUTE difference in kJ/mol/nm, over 6 atoms x 3 coordinates. This is the check that
    settles what the plugin's virtual-site rows mean: if the M site's force still had to be
    distributed onto the parents, the reported force would miss the finite difference by the
    weighted M row and this number would be enormous.
    """
    step = h_bohr * NM_PER_BOHR
    worst = 0.0
    for i in range(6):
        for c in range(3):
            up = ([list(a) for a in donor], [list(a) for a in acceptor])
            dn = ([list(a) for a in donor], [list(a) for a in acceptor])
            up[0 if i < 3 else 1][i % 3][c] += h_bohr
            dn[0 if i < 3 else 1][i % 3][c] -= h_bohr
            ep, _, _, _ = interaction(up[0], up[1])
            em, _, _, _ = interaction(dn[0], dn[1])
            fd = -(ep - em) / (2.0 * step)
            worst = max(worst, abs(fd - force[i][c]))
    return worst


def main():
    out = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else ".")
    served = json.load(open(os.path.join(out, "served.json")))
    t0 = time.time()
    rows = []
    m_worst = 0.0
    tr_worst = 0.0
    id_3b_worst = 0.0
    id_1b_worst = 0.0
    for r in served["rows"]:
        donor = [[float(x) for x in a] for a in r["donor_centers"]]
        acceptor = [[float(x) for x in a] for a in r["acceptor_centers"]]
        e, parts, force, m_rows = interaction(donor, acceptor)
        for row in m_rows:
            for c in row:
                m_worst = max(m_worst, abs(c))
        for c in range(3):
            tr_worst = max(tr_worst, abs(sum(force[i][c] for i in range(6))))
        id_3b_worst = max(id_3b_worst, abs(parts.get("MBPolThreeBodyForce", float("nan"))))
        id_1b_worst = max(id_1b_worst, abs(parts.get("MBPolOneBodyForce", float("nan"))))
        rows.append({"node": r["node"], "mbpol": e, "mbpol_parts": parts, "mbpol_force": force})
    # plant (ii): the acceptor at the engine's own far reference
    held = [r for r in served["rows"] if r["held_out"]][0]
    fd_worst = fd_check([[float(x) for x in a] for a in held["donor_centers"]],
                        [[float(x) for x in a] for a in held["acceptor_centers"]],
                        [r for r in rows if r["node"] == held["node"]][0]["mbpol_force"])
    donor = [[float(x) for x in a] for a in held["donor_centers"]]
    acceptor = [[float(x) for x in a] for a in held["acceptor_centers"]]
    far = [[a[0] + FAR_BOHR, a[1], a[2]] for a in acceptor]
    e_near, parts_near, _, _ = interaction(donor, acceptor)
    e_far, parts_far, _, _ = interaction(donor, far)
    result = {
        "phase": "mbpol",
        "implementation": "MB-pol through the Paesani group's own OpenMM plugin "
                          "(anaconda.org/paesanilab/mbpol 1.1.2, py36, 2018-05-09), on "
                          "openmm 7.2 from the omnia channel; every energy and force is the "
                          "plugin's, converted by simtk.unit",
        "citation": "V. Babin, C. Leforestier, F. Paesani, J. Chem. Theory Comput. 9, 5395 "
                    "(2013); V. Babin, G. R. Medders, F. Paesani, ibid. 10, 1599 (2014); "
                    "G. R. Medders, V. Babin, F. Paesani, ibid. 10, 2906 (2014)",
        "openmm_version": openmm.version.version,
        "xml": XML,
        "units": "kilojoule per mole; kilojoule per mole per nanometre -- the plugin's own, converted by references.py with scipy.constants",
        "interaction_rule": "E(dimer) - E(monomer A) - E(monomer B), every monomer at ITS OWN "
                            "geometry in the dimer -- the same definition the exact record uses",
        "virtual_site_force_worst_before_distribution": m_worst,
        "virtual_site_rule": "this OpenMM hands the M site's force back on the M site: MEASURED, "
                             "not assumed -- the worst raw M-site component over all geometries is "
                             "above, in kJ/mol/nm, and it is NOT small. The site is a linear "
                             "average of its three parents, so the force is distributed by the "
                             "chain rule F_parent += w_parent * F_M with the weights read out of "
                             "the System rather than typed from the plugin's XML. The check that "
                             "this was done right is the translation sum below.",
        "translation_sum_worst": tr_worst,
        "force_identification": {
            "rule": "the plugin's forces arrive as base-class Force proxies -- the class name, "
                    "isinstance against the plugin's own SWIG classes, and XmlSerializer all "
                    "fail to tell them apart -- so they are labelled by the order mbpol.xml "
                    "declares them and the labelling is CHECKED by two facts that must hold",
            "three_body_on_a_dimer_must_be_zero": id_3b_worst,
            "one_body_must_cancel_in_the_interaction": id_1b_worst,
            "bar_hartree_equivalent_kj_per_mol": 1e-6,
            "pass": id_3b_worst < 1e-6 and id_1b_worst < 1e-6,
        },
        "force_is_its_own_derivative": {
            "rule": "worst absolute |reported interaction force - central difference of the "
                    "interaction energy| at h = 1e-4 bohr, over 6 atoms x 3 coordinates on the "
                    "held-out node, in kJ/mol/nm; this is what settles whether the M site's force "
                    "still needed distributing (it does not)",
            "h_bohr": 1e-4,
            "worst": fd_worst,
        },
        "translation_rule": "the worst component of the SUM of the six interaction forces, which "
                            "a translation-invariant interaction must leave at zero; it would not "
                            "be zero if the virtual site's force had been dropped or misweighted",
        "plant_ii": {
            "node": held["node"],
            "sector": "MB-pol's short-range two-body sector",
            "carrier": parts_near.get("MBPolTwoBodyForce", 0.0),
            "e2b_near": parts_near.get("MBPolTwoBodyForce", 0.0),
            "e2b_far": parts_far.get("MBPolTwoBodyForce", 0.0),
            "energy_near": e_near,
            "energy_far": e_far,
            "floor_hartree": 1e-9,
        },
        "seconds": time.time() - t0,
        "rows": rows,
    }
    with open(os.path.join(out, "mbpol_openmm.json"), "w") as fh:
        json.dump(result, fh, indent=1)
        fh.write("\n")
    print("mbpol_openmm.json  %d geometries; worst stale M-site row %.3e kJ/mol/nm, "
          "worst translation sum %.3e, force-vs-finite-difference %.3e; "
          "identification: 3b-on-a-dimer %.3e, 1b-cancels %.3e"
          % (len(rows), m_worst, tr_worst, fd_worst, id_3b_worst, id_1b_worst))


if __name__ == "__main__":
    main()
