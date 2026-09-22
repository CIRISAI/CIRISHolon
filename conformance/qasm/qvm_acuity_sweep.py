#!/usr/bin/env python3
"""QVM-ACUITY-1's sweep: the prereg's instance grid through `qvm_acuity`,
on cores 21-27, writing `qvm_acuity_results.json` and S5's comparison table.

QVM_ACUITY1_PREREG.md, §1: random Clifford+T circuits, `n in {12,16,20,24}`,
`t in {8,12,16,20,24,28}`, T gates placed at random among `20n` Clifford
gates, five seeds each; the observable is one amplitude and one 4-qubit
marginal; the acuity `eps in {1e-1, 1e-2, 1e-3}`. Referees: the exact
statevector for `n <= 20`, the FULL exact branch sum wherever `N(t) <= 3^6*2^2`.

S5's table is what this writes: the budgeted sum's wall beside the full sum's
beside the statevector's, at every `(n, t, eps)`, plus the measured exponent
of the full sum against the published `2^{0.396 t}`.

WHICH BACKEND RAN IS PRINTED IN EVERY ROW. If `holon::acuity` was not linked
into the driver, the driver falls back to the full sum (every branch, `eps`
ignored, remainder 0) and the budgeted and full-sum columns are THE SAME
MEASUREMENT: the table says so in its header and the JSON says so in
`backend`. A fallback run is not evidence about acuity; it is evidence about
the locate/price/sum pipeline and about the two referees.

Cores: every subprocess is `taskset -c 21-27` — campaigns own 0-20 and 28.
"""
import json
import os
import statistics
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
DRIVER = os.environ.get(
    "QVM_ACUITY",
    os.path.join(HERE, "..", "..", "engine", "target", "release", "qvm_acuity"),
)
CORES = os.environ.get("QVM_ACUITY_CORES", "21-27")
NS = [int(x) for x in os.environ.get("QVM_ACUITY_N", "12,16,20").split(",")]
TS = [int(x) for x in os.environ.get("QVM_ACUITY_T", "8,12,16,20,24,28").split(",")]
EPS = [float(x) for x in os.environ.get("QVM_ACUITY_EPS", "1e-1,1e-2,1e-3").split(",")]
SEEDS = [int(x) for x in os.environ.get("QVM_ACUITY_SEEDS", "1,2,3,4,5").split(",")]
SHARDS = int(os.environ.get("QVM_ACUITY_SHARDS", "1"))
MAX_REFEREE = int(os.environ.get("QVM_ACUITY_MAX_REFEREE", "20"))


def run(args):
    """One driver invocation, pinned. Returns (sector_line, price_line, dict)."""
    cmd = ["taskset", "-c", CORES, DRIVER] + args
    p = subprocess.run(cmd, capture_output=True, text=True)
    if p.returncode != 0:
        raise RuntimeError(f"{' '.join(cmd)}\n{p.stderr}")
    lines = [l for l in p.stdout.strip().split("\n") if l]
    return lines[0], lines[1], json.loads(lines[-1])


def med(xs):
    xs = [x for x in xs if x is not None]
    return statistics.median(xs) if xs else None


