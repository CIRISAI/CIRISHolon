#!/usr/bin/env python3
"""Pre-review check for a DRAFT preregistration: what a machine can refuse before a reviewer is asked.

Why this exists. On 2026-11-23 three drafts went to adversarial review and all three came
back REDESIGN. Most of what the reviewers found needed a reader. Some of it was arithmetic
the draft could have carried and a program could have checked:

  * a bar sitting beside a cell of the draft's own table (ANOMALY-1: a kill at 0.22 beside
    0.225, a kill at 0.16 beside 0.171), or a reading that lands astride two bars
    (SEAM-NS-0: 0.15-0.21 against 0.2)                       -> M-SORTS-NOT-SEPARATES
  * a plant that, computed from its own specification, lands on the wrong side of the bar
    it must clear (SEAM-NS-0's P2: 0.14 and 0.25 against 0.3) -> M-PLANT-OBS
  * a reserved region that contains the campaign's own points once a lattice shift is
    applied (ANOMALY-1)                       -> M-RESERVE-BREACHED, M-NAMED-POINT-SHIFTED
  * a number cited to a file that does not hold it            -> M-STALE-INSTRUMENT

This tool checks ONLY those. It does not replace the review and it does not replace
`prereg_audit.py` (which checks words: witnesses, misfit citations, gate rows). A draft
that passes here has shown its arithmetic is self-consistent, not that it is right.

THE CONVENTION. The draft carries one or more fenced blocks whose info string is
`prereview`, each a JSON object or a list of them. Prose is not parsed for meaning. Every
object has a "kind":

  stake      {"kind": "stake", "id": "S2", "quantity": "D_view",
              "margin": 0.03, "margin_reason": "a static misreading moves it by 0.03 (§6)",
              "regions": [{"name": "MET", "hi": 0.07}, {"name": "KILL", "lo": 0.16}],
              "separates": [["closed", "open"]],
              "table": [{"hypothesis": "closed", "label": "one mode", "reads": 0.000, "expect": "MET"},
                        {"hypothesis": "open", "label": "phi 0.30", "reads": [0.24, 0.26], "expect": "KILL"}]}
             A region is a closed interval; omit "lo" or "hi" for a half-line. Whatever no
             region covers is called BETWEEN. "reads" is a number or a [low, high] range.
             "separates" names the pairs of hypotheses the stake exists to tell apart.
  plant      {"kind": "plant", "id": "P2", "stake": "S2", "lands": 0.253, "must": "KILL"}
             "must" is a region name of that stake, or an interval {"lo": .., "hi": ..}.
             "lands" is the plant's value ON PAPER, from its own specification.
  branches   {"kind": "branches",
              "outcomes": {"gate": ["pass", "fail"], "S2": ["MET", "BETWEEN", "KILL"]},
              "branches": [{"id": "e", "when": {"gate": ["fail"]}},
                           {"id": "a", "when": {"gate": ["pass"], "S2": ["MET"]}}, ...]}
             A branch matches an outcome when every key of its "when" allows it; a missing
             key allows anything. "first_match": true grades an ORDERED list (the first
             matching branch is the one entered); without it, a doubly-matched outcome is
             refused.
  numbers    {"kind": "numbers", "cited": [{"value": "0.180", "source": "conformance/x/Y.md"},
                                           {"value": "1,500 m/s", "source": "literature: Sette et al. 1996"}]}
             A source is a path in the tree (the value, or "find" if given, must occur in
             that file verbatim), or starts with "literature:", "textbook:" or "derived:".
  new_files  {"kind": "new_files", "paths": ["conformance/x/instrument.py"]}
             Paths the draft names that do not exist yet.
  reserve    {"kind": "reserve", "id": "rung-IV",
              "region": {"x": {"lo": 1.5, "hi": 25}, "mass": {"lo": 0, "lo_open": true}},
              "meaning": {"x": "bare", "mass": "shifted"},
              "shifts": {"mass": "mass + 1/(8*sqrt(x))"},
              "known_inside": "first-order theory fixes the small-mass surface"}
             "shifts" maps a declared (bare) point to the point the instrument is really
             at. With no shift, write "shifts": {} and "no_shift_reason".
  points     {"kind": "points", "of": "campaign", "label": "S1",
              "points": [{"x": 4, "mass": 0}, {"x": 9, "mass": 0}]}
             Every point the campaign, its gates, plants and pilots will compute.

WHAT IT CHECKS
  (a) every stake has a table of at least two hypotheses; every region edge is at least the
      stated margin from every cell; every row lands where it says; no region holds rows of
      two hypotheses the stake is meant to separate; every region is reached by some row.
  (b) every stake has a plant; every plant's paper value lies in the region it must reach,
      at least the margin from its edges.
  (c) every outcome of the declared outcome space enters exactly one branch.
  (d) every path the draft names in backticks exists in the tree (or is declared new), and
      every cited number occurs in the file it cites. Numbers with a unit that sit in prose
      with no path nearby and no entry in a `numbers` block are COUNTED and listed, and
      refused only under --strict-numbers.
  (e) if the draft reserves a region, every declared point, bare and shifted, lies outside it.

WHAT IT CANNOT CHECK, said here so that a pass is not read as more than it is
  - that the table's numbers are right, or that its hypotheses are the right ones;
  - that the declared outcome space is the real one, or that the prose branches say what
    the block says;
  - that a cited number MEANS in its file what the draft says it means, or was measured
    in the regime the draft spends it in (M-IDLE-CALIBRATED-TIMEOUT stays a reader's job);
  - that the declared points are all the points, or that closed-form theory does not
    already fix the reserved region;
  - anything about prior art, blindness of the author, or whether the stake is forced.

Exit 0 = may go to a reviewer; nonzero = refused, reasons printed.
Usage: python3 Audit/prereview_check.py [--root DIR] [--strict-numbers] DRAFT.md [...]
Tests: python3 Audit/test_prereview_check.py
"""
import ast
import itertools
import json
import math
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BLOCK = re.compile(r"```prereview[ \t]*\n(.*?)\n```", re.S)
ANY_FENCE = re.compile(r"```.*?```", re.S)
PATH_TOKEN = re.compile(r"`([A-Za-z0-9_][A-Za-z0-9_./-]*\.(?:md|py|rs|json|jsonl|log|txt|tsv|sh|lean|toml))`")
UNIT_NUMBER = re.compile(
    r"(?<![A-Za-z0-9_.])(\d[\d,]*(?:\.\d+)?(?:e-?\d+)?)\s?"
    r"(m/s|fs|ps|ns|Å|bohr|K|mHa|Ha|hartree|dex|%|Pa s|kT|GiB|GB|MB|nm⁻¹|Å⁻¹|wall-s|core-s|shots/s)(?![A-Za-z])")
