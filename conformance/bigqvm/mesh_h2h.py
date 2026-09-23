#!/usr/bin/env python3
"""MESH-CLIFFORD-1 — the G3 head-to-head and the G4 memory gate.

Freeze: `conformance/bigqvm/MESH_CLIFFORD_PREREG.md` (gates G3 and G4; G1's
bit-identity is the ENGINE's gate, and this harness carries its cross-check
because a timing taken on an arm that computed a different record is not a
measurement of anything).

WHAT IT MEASURES. For each distance `d` and each shard count `S`, the wall of
the sharded Clifford engine on the rotated surface code (`--mode bench`, the
rounds only, build and alloc excluded — the same number `surface_h2h.py`
compares against stim), plus stim on the IDENTICAL circuit, which the binary
itself emits (`--stim PATH`), so this compares engines and not circuit
generators. Peak RSS is taken on every run: G4 is its ratio to `S = 1`.

METHODOLOGY, inherited from this lane's bake-off discipline (BENCHMARKS
entries six/eight/ten, and the twenty-seventh entry's corrections):

  * ARMS INTERLEAVED. Every arm of a distance — each `S`, and stim — runs once
    per repetition, and the arm ORDER ROTATES by repetition, so a drifting
    machine hits every arm equally instead of whichever ran last. A sweep that
    runs all of S=1 and then all of S=8 measures the machine's drift as if it
    were the cut.
  * PINNED. Both arms run under `taskset -c <--cores>`, and the harness sets
    its own affinity to the same set so the in-process stim arm is pinned
    identically. This box is an i9-13900HX and the d=101 verdict FLIPPED
    between placements (0.822 unpinned, 1.201 P-core, 0.989 E-core); a quiet
    window fixes contention and does nothing about heterogeneity.
  * MEDIAN AND MIN AND SPREAD. The median carries contention; the minimum is
    the robust estimator because interference can only ADD time. Spread is
    max/min, and the stake is graded on the medians with the spreads required
    not to overlap.
  * CONDITIONS AT BOTH ENDS (fleet rule, 2026-09-01, `conformance/lib/
    run_conditions.sh`): loadavg, core class, and clock as a fraction of
    advertised, stamped at launch and at exit. A record without its conditions
    is not a record.
  * THE LOAD GATE. The prereg refuses a table taken on a loaded box. If the
    1-minute loadavg exceeds `--max-load` at EITHER end, the JSON is not
    written to `--out`; the data is kept beside it under a `.not-citable-*`
    name, and the refusal says so. `--dry-run` writes to `--out` anyway and
    stamps the file `"citable": false, "protocol_validation": true`.

THE TWO THINGS THIS HARNESS REFUSES TO DO.

  1. It will not time an `S != 1` arm the binary has not ECHOED BACK. The
     flagship's argument parser ignores flags it does not know, so a binary
     built before `--shards` landed, handed `--shards 8`, runs S=1 silently
     and reports nothing — and would hand this harness a fabricated "S = 8"
     row that is really the S = 1 engine measured twice. The echo (metadata
     `shards.count`, or the stderr line) is the only evidence that the flag
     did anything, so without it every S != 1 arm is refused by name.
  2. It will not time a (d, S) whose measurement-record hash differs from that
     d's S = 1 hash. That is G1's cross-check standing inside the timing
     harness: before any arm of a distance is timed, each S is PROBED once and
     its record hash compared to S = 1's. A differing hash is reported as a G1
     failure and that arm is dropped from the timing, not timed and caveated.
     (The probe doubles as the warm-up run, whose timing is discarded anyway.)

THE CONTRACT WITH THE ENGINE. Two readings, JSON first and stderr second;
either alone suffices, and both are legitimately absent at S = 1 on the
engine as it stands today.

  metadata (inside `results[0].metadata`, which `--json` and stdout carry):
      "shards": {"count": 8, "crossing_cx": 12345, "total_cx": 98765,
                 "crossing_fraction": 0.125},
      "record_hash": "sha256:<hex>"
  stderr (matched case-insensitively, anywhere in a line):
        shards S=8  crossing 12345/98765 = 0.12500
        record sha256 3f9a1b...

`record_hash` is compared as an OPAQUE STRING: any stable digest of the full
measurement record works, as long as it is the same function at every S and
depends on the record alone — not on S, thread count or timings.

USAGE

    conformance/bigqvm/mesh_h2h.py [BINARY] --cores 8-15 \
        --d 45,141,221 --shards 1,2,4,8 --reps 5 \
        --out conformance/bigqvm/mesh_h2h_pcore.json

    --no-stim          skip the stim arm (engine-against-itself only)
    --max-load 8       refuse to write a citable table above this 1-min loadavg
    --dry-run          write anyway, stamped as a protocol validation
    --selftest         grade three synthetic tables and exit (no machine time)

stim lives in a venv on this box. If the interpreter running this file has no
`stim`, the harness DISCOVERS one and re-execs itself under it rather than
failing — and refuses loudly, with the command to build one, if there is none.
"""
import argparse
import glob
import hashlib
import json
import os
import re
import statistics
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
DEFAULT_BIN = os.path.join(ROOT, "engine", "target", "release", "examples", "surface_flagship")
USE_TIME = os.path.exists("/usr/bin/time")

# ---- the engine contract, as regexes -------------------------------------
RE_SHARDS = re.compile(r"shards\s*[:= ]?\s*S\s*=\s*(\d+)", re.I)
RE_CROSS = re.compile(r"crossing\s*[:= ]?\s*(\d+)\s*/\s*(\d+)", re.I)
RE_CROSSFRAC = re.compile(r"crossing[^=\n]*=\s*([0-9]*\.?[0-9]+(?:[eE][-+]?\d+)?)", re.I)
RE_HASH = re.compile(r"record\s*(?:hash|sha-?256)?\s*[:= ]\s*(?:sha-?256:)?([0-9a-fA-F]{8,64})", re.I)


