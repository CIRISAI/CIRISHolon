#!/usr/bin/env python3
"""holonc: the reference compiler of the holon declaration, version 0 (standard/HOLON-DECL.md).

It reads the closed vocabularies (standard/vocabulary.json) and every declaration in the
registry (standard/registry/*.json) and gives each declaration ONE answer:

    admitted            every check held and everything it cites is held here
    waiting             every check held, but it cites evidence or an engine that is not held
                        here, or it stands on a declaration that is waiting
    refused             with the SET of every failing check's token, sorted

The answer does not depend on the order of the checks. They run in two phases. Phase A is
shape: members, types, ids and closed vocabularies. A declaration that fails phase A is
refused with phase A's tokens alone, because nothing more can be read from it safely. Phase
B is meaning: every check runs, and every failure is collected. It executes nothing a
declaration names, and reads no path outside the repository.

    python3 standard/tools/holonc.py                  check the registry; exit 1 if any is refused
    python3 standard/tools/holonc.py --vectors        the known answers (standard/vectors/*.json):
                                                      each file states what it expects, and every
                                                      refusal token of the vocabulary must be
                                                      expected by at least one vector
    python3 standard/tools/holonc.py --write          regenerate standard/HOLONS.md
    python3 standard/tools/holonc.py --check          fail if standard/HOLONS.md is stale
    python3 standard/tools/holonc.py --ladder FILE    also check the ENGINE against the registry;
                                                      FILE is the output of
                                                      `cargo run -p holon-draw-ladder --example rungs_dump`
    python3 standard/tools/holonc.py --json           print the answers as one JSON document

Only the Python standard library is used.
"""

import argparse
import hashlib
import json
import math
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
STD = HERE.parent
REPO = STD.parent
SPEC = "ciris/holon-decl/0"
ID = re.compile(r"[a-z][a-z0-9]*(_[a-z0-9]+)*(\.[a-z][a-z0-9]*(_[a-z0-9]+)*)*")
SHA = re.compile(r"[0-9a-f]{64}")
DATE = re.compile(r"[0-9]{4}-[0-9]{2}-[0-9]{2}")
TEXT = re.compile(r"[^\n\r]*[^ \t\r\n][^\n\r]*")  # one line, not only white space: text1 of holon_decl.cddl
GATE = re.compile(r'\bok "([^"$]+)"')  # a gate is the whole message of an ok line of engine/ci-gates.sh