RESERVE_WORDS = ("stay unread", "reserved region", "unread region")
MAX_OUTCOMES = 200_000


def blocks(text):
    """Every object in every `prereview` block, and the errors met while reading them."""
    out, errs = [], []
    for n, m in enumerate(BLOCK.finditer(text), 1):
        try:
            obj = json.loads(m.group(1))
        except json.JSONDecodeError as e:
            errs.append(f"prereview block {n} is not JSON: {e.msg} at line {e.lineno}")
            continue
        for o in (obj if isinstance(obj, list) else [obj]):
            if not isinstance(o, dict) or "kind" not in o:
                errs.append(f"prereview block {n}: an entry has no \"kind\"")
            else:
                out.append(o)
    return out, errs


def span(cell):
    """A cell as (low, high); a number is a degenerate range."""
    if isinstance(cell, (int, float)) and not isinstance(cell, bool):
        return float(cell), float(cell)
    if (isinstance(cell, list) and len(cell) == 2
            and all(isinstance(c, (int, float)) and not isinstance(c, bool) for c in cell)):
        return float(min(cell)), float(max(cell))
    raise ValueError(f"not a number or a [low, high] range: {cell!r}")


def inside(v, region):
    return region.get("lo", -math.inf) <= v <= region.get("hi", math.inf)


