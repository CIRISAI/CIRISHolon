#!/usr/bin/env python3
"""SCOPE-0: the shape of what the model holds, stage by stage.

  build_shape_table.py RATCHET_CHECKOUT RATCHET_COMMIT TRACE_CHECK_BINARY OUT_DIR

For every character of the eight traced stories, the model's running state after each of
its 13 stages (the sum of the writes so far; each trace is first checked to conserve) is a
point in 192 dimensions. Each stage is shown along ITS OWN three widest directions, then
turned to sit as near the stage before as a rotation allows, so the cloud can be watched
moving from stage to stage. (A first version used one fixed set of directions for all
stages; it kept 8 to 36 % of the spread and drew a small blob.)

A projection to three dimensions shows some of the shape and hides the rest; the share of
spread it keeps is printed per stage, with how many directions the stage really uses and
whether its widest plane is a clock (position in the story). Needs numpy.
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
    keep = np.array([m["pos"] >= 13 for m in meta])                                # the opening marker is the same in each story
    X, meta = X[keep], [m for m, k_ in zip(meta, keep) if k_]
    N, K, W = X.shape
    pos = np.array([m["pos"] for m in meta], dtype=float)
    place = np.array([m["place"] for m in meta])
    byte = np.array([m["byte"] for m in meta])

    def between(A, labels):
        tot = ((A - A.mean(0)) ** 2).sum()
        return float(sum((labels == g).sum() * ((A[labels == g].mean(0) - A.mean(0)) ** 2).sum() for g in np.unique(labels)) / tot)

    def rank(v):
        r = np.empty(len(v))
        r[np.argsort(v, kind="stable")] = np.arange(len(v))
        return r
    Y, kept, dims, ring, sep_place, sep_char, sep_place_full = [], [], [], [], [], [], []
    prev = None
    for k in range(K):
        A = X[:, k] - X[:, k].mean(0)
        u, sv, vt = np.linalg.svd(A, full_matrices=False)
        p = sv ** 2 / (sv ** 2).sum()
        c = np.cumsum(p)
        dims.append({"effective": round(float(1 / (p ** 2).sum()), 1), "for_90_percent": int(np.searchsorted(c, 0.90) + 1), "for_99_percent": int(np.searchsorted(c, 0.99) + 1)})
        y = A @ vt[:3].T                                                           # this stage's OWN three widest directions
        y = y / np.sqrt((y ** 2).sum(1).mean())
        if prev is not None:                                                       # turn it to sit as near the last stage as a rotation allows
            uu, _, vv = np.linalg.svd(y.T @ prev)
            y = y @ (uu @ vv)
        prev = y
        Y.append(y)
        kept.append(float(p[:3].sum()))
        # is the widest plane a clock? angle around the centre of the two widest directions against position in the story
        a2 = A @ vt[:2].T
        ang = np.arctan2(a2[:, 1], a2[:, 0])
        o = np.sort(ang)
        gaps = np.diff(np.concatenate([o, [o[0] + 2 * np.pi]]))
        cut = o[(np.argmax(gaps) + 1) % len(o)]
        th = (ang - cut) % (2 * np.pi)
        ring.append(round(abs(float(np.corrcoef(rank(th), rank(pos))[0, 1])), 3))
        sep_place.append(round(between(y, place), 3))
        sep_char.append(round(between(y, byte), 3))
        sep_place_full.append(round(between(A, place), 3))
    Y = np.stack(Y, 1)
    table = {"what": "SCOPE-0 shapes: each character of eight stories as a point, after each of the model's 13 stages, seen along that stage's own three widest directions",
             "model": "goldworm-v6", "ratchet_commit": commit, "stages": man["writes"], "points": meta,
             "xyz": [[[round(float(c), 3) for c in Y[n, k]] for k in range(K)] for n in range(N)],
             "share_of_spread_kept_by_the_projection": [round(x, 3) for x in kept], "room_in_use": dims, "width": int(W),
             "clock": ring, "separation": {"place_in_word_3d": sep_place, "which_character_3d": sep_char, "place_in_word_192d": sep_place_full},
             "how": "per stage: centred; the stage's own three directions of greatest spread; each stage's picture is then turned (a rotation, no stretching) to sit as near the stage before as possible, so the cloud can be watched moving. 'clock' is the rank correlation between a point's angle around the two widest directions and its position in the story",
             "places": ["space or line break", "punctuation", "first letter of a word", "inside a word", "last letter of a word", "a one-letter word"]}
    os.makedirs(out, exist_ok=True)
    body = json.dumps(table, separators=(",", ":")).encode()
    open(os.path.join(out, "scope_shapes.json"), "wb").write(body)
    print(N, "points x", K, "stages;", len(body), "bytes")
    print("stage     kept3  eff.dim  90%  clock  by place  by char")
    for k in range(K):
        print("%2d %-6s %.2f   %5.1f  %4d   %.2f    %.2f     %.2f" % (k + 1, man["writes"][k], kept[k], dims[k]["effective"], dims[k]["for_90_percent"], ring[k], sep_place[k], sep_char[k]))


if __name__ == "__main__":
    main()