# ---------------------------------------------------------------- phase A: shape
# A shape is: ("str",) a non-empty string; ("id",); ("int",); ("num",) a finite number >= 0;
# ("anynum",) a finite number; ("word", vocabulary); ("list", shape, least length);
# ("map", {member: (shape, required)}); ("one", [shapes]) the first that fits; ("sha",); ("date",).
def shapes():
    ref = ("one", [("map", {"path": (("str",), True), "note": (("str",), False)}),
                   ("map", {"external": (("str",), True), "edition": (("str",), False)})])
    evid = ("one", [("map", {"path": (("str",), True), "sha256": (("sha",), False)}),
                    ("map", {"external": (("str",), True)}),
                    ("map", {"gate": (("str",), True)})])
    budget = ("map", {"quantity": (("str",), True), "bound": (("num",), True), "unit": (("word", "budget_unit"), True)})
    consumer = ("map", {"id": (("str",), True), "kind": (("word", "consumer"), True), "budget": (budget, False),
                        "holon": (("id",), False), "stated": (ref, False)})
    view = ("map", {"id": (("str",), True), "reads": (("str",), True), "consumers": (("list", consumer, 0), True)})
    part = ("map", {"id": (("str",), True), "what": (("str",), True), "status": (("word", "status"), True), "source": (("list", ref, 1), False)})
    cert = ("map", {
        "kind": (("word", "certificate"), True),
        "for": (("map", {"view": (("str",), True), "consumer": (("str",), True)}), False),
        "bound": (("map", {"value": (("num",), True), "unit": (("word", "budget_unit"), True)}), False),
        "basis": (("word", "basis"), False),
        "evidence": (("list", evid, 0), False),
        "parts": (("map", {
            "section_law": (("map", {"evidence": (("list", evid, 0), True)}), False),
            "transport_cost": (("map", {"evidence": (("list", evid, 0), True),
                                        "reference": (("map", {"path": (("str",), True), "sha256": (("sha",), True), "frozen_at": (("date",), True)}), False)}), False),
        }), False),
    })
    draw = ("map", {"marks": (("list", ("word", "mark"), 0), True), "label": (("str",), True),
                    "join_below": (("word", "join"), False), "join_above": (("word", "join"), False),
                    "fade_owed": (("str",), False), "rung": (("str",), False),
                    "span_z": (("list", ("anynum",), 2, 2), False)})
    return ("map", {
        "spec": (("str",), True), "id": (("id",), True), "name": (("str",), True), "plain": (("str",), True),
        "status": (("word", "status"), True), "standing": (("word", "standing"), True),
        "carrier": (("map", {"state": (("str",), True), "unit_log10_m": (("int",), False)}), True),
        "step": (("map", {"kind": (("word", "step"), True), "engine": (("str",), False), "engine_external": (("str",), False)}), True),
        "below": (("list", ("map", {"id": (("id",), True), "relation": (("word", "relation"), True), "basis": (ref, False)}), 0), True),
        "depicts": (("map", {"id": (("id",), True)}), False),
        "parts": (("list", part, 1), False),
        "source": (("list", ref, 1), False),
        "views": (("list", view, 0), True),
        "regime": (("str",), False),
        "budget_window": (("list", ("num",), 2, 2), False),
        "certificate": (cert, True),
        "carried_by": (("str",), False),
        "tested_against": (("list", ("str",), 1), False),
        "drops": (("list", ("map", {"what": (("str",), True), "price": (("str",), True)}), 1), False),
        "kill": (("map", {"text": (("str",), True), "stated_at": (("date",), True)}), False),
        "draw": (draw, False),
        "not_claimed": (("list", ("str",), 1), False),
        "owed": (("list", ("str",), 1), False),
    })


def fits(x, shape, V, where, fail):
    """Check x against shape, reporting through fail(token, why). True when it fits."""
    k = shape[0]
    ok = True
    if k == "str":
        ok = isinstance(x, str) and TEXT.fullmatch(x) is not None
    elif k == "id":
        if not isinstance(x, str):
            ok = False
        elif not ID.fullmatch(x):
            fail("id_malformed", f"{where}: {x!r}")
            return False
    elif k == "sha":
        ok = isinstance(x, str) and SHA.fullmatch(x) is not None
    elif k == "date":
        ok = isinstance(x, str) and DATE.fullmatch(x) is not None
    elif k == "int":
        ok = isinstance(x, int) and not isinstance(x, bool)
    elif k in ("num", "anynum"):
        try:
            ok = isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(float(x)) and (k == "anynum" or x >= 0)
        except OverflowError:
            ok = False
    elif k == "word":
        if not isinstance(x, str):
            ok = False
        elif x not in V[shape[1]]:
            fail("vocabulary_unregistered", f"{where}: {x!r} is not in the {shape[1]} vocabulary")
            return False
    elif k == "list":
        if not isinstance(x, list) or len(x) < shape[2] or (len(shape) > 3 and len(x) > shape[3]):
            ok = False
        else:
            return all([fits(e, shape[1], V, f"{where}[{i}]", fail) for i, e in enumerate(x)])
    elif k == "map":
        if not isinstance(x, dict):
            ok = False
        else:
            good = True
            for name, (_, required) in shape[1].items():
                if required and name not in x:
                    fail("member_missing", f"{where}.{name} is absent")
                    good = False
            for name in x:
                if name not in shape[1]:
                    fail("member_unexpected", f"{where}.{name}")
                    good = False
                elif not fits(x[name], shape[1][name][0], V, f"{where}.{name}", fail):
                    good = False
            return good
    elif k == "one":
        if any(fits(x, alt, V, where, lambda *_: None) for alt in shape[1]):
            return True
        ok = False
    if not ok:
        fail("type_wrong", f"{where} is not a well-formed {k}")
    return ok


# ---------------------------------------------------------------- phase B: meaning
def held(path, root):
    """The file a repository path names, or None: relative, inside the repository, a regular file."""
    if not isinstance(path, str) or path.startswith("/") or ".." in pathlib.PurePosixPath(path).parts:
        return None
    try:
        r = (root / path).resolve()
        r.relative_to(root.resolve())
    except (ValueError, OSError):
        return None
    return r if r.is_file() else None