def landing(cell, regions):
    """The region a cell lands in: a name, 'BETWEEN', or 'STRADDLES a/b' when its two ends disagree."""
    lo, hi = span(cell)

    def one(v):
        hit = [r["name"] for r in regions if inside(v, r)]
        return "+".join(hit) if hit else "BETWEEN"
    a, b = one(lo), one(hi)
    if a != b:
        return f"STRADDLES {a}/{b}"
    # both ends agree; a range can still jump OVER a narrow region
    for r in regions:
        for e in (r.get("lo"), r.get("hi")):
            if e is not None and lo < e < hi:
                return f"STRADDLES {a}/{r['name']}"
    return a


def touches(cell, regions):
    """Every region (and BETWEEN) that some part of a cell's range falls in."""
    lo, hi = span(cell)
    probe = {lo, hi}
    for r in regions:
        for e in (r.get("lo"), r.get("hi")):
            if e is not None and lo < e < hi:
                step = 1e-9 * max(1.0, abs(e))
                probe.update((e - step, e, e + step))
    out = set()
    for v in probe:
        hit = [r["name"] for r in regions if inside(v, r)]
        out.update(hit or ["BETWEEN"])
    return out


def edge_distance(cell, edge):
    lo, hi = span(cell)
    return 0.0 if lo <= edge <= hi else min(abs(edge - lo), abs(edge - hi))