def main():
    if not os.path.exists(DRIVER):
        sys.exit(
            f"{DRIVER} is not built. Build it with\n"
            f"  taskset -c {CORES} cargo build --release -p holon --bin qvm_acuity\n"
            f"from engine/, or set QVM_ACUITY."
        )
    loadavg = os.getloadavg()
    print(
        f"QVM-ACUITY-1 sweep: n={NS} t={TS} eps={EPS} seeds={SEEDS} shards={SHARDS} "
        f"on cores {CORES}; load average {loadavg[0]:.2f}"
    )
    runs = []
    backend = None
    t0 = time.perf_counter()

    for n in NS:
        for t in TS:
            for seed in SEEDS:
                base = ["--random", str(n), str(t), str(seed),
                        "--shards", str(SHARDS), "--max-referee", str(MAX_REFEREE)]
                amp = ["--amp-argmax"] if n <= MAX_REFEREE else ["--amp-skeleton"]
                # the FULL sum, eps = 0: the prereg's second referee and S2's
                # own arm (every branch evaluated).
                _, _, full = run(base + amp + ["--eps", "0", "--label", "full"])
                full["obs"] = "amp"
                full["arm"] = "full"
                runs.append(full)
                backend = backend or full["backend"]
                for eps in EPS:
                    _, _, r = run(base + amp + ["--eps", repr(eps), "--label", "budgeted"])
                    r["obs"] = "amp"
                    r["arm"] = "budgeted"
                    runs.append(r)
                # the MARGINAL: located and priced (the sector's own numbers).
                # Its value is a 2^{|cone|-4}-fold sum of amplitudes, which the
                # driver reports as null above --marginal-cap; the sector, the
                # price and the removed count are what this arm is for.
                _, _, m = run(base + ["--marginal", "0,1,2,3", "--marginal-cap", "16"])
                m["obs"] = "marginal"
                m["arm"] = "sector"
                runs.append(m)
                print(
                    f"  n={n:3d} t={t:3d} seed={seed}  t_eff={full['t_eff']:3d} "
                    f"N_pred={full['N_pred']:6d} removed(marg)={m['removed']:2d} "
                    f"cone={m['light_cone']:3d}  sum={full['wall_source_s']+full['wall_sum_s']:8.4f}s "
                    f"sv={full['wall_referee_s'] if full['wall_referee_s'] else float('nan'):8.4f}s "
                    f"err={full['abs_error']}",
                    flush=True,
                )
    wall = time.perf_counter() - t0

    # ---------------- S5's table ----------------
    table = []
    for n in NS:
        for t in TS:
            fulls = [r for r in runs if r["obs"] == "amp" and r["arm"] == "full"
                     and r["n"] == n and r["t"] == t]
            if not fulls:
                continue
            sv_wall = med([r["wall_referee_s"] for r in fulls])
            full_wall = med([r["wall_source_s"] + r["wall_sum_s"] for r in fulls])
            margs = [r for r in runs if r["obs"] == "marginal" and r["n"] == n and r["t"] == t]
            for eps in EPS:
                bs = [r for r in runs if r["obs"] == "amp" and r["arm"] == "budgeted"
                      and r["n"] == n and r["t"] == t and r["eps"] == eps]
                if not bs:
                    continue
                table.append({
                    "n": n, "t": t, "eps": eps,
                    "t_eff": fulls[0]["t_eff"],
                    "N_pred": fulls[0]["N_pred"],
                    "evaluated_med": med([r["evaluated"] for r in bs]),
                    "fraction_med": med([r["evaluated"] / r["N_pred"] for r in bs]),
                    "budgeted_wall_s": med([r["wall_source_s"] + r["wall_sum_s"] for r in bs]),
                    "full_sum_wall_s": full_wall,
                    "statevector_wall_s": sv_wall,
                    "abs_error_max": max([r["abs_error"] for r in bs if r["abs_error"] is not None],
                                         default=None),
                    "remainder_max": max([r["remainder"] for r in bs if r["remainder"] is not None],
                                         default=None),
                    "removed_marginal_med": med([r["removed"] for r in margs]),
                    "cone_med": med([r["light_cone"] for r in margs]),
                })

    # ------------- the full sum's measured exponent -------------
    # S5: the published scaling is 2^{0.396 t}. TWO slopes are reported, and
    # the distinction is the whole point:
    #
    #   * BRANCHES — log2(N_pred) against t. This is the rate the published
    #     exponent is a statement about, and it is what `expected_branches`
    #     realises.
    #   * WALL — log2(seconds) against t, which is the branch rate PLUS the
    #     per-branch cost, and the per-branch cost grows with t because the
    #     affine state each branch evolves is n + t wide. Quoting the wall
    #     slope against the published exponent charges the decomposition for
    #     the carrier's width.
    #
    # The third column is their difference: the per-branch overhead's own
    # slope, which is where a gap, if there is one, actually lives.
    import math

    def slope(pts):
        xs = [x for x, _ in pts]
        ys = [math.log2(y) for _, y in pts]
        mx, my = sum(xs) / len(xs), sum(ys) / len(ys)
        den = sum((x - mx) ** 2 for x in xs)
        return sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / den

    expo = []
    for n in NS:
        rows_n = [r for r in runs if r["obs"] == "amp" and r["arm"] == "full" and r["n"] == n]
        wall_pts, br_pts = [], []
        for t in TS:
            here = [r for r in rows_n if r["t"] == t]
            if not here:
                continue
            w = med([r["wall_source_s"] + r["wall_sum_s"] for r in here])
            if w and w > 0:
                wall_pts.append((t, w))
                br_pts.append((t, here[0]["N_pred"]))
        if len(wall_pts) >= 2:
            sw, sb = slope(wall_pts), slope(br_pts)
            expo.append({
                "n": n,
                "slope_log2_branches_per_T": sb,
                "slope_log2_wall_per_T": sw,
                "slope_per_branch_overhead": sw - sb,
                "wall_points": wall_pts,
                "branch_points": br_pts,
            })

    out = {
        "prereg": "QVM_ACUITY1_PREREG.md",
        "driver": os.path.abspath(DRIVER),
        "backend": backend,
        "backend_is_fallback": backend == "full-sum-fallback",
        "cores": CORES,
        "shards": SHARDS,
        "loadavg": list(loadavg),
        "grid": {"n": NS, "t": TS, "eps": EPS, "seeds": SEEDS},
        "sweep_wall_s": wall,
        "runs": runs,
        "table": table,
        "full_sum_exponent": expo,
    }
    jpath = os.path.join(HERE, "qvm_acuity_results.json")
    json.dump(out, open(jpath, "w"), indent=1)

    md = [
        "# QVM-ACUITY-1 — S5's comparison table",
        "",
        f"`{os.path.basename(DRIVER)}`, backend `{backend}`, shards {SHARDS}, cores {CORES}, "
        f"load average {loadavg[0]:.2f}, sweep wall {wall:.1f}s.",
        "",
    ]
    if backend == "full-sum-fallback":
        md += [
            "> **The budgeted column IS the full-sum column.** `holon::acuity` was not linked",
            "> into this driver, so every branch was evaluated, `eps` was ignored and the",
            "> remainder is certified `0` because nothing was left out. The two wall columns are",
            "> two measurements of the same work and their difference is run-to-run noise. What",
            "> this table does say: the located sector, the price stated before the run, the",
            "> exact branch sum against the statevector, and the walls of both referees.",
            "",
        ]
    md += [
        "| n | t | t_eff | eps | N_pred | evaluated | k/N | budgeted wall (s) | "
        "full-sum wall (s) | statevector wall (s) | max abs err | max remainder | "
        "removed (marginal) | cone |",
        "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|",
    ]
    for r in table:
        def f(x, k=6):
            return "null" if x is None else f"{x:.{k}g}"
        md.append(
            f"| {r['n']} | {r['t']} | {r['t_eff']} | {r['eps']:.0e} | {r['N_pred']} | "
            f"{f(r['evaluated_med'])} | {f(r['fraction_med'], 3)} | {f(r['budgeted_wall_s'], 4)} | "
            f"{f(r['full_sum_wall_s'], 4)} | {f(r['statevector_wall_s'], 4)} | "
            f"{f(r['abs_error_max'], 3)} | {f(r['remainder_max'], 3)} | "
            f"{f(r['removed_marginal_med'], 3)} | {f(r['cone_med'], 3)} |"
        )
    md += ["", "## The full sum's measured exponent", "",
           "S5 checks the full branch sum against the published `2^{0.396 t}`. TWO slopes are "
           "reported, because they are two different claims. BRANCHES is `log2(N_pred)` against "
           "`t` — the rate the published exponent is about. WALL is `log2(seconds)` against "
           "`t`, which is the branch rate PLUS the per-branch cost, and the per-branch cost "
           "grows with `t` because each branch evolves an affine state `n + t` wide. The last "
           "column is the difference: the overhead's own slope, which is where any gap lives.",
           "",
           "| n | BRANCHES (log2 N per T) | WALL (log2 s per T) | published | branch gap | "
           "per-branch overhead |", "|---|---|---|---|---|---|"]
    for e in expo:
        md.append(
            f"| {e['n']} | {e['slope_log2_branches_per_T']:.4f} | "
            f"{e['slope_log2_wall_per_T']:.4f} | 0.3963 | "
            f"{e['slope_log2_branches_per_T'] - 0.3963:+.4f} | "
            f"{e['slope_per_branch_overhead']:+.4f} |"
        )
    mpath = os.path.join(HERE, "qvm_acuity_table.md")
    open(mpath, "w").write("\n".join(md) + "\n")
    print(f"wrote {jpath}\nwrote {mpath}")
    worst = max([r["abs_error"] for r in runs if r.get("abs_error") is not None], default=None)
    print(f"worst |value - referee| anywhere: {worst}")


if __name__ == "__main__":
    main()
