#!/usr/bin/env python3
"""SCOPE-0: a language model as a scene. Builds the table an explorer reads.

  build_scope_table.py RATCHET_CHECKOUT RATCHET_COMMIT HANDOFF_DIR OUT_DIR
  build_scope_table.py plants

The twin draws a world from tables that say, for each place, a value with its reference and
verdict, or nothing. This is the same contract with a model's sites in place of places:

  a SITE    is (model, layer, component, position);
  a LAYER   is one exact kind of reading (never two kinds on one map);
  a CELL    is a value with its interval or its null, the experiment and file it came from,
            and a STANDING: preregistered | exploratory | void;
  no cell   means nobody measured it. Nothing is interpolated, carried from a neighbouring
            site, or averaged across kinds.

Nothing is measured here. Every value is copied from a result file: files of the RATCHET
kindscope series are read with `git show COMMIT:path` (committed results only), files of the
hand-over bundle by path with their sha256 recorded. Standard library only.
"""
import hashlib
import json
import os
import subprocess
import sys

KS = "experiments/kindscope/"
STANDINGS = ("preregistered", "exploratory", "void")

MODELS = {
    "goldworm-v6": {
        "title": "GoldWorm v6 (2.76 M parameters, byte level)", "layers": 6, "components": ["embed", "attn", "mlp"],
        "frame": "closed", "frame_reason": "CAL-1: the writes sum to the final state (relative error 0.0) and rebuild the logits to 2.3e-5",
    },
    "gemma-4-e2b-q4": {
        "title": "Gemma 4 E2B (4-bit file of the hand-over)", "layers": 35, "components": ["state", "attn", "mlp", "ple"],
        "frame": "not closed", "frame_reason": "the hand-over's attribution fails its additivity check (relative error 1.19); the file rescales the stream at each layer; a calibration (CAL-G) is frozen and not run",
    },
}

LAYERS = [
    {"id": "gw.identity", "model": "goldworm-v6", "title": "Word identity", "kind": "share of a write's variance explained by the word's form (eta squared), at a word's last byte",
     "unit": "share", "domain": [0, 1], "null": "the same with forms shuffled, 95th percentile", "question": "Where is the word written?"},
    {"id": "gw.read.pos", "model": "goldworm-v6", "title": "Part of speech, readable", "kind": "accuracy of a linear probe for the part of speech (10 classes) on words not seen in fitting",
     "unit": "accuracy", "domain": [0, 1], "null": "the same probe on shuffled labels, 95th percentile", "question": "Where can a reader find the category?"},
    {"id": "gw.parallax", "model": "goldworm-v6", "title": "Reading against writing", "kind": "angle between the direction a probe reads adjective-or-noun from and the direction the model writes it with",
     "unit": "degrees", "domain": [0, 90], "null": "two random directions in 192 dimensions: 81.9 to 98.1 degrees (95 %)", "question": "Is what a reader finds what the model uses?"},
    {"id": "gw.push.write", "model": "goldworm-v6", "title": "Pushed along the writing direction", "kind": "change in log-odds of an adjective after a determiner when the layer's state is pushed along the WRITING direction at half the write's norm",
     "unit": "nats", "domain": [-0.6, 0.6], "null": "zero; the interval is beside each value", "question": "Does pushing where the model writes move the behaviour?"},
    {"id": "gw.push.read", "model": "goldworm-v6", "title": "Pushed along the reading direction", "kind": "the same push along the probe's READING direction at half the write's norm",
     "unit": "nats", "domain": [-0.6, 0.6], "null": "zero; the interval is beside each value", "question": "Does pushing where a reader looks move the behaviour?"},
    {"id": "gw.knock", "model": "goldworm-v6", "title": "Does attribution predict a knock-out?", "kind": "correlation, over positions, between a write's direct (linear) attribution to the answer and the realised change when that write is knocked out",
     "unit": "correlation", "domain": [-1, 1], "null": "none stored", "question": "Can the effect of removing a block be read off without removing it?"},
    {"id": "gm.read.kind", "model": "gemma-4-e2b-q4", "title": "The kind, readable", "kind": "accuracy of a linear probe for the twelve kinds on the layer's state, asked prompt, cross-validated",
     "unit": "accuracy", "domain": [0, 1], "null": "chance is 1/12; no shuffled probe is stored in this file", "question": "Where can a reader find the kind?"},
    {"id": "gm.lens.kind", "model": "gemma-4-e2b-q4", "title": "The kind, said", "kind": "accuracy of the model's OWN head applied to the layer's state (the twelve kind names)",
     "unit": "accuracy", "domain": [0, 1], "null": "chance is 1/12", "question": "From where could the model itself say the kind?"},
    {"id": "gm.closure", "model": "gemma-4-e2b-q4", "title": "Is the kind view closed?", "kind": "what the full state adds (R squared) to the kind view's prediction of its own next write, asked prompt",
     "unit": "R squared added", "domain": [0, 0.15], "null": "the same for a view built from shuffled labels", "question": "Does the kind predict its own next step, or does it need the rest?"},
    {"id": "gm.remove", "model": "gemma-4-e2b-q4", "title": "Removed, is it rebuilt?", "kind": "final-layer probe accuracy after the kind subspace is removed at this layer, asked prompt",
     "unit": "accuracy", "domain": [0, 1], "null": "the same after removing a variance-matched subspace disjoint from the kinds (control B)", "question": "From which layer on is the kind carried and not rebuilt?"},
    {"id": "gm.attrib", "model": "gemma-4-e2b-q4", "title": "Who pushes the answer (uncalibrated)", "kind": "direct attribution of the answer margin to each block's write, items of kind Premises",
     "unit": "logit units", "domain": [-15, 15], "null": "none; the frame does not close", "question": "Which block pushes the answer toward or away from the right name?"},
]