def evidence(items, where, status, root, gates, fail, waiting):
    if not items:
        fail("evidence_missing", f"{where}: no evidence")
        return
    for e in items:
        if "path" in e:
            f = held(e["path"], root)
            if f is None:
                fail("evidence_not_found", f"{where}: {e['path']!r} is not a file of this repository")
            elif "sha256" in e:
                h = hashlib.sha256()
                with open(f, "rb") as fh:
                    for chunk in iter(lambda: fh.read(1 << 20), b""):
                        h.update(chunk)
                if h.hexdigest() != e["sha256"]:
                    fail("evidence_digest", f"{where}: {e['path']} hashes to {h.hexdigest()}")
            elif status == "measured":
                fail("evidence_undigested", f"{where}: {e['path']} is cited for a measured holon with no sha256")
        elif "gate" in e:
            if e["gate"] not in gates:
                fail("gate_not_found", f"{where}: no gate of engine/ci-gates.sh says {e['gate']!r}")
        else:
            waiting.append(f"{where}: {e['external']}")


def meaning(d, world, root, gates):
    """Phase B on one well-shaped declaration. Returns (tokens, [why], [waiting on])."""
    tokens, whys, waiting = set(), [], []

    def fail(token, why):
        tokens.add(token)
        whys.append(f"{token}: {why}")

    status, standing, step, cert = d["status"], d["standing"], d["step"], d["certificate"]
    kind = cert["kind"]

    # what it stands on
    rel = {}
    for b in d["below"]:
        rel[b["id"]] = b["relation"]
        if b["id"] not in world:
            fail("below_unresolved", f"below names {b['id']!r}, which no declaration carries")
        if b["relation"] in ("function", "shared_edge") and held(b.get("basis", {}).get("path"), root) is None:
            fail("relation_unbacked", f"below {b['id']}: a {b['relation']} relation cites no file of this repository as its basis")
    if "depicts" in d and d["depicts"]["id"] not in world:
        fail("depicts_unresolved", f"depicts names {d['depicts']['id']!r}, which no declaration carries")

    # what the status owes
    if status in ("declared", "catalogue", "measured") and not d.get("source"):
        fail("source_missing", f"a {status} holon cites no source")
    for p in d.get("parts", []):
        if p["status"] in ("declared", "catalogue", "measured") and not p.get("source"):
            fail("source_missing", f"part {p['id']} is {p['status']} and cites no source")
    for s in d.get("source", []) + [s for p in d.get("parts", []) for s in p.get("source", [])]:
        if "path" in s and held(s["path"], root) is None:
            fail("source_not_found", f"source {s['path']!r} is not a file of this repository")
    if status == "wager" and "kill" not in d:
        fail("kill_missing", "a wager states no dated kill")
    if status == "simulated" and step["kind"] != "computed":
        fail("simulated_not_computed", f"status simulated with step.kind {step['kind']}")
    eng, ext = step.get("engine"), step.get("engine_external")
    if step["kind"] == "computed":
        if (eng is None) == (ext is None):
            fail("engine_missing", "a computed step names exactly one of engine and engine_external")
        elif eng is not None and held(eng, root) is None:
            fail("engine_missing", f"step.engine {eng!r} is not a file of this repository")
        elif ext is not None:
            waiting.append(f"step: {ext}")
    elif eng is not None or ext is not None:
        fail("engine_missing", f"a {step['kind']} step names an engine")

    # views, consumers, budgets
    if standing == "medium" and not d["views"]:
        fail("medium_without_view", "a medium with no view")
    consumers, pairs = {}, {}
    for v in d["views"]:
        if not v["consumers"]:
            fail("view_without_consumer", f"view {v['id']} names no consumer")
        for c in v["consumers"]:
            if consumers.setdefault(c["id"], c["kind"]) != c["kind"]:
                fail("consumer_conflict", f"consumer {c['id']} is {consumers[c['id']]} in one view and {c['kind']} in another")
            pairs[(v["id"], c["id"])] = c
            b = c.get("budget")
            if b and c["kind"] == "renderer" and b["unit"] not in ("px", "level"):
                fail("budget_unit_mismatch", f"renderer {c['id']} budgets in {b['unit']}")
            if c["kind"] == "tier_above":
                above = world.get(c.get("holon"))
                if above is None or d["id"] not in [x["id"] for x in above["below"]]:
                    fail("consumer_unresolved", f"tier_above consumer {c['id']} names no declared holon that stands on this one")
            if c["kind"] == "law" and held(c.get("stated", {}).get("path"), root) is None:
                fail("consumer_unresolved", f"law consumer {c['id']} cites no file of this repository where the law is stated")

    # the certificate, and what it is a certificate FOR
    if status == "measured" and kind == "none":
        fail("certificate_owed", "a measured holon with no certificate")
    if status == "illustration" and kind != "none":
        fail("illustration_certified", "an illustration carrying a certificate")
    if kind == "scene" and step["kind"] != "generated":
        fail("scene_not_generated", f"a scene certificate on a {step['kind']} holon")
    if kind in ("closure", "bound") and step["kind"] == "generated":
        fail("scene_not_generated", f"a generated holon holds a {kind} certificate; only the scene certificate describes it")
    if kind == "none":
        if any(k in cert for k in ("for", "bound", "basis", "evidence", "parts")):
            fail("certificate_unbound", "certificate.kind is none and it carries a certificate's members")
    else:
        f, bound = cert.get("for"), cert.get("bound")
        target = pairs.get((f["view"], f["consumer"])) if f else None
        if not f or not bound or "basis" not in cert or target is None:
            fail("certificate_unbound", "a certificate does not name a view and consumer of this holon, its bound, and whether that is a price or a realised cost")
        else:
            b = target.get("budget")
            if not b:
                fail("consumer_without_budget", f"the certificate is for {f['consumer']} on {f['view']}, which states no budget")
            elif b["unit"] != bound["unit"] or bound["value"] > b["bound"]:
                fail("certificate_exceeds_budget", f"the certificate's {bound['value']} {bound['unit']} does not meet the budget of {b['bound']} {b['unit']}")
        if "regime" not in d:
            fail("regime_missing", f"a {kind} certificate with no regime")
        if kind == "scene":
            parts = cert.get("parts", {})
            sl, tc = parts.get("section_law"), parts.get("transport_cost")
            if not sl or not tc or "reference" not in tc:
                fail("scene_certificate_incomplete", "a scene certificate needs a section law, a transport cost, and the frozen reference sample it is measured against")
            for name, part in (("section_law", sl), ("transport_cost", tc)):
                if part:
                    evidence(part["evidence"], f"certificate.parts.{name}", status, root, gates, fail, waiting)
            if tc and "reference" in tc:
                evidence([{k: tc["reference"][k] for k in ("path", "sha256")}], "certificate.parts.transport_cost.reference", status, root, gates, fail, waiting)
        else:
            evidence(cert.get("evidence"), "certificate", status, root, gates, fail, waiting)

    # what the standing owes
    if standing in ("tier", "sector") and kind != "closure":
        fail("tier_uncertified", f"a {standing} whose certificate is {kind}")
    if standing == "tier":
        cb = d.get("carried_by")
        if cb not in consumers or cert.get("for", {}).get("consumer") != cb:
            fail("tier_uncarried", f"carried_by {cb!r} is not the consumer its certificate is for")
    elif "carried_by" in d:
        fail("carried_not_tier", f"a {standing} that says it is carried")
    t = d.get("tested_against")
    if standing == "sector":
        if not t or any(x not in consumers for x in t):
            fail("sector_untested", "a sector does not list the consumers of its views it was tested against")
    elif t is not None:
        fail("sector_untested", f"a {standing} lists tested_against, which is a sector's member")

    # how it is drawn
    draw = d.get("draw")
    if status == "illustration" and draw is None:
        fail("illustration_undrawn", "an illustration with no draw view, and so no label saying it is one")
    if draw is not None:
        if draw.get("join_below", "none") == "fade":
            if any(r == "theory_change" for r in rel.values()):
                fail("fade_across_theories", "a fade below across a theory_change")
            if any(r == "context" for r in rel.values()) and "fade_owed" not in draw:
                fail("fade_unbacked", "a fade below across a context relation, with no fade_owed saying what would make it honest")
        for b in d["below"]:
            ld = world.get(b["id"], {}).get("draw")
            if ld is not None and ld.get("join_above", "none") != draw.get("join_below", "none"):
                fail("join_disagrees", f"this joins below by {draw.get('join_below', 'none')} and {b['id']} joins above by {ld.get('join_above', 'none')}")
    return tokens, whys, waiting


