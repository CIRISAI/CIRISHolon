#!/usr/bin/env python3
"""Check INTERFACE.md section 9 against the screen standard's registry (CSD/5 draft 3).

    python3 engine/crates/holon-draw/tools/check_registry.py [--interface FILE] [--registry FILE]

The registry is vendored at ../registry/ui_primitives.json (its README names the tag and hash).
The check refuses (exit 1, listing every finding) when any of these fails:

  - the registry's SHA-256 is the pinned one;
  - section 9 has the registry's 24 primitives, in the registry's order, with no others;
  - each primitive's prefixes are the registry's;
  - each prefix has a root tag form `prefix_<surface>_<instance>`, followed by `[.key]` exactly
    when the primitive is keyed;
  - each primitive's slot rows are the registry's slots, no more and no fewer (`@states` is
    written `__<state>`, one form for each of the registry's states);
  - each slot's tag form is `...__slot`, followed by `[.key]` exactly when the primitive is keyed.
    This is the grammar's order, `prefix_surface_instance[__slot][.key]`: everything after the
    first '.' is the key, so a slot written after the key would be read as part of it;
  - the total number of slots is the registry's (60);
  - the shell prefixes listed under the table are the registry's;
  - the header's counts say the registry's numbers.

Only the tag forms are checked. The marks column is this repository's own design, and no
registry speaks to it.
"""

import argparse
import hashlib
import json
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
PINNED = "6e8d058c14083cc8d26210d29e5bb0c93989b11306fd80a2c2a81f5adfc1510d"
ROOT_FORM = re.compile(r"^([a-z]+)_<surface>_<instance>(\[\.key\])?$")
SLOT_FORM = re.compile(r"^…__([a-z0-9_]+|<state>)(\[\.key\])?$")


def section(text, number):
    """The lines of the section headed `## <number>.`, up to the next `## ` heading."""
    lines = text.splitlines()
    start = next((i for i, l in enumerate(lines) if l.startswith(f"## {number}.")), None)
    if start is None:
        return None
    end = next((i for i in range(start + 1, len(lines)) if lines[i].startswith("## ")), len(lines))
    return lines[start:end]


def cells(line):
    return [c.strip() for c in line.strip().strip("|").split("|")]


def ticked(cell):
    return re.findall(r"`([^`]+)`", cell)


def check(interface, registry_path):
    out = []
    raw = registry_path.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    if digest != PINNED:
        out.append(f"registry hash {digest} is not the pinned {PINNED}")
    reg = json.loads(raw)
    prims = reg["primitives"]
    states = list(reg["states"])

    sec = section(interface.read_text(encoding="utf-8"), 9)
    if sec is None:
        return out + ["INTERFACE.md has no section 9"]

    table = [l for l in sec if l.startswith("|")][2:]  # past the header and its rule
    rows = []  # (name, prefixes, [root forms], [(slot, keyed form?)])
    for line in table:
        c = cells(line)
        if len(c) != 4:
            out.append(f"a row with {len(c)} cells: {line[:80]}")
            continue
        name = re.fullmatch(r"\*\*([A-Za-z]+)\*\*", c[0])
        if name:
            roots = ticked(c[2])
            rows.append([name.group(1), ticked(c[1]), roots, []])
        elif c[0] == "" and rows:
            for form in ticked(c[2]):
                rows[-1][3].append(form)
        else:
            out.append(f"a row that is neither a primitive nor one of its slots: {line[:80]}")

    names = [r[0] for r in rows]
    if names != list(prims):
        out.append(f"primitives {names} are not the registry's {list(prims)}")

    total = 0
    for name, prefixes, roots, slot_forms in rows:
        p = prims.get(name)
        if p is None:
            out.append(f"{name}: not in the registry")
            continue
        keyed = p["keyed"]
        if prefixes != p["prefixes"]:
            out.append(f"{name}: prefixes {prefixes}, the registry's {p['prefixes']}")
        root_prefixes = []
        for form in roots:
            m = ROOT_FORM.match(form)
            if not m:
                out.append(f"{name}: root tag form {form!r} is not prefix_<surface>_<instance>[.key]")
                continue
            root_prefixes.append(m.group(1))
            if bool(m.group(2)) != keyed:
                out.append(f"{name}: root {form!r} {'lacks' if keyed else 'has'} [.key] but the primitive is {'' if keyed else 'not '}keyed")
        if root_prefixes != p["prefixes"]:
            out.append(f"{name}: root forms for {root_prefixes}, the registry's prefixes {p['prefixes']}")
        slots = []
        for form in slot_forms:
            m = SLOT_FORM.match(form)
            if not m:
                out.append(f"{name}: slot tag form {form!r} is not …__slot[.key] (the slot comes before the key)")
                continue
            slots.append("@states" if m.group(1) == "<state>" else m.group(1))
            if bool(m.group(2)) != keyed:
                out.append(f"{name}: slot {form!r} {'lacks' if keyed else 'has'} [.key] but the primitive is {'' if keyed else 'not '}keyed")
        if slots != p["slots"]:
            out.append(f"{name}: slots {slots}, the registry's {p['slots']}")
        total += len(p["slots"])

    want_total = sum(len(p["slots"]) for p in prims.values())
    if total != want_total:
        out.append(f"{total} slots checked, the registry has {want_total}")

    body = "\n".join(sec)
    shell = re.search(r"Shell prefixes.*?:\s*((?:`[a-z]+_<name>`,?\s*)+)", body, re.S)
    found = re.findall(r"`([a-z]+)_<name>`", shell.group(1)) if shell else []
    if found != list(reg["shell_prefixes"]):
        out.append(f"shell prefixes {found}, the registry's {list(reg['shell_prefixes'])}")

    for n, what in [(len(prims), "primitives"), (want_total, "slots")]:
        if f"{n} {what}" not in body:
            out.append(f"section 9's header does not say '{n} {what}'")
    if f"{len(states)} in all" not in body:
        out.append(f"section 9 does not say the registry's {len(states)} states ('{len(states)} in all')")
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--interface", type=pathlib.Path, default=HERE.parents[3] / "INTERFACE.md")
    ap.add_argument("--registry", type=pathlib.Path, default=HERE.parent / "registry" / "ui_primitives.json")
    a = ap.parse_args()
    found = check(a.interface, a.registry)
    if found:
        print(f"INTERFACE.md section 9 against the registry: {len(found)} finding(s)")
        for f in found:
            print("  -", f)
        return 1
    reg = json.loads(a.registry.read_bytes())
    n = sum(len(p["slots"]) for p in reg["primitives"].values())
    print(f"INTERFACE.md section 9 against the registry: {len(reg['primitives'])} primitives, {n} slots, {len(reg['shell_prefixes'])} shell prefixes; every tag form in the grammar's order")
    return 0


if __name__ == "__main__":
    sys.exit(main())