class Refuse(Exception):
    pass


def row(layer, site, value, standing, source, interval=None, n=None, null=None, stake=None, category=None, note=None):
    return {"layer": layer, "site": site, "value": value, "interval": interval, "n": n, "null": null, "standing": standing, "stake": stake,
            "category": category, "source": source, "note": note}


def site(layer, component, position=None):
    return {"layer": layer, "component": component, "position": position}


def check(table):
    """The contract. Raises `Refuse` with the reason."""
    layers = {l["id"]: l for l in table["layers"]}
    seen = set()
    for r in table["rows"]:
        l = layers.get(r["layer"])
        if l is None:
            raise Refuse("a row of no declared layer: %s" % r["layer"])
        m = table["models"][l["model"]]
        s = r["site"]
        if s["component"] not in m["components"] or not (0 <= s["layer"] < m["layers"]):
            raise Refuse("a site the model does not have: %s %s" % (l["model"], s))
        if not isinstance(r["value"], (int, float)) or r["value"] != r["value"] or abs(r["value"]) == float("inf"):
            raise Refuse("a value that is not a finite number in %s at %s" % (r["layer"], s))
        if r["standing"] not in STANDINGS:
            raise Refuse("a standing that is none of %s: %s" % (STANDINGS, r["standing"]))
        src = r["source"]
        if not (src.get("experiment") and src.get("file") and src.get("key")):
            raise Refuse("a row with no experiment, file or key in %s at %s" % (r["layer"], s))
        if r["standing"] == "preregistered" and r["interval"] is None and r["null"] is None:
            raise Refuse("a preregistered row with neither an interval nor a null in %s at %s" % (r["layer"], s))
        if r["standing"] != "void" and m["frame"] != "closed" and l["id"].endswith("attrib"):
            raise Refuse("an attribution in a frame that does not close must be void")
        k = (r["layer"], s["layer"], s["component"], s["position"], r["category"])
        if k in seen:
            raise Refuse("two rows for one cell: %s" % (k,))
        seen.add(k)
        if r["interval"] is not None and not (r["interval"][0] <= r["value"] <= r["interval"][1]):
            raise Refuse("a value outside its own interval in %s at %s" % (r["layer"], s))
    return len(table["rows"])