def read_ladder(text, V):
    back = {label: status for status, label in V["label"].items()}
    rungs = {}
    for line in text.splitlines():
        c = line.split("\t")
        if len(c) < 9:
            continue
        rungs[c[0]] = {"unit": int(c[1]), "label": c[2], "status": back.get(c[2]), "span": [float(c[3]), float(c[4])],
                       "cut_below": c[5] == "true", "cut_above": c[6] == "true", "marks": sorted(x for x in c[7].split(",") if x)}
    return rungs


def ladder_check(d, ladder, order, fail):
    dr = d["draw"]
    r = ladder.get(dr["rung"])
    if r is None:
        return fail(f"the engine has no rung named {dr['rung']!r}")
    if d["name"] != dr["rung"]:
        fail("name is not the rung's name")
    if r["status"] != d["status"]:
        fail(f"the engine draws it as {r['label']!r}, the declaration says {d['status']}")
    if dr["label"] != r["label"]:
        fail(f"the engine's label is {r['label']!r}, the declaration's {dr['label']!r}")
    if r["unit"] != d["carrier"].get("unit_log10_m"):
        fail(f"the engine's unit is 1e{r['unit']} m, the declaration's {d['carrier'].get('unit_log10_m')}")
    if sorted(dr["marks"]) != r["marks"]:
        fail(f"the engine emits {r['marks']}, the declaration says {sorted(dr['marks'])}")
    if [round(x, 3) for x in dr.get("span_z", [])] != r["span"]:
        fail(f"the engine's band is {r['span']}, the declaration's {dr.get('span_z')}")
    i = order.index(dr["rung"])
    for side, cut, edge in (("join_below", r["cut_below"], i == 0), ("join_above", r["cut_above"], i == len(order) - 1)):
        want = "cut" if cut else ("none" if edge else "fade")
        if dr.get(side, "none") != want:
            fail(f"the engine's {side} is {want}, the declaration's {dr.get(side, 'none')}")


