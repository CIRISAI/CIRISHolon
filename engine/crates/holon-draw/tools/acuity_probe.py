#!/usr/bin/env python3
"""The pixel half of holon-draw's acuity gates: a FALSIFIER and a tightness meter.

The allocator's per-tile bound (`Candidate::error_px`, surfaced as `Frame::remainder_px`)
is an a-priori UPPER bound on how many pixels a tile is wrong by if drawn coarse. It is the
certificate. This tool draws fine and coarse with the real backend (`web/draw.js`, through
`web/probe.html`) and measures how far the drawing moved. That is a sampled LOWER bound: it
can kill the certificate and can never prove it.

    # 1. write the cases (the plants, a ball from every side, the standard ladder on a zoom)
    cd engine && cargo run -p holon-draw-ladder --example acuity_dump -- <cases dir>
    # 2. draw them and judge
    python3 crates/holon-draw/tools/acuity_probe.py --cases <cases dir> [--rust-log FILE]

`--rust-log` is the captured output of the Rust gates, quoted in the results file:
    cargo test -p holon-draw --test acuity_gates -- --nocapture
    cargo test -p holon-draw-ladder --test acuity_ladder -- --nocapture

Writes results/ACUITY_PROBE.json and results/ACUITY_PROBE.md. Exits 1 if a gate that must
hold does not (a plant not read, a plant not firing, the pixel probe seeing a zero-pixel
plant). A KILL of the certificate on the ladder or on the ball is a FINDING, reported and
exit 2, not a failure of the tool.

Headless Chromium with SwiftShader: one rasteriser, on the CPU. One browser at a time.
No network beyond 127.0.0.1.
"""

import argparse
import functools
import hashlib
import http.server
import json
import math
import os
import statistics
import subprocess
import sys
import threading
import time

HERE = os.path.dirname(os.path.abspath(__file__))
CRATE = os.path.abspath(os.path.join(HERE, ".."))
WEB = os.path.join(CRATE, "web")
RESULTS = os.path.join(CRATE, "results")
# A certificate under this many pixels is not compared as a ratio (the close look's grain is
# about 0.02 px); it is still tested for a kill.
RESOLVED_PX = 0.1
CHROMIUM_ARGS = ["--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"]


class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = dict(http.server.SimpleHTTPRequestHandler.extensions_map)
    extensions_map.update({".js": "text/javascript", ".json": "application/json", ".bin": "application/octet-stream"})
    cases = None

    def log_message(self, *args):
        pass

    def end_headers(self):
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def translate_path(self, path):
        clean = path.split("?")[0]
        if clean.startswith("/cases/"):
            return os.path.join(Handler.cases, os.path.basename(clean))
        return super().translate_path(path)


def serve(root, cases):
    Handler.cases = cases
    httpd = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Handler, directory=root))
    httpd.daemon_threads = True
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


