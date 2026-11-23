#!/usr/bin/env python3
"""Time N small flat-map tiles drawn one draw() per tile against all in one draw() (BATCH MATRIX).

    python3 crates/holon-draw/tools/tiles_timing.py [--n 8,44,200] [--reps 3] [--measure 6] [--settle 3]
                                                     [--size 1280x800] [--web DIR] [--count-only]

The page is web/bench-tiles.html: N tiles of 11 records in 5 batches each (tilesScene ?tiles=1 in
web/test-scene.js), every tile RESIDENT in a buffer of its own, drawn every animation frame
  A  percall  one draw() per tile, its matrix the call's vp, then the GL state the CEWP twin puts
              back after each of its draw() calls (tiling.js, tlHdTile), the twin's way today;
  B  one      one draw() for all, each batch carrying its tile's `vp` and `pixelsPerUnit`.
On the machine's GPU: Google Chrome (channel "chrome") through EGL, vsync and the frame-rate
limit off, so frames run unthrottled; the CEWP twin's tools/page/hd_timing/run.py runs the same
way. For each N, the runs are INTERLEAVED (A, B, A, B, ...; --reps pairs), each in a fresh page:
the page settles for --settle seconds, then every animation-frame interval is recorded for
--measure seconds. Reported per run and per N: the median and 95th percentile frame interval,
the mean interval (the measured time over the frames counted, which no timer's grain blurs),
the median and 95th percentile of the script time spent issuing the frame's draw() calls (and
the restores); and the GL calls of one frame of each (a separate page that wraps every
method of the context, ?count=1), with the calls per tile.
The frame interval is the whole page's frame (here nothing else is drawn), so it includes
Chrome's own compositing; on a shared machine it is noisy, so the spread over the reps is
printed beside the medians. With --web DIR the page is served from DIR. With --old DIR a third
arm, `old`, is interleaved with the other two: ?mode=percall served from DIR (a web/ directory
holding an older draw.js beside this bench page and test-scene.js), so the per-call path before
and after a change is timed under the same conditions. One Chrome at a time. No network beyond
127.0.0.1.
"""

import argparse
import functools
import http.server
import json
import os
import statistics
import sys
import threading
import time

from playwright.sync_api import sync_playwright