def build(read_k, read_h):
    """`read_k(name)`, `read_h(name)` return (parsed json, provenance string)."""
    rows = []

    def src(exp, f, key, prov):
        return {"experiment": exp, "file": f, "key": key, "provenance": prov}
    d, p = read_k("mlp0_1_result.json")
    for name, (li, comp) in {"wte": (0, "embed"), "attn0": (0, "attn"), "mlp0": (0, "mlp"), "attn1": (1, "attn"), "mlp1": (1, "mlp")}.items():
        c = d["M1"][name]
        rows.append(row("gw.identity", site(li, comp, "last"), c["eta2"], "preregistered", src("MLP0-1", "mlp0_1_result.json", "M1." + name, p),
                        null={"kind": "shuffled forms, 95th percentile", "value": c["null_p95"]}, stake="held"))
    d, p = read_k("vocab1_result.json")
    for key, c in d["cells"].items():
        pos, st = key.split("/")
        comp, li = ("embed", 0) if st == "embed" else (st[:-1], int(st[-1]))
        rows.append(row("gw.read.pos", site(li, comp, pos), c["acc_novel"], "exploratory", src("VOCAB-1", "vocab1_result.json", "cells." + key, p),
                        null={"kind": "shuffled labels, 95th percentile", "value": c["V1"]["null_p95"]},
                        note="provisional: the series marks this run unconverged; majority class %.3f" % d["majority_acc_test"]))
    d, p = read_k("edit2_result.json")
    for li, v in d["parallax_deg"].items():
        rows.append(row("gw.parallax", site(int(li), "mlp"), v, "exploratory", src("EDIT-2", "edit2_result.json", "parallax_deg." + li, p),
                        null={"kind": "random pair in 192 dimensions, 95 % band", "value": [81.9, 98.1]}, note="descriptive; the random band is from the series' notes, not this file"))
    for key, c in d["edits"].items():
        li, which, size = key.split("/")
        if size != "0.5":
            continue
        t = c["T_adj"]
        rows.append(row("gw.push." + which, site(int(li), "mlp"), t["mean"], "preregistered", src("EDIT-2", "edit2_result.json", "edits.%s.T_adj" % key, p),
                        interval=t["ci95"], n=t["n"], stake="failed" if which == "write" else None,
                        note="the stake (writing beats reading at every layer) failed; the measurement stands" if which == "write" else None))
    d, p = read_k("cal1_result_tanh.json")
    for name, c in d["R1"].items():
        rows.append(row("gw.knock", site(int(name[-1]), name[:-1]), c["corr"], "exploratory", src("CAL-1", "cal1_result_tanh.json", "R1." + name, p),
                        note="measured, not gated; mean realised change %.2f nats against %.3f predicted" % (c["mean_abs_real"], c["mean_abs_lin"])))
    d, p = read_h("f1.json")
    for c in d:
        rows.append(row("gm.read.kind", site(c["layer"], "state"), c["probe"], "preregistered", src("PROBE-HOLON-1 amendment 3", "f1.json", "[%d].probe" % c["layer"], p),
                        null={"kind": "chance", "value": 1 / 12}))
        rows.append(row("gm.lens.kind", site(c["layer"], "state"), c["lens"], "preregistered", src("PROBE-HOLON-1 amendment 3", "f1.json", "[%d].lens" % c["layer"], p),
                        null={"kind": "chance", "value": 1 / 12}))
    d, p = read_h("holon_obs.json")
    for c in d["asked"]["P2"]:
        rows.append(row("gm.closure", site(c["layer"], "state"), c["inc"], "preregistered", src("PROBE-HOLON-1 P2", "holon_obs.json", "asked.P2[%d].inc" % c["layer"], p),
                        null={"kind": "a view from shuffled labels", "value": c["shuf_inc"]}, stake="killed",
                        note="the view predicts its own write no better than a shuffled one (%.2f against %.2f)" % (c["view"], c["shuf_view"])))
    d, p = read_h("holon_int2.json")
    for key, c in d.items():
        if c["mode"] != "asked":
            continue
        rows.append(row("gm.remove", site(c["k"], "state"), c["kind"]["probe"], "preregistered", src("PROBE-HOLON-1 amendment 1", "holon_int2.json", key + ".kind.probe", p),
                        null={"kind": "a variance-matched subspace disjoint from the kinds removed instead (control B)", "value": c["B"]["probe"]},
                        note="clean %.3f; control B removed %.2f of the variance the kind subspace did" % (c["probe_clean"], c["B"]["var_train"])))
    d, p = read_h("attrib_Premises.json")
    for c in d:
        rows.append(row("gm.attrib", site(c["layer"], c["comp"]), c["direct"], "void", src("scope.py attribution", "attrib_Premises.json", "[layer %d %s].direct" % (c["layer"], c["comp"]), p),
                        category="Premises", note="void: the frame does not close on this file, so these do not sum to the answer"))
    return {"what": "SCOPE-0: what has been measured of two language models, site by site, copied from result files and never computed here",
            "models": MODELS, "layers": LAYERS, "standings": {
                "preregistered": "measured under a preregistration, with an interval or a null beside it",
                "exploratory": "measured while exploring, or provisional by the series' own label; a lead, not a finding",
                "void": "a number exists but its instrument does not hold here (the frame does not close, or the stake's plant failed); drawn as void, never as a value"},
            "rows": rows}


# ---------------------------------------------------------------- plants