def check_stakes(objs):
    errs = []
    stakes = {o.get("id"): o for o in objs if o["kind"] == "stake"}
    plants = [o for o in objs if o["kind"] == "plant"]
    if not stakes:
        errs.append("no stake is declared in a prereview block (check a): the draft carries no table "
                    "of what its graded quantity reads under each hypothesis [M-SORTS-NOT-SEPARATES]")
    for sid, s in stakes.items():
        tag = f"stake {sid}"
        regions, table = s.get("regions") or [], s.get("table") or []
        margin = s.get("margin")
        if not isinstance(margin, (int, float)) or isinstance(margin, bool) or margin <= 0:
            errs.append(f"{tag}: no positive \"margin\" (how far a bar must sit from every cell)")
            continue
        if not str(s.get("margin_reason", "")).strip():
            errs.append(f"{tag}: the margin {margin} carries no \"margin_reason\"; a margin is the reading's "
                        f"own uncertainty, stated, not a number chosen to pass")
        if not regions or any("name" not in r or ("lo" not in r and "hi" not in r) for r in regions):
            errs.append(f"{tag}: \"regions\" must each have a name and at least one of lo, hi")
            continue
        hyps = {r.get("hypothesis") for r in table}
        if len(table) < 2 or len(hyps) < 2:
            errs.append(f"{tag}: the separation table needs at least two rows under two different hypotheses "
                        f"(it has {len(table)} rows, {len(hyps)} hypotheses) [M-SORTS-NOT-SEPARATES]")
            continue
        bad_cell = False
        for r in table:
            try:
                span(r.get("reads"))
            except ValueError as e:
                errs.append(f"{tag}: row {r.get('label')!r}: {e}")
                bad_cell = True
        if bad_cell:
            continue
        edges = [(r["name"], k, r[k]) for r in regions for k in ("lo", "hi") if k in r]
        for name, k, e in edges:
            for r in table:
                d = edge_distance(r["reads"], e)
                if d < margin:
                    how = "contains it" if d == 0 else f"is {d:.4g} away"
                    errs.append(f"{tag}: the bar {name} {k} = {e} is within the margin {margin} of the cell "
                                f"{r.get('label')!r} = {r['reads']} ({how}); the branch would be decided by "
                                f"the reading's own uncertainty [M-SORTS-NOT-SEPARATES]")
        where, reach = {}, {}
        for r in table:
            land = landing(r["reads"], regions)
            where.setdefault(land, set()).add(r.get("hypothesis"))
            for name in touches(r["reads"], regions):
                reach.setdefault(name, set()).add(r.get("hypothesis"))
            if r.get("expect") is None:
                errs.append(f"{tag}: row {r.get('label')!r} does not say where it is expected to land")
            elif r["expect"] != land:
                errs.append(f"{tag}: row {r.get('label')!r} reads {r['reads']} and lands in {land}, "
                            f"not in {r['expect']} as the draft says")
        pairs = s.get("separates")
        if not pairs:
            errs.append(f"{tag}: no \"separates\" pairs; the draft must name the hypotheses this stake exists "
                        f"to tell apart [M-SORTS-NOT-SEPARATES]")
        for pair in pairs or []:
            a, b = pair
            for missing in (h for h in (a, b) if h not in hyps):
                errs.append(f"{tag}: \"separates\" names hypothesis {missing!r}, which has no row in the table")
            for land, hs in sorted(reach.items()):
                if a in hs and b in hs:
                    errs.append(f"{tag}: rows of {a!r} and of {b!r} both reach {land}; the quantity does not "
                                f"separate the two readings it exists to separate [M-SORTS-NOT-SEPARATES]")
        for r in regions:
            if not any(land == r["name"] for land in where):
                errs.append(f"{tag}: no row of the table lands in {r['name']}; a region nothing reaches is a "
                            f"branch that cannot be entered [M-JOINT-PASS-REGION]")
        mine = [p for p in plants if p.get("stake") == sid]
        if not mine:
            errs.append(f"{tag}: no plant is declared for it (check b) [M-PLANT-OBS]")
        for p in mine:
            ptag = f"plant {p.get('id')} on {sid}"
            must = p.get("must")
            region = next((r for r in regions if r["name"] == must), None) if isinstance(must, str) else must
            if not isinstance(region, dict) or ("lo" not in region and "hi" not in region):
                errs.append(f"{ptag}: \"must\" is neither a region of the stake nor an interval")
                continue
            try:
                lo, hi = span(p.get("lands"))
            except ValueError as e:
                errs.append(f"{ptag}: no paper landing value ({e}); a plant's value is arithmetic on its own "
                            f"specification and is owed before the run [M-PLANT-OBS]")
                continue
            pm = p.get("margin", margin)
            ok = all(inside(v, region) for v in (lo, hi))
            near = min([edge_distance(p["lands"], region[k]) for k in ("lo", "hi") if k in region])
            if not ok:
                errs.append(f"{ptag}: ON PAPER it lands at {p['lands']}, outside the region it must reach "
                            f"({ {k: region[k] for k in ('lo', 'hi') if k in region} }); by the house rule a "
                            f"failed plant voids the campaign before any read [M-PLANT-OBS]")
            elif near < pm:
                errs.append(f"{ptag}: it lands at {p['lands']}, only {near:.4g} within its bar (margin {pm}); "
                            f"a plant that close fires or not by noise [M-PLANT-OBS]")
    for p in plants:
        if p.get("stake") not in stakes:
            errs.append(f"plant {p.get('id')}: names stake {p.get('stake')!r}, which is not declared")
    return errs


