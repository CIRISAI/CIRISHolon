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

With CROSS_COMMIT it adds CROSS-1 (`cross1_result.json` for the frozen test's floor, and
the exploratory re-read `cross1_reread.json`): real pairs whose two words the tagger gave
DIFFERENT parts of speech, one row per distinct word pair, with, by stage, the cosine of
the state difference at the word's last byte with the map's (category A minus category
B) axis, one opening's wake by byte, and the re-read's flag where a reader disputes the
tagger's label (44 of 67; one reader's list).

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
        rr = json.loads(subprocess.run(["git", "-C", repo, "show", cc + ":experiments/kindscope/cross1_reread.json"], check=True, capture_output=True).stdout)
        assert rr["states"] == S and rr["categories"] == C
        key = lambda q: (q["a_word"], q["b_word"], q["a_pos"], q["b_pos"])
        wakes = {key(q): q for q in rr["wakes"]}
        rows = []
        for q in rr["distinct_word_pairs"]:
            wk = wakes.get(key(q))
            if wk is None:
                continue                      # no by-byte wake was written for this word pair
            (a0, a1), (b0, b1) = wk["changed_span_a"], wk["changed_span_b"]
            ta, tb = wk["text_a"], wk["text_b"]
            assert ta[a0 - PREFIX:a1 - PREFIX] == q["a_word"] and tb[b0 - PREFIX:b1 - PREFIX] == q["b_word"], key(q)
            assert wk["offsets_from_word_end"][0] == 0
            end = a1 - PREFIX - 1
            rows.append({"words": [q["a_word"], q["b_word"]], "types": [C.index(q["a_pos"]), C.index(q["b_pos"])], "n": q["occurrences"],
                         "disputed": q.get("disputed") or None, "align": [round(v, 3) for v in q["align_meandiff"]],
                         "text": [ta, tb], "at": [a0 - PREFIX, b0 - PREFIX], "wlen": [a1 - a0, b1 - b0],
                         "chars": [(ta[end + o] if end + o < len(ta) else "") for o in wk["offsets_from_word_end"]],
                         "state": [[round(v / yard[k], 3) for v in wk["norm"][k]] for k in range(K)],
                         "share": [[round(v, 3) for v in wk["share_in_map"][k]] for k in range(K)]})
        open_class = {C.index("ADJ"), C.index("NOUN")}
        rows.sort(key=lambda r: (r["disputed"] is not None, set(r["types"]) != open_class, r["n"] > 5, r["words"]))
        tri = lambda name: [([round(v, 3) for v in [rr["summary"][s][name]["mean"]] + rr["summary"][s][name]["ci95"]] if s in rr["summary"] else None) for s in S]   # the re-read summarises four stages
        doc["cross"] = {
            "ratchet_commit": cc, "prereg_sha256": x["prereg_sha256"], "pairs_read": x["n_cross"],
            "distinct_word_pairs": rr["counts"]["distinct"], "disputed": rr["counts"]["disputed"], "drawn": len(rows),
            "status": rr["status"],
            "floor_p95": [round(x["by_state"][s]["floor"]["p95_of_draw_means"], 3) for s in S],
            "adj_noun_undisputed": tri("adj_noun_undisputed"), "adj_noun_disputed": tri("adj_noun_disputed"),
            "summary_stages": [s for s in S if s in rr["summary"]],
            "n": {k: rr["summary"][S[0]][k]["n"] for k in ("adj_noun_undisputed", "adj_noun_disputed", "all_undisputed", "all_disputed")},
            "rows": rows,
        }
    path = os.path.join(out, "scope_mapmotion.json")
    json.dump(doc, open(path, "w", encoding="utf-8"), ensure_ascii=False, separators=(",", ":"))
    print(path, os.path.getsize(path), "bytes;", len(pairs), "pairs; largest lift", max(max(r) for r in lift))


if __name__ == "__main__":
    main()
