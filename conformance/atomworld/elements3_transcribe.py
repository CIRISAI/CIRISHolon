#!/usr/bin/env python3
"""Generate the ELEMENTS-3 registry block (Z = 19..54) from the pinned tabulations.

# Why this is generated and not typed

The oxygen defect is in `elements.rs`'s module header: one digit of one exponent was
transcribed wrong, every energy stayed self-consistent, every gate stayed green, and the
crate was quietly solving a different model. ELEMENTS-3 asks for 36 more elements, 130
more shells and roughly 800 more declared numbers of exactly that kind. Hand transcription
at that volume is not a risk to be managed, it is a defect to be scheduled.

So the numbers are not typed. They are read from two pinned source files committed beside
this script -- the Basis Set Exchange's STO-3G tabulation and NIST's relative atomic masses
-- and the Rust is emitted. What a human reviews is this script and the structural gates,
not 800 digits.

# What is NOT automated

The (n, l) assignment. The Basis Set Exchange lists shells in a canonical order that is
NOT ascending in principal quantum number: gallium's third listed shell is 4s4p and its
fourth is 3s3p3d, because STO-3G groups the d function with the sp set that shares its
exponents. Getting that wrong would relabel a core shell as a valence one with every
number still correct, which is a defect no digit-level check can see.

It is therefore derived here from a fact of the tabulation rather than assumed: STO-3G's
contraction coefficients are universal per (n, l) fit, so the coefficient triple IDENTIFIES
which fit a shell belongs to. `FAMILIES` below is that map, and `verify_assignment` checks
it three independent ways -- no duplicate (n, l) within an element, the shell set matches
the aufbau occupancy (up to STO-3G's convention of carrying an unoccupied p partner for
each declared s), and the leading exponent decreases with n at fixed l.

# Usage

    python3 conformance/atomworld/elements3_transcribe.py --check
    python3 conformance/atomworld/elements3_transcribe.py --emit > /tmp/block.rs
"""

import argparse
import json
import math
import os
import re
import sys
from decimal import Decimal, ROUND_HALF_EVEN

HERE = os.path.dirname(os.path.abspath(__file__))
BASIS = os.path.join(HERE, "elements3_sto3g.json")
MASSES = os.path.join(HERE, "elements3_masses.json")

# Declared exponents carry eight decimal places, which is the convention every element
# already in the registry uses and which the ratio gate's derived tolerance is written
# against. The underlying tabulation carries ten significant digits, so for exponents above
# ~100 the declaration is EXACT and for small ones it is a rounding -- the gate's tolerance
# accounts for both, see `derived_bound`.
DECIMALS = Decimal("1E-8")

# The coefficient triple's leading entry identifies the (n, l) fit it belongs to.
# Two (n, l) pairs carry TWO fits each: 3s/3p and 4s/4p were fitted once for their own row
# and again for the rows below, and the tabulation keeps both. That is a fact about STO-3G,
# not a transcription artifact, and the universality gate is written to expect it.
FAMILIES = {
    (0, "0.15432897"): (1, 0),
    (0, "-0.09996723"): (2, 0),
    (0, "-0.21962037"): (3, 0),   # Na..Ca fit
    (0, "-0.22776350"): (3, 0),   # Sc..Xe fit
    (0, "-0.30884412"): (4, 0),   # K..Sr fit
    (0, "-0.33061006"): (4, 0),   # Y..Xe fit
    (0, "-0.38426426"): (5, 0),
    (1, "0.15591627"): (2, 1),
    (1, "0.01058760"): (3, 1),    # Na..Ca fit
    (1, "0.00495151"): (3, 1),    # Sc..Xe fit
    (1, "-0.12154686"): (4, 1),   # K..Sr fit
    (1, "-0.12839276"): (4, 1),   # Y..Xe fit
    (1, "-0.34816915"): (5, 1),
    (2, "0.21976795"): (3, 2),
    (2, "0.12506621"): (4, 2),
}

# Rust constant names for each fit family, in the same order.
COEFF_CONST = {
    (0, "0.15432897"): "C_1S",
    (0, "-0.09996723"): "C_2S",
    (0, "-0.21962037"): "C_3S",
    (0, "-0.22776350"): "C_3S_HEAVY",
    (0, "-0.30884412"): "C_4S",
    (0, "-0.33061006"): "C_4S_HEAVY",
    (0, "-0.38426426"): "C_5S",
    (1, "0.15591627"): "C_2P",
    (1, "0.01058760"): "C_3P",
    (1, "0.00495151"): "C_3P_HEAVY",
    (1, "-0.12154686"): "C_4P",
    (1, "-0.12839276"): "C_4P_HEAVY",
    (1, "-0.34816915"): "C_5P",
    (2, "0.21976795"): "C_3D",
    (2, "0.12506621"): "C_4D",
}