def check_branches(objs):
    errs, notes = [], []
    decl = [o for o in objs if o["kind"] == "branches"]
    if not decl:
        errs.append("no branches block (check c): the outcome space and the branch entered for each outcome "
                    "are not declared, so 'exactly one branch per outcome' cannot be checked")
    for d in decl:
        outcomes, brs = d.get("outcomes") or {}, d.get("branches") or []
        if not outcomes or not brs:
            errs.append("branches block: needs \"outcomes\" and \"branches\"")
            continue
        size = math.prod(len(v) for v in outcomes.values())
        if size > MAX_OUTCOMES:
            errs.append(f"branches block: {size} outcomes is more than this check enumerates ({MAX_OUTCOMES})")
            continue
        for b in brs:
            for k, allowed in (b.get("when") or {}).items():
                if k not in outcomes:
                    errs.append(f"branch {b.get('id')}: conditions on {k!r}, which is not in the outcome space")
                else:
                    for v in allowed:
                        if v not in outcomes[k]:
                            errs.append(f"branch {b.get('id')}: allows {k} = {v!r}, which is not an outcome of {k}")
        keys = list(outcomes)
        none, many, used = [], [], set()
        for combo in itertools.product(*(outcomes[k] for k in keys)):
            got = dict(zip(keys, combo))
            hit = [b.get("id") for b in brs
                   if all(got[k] in allowed for k, allowed in (b.get("when") or {}).items() if k in got)]
            if d.get("first_match"):
                hit = hit[:1]
            used.update(hit)
            (none if not hit else many if len(hit) > 1 else []).append((got, hit))
        for got, _ in none[:4]:
            errs.append(f"branches: the outcome {got} enters NO branch")
        if len(none) > 4:
            errs.append(f"branches: {len(none)} outcomes in all enter no branch")
        for got, hit in many[:4]:
            errs.append(f"branches: the outcome {got} enters {len(hit)} branches ({', '.join(map(str, hit))})")
        if len(many) > 4:
            errs.append(f"branches: {len(many)} outcomes in all enter more than one branch")
        for b in brs:
            if b.get("id") not in used:
                errs.append(f"branch {b.get('id')}: no outcome of the declared space enters it")
        notes.append(f"branches: {size} outcomes enumerated over {len(keys)} readings; the outcome space is the "
                     f"draft's own declaration and was not checked against its prose")
    return errs, notes


def tracked(root):
    try:
        out = subprocess.run(["git", "-C", str(root), "ls-files"], capture_output=True, text=True, timeout=60)
        files = out.stdout.split("\n") if out.returncode == 0 else []
    except (OSError, subprocess.SubprocessError):
        files = []
    files = [f for f in files if f]
    if not files:  # not a git tree (the tests' temporary directories): walk it
        files = [str(p.relative_to(root)) for p in root.rglob("*") if p.is_file()]
    return files


def resolver(root, draft):
    files = tracked(root)
    names = {}
    for f in files:
        names.setdefault(f.rsplit("/", 1)[-1], []).append(f)

    def resolve(tok):
        for base in (root, draft.parent):
            if (base / tok).is_file():
                return base / tok
        if "/" in tok:
            hit = [f for f in files if f.endswith("/" + tok) or f == tok]
        else:
            hit = names.get(tok, [])
        return (root / hit[0]) if hit else None
    return resolve


