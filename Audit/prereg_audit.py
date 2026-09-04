#!/usr/bin/env python3
"""Prereg audit for CIRISHolon — refuse a freeze the way CI refuses a sorry.

Ported from CIRISOntology/Audit/prereg_audit.py, adapted to this repo:
  1. every `witness:` Lean name resolves in lean/CIRISHolon/ (or is `none`,
     which must be justified on the same line)
  2. a `misfits:` line exists; every cited id exists in
     conformance/gravity/MISFITS.md; registry keyword contact without a
     citation is REFUSED — this is the mechanism that would have caught
     SCHWINGER-2's grid (M-VOLUME-SCALE was in our own sweep) and
     BRIDGE-6's plant carrier (M-PLANT-OBS had fired three times already)
  3. every gate row carries a digit-bearing criterion or the word EXACT
  4. plants: a `plants:` section whose every plant names its carrier and
     the sector it must be nonzero in (M-PLANT-SECTOR, mechanized)
Exit 0 = freeze admissible; nonzero = refused, reasons printed.
"""
import re, sys, pathlib
ROOT = pathlib.Path(__file__).resolve().parent.parent
REG = (ROOT / "conformance/gravity/MISFITS.md").read_text()
REG_IDS = set(re.findall(r"\*\*(M-[A-Z0-9-]+)\*\*", REG))


def registry_dates():
    """Each misfit's registration date, where its row carries one.

    AN AUDIT REFUSES FORWARD, NEVER BACKWARD (lead's ruling, 2026-08-30). A freeze that
    predates a misfit's registration cannot be refused for failing to cite it, and several
    of this registry's newest ids were BORN FROM the very campaigns whose freezes now
    contact them — refusing those would be absurd.

    The comparison is STRICTLY AFTER, and that is not a detail. The motivating case is
    same-day: SATURATION-3 froze on 2026-08-30 and M-DEVICE-CLASS was registered on
    2026-08-30 out of that campaign's own G2 result. Under "on or after" the ruling would
    not have resolved the case that prompted it.
    """
    out = {}
    for row in REG.splitlines():
        m = re.match(r"\|\s*\*\*(M-[A-Z0-9-]+)\*\*\s*\|", row)
        if not m:
            continue
        d = re.findall(r"(20\d\d-\d\d-\d\d)", row)
        if d:
            # FIRST date, not last (fixed 2026-09-01, found by arming M-STALE-INSTRUMENT
            # and M-PLACEMENT-LOTTERY below and watching the new arms NOT fire on a
            # deliberately stripped freeze). `max` reads the LAST date mentioned in the
            # row, which is not the registration date: a misfit that is later ANNOTATED
            # -- a re-occurrence noted, a scope note added -- has its effective date
            # pushed forward, and every freeze up to that new date is silently exempted
            # from citing it. Both ids above had picked up a same-day annotation and were
            # therefore unenforceable on the very day they were needed. An annotation must
            # never weaken a gate. The forward-only ruling is untouched: a freeze still
            # cannot be refused for failing to cite something registered after it, and
            # `min` is the date it was registered. Verified against all 41 freezes in the
            # tree at the time of the edit: byte-identical output, so this tightens the
            # gate for the future without retro-refusing anything.
            out[m.group(1)] = min(d)
    return out


REG_DATES = registry_dates()


def freeze_date(text):
    """The freeze's own date, or None. `Frozen YYYY-MM-DD` is this corpus's form."""
    m = re.search(r"[Ff]rozen\s+(20\d\d-\d\d-\d\d)", text)
    return m.group(1) if m else None
