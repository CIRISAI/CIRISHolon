#!/usr/bin/env python3
"""SCOPE-0: the shape of what the model holds, stage by stage.

  build_shape_table.py RATCHET_CHECKOUT RATCHET_COMMIT TRACE_CHECK_BINARY OUT_DIR

For every character of the eight traced stories, the model's running state after each of
its 13 stages (the sum of the writes so far; each trace is first checked to conserve) is a
point in 192 dimensions. This projects all of them onto ONE fixed set of three directions,
so the same cloud can be watched changing shape from stage to stage.

What is computed here, and it is a choice: each stage's cloud is centred and scaled to unit
average size (the state grows with depth), and the three directions are the three of
greatest spread over all stages pooled. A projection to three dimensions shows some of the
shape and hides the rest; the share of spread it keeps is printed per stage. Needs numpy.
"""
import hashlib
import json
import os
import subprocess
import sys
import tempfile

import numpy as np

T = "experiments/kindscope/traces/"


def place_in_word(b):
    """0 space or line break, 1 punctuation or other, 2 first letter, 3 inside, 4 last letter, 5 a one-letter word."""
    letter = [(65 <= c <= 90) or (97 <= c <= 122) or c == 39 for c in b]
    out = []
    for i, c in enumerate(b):
        if c in (32, 10):
            out.append(0)
        elif not letter[i]:
            out.append(1)
        else:
            first = i == 0 or not letter[i - 1]
            last = i == len(b) - 1 or not letter[i + 1]
            out.append(5 if first and last else 2 if first else 4 if last else 3)
    return out


def main():
    ratchet, commit, checker, out = sys.argv[1:5]
    show = lambda name: subprocess.run(["git", "-C", ratchet, "show", "%s:%s%s" % (commit, T, name)], capture_output=True, check=True).stdout
    man = json.loads(show("manifest.json"))
    states, meta = [], []
    with tempfile.TemporaryDirectory() as td:
        for it in man["items"]:
            b = show(it["trace"])
            assert hashlib.sha256(b).hexdigest() == it["trace_sha256"], "digest"
            p = os.path.join(td, "t.trace")
            open(p, "wb").write(b)
            r = subprocess.run([checker, p], capture_output=True, text=True)
            assert r.returncode == 0 and r.stdout.startswith("CONSERVED") and "tolerance 0e0" in r.stdout.split("\n")[0], "not conserved: item %s" % it["item"]
            nl = b.index(b"\n")
            h = b[:nl].decode().split()
            P, W, K = int(h[3]), int(h[5]), int(h[7])
            v = np.frombuffer(b[nl + 1:], dtype="<f4").reshape(P, K + 1, W)
            states.append(np.cumsum(v[:, :K, :].astype(np.float64), axis=1))       # (P, K, W): the state after each stage
            rd = json.loads(show(it["trace"].replace(".trace", ".readings.json")))
            lp = rd["next_byte_logprob_true"] + [0.0]
            pw = place_in_word(rd["bytes"])
            for i in range(P):
                meta.append({"story": it["item"], "pos": i, "byte": rd["bytes"][i], "place": pw[i], "surprise": round(-lp[i], 2)})
    X = np.concatenate(states, 0)                                                  # (N, K, W)
    N, K, W = X.shape
    keep = np.array([m["pos"] >= 13 for m in meta])                                # the opening marker is the same in each story
    Z = np.empty_like(X)
    for k in range(K):
        c = X[keep, k].mean(0)
        s = np.sqrt(((X[keep, k] - c) ** 2).sum(1).mean())
        Z[:, k] = (X[:, k] - c) / s
    pooled = Z[keep].reshape(-1, W)
    _, sv, vt = np.linalg.svd(pooled - pooled.mean(0), full_matrices=False)
    B = vt[:3]
    Y = Z @ B.T                                                                    # (N, K, 3)
    kept = [float((Y[keep, k] ** 2).sum() / (Z[keep, k] ** 2).sum()) for k in range(K)]

    def separation(labels, k, full):
        """Share of the cloud's spread that lies BETWEEN the groups (0 none, 1 all), at stage k."""
        A = (Z if full else Y)[keep, k]
        L = np.asarray(labels)[keep]
        tot = ((A - A.mean(0)) ** 2).sum()
        btw = sum((L == g).sum() * ((A[L == g].mean(0) - A.mean(0)) ** 2).sum() for g in np.unique(L))
        return float(btw / tot)
    place = [m["place"] for m in meta]
    byte = [m["byte"] for m in meta]
    sep = {"place_in_word_3d": [round(separation(place, k, False), 3) for k in range(K)], "place_in_word_192d": [round(separation(place, k, True), 3) for k in range(K)],
           "which_character_3d": [round(separation(byte, k, False), 3) for k in range(K)], "which_character_192d": [round(separation(byte, k, True), 3) for k in range(K)]}
    table = {"what": "SCOPE-0 shapes: each character of eight stories as a point, after each of the model's 13 stages, in one fixed three-dimensional projection",
             "model": "goldworm-v6", "ratchet_commit": commit, "stages": man["writes"], "points": meta,
             "xyz": [[[round(float(c), 3) for c in Y[n, k]] for k in range(K)] for n in range(N)],
             "share_of_spread_kept_by_the_projection": [round(x, 3) for x in kept], "separation": sep,
             "how": "per stage: centred and scaled to unit average size; three directions of greatest spread over all stages pooled; the opening marker's 13 positions are projected but not used to choose directions",
             "places": ["space or line break", "punctuation", "first letter of a word", "inside a word", "last letter of a word", "a one-letter word"]}
    os.makedirs(out, exist_ok=True)
    body = json.dumps(table, separators=(",", ":")).encode()
    open(os.path.join(out, "scope_shapes.json"), "wb").write(body)
    print(N, "points x", K, "stages;", len(body), "bytes")
    print("share of spread kept in 3D by stage:", [round(x, 2) for x in kept])
    for k, v in sep.items():
        print(k, v)


if __name__ == "__main__":
    main()