def check_numbers(objs, text, root, draft, strict):
    errs, notes = [], []
    resolve = resolver(root, draft)
    new = {p for o in objs if o["kind"] == "new_files" for p in o.get("paths", [])}
    prose = BLOCK.sub("", text)
    missing = sorted({t for t in PATH_TOKEN.findall(prose) if t not in new and resolve(t) is None})
    for t in missing:
        errs.append(f"the draft names `{t}`, which is not in the tree and is not declared in a new_files block "
                    f"(check d) [M-STALE-INSTRUMENT]")
    cited = [c for o in objs if o["kind"] == "numbers" for c in o.get("cited", [])]
    known = set()
    for c in cited:
        val, src = str(c.get("value", "")), str(c.get("source", ""))
        known.add(val)
        known.add(str(c.get("find", val)))
        if not val or not src:
            errs.append(f"numbers: an entry lacks a value or a source: {c}")
            continue
        head = src.split(":", 1)[0]
        if head in ("literature", "textbook", "derived"):
            if not src.split(":", 1)[1].strip():
                errs.append(f"numbers: {val!r} is cited to '{head}:' with nothing after it")
            continue
        path = resolve(src)
        if path is None:
            errs.append(f"numbers: {val!r} is cited to `{src}`, which is not in the tree (check d)")
            continue
        find = str(c.get("find", val))
        try:
            body = path.read_text(errors="replace")
        except OSError as e:
            errs.append(f"numbers: cannot read `{src}`: {e}")
            continue
        if find not in body:
            errs.append(f"numbers: {find!r} does not occur in `{src}`, the file the draft cites it to (check d)")
    loose = []
    for para in re.split(r"\n\s*\n", ANY_FENCE.sub("", prose)):
        hits = UNIT_NUMBER.findall(para)
        if not hits:
            continue
        if any(resolve(t) is not None for t in PATH_TOKEN.findall(para)):
            continue
        for num, unit in hits:
            if not any(num in k for k in known):
                loose.append(f"{num} {unit}")
    if loose:
        shown = ", ".join(loose[:6]) + (" ..." if len(loose) > 6 else "")
        msg = (f"{len(loose)} number(s) with a unit sit in a paragraph that names no existing file and are not "
               f"in a numbers block: {shown}")
        if strict:
            errs.append(msg + " (check d, --strict-numbers)")
        else:
            notes.append(msg + " (reported; refused only under --strict-numbers)")
    return errs, notes


ALLOWED = {ast.Expression, ast.BinOp, ast.UnaryOp, ast.Constant, ast.Name, ast.Load, ast.Call,
           ast.Add, ast.Sub, ast.Mult, ast.Div, ast.Pow, ast.USub, ast.UAdd}
FUNCS = {"sqrt": math.sqrt, "abs": abs, "log": math.log, "exp": math.exp, "pi": math.pi}


def evaluate(expr, point):
    """Arithmetic on a point's own parameters. Names, numbers, + - * / **, sqrt abs log exp pi."""
    tree = ast.parse(str(expr), mode="eval")
    for node in ast.walk(tree):
        if type(node) not in ALLOWED:
            raise ValueError(f"not arithmetic: {type(node).__name__}")
        if isinstance(node, ast.Call) and not (isinstance(node.func, ast.Name) and node.func.id in FUNCS):
            raise ValueError("only sqrt, abs, log, exp may be called")
        if isinstance(node, ast.Name) and node.id not in FUNCS and node.id not in point:
            raise ValueError(f"unknown name {node.id!r}")
    return float(eval(compile(tree, "<shift>", "eval"), {"__builtins__": {}}, {**FUNCS, **point}))


def in_region(point, region):
    """True, False, or the name of a parameter the point does not give."""
    for k, c in region.items():
        if k not in point:
            return k
        v = point[k]
        lo, hi = c.get("lo", -math.inf), c.get("hi", math.inf)
        if v < lo or v > hi or (c.get("lo_open") and v <= lo) or (c.get("hi_open") and v >= hi):
            return False
        if any(abs(v - e) <= 1e-12 for e in c.get("except", [])):
            return False
    return True