class MemoryRefusal(Exception):
    """One size does not fit right now; the others may. Skip it, keep the window."""


class EngineFailure(Exception):
    """A real failure — not a memory refusal. Never silently absorbed."""


# ---- stim discovery, then re-exec ----------------------------------------
def ensure_stim():
    try:
        import stim  # noqa: F401
        return
    except ImportError:
        pass
    if os.environ.get("MESH_H2H_REEXEC"):
        sys.exit("REFUSING: re-exec'd into a python that still has no stim.")
    cands = []
    if os.environ.get("STIMPY"):
        cands.append(os.environ["STIMPY"])
    cands += [
        os.path.join(ROOT, ".venv", "bin", "python"),
        os.path.expanduser("~/.venvs/stim/bin/python"),
        # Known venvs on this box. A HARDCODED per-session scratchpad path is a
        # reproducibility defect invisible from inside the session that wrote
        # it (credit: saturation3-mesh), so these are candidates to be TESTED,
        # never a path to be trusted.
        os.path.expanduser("~/CIRISOntology/scratchpad/temporal-share/qenv/bin/python"),
    ]
    cands += sorted(glob.glob("/tmp/claude-*/*/*/scratchpad/stimvenv/bin/python"))
    for c in cands:
        if c and os.path.isfile(c) and os.access(c, os.X_OK):
            r = subprocess.run([c, "-c", "import stim"], capture_output=True)
            if r.returncode == 0:
                os.environ["MESH_H2H_REEXEC"] = "1"
                print(f"# no stim in {sys.executable}; re-exec under {c}", flush=True)
                os.execv(c, [c, os.path.abspath(__file__)] + sys.argv[1:])
    sys.exit(
        "REFUSING: no python with stim found.\n"
        "  set STIMPY=/path/to/python, or:\n"
        "    python3 -m venv .venv && .venv/bin/pip install stim\n"
        "  or pass --no-stim to run the engine arms alone."
    )


# ---- conditions, every line measured -------------------------------------
def core_classes():
    """P vs E derived from SMT siblings, not from this box's numbering."""
    p, e = [], []
    for path in sorted(glob.glob("/sys/devices/system/cpu/cpu[0-9]*")):
        name = os.path.basename(path)[3:]
        if not name.isdigit():
            continue
        try:
            with open(os.path.join(path, "topology", "thread_siblings_list")) as fh:
                sib = fh.read().strip()
        except OSError:
            continue
        (p if ("," in sib or "-" in sib) else e).append(int(name))
    return p, e


def clock_frac(cpu):
    try:
        with open(f"/sys/devices/system/cpu/cpu{cpu}/cpufreq/scaling_cur_freq") as fh:
            cur = int(fh.read())
        with open(f"/sys/devices/system/cpu/cpu{cpu}/cpufreq/cpuinfo_max_freq") as fh:
            mx = int(fh.read())
        return {"cpu": cpu, "cur_ghz": cur / 1e6, "max_ghz": mx / 1e6,
                "frac_of_advertised": cur / mx}
    except (OSError, ZeroDivisionError, ValueError):
        return {"cpu": cpu, "frac_of_advertised": None}


def mem_available_gb():
    try:
        with open("/proc/meminfo") as fh:
            for line in fh:
                if line.startswith("MemAvailable:"):
                    return int(line.split()[1]) / 1048576.0
    except OSError:
        pass
    return None


def conditions(when, cores):
    p, e = core_classes()
    return {
        "when": when,
        "wallclock": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "loadavg": list(os.getloadavg()),
        "n_p_class": len(p),
        "n_e_class": len(e),
        "affinity": sorted(os.sched_getaffinity(0)),
        "clock_pinned": [clock_frac(c) for c in sorted(cores)[:4]],
        "clock_p_class": clock_frac(p[0]) if p else None,
        "clock_e_class": clock_frac(e[0]) if e else None,
        "mem_available_gb": mem_available_gb(),
    }


def print_conditions(cond):
    la = cond["loadavg"]
    print(f"--- run conditions, {cond['when']} [ALL MEASURED] ---")
    print(f"  loadavg           {la[0]:.2f} {la[1]:.2f} {la[2]:.2f}")
    print(f"  cores             {cond['n_p_class']} SMT (P-class), "
          f"{cond['n_e_class']} non-SMT (E-class)")
    print(f"  affinity          {cond['affinity']}")
    print("                    ^ taskset RESTRICTS, it does not RESERVE: sibling load is a")
    print("                      MEASURED COVARIATE here, not a controlled one.")
    for c in cond["clock_pinned"]:
        f = c.get("frac_of_advertised")
        if f is None:
            print(f"  clock cpu{c['cpu']:<3d}      n/a")
        else:
            print(f"  clock cpu{c['cpu']:<3d}      {100 * f:3.0f}% of advertised "
                  f"({c['cur_ghz']:.2f} of {c['max_ghz']:.2f} GHz)")
    mg = cond.get("mem_available_gb")
    print(f"  MemAvailable      {mg:.2f} GB" if mg else "  MemAvailable      n/a")
    print(f"--- end conditions, {cond['when']} ---")


