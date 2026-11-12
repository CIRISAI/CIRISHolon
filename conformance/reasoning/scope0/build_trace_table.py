#!/usr/bin/env python3
"""SCOPE-0: one prompt passing through a model. Builds the per-item table the explorer reads.

  build_trace_table.py RATCHET_CHECKOUT RATCHET_COMMIT TRACE_CHECK_BINARY OUT_DIR

For each trace of the RATCHET kindscope series at the given commit: the file's digest must be
the manifest's; the engine's `trace_check` must say CONSERVED at tolerance 0 (the writes, in
the order the model adds them, reproduce the state bit for bit) or the item is REFUSED and
not written. What is kept per position is small: the size of each block's write, the
series' own readings along its named directions, and how well the next byte was predicted.
The only thing computed here is the norm of each write. Standard library only.
"""
import array
import hashlib
import json
import math
import os
import subprocess
import sys
import tempfile

T = "experiments/kindscope/traces/"


def sig(x, n=4):
    return float("%.*g" % (n, x))


def main():
    ratchet, commit, checker, out = sys.argv[1:5]
    show = lambda name: subprocess.run(["git", "-C", ratchet, "show", "%s:%s%s" % (commit, T, name)], capture_output=True, check=True).stdout
    man = json.loads(show("manifest.json"))
    items, refused = [], []
    with tempfile.TemporaryDirectory() as td:
        for it in man["items"]:
            b = show(it["trace"])
            if hashlib.sha256(b).hexdigest() != it["trace_sha256"]:
                refused.append({"item": it["item"], "why": "the trace's digest is not the manifest's"})
                continue
            p = os.path.join(td, "t.trace")
            open(p, "wb").write(b)
            r = subprocess.run([checker, p], capture_output=True, text=True)
            first = r.stdout.split("\n")[0]
            if r.returncode != 0 or not first.startswith("CONSERVED") or "tolerance 0e0" not in first:
                refused.append({"item": it["item"], "why": first})
                continue
            nl = b.index(b"\n")
            h = b[:nl].decode().split()
            P, W, K = int(h[3]), int(h[5]), int(h[7])
            names = h[8:]
            v = array.array("f")
            v.frombytes(b[nl + 1:])
            if sys.byteorder != "little":
                v.byteswap()
            norms = []
            for pos in range(P):
                base = pos * (K + 1) * W
                norms.append([sig(math.sqrt(sum(x * x for x in v[base + k * W: base + (k + 1) * W]))) for k in range(K)])
            rd = json.loads(show(it["trace"].replace(".trace", ".readings.json")))
            fitted = [bool(x) for x in rd["fitted_site"]]
            z = rd["z_readings_after_mlp"]
            # readings are kept ONLY at the sites their directions were fitted on; elsewhere the projection means nothing
            readings = {name: {L: [(sig(z[L][name][i]) if fitted[i] and z[L][name][i] is not None else None) for i in range(P)] for L in sorted(z)}
                        for name in ("probe", "write", "resid", "next_byte_forecast")}
            att = json.loads(show(it["trace"].replace(".trace", ".attention.json")))["layers"]
            wiring = {L: [[[int(j), sig(w, 3)] for j, w in row] for row in att[L]["top5_mean_over_heads"]] for L in sorted(att)}
            items.append({"item": it["item"], "looks_back_at": wiring, "source": rd["source"], "bytes": rd["bytes"], "positions": P, "writes": names,
                          "conserved": first, "c4_max_abs_logit_residual": max(rd["c4_max_abs_logit_residual"]),
                          "write_norm": norms, "fitted_site": fitted, "readings_after_mlp": readings, "next_byte_logprob_true": [sig(x) for x in rd["next_byte_logprob_true"]],
                          "trace_sha256": it["trace_sha256"]})
    cal = json.loads(subprocess.run(["git", "-C", ratchet, "show", "%s:experiments/kindscope/cal1_result_tanh.json" % commit], capture_output=True, check=True).stdout)["R1"]
    cost = {k: {"realised": sig(v["mean_abs_real"]), "direct": sig(v["mean_abs_lin"])} for k, v in cal.items()}
    table = {"knock_out_cost": cost, "what": "SCOPE-0 traces: one prompt passing through GoldWorm v6, position by position. Each item was checked: the writes sum to the state bit for bit.",
             "model": "goldworm-v6", "ratchet_commit": commit, "directions_from": man.get("directions_from"), "items": items, "refused": refused,
             "kinds": {"write_norm": "the length (L2 norm) of what each block added to the stream at this position, computed here from the trace",
                       "readings": "standard deviations from a typical fitted site, per layer and direction; present ONLY at the byte before a word that follows a determiner, where the directions were fitted; null elsewhere",
                       "probe": "along the direction a probe READS adjective-or-noun from; positive points toward an adjective next",
                       "write": "along the direction the model WRITES it with; positive points toward an adjective next",
                       "resid": "along the part of the writing direction that is not a next-byte forecast",
                       "next_byte_forecast": "along the forecast direction of the byte that actually comes next, when it is one of the 16 fitted",
                       "looks_back_at": "for each looking-back (attention) stage and position: the five earlier positions it draws on most, with the weight averaged over the stage's 4 heads (the series' export)",
                       "knock_out_cost": "CAL-1 R1: mean change in the true next byte's log-probability (nats) when that block's write is zeroed, beside what its direct attribution predicted",
                       "next_byte_logprob_true": "log-probability the model gave the byte that came next (nats)"}}
    os.makedirs(out, exist_ok=True)
    body = json.dumps(table, separators=(",", ":")).encode()
    open(os.path.join(out, "scope_traces.json"), "wb").write(body)
    json.dump({"reference": "SCOPE-0 traces (one prompt passing through a model)", "version": 3, "changes": "version 3 (2026-11-12): what each looking-back stage draws on is added (top five sources per position). Version 2 (2026-11-12), on the instrument owner's corrections: readings were drawn at every byte though their directions mean something only at the sites they were fitted on, and were raw projections not comparable across layers; they are now z-scores kept at fitted sites alone; the third direction was mislabelled; the knock-out cost of each block is added, because the length of a write is not how much it matters", "table": "scope_traces.json", "bytes": len(body), "sha256": hashlib.sha256(body).hexdigest(),
               "items": len(items), "refused": refused, "provenance": {"ratchet_commit": commit, "traces": {str(i["item"]): i["trace_sha256"] for i in items}},
               "check": "each item passed holon-closure's trace_check at tolerance 0 before it was written; an item that does not conserve is refused and listed",
               "verdict": {"all_rows": "unresolved", "reason": "readings of another session's instrument on eight unseen stories; nothing scored here"},
               "holds_no_real_feature": "no person or place; eight stories from a public corpus of generated children's stories, and aggregate numbers per position",
               "limits": ["eight items of 256 bytes each", "the readings' directions were fitted by the RATCHET session on other stories", "the full writes (192 numbers each) are not in this table, only their lengths"]},
              open(os.path.join(out, "traces_sidecar.json"), "w"), indent=1)
    print(len(items), "items written,", len(refused), "refused;", len(body), "bytes")
    return 0 if items and not refused else 1


if __name__ == "__main__":
    sys.exit(main())