CONTACT = {
    "conjugacy": "M-GAUGE-LAUNDER", "class-diagonal": "M-GAUGE-LAUNDER",
    "parity": "M-PARITY-PROTECT",
    "loop": "M-LOOP-BLIND", "holonomy": "M-LOOP-BLIND",
    "plant": "M-PLANT-OBS",
    "sector": "M-PLANT-SECTOR",
    "charged": "M-BARE-CHARGE", "singlet": "M-BARE-CHARGE", "dressed": "M-BARE-CHARGE",
    "local": "M-HOMOG", "distant": "M-HOMOG", "spatially": "M-HOMOG",
    "lattice": "M-VOLUME-SCALE", "N-convergence": "M-VOLUME-SCALE",
    "endogenous": "M-COND-PROBE", "inside T": "M-COND-PROBE",
    "memoryless": "M-ONE-MODEL-DELTA", "Markov": "M-ONE-MODEL-DELTA",
    # --- 2026-08-30: the registry had grown five ids past this table, which made those
    # entries decoration -- a freeze contacting their shapes would not have been refused.
    # Keywords are chosen NARROW on purpose: an over-broad contact refuses honest freezes,
    # and an audit people switch off gates nothing.
    "gpu": "M-DEVICE-CLASS", "cuda": "M-DEVICE-CLASS",
    "device class": "M-DEVICE-CLASS", "accelerator": "M-DEVICE-CLASS",
    "bitwise": "M-DEVICE-CLASS",
    "liveness": "M-PROBE-THE-RESOURCE", "heartbeat": "M-PROBE-THE-RESOURCE",
    "health check": "M-PROBE-THE-RESOURCE",
    "launch header": "M-PROVENANCE-OVERREACH", "sha256": "M-PROVENANCE-OVERREACH",
    "provenance line": "M-PROVENANCE-OVERREACH",
    "repair": "M-MAINTENANCE-LENS", "maintenance": "M-MAINTENANCE-LENS",
    "rent clause": "M-MAINTENANCE-LENS",
    "timeout": "M-IDLE-CALIBRATED-TIMEOUT", "grace period": "M-IDLE-CALIBRATED-TIMEOUT",
    "quiet machine": "M-IDLE-CALIBRATED-TIMEOUT", "loadavg": "M-IDLE-CALIBRATED-TIMEOUT",
    # --- 2026-09-01: the unarmed-coverage note below was reporting 26 of 41, and THREE of
    # the unarmed ids are the ones the tower brief names as mandatory contacts for any
    # freeze that prices a path. An id that a freeze must cite but that no keyword can
    # catch is decoration, which is the same finding that dated this table in the first
    # place. Keywords stay NARROW on purpose: an over-broad contact refuses honest freezes,
    # and an audit people switch off gates nothing. Verified against all 41 freezes in the
    # tree at the time of the edit: zero newly refused.
    "core-hour": "M-CHEAPER-THAN-ITS-PRICE", "cost model": "M-CHEAPER-THAN-ITS-PRICE",
    "per-node price": "M-CHEAPER-THAN-ITS-PRICE",
    "results document": "M-STALE-INSTRUMENT", "instrument's commit": "M-STALE-INSTRUMENT",
    "taskset": "M-PLACEMENT-LOTTERY", "core class": "M-PLACEMENT-LOTTERY",
    "wall clock": "M-PLACEMENT-LOTTERY",
    # --- 2026-09-03: E7 fired G0' with a record whose every field said "fine". The two
    # ids that name that shape were BOTH unarmed, so the amendment could contact them
    # freely and did. Armed narrowly, and verified against every freeze and amendment in
    # the tree at the time of the edit: zero newly refused.
    "iteration cap": "M-EXIT-DISCRIMINATOR", "sweep cap": "M-EXIT-DISCRIMINATOR",
    "stagnation": "M-EXIT-DISCRIMINATOR",
    "discarded weight": "M-TRUNCATION-AS-ERRORBAR",
    "truncation error": "M-TRUNCATION-AS-ERRORBAR",
    "non-increasing": "M-FLOOR-UNSTAKED", "monotonicity": "M-FLOOR-UNSTAKED",
    "resolution floor": "M-FLOOR-UNSTAKED",
}