def compile_all(decls, V, root, ladder=None):
    """decls: [(label, declaration)]. Returns one record per declaration: label, id, answer, tokens, why, waiting."""
    shape = shapes()
    gate_file = root / "engine" / "ci-gates.sh"
    gates = set(GATE.findall(gate_file.read_text(encoding="utf-8"))) if gate_file.is_file() else set()
    out, world, count = [], {}, {}
    for label, d in decls:
        tokens, whys = set(), []

        def fail(token, why, tokens=tokens, whys=whys):
            tokens.add(token)
            whys.append(f"{token}: {why}")

        if not isinstance(d, dict):
            fail("type_wrong", "a declaration is a map")
        else:
            if d.get("spec") != SPEC:
                fail("spec_unknown", f"spec is {d.get('spec')!r}")
            fits(d, shape, V, "decl", fail)
        rec = {"label": label, "id": d["id"] if not tokens else None, "tokens": tokens, "why": whys, "waiting": [], "shaped": not tokens, "decl": d}
        out.append(rec)
        if rec["shaped"]:
            count[d["id"]] = count.get(d["id"], 0) + 1
            world[d["id"]] = d
    # a duplicated id names nothing: whatever points at it is unresolved, whichever copy came first
    for i in [i for i, n in count.items() if n > 1]:
        del world[i]
    shaped = [rec for rec in out if rec["shaped"]]
    for rec in shaped:
        d = rec["decl"]
        t, w, wait = meaning(d, world, root, gates)
        rec["tokens"] |= t
        rec["why"] += w
        rec["waiting"] = wait
        if count[d["id"]] > 1:
            rec["tokens"].add("id_duplicate")
            rec["why"].append(f"id_duplicate: {count[d['id']]} declarations carry {d['id']}")
    under = lambda d: [b["id"] for b in d["below"]] + ([d["depicts"]["id"]] if "depicts" in d else [])
    below = {i: under(d) for i, d in world.items()}
    for rec in shaped:
        if rec["id"] not in world:
            continue
        seen, stack = set(), list(below[rec["id"]])
        while stack:
            x = stack.pop()
            if x == rec["id"]:
                rec["tokens"].add("below_cycle")
                rec["why"].append(f"below_cycle: {rec['id']} stands on itself")
                break
            if x not in seen:
                seen.add(x)
                stack.extend(below.get(x, []))
    if ladder is not None:
        order, claimed = list(ladder), {}
        for rec in shaped:
            d = rec["decl"]
            if "rung" in d.get("draw", {}):
                claimed.setdefault(d["draw"]["rung"], []).append(rec)

                def lf(why, rec=rec):
                    rec["tokens"].add("ladder_disagrees")
                    rec["why"].append(f"ladder_disagrees: {why}")

                ladder_check(d, ladder, order, lf)
        for name, recs in claimed.items():
            if len(recs) > 1:
                for rec in recs:
                    rec["tokens"].add("ladder_disagrees")
                    rec["why"].append(f"ladder_disagrees: {len(recs)} declarations claim the rung {name!r}")
        for name in order:
            if name not in claimed:
                out.append({"label": f"(engine rung) {name}", "id": None, "tokens": {"ladder_disagrees"},
                            "why": ["ladder_disagrees: the engine draws this rung and no declaration carries it"], "waiting": [], "shaped": False, "decl": None})
    # what stands on a refused declaration is refused, and what stands on a waiting one waits: to a fixed point
    by = {rec["id"]: rec for rec in shaped if rec["id"] in world}
    changed = True
    while changed:
        changed = False
        for rec in shaped:
            for x in under(rec["decl"]):
                o = by.get(x)
                if o is None or o is rec:
                    continue
                if o["tokens"] and "below_refused" not in rec["tokens"]:
                    rec["tokens"].add("below_refused")
                    rec["why"].append(f"below_refused: it stands on {x}, which is refused")
                    changed = True
                elif not o["tokens"] and o["waiting"] and f"on {x}, which is waiting" not in rec["waiting"]:
                    rec["waiting"].append(f"on {x}, which is waiting")
                    changed = True
    for rec in out:
        rec["answer"] = "refused" if rec["tokens"] else ("waiting" if rec["waiting"] else "admitted")
        rec["tokens"] = sorted(rec["tokens"])
    return out


