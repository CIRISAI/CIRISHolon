"""The referee's declared basis and the engine's are ONE model, checked by
parsing the engine's source rather than by trusting a transcription.

WHY THIS EXISTS.  Gates R1 and R2 compare two independent implementations at
1e-10 hartree.  That comparison is only meaningful if both are computing the
SAME model, and the model is 24 new exponents typed by hand into a Python file
from a Rust file.  A single mistyped digit would move an energy by far more than
1e-10 and would present as an integral bug in whichever side was looked at
second.  So the two declarations are compared directly, before any energy is
computed, and the comparison reads the engine's source -- it does not read a
summary of it, and no number below is written down twice.

DIRECTION MATTERS, AND BOTH ARE CHECKED.  A test that only asks "is every
referee exponent present in the engine" passes when the referee is missing an
element entirely.  A test that only asks the reverse passes when the referee
carries an element the engine does not.  Both directions are asserted, and so is
the SHELL ORDER, which fixes the basis-function numbering and is invisible in a
total energy.

THE FENCE.  The engine defines `ShellKind::D3` and `C_3D`, and `md.rs`
implements l = 2.  No declared species uses any of it.  That is asserted here as
a property of the ENGINE's table, because if a d shell ever enters a species the
referee stops being a referee for it -- silently, since it would simply build a
smaller basis and report a higher energy that looks like a converged answer.
"""
import decimal
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import basis2                                            # noqa: E402

ENGINE = os.environ.get(
    "HOLON_ELEMENTS_RS",
    "/home/emoore/CIRISHolon/engine/crates/holon-chem/src/elements.rs")

FAILED = []
CHECKS = [0]


def check(cond, label):
    CHECKS[0] += 1
    if not cond:
        FAILED.append(label)
        print("  FAIL  %s" % label)
    else:
        print("  ok    %s" % label)


def D(x):
    return decimal.Decimal(str(x).strip())


# ---------------------------------------------------------------------------
# Parse the engine's declaration.
# ---------------------------------------------------------------------------
_ARR = r"\[\s*([-\d.eE+]+)\s*,\s*([-\d.eE+]+)\s*,\s*([-\d.eE+]+)\s*\]"


def parse_engine(src):
    """{Z: {'sym':.., 's1':(..), 'sp':(..)|None, 'sp2':.., 'sp3':..}} plus the
    C_* constants and the shell order each macro emits."""
    out = {}
    consts = {}
    for name in ("C_2S", "C_2P", "C_3S", "C_3P", "C_3D"):
        m = re.search(r"pub const %s: \[f64; 3\] = %s;" % (name, _ARR), src)
        if m:
            consts[name] = tuple(m.groups())
    # C_1S is `= H_COEFFS;` -- a reference, not a literal.  Record that it IS a
    # reference: a literal here would be the second transcription the crate's
    # own header says it refuses to make.
    consts["C_1S_is_reference"] = bool(
        re.search(r"pub const C_1S: \[f64; 3\] = H_COEFFS;", src))

    for m in re.finditer(
            r"first_row!\(\s*\w+,\s*\"(\w+)\",\s*(\d+),\s*([\d.]+),\s*\"([^\"]+)\",\s*"
            r"s1 = %s\s*(?:,\s*sp = %s\s*)?\);" % (_ARR, _ARR), src):
        g = m.groups()
        sym, Z = g[0], int(g[1])
        s1 = tuple(g[4:7])
        sp = tuple(g[7:10]) if g[7] is not None else None
        out[Z] = dict(sym=sym, s1=s1, sp=sp, row=1,
                      mass=g[2], isotope=g[3])

    for m in re.finditer(
            r"second_row!\(\s*\w+,\s*\"(\w+)\",\s*(\d+),\s*([\d.]+),\s*\"([^\"]+)\",\s*"
            r"s1 = %s\s*,\s*sp2 = %s\s*,\s*sp3 = %s\s*\);"
            % (_ARR, _ARR, _ARR), src):
        g = m.groups()
        out[int(g[1])] = dict(sym=g[0], s1=tuple(g[4:7]),
                              sp2=tuple(g[7:10]), sp3=tuple(g[10:13]), row=2,
                              mass=g[2], isotope=g[3])
    return out, consts


