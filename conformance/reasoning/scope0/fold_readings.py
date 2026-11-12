#!/usr/bin/env python3
"""SCOPE-0: how much room the model uses. Exploratory readings from the eight checked traces.

  fold_readings.py RATCHET_CHECKOUT RATCHET_COMMIT OUT.json

Per stage, for the state after it and for what the stage itself wrote, over the 1,944
characters of eight stories (opening marker left out): the EFFECTIVE number of directions in
use (participation ratio of the spread), how many directions hold 90 % and 99 % of it, and
each character's coordinates in that stage's OWN two widest directions (for a filmstrip).
Eight stories of one model. Nothing here was predicted first. Needs numpy.
"""
import json
import subprocess
import sys

import numpy as np

T = "experiments/kindscope/traces/"


def dims(A):
    A = A - A.mean(0)
    s = np.linalg.svd(A, compute_uv=False) ** 2
    p = s / s.sum()
    c = np.cumsum(p)
    return {"effective": round(float(1 / (p ** 2).sum()), 1), "for_90_percent": int(np.searchsorted(c, 0.90) + 1), "for_99_percent": int(np.searchsorted(c, 0.99) + 1)}


def main():
    ratchet, commit, out = sys.argv[1:4]
    show = lambda name: subprocess.run(["git", "-C", ratchet, "show", "%s:%s%s" % (commit, T, name)], capture_output=True, check=True).stdout
    man = json.loads(show("manifest.json"))
    W_, S_, place = [], [], []
    import importlib.util, os
    spec = importlib.util.spec_from_file_location("b", os.path.join(os.path.dirname(os.path.abspath(__file__)), "build_shape_table.py"))
    b = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(b)
    for it in man["items"]:
        raw = show(it["trace"])
        nl = raw.index(b"\n")
        h = raw[:nl].decode().split()
        P, W, K = int(h[3]), int(h[5]), int(h[7])
        v = np.frombuffer(raw[nl + 1:], dtype="<f4").reshape(P, K + 1, W).astype(np.float64)
        W_.append(v[13:, :K])
        S_.append(np.cumsum(v[:, :K], axis=1)[13:])
        rd = json.loads(show(it["trace"].replace(".trace", ".readings.json")))
        place += b.place_in_word(rd["bytes"])[13:]
    Wr, St = np.concatenate(W_), np.concatenate(S_)
    K = St.shape[1]
    res = {"stages": man["writes"], "characters": int(St.shape[0]), "width": int(St.shape[2]), "state": [], "write": [], "own_view": [], "place": place,
           "share_in_own_two_widest": []}
    for k in range(K):
        res["state"].append(dims(St[:, k]))
        res["write"].append(dims(Wr[:, k]))
        A = St[:, k] - St[:, k].mean(0)
        u, s, vt = np.linalg.svd(A, full_matrices=False)
        Y = A @ vt[:2].T
        Y = Y / np.abs(Y).max()
        res["own_view"].append([[round(float(a), 3), round(float(c), 3)] for a, c in Y])
        res["share_in_own_two_widest"].append(round(float((s[:2] ** 2).sum() / (s ** 2).sum()), 3))
    # how much of what the LAST stage writes lies along its single widest direction
    A = Wr[:, K - 1]
    s = np.linalg.svd(A, compute_uv=False) ** 2        # about zero, not the mean: is the write mostly one fixed push?
    res["last_stage_write_share_along_one_direction_uncentred"] = round(float(s[0] / s.sum()), 3)
    res["mean_write_share_of_squared_norm_by_stage"] = [round(float((Wr[:, k].mean(0) ** 2).sum() / (Wr[:, k] ** 2).sum(1).mean()), 3) for k in range(K)]
    json.dump(res, open(out, "w"))
    print("stage      state: eff  90%  99%   |  write: eff  90%  99%   | fixed part of the write | own 2D keeps")
    for k in range(K):
        a, w = res["state"][k], res["write"][k]
        print("%2d %-6s      %5.1f %4d %4d   |      %5.1f %4d %4d   |        %.2f          |   %.2f" % (k + 1, res["stages"][k], a["effective"], a["for_90_percent"], a["for_99_percent"],
              w["effective"], w["for_90_percent"], w["for_99_percent"], res["mean_write_share_of_squared_norm_by_stage"][k], res["share_in_own_two_widest"][k]))


if __name__ == "__main__":
    main()
