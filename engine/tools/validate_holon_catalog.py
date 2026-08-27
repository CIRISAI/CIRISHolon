#!/usr/bin/env python3
"""Validate the machine-readable CIRISHolon chemistry/material source catalog.

No third-party packages. This is a source-data gate, not a runtime dependency.
The no_std simulator should consume generated static tables after this validation step.
Known chemical bond graphs are validation targets, never dynamics inputs.
"""

from __future__ import annotations

import csv
import json
import math
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
CAT = ROOT / "data" / "holon_catalog"


class CatalogError(Exception):
    pass


def load_json(name: str):
    with (CAT / name).open("r", encoding="utf-8") as f:
        return json.load(f)


def load_csv(name: str):
    with (CAT / name).open("r", encoding="utf-8", newline="") as f:
        return list(csv.DictReader(f))


def load_jsonl(name: str):
    out = []
    with (CAT / name).open("r", encoding="utf-8") as f:
        for line_no, line in enumerate(f, 1):
            if line.strip():
                try:
                    out.append(json.loads(line))
                except json.JSONDecodeError as e:
                    raise CatalogError(f"{name}:{line_no}: invalid JSON: {e}") from e
    return out


def source_ids(obj):
    if isinstance(obj, dict):
        for k, v in obj.items():
            if (k == "source_id" or k.endswith("_source_id")) and isinstance(v, str):
                yield v
            yield from source_ids(v)
    elif isinstance(obj, list):
        for v in obj:
            yield from source_ids(v)


def require_sources(obj, known, context):
    missing = sorted({s for s in source_ids(obj) if s not in known})
    if missing:
        raise CatalogError(f"{context}: unresolved provenance IDs: {missing}")


def number_or_blank(row, key):
    text = row[key].strip()
    if not text:
        return None
    value = float(text)
    if not math.isfinite(value):
        raise CatalogError(f"non-finite {key}: {row}")
    return value


def validate_elements(rows, known_sources):
    if len(rows) != 118:
        raise CatalogError(f"elements.csv: expected 118 rows, got {len(rows)}")
    zs = [int(r["atomic_number"]) for r in rows]
    if zs != list(range(1, 119)):
        raise CatalogError("elements.csv: atomic numbers must be exactly 1..118 in order")
    symbols = [r["symbol"] for r in rows]
    if len(symbols) != len(set(symbols)):
        raise CatalogError("elements.csv: duplicate element symbol")
    for r in rows:
        value = number_or_blank(r, "abridged_standard_atomic_weight_u")
        unc = number_or_blank(r, "uncertainty_u")
        if value is not None and value <= 0:
            raise CatalogError(f"elements.csv: non-positive atomic weight: {r}")
        if unc is not None and unc <= 0:
            raise CatalogError(f"elements.csv: non-positive uncertainty: {r}")
        if r["source_id"] not in known_sources:
            raise CatalogError(f"elements.csv: unresolved source: {r['source_id']}")
    return set(symbols)


def validate_configs(rows, known_sources):
    zs = [int(r["atomic_number"]) for r in rows]
    if zs != list(range(1, 93)):
        raise CatalogError("electron_configurations.csv: seed must contain exactly H..U (Z=1..92)")
    for r in rows:
        if not r["neutral_ground_configuration"].strip():
            raise CatalogError(f"electron_configurations.csv: blank configuration: {r}")
        if r["source_id"] not in known_sources:
            raise CatalogError(f"electron_configurations.csv: unresolved source: {r['source_id']}")


def validate_species(rows, known_sources):
    ids = set()
    for r in rows:
        if r["id"] in ids:
            raise CatalogError(f"species.csv: duplicate id {r['id']}")
        ids.add(r["id"])
        if not r["formula"].strip():
            raise CatalogError(f"species.csv: blank formula: {r['id']}")
        mm = float(r["molar_mass_g_mol"])
        if not math.isfinite(mm) or mm <= 0:
            raise CatalogError(f"species.csv: invalid molar mass: {r['id']}")
        if r["source_id"] not in known_sources:
            raise CatalogError(f"species.csv: unresolved source: {r['source_id']}")
    return ids