def parse_explicit_species(src):
    """Elements declared as explicit `Species { ... }` structs rather than
    through the row macros.

    The engine grew past argon while this referee was being written -- it now
    declares up to xenon, most of them with a `ShellKind::D3` shell.  Those are
    OUT OF SCOPE here and the point of parsing them is to say so with a number
    rather than to assume the file still stops where it did.
    """
    out = {}
    for m in re.finditer(
            r"^pub const (\w+): Species = Species \{(.*?)^\};",
            src, re.S | re.M):
        name, body = m.group(1), m.group(2)
        # A macro BODY also contains `Species = Species {`, with its `};`
        # indented -- so a span-matching regex swallows both macros and reports
        # their template shells as hydrogen's.  A macro body is recognisable by
        # its metavariables; a real declaration has none.
        if "$" in body or "macro_rules" in body:
            continue
        zm = re.search(r"\bz:\s*(\d+)", body)
        sm = re.search(r"symbol:\s*\"(\w+)\"", body)
        if not zm:
            continue
        kinds = re.findall(r"kind:\s*ShellKind::(\w+)", body)
        out[int(zm.group(1))] = dict(const=name,
                                     sym=sm.group(1) if sm else "?",
                                     kinds=kinds)
    return out


def engine_shells(rec, consts):
    """The (l, exponents, coefficients) sequence the engine's macro emits, in
    the order it emits it."""
    if rec["row"] == 1:
        sh = [(0, rec["s1"], "C_1S")]
        if rec["sp"] is not None:
            sh += [(0, rec["sp"], "C_2S"), (1, rec["sp"], "C_2P")]
        return sh
    return [(0, rec["s1"], "C_1S"),
            (0, rec["sp2"], "C_2S"), (1, rec["sp2"], "C_2P"),
            (0, rec["sp3"], "C_3S"), (1, rec["sp3"], "C_3P")]


def compare(engine, consts, table, label=""):
    """Returns a list of disagreement strings.  Empty means one model."""
    bad = []
    coeffs = {"C_1S": basis2.C_1S, "C_2S": basis2.C_2S, "C_2P": basis2.C_2P,
              "C_3S": basis2.C_3S, "C_3P": basis2.C_3P}
    # every element the ENGINE declares (H is declared in sto3g.rs, not here)
    for Z, rec in sorted(engine.items()):
        if Z not in table:
            bad.append("Z=%d (%s) is in the engine and not in the referee"
                       % (Z, rec["sym"]))
            continue
        want = engine_shells(rec, consts)
        got = table[Z]
        if len(want) != len(got):
            bad.append("Z=%d shell COUNT %d engine vs %d referee"
                       % (Z, len(want), len(got)))
            continue
        for i, ((lw, ew, cname), (lg, eg, cg)) in enumerate(zip(want, got)):
            if lw != lg:
                bad.append("Z=%d shell %d: l %d engine vs %d referee (ORDER)"
                           % (Z, i, lw, lg))
            if [D(x) for x in ew] != [D(x) for x in eg]:
                bad.append("Z=%d shell %d exponents: %s engine vs %s referee"
                           % (Z, i, list(ew), list(eg)))
            if [D(x) for x in coeffs[cname]] != [D(x) for x in cg]:
                bad.append("Z=%d shell %d coefficients (%s): %s vs %s"
                           % (Z, i, cname, list(coeffs[cname]), list(cg)))
    # and the other direction, minus hydrogen which the engine declares in
    # sto3g.rs by reference
    for Z in sorted(table):
        if Z == 1:
            continue
        if Z not in engine:
            bad.append("Z=%d is in the referee and not in the engine" % Z)
    return bad