KIND = {(1, 0): "S1", (2, 0): "S2", (2, 1): "P2", (3, 0): "S3", (3, 1): "P3",
        (3, 2): "D3", (4, 0): "S4", (4, 1): "P4", (4, 2): "D4", (5, 0): "S5",
        (5, 1): "P5"}

ELEMENT_NAME = {
    19: "POTASSIUM", 20: "CALCIUM", 21: "SCANDIUM", 22: "TITANIUM", 23: "VANADIUM",
    24: "CHROMIUM", 25: "MANGANESE", 26: "IRON", 27: "COBALT", 28: "NICKEL",
    29: "COPPER", 30: "ZINC", 31: "GALLIUM", 32: "GERMANIUM", 33: "ARSENIC",
    34: "SELENIUM", 35: "BROMINE", 36: "KRYPTON", 37: "RUBIDIUM", 38: "STRONTIUM",
    39: "YTTRIUM", 40: "ZIRCONIUM", 41: "NIOBIUM", 42: "MOLYBDENUM", 43: "TECHNETIUM",
    44: "RUTHENIUM", 45: "RHODIUM", 46: "PALLADIUM", 47: "SILVER", 48: "CADMIUM",
    49: "INDIUM", 50: "TIN", 51: "ANTIMONY", 52: "TELLURIUM", 53: "IODINE", 54: "XENON",
}


def fam_key(am, coeffs):
    return (am, f"{float(coeffs[0]):.8f}")


def load():
    basis = json.load(open(BASIS))["elements"]
    masses = json.load(open(MASSES))
    return basis, masses


def shells_of(basis, z):
    """[(n, l, [exponent Decimals], family_key)] for one element, ascending in (n, l)."""
    out = []
    for s in basis[str(z)]["electron_shells"]:
        exps = [Decimal(e) for e in s["exponents"]]
        for am, cs in zip(s["angular_momentum"], s["coefficients"]):
            k = fam_key(am, cs)
            if k not in FAMILIES:
                raise SystemExit(f"Z={z}: shell coefficients {k} match no known STO-3G fit")
            n, l = FAMILIES[k]
            out.append((n, l, exps, k))
    out.sort(key=lambda t: (t[0], t[1]))
    return out


def aufbau(z):
    order = [(1, 0), (2, 0), (2, 1), (3, 0), (3, 1), (4, 0), (3, 2), (4, 1),
             (5, 0), (4, 2), (5, 1)]
    cap = {0: 2, 1: 6, 2: 10}
    rem, out = z, set()
    for nl in order:
        if rem <= 0:
            break
        out.add(nl)
        rem -= cap[nl[1]]
    return out


def verify_assignment(basis, lo=1, hi=54):
    """Three independent checks that the (n, l) labels are the right ones."""
    problems = []
    for z in range(lo, hi + 1):
        sh = shells_of(basis, z)
        labels = [(n, l) for n, l, _, _ in sh]
        if len(labels) != len(set(labels)):
            problems.append(f"Z={z}: two shells claim the same (n, l): {sorted(labels)}")
            continue
        occ = aufbau(z)
        # STO-3G declares s and p together, so an occupied s may bring an EMPTY p partner.
        allowed = set(occ) | {(n, 1) for (n, l) in occ if l == 0}
        if not (occ <= set(labels) <= allowed):
            problems.append(f"Z={z}: shells {sorted(set(labels))} vs aufbau {sorted(occ)}")
        by_l = {}
        for n, l, ex, _ in sh:
            by_l.setdefault(l, []).append((n, ex[0]))
        for l, v in by_l.items():
            v.sort()
            for a, b in zip(v, v[1:]):
                if not a[1] > b[1]:
                    problems.append(
                        f"Z={z}: l={l} leading exponent does not fall from n={a[0]} to n={b[0]}"
                    )
    return problems


def q(x: Decimal) -> str:
    """Eight decimal places, round-half-even, rendered without an exponent."""
    return f"{x.quantize(DECIMALS, rounding=ROUND_HALF_EVEN):f}"


def derived_bound(v: float) -> float:
    """Half a unit in the last place the DECLARATION can carry.

    Two limits apply and the honest bound is the coarser of them: the declaration carries
    eight decimal places, and the tabulation it comes from carries ten significant digits.
    For a small exponent the first binds; for a large one the second does.
    """
    return max(0.5e-8, 0.5 * 10 ** (math.floor(math.log10(v)) - 9))