# ---- the arms ------------------------------------------------------------
def parse_cores(spec):
    """'21-27' or '8,10,12' or '8-11,20' -> (raw string for taskset, set of ints)."""
    out = set()
    for part in spec.split(","):
        part = part.strip()
        if not part:
            continue
        if "-" in part:
            a, b = part.split("-", 1)
            out.update(range(int(a), int(b) + 1))
        else:
            out.add(int(part))
    if not out:
        raise ValueError(f"--cores {spec!r} names no CPU")
    return spec, out


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def parse_time_v(path):
    """Peak RSS in bytes from /usr/bin/time -v, or None."""
    try:
        with open(path) as fh:
            for line in fh:
                if "Maximum resident set size" in line:
                    return int(line.rsplit(":", 1)[1].strip()) * 1024
    except (OSError, ValueError, IndexError):
        pass
    return None


def extract_shard_fields(meta, stderr):
    """(S_echoed, crossing_cx, total_cx, crossing_fraction, record_hash).

    JSON metadata first, stderr second. Every field may be None: at S = 1 on
    today's engine all of them are, and that is a legal reading.
    """
    S = cx = tot = frac = rec = None
    sh = meta.get("shards") if isinstance(meta, dict) else None
    if isinstance(sh, dict):
        S = sh.get("count")
        cx = sh.get("crossing_cx")
        tot = sh.get("total_cx")
        frac = sh.get("crossing_fraction")
    elif isinstance(sh, int):
        S = sh
    if isinstance(meta, dict) and meta.get("record_hash"):
        rec = str(meta["record_hash"])
    if S is None:
        m = RE_SHARDS.search(stderr)
        if m:
            S = int(m.group(1))
    if cx is None:
        m = RE_CROSS.search(stderr)
        if m:
            cx, tot = int(m.group(1)), int(m.group(2))
    if frac is None:
        if cx is not None and tot:
            frac = cx / tot
        else:
            m = RE_CROSSFRAC.search(stderr)
            if m:
                frac = float(m.group(1))
    if rec is None:
        m = RE_HASH.search(stderr)
        if m:
            rec = m.group(1)
    return S, cx, tot, frac, rec


def run_engine(args, d, S):
    """One engine run. Returns a dict; raises MemoryRefusal / EngineFailure."""
    tmpjson = os.path.join(args.tmpdir, f"mesh_h2h_d{d}_S{S}.json")
    rssfile = os.path.join(args.tmpdir, f"mesh_h2h_d{d}_S{S}.time")
    cmd = []
    if USE_TIME:
        cmd += ["/usr/bin/time", "-v", "-o", rssfile]
    # RULE OF THE MACHINE: every timing run is pinned, explicitly, here.
    cmd += ["taskset", "-c", args.cores]
    cmd += [args.binary, "--d", str(d), "--mode", "bench", "--rounds", str(args.rounds),
            "--seed", str(args.seed), "--shards", str(S), "--json", tmpjson]
    # Amendment 2: the engine's numbering and the re-aim are the arm's identity, passed
    # through verbatim and echoed into the table's header.
    if getattr(args, "engine_args", ""):
        cmd += args.engine_args.split()
    if getattr(args, "exercise_refusal", False):
        # The binary's own test hook, symmetric to --no-guard: refuse
        # unconditionally, so the per-size SKIP path below is exercised rather
        # than assumed. A fallback nothing ever runs is an untested claim.
        cmd.append("--force-refuse")
    t0 = time.perf_counter()
    p = subprocess.run(cmd, capture_output=True, text=True)
    proc_wall = time.perf_counter() - t0
    if p.returncode == 2:
        # The binary's OWN guard refused: no room right now. That is one size
        # being unaffordable, not the sweep being invalid — aborting here
        # throws away the other sizes and, in a quiet window, the window.
        last = p.stderr.strip().splitlines()[-1] if p.stderr.strip() else "refused"
        raise MemoryRefusal(last)
    if p.returncode != 0:
        raise EngineFailure(f"d={d} S={S} rc={p.returncode}: {p.stderr[-2000:]}")
    try:
        meta = json.loads(p.stdout)["results"][0]["metadata"]
    except (ValueError, KeyError, IndexError):
        try:
            with open(tmpjson) as fh:
                meta = json.load(fh)["results"][0]["metadata"]
        except Exception as exc:
            raise EngineFailure(
                f"d={d} S={S}: no parsable JSON ({exc}); stderr tail: {p.stderr[-800:]}")
    S_echo, cx, tot, frac, rec = extract_shard_fields(meta, p.stderr)
    return {
        "wall_s": meta["timing_seconds"]["wall"],
        "process_wall_s": proc_wall,
        "measurements": meta["measurements"]["total"],
        "peak_rss_bytes_self": meta.get("peak_rss_bytes"),
        "peak_rss_bytes_time": parse_time_v(rssfile) if USE_TIME else None,
        "shards_echoed": S_echo,
        "crossing_cx": cx,
        "total_cx": tot,
        "crossing_fraction": frac,
        "record_hash": rec,
    }


def emit_stim_circuit(args, d, path):
    """The binary writes the circuit BEFORE its memory guard runs, so a
    refusal still leaves a usable file; `check=True` here would abort a whole
    sweep over a size that merely does not fit right now."""
    p = subprocess.run(
        ["taskset", "-c", args.cores, args.binary, "--d", str(d), "--mode", "bench",
         "--rounds", str(args.rounds), "--seed", str(args.seed),
         "--stim", path, "--json", os.devnull],
        capture_output=True, text=True,
    )
    if not os.path.exists(path) or os.path.getsize(path) == 0:
        raise EngineFailure(
            f"emit_stim d={d} produced no circuit (rc={p.returncode}): {p.stderr[-500:]}")
    return path