def sha(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


# ------------------------------------------------- the instrument, a second time

def unpack(s, n):
    import numpy as np
    v = np.frombuffer(s.encode("latin1"), dtype=np.uint8) - 48
    bits = ((v[:, None] >> np.arange(6)) & 1).reshape(-1)
    return bits[:n].astype(bool)


def remeasure(native):
    """The same two distances from the same two masks, by scipy's distance transform: an
    implementation that shares nothing with the page's."""
    import numpy as np
    from scipy.ndimage import distance_transform_edt as edt
    w, h = native["w"], native["h"]
    a = unpack(native["fine_mask"], w * h).reshape(h, w)
    b = unpack(native["test_mask"], w * h).reshape(h, w)
    d = a ^ b
    if not d.any():
        return 0.0, 0.0
    thick = 2 * edt(np.pad(d, 1)).max() - 1
    yy, xx = np.mgrid[0:h, 0:w]
    edge = np.minimum(np.minimum(xx + 1, w - xx), np.minimum(yy + 1, h - yy))
    haus = 0.0
    for mine, other in ((a, b), (b, a)):
        if not other.any():
            continue
        dist = edt(~other)
        ok = mine & ~other & (dist <= edge)
        if ok.any():
            haus = max(haus, float(dist[ok].max()))
    return haus, float(thick)


# ------------------------------------------------- reading the results

def tile_reading(t):
    """The largest displacement read on a tile through its close looks, in device pixels,
    with the grain of that reading; None if it had no close look."""
    looks = t.get("close") or []
    if not looks:
        return None
    best = max(looks, key=lambda p: p["haus"])
    return best["haus"], 1.0 / best["scale"]


def quantiles(v):
    v = sorted(v)
    if not v:
        return None
    pick = lambda q: v[min(len(v) - 1, int(math.ceil(q * len(v))) - 1)]
    return {"n": len(v), "min": v[0], "median": statistics.median(v), "p90": pick(0.9), "max": v[-1]}


def fmt(x, nd=2):
    return "-" if x is None else f"{x:.{nd}f}"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--cases", required=True, help="the directory acuity_dump wrote")
    ap.add_argument("--out", default=RESULTS)
    ap.add_argument("--rust-log", default=None, help="captured output of the Rust gates, quoted in the results")
    ap.add_argument("--only", default=None, help="comma-separated groups to run (for a quick look; writes nothing)")
    ap.add_argument("--rows", default=None, help="with a partial run: write the per-tile readings here")
    ap.add_argument("--every", type=int, default=1, help="run every n-th ladder case (for a quick look; writes nothing)")
    args = ap.parse_args()

    with open(os.path.join(args.cases, "cases.json")) as f:
        index = json.load(f)
    cases = index["cases"]
    only = set(args.only.split(",")) if args.only else None
    partial = bool(only) or args.every != 1

    from playwright.sync_api import sync_playwright
    httpd = serve(WEB, os.path.abspath(args.cases))
    port = httpd.server_address[1]
    results, t0 = [None] * len(cases), time.time()
    with sync_playwright() as pw:
        browser = pw.chromium.launch(headless=True, args=CHROMIUM_ARGS)
        page = browser.new_page(viewport={"width": 1280, "height": 800})
        errors = []
        page.on("pageerror", lambda e: errors.append(str(e)))
        page.on("console", lambda m: errors.append(m.text) if m.type == "error" else None)
        page.goto(f"http://127.0.0.1:{port}/probe.html")
        page.wait_for_function("document.body.dataset.ready === '1'", timeout=120000)
        n = page.evaluate("probe.load('/cases/')")
        assert n == len(cases)
        context = page.evaluate("probe.context()")
        n_close = page.evaluate("probe.n_close")
        seen_ladder = 0
        for i, c in enumerate(cases):
            if only and c["group"] not in only:
                continue
            if c["group"] == "ladder":
                seen_ladder += 1
                if seen_ladder % args.every:
                    continue
            keep = c["group"].startswith("plant") and c["measure"] == "native"
            results[i] = page.evaluate("([i, k]) => probe.run(i, k)", [i, keep])
            if i % 50 == 0:
                print(f"  {i}/{len(cases)}  {time.time() - t0:.0f} s  {c['name']}", flush=True)
        browser.close()
    httpd.shutdown()
    if errors:
        print("PAGE ERRORS:", errors[:5])
        return 1

    gates, findings = [], []

    def gate(name, ok, detail):
        gates.append({"gate": name, "ok": bool(ok), "detail": detail})
        print(("  ok    " if ok else "  FAIL  ") + name + ": " + detail)

    # ---------------- (a) the planted errors of known size
    plants, instrument = [], []
    for c, r in zip(cases, results):
        if r is None or not c["group"].startswith("plant") or c["group"] == "plant_zero":
            continue
        t = r["tiles"][0]
        bound, k = t["bound_px"], c["expect_px"]
        slack = c.get("expect_slack_px", 0.0)
        if c["measure"] == "native":
            m, grain = t["native"], 1.0
            try:
                h2, th2 = remeasure(m)
                instrument.append({"case": c["name"], "page": [m["haus"], m["thick"]], "scipy": [h2, th2]})
            except ImportError:
                pass
        else:
            m, grain = t["close"][0], 1.0 / t["close"][0]["scale"]
        read = m["thick"] if c["group"] == "plant_drop" else m["haus"]
        which = "thick" if c["group"] == "plant_drop" else "haus"
        # `thick` is read between pixel centres: up to 3 grains under on a round region
        lo, hi = k - slack - (3.0 if which == "thick" else 1.5) * grain, k + 1.5 * grain
        killed = read > bound + 1.5 * grain
        row = {"case": c["name"], "group": c["group"], "measure": which, "planted_px": k, "read_px": read, "other_px": m["haus" if which == "thick" else "thick"], "grain_px": grain, "accept": [lo, hi], "certificate_px": bound, "killed": killed, "expect_kill": c["expect_kill"]}
        plants.append(row)
        gate(f"(a) the probe reads the plant: {c['name']}", lo <= read <= hi, f"planted {k:.3f} px, read {read:.3f} px ({which}, grain {grain:.3f}); accepted {lo:.3f} .. {hi:.3f}")
        if c["expect_kill"]:
            gate(f"(a) the falsifier FIRES on a planted kill: {c['name']}", killed, f"certificate {bound:.3f} px, read {read:.3f} px")
        else:
            gate(f"(a) the certificate covers the plant: {c['name']}", bound >= k and not killed, f"certificate {bound:.3f} px >= planted {k:.3f} px, read {read:.3f} px")
    for row in instrument:
        gate(f"(a) the page's distances are scipy's: {row['case']}", all(abs(a - b) < 1e-6 for a, b in zip(row["page"], row["scipy"])), f"page {row['page']}, scipy {row['scipy']}")

    # ---------------- (b) the zero-pixel plants
    zero = []
    for c, r in zip(cases, results):
        if r is None or c["group"] != "plant_zero":
            continue
        fired, control = c["views_gate_disagreements"] > 0, c["name"].startswith("control")
        zero.append({"case": c["name"], "pixels_that_differ": r["rgba_diff"], "pixels_drawn": r["pixels_drawn"], "records_fine": c["records_fine"], "records_test": c["records_test"], "pick_and_report_disagreements": c["views_gate_disagreements"], "first": c["views_gate_first"], "points_picked": c["points_picked"]})
        if control:
            gate(f"(b) {c['name']}", r["rgba_diff"] == 0 and not fired, f"{r['rgba_diff']} pixels differ of {r['pixels_drawn']} drawn; the pick-and-report gate is silent")
        else:
            gate(f"(b) the pixel probe MISSES it: {c['name']}", r["rgba_diff"] == 0 and r["pixels_drawn"] > 0, f"{r['rgba_diff']} pixels differ of {r['pixels_drawn']} drawn (every channel compared)")
            gate(f"(b) the pick-and-report gate FIRES on it: {c['name']}", fired, f"{c['views_gate_disagreements']} disagreements over {c['points_picked']} points and every tag, e.g. {c['views_gate_first'][:1]}")

    # ---------------- a ball from every side
    far, window = [], {}
    for c, r in zip(cases, results):
        if r is None or c["group"] not in ("ball_far", "ball_window"):
            continue
        t = r["tiles"][0]
        if c["group"] == "ball_far":
            far.append({"pitch": c["pitch_deg"], "yaw": c["yaw_deg"], "read": t["native"]["haus"], "bound": t["bound_px"], "r": c["r_px"]})
        else:
            look = t["close"][0]
            key = (c["window"], c["place"])
            window.setdefault(key, []).append({"pitch": c["pitch_deg"], "yaw": c["yaw_deg"], "read": look["haus"], "grain": 1 / look["scale"], "bound": t["bound_px"], "r": c["r_px"], "off_axis_px": c["off_axis_px"], "unread": look["unread"]})
    ball = {"far": None, "window": []}
    if far:
        worst = max(far, key=lambda x: x["read"])
        pole = next(x for x in far if x["pitch"] == 90 and x["yaw"] == 0)
        ball["far"] = {"views": len(far), "r_px": worst["r"], "certificate_px": worst["bound"], "worst_read_px": worst["read"], "worst_at": [worst["pitch"], worst["yaw"]], "pole_on_read_px": pole["read"], "least_read_px": min(x["read"] for x in far), "kills": sum(x["read"] > x["bound"] + 1.5 for x in far), "ratio": quantiles([x["bound"] / x["read"] for x in far if x["read"] > 0])}
    for (win, place), rows in sorted(window.items()):
        worst = max(rows, key=lambda x: x["read"])
        kills = [x for x in rows if x["read"] > x["bound"] + 1.5 * x["grain"]]
        ball["window"].append({"window": win, "place": place, "views": len(rows), "r_px": worst["r"], "off_axis_px": worst["off_axis_px"], "certificate_px": worst["bound"], "worst_read_px": worst["read"], "worst_at": [worst["pitch"], worst["yaw"]], "grain_px": worst["grain"], "kills": len(kills), "ratio": quantiles([x["bound"] / x["read"] for x in rows if x["read"] > 0])})
        if kills:
            findings.append(f"a ball in the {place} of a {win} window: read {worst['read']:.2f} px against a certificate of {worst['bound']:.2f} px, in {len(kills)} of {len(rows)} views")

    # ---------------- (c) the ladder
    rows = []
    for c, r in zip(cases, results):
        if r is None or c["group"] != "ladder":
            continue
        for t in r["tiles"]:
            if t["bound_px"] <= 0:
                continue
            row = {"rung": c["rung"], "z": c["z"], "window": c["window"], "yaw": c["yaw"], "pitch": c["pitch"], "tile": t["id"], "bound_px": t["bound_px"]}
            reading = tile_reading(t)
            if reading:
                row["read_px"], row["grain_px"] = reading
                row["marks"] = [{"kind": p["kind"], "bound_px": p["mark_bound_px"], "read_px": p["haus"], "thick_px": p["thick"], "grain_px": 1 / p["scale"], "unread": p["unread"]} for p in t["close"]]
            if "native" in t:
                nat = t["native"]
                row["native"] = {"n_diff": nat["n_diff"], "haus": nat["haus"], "thick": nat["thick"], "shade_most": nat["shading"]["most"], "shade_over_8": nat["shading"]["over_8"], "both": nat["shading"]["both"]}
            rows.append(row)
    read = [x for x in rows if "read_px" in x]
    kills = [x for x in read if x["read_px"] > x["bound_px"] + 1.5 * x["grain_px"]]
    mark_kills = [(x, m) for x in read for m in x["marks"] if m["read_px"] > m["bound_px"] + 1.5 * m["grain_px"]]
    for x in kills[:20]:
        findings.append(f"ladder: {x['rung']} at z {x['z']} on {x['window']} (yaw {x['yaw']}, pitch {x['pitch']}), tile {x['tile']}: read {x['read_px']:.3f} px against a certificate of {x['bound_px']:.3f} px")

    def dist(sel):
        got = [x for x in sel if "read_px" in x]
        # the ratio is taken where the CERTIFICATE is at least RESOLVED_PX (chosen on the
        # certificate, never on the reading): under that a reading is a grain or two of the
        # close look and its ratio is noise
        res = [x for x in got if x["bound_px"] >= RESOLVED_PX]
        some = [x for x in res if x["read_px"] > 0]
        q = quantiles([x["bound_px"] / x["read_px"] for x in some])
        return {"tiles": len(sel), "looked_at": len(got), "resolved": len(res), "read_zero": len(res) - len(some), "ratio": q, "largest_read_px": max((x["read_px"] for x in got), default=None), "largest_certificate_px": max((x["bound_px"] for x in sel), default=None), "kills": sum(x["read_px"] > x["bound_px"] + 1.5 * x["grain_px"] for x in got)}

    by_zoom = []
    for rung in sorted({x["rung"] for x in rows}, key=lambda n: min(x["z"] for x in rows if x["rung"] == n)):
        zs = sorted({math.floor(x["z"] * 4 + 1e-9) / 4 for x in rows if x["rung"] == rung})
        for zb in zs:
            sel = [x for x in rows if x["rung"] == rung and zb <= x["z"] + 1e-9 < zb + 0.25]
            by_zoom.append({"rung": rung, "z_from": zb, "z_to": zb + 0.25, **dist(sel)})
    by_rung = [{"rung": rung, **dist([x for x in rows if x["rung"] == rung])} for rung in sorted({x["rung"] for x in rows})]
    by_window = [{"window": w, **dist([x for x in rows if x["window"] == w])} for w in sorted({x["window"] for x in rows})]
    nat = [x["native"] for x in rows if "native" in x]
    native = None
    if nat:
        native = {"tiles": len(nat), "no_pixel_differs": sum(n["n_diff"] == 0 for n in nat), "most_pixels_differ": max(n["n_diff"] for n in nat), "largest_haus_px": max(n["haus"] for n in nat), "largest_thick_px": max(n["thick"] for n in nat), "largest_shading_step": max(n["shade_most"] for n in nat), "tiles_with_a_shading_step_over_8": sum(n["shade_over_8"] > 0 for n in nat)}

    ok = all(g["ok"] for g in gates)
    out = {
        "what": "holon-draw acuity probe: a falsifier and a tightness meter for the allocator's per-tile bound. Never a certificate.",
        "date": time.strftime("%Y-%m-%d"),
        "renderer": context, "chromium_args": CHROMIUM_ARGS, "close_look_canvas_px": n_close,
        "draw_js_sha256": sha(os.path.join(WEB, "draw.js")), "probe_html_sha256": sha(os.path.join(WEB, "probe.html")),
        "cases": len(cases), "cases_run": sum(r is not None for r in results), "seconds": round(time.time() - t0),
        "eps_px": index["eps_px"], "ladder_rungs": index["ladder_rungs"],
        "gates": gates, "gates_ok": ok, "findings": findings,
        "plants": plants, "instrument_check": instrument, "zero_pixel": zero, "ball": ball,
        "ladder": {"overall": dist(rows), "by_rung": by_rung, "by_window": by_window, "by_zoom": by_zoom, "native_grid": native, "kills": kills, "marks_over_their_own_term": len(mark_kills), "marks_looked_at": sum(len(x["marks"]) for x in read)},
    }
    print(f"\n{len(gates)} gates, {'all hold' if ok else 'SOME FAIL'}; {len(findings)} findings; {out['seconds']} s")
    for f in findings[:10]:
        print("  FINDING:", f)
    if partial:
        print("(a partial run: nothing written to the results)")
        if args.rows:
            with open(args.rows, "w") as f:
                json.dump({"rows": rows, "ball": ball}, f)
        print(json.dumps(out["ladder"]["overall"], indent=1))
        return 0 if ok else 1
    os.makedirs(args.out, exist_ok=True)
    out["ladder"]["tiles"] = rows
    with open(os.path.join(args.out, "ACUITY_PROBE.json"), "w") as f:
        json.dump(out, f, indent=1)
    rust = open(args.rust_log).read() if args.rust_log else None
    with open(os.path.join(args.out, "ACUITY_PROBE.md"), "w") as f:
        f.write(markdown(out, rust))
    print("wrote", os.path.join(args.out, "ACUITY_PROBE.json"), "and .md")
    return 1 if not ok else (2 if findings else 0)


# ------------------------------------------------- the results file

def ratio_cells(d):
    q = d["ratio"]
    if not q:
        return "- | - | - | -"
    return f"{q['min']:.2f} | {q['median']:.2f} | {q['p90']:.2f} | {q['max']:.1f}"


def markdown(o, rust):
    L = []
    w = L.append
    lad = o["ladder"]
    w("# The acuity probe: falsifier gates for holon-draw's allocator")
    w("")
    w(f"Run {o['date']}. Written by `tools/acuity_probe.py` from `results/ACUITY_PROBE.json`; do not edit by hand.")
    w("")
    w("## What is certified, and what this is")
    w("")
    w("The allocator's per-tile bound (`Candidate::error_px`; the largest left coarse is `Frame::remainder_px`) is an")
    w("a-priori UPPER bound on how many pixels a tile is wrong by if drawn coarse. That bound is the certificate, and")
    w("it stays the certificate.")
    w("")
    w("Everything below compares a coarse drawing with a fine one. A comparison is a sampled LOWER bound on the error:")
    w("it can KILL the certificate (by reading more than it allows) and it can never prove it. It is a FALSIFIER, and,")
    w("where it does not kill, a meter of how loose the certificate is. It is not a certificate.")
    w("")
    w("A frame is read three ways at once: the pixels, the pick and the report. A coarse draw has to stay inside all")
    w("three, which is why a tagged mark is coarsened and never removed. The pixel probe alone cannot see the other two.")
    w("")
    w("## The gates")
    w("")
    w("| gate | holds | reading |")
    w("|---|---|---|")
    for g in o["gates"]:
        w(f"| {g['gate']} | {'yes' if g['ok'] else '**NO**'} | {g['detail']} |")
    w("")
    w(f"{len(o['gates'])} gates; {'all hold' if o['gates_ok'] else 'SOME DO NOT HOLD'}.")
    w("")
    w("## Findings (kills of the certificate)")
    w("")
    if o["findings"]:
        for f in o["findings"]:
            w(f"- {f}")
    else:
        w("None: no reading exceeded its certificate by more than the reading's grain.")
    w("")
    w("## (a) The pixel probe, on planted errors of known size")
    w("")
    w("THE MEASURE is a distance, not a count. `haus` is the outline's displacement: the farthest that any pixel covered")
    w("in one drawing and not in the other lies from the other drawing's coverage (the symmetric Hausdorff distance of")
    w("the two covered regions). It is the measure for a mark drawn on a cheaper mesh, whose outline falls short.")
    w("`thick` is the width of the wrong region (the diameter of the largest disc inside the pixels that differ). It is")
    w("the measure for a DROPPED mark, where the wrong region is the mark itself, and it agrees with `haus` on a moved")
    w("outline. A close look reads both on a window of the view rasterised S times finer, in device pixels with a grain")
    w("of 1/S.")
    w("")
    w("| plant | measure | planted px | read px | the other measure | grain | certificate px | certificate killed |")
    w("|---|---|---|---|---|---|---|---|")
    for p in o["plants"]:
        w(f"| {p['case']} | {p['measure']} | {p['planted_px']:.3f} | {p['read_px']:.3f} | {p['other_px']:.3f} | {p['grain_px']:.3f} | {p['certificate_px']:.3f} | {'YES (planted)' if p['killed'] and p['expect_kill'] else 'YES' if p['killed'] else 'no'} |")
    w("")
    w("The shift plants move a disc and a box by k pixels only in the frame under test. The core prices that tile at 0")
    w("(nothing in it is coarsened), so there the probe must both read k and find the certificate killed: the falsifier")
    w("is shown to fire. The ball is the real coarse path (`Batch::coarse`, the 10 by 6 mesh) seen down its pole, where")
    w("the outline is a decagon; it is read against the FULL mesh (28 by 14), which is itself up to r (1 - cos(pi/28))")
    w("inside the true ball, so the reading may be that much under k.")
    if o["instrument_check"]:
        w("")
        w(f"The page's two distances were recomputed from the same masks by scipy's distance transform for {len(o['instrument_check'])} plants and agree to 1e-6.")
    w("")
    w("## (b) A coarse draw that moves no pixel")
    w("")
    w("One tile on a flat map: a small tagged box (tag 2, detail 1) wholly inside an opaque tagged box, and two discs of")
    w("one colour and size (tags 3 and 4). Two plants are applied to the core's own coarse frame: the coarse draw DROPS")
    w("the tagged mark it should only have flagged (the fault the 2026-11-15 correction in `lib.rs` removed, put back),")
    w("and the coarse draw hands two marks each other's tag.")
    w("")
    w("| frame under test | records drawn (fine, test) | pixels that differ, of those drawn | pick-and-report disagreements | first |")
    w("|---|---|---|---|---|")
    for z in o["zero_pixel"]:
        first = z["first"][0] if z["first"] else "-"
        w(f"| {z['case']} | {z['records_fine']:.0f}, {z['records_test']:.0f} | {z['pixels_that_differ']} of {z['pixels_drawn']} | {z['pick_and_report_disagreements']:.0f} | {first} |")
    w("")
    w("Every channel of every pixel is compared. The pixel probe reads nothing on either plant; the gate over the other")
    w("two views (`gates::views_disagree`: `Frame::pick` at a grid of points and at every mark, `Frame::report` for every")
    w("tag) fires on both and is silent on the control. For the exchanged tags the Rust gate also shows the miss holds on")
    w("any backend: the bytes a shader is given are identical.")
    w("")
    w("## A ball from every side: how tight the sphere's term is")
    w("")
    b = o["ball"]
    if b["far"]:
        f_ = b["far"]
        r = f_["r_px"]
        w(f"From far (orthographic to a part in a million), a ball {r:.0f} px in radius, {f_['views']} views over one face of the mesh")
        w("(pitch 0 to 90 by 5 degrees, yaw 0 to 36 by 6), coarse against fine, at the device's own pixels:")
        w("")
        w(f"- certificate {f_['certificate_px']:.2f} px ({100 * f_['certificate_px'] / r:.2f} % of the radius: `coarse_shortfall` times the DIAMETER);")
        w(f"- down the pole the outline moves {f_['pole_on_read_px']:.2f} px ({100 * f_['pole_on_read_px'] / r:.2f} %): the 1 - cos(pi/10) = 4.89 % that `coarse_shortfall` names;")
        w(f"- the worst view read is pitch {f_['worst_at'][0]:.0f}, yaw {f_['worst_at'][1]:.0f}: {f_['worst_read_px']:.2f} px ({100 * f_['worst_read_px'] / r:.2f} %). The Rust gate derives 7.84 % against the true ball for a view edge-on to a face beside the equator; the full mesh is itself up to 1.26 % inside;")
        w(f"- certificate over reading across the views: least {f_['ratio']['min']:.2f}, median {f_['ratio']['median']:.2f}, largest {f_['ratio']['max']:.2f}; kills {f_['kills']}.")
        w("")
    if b["window"]:
        w("With the ladder's camera (`rung_camera`: the eye's distance fixed by the zoom, the field of view widening with the")
        w("window), a ball of 60 px nominal radius at the centre and in the top right corner of each window, 36 views each")
        w("(pitch 0 to 80 by 10, yaw 0 to 27 by 9), through a close look. Off the axis a perspective view stretches a ball")
        w("along the line from the centre, and `Camera::pixels` prices it at its nominal size:")
        w("")
        w("| window | place | off axis px | certificate px | worst read px | grain | least ratio | median ratio | views that kill |")
        w("|---|---|---|---|---|---|---|---|---|")
        for x in b["window"]:
            q = x["ratio"]
            w(f"| {x['window']} | {x['place']} | {x['off_axis_px']:.0f} | {x['certificate_px']:.2f} | {x['worst_read_px']:.2f} | {x['grain_px']:.2f} | {q['min']:.2f} | {q['median']:.2f} | {x['kills']} of {x['views']} |")
        w("")
    w("## (c) The ratio on the real ladder")
    w("")
    w("The standard ladder with no host, flown from the finest zoom to the coarsest: the page's opening pose (yaw 0.8,")
    w("pitch 0.42) every 0.05 of z on three windows (1280x800; 1170x2532, a phone at three device pixels a point;")
    w("3840x2160), and two more poses (yaw -1.9, pitch 0.1; yaw 2.6, pitch 1.2) every 0.15 on the desk window. The")
    w("simulated rungs are stepped between poses as the page steps them. Every frame in which the allocator (the real")
    w("one, `Acuity { eps_px: 1 }`) left a tile coarse with a bound above zero is a case; the tool checks that the frame")
    w("it judges is the frame `Ladder::passes` built.")
    w("")
    w("Per coarse tile: its certificate (read from the core: the tile framed alone, forced coarse) over the largest")
    w("outline displacement read on it. The reading is through close looks at up to four of its marks (the two with")
    w("the largest term of the bound, and the two with the")
    w("largest term times the stretch a perspective view gives a ball off its axis), each drawn with the tile alone. A")
    w("ratio of 1 would be a tight bound; under 1 is a kill. The ratio is taken over tiles whose CERTIFICATE is at least")
    w("0.1 px (a choice made on the certificate, never on the reading): under that a reading is a grain or two of the")
    w("close look. Every tile looked at is tested for a kill (a reading over its certificate by more than 1.5 grains).")
    w("")
    w("| rung | coarse tiles | looked at | certificate >= 0.1 px | of those, read 0 | least ratio | median | 90th pct | largest | largest read px | largest certificate px | kills |")
    w("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for x in lad["by_rung"] + [{"rung": "ALL", **lad["overall"]}]:
        w(f"| {x['rung']} | {x['tiles']} | {x['looked_at']} | {x['resolved']} | {x['read_zero']} | {ratio_cells(x)} | {fmt(x['largest_read_px'], 3)} | {fmt(x['largest_certificate_px'], 3)} | {x['kills']} |")
    w("")
    w("By window:")
    w("")
    w("| window | coarse tiles | looked at | certificate >= 0.1 px | least ratio | median | 90th pct | largest | largest read px | kills |")
    w("|---|---|---|---|---|---|---|---|---|---|")
    for x in lad["by_window"]:
        w(f"| {x['window']} | {x['tiles']} | {x['looked_at']} | {x['resolved']} | {ratio_cells(x)} | {fmt(x['largest_read_px'], 3)} | {x['kills']} |")
    w("")
    w("By zoom (z is log10 of the metres a device pixel spans; bins of a quarter decade, all poses and windows):")
    w("")
    w("| rung | z | coarse tiles | looked at | certificate >= 0.1 px | of those, read 0 | least ratio | median | 90th pct | largest | largest read px | largest certificate px | kills |")
    w("|---|---|---|---|---|---|---|---|---|---|---|---|---|")
    for x in lad["by_zoom"]:
        w(f"| {x['rung']} | {x['z_from']:.2f} .. {x['z_to']:.2f} | {x['tiles']} | {x['looked_at']} | {x['resolved']} | {x['read_zero']} | {ratio_cells(x)} | {fmt(x['largest_read_px'], 3)} | {fmt(x['largest_certificate_px'], 3)} | {x['kills']} |")
    w("")
    w(f"Marks looked at: {lad['marks_looked_at']}. Marks whose reading exceeded their OWN term of the bound (the tile's certificate is the largest term, so this can happen without a kill): {lad['marks_over_their_own_term']}.")
    n = lad["native_grid"]
    if n:
        w("")
        w(f"At the device's own pixels (the desk window, each coarse tile drawn alone, whole view): of {n['tiles']} tiles, {n['no_pixel_differs']} change no pixel's coverage at all; the most pixels any changes is {n['most_pixels_differ']}; the largest displacement read on that grid is {n['largest_haus_px']:.2f} px and the widest wrong region {n['largest_thick_px']:.2f} px (a single pixel that flips reads 1). Where both drawings cover a pixel the largest colour step between them is {n['largest_shading_step']} of 255, and {n['tiles_with_a_shading_step_over_8']} tiles have a step over 8: the cheap mesh shades differently inside its outline, which the bound does not price and this probe does not judge.")
    w("")
    w("## (d) Drawing never writes, and the other two views on the ladder (Rust)")
    w("")
    w("`cargo test -p holon-draw --test acuity_gates` and `cargo test -p holon-draw-ladder --test acuity_ladder`.")
    w("The allocator is forced through two `Allocator` impls that live in the tests (`tests/gates/mod.rs`: `AllFine`,")
    w("`AllCoarse`); nothing in `src/` was changed.")
    w("")
    w("- The hydrogen rung (`holon-render`'s `Sim` through `holon-draw-atoms`) and the hadron chain, each run 240 frames")
    w("  from a fresh scene with every frame forced ALL FINE and again ALL COARSE, on the same clock and zoom path: the")
    w("  simulation's state (every atom's position and velocity and the substep count; the chain's densities, link")
    w("  energies, time, norm and energy) is bit-identical after every frame, and so is every report value that is not")
    w("  the allocation's own statement of itself (`Shown::coarse` and the `shown` digest). A third run, all fine again,")
    w("  is the control that the run repeats at all.")
    w("- The whole standard ladder flown from the ground to the end of the record twice, every pass framed all fine in")
    w("  one flight and all coarse in the other: what every scene hands the allocator on every frame is bit-identical.")
    w("- On a sweep of the whole ladder at three poses and two windows, the pick (at a grid and at every mark) and the")
    w("  report (every tag) of the frame as allocated, and of the frame forced all coarse, equal the all-fine frame's;")
    w("  what is left coarse is within the pixel of budget; and the frame judged is the frame `Ladder::passes` built.")
    if rust:
        w("")
        w("```")
        for line in rust.splitlines():
            if line.startswith(("test ", "running ", "cheap ball", "plant ", "hydrogen", "hadron", "whole ladder")) or " frames, " in line:
                w(line)
        w("```")
    w("")
    w("## What each gate shows, and what it does not")
    w("")
    w("- **(a)** shows that the probe reads a displacement of known size to within its grain, at the device's pixels and")
    w("  under a pixel through a close look, that it fires when a certificate is too small (the shift plant), and that the")
    w("  core's certificate covers the two errors it prices (a cheap mesh, a dropped decoration) in the planted views. It")
    w("  does not show the certificate holds in any other view.")
    w("- **(b)** shows that a pixels-only probe is blind to a coarse draw that changes a pick or a report, and that the")
    w("  gate over those two views is not. It does not show that the pick and the report are RIGHT, only that coarse and")
    w("  fine agree at the points and tags compared.")
    w("- **(c)** is a meter. Each ratio is a certificate over a LOWER bound of the error, so the true ratio is at most")
    w("  what is printed. No kill here is not a proof: the poses are a sample (five flights, three windows), the marks")
    w("  looked at are a sample of each tile (chosen where the bound's own terms are largest), each tile is drawn ALONE")
    w("  (an outline that moves because another tile's ball in front of it shrank is not read), only coverage is judged")
    w("  (not shading, and not how faint a rung is in a cross-fade), and the reference is the full mesh, not the true ball.")
    w("- **(d)** shows that on these runs the simulations and the reports did not depend on the allocation. It is the")
    w("  Rust core and the scenes; it does not exercise the browser (`abi.rs` and `draw.js`), where the allocator is")
    w("  fixed and cannot be forced without a hook that was not added.")
    w(f"- **The rasteriser** is SwiftShader in headless Chromium ({o['renderer']['renderer']}), antialiasing off; the")
    w("  ladder page asks for antialiasing. Another GPU's rasteriser may cover a pixel on an edge differently. The")
    w("  distances read at close range are of geometry many pixels wide and do not hang on one edge pixel; the readings")
    w("  at the device's own pixels do.")
    w("")
    w("## Reproduce")
    w("")
    w("```")
    w("cd engine")
    w("cargo test -p holon-draw --test acuity_gates -- --nocapture")
    w("cargo test -p holon-draw-ladder --test acuity_ladder -- --nocapture")
    w("cargo run -p holon-draw-ladder --example acuity_dump -- <cases dir>")
    w("python3 crates/holon-draw/tools/acuity_probe.py --cases <cases dir> --rust-log <the two tests' output>")
    w("```")
    w("")
    w(f"`draw.js` sha256 `{o['draw_js_sha256'][:16]}`, `probe.html` `{o['probe_html_sha256'][:16]}`; {o['cases_run']} cases in {o['seconds']} s; close looks on a {o['close_look_canvas_px']} px canvas.")
    w("")
    return "\n".join(L)


if __name__ == "__main__":
    sys.exit(main())