def check_reserve(objs, text):
    errs = []
    reserves = [o for o in objs if o["kind"] == "reserve"]
    points = [o for o in objs if o["kind"] == "points"]
    low = BLOCK.sub("", text).lower()
    if not reserves:
        word = next((w for w in RESERVE_WORDS if w in low), None)
        if word:
            errs.append(f"the draft says '{word}' but declares no reserve block (check e); a reserve nobody can "
                        f"test is kept by honour only [M-RESERVE-BREACHED]")
        return errs
    if not any(p.get("points") for p in points):
        errs.append("a region is reserved but no points block declares what the campaign, its gates, plants "
                    "and pilots will compute, so the reserve cannot be tested [M-RESERVE-BREACHED]")
    for r in reserves:
        tag = f"reserve {r.get('id')}"
        region = r.get("region") or {}
        if not region:
            errs.append(f"{tag}: no \"region\"")
            continue
        meaning = r.get("meaning") or {}
        for k in region:
            if meaning.get(k) not in ("bare", "shifted"):
                errs.append(f"{tag}: does not say whether {k!r} is the bare parameter or the shifted one "
                            f"[M-NAMED-POINT-SHIFTED]")
        if "shifts" not in r:
            errs.append(f"{tag}: no \"shifts\"; state the map from a named point to the point the instrument is "
                        f"really at, or \"shifts\": {{}} with a no_shift_reason [M-NAMED-POINT-SHIFTED]")
        elif not r["shifts"] and not str(r.get("no_shift_reason", "")).strip():
            errs.append(f"{tag}: \"shifts\" is empty and no no_shift_reason says why [M-NAMED-POINT-SHIFTED]")
        if not str(r.get("known_inside", "")).strip():
            errs.append(f"{tag}: no \"known_inside\"; say what closed-form theory already fixes within the "
                        f"region, since a blind stake belongs where that theory fails [M-RESERVE-BREACHED]")
        for group in points:
            for pt in group.get("points", []):
                forms = [("as declared", dict(pt))]
                if r.get("shifts"):
                    moved = dict(pt)
                    try:
                        for k, expr in r["shifts"].items():
                            moved[k] = evaluate(expr, pt)
                    except (ValueError, SyntaxError, ZeroDivisionError, TypeError) as e:
                        errs.append(f"{tag}: cannot apply its shift to the point {pt}: {e}")
                        continue
                    forms.append(("after the shift", moved))
                for how, q in forms:
                    res = in_region(q, region)
                    where = f"{group.get('of', '?')} point {pt} ({group.get('label', '')})"
                    if res is True:
                        shown = {k: (round(v, 6) if isinstance(v, float) else v) for k, v in q.items()}
                        errs.append(f"{tag}: the {where} lies INSIDE the reserved region {how}: {shown} "
                                    f"[M-RESERVE-BREACHED]")
                    elif res is not False:
                        errs.append(f"{tag}: the {where} does not give {res!r}, so it cannot be placed "
                                    f"against the reserve")
    return errs


def check(path, root=ROOT, strict=False):
    path = pathlib.Path(path)
    text = path.read_text()
    objs, errs = blocks(text)
    notes = []
    if not objs and not errs:
        return (["the draft carries no ```prereview block; the convention is in the header of "
                 "Audit/prereview_check.py and in Audit/PREREVIEW.md"], notes)
    unknown = sorted({o["kind"] for o in objs} - {"stake", "plant", "branches", "numbers", "new_files",
                                                  "reserve", "points"})
    for k in unknown:
        errs.append(f"unknown block kind {k!r}")
    errs += check_stakes(objs)
    e, n = check_branches(objs)
    errs += e
    notes += n
    e, n = check_numbers(objs, text, pathlib.Path(root), path, strict)
    errs += e
    notes += n
    errs += check_reserve(objs, text)
    return errs, notes


def main(argv):
    root, strict, files = ROOT, False, []
    it = iter(argv)
    for a in it:
        if a == "--root":
            root = pathlib.Path(next(it)).resolve()
        elif a == "--strict-numbers":
            strict = True
        else:
            files.append(a)
    if not files:
        print(__doc__)
        return 2
    bad = 0
    for f in files:
        errs, notes = check(f, root, strict)
        tag = pathlib.Path(f).name
        if errs:
            bad += 1
            print(f"REFUSED  {tag}  ({len(errs)} reason(s); not ready for a reviewer)")
            for e in errs:
                print(f"    - {e}")
        else:
            print(f"PASSED   {tag}  (its own arithmetic is consistent; this is not a review)")
        for n in notes:
            print(f"    note: {n}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