def stim_time(circ):
    """The simulator's tableau is GB-scale at the large distances; free it
    before the other arm allocates its own, or the arms fight over a shared
    box instead of taking turns on it."""
    import gc
    import stim
    sim = stim.TableauSimulator()
    sim.set_num_qubits(circ.num_qubits)
    t0 = time.perf_counter()
    sim.do(circ)
    el = time.perf_counter() - t0
    del sim
    gc.collect()
    return el


# ---- statistics ----------------------------------------------------------
def summarize(times):
    return {
        "median_s": statistics.median(times),
        "min_s": min(times),
        "max_s": max(times),
        "spread": max(times) / min(times) if min(times) > 0 else None,
        "all_s": list(times),
    }


def disjoint(fast, slow):
    """Do the two arms' [min, max] intervals fail to overlap, fast below slow?"""
    if not fast or not slow:
        return None
    return fast["max_s"] < slow["min_s"]


# ---- the grade -----------------------------------------------------------
def grade(rows, stim_rows, args):
    """G1's cross-check, G2's number, G3's stake and G4's ceiling, each with
    the prereg's own arithmetic and nothing added to it."""
    by = {(r["d"], r["shards_requested"]): r for r in rows if r.get("timed")}
    g = {}

    # --- G1 cross-check (the engine owns G1; this is its shadow here) ---
    fails = [r for r in rows if r.get("hash_verdict") == "DIFFERS"]
    unavail = [r for r in rows if r.get("hash_verdict") == "UNAVAILABLE"]
    checked = [r for r in rows if r.get("hash_verdict") == "MATCHES"]
    if fails:
        g["G1_crosscheck"] = {
            "verdict": "FAIL",
            "why": "record hash differs from S=1 at " +
                   ", ".join(f"d={r['d']} S={r['shards_requested']}" for r in fails),
            "consequence": "those arms were NOT timed (prereg branch (e): nothing is read)",
        }
    elif checked:
        g["G1_crosscheck"] = {
            "verdict": "PASS",
            "why": f"{len(checked)} (d,S) arm(s) carry the S=1 record hash bit for bit",
        }
    else:
        g["G1_crosscheck"] = {
            "verdict": "UNAVAILABLE",
            "why": "the binary reports no record hash; "
                   f"{len(unavail)} arm(s) unchecked. G1 stands on the engine's own gate alone.",
        }

    # --- G2, the crossing fraction (engine's gate; the number is measured here) ---
    have = [r for r in rows if r.get("crossing_fraction") is not None]
    viol = [r for r in have
            if r["shards_requested"] and r["shards_requested"] >= 8
            and r["d"] >= 141 and r["crossing_fraction"] >= 0.25]
    if not have:
        g["G2_crossing"] = {"verdict": "NOT MEASURED",
                            "why": "the binary reports no crossing count"}
    elif viol:
        g["G2_crossing"] = {
            "verdict": "KILL",
            "why": "crossing fraction >= 0.25 at " + ", ".join(
                f"d={r['d']} S={r['shards_requested']} ({r['crossing_fraction']:.4f})"
                for r in viol),
        }
    else:
        g["G2_crossing"] = {
            "verdict": "MET",
            "why": "crossing fraction < 0.25 at every S>=8, d>=141 measured",
            "max_seen": max(r["crossing_fraction"] for r in have),
        }

    # --- G3, the stake ---
    sd, ss = args.stake_d, args.stake_s
    a1, a8 = by.get((sd, 1)), by.get((sd, ss))
    st = next((s for s in stim_rows if s["d"] == sd and s.get("timed")), None)
    if a8 is None or a1 is None:
        missing = []
        if a8 is None:
            missing.append(f"S={ss}")
        if a1 is None:
            missing.append("S=1")
        g["G3_headtohead"] = {
            "verdict": "NOT GRADED",
            "why": f"the stake row is d={sd}, S={ss} against S=1; "
                   f"{' and '.join(missing)} absent AT d={sd} in this run",
        }
    else:
        r1 = a8["wall"]["median_s"] / a1["wall"]["median_s"]
        rs = (a8["wall"]["median_s"] / st["wall"]["median_s"]) if st else None
        d1 = disjoint(a8["wall"], a1["wall"])
        ds = disjoint(a8["wall"], st["wall"]) if st else None
        met = (r1 <= 0.5) and (rs is not None and rs <= 0.5) and bool(d1) and bool(ds)
        kill = r1 > 0.8
        g["G3_headtohead"] = {
            "verdict": "MET" if met else ("KILL" if kill else "UNDECIDED"),
            "stake": f"d={sd}, S={ss}: wall <= 0.5x S=1 AND <= 0.5x stim, spreads not overlapping",
            "ratio_to_s1_median": r1,
            "ratio_to_stim_median": rs,
            "spreads_disjoint_vs_s1": d1,
            "spreads_disjoint_vs_stim": ds,
            "why": (
                f"S={ss} median {a8['wall']['median_s']:.4f}s is {r1:.3f}x S=1 "
                f"({a1['wall']['median_s']:.4f}s)"
                + (f" and {rs:.3f}x stim ({st['wall']['median_s']:.4f}s)" if rs is not None
                   else "; NO STIM ARM in this run, so the '<= 0.5x stim' half is ungraded")
            ),
            "branch": ("(a) if G1, G2 and G4 are also met" if met
                       else ("(b) the cut does not pay on this box; the bandwidth wall is the "
                             "finding, banked as such" if kill
                             else "neither the stake nor the kill: report as measured")),
        }

    # --- G4, memory ---
    def rss_ok(r):
        return bool(r) and bool(r.get("peak_rss_bytes"))

    if not (rss_ok(a1) and rss_ok(a8)):
        alt = None
        for dd in sorted({r["d"] for r in rows if r.get("timed")}, reverse=True):
            x1, x8 = by.get((dd, 1)), by.get((dd, ss))
            if rss_ok(x1) and rss_ok(x8):
                alt = (dd, x1, x8)
                break
        if alt is None:
            g["G4_memory"] = {
                "verdict": "NOT GRADED",
                "why": f"no distance in this run carries both S=1 and S={ss} with a peak RSS",
            }
        else:
            dd, x1, x8 = alt
            ratio = x8["peak_rss_bytes"] / x1["peak_rss_bytes"]
            g["G4_memory"] = {
                "verdict": "MET" if ratio <= 1.25 else "KILL",
                "why": f"NOT at the staked distance: graded at d={dd} (d={sd} absent). "
                       f"RSS ratio S={ss}/S=1 = {ratio:.4f}",
                "graded_at_d": dd, "rss_ratio": ratio, "ceiling": 1.25,
            }
    else:
        ratio = a8["peak_rss_bytes"] / a1["peak_rss_bytes"]
        g["G4_memory"] = {
            "verdict": "MET" if ratio <= 1.25 else "KILL",
            "why": f"d={sd}: peak RSS S={ss} {a8['peak_rss_bytes'] / 1e9:.3f} GB against "
                   f"S=1 {a1['peak_rss_bytes'] / 1e9:.3f} GB = {ratio:.4f}x",
            "graded_at_d": sd, "rss_ratio": ratio, "ceiling": 1.25,
        }
    # the worst ratio anywhere, which is what a reader actually wants
    worst = None
    for (dd, S), r in by.items():
        b = by.get((dd, 1))
        if S != 1 and rss_ok(b) and rss_ok(r):
            q = r["peak_rss_bytes"] / b["peak_rss_bytes"]
            if worst is None or q > worst[0]:
                worst = (q, dd, S)
    if worst:
        g["G4_memory"]["worst_ratio_anywhere"] = {
            "ratio": worst[0], "d": worst[1], "shards": worst[2]}
    return g