def load_registry(p):
    return [(f.name, json.loads(f.read_text(encoding="utf-8"))) for f in sorted(p.glob("*.json"))]


def table(decls, answers):
    ans = {r["label"]: r for r in answers}
    lines = ["# The holons declared here", "",
             "*GENERATED by `python3 standard/tools/holonc.py --write` from `standard/registry/`. Never edited: `--check` fails if it differs.*", "",
             "| id | name | status | standing | step | stands on | views: consumer (budget) | certificate | answer | declaration sha256 |",
             "|---|---|---|---|---|---|---|---|---|---|"]
    for label, d in decls:
        views = "; ".join(
            f"{v['id']}: " + ", ".join(c["id"] + (f" ({c['budget']['bound']:g} {c['budget']['unit']})" if c.get("budget") else "") for c in v["consumers"])
            for v in d["views"]) or "none"
        below = ", ".join(f"{b['id']} ({b['relation']})" for b in d["below"]) or "nothing"
        if "depicts" in d:
            below += f"; depicts {d['depicts']['id']}"
        status = d["status"] + ("; parts: " + ", ".join(f"{p['id']} {p['status']}" for p in d["parts"]) if d.get("parts") else "")
        c = d["certificate"]
        cert = c["kind"] if c["kind"] == "none" else f"{c['kind']}, {c['basis']}, {c['bound']['value']:g} {c['bound']['unit']} for {c['for']['consumer']}"
        sha = hashlib.sha256((STD / "registry" / label).read_bytes()).hexdigest()[:16]
        lines.append(f"| `{d['id']}` | {d['name']} | {status} | {d['standing']} | {d['step']['kind']} | {below} | {views} | {cert} | {ans[label]['answer']} | `{sha}…` |")
    by = {}
    for _, d in decls:
        by[d["status"]] = by.get(d["status"], 0) + 1
    lines += ["", f"{len(decls)} declarations: " + ", ".join(f"{n} {s}" for s, n in sorted(by.items())) + "."]
    owed = [(d["id"], o) for _, d in decls for o in ([d["draw"]["fade_owed"]] if "fade_owed" in d.get("draw", {}) else []) + d.get("owed", [])]
    if owed:
        lines += ["", "## Owed", ""] + [f"- `{i}`: {o}" for i, o in owed]
    return "\n".join(lines) + "\n"