# ---------------------------------------------------------------------------
def main():
    if not os.path.exists(ENGINE):
        print("engine source not found at %s -- set HOLON_ELEMENTS_RS" % ENGINE)
        return 2
    src = open(ENGINE).read()
    engine, consts = parse_engine(src)

    print("\n1. the engine's declaration, as parsed")
    check(len(engine) == 17,
          "17 elements parsed from elements.rs (He..Ar; H lives in sto3g.rs): "
          "got %d" % len(engine))
    check(sorted(engine) == list(range(2, 19)),
          "they are Z = 2..18 with no gaps")
    check(consts.get("C_1S_is_reference"),
          "the engine's C_1S is H_COEFFS by reference, not a second copy")
    for c in ("C_2S", "C_2P", "C_3S", "C_3P"):
        check(c in consts, "%s parsed" % c)

    print("\n2. exponents, coefficients and SHELL ORDER, both directions")
    bad = compare(engine, consts, basis2.STO3G_18)
    check(not bad, "the two declarations are one model over Z = 1..18"
          + ("" if not bad else "; %d disagreements: %s"
             % (len(bad), "; ".join(bad[:6]))))

    print("\n2b. masses, symbols and isotope labels for the second row")
    mbad = []
    for Z, rec in sorted(engine.items()):
        if Z < 11:
            continue
        if basis2.SYMBOL.get(Z) != rec["sym"]:
            mbad.append("Z=%d symbol %r vs %r"
                        % (Z, rec["sym"], basis2.SYMBOL.get(Z)))
        if D(basis2.ISOTOPE_MASS_U.get(Z, "0")) != D(rec["mass"]):
            mbad.append("Z=%d mass %s vs %s"
                        % (Z, rec["mass"], basis2.ISOTOPE_MASS_U.get(Z)))
        if basis2.ISOTOPE.get(Z) != rec["isotope"]:
            mbad.append("Z=%d isotope %r vs %r"
                        % (Z, rec["isotope"], basis2.ISOTOPE.get(Z)))
    check(not mbad, "symbol, isotope and mass agree for Na..Ar"
          + ("" if not mbad else "; %s" % "; ".join(mbad)))
    check(len(basis2.ISOTOPE_MASS_U) == 8,
          "eight second-row masses are declared (not one fewer, which a "
          "one-direction check would pass)")
    mutmass = dict(basis2.ISOTOPE_MASS_U)
    mutmass[17] = "34.968852683"
    check(D(mutmass[17]) != D(engine[17]["mass"]),
          "a last-digit change to chlorine's mass is a real difference (the "
          "mutation this check would have to catch)")

    print("\n3. the first row is shared, not re-transcribed")
    import elements_core as EC
    first = {Z: v for Z, v in basis2.STO3G_18.items() if Z <= 10}
    check(all(first[Z] is EC.STO3G_SHELLS[Z] for Z in first),
          "every first-row entry is the SAME OBJECT as ELEMENTS-1's, so the "
          "first row cannot drift between the campaigns")

    print("\n4. the fence: the second row carries no d shell, and the scope has an edge")
    explicit = parse_explicit_species(src)
    check(max(l for sh in basis2.STO3G_18.values() for (l, _, _) in sh) <= 1,
          "the referee's table has max l = 1")
    in_scope_kinds = []
    for Z, rec in explicit.items():
        if Z <= 18:
            in_scope_kinds.append((Z, rec["sym"], rec["kinds"]))
    check(not in_scope_kinds,
          "no element at or below argon is declared as an explicit struct; "
          "Z <= 18 is entirely the two row macros, which carry s and p only"
          + ("" if not in_scope_kinds else "; %r" % in_scope_kinds))
    d_in_scope = [Z for Z in engine if Z <= 18
                  and any(k.startswith("D") for k in
                          explicit.get(Z, {}).get("kinds", []))]
    check(not d_in_scope,
          "no in-scope element carries a d shell (%r)" % d_in_scope)

    # THE EDGE, STATED WITH A NUMBER RATHER THAN ASSUMED.
    #
    # `elements.rs` grew past argon DURING this lane's first pass: it now
    # declares elements up to xenon, most with a ShellKind::D3 shell, and md.rs
    # implements l = 2 for them.  None of that is wrong.  What matters here is
    # that the referee's declared model stops at argon, so for any species above
    # it there IS no referee -- and a coverage gate that reads "the engine's
    # curves match the referee" is VACUOUSLY true for a pair it cannot grade.
    above = sorted(Z for Z in explicit if Z > 18)
    print("     engine declares %d elements above argon (Z = %d..%d); the "
          "referee declares none of them" % (len(above), min(above),
                                             max(above)) if above else
          "     engine declares nothing above argon")
    check(all(Z not in basis2.STO3G_18 for Z in above),
          "the referee's table contains no element it cannot compute")
    # and the refusal is a REFUSAL, not a silently smaller basis
    fired = False
    try:
        basis2.shells_for(19)
    except KeyError as e:
        fired = "outside" in str(e) or "19" in str(e)
    except Exception:
        fired = False
    check(fired,
          "asking the referee for an out-of-scope element RAISES rather than "
          "returning a smaller basis that would look converged")

    print("\n5. the failing case, made to happen")
    # One digit, in the middle of one exponent, in the element the D1 bridge is
    # staked on.  A check that has never fired is indistinguishable from a check
    # that cannot.
    mutated = {Z: tuple(sh) for Z, sh in basis2.STO3G_18.items()}
    l, e, c = mutated[14][1]
    mutated[14] = (mutated[14][0], (l, (e[0], "5.38970687", e[2]), c)) \
        + mutated[14][2:]
    bad2 = compare(engine, consts, mutated)
    check(len(bad2) == 0,
          "control: the unmutated digit still agrees (this mutation was a "
          "no-op; see below)")
    l, e, c = basis2.STO3G_18[14][1]
    mut2 = dict(mutated)
    mut2[14] = (mut2[14][0], (l, (e[0], "5.38970680", e[2]), c)) \
        + basis2.STO3G_18[14][2:]
    bad3 = compare(engine, consts, mut2)
    check(len(bad3) == 1 and "Z=14 shell 1 exponents" in bad3[0],
          "one changed digit in silicon's 2s/2p exponent is caught, and named "
          "(%s)" % (bad3[0][:70] if bad3 else "NOT CAUGHT"))
    dropped = {Z: v for Z, v in basis2.STO3G_18.items() if Z != 17}
    bad4 = compare(engine, consts, dropped)
    check(any("Z=17" in b and "engine and not in the referee" in b
              for b in bad4),
          "a missing element is caught (the direction a one-way check misses)")
    extra = dict(basis2.STO3G_18)
    extra[19] = basis2.STO3G_18[18]
    bad5 = compare(engine, consts, extra)
    check(any("Z=19" in b for b in bad5),
          "an element the engine does not have is caught (the other direction)")
    reordered = dict(basis2.STO3G_18)
    s = list(basis2.STO3G_18[17])
    s[3], s[4] = s[4], s[3]                      # 3s and 3p swapped
    reordered[17] = tuple(s)
    bad6 = compare(engine, consts, reordered)
    check(any("ORDER" in b for b in bad6),
          "a swapped 3s/3p shell is caught as an ORDER fault -- the fault that "
          "leaves every total energy unchanged")

    print("\n6. what the fingerprints say")
    print("     MIXTURES-1  %s" % basis2.fingerprint())
    print("     ELEMENTS-1  %s" % basis2.elements1_fingerprint())
    check(basis2.fingerprint() != basis2.elements1_fingerprint(),
          "the two campaigns' tables fingerprint differently, so neither can "
          "read the other's cache")

    print("\n%d checks, %d FAIL" % (CHECKS[0], len(FAILED)))
    for f in FAILED:
        print("   FAILED: %s" % f)
    return 1 if FAILED else 0


if __name__ == "__main__":
    raise SystemExit(main())