# ---- printing ------------------------------------------------------------
def print_table(rows, stim_rows):
    print()
    print("  d      n       S    cross     wall med      min     spread    /S=1    /stim    "
          " peak RSS   /S=1   hash")
    print("  " + "-" * 112)
    by = {(r["d"], r["shards_requested"]): r for r in rows}
    for d in sorted({r["d"] for r in rows}):
        st = next((s for s in stim_rows if s["d"] == d and s.get("timed")), None)
        base = by.get((d, 1))
        # A distance the engine's memory guard refused has ONE row, with no S
        # on it. Without this it would print nothing at all and read as absent
        # rather than as skipped.
        whole = by.get((d, None))
        if whole is not None:
            print(f"  {d:<6d} {whole['n']:<7d} ALL  SKIPPED — {whole.get('why', 'refused')}")
        for S in sorted(x for x in {r["shards_requested"] for r in rows if r["d"] == d}
                        if x is not None):
            r = by[(d, S)]
            if not r.get("timed"):
                print(f"  {d:<6d} {r['n']:<7d} {S:<4d} NOT TIMED — {r.get('why', 'refused')}")
                continue
            w = r["wall"]
            cf = (f"{r['crossing_fraction']:.4f}"
                  if r.get("crossing_fraction") is not None else "  --  ")
            q1 = (w["median_s"] / base["wall"]["median_s"]
                  if base and base.get("timed") else None)
            qs = (w["median_s"] / st["wall"]["median_s"]) if st else None
            rss = r.get("peak_rss_bytes")
            qr = (rss / base["peak_rss_bytes"]
                  if (rss and base and base.get("peak_rss_bytes")) else None)
            print(f"  {d:<6d} {r['n']:<7d} {S:<4d} {cf:>7s}  {w['median_s']:9.4f} "
                  f"{w['min_s']:9.4f} {w['spread']:7.3f}x  "
                  f"{('%6.3f' % q1) if q1 is not None else '   -- '} "
                  f"{('%7.3f' % qs) if qs is not None else '    -- '}  "
                  f"{(rss / 1e9 if rss else 0):8.3f}G "
                  f"{('%6.3f' % qr) if qr is not None else '   -- '}  "
                  f"{(r.get('record_hash') or '--')[:12]}")
        if st:
            w = st["wall"]
            print(f"  {d:<6d} {st['n']:<7d} {'stim':<4s} {'  --  ':>7s}  {w['median_s']:9.4f} "
                  f"{w['min_s']:9.4f} {w['spread']:7.3f}x")
    print()


def print_grade(g):
    print("  GRADE (MESH_CLIFFORD_PREREG.md §2):")
    for k in ("G1_crosscheck", "G2_crossing", "G3_headtohead", "G4_memory"):
        if k not in g:
            continue
        v = g[k]
        print(f"    {k:<16s} {v['verdict']:<11s} {v.get('why', '')}")
        if k == "G3_headtohead" and "branch" in v:
            print(f"    {'':<16s} {'':<11s} branch: {v['branch']}")