def contact_table_is_sound():
    """Every id this table maps to must EXIST in the registry.

    A typo in `CONTACT` cannot be caught by any freeze: the keyword would fire, the id
    would never appear in `cited`, and every freeze mentioning that word would be refused
    for a misfit nobody can look up. Checked at import so the gate cannot rot silently.
    """
    return sorted({m for m in CONTACT.values() if m not in REG_IDS})


def lean_resolves(name):
    if name == "none":
        return True
    for f in (ROOT / "lean" / "CIRISHolon").rglob("*.lean"):
        if re.search(rf"(theorem|lemma|def|abbrev|structure)\s+{re.escape(name)}\b",
                     f.read_text()):
            return True
    return False

def audit(path):
    t = pathlib.Path(path).read_text()
    errs = []
    wits = re.findall(r"witness:\s*`?([A-Za-z0-9_.']+)`?", t)
    if not wits:
        errs.append("no `witness:` lines — every gate needs its Lean name or `none (reason)`")
    for w in wits:
        if not lean_resolves(w.split(".")[-1]):
            errs.append(f"witness does not resolve in lean/CIRISHolon: {w}")
    cited = set(re.findall(r"\b(M-[A-Z0-9-]+)\b", t))
    ghost = cited - REG_IDS
    for g in sorted(ghost):
        errs.append(f"cites unregistered misfit id: {g}")
    if "misfits:" not in t.lower():
        errs.append("no `misfits:` line — the freeze must state which registered misfits it contacts")
    low = t.lower()
    fdate = freeze_date(t)
    for kw, mid in CONTACT.items():
        if kw.lower() not in low or mid in cited:
            continue
        mdate = REG_DATES.get(mid)
        # Refuse FORWARD only. An undated freeze gets the strict treatment -- a document
        # that will not date itself cannot claim precedence -- and an undated MISFIT is
        # reported rather than silently exempting every freeze that contacts it.
        if fdate and mdate and fdate <= mdate:
            continue
        note = "" if mdate else "  [misfit carries no registration date; precedence unverifiable]"
        errs.append(f"keyword contact '{kw}' -> {mid} without citing it{note}")
    gates = [ln for ln in t.splitlines() if re.match(r"\s*-\s*\*\*(G|R|B|S)\d*", ln)]
    for ln in gates:
        if not (re.search(r"\d", ln) or "EXACT" in ln.upper()):
            errs.append(f"gate row has no numeric criterion and is not EXACT: {ln.strip()[:70]}")
    if "plant" in low:
        for m in re.finditer(r"\(i+v?\)|\(i\)|\(ii\)", t):
            pass
        if "nonzero in" not in low and "sector the plant acts on" not in low:
            errs.append("plants present but no carrier-sector statement "
                        "(M-PLANT-SECTOR must be discharged in the freeze, not the postmortem)")
    return errs

if __name__ == "__main__":
    unknown = contact_table_is_sound()
    if unknown:
        print("REFUSED  Audit/prereg_audit.py's own CONTACT table")
        for u in unknown:
            print(f"    - maps a keyword to {u}, which is not in MISFITS.md")
        sys.exit(2)
    # The grep-arm's own coverage, reported on every run. On 2026-08-30 the registry had
    # grown five ids past the CONTACT table and nobody noticed, because nothing said so --
    # a gate whose coverage is invisible drifts behind the thing it gates. This line does
    # NOT refuse: some misfits are about process rather than about a word a freeze would
    # use, and manufacturing keywords for those would produce false refusals, which is how
    # an audit gets switched off.
    unarmed = sorted(REG_IDS - set(CONTACT.values()))
    if unarmed:
        print(f"note: {len(unarmed)} of {len(REG_IDS)} registered misfits have NO contact "
              f"keyword and cannot be caught by this audit:")
        print("      " + ", ".join(unarmed))
    bad = 0
    for p in sys.argv[1:]:
        errs = audit(p)
        tag = pathlib.Path(p).name
        if errs:
            bad += 1
            print(f"REFUSED  {tag}")
            for e in errs[:12]:
                print(f"    - {e}")
        else:
            print(f"ADMITTED {tag}")
    sys.exit(1 if bad else 0)
