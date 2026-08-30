"""MODEL DEFINITION: the STO-3G table for Z = 1 to 18.

This file is the MIXTURES-1 referee's declared basis and nothing else.  It holds
the eight second-row elements' exponents; the first row is IMPORTED from the
ELEMENTS-1 referee rather than re-typed, because two transcriptions of one table
are two things that can drift and the H2 bank grades against that one.

WHAT THE SECOND ROW COSTS IN NEW MACHINERY: NOTHING.

The brief for this lane said d-orbital integrals would be needed.  They are not,
and the frozen prereg contains its own proof: it states that Ar2 is ONE
determinant.  Ar2 is one determinant only if argon carries 9 basis functions --
1s, 2s, 2p, 3s, 3p, which is s and p and no more -- because 36 electrons in 18
spatial orbitals leaves na = nb = 18 and C(18,18)^2 = 1.  Its other stated
figure agrees: Na2 at about 1e9 determinants is C(18,11)^2 = 1.013e9, again 18
orbitals.  The engine agrees a third time: `elements.rs` builds Na..Ar from a
macro named `second_row!` that declares FIVE shells, s1/2s/2p/3s/3p, and while
the crate does define `ShellKind::D3` and `C_3D`, NO species uses them.

So the second row is reached with the l <= 1 integrals ELEMENTS-1 already
validated on F2 and Ne2, and the only genuinely new declared input is 24
exponents.  That is worth saying plainly because the alternative -- adding d
functions to be safe -- would not have been a safe superset.  It would have been
a DIFFERENT MODEL from the one the engine computes, and every R1 and R2
comparison would have failed for a reason that looked like an integral bug.

(`holon-chem/src/md.rs` does implement l = 2, and `tests/elements.rs` exercises
`C_3D`.  That code is real and correct; it is simply not reached by any declared
species, which is a different status from wrong, and is recorded here so the
next lane does not rediscover it.)

THE FINGERPRINT CHANGES, AND THAT IS THE POINT.

`basis_fingerprint` hashes the whole table, so this table's fingerprint is not
ELEMENTS-1's, and a cache record computed under one is refused by the other.
That is correct rather than inconvenient: the two campaigns declare different
models -- the same first row inside a larger table -- and a shared cache would
be the mechanism by which one campaign's numbers silently entered the other's
artifact.  The two caches are separate directories for the same reason.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
_E1 = os.path.join(HERE, "elements1")
if _E1 not in sys.path:
    sys.path.insert(0, _E1)

import elements_core as EC          # noqa: E402  (path set above)

# The universal contraction coefficients.  The first three are ELEMENTS-1's, by
# reference: they are read out of the imported table rather than re-typed, so a
# change to the first row cannot leave this file behind.
C_1S = EC.STO3G_SHELLS[1][0][2]
C_2S = EC.STO3G_SHELLS[3][1][2]
C_2P = EC.STO3G_SHELLS[3][2][2]
# The 3s and 3p contractions are new to this campaign; they are the universal
# STO-3G values and are transcribed at the eight decimals the engine declares.
C_3S = ("-0.21962037", "0.22559543", "0.90039843")
C_3P = ("0.01058760", "0.59516701", "0.46200101")

# DECLARED INPUT: the second-row exponents, at the eight decimals the engine's
# `elements.rs` declares them to.  Each element gives three sets: the 1s
# exponents, the shared 2s/2p exponents, and the shared 3s/3p exponents.
SECOND_ROW_EXPONENTS = {
    11: (  # Na
        ("250.77243000", "45.67851117", "12.36238776"),
        ("12.04019274", "2.79788186", "0.90995802"),
        ("1.47874062", "0.41256488", "0.16147510"),
    ),
    12: (  # Mg
        ("299.23741370", "54.50646845", "14.75157752"),
        ("15.12182352", "3.51398658", "1.14285750"),
        ("1.39544829", "0.38932653", "0.15237977"),
    ),
    13: (  # Al
        ("351.42147670", "64.01186067", "17.32410761"),
        ("18.89939621", "4.39181323", "1.42835397"),
        ("1.39544829", "0.38932653", "0.15237977"),
    ),
    14: (  # Si
        ("407.79755140", "74.28083305", "20.10329229"),
        ("23.19365606", "5.38970687", "1.75289995"),
        ("1.47874062", "0.41256488", "0.16147510"),
    ),
    15: (  # P
        ("468.36563780", "85.31338559", "23.08913156"),
        ("28.03263958", "6.51418258", "2.11861435"),
        ("1.74310323", "0.48632138", "0.19034289"),
    ),
    16: (  # S
        ("533.12573590", "97.10951830", "26.28162542"),
        ("33.32975173", "7.74511752", "2.51895260"),
        ("2.02919427", "0.56614005", "0.22158338"),
    ),
    17: (  # Cl
        ("601.34561360", "109.53585420", "29.64467686"),
        ("38.96041889", "9.05356348", "2.94449983"),
        ("2.12938650", "0.59409343", "0.23252414"),
    ),
    18: (  # Ar
        ("674.44651840", "122.85127530", "33.24834945"),
        ("45.16424392", "10.49519900", "3.41336445"),
        ("2.62136652", "0.73135461", "0.28624724"),
    ),
}

SYMBOL = {11: "Na", 12: "Mg", 13: "Al", 14: "Si",
          15: "P", 16: "S", 17: "Cl", 18: "Ar"}

# DECLARED INPUT (measured): the most abundant isotope's atomic mass, in
# unified atomic mass units.  These enter NOTHING electronic -- the model is
# Born-Oppenheimer and knows no nuclear mass -- and are carried only so the
# renderer contract can form a reduced mass.  They are checked against the
# engine's `elements.rs` in test_basis_matches_engine.py for the same reason the
# exponents are: a mistyped mass is invisible in every energy and wrong in every
# timescale.
ISOTOPE_MASS_U = {
    11: "22.9897692820", 12: "23.985041697", 13: "26.98153853",
    14: "27.976926535", 15: "30.973761998", 16: "31.972071174",
    17: "34.968852682", 18: "39.9623831225",
}
ISOTOPE = {11: "23Na", 12: "24Mg", 13: "27Al", 14: "28Si",
           15: "31P", 16: "32S", 17: "35Cl", 18: "40Ar"}


def _second_row_shells(Z):
    """The five shells, in the order they enter the basis: 1s 2s 2p 3s 3p.

    ORDER IS PART OF THE MODEL, not a presentation choice: it fixes which basis
    function is index 0, and the engine's `second_row!` macro lists them in this
    order.  A referee that agreed on every exponent and disagreed on the order
    would produce the same energies and a different orbital numbering, which is
    invisible in a total energy and fatal in anything resolved per orbital.
    """
    s1, sp2, sp3 = SECOND_ROW_EXPONENTS[Z]
    return (
        (0, s1, C_1S),
        (0, sp2, C_2S),
        (1, sp2, C_2P),
        (0, sp3, C_3S),
        (1, sp3, C_3P),
    )


MAX_Z = 18


def shells_for(Z):
    """The declared shells for one element, or a REFUSAL.

    The engine now declares elements up to xenon; this referee's declared model
    stops at argon.  Silently returning nothing for Z = 19 would build a smaller
    basis and report a HIGHER energy that looks perfectly converged -- the
    failure shape this campaign keeps finding.  So the boundary raises.
    """
    if Z not in STO3G_18:
        raise KeyError(
            "Z=%d is outside the MIXTURES-1 declared model (Z = 1..%d); the "
            "engine may declare it, this referee does not, and there is "
            "therefore no referee value for any species containing it"
            % (Z, MAX_Z))
    return STO3G_18[Z]


def build_table():
    """{Z: ((l, exponents, coefficients), ...)} for Z = 1..18."""
    t = dict(EC.STO3G_SHELLS)               # the first row, by reference
    for Z in sorted(SECOND_ROW_EXPONENTS):
        t[Z] = _second_row_shells(Z)
    return t


STO3G_18 = build_table()

# n_functions per element: s -> 1, p -> 3.  No shell here has l > 1.
NBF = {Z: sum(1 if l == 0 else 3 for (l, _, _) in shells)
       for Z, shells in STO3G_18.items()}


def fingerprint():
    return EC.basis_fingerprint(STO3G_18)


def elements1_fingerprint():
    return EC.basis_fingerprint(EC.STO3G_SHELLS)


if __name__ == "__main__":
    print("MIXTURES-1 declared basis: Z = 1..%d" % max(STO3G_18))
    print("fingerprint            %s" % fingerprint())
    print("ELEMENTS-1 fingerprint %s  (different by construction; a record "
          "under one is refused by the other)" % elements1_fingerprint())
    print()
    print("%-4s %-3s %-6s %s" % ("Z", "sym", "nbf", "shells (l)"))
    for Z in sorted(STO3G_18):
        sym = EC.ELEMENT_SYMBOL.get(Z, SYMBOL.get(Z, "?"))
        print("%-4d %-3s %-6d %s"
              % (Z, sym, NBF[Z],
                 " ".join(str(l) for (l, _, _) in STO3G_18[Z])))
    assert max(l for sh in STO3G_18.values() for (l, _, _) in sh) <= 1, \
        "a shell with l > 1 entered the declared table"
    print("\nmax l in the declared table: 1  (no d shell; see the header)")
