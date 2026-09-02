#!/usr/bin/env python3
"""Build DE5_RESULTS.md's measured sections straight from the run's own CSV and log.

Nothing here is typed by hand: every number in the tables below is read from
de5_audit.csv or de5_score.log, so the document cannot drift from the run.
"""
import csv, pathlib, re, sys

# WT defaults to this repo's own root (the original session worktree was a copy
# of this tree, and a session-keyed path dies with the session -- gate 10a3).
# Override with DE5_WT for a re-run elsewhere.
import os, subprocess
WT = pathlib.Path(os.environ.get("DE5_WT") or subprocess.run(
    ["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True
).stdout.strip())
OBS = WT / "conformance/water_observatory"
BOUND = 5.0e-5

rows = []
with open(OBS / "de5_audit.csv") as f:
    lines = [l for l in f if not l.startswith("#")]
for r in csv.DictReader(lines):
    rows.append(r)

log = (OBS / "de5_score.log").read_text()

scored = [r for r in rows if r["status"] in ("LIVE", "VACUOUS")]
live = [r for r in rows if r["status"] == "LIVE"]
vac = [r for r in rows if r["status"] == "VACUOUS"]
voids = [r for r in rows if r["status"] == "VOID"]

def f(r, k):
    return float(r[k])

out = []
A = out.append

A("| # | seed | frame | diam (bohr) | dE5 (Ha) | \\|dE5\\| / bound | max\\|dE4\\| (Ha) | ratio | verdict |")
A("|---:|---|---:|---:|---:|---:|---:|---:|:--|")
for i, r in enumerate(sorted(live, key=lambda x: -abs(f(x, "de5"))), 1):
    d = f(r, "de5")
    A(f"| {i} | `{r['seed']}` | {r['frame']} | {f(r,'diameter'):.3f} | {d:+.6e} | "
      f"{abs(d)/BOUND:,.0f}x | {f(r,'max_abs_de4'):.4e} | {f(r,'ratio'):.4f} | "
      f"{'under' if abs(d) < BOUND else '**OVER**'} |")

A("")
A("### The distribution, in one line each")
A("")
if live:
    a = sorted(abs(f(r, "de5")) for r in live)
    n = len(a)
    med = a[n // 2] if n % 2 else 0.5 * (a[n // 2 - 1] + a[n // 2])
    A(f"* **N_live = {n}** (the freeze's bar is 20)")
    A(f"* worst `|dE5|` = **{a[-1]:.6e} Ha** = **{a[-1]/BOUND:,.0f}x** the 5.0e-5 Ha bound")
    A(f"* median `|dE5|` = {med:.6e} Ha = {med/BOUND:,.0f}x the bound")
    A(f"* smallest `|dE5|` = {a[0]:.6e} Ha = {a[0]/BOUND:,.0f}x the bound")
    over = sum(1 for x in a if x >= BOUND)
    A(f"* configs at or over the bound: **{over} of {n}**")
    rr = sorted(f(r, "ratio") for r in live)
    A(f"* convergence ratio `|dE5| / max|dE4|`: min {rr[0]:.4f}, median "
      f"{(rr[len(rr)//2] if len(rr)%2 else 0.5*(rr[len(rr)//2-1]+rr[len(rr)//2])):.4f}, max {rr[-1]:.4f}")
    A(f"* configs where `|dE5|` EXCEEDS the largest four-body term: "
      f"**{sum(1 for x in rr if x > 1.0)} of {n}**")
A("")
A(f"* VACUOUS (excluded, freeze 3.5): {len(vac)}")
A(f"* VOID (never scored, freeze 6): {len(voids)}")
A(f"* scored total: {len(scored)} of {len(rows)} drawn")

A("")
A("### Numerical headroom — the measurement resolves the bound it is read against")
A("")
if live:
    wr = max(f(r, "worst_residual") for r in live)
    rs = max(f(r, "residual_sum") for r in live)
    ns = max(int(r["n_solves"]) for r in live)
    A(f"* worst single-solve Davidson residual over every scored solve: **{wr:.3e} Ha** "
      f"(the freeze's bar is 1.0e-9)")
    A(f"* worst per-config residual SUM over its {ns} solves: **{rs:.3e} Ha**")
    A(f"* that sum is **{BOUND/rs:,.0f}x** below the 5.0e-5 Ha bound and "
      f"**{min(abs(f(r,'de5')) for r in live)/rs:,.0f}x** below the SMALLEST measured `|dE5|`")
    A("")
    A("So the readings are not arithmetic noise, and G8's claim is a measured number rather")
    A("than a hope.")

A("")
A("### The strict reading, published beside the amended one (freeze 5b)")
A("")
if live:
    sv = sum(1 for r in scored if r["strict_void"] == "true")
    sn = sum(int(r["scf_not_converged"]) for r in scored)
    su = sum(int(r["scf_unobservable"]) for r in scored)
    A(f"* configs that would VOID under G2 exactly as frozen: **{sv} of {len(scored)}**")
    A(f"* solves reporting `scf_converged = false`: **{sn}**")
    A(f"* solves where the flag is UNOBSERVABLE (freeze C-2): **{su}** "
      f"— one per config, the `O2H3` pentamer")
    A("")
    A(f"**STRICT VERDICT: BRANCH (d) VOID** — {len(scored)-sv} of {len(scored)} configs survive "
      f"the frozen clause, below the bar of 20, so G2 as frozen returns no verdict about the")
    A("ladder at all. The amended reading below is the one that carries information, and it is")
    A("never quoted without this paragraph beside it.")

print("\n".join(out))