HERE = os.path.dirname(os.path.abspath(__file__))
WEB = os.path.abspath(os.path.join(HERE, "..", "web"))
ARGS = ["--use-angle=gl-egl", "--ignore-gpu-blocklist", "--disable-gpu-vsync", "--disable-frame-rate-limit"]


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def end_headers(self):
        # cross-origin isolated, so performance.now() and the frame timestamps are not coarsened to 0.1 ms
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        super().end_headers()


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--n", default="8,44,200")
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--measure", type=float, default=6)
    ap.add_argument("--settle", type=float, default=3)
    ap.add_argument("--size", default="1280x800")
    ap.add_argument("--web", default=WEB)
    ap.add_argument("--modes", default="percall,one")
    ap.add_argument("--old", default=None, help="a web/ directory with an older draw.js: adds the arm `old` (per call)")
    ap.add_argument("--count-only", action="store_true")
    ap.add_argument("--json", default=None, help="write every number here")
    args = ap.parse_args()
    w, h = (int(x) for x in args.size.split("x"))
    modes = args.modes.split(",") + (["old"] if args.old else [])
    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Quiet, directory=args.web))
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    base = "http://127.0.0.1:%d/bench-tiles.html" % srv.server_address[1]
    old_base = None
    if args.old:
        srv2 = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Quiet, directory=args.old))
        threading.Thread(target=srv2.serve_forever, daemon=True).start()
        old_base = "http://127.0.0.1:%d/bench-tiles.html" % srv2.server_address[1]
    out = {"size": [w, h], "args": ARGS, "runs": [], "counts": []}
    with sync_playwright() as pw:
        b = pw.chromium.launch(channel="chrome", headless=True, args=ARGS)
        out["chrome"] = b.version

        def page(qs):
            pg = b.new_page(viewport={"width": w, "height": h}, device_scale_factor=1)
            errs = []
            pg.on("pageerror", lambda e: errs.append(str(e)))
            pg.on("console", lambda m: errs.append(m.text) if m.type == "error" and "favicon" not in m.text and "404" not in m.text else None)
            if "mode=old" in qs:
                pg.goto(old_base + "?" + qs.replace("mode=old", "mode=percall"))
            else:
                pg.goto(base + "?" + qs)
            pg.wait_for_function("document.body.dataset.ready === '1'", timeout=180000)
            return pg, errs

        for n in [int(x) for x in args.n.split(",")]:
            for mode in modes:
                pg, errs = page("n=%d&mode=%s&count=1" % (n, mode))
                c = pg.evaluate("JSON.parse(document.body.dataset.count)")
                info = pg.evaluate("JSON.parse(document.body.dataset.info)")
                pg.close()
                c.update(n=n, mode=mode, errors=errs, glError=info.get("glError"))
                out["counts"].append(c)
                out["renderer"] = info.get("renderer")
                top = sorted(c["counts"].items(), key=lambda kv: -kv[1])
                print("count n=%-4d %-8s %6d GL calls a frame, %.1f a tile, %d draw() calls, %d draws; glError %s%s" % (
                    n, mode, c["total"], c["total"] / n, c["calls"], c["drawCalls"], info.get("glError"), ("; errors " + str(errs[:2])) if errs else ""))
                print("      " + ", ".join("%s %d" % kv for kv in top))
            if args.count_only:
                continue
            for rep in range(args.reps):
                for mode in modes:
                    pg, errs = page("n=%d&mode=%s" % (n, mode))
                    time.sleep(args.settle)
                    r = pg.evaluate("(ms) => window.__bench.start(ms)", int(args.measure * 1000))
                    iso = pg.evaluate("JSON.parse(document.body.dataset.info).isolated")
                    pg.close()
                    t, dr = r["t"], sorted(r["draw"])
                    iv = [y - x for x, y in zip(t, t[1:])]
                    p95 = lambda v: v[int(0.95 * (len(v) - 1))]
                    row = dict(n=n, mode=mode, rep=rep, frames=len(t), errors=errs, isolated=iso,
                               median=round(statistics.median(iv), 4) if iv else None,
                               p95=round(p95(sorted(iv)), 4) if iv else None,
                               mean=round((t[-1] - t[0]) / (len(t) - 1), 4) if len(t) > 1 else None,
                               draw_median=round(statistics.median(dr), 4) if dr else None,
                               draw_p95=round(p95(dr), 4) if dr else None)
                    out["runs"].append(row)
                    print("time  n=%-4d %-8s rep %d: frame median %7.3f p95 %7.3f mean %7.3f ms; drawing median %7.3f p95 %7.3f ms; %d frames%s%s" % (
                        n, mode, rep, row["median"] or -1, row["p95"] or -1, row["mean"] or -1, row["draw_median"] or -1, row["draw_p95"] or -1,
                        row["frames"], "" if iso else " (NOT isolated: timers coarse)", ("; errors " + str(errs[:2])) if errs else ""))
                    sys.stdout.flush()
        b.close()
    srv.shutdown()
    print("renderer: %s; chrome %s; %dx%d at dpr 1" % (out.get("renderer"), out.get("chrome"), w, h))
    if out["runs"]:
        print("each rep's number, then [median of the reps]")
        print("%-5s %-8s %-34s %-34s %-34s %-34s" % ("n", "mode", "frame median ms", "frame p95 ms", "frame mean ms", "drawing median ms (script)"))
        for n in sorted({r["n"] for r in out["runs"]}):
            meds = {}
            for mode in modes:
                rs = [r for r in out["runs"] if r["n"] == n and r["mode"] == mode]
                col = lambda k: " ".join("%.3f" % r[k] for r in rs) + " [%.3f]" % statistics.median(r[k] for r in rs)
                meds[mode] = statistics.median(r["median"] for r in rs)
                meds[mode + "_mean"] = statistics.median(r["mean"] for r in rs)
                meds[mode + "_draw"] = statistics.median(r["draw_median"] for r in rs)
                print("%-5d %-8s %-34s %-34s %-34s %-34s" % (n, mode, col("median"), col("p95"), col("mean"), col("draw_median")))
            for a, bb in (("one", "percall"), ("one", "old"), ("percall", "old")):
                if a in modes and bb in modes:
                    print("      %s / %s: frame median %.3f, frame mean %.3f, drawing %.3f" % (a, bb, meds[a] / meds[bb], meds[a + "_mean"] / meds[bb + "_mean"], meds[a + "_draw"] / meds[bb + "_draw"]))
    if args.json:
        with open(args.json, "w") as f:
            json.dump(out, f, indent=1)


if __name__ == "__main__":
    main()