# ---- the self-test of the grader ----------------------------------------
def selftest(args):
    """Grade three synthetic tables. This exercises the arithmetic of the
    stake and the kill WITHOUT fabricating a reading — the numbers below are
    made up on purpose, are never written to a results file, and exist so the
    grading path is tested rather than assumed (this lane's own rule: a
    fallback nothing ever runs is an untested claim)."""
    def tbl(w1, w8, stim_w, rss1, rss8, spread=1.01):
        def arm(d, S, w, rss):
            times = [w / spread, w, w * spread]
            return {"d": d, "n": 2 * d * d - 1, "shards_requested": S, "timed": True,
                    "wall": summarize(times), "peak_rss_bytes": rss,
                    "crossing_fraction": 0.12 if S > 1 else 0.0,
                    "record_hash": "deadbeef", "hash_verdict": "MATCHES"}
        rows = [arm(args.stake_d, 1, w1, rss1), arm(args.stake_d, args.stake_s, w8, rss8)]
        srow = {"d": args.stake_d, "n": 2 * args.stake_d ** 2 - 1, "timed": True,
                "wall": summarize([stim_w / spread, stim_w, stim_w * spread])}
        return rows, [srow]

    cases = [
        ("the stake is met (S=8 at 0.30x S=1, 0.35x stim, spreads disjoint, RSS 1.10x)",
         tbl(100.0, 30.0, 85.0, 10e9, 11e9)),
        ("the kill fires (S=8 at 0.95x S=1 — branch (b), the bandwidth wall)",
         tbl(100.0, 95.0, 85.0, 10e9, 11e9)),
        ("neither: 0.60x S=1 but 0.71x stim, and G4 breached at 1.40x",
         tbl(100.0, 60.0, 85.0, 10e9, 14e9)),
    ]
    print("GRADER SELF-TEST — synthetic numbers, never banked, never written\n")
    for name, (rows, srows) in cases:
        print(f"  case: {name}")
        g = grade(rows, srows, args)
        for k in ("G3_headtohead", "G4_memory"):
            print(f"    {k:<16s} {g[k]['verdict']:<11s} {g[k].get('why', '')}")
        print()
    return 0


# ---- main ----------------------------------------------------------------
def build_parser():
    ap = argparse.ArgumentParser(description="MESH-CLIFFORD-1 G3/G4 harness")
    ap.add_argument("binary", nargs="?", default=DEFAULT_BIN,
                    help="the surface_flagship release binary "
                         "(default: engine/target/release/examples/surface_flagship)")
    ap.add_argument("--d", default="45,141,221", help="distances (prereg G3: 45,141,221)")
    ap.add_argument("--shards", default="1,2,4,8", help="shard counts (prereg G3: 1,2,4,8)")
    ap.add_argument("--reps", type=int, default=5)
    ap.add_argument("--rounds", type=int, default=3)
    ap.add_argument("--seed", type=int, default=1,
                    help="held FIXED across S — the record hash is only comparable at one seed")
    ap.add_argument("--cores", default=None,
                    help="taskset range, e.g. 8-15 (P) or 16-23 (E). REQUIRED.")
    ap.add_argument("--stim", dest="stim", action="store_true", default=True,
                    help="run the stim arm on the identical circuit (default on)")
    ap.add_argument("--no-stim", dest="stim", action="store_false")
    ap.add_argument("--out", default=os.path.join(HERE, "mesh_h2h.json"))
    ap.add_argument("--engine-args", default="--layout banded",
                    help="extra flags for every engine run (Amendment 2: '--layout banded' is the "
                         "cut as frozen after branch (c); add '--transpose-parallel' for the re-aim)")
    ap.add_argument("--tmpdir", default="/tmp")
    ap.add_argument("--max-load", type=float, default=8.0,
                    help="refuse to write a citable table if the 1-min loadavg exceeds this "
                         "at either end (the prereg refuses a table taken on a loaded box)")
    ap.add_argument("--dry-run", action="store_true",
                    help="a PROTOCOL VALIDATION, not a reading: write the JSON even if the "
                         "load gate refuses, stamped citable=false")
    ap.add_argument("--note", default="", help="free text stamped into the JSON")
    ap.add_argument("--stake-d", type=int, default=221, help="the prereg's staked distance")
    ap.add_argument("--stake-s", type=int, default=8, help="the prereg's staked shard count")
    ap.add_argument("--selftest", action="store_true",
                    help="grade synthetic tables and exit; takes no machine time")
    ap.add_argument("--exercise-refusal", action="store_true",
                    help="pass the binary's --force-refuse test hook, so the per-size "
                         "memory-skip path is exercised rather than assumed")
    return ap