def run_vectors(V):
    fails, seen, multi = [], set(), 0
    files = sorted((STD / "vectors").glob("*.json"))
    for f in files:
        doc = json.loads(f.read_text(encoding="utf-8"))
        ladder = read_ladder(doc["ladder"], V) if "ladder" in doc else None
        got = compile_all([(f"{f.name}#{i}", x) for i, x in enumerate(doc["declarations"])], V, REPO, ladder)
        want = doc["expect"]
        if isinstance(want, str):
            bad = [f"{r['label']} is {r['answer']} {r['tokens']}" for r in got if r["answer"] != want]
            if bad:
                fails.append(f"{f.name}: expected every declaration {want}: " + "; ".join(bad))
        else:
            tokens = sorted({t for r in got for t in r["tokens"]})
            if tokens != sorted(want):
                fails.append(f"{f.name}: expected exactly {sorted(want)}, got {tokens}")
            seen |= set(want)
            multi += len(want) > 1
    for token in V["refusal"]:
        if token not in seen:
            fails.append(f"no vector expects the refusal {token}")
    for token in sorted(seen - set(V["refusal"])):
        fails.append(f"a vector expects {token}, which is not a refusal of the vocabulary")
    if multi < 3:
        fails.append(f"only {multi} vectors carry more than one fault; at least 3 are needed to show the answer is a set")
    return files, seen, multi, fails


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    for flag in ("--vectors", "--write", "--check", "--json"):
        ap.add_argument(flag, action="store_true")
    ap.add_argument("--ladder")
    a = ap.parse_args()
    V = json.loads((STD / "vocabulary.json").read_text(encoding="utf-8"))
    decls = load_registry(STD / "registry")
    ladder = read_ladder(pathlib.Path(a.ladder).read_text(encoding="utf-8"), V) if a.ladder else None
    answers = compile_all(decls, V, REPO, ladder)
    say = (lambda *_: None) if a.json else print
    for r in answers:
        if r["answer"] == "refused":
            say(f"  refused  {r['label']}: {r['tokens']}")
            for w in r["why"]:
                say(f"             {w}")
        elif r["answer"] == "waiting":
            say(f"  waiting  {r['label']}: " + "; ".join(r["waiting"]))
    n = {s: sum(1 for r in answers if r["answer"] == s) for s in ("admitted", "waiting", "refused")}
    say(f"registry: {len(decls)} declarations; {n['admitted']} admitted, {n['waiting']} waiting, {n['refused']} refused"
        + (f"; the engine's {len(ladder)} rungs checked against it" if ladder is not None else ""))
    status = 1 if n["refused"] else 0
    report = {"registry": [{k: r[k] for k in ("label", "id", "answer", "tokens", "why", "waiting")} for r in answers]}

    if a.vectors:
        files, seen, multi, fails = run_vectors(V)
        say(f"vectors: {len(files)} files; {len(seen)} of {len(V['refusal'])} refusals expected; {multi} with more than one fault; {len(fails)} failure(s)")
        for x in fails:
            say("  -", x)
        report["vectors"] = {"files": len(files), "failures": fails}
        status |= 1 if fails else 0

    if a.write or a.check:
        out = STD / "HOLONS.md"
        if n["refused"]:
            say("HOLONS.md is not written while a declaration is refused")
            status = 1
        else:
            text = table(decls, answers)
            if a.write:
                out.write_text(text, encoding="utf-8")
                say(f"wrote {out.relative_to(REPO)}")
            elif not out.exists() or out.read_text(encoding="utf-8") != text:
                say("standard/HOLONS.md is stale: run holonc.py --write")
                status = 1
    if a.json:
        print(json.dumps(report, indent=1))
    return status


if __name__ == "__main__":
    sys.exit(main())