def validate_chemistry_targets(records, species_ids, element_symbols):
    seen = set()
    for r in records:
        sid = r.get("species_id")
        if not sid or sid in seen or sid not in species_ids:
            raise CatalogError(f"chemistry target has missing/duplicate/unknown species_id {sid!r}")
        seen.add(sid)
        if r.get("role") != "withheld_validation_target" or r.get("must_not_parameterize_dynamics") is not True:
            raise CatalogError(f"{sid}: chemical topology must be explicitly withheld from dynamics")
        topo = r.get("target_topology")
        if not isinstance(topo, dict):
            raise CatalogError(f"{sid}: target_topology missing")
        atom_ids = set()
        for atom in topo.get("atoms", []):
            aid, elem = atom.get("id"), atom.get("element")
            if not isinstance(aid, int) or aid < 0 or aid in atom_ids or elem not in element_symbols:
                raise CatalogError(f"{sid}: malformed atom target {atom!r}")
            atom_ids.add(aid)
        if not atom_ids:
            raise CatalogError(f"{sid}: no atoms in target")
        for bond in topo.get("bonds", []):
            a, b, order = bond.get("a"), bond.get("b"), bond.get("order")
            if a not in atom_ids or b not in atom_ids or a == b:
                raise CatalogError(f"{sid}: invalid bond endpoints")
            if not isinstance(order, (int, float)) or order <= 0:
                raise CatalogError(f"{sid}: invalid bond order")


def check_fraction_value(v, path):
    if not isinstance(v, (int, float)) or not math.isfinite(v) or not (0 <= v <= 1):
        raise CatalogError(f"{path}: fraction must be finite and in [0,1], got {v!r}")


def validate_composition(comp, material_id):
    if comp is None:
        return
    components = comp.get("components")
    if not isinstance(components, dict) or not components:
        raise CatalogError(f"{material_id}: composition requires components")
    exact_total = 0.0
    all_exact = True
    remainder_count = 0
    for name, spec in components.items():
        path = f"{material_id}.composition.{name}"
        if isinstance(spec, (int, float)):
            check_fraction_value(spec, path)
            exact_total += float(spec)
        elif isinstance(spec, dict):
            all_exact = False
            if spec.get("remainder") is True:
                remainder_count += 1
            for key in ("min", "max", "value"):
                if key in spec:
                    check_fraction_value(spec[key], f"{path}.{key}")
            if "min" in spec and "max" in spec and spec["min"] > spec["max"]:
                raise CatalogError(f"{path}: min > max")
        else:
            raise CatalogError(f"{path}: unsupported component specification")
    if remainder_count > 1:
        raise CatalogError(f"{material_id}: more than one remainder component")
    if all_exact and abs(exact_total - 1.0) > 1e-9:
        raise CatalogError(f"{material_id}: exact composition sums to {exact_total}, not 1")


def validate_materials(records, known_sources):
    ids = set()
    for m in records:
        mid = m.get("id")
        if not mid or mid in ids:
            raise CatalogError(f"materials.jsonl: missing/duplicate id {mid!r}")
        ids.add(mid)
        require_sources(m, known_sources, mid)
        validate_composition(m.get("composition"), mid)
        for field, state in m.get("simulator_fields", {}).items():
            if state not in {"resolved", "condition_resolved", "unresolved"}:
                raise CatalogError(f"{mid}.simulator_fields.{field}: invalid state {state!r}")


def main() -> int:
    manifest = load_json("manifest.json")
    provenance = load_json(manifest["files"]["provenance"])
    known_sources = set(provenance["sources"])

    elements = load_csv(manifest["files"]["elements"])
    configs = load_csv(manifest["files"]["electron_configurations"])
    species = load_csv(manifest["files"]["species"])
    targets = load_jsonl(manifest["files"]["chemistry_validation_targets"])
    materials = load_jsonl(manifest["files"]["materials"])
    mixing = load_json(manifest["files"]["mixture_rules"])

    element_symbols = validate_elements(elements, known_sources)
    validate_configs(configs, known_sources)
    species_ids = validate_species(species, known_sources)
    validate_chemistry_targets(targets, species_ids, element_symbols)
    validate_materials(materials, known_sources)
    require_sources(mixing, known_sources, "mixture_rules.json")

    print(json.dumps({
        "catalog": manifest["catalog_id"],
        "version": manifest["catalog_version"],
        "elements": len(elements),
        "electron_configurations": len(configs),
        "species": len(species),
        "chemistry_validation_targets": len(targets),
        "materials": len(materials),
        "provenance_sources": len(known_sources),
        "status": "valid"
    }, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except CatalogError as e:
        print(f"catalog validation failed: {e}", file=sys.stderr)
        raise SystemExit(2)