def check_ratios(basis, lo=1, hi=54, margin=4.0):
    """The gate T1 will run in Rust, run here on the values that will be DECLARED."""
    fams = {}
    for z in range(lo, hi + 1):
        for n, l, ex, k in shells_of(basis, z):
            vals = [float(q(e)) for e in ex]
            fams.setdefault(k, []).append((z, vals))
    worst_overall = 0.0
    rows = []
    for k, members in sorted(fams.items()):
        if len(members) < 2:
            rows.append((COEFF_CONST[k], len(members), None, None))
            continue
        worst_fam = 0.0
        for i, j in [(0, 1), (1, 2)]:
            cand = []
            for z, v in members:
                b = derived_bound(v[i]) / v[i] + derived_bound(v[j]) / v[j]
                cand.append((z, v[i] / v[j], b))
            ref = min(cand, key=lambda t: t[2])
            for z, r, b in cand:
                if z == ref[0]:
                    continue
                worst_fam = max(worst_fam, abs(r - ref[1]) / ref[1] / (b + ref[2]))
        worst_overall = max(worst_overall, worst_fam)
        rows.append((COEFF_CONST[k], len(members), worst_fam, margin))
    return rows, worst_overall


def emit(basis, masses):
    out = []
    w = out.append
    for z in range(19, 55):
        sh = shells_of(basis, z)
        m = masses[str(z)]
        name = ELEMENT_NAME[z]
        w(f"pub const {name}: Species = Species {{")
        w(f'    symbol: "{m["sym"]}",')
        w(f"    z: {z},")
        w(f'    mass_u: {m["mass"]},')
        w(f'    isotope: "{m["mn"]}{m["sym"]}",')
        w("    shells: &[")
        for n, l, ex, k in sh:
            w("        Shell {")
            w(f"            kind: ShellKind::{KIND[(n, l)]},")
            w(f"            alpha: [{q(ex[0])}, {q(ex[1])}, {q(ex[2])}],")
            w(f"            coeff: {COEFF_CONST[k]},")
            w("        },")
        w("    ],")
        w("};")
    return "\n".join(out)


def emit_pin(basis, lo=1, hi=54):
    """The tabulation, flattened, with NO (n, l) assignment applied.

    One line per (element, angular momentum) contraction, carrying the source's own digits:

        Z  l  c0 c1 c2  a0 a1 a2

    The point of withholding the (n, l) label is that the Rust gate reading this file is
    then checking the DIGITS against the source without inheriting this script's opinion
    about which shell is which. The label is checked separately, by structure, in
    `verify_assignment` and in the ratio gate.
    """
    out = ["# STO-3G, flattened from the pinned Basis Set Exchange tabulation.",
           "# Emitted by conformance/atomworld/elements3_transcribe.py --pin. Do not hand-edit.",
           "# Z l c0 c1 c2 a0 a1 a2   (source precision, no rounding applied)"]
    for z in range(lo, hi + 1):
        for s in basis[str(z)]["electron_shells"]:
            exps = [str(Decimal(e)) for e in s["exponents"]]
            for am, cs in zip(s["angular_momentum"], s["coefficients"]):
                cc = [str(Decimal(c)) for c in cs]
                out.append(f"{z} {am} " + " ".join(cc) + " " + " ".join(exps))
    return "\n".join(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--emit", action="store_true")
    ap.add_argument("--pin", action="store_true")
    a = ap.parse_args()
    basis, masses = load()

    problems = verify_assignment(basis)
    if problems:
        for p in problems:
            print("ASSIGNMENT PROBLEM:", p, file=sys.stderr)
        raise SystemExit(1)

    if a.check:
        print(f"(n, l) assignment: clean over Z = 1..54, {len(FAMILIES)} fit families")
        rows, worst = check_ratios(basis)
        print(f"{'family':<12} {'members':>7}  worst ratio deviation, in units of its own bound")
        for nm, cnt, worst_fam, margin in rows:
            s = "single member, no ratio" if worst_fam is None else f"{worst_fam:8.3f}x  (gate fires at {margin:.0f}x)"
            print(f"{nm:<12} {cnt:>7}  {s}")
        print(f"\nworst over all families: {worst:.3f}x its own derived rounding bound")
        # Technetium has no stable isotope; its declared mass is a CHOICE, not an abundance.
        tc = masses["43"]
        print(f"\nnote: Z=43 {tc['sym']} natural abundance {tc['comp']} -- "
              f"declared isotope {tc['mn']}{tc['sym']} is representative, not most-abundant")
    if a.emit:
        print(emit(basis, masses))
    if a.pin:
        print(emit_pin(basis))


if __name__ == "__main__":
    main()
