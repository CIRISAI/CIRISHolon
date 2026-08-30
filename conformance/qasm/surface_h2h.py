#!/usr/bin/env python3
"""SURFACE-CODE HEAD-TO-HEAD: holon's adaptive Clifford engine vs stim, on
the IDENTICAL circuit.

Methodology, following the bake-off discipline (BENCHMARKS entries six/eight/
ten): engine-only timing on both sides, medians of N, arms INTERLEAVED so a
drifting machine load hits both equally rather than whichever ran second.

The circuit is OURS in both arms — `surface_flagship --mode bench --stim`
emits the same four-step rotated-surface-code extraction cycle it runs, in
stim's format, so this compares ENGINES and not circuit generators.

Both arms perform full exact adaptive simulation: mid-circuit measurement
with the state collapsing, not Pauli-frame sampling.

Usage: surface_h2h.py [--d 21,45,101] [--rounds 3] [--reps 3]
"""
import argparse, json, os, statistics, subprocess, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
FLAGSHIP = os.environ.get(
    "FLAGSHIP",
    os.path.join(ROOT, "engine", "target", "release", "examples", "surface_flagship"),
)


def ours(d, rounds, tmp):
    """Wall seconds for the rounds themselves (build/alloc excluded, as
    stim's circuit construction is excluded on its side)."""
    out = subprocess.run(
        [FLAGSHIP, "--d", str(d), "--mode", "bench", "--rounds", str(rounds),
         "--json", tmp],
        capture_output=True, text=True,
    )
    if out.returncode != 0:
        raise RuntimeError(f"ours failed (rc={out.returncode}): {out.stderr[-2000:]}")
    m = json.loads(out.stdout)["results"][0]["metadata"]
    return m["timing_seconds"]["wall"], m


def emit_stim(d, rounds, path):
    subprocess.run(
        [FLAGSHIP, "--d", str(d), "--mode", "bench", "--rounds", str(rounds),
         "--stim", path, "--json", os.devnull],
        capture_output=True, text=True, check=True,
    )
    return path


def stim_time(circ):
    # The simulator's tableau is GB-scale at the large distances; free it
    # before the other arm allocates its own, or the two arms fight over a
    # shared box rather than taking turns on it.
    import gc, stim
    sim = stim.TableauSimulator()
    sim.set_num_qubits(circ.num_qubits)
    t0 = time.perf_counter()
    sim.do(circ)
    el = time.perf_counter() - t0
    del sim
    gc.collect()
    return el


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--d", default="21,45,101")
    ap.add_argument("--rounds", type=int, default=3)
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--out", default=os.path.join(HERE, "surface_h2h_results.json"))
    ap.add_argument("--tmpdir", default="/tmp")
    args = ap.parse_args()

    import stim
    ds = [int(x) for x in args.d.split(",")]
    print(f"surface-code head-to-head: holon coladaptive vs stim {stim.__version__}")
    print(f"medians of {args.reps}, arms interleaved, {args.rounds} rounds, "
          f"identical circuits (ours, emitted to stim format)")
    print(f"loadavg at start: {os.getloadavg()}")
    print()

    rows = []
    for d in ds:
        path = os.path.join(args.tmpdir, f"sc_h2h_d{d}.stim")
        emit_stim(d, args.rounds, path)
        circ = stim.Circuit(open(path).read())
        tmpjson = os.path.join(args.tmpdir, f"sc_h2h_d{d}.json")

        o_times, s_times, meta = [], [], None
        for _ in range(args.reps):
            t, meta = ours(d, args.rounds, tmpjson)
            o_times.append(t)
            s_times.append(stim_time(circ))
        # On a CONTENDED box the median still carries the contention; the
        # MINIMUM is the standard robust estimator because interference can
        # only ever add time, never remove it. Both are reported, and the
        # spread with them, so a reader can see which regime this ran in.
        o_med, s_med = statistics.median(o_times), statistics.median(s_times)
        o = min(o_times)
        s = min(s_times)
        n = 2 * d * d - 1
        rows.append({
            "d": d, "n": n, "rounds": args.rounds,
            "measurements": meta["measurements"]["total"],
            "ours_s": o, "stim_s": s, "ratio": o / s,
            "ours_median_s": o_med, "stim_median_s": s_med,
            "ratio_median": o_med / s_med,
            "ours_spread": max(o_times) / min(o_times),
            "stim_spread": max(s_times) / min(s_times),
            "ours_all": o_times, "stim_all": s_times,
            "peak_rss_bytes": meta["peak_rss_bytes"],
        })
        verdict = "WE LEAD" if o < s else "stim leads"
        print(f"d={d:4d} n={n:7d} meas={meta['measurements']['total']:8d}  "
              f"ours {o:9.3f} s   stim {s:9.3f} s   ours/stim {o/s:6.3f}  {verdict} "
              f"{max(o,s)/min(o,s):.2f}x   [median ratio {o_med/s_med:5.2f}, "
              f"spread ours {max(o_times)/min(o_times):.2f}x stim "
              f"{max(s_times)/min(s_times):.2f}x]")
        os.remove(path)

    lead = sum(1 for r in rows if r["ratio"] < 1.0)
    print()
    print(f"VERDICT: ahead at {lead}/{len(rows)} distances; "
          f"best {min(r['ratio'] for r in rows):.3f}, "
          f"worst {max(r['ratio'] for r in rows):.3f}")
    json.dump({"stim_version": stim.__version__, "reps": args.reps,
               "rounds": args.rounds, "loadavg": os.getloadavg(), "rows": rows},
              open(args.out, "w"), indent=1)
    print(f"wrote {args.out}")


if __name__ == "__main__":
    main()