def plants():
    base = {"models": MODELS, "layers": LAYERS, "rows": []}
    good = row("gw.identity", site(0, "mlp", "last"), 0.84, "preregistered", {"experiment": "E", "file": "f", "key": "k"}, null={"kind": "n", "value": 0.01})
    res = {}

    def refused(r, extra=None):
        t = dict(base, rows=[good] + ([extra] if extra else []) if r is None else [r])
        try:
            check(t)
            return False
        except Refuse:
            return True
    res["P-GOOD"] = check(dict(base, rows=[good])) == 1
    res["P-NOSITE"] = refused(dict(good, site=site(9, "mlp")))                     # GoldWorm has six layers
    res["P-NOCOMP"] = refused(dict(good, site=site(0, "ple")))                     # and no per-layer embedding
    res["P-NAN"] = refused(dict(good, value=float("nan")))
    res["P-NOSOURCE"] = refused(dict(good, source={"experiment": "E", "file": "", "key": "k"}))
    res["P-BARE"] = refused(dict(good, null=None))                                 # preregistered needs an interval or a null
    res["P-STANDING"] = refused(dict(good, standing="certified"))                  # not a word of this table
    res["P-TWICE"] = refused(None, extra=dict(good))
    res["P-OUTSIDE"] = refused(dict(good, interval=[0.1, 0.2]))
    att = row("gm.attrib", site(3, "attn"), -14.4, "exploratory", {"experiment": "E", "file": "f", "key": "k"})
    res["P-FRAME"] = refused(att) and not refused(dict(att, standing="void"))      # an attribution in an open frame is void or refused
    return res


def main():
    if sys.argv[1] == "plants":
        r = plants()
        for k, v in r.items():
            print("%-11s %s" % (k, "PASS" if v else "FAIL"))
        return 0 if all(r.values()) else 1
    ratchet, commit, handoff, out = sys.argv[2 - 1 + 0], sys.argv[2], sys.argv[3], sys.argv[4]
    if not all(plants().values()):
        print("a plant failed; nothing written")
        return 1
    used = {}

    def read_k(name):
        b = subprocess.run(["git", "-C", ratchet, "show", "%s:%s%s" % (commit, KS, name)], capture_output=True, check=True).stdout
        used["RATCHET:" + KS + name] = hashlib.sha256(b).hexdigest()
        return json.loads(b), "CIRISAI/RATCHET at %s" % commit[:12]

    def read_h(name):
        b = open(os.path.join(handoff, name), "rb").read()
        used["handoff:" + name] = hashlib.sha256(b).hexdigest()
        return json.loads(b), "kindscope hand-over bundle of 2026-11-11 (not a repository; sha256 in the sidecar)"
    table = build(read_k, read_h)
    n = check(table)
    os.makedirs(out, exist_ok=True)
    body = json.dumps(table, indent=1, ensure_ascii=False).encode()
    open(os.path.join(out, "scope_table.json"), "wb").write(body)
    per = {}
    for r in table["rows"]:
        per.setdefault(r["layer"], {}).setdefault(r["standing"], 0)
        per[r["layer"]][r["standing"]] += 1
    side = {"reference": "SCOPE-0 (what has been measured of two language models, by site)", "version": 1, "table": "scope_table.json", "bytes": len(body),
            "sha256": hashlib.sha256(body).hexdigest(), "rows": n, "rows_by_layer_and_standing": per,
            "serves": ["a model explorer: one exact kind of reading per map; a cell with no row is drawn as not measured"],
            "provenance": {"ratchet_commit": commit, "files": used},
            "verdict": {"all_rows": "unresolved", "reason": "copied readings of other sessions' experiments; nothing here was re-derived or scored by this repository"},
            "law": "a cell is a reading in a regime: its model, file, prompt, position and kind are part of the value. No cell is carried to a neighbouring site, to another model, or to another kind of reading.",
            "standings": table["standings"],
            "holds_no_real_feature": "no person or place; sites are layers and blocks of two public model files, values are aggregate statistics over prompts",
            "limits": ["two models, one checkpoint and one quantisation each", "GoldWorm rows are English children's stories at byte level; Gemma rows are 124 curated items",
                       "intervals are 95 % intervals as stored; no file stores a standard error", "the hand-over bundle is not under version control; its files are pinned by digest alone",
                       "which experiments a layer draws on was chosen by the CIRISHolon session; the RATCHET session owns the instrument and has not yet confirmed this selection"]}
    json.dump(side, open(os.path.join(out, "sidecar.json"), "w"), indent=1, ensure_ascii=False)
    print(n, "rows;", json.dumps(per))
    return 0


if __name__ == "__main__":
    sys.exit(main())