def main():
    args = build_parser().parse_args()

    if args.selftest:
        sys.exit(selftest(args))

    if args.cores is None:
        sys.exit("REFUSING: --cores is required. This box is hybrid (P-cores have an SMT\n"
                 "  sibling, E-cores do not) and an unpinned ratio is a ratio of two\n"
                 "  placements, not of two engines. See BENCHMARKS' twenty-seventh entry.")
    cores_raw, cores_set = parse_cores(args.cores)
    args.cores = cores_raw
    if not os.path.exists(args.binary):
        sys.exit(f"REFUSING: no binary at {args.binary}\n"
                 f"  build it: cd engine && taskset -c {cores_raw} cargo build --release "
                 f"-p holon --example surface_flagship")
    if args.stim:
        ensure_stim()
    # The harness's own affinity, so the IN-PROCESS stim arm is pinned to the
    # same set the taskset'd engine arms get. Children inherit it too; the
    # explicit taskset on each engine run is belt and braces, and it is what
    # the lane's rule of the machine asks for in so many words.
    os.sched_setaffinity(0, cores_set)

    ds = [int(x) for x in args.d.split(",") if x.strip()]
    ss = sorted({int(x) for x in args.shards.split(",") if x.strip()})
    if 1 not in ss:
        sys.exit("REFUSING: S=1 is the baseline of every ratio in G3 and G4; include it.")

    stim_version = None
    if args.stim:
        import stim
        stim_version = stim.__version__

    print("MESH-CLIFFORD-1 — G3 (head-to-head across shard counts) and G4 (memory)")
    print("prereg: conformance/bigqvm/MESH_CLIFFORD_PREREG.md §2 G3, G4")
    print(f"binary: {args.binary}")
    print(f"        sha256 {sha256_file(args.binary)[:16]}  mtime "
          f"{time.strftime('%Y-%m-%dT%H:%M:%S', time.localtime(os.path.getmtime(args.binary)))}")
    print(f"d = {ds}   S = {ss}   reps = {args.reps}   rounds = {args.rounds}   "
          f"seed = {args.seed}")
    print(f"cores = {cores_raw}   stim = {stim_version or 'OFF'}   max-load = {args.max_load}")
    print("arms interleaved, arm order rotated per repetition; wall is the ROUNDS only")
    if args.dry_run:
        print("*** --dry-run: this is a PROTOCOL VALIDATION, not a reading. "
              "The JSON is stamped citable=false. ***")
    print()
    cond_start = conditions("at launch", cores_set)
    print_conditions(cond_start)
    print()

    rows, stim_rows = [], []
    for d in ds:
        n = 2 * d * d - 1
        print(f"=== d={d} (n={n}) ===")

        # ---- the stim circuit, emitted once by our own binary ----
        circ = None
        stim_path = os.path.join(args.tmpdir, f"mesh_h2h_d{d}.stim")
        if args.stim:
            try:
                emit_stim_circuit(args, d, stim_path)
                import stim
                with open(stim_path) as fh:
                    circ = stim.Circuit(fh.read())
            except EngineFailure as exc:
                print(f"  stim arm unavailable: {exc}")
                circ = None

        # ---- PROBE PASS: the record hash, the crossing fraction, the echo ----
        # Nothing is timed until this pass has spoken. It doubles as the
        # warm-up, whose timing is discarded either way.
        probes, skipped = {}, None
        for S in ss:
            try:
                pr = run_engine(args, d, S)
            except MemoryRefusal as exc:
                skipped = str(exc)
                break
            except EngineFailure as exc:
                print(f"  S={S} PROBE FAILED: {exc}")
                probes[S] = {"error": str(exc)}
                continue
            probes[S] = pr
            ce = pr["shards_echoed"]
            cf = (f"{pr['crossing_fraction']:.5f}"
                  if pr["crossing_fraction"] is not None else "--")
            print(f"  probe S={S:<2d} echo={str(ce) if ce is not None else '--':<4s} "
                  f"crossing={cf:<9s} hash={(pr['record_hash'] or '--')[:16]:<16s} "
                  f"wall={pr['wall_s']:.4f}s (discarded)")
        if skipped:
            print(f"  d={d} SKIPPED — the engine's own guard refused for memory: {skipped}")
            rows.append({"d": d, "n": n, "shards_requested": None, "timed": False,
                         "why": f"memory refusal: {skipped}"})
            if args.stim and os.path.exists(stim_path):
                os.remove(stim_path)
            continue

        base_hash = probes.get(1, {}).get("record_hash")

        timed_S = []
        for S in ss:
            pr = probes.get(S)
            if pr is None or "error" in pr:
                rows.append({"d": d, "n": n, "shards_requested": S, "timed": False,
                             "why": pr.get("error", "no probe") if pr else "no probe",
                             "hash_verdict": "UNAVAILABLE"})
                continue
            # (1) the echo. Without it an S != 1 arm is the S = 1 engine wearing
            #     a label, because the flagship's parser ignores flags it does
            #     not know.
            if S != 1 and pr["shards_echoed"] is None:
                why = ("the binary did not echo S back: it has no --shards (or does not report "
                       "it), so this arm would be the S=1 engine mislabelled")
                print(f"  S={S} REFUSED — {why}")
                rows.append({"d": d, "n": n, "shards_requested": S, "timed": False,
                             "why": why, "hash_verdict": "UNAVAILABLE"})
                continue
            if S != 1 and pr["shards_echoed"] != S:
                why = f"requested S={S} but the binary used S={pr['shards_echoed']}"
                print(f"  S={S} REFUSED — {why}")
                rows.append({"d": d, "n": n, "shards_requested": S, "timed": False, "why": why,
                             "shards_echoed": pr["shards_echoed"],
                             "hash_verdict": "UNAVAILABLE"})
                continue
            # (2) G1's cross-check. A timing on an arm that computed a
            #     different record is not a measurement of anything.
            if base_hash and pr["record_hash"]:
                if pr["record_hash"] != base_hash:
                    why = (f"record hash {pr['record_hash'][:16]} differs from S=1's "
                           f"{base_hash[:16]} — G1 FAILS here; prereg branch (e): "
                           f"nothing is read")
                    print(f"  S={S} REFUSED — {why}")
                    rows.append({"d": d, "n": n, "shards_requested": S, "timed": False,
                                 "why": why, "record_hash": pr["record_hash"],
                                 "hash_verdict": "DIFFERS"})
                    continue
                hv = "MATCHES"
            else:
                hv = "UNAVAILABLE"
            timed_S.append((S, pr, hv))

        if not timed_S:
            print(f"  d={d}: no arm survived the probe pass; nothing timed")
            if args.stim and os.path.exists(stim_path):
                os.remove(stim_path)
            continue

        # ---- TIMED PASS: arms interleaved, order rotated per repetition ----
        arms = [("S", S) for S, _, _ in timed_S] + ([("stim", None)] if circ is not None else [])
        times = {a: [] for a in arms}
        rss = {S: [] for S, _, _ in timed_S}
        rss_t = {S: [] for S, _, _ in timed_S}
        failed = False
        for rep in range(args.reps):
            k = rep % len(arms)
            for a in arms[k:] + arms[:k]:
                if a[0] == "stim":
                    times[a].append(stim_time(circ))
                    continue
                try:
                    r = run_engine(args, d, a[1])
                except MemoryRefusal as exc:
                    print(f"  d={d} S={a[1]} refused mid-sweep for memory: {exc}; "
                          f"dropping this distance")
                    failed = True
                    break
                except EngineFailure as exc:
                    print(f"  d={d} S={a[1]} FAILED mid-sweep: {exc}")
                    failed = True
                    break
                times[a].append(r["wall_s"])
                if r["peak_rss_bytes_self"]:
                    rss[a[1]].append(r["peak_rss_bytes_self"])
                if r["peak_rss_bytes_time"]:
                    rss_t[a[1]].append(r["peak_rss_bytes_time"])
            if failed:
                break
        if failed:
            if args.stim and os.path.exists(stim_path):
                os.remove(stim_path)
            continue

        for S, pr, hv in timed_S:
            peak_self = max(rss[S]) if rss[S] else None
            peak_time = max(rss_t[S]) if rss_t[S] else None
            disagree = (abs(peak_time - peak_self) / peak_self
                        if (peak_self and peak_time) else None)
            rows.append({
                "d": d, "n": n, "shards_requested": S, "shards_echoed": pr["shards_echoed"],
                "timed": True, "rounds": args.rounds, "seed": args.seed,
                "measurements": pr["measurements"],
                "wall": summarize(times[("S", S)]),
                "crossing_cx": pr["crossing_cx"], "total_cx": pr["total_cx"],
                "crossing_fraction": pr["crossing_fraction"],
                "record_hash": pr["record_hash"], "hash_verdict": hv,
                # G4's number. The binary's own VmHWM is the primary (it is the
                # process measuring itself, and it is what BENCHMARKS' memory
                # rows already carry); /usr/bin/time -v is the INDEPENDENT
                # check, and a disagreement over 5% is flagged, never averaged.
                "peak_rss_bytes": peak_self or peak_time,
                "peak_rss_bytes_self": peak_self,
                "peak_rss_bytes_time_v": peak_time,
                "peak_rss_disagreement": disagree,
                "peak_rss_flag": ("DISAGREE >5%" if disagree and disagree > 0.05 else None),
            })
        if circ is not None:
            stim_rows.append({"d": d, "n": n, "timed": True,
                              "wall": summarize(times[("stim", None)])})
        if args.stim and os.path.exists(stim_path):
            os.remove(stim_path)

    # ---- the table, the grade, the conditions at the far end ----
    print_table(rows, stim_rows)
    g = grade(rows, stim_rows, args)
    print_grade(g)
    print()
    cond_end = conditions("at exit", cores_set)
    print_conditions(cond_end)

    l0, l1 = cond_start["loadavg"][0], cond_end["loadavg"][0]
    over = [w for w, v in (("start", l0), ("end", l1)) if v > args.max_load]
    citable = not over
    doc = {
        "harness": "conformance/bigqvm/mesh_h2h.py",
        "prereg": "conformance/bigqvm/MESH_CLIFFORD_PREREG.md",
        "gates": ["G3", "G4", "G1 cross-check", "G2 crossing fraction (number only)"],
        "protocol_validation": bool(args.dry_run),
        "citable": bool(citable and not args.dry_run),
        "note": args.note,
        "binary": args.binary,
        "binary_sha256": sha256_file(args.binary),
        "binary_mtime": os.path.getmtime(args.binary),
        "stim_version": stim_version,
        "engine_args": getattr(args, "engine_args", ""),
        "cores": cores_raw,
        "reps": args.reps, "rounds": args.rounds, "seed": args.seed,
        "d_requested": ds, "shards_requested": ss,
        "max_load": args.max_load,
        "load_gate": {"passed": citable, "over_at": over,
                      "loadavg_start": cond_start["loadavg"],
                      "loadavg_end": cond_end["loadavg"]},
        "conditions_start": cond_start,
        "conditions_end": cond_end,
        "loadavg_start": cond_start["loadavg"],
        "loadavg_end": cond_end["loadavg"],
        "rows": rows,
        "stim_rows": stim_rows,
        "verdict": g,
    }

    print()
    print(f"loadavg start {l0:.2f} -> end {l1:.2f}   (gate: <= {args.max_load})")
    if citable:
        with open(args.out, "w") as fh:
            json.dump(doc, fh, indent=1)
        print(f"wrote {args.out}")
    elif args.dry_run:
        with open(args.out, "w") as fh:
            json.dump(doc, fh, indent=1)
        print(f"LOAD GATE FAILED at {', '.join(over)} "
              f"(loadavg {l0:.2f} / {l1:.2f} > {args.max_load}).")
        print(f"--dry-run: wrote {args.out} anyway, stamped citable=false and "
              f"protocol_validation=true. It is a protocol validation, not a reading.")
    else:
        alt = args.out.replace(".json", "") + f".not-citable-load{max(l0, l1):.0f}.json"
        with open(alt, "w") as fh:
            json.dump(doc, fh, indent=1)
        print(f"REFUSING to write {args.out}: the 1-minute loadavg exceeded --max-load "
              f"{args.max_load} at {', '.join(over)} (start {l0:.2f}, end {l1:.2f}).")
        print("  The prereg refuses a table taken on a loaded box. The data is KEPT at")
        print(f"  {alt} and is explicitly NOT citable as a quiet-machine reading.")
        sys.exit(3)


if __name__ == "__main__":
    main()
