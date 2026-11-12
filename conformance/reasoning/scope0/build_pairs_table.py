#!/usr/bin/env python3
"""SCOPE-0: a change travelling through the model.

  build_pairs_table.py RATCHET_CHECKOUT RATCHET_COMMIT PAIRS_DIR TRACE_CHECK_BINARY OUT_DIR

A kind is a type of CHANGE, so the object here is a pair: two real story openings that
differ in exactly one word (the RATCHET series' pairs; real text, identical up to the
change). For each pair and each of the model's 13 stages this computes ONE thing: how far
apart the model's state on the two versions is, at the changed word's last byte and at
each byte after it up to the end of the sentence.

"How far apart" is the distance between the two states divided by the typical distance
between the states of two unrelated characters at that stage (the yardstick), so 0 means
the same and about 1 means as different as unrelated text. Before the change the two
states must be exactly equal; that is checked for each pair, and the null pairs
(identical sentence) must read 0 throughout. Each trace's digest is checked against the
manifest and each trace must conserve. The summaries by sort use only pairs whose two
words are the same length: where lengths differ, each later byte also sits at a shifted
position, a second change mixed into the first. Nothing here assigns a kind. Needs numpy.
"""
import hashlib
import json
import os
import subprocess
import sys

import numpy as np


def main():
    ratchet, commit, pdir, checker, out = sys.argv[1:6]
    man = json.loads(subprocess.run(["git", "-C", ratchet, "show", "%s:experiments/kindscope/pairs/pairs.json" % commit], capture_output=True, check=True).stdout)
    K = len(man["writes"])

    def load(m):
        p = os.path.join(pdir, m["trace"])
        raw = open(p, "rb").read()
        assert hashlib.sha256(raw).hexdigest() == m["sha256"], "digest: " + m["trace"]
        r = subprocess.run([checker, p], capture_output=True, text=True)
        assert r.returncode == 0 and "tolerance 0e0" in r.stdout.split("\n")[0], "not conserved: " + m["trace"]
        nl = raw.index(b"\n")
        h = raw[:nl].decode().split()
        P, W, k = int(h[3]), int(h[5]), int(h[7])
        v = np.frombuffer(raw[nl + 1:], dtype="<f4").reshape(P, k + 1, W).astype(np.float64)
        return v[:, :k], np.cumsum(v[:, :k], axis=1)
    loaded = {}
    for p in man["pairs"]:
        loaded[p["pair_id"]] = [load(m) for m in p["members"]]
    # the yardstick: sqrt(2) x the RMS distance of a state from its stage's mean, over each member's bytes after the marker
    allS = np.concatenate([loaded[p["pair_id"]][i][1][15:] for p in man["pairs"] for i in (0, 1)])
    allW = np.concatenate([loaded[p["pair_id"]][i][0][15:] for p in man["pairs"] for i in (0, 1)])
    yard = np.sqrt(2) * np.sqrt(((allS - allS.mean(0)) ** 2).sum(2).mean(0))
    yardw = np.sqrt(2) * np.sqrt(((allW - allW.mean(0)) ** 2).sum(2).mean(0))
    pairs, worst_before = [], 0.0
    for p in man["pairs"]:
        (Wa, Sa), (Wb, Sb) = loaded[p["pair_id"]]
        a, b = p["members"]
        null = p["sort"].startswith("null")
        if null:
            n = min(a["window_end"], b["window_end"])
            d = np.sqrt(((Sa[:n] - Sb[:n]) ** 2).sum(2))
            pairs.append({"pair_id": p["pair_id"], "sort": "nothing changed", "null": True, "text": [a["text"][:n], b["text"][:n]],
                          "largest_difference": float(d.max()), "positions": int(n)})
            continue
        au = p["aligned_until"]
        before = float(np.abs(Sa[:au + 1] - Sb[:au + 1]).max())
        worst_before = max(worst_before, before)
        ea, eb = a["changed_span"][1], b["changed_span"][1]
        J = min(a["window_end"] - ea, b["window_end"] - eb)
        ia, ib = np.arange(ea - 1, ea + J), np.arange(eb - 1, eb + J)        # offset 0 is the changed word's last byte
        rel = np.sqrt(((Sa[ia] - Sb[ib]) ** 2).sum(2)) / yard                # (J + 1, K)
        relw = np.sqrt(((Wa[ia] - Wb[ib]) ** 2).sum(2)) / yardw
        same_after = [bool(x == y) for x, y in zip(a["bytes"][ea:ea + J], b["bytes"][eb:eb + J])]
        pairs.append({"pair_id": p["pair_id"], "sort": p["sort"], "null": False, "text": [a["text"], b["text"]], "changed_word": [a["changed_word"], b["changed_word"]],
                      "changed_span": [a["changed_span"], b["changed_span"]], "after": a["text"][ea:ea + J], "after_is_the_same_text": all(same_after), "same_length": bool(len(a["changed_word"]) == len(b["changed_word"])),
                      "largest_difference_before_the_change": before,
                      "state": [[round(float(x), 4) for x in row] for row in rel.T],      # [stage][offset]
                      "write": [[round(float(x), 4) for x in row] for row in relw.T]})
    sorts = {}
    for q in pairs:
        if q["null"] or not q["same_length"]:
            continue                      # a word of another length shifts each later byte's position, which is a second change
        s = sorts.setdefault(q["sort"], {"pairs": 0, "at_the_change_by_stage": np.zeros(K), "one_byte_after": np.zeros(K), "n_after": 0, "last_stage_by_offset": {}})
        s["pairs"] += 1
        st = np.array(q["state"])
        s["at_the_change_by_stage"] += st[:, 0]
        if st.shape[1] > 1:
            s["one_byte_after"] += st[:, 1]
            s["n_after"] += 1
        for j in range(st.shape[1]):
            s["last_stage_by_offset"].setdefault(j, []).append(float(st[-1, j]))
    summary = {k: {"pairs": v["pairs"], "at_the_change_by_stage": [round(float(x), 3) for x in v["at_the_change_by_stage"] / v["pairs"]],
                   "one_byte_after_by_stage": [round(float(x), 3) for x in v["one_byte_after"] / max(v["n_after"], 1)], "pairs_with_a_byte_after": v["n_after"],
                   "last_stage_by_bytes_after": {str(j): [round(float(np.mean(x)), 3), len(x)] for j, x in sorted(v["last_stage_by_offset"].items()) if len(x) >= 5}}
               for k, v in sorts.items()}
    table = {"what": "SCOPE-0 pairs: how far apart the model's state is on two real story openings that differ in one word, by stage and by bytes after the change",
             "model": "goldworm-v6", "ratchet_commit": commit, "stages": man["writes"], "device": man.get("device"),
             "yardstick": "distance between the two states, over sqrt(2) times the RMS distance of a state from its stage's mean (two unrelated characters read about 1)",
             "checks": {"largest_difference_before_any_change": worst_before, "largest_difference_in_a_null_pair": max(q["largest_difference"] for q in pairs if q["null"]),
                        "traces_conserved": 2 * len(man["pairs"])},
             "by_sort_uses": "pairs whose two words have the same number of bytes, so that nothing but the word differs; where lengths differ, each later byte also sits at a shifted position, a second change mixed into the first",
             "sorts_are": "the RATCHET session's description of what differs (a tagger's word class on the changed word); they are NOT kinds of change, and no kind is assigned here",
             "by_sort": summary, "pairs": pairs}
    os.makedirs(out, exist_ok=True)
    body = json.dumps(table, separators=(",", ":")).encode()
    open(os.path.join(out, "scope_pairs.json"), "wb").write(body)
    print(len(pairs), "pairs;", len(body), "bytes; largest difference before any change", worst_before, "; in a null pair", table["checks"]["largest_difference_in_a_null_pair"])
    for k, v in summary.items():
        print("%-14s n=%2d  at the change, stage 1..13: %s" % (k, v["pairs"], " ".join("%.2f" % x for x in v["at_the_change_by_stage"])))
        print("%-14s (%2d)  one byte after:            %s" % ("", v["pairs_with_a_byte_after"], " ".join("%.2f" % x for x in v["one_byte_after_by_stage"])))
        print("%-14s       last stage, by bytes after: %s" % ("", " ".join("%s:%.2f(%d)" % (j, x[0], x[1]) for j, x in v["last_stage_by_bytes_after"].items())))


if __name__ == "__main__":
    main()
