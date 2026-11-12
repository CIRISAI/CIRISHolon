#!/usr/bin/env python3
"""SCOPE-0: the map and the motion in one table.

  build_mapmotion_table.py RATCHET_CHECKOUT RATCHET_COMMIT OUT_DIR [CROSS_COMMIT]

Reads the RATCHET series' `experiments/kindscope/map_motion.json` (exploratory, GoldWorm)
and writes `scope_mapmotion.json`, the part the explorer draws:

- the MAP: for each of 13 stages and 10 parts of speech, how much better than the base
  rate a linear reading tells that category at a word's last byte (held-out accuracy
  minus base rate; the directions themselves stay in the RATCHET file);
- the MOTION: for each real pair of story openings that differ in one word of equal
  length, by stage and by byte from the changed word's first byte, how far apart the
  model holds the two (over the mean distance of two unrelated openings at that stage,
  so about 1 is unrelated), and the share of that difference's squared size that lies
  inside the map's ten directions;
- the FOOTPRINT: at the changed word's last byte, the cosine of the difference with each
  category's reading direction, beside the same for unrelated openings (the floor).

With CROSS_COMMIT it adds the CROSS-1 rows (`cross1_result.json`): real pairs whose two
words are of DIFFERENT parts of speech, grouped by distinct word pair (one word pair
counted once: 94 of the 173 pairs are "lived" / "was"), with, by stage, the cosine of the
state difference at the word's last byte with the map's (category A minus category B)
axis, and that test's same-category floor.

Nothing is fitted here. Divisions and means only. No kind is assigned.
"""
import json
import os
import statistics as st
import subprocess
import sys

PREFIX = len("<|endoftext|>\n\n")
PLAIN = {"NOUN": "thing", "VERB": "action", "ADJ": "describing", "ADV": "how", "PRON": "he/she/it",
         "DET": "the/a", "ADP": "in/on/to", "AUX": "is/was", "CCONJ": "and/but", "PROPN": "name"}


def main():
    repo, commit, out = sys.argv[1], sys.argv[2], sys.argv[3]
    raw = subprocess.run(["git", "-C", repo, "show", commit + ":experiments/kindscope/map_motion.json"],
                         check=True, capture_output=True).stdout
    m = json.loads(raw)
    S, C = m["states"], m["categories"]
    K = len(S)
    sc = m["map"]["scores"]
    lift = [[round(sc[s + "/" + c]["acc"] - sc[s + "/" + c]["base_rate"], 3) for c in C] for s in S]
    nulls = m["motion"]["null_unrelated_openings"]
    yard = [st.mean(v for n in nulls for v in n["norm"][k]) for k in range(K)]
    floor = [[round(st.mean(n["abs_proj_probe_mean"][k][c] for n in nulls) / yard[k], 3) for c in range(len(C))] for k in range(K)]
    pairs = []
    for p in m["motion"]["pairs"]:
        a, b = p["changed_span"]
        w = b - a
        assert len(p["a_word"]) == len(p["b_word"]) == w, p["pair_id"]
        ta = p["text_a"]
        assert p["from_byte"] == a, p["pair_id"]
        s0 = a - PREFIX                   # the spans count the model's 15-byte opening marker
        assert ta[s0:s0 + w] == p["a_word"], p["pair_id"]
        J = len(p["norm"][0])
        foot = []
        for k in range(K):
            n = p["norm"][k][w - 1]
            foot.append([round(x / n, 3) if n > 1e-9 else 0.0 for x in p["proj_probe"][k][w - 1]])
        pairs.append({
            "id": p["pair_id"], "sort": p["sort"], "words": [p["a_word"], p["b_word"]],
            "text": [ta, ta[:s0] + p["b_word"] + ta[s0 + w:]], "at": s0, "len": w,
            "chars": [ta[s0 + j] if s0 + j < len(ta) else "" for j in range(J)],
            "state": [[round(v / yard[k], 3) for v in p["norm"][k]] for k in range(K)],
            "share": [[round(v, 3) for v in p["share_in_map"][k]] for k in range(K)],
            "foot": foot,
        })
    doc = {
        "what": "SCOPE-0 map and motion: where parts of speech can be read, and how one-word changes move relative to those readings",
        "model": m["model"], "status": "exploratory; RATCHET's table, redrawn", "ratchet_commit": commit,
        "stages": S, "categories": C, "plain": [PLAIN[c] for c in C],
        "lift": lift, "floor": floor, "yardstick": [round(y, 3) for y in yard],
        "yardstick_is": "mean distance between the states of two unrelated openings at that stage (200 pairs)",
        "share_by_stage": m["motion"]["summary"]["share_in_map_pairs_by_state"],
        "share_unrelated_by_stage": m["motion"]["summary"]["share_in_map_null_by_state"],
        "share_random": m["motion"]["summary"]["random_10_of_192_expectation"],
        "pairs_are": "real pairs whose two words have the same length AND the same part of speech, so the map's category does not change",
        "pairs": pairs,
    }
    if len(sys.argv) > 4:
        cc = sys.argv[4]
        x = json.loads(subprocess.run(["git", "-C", repo, "show", cc + ":experiments/kindscope/cross1_result.json"], check=True, capture_output=True).stdout)
        assert x["states"] == S
        groups = {}
        for q in x["pairs"]:
            groups.setdefault((q["a_word"], q["b_word"], q["a_pos"], q["b_pos"]), []).append(q)
        rows = []
        for (aw, bw, ap, bp), g in groups.items():
            if ap not in C or bp not in C:
                continue
            rows.append({"words": [aw, bw], "types": [C.index(ap), C.index(bp)], "n": len(g),
                         "equal_length": all(q["equal_length"] for q in g),
                         "align": [round(st.mean(q["align_meandiff"][k] for q in g), 3) for k in range(K)],
                         "share": [round(st.mean(q["share_in_map"][k] for q in g), 3) for k in range(K)]})
        open_class = {C.index("ADJ"), C.index("NOUN")}
        rows.sort(key=lambda r: (set(r["types"]) != open_class, not r["equal_length"], r["n"] > 5, r["words"]))
        doc["cross"] = {
            "ratchet_commit": cc, "prereg_sha256": x["prereg_sha256"], "pairs_read": x["n_cross"], "distinct_word_pairs": len(rows),
            "floor_p95": [round(x["by_state"][s]["floor"]["p95_of_draw_means"], 3) for s in S],
            "adj_noun": [[round(v, 3) for v in [x["by_state"][s]["by_pair"]["ADJ/NOUN"]["mean"]] + x["by_state"][s]["by_pair"]["ADJ/NOUN"]["ci95"]] for s in S],
            "equal_length": [[round(v, 3) for v in [x["by_state"][s]["equal_length_only"]["mean"]] + x["by_state"][s]["equal_length_only"]["ci95"]] for s in S],
            "share_by_stage": [round(x["by_state"][s]["share_in_map_cross"], 3) for s in S],
            "rows": rows,
        }
    path = os.path.join(out, "scope_mapmotion.json")
    json.dump(doc, open(path, "w", encoding="utf-8"), ensure_ascii=False, separators=(",", ":"))
    print(path, os.path.getsize(path), "bytes;", len(pairs), "pairs; largest lift", max(max(r) for r in lift))


if __name__ == "__main__":
    main()
