"""Run ELEMENTS-1's stage machinery on MIXTURES-1's species.

    python3 curves2.py --energies Cl2 HCl
    python3 curves2.py --stencil --hermite --spin --recertify S2
    python3 curves2.py --probe --assemble Cl2

Same stages, same flags, same guards as `elements1/build_curves.py`, because it
IS that file: this launcher only arranges the three things it reads from its
environment, then calls its `main()`.

THE THREE THINGS, and what each one would do if it were left wrong.

1. `sys.path` -- this directory ahead of `elements1/`, so `import species`
   resolves to the shim.  Wrong: the pipeline would run ELEMENTS-1's nine
   species instead of this campaign's eight, and succeed at it.

2. `build_curves.HERE` -- the module computes it from its own `__file__`, which
   is `elements1/`.  Every path it builds from that would land in the OTHER
   campaign's directory: it would read ELEMENTS-1's `elements_atoms.json` for
   asymptotes, take ELEMENTS-1's run locks, and OVERWRITE its
   `elements_potential_partial.json`.  Wrong here is not a wrong number, it is
   damage to a landed campaign, which is why it is the first thing asserted.

3. the table and cache bindings -- carried by importing the shim, which imports
   `m1core`.  See the note there.

THE ATOMS FILE'S NAME IS INHERITED, DELIBERATELY.  `build_curves.py` asks for
`elements_atoms.json` by a literal string.  Rather than edit that file, this
directory carries a symlink of that name pointing at `mixtures_atoms.json`,
which is the honest name and the one `build_atoms2.py` writes.  The link is
(re)made below, and the file it resolves to is checked to carry THIS campaign's
model string -- because a symlink to the wrong atoms file is exactly the kind of
mistake that produces a plausible curve with a wrong asymptote.
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
_E1 = os.path.join(HERE, "elements1")
sys.path.insert(0, HERE)
if _E1 not in sys.path:
    sys.path.append(_E1)

import species as SP                # the shim; imports m1core, binds the table
import m1core as M                  # noqa: E402
import build_curves as B            # noqa: E402

ATOMS_REAL = "mixtures_atoms.json"
ATOMS_LINK = "elements_atoms.json"
MODEL = "MIXTURES1/STO-3G/FCI"


def _link_atoms():
    real = os.path.join(HERE, ATOMS_REAL)
    link = os.path.join(HERE, ATOMS_LINK)
    if os.path.islink(link) or os.path.exists(link):
        if os.path.islink(link) and os.readlink(link) == ATOMS_REAL:
            pass
        else:
            os.remove(link)
            os.symlink(ATOMS_REAL, link)
    else:
        os.symlink(ATOMS_REAL, link)
    if not os.path.exists(real):
        # A dangling link is fine for the stages that never read it, and a
        # FileNotFoundError from inside build_curves' assemble is not -- it
        # names the LINK, in a directory where no such file was ever meant to
        # exist, which reads like a broken install rather than a missing input.
        needs = {"--assemble", "--minima"}
        if not sys.argv[1:] or needs & set(sys.argv[1:]):
            raise SystemExit(
                "MIXTURES-1: %s does not exist yet, and the stage you asked "
                "for reads it for the separated-atom energies.  Run\n"
                "    python3 build_atoms2.py\n"
                "first (gate R1)." % ATOMS_REAL)
        return "not built yet (not needed by these stages)"
    with open(real) as f:
        got = json.load(f).get("model")
    if got != MODEL:
        raise RuntimeError(
            "%s carries model %r, not %r.  The asymptotes an assemble reads "
            "come from this file, so the wrong one produces a plausible curve "
            "with a wrong dissociation limit." % (ATOMS_REAL, got, MODEL))
    return "ok"


def bind():
    """Point build_curves at THIS directory, and prove it moved."""
    was = B.HERE
    B.HERE = HERE
    problems = []
    if os.path.abspath(B.HERE) != os.path.abspath(HERE):
        problems.append("build_curves.HERE did not move")
    if os.path.abspath(was) == os.path.abspath(HERE):
        problems.append(
            "build_curves.HERE was ALREADY this directory, so the rebinding "
            "proves nothing -- this check has gone vacuous")
    if os.path.abspath(M.CACHE) == os.path.abspath(os.path.join(_E1, "cache")):
        problems.append("the cache is still ELEMENTS-1's")
    if SP.__file__ is None or os.path.dirname(
            os.path.abspath(SP.__file__)) != os.path.abspath(HERE):
        problems.append("`import species` resolved to %r, not the shim"
                        % getattr(SP, "__file__", None))
    if set(SP.DIATOMICS) & {"H2", "N2", "CO", "F2", "Li2", "LiH", "HF",
                            "He2", "Ne2"}:
        problems.append("the species set overlaps ELEMENTS-1's")
    if problems:
        raise RuntimeError("MIXTURES-1 launcher binding is wrong: "
                           + "; ".join(problems))
    return was


POTENTIAL = "elements_potential_partial.json"
POTENTIAL_FULL = "elements_potential.json"


def restamp_model():
    """Put THIS campaign's model on the file the assembler just wrote.

    `build_curves.py` writes `model="ELEMENTS1/STO-3G/FCI"` as a literal, and
    the file's NAME is a literal too -- both were written when there was one
    campaign.  The name is harmless and inherited (it sits in this directory and
    nothing else reads it), but the model string is not: it travels into
    anything that reads the assembled file and would label second-row curves as
    ELEMENTS-1's.

    So it is corrected here, after the assemble, rather than by editing shared
    code that a live campaign is running through.  The correction reports
    itself, and refuses a file that carries neither campaign's name -- because
    the one thing worse than a wrong label is a silently rewritten one.
    """
    out = []
    for fn in (POTENTIAL, POTENTIAL_FULL):
        p = os.path.join(HERE, fn)
        if not os.path.exists(p):
            continue
        with open(p) as f:
            obj = json.load(f)
        got = obj.get("model")
        if got == MODEL:
            continue
        if got != "ELEMENTS1/STO-3G/FCI":
            raise RuntimeError(
                "%s carries model %r, which is neither this campaign's nor the "
                "one the shared assembler stamps. Refusing to relabel it."
                % (fn, got))
        obj["model"] = MODEL
        obj["model_restamped_from"] = got
        obj["model_restamp_note"] = (
            "build_curves.py stamps ELEMENTS-1's model as a literal; this file "
            "was produced by that code running on MIXTURES-1's species, table "
            "and cache. The label is corrected here, the original is kept.")
        with open(p, "w") as f:
            json.dump(obj, f, indent=1)
        out.append(fn)
    return out


if __name__ == "__main__":
    was = bind()
    atoms = _link_atoms()
    print("MIXTURES-1 curves")
    print("  species      %s" % " ".join(sorted(SP.DIATOMICS)))
    print("  table        %s (Z = 1..%d)" % (M.FINGERPRINT, max(M.TABLE)))
    print("  cache        %s" % os.path.relpath(M.CACHE, HERE))
    print("  build_curves %s  ->  %s"
          % (os.path.relpath(was, HERE), "."))
    print("  atoms        %s -> %s  [%s]" % (ATOMS_LINK, ATOMS_REAL, atoms))
    print(flush=True)
    B.main()
    for fn in restamp_model():
        print("  restamped %s: model -> %s" % (fn, MODEL))
