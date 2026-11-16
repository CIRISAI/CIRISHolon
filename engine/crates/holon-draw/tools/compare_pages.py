#!/usr/bin/env python3
"""Screenshot and time one browser page of docs/, the same way for any page.

Run it once per page (the "before" page, then the "after" page) with the same
flags, and compare the two JSON files it writes.

    python3 compare_pages.py atoms3d/index.html --label before \
        --ready '!!document.getElementById("boot") && document.getElementById("boot").hidden' \
        --fail  '!!document.querySelector("#boot-note.error")' \
        --pixels 'canvas'
    python3 compare_pages.py atoms-draw/index.html --label after --pixels 'canvas'

What "ready" means here, and it is two numbers on purpose:

  ready_js_ms      the first time the page's own ready expression is true
                   (default: document.body.dataset.ready === "1"), polled in the
                   page every 16 ms from an init script. This is the page's CLAIM.
  ready_pixels_ms  with --pixels SELECTOR: the first time, after the claim, that a
                   screenshot of that element is not one flat colour. This is the
                   harness's OBSERVATION, at a 200 ms polling grain plus the cost
                   of a screenshot, and it is the same test for every page.

`ready_ms` is the pixel number when --pixels is given and the claim otherwise.

WHAT THIS DOES NOT MEASURE. Chromium is launched headless with SwiftShader, so
every "GPU" operation is done on the CPU. Absolute frame times are not those of
any device with a GPU. The comparison between two pages under the same
conditions is what this measures, and nothing else. The phone mode's 6x CPU
throttle slows the renderer's main thread only; the SwiftShader rasteriser runs
in another process and is not slowed by it.

Outputs go to --out: <label>_<mode>.json, and for run 1 two screenshots.
No network beyond 127.0.0.1. One browser at a time.
"""

import argparse
import functools
import gzip
import http.server
import io
import json
import os
import statistics
import sys
import threading
import time

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))
DEFAULT_ROOT = os.path.join(REPO, "docs")
DEFAULT_OUT = (
    "/tmp/claude-1000/-home-emoore-CIRISHolon/"
    "4066a70d-c28a-428f-932b-46e9476d96db/scratchpad/compare"
)

CHROMIUM_ARGS = [
    "--use-angle=swiftshader",
    "--enable-unsafe-swiftshader",
    "--ignore-gpu-blocklist",
    "--enable-precise-memory-info",
]

CAVEAT = (
    "Rendering is SwiftShader (CPU rasterisation in headless Chromium), so absolute "
    "frame times are not a GPU device's. The comparison between pages measured under "
    "the same conditions is what this measures."
)

MODES = {
    "desktop": dict(viewport={"width": 1280, "height": 800}, device_scale_factor=1,
                    is_mobile=False, has_touch=False, cpu_throttle=1),
    "phone": dict(viewport={"width": 390, "height": 844}, device_scale_factor=3,
                  is_mobile=True, has_touch=True, cpu_throttle=6),
}


# ---------------------------------------------------------------- the server

class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = dict(http.server.SimpleHTTPRequestHandler.extensions_map)
    extensions_map.update({".wasm": "application/wasm", ".js": "text/javascript",
                           ".mjs": "text/javascript", ".json": "application/json"})
    sent = {}          # path -> bytes written for the most recent run
    lock = threading.Lock()

    def log_message(self, *args):
        pass

    def end_headers(self):
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def copyfile(self, source, outputfile):
        n = 0
        while True:
            buf = source.read(1 << 16)
            if not buf:
                break
            outputfile.write(buf)
            n += len(buf)
        with self.lock:
            key = self.path.split("?")[0]
            Handler.sent[key] = Handler.sent.get(key, 0) + n


def serve(root):
    handler = functools.partial(Handler, directory=root)
    httpd = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    httpd.daemon_threads = True
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    return httpd, thread


# ------------------------------------------------- what is injected in the page

INIT_JS = r"""
(() => {
  const READY_EXPR = __READY__;
  const FAIL_EXPR = __FAIL__;
  const MARKS = __MARKS__;   // name -> expression; each is timed when first true
  const C = (window.__cmp = {
    readyAt: null, failAt: null, longtasks: [], gaps: [], frames: [], pageRaf: [],
    wasmMemories: [], longtaskSupported: false, marks: {},
  });
  const test = (src) => { try { return !!(0, eval)(src); } catch (e) { return false; } };

  // The page's claim, polled. 16 ms grain; a blocked main thread delays it for any
  // page alike.
  const poll = setInterval(() => {
    if (C.readyAt === null && test(READY_EXPR)) C.readyAt = performance.now();
    if (FAIL_EXPR && C.failAt === null && test(FAIL_EXPR)) C.failAt = performance.now();
    let pending = false;
    for (const k of Object.keys(MARKS)) {
      if (!(k in C.marks) && test(MARKS[k])) C.marks[k] = performance.now();
      if (!(k in C.marks)) pending = true;
    }
    if (C.readyAt !== null && !pending && (!FAIL_EXPR || C.failAt !== null)) clearInterval(poll);
  }, 16);

  try {
    new PerformanceObserver((list) => {
      for (const e of list.getEntries()) C.longtasks.push([e.startTime, e.duration]);
    }).observe({ type: "longtask", buffered: true });
    C.longtaskSupported = true;
  } catch (e) {}

  // A heartbeat, because the longtask observer was MEASURED to miss work: a page that
  // ran about 1.5 s of synchronous wasm in a module after a top-level await reported
  // no long task for it. A 10 ms timer that arrives late is a stalled main thread
  // whatever the cause; every gap over 50 ms is kept as [start, length].
  let beat = performance.now();
  setInterval(() => {
    const n = performance.now();
    if (n - beat > 50) C.gaps.push([beat, n - beat]);
    beat = n;
  }, 10);

  // The harness's own frame clock, on the browser's native requestAnimationFrame
  // (captured before the page can wrap it), running from the start of the document.
  const nativeRaf = window.requestAnimationFrame.bind(window);
  const tick = (t) => { C.frames.push(performance.now()); nativeRaf(tick); };
  nativeRaf(tick);

  // The page's own frames: every callback the page hands to requestAnimationFrame
  // is timed. A page that draws only on demand shows few of these while the
  // harness clock above still runs at the display rate; both are reported.
  window.requestAnimationFrame = (cb) =>
    nativeRaf((t) => {
      const s = performance.now();
      try { return cb(t); } finally { C.pageRaf.push([s, performance.now() - s]); }
    });

  // WebAssembly linear memory, if the page instantiates a module. Optional.
  const grab = (res) => {
    try {
      const inst = res && (res.instance || res);
      const ex = inst && inst.exports;
      if (ex) for (const k of Object.keys(ex))
        if (ex[k] instanceof WebAssembly.Memory) C.wasmMemories.push(ex[k]);
    } catch (e) {}
    return res;
  };
  for (const name of ["instantiate", "instantiateStreaming"]) {
    const orig = WebAssembly[name];
    if (typeof orig === "function")
      WebAssembly[name] = function (...a) { return orig.apply(this, a).then(grab); };
  }
  const Inst = WebAssembly.Instance;
  WebAssembly.Instance = new Proxy(Inst, {
    construct(target, a) { const i = new target(...a); grab(i); return i; },
  });
})();
"""

COLLECT_JS = r"""
async () => {
  const C = window.__cmp;
  const nav = performance.getEntriesByType("navigation")[0];
  const pick = (e) => ({
    url: e.name, initiator: e.initiatorType || "navigation",
    transferSize: e.transferSize, encodedBodySize: e.encodedBodySize,
    decodedBodySize: e.decodedBodySize,
    start_ms: e.startTime, duration_ms: e.duration,
  });
  const resources = (nav ? [pick(nav)] : []).concat(
    performance.getEntriesByType("resource").map(pick));
  let webgl = null;
  try {
    const gl = document.createElement("canvas").getContext("webgl2");
    const ext = gl && gl.getExtension("WEBGL_debug_renderer_info");
    webgl = gl ? (ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL)
                      : gl.getParameter(gl.RENDERER)) : "no webgl2";
  } catch (e) { webgl = "webgl2 probe threw: " + e; }
  let webgpu = "no navigator.gpu";
  if (navigator.gpu) {
    try { webgpu = (await navigator.gpu.requestAdapter()) ? "adapter" : "navigator.gpu, null adapter"; }
    catch (e) { webgpu = "requestAdapter threw: " + e; }
  }
  const mem = performance.memory || {};
  return {
    readyAt: C.readyAt, failAt: C.failAt, marks: C.marks,
    longtaskSupported: C.longtaskSupported, longtasks: C.longtasks, gaps: C.gaps,
    frames: C.frames, pageRaf: C.pageRaf, resources,
    usedJSHeapSize: mem.usedJSHeapSize ?? null, totalJSHeapSize: mem.totalJSHeapSize ?? null,
    wasmMemoryBytes: C.wasmMemories.length
      ? [...new Set(C.wasmMemories)].reduce((s, m) => s + m.buffer.byteLength, 0) : null,
    engine: (document.body && document.body.dataset.engine) || null,
    canvases: [...document.querySelectorAll("canvas")].map((c) => ({
      id: c.id, width: c.width, height: c.height,
      css: [c.clientWidth, c.clientHeight] })),
    userAgent: navigator.userAgent, webgl, webgpu,
    devicePixelRatio: window.devicePixelRatio,
  };
}
"""


# ------------------------------------------------------------------ helpers

def flat_fraction(png_bytes):
    """Fraction of pixels that differ from the most common colour (0 = flat)."""
    from PIL import Image
    import numpy as np
    im = Image.open(io.BytesIO(png_bytes)).convert("RGB")
    im.thumbnail((640, 640))  # nearest is not needed; any resample keeps flat flat
    a = np.asarray(im).reshape(-1, 3).astype(np.uint32)
    codes = (a[:, 0] << 16) | (a[:, 1] << 8) | a[:, 2]
    values, counts = np.unique(codes, return_counts=True)
    return float(1.0 - counts.max() / codes.size), int(values.size)


def pct(sorted_values, q):
    if not sorted_values:
        return None
    i = min(len(sorted_values) - 1, max(0, int(round(q * (len(sorted_values) - 1)))))
    return sorted_values[i]


def interval_stats(stamps, t0, t1):
    # The window's two edges count as stamps, so a clock that stops part-way through
    # the window shows up as one long gap and a low rate rather than vanishing. The
    # rate is frames over the WHOLE window as it actually elapsed in the page.
    inside = [t for t in stamps if t0 <= t <= t1]
    edged = [t0] + inside + [t1]
    gaps = sorted(b - a for a, b in zip(edged, edged[1:]))
    if not inside:
        return dict(frames=0, fps=0.0, median_ms=None, p95_ms=None, max_ms=round(t1 - t0, 2))
    return dict(frames=len(inside), fps=round(len(inside) / ((t1 - t0) / 1000.0), 2),
                median_ms=round(statistics.median(gaps), 2),
                p95_ms=round(pct(gaps, 0.95), 2), max_ms=round(gaps[-1], 2))


def longtask_stats(tasks, t0, t1):
    durs = [d for s, d in tasks if t0 <= s < t1]
    return dict(count_over_50ms=len(durs), longest_ms=round(max(durs), 1) if durs else 0.0,
                total_ms=round(sum(durs), 1),
                blocking_ms=round(sum(d - 50 for d in durs), 1))


_gzip_cache = {}


def gzip_size(path):
    key = (path, os.path.getmtime(path), os.path.getsize(path))
    if key not in _gzip_cache:
        n = 0

        class Count(io.RawIOBase):
            def writable(self):
                return True

            def write(self, b):
                nonlocal n
                n += len(b)
                return len(b)
        with open(path, "rb") as f, gzip.GzipFile(fileobj=Count(), mode="wb", compresslevel=6) as g:
            while True:
                buf = f.read(1 << 20)
                if not buf:
                    break
                g.write(buf)
        _gzip_cache[key] = n
    return _gzip_cache[key]


# ------------------------------------------------------------------ one run

def one_run(pw, args, mode, base_url, run_index):
    from playwright.sync_api import TimeoutError as PwTimeout
    m = MODES[mode]
    with Handler.lock:
        Handler.sent = {}
    load_before = os.getloadavg()[0]
    browser = pw.chromium.launch(headless=True, args=CHROMIUM_ARGS)
    out = dict(run=run_index, mode=mode, error=None, host_load1_before=round(load_before, 2))
    try:
        context = browser.new_context(
            viewport=m["viewport"], device_scale_factor=m["device_scale_factor"],
            is_mobile=m["is_mobile"], has_touch=m["has_touch"])
        page = context.new_page()
        console_errors, console_warnings, page_errors = [], [], []
        responses = {}

        def on_console(msg):
            if msg.type == "error":
                console_errors.append(msg.text[:600])
            elif msg.type == "warning":
                console_warnings.append(msg.text[:300])
        page.on("console", on_console)
        page.on("pageerror", lambda e: page_errors.append(str(e)[:600]))
        page.on("response", lambda r: responses.__setitem__(
            r.url, dict(status=r.status, content_length=r.headers.get("content-length"),
                        content_type=r.headers.get("content-type"))))
        page.add_init_script(INIT_JS.replace("__READY__", json.dumps(args.ready))
                             .replace("__FAIL__", json.dumps(args.fail))
                             .replace("__MARKS__", json.dumps(getattr(args, "marks", None) or {})))
        if m["cpu_throttle"] != 1:
            cdp = context.new_cdp_session(page)
            cdp.send("Emulation.setCPUThrottlingRate", {"rate": m["cpu_throttle"]})

        url = base_url + args.page
        wall0 = time.monotonic()
        page.goto(url, wait_until="commit", timeout=args.timeout * 1000)
        try:
            page.wait_for_function(
                "window.__cmp && (window.__cmp.readyAt !== null || window.__cmp.failAt !== null)",
                timeout=args.timeout * 1000, polling=50)
        except PwTimeout:
            out["error"] = f"ready expression not true within {args.timeout} s"
        state = page.evaluate("({r: window.__cmp.readyAt, f: window.__cmp.failAt})")
        if state["r"] is None and state["f"] is not None:
            out["error"] = "the page's fail expression became true before ready"
        ready_js = state["r"]
        ready_pixels = None
        pixel_probe = None
        if out["error"] is None and args.pixels:
            last_blank = None
            deadline = time.monotonic() + args.timeout
            shots = 0
            while True:
                t0 = page.evaluate("performance.now()")
                png = page.locator(args.pixels).first.screenshot(type="png")
                t1 = page.evaluate("performance.now()")
                shots += 1
                frac, colours = flat_fraction(png)
                if frac >= args.pixel_fraction:
                    ready_pixels = t0
                    pixel_probe = dict(selector=args.pixels, non_flat_fraction=round(frac, 5),
                                       distinct_colours=colours, screenshots_taken=shots,
                                       last_blank_ms=last_blank, shot_began_ms=round(t0, 1),
                                       shot_ended_ms=round(t1, 1),
                                       threshold=args.pixel_fraction)
                    break
                last_blank = round(t0, 1)
                if time.monotonic() > deadline:
                    out["error"] = (f"{args.pixels} stayed one flat colour for {args.timeout} s "
                                    f"after the ready expression (fraction {frac:.5f})")
                    break
                page.wait_for_timeout(200)
        ready = ready_pixels if args.pixels else ready_js

        # Extra marks (e.g. "calibrated"): the frame window does not begin until every
        # one is true, so a start-up burst that follows the first frame is counted as
        # start-up for this page as it is for a page that bursts before its first frame.
        marks = getattr(args, "marks", None) or {}
        startup = ready
        if out["error"] is None and marks:
            try:
                page.wait_for_function(
                    "(names) => names.every((k) => k in window.__cmp.marks)",
                    arg=list(marks), timeout=args.timeout * 1000, polling=50)
            except PwTimeout:
                out["error"] = f"marks not all true within {args.timeout} s: {list(marks)}"
            got = page.evaluate("window.__cmp.marks")
            out["marks_ms"] = {k: round(v, 1) for k, v in got.items()}
            if out["error"] is None:
                startup = max([ready] + list(got.values()))
        out["startup_ms"] = round(startup, 1) if startup is not None else None

        if out["error"] is None:
            # ready + 1 s: a screenshot on run 1; the same wait on every run.
            page.wait_for_timeout(1000)
            if run_index == 1:
                p = os.path.join(args.out, f"{args.label}_{mode}_ready1s.png")
                page.screenshot(path=p)
                out["screenshot_ready_plus_1s"] = p
                page.wait_for_timeout(300)  # let the capture's own stall pass
            w0 = page.evaluate("performance.now()")
            page.wait_for_timeout(int(args.window * 1000))
            w1 = page.evaluate("performance.now()")
        data = page.evaluate(COLLECT_JS)
        if out["error"] is None and run_index == 1:
            p = os.path.join(args.out, f"{args.label}_{mode}_end.png")
            page.screenshot(path=p)
            out["screenshot_end"] = p
        fail_at_end = page.evaluate("window.__cmp.failAt")

        with Handler.lock:
            sent = dict(Handler.sent)
        resources = []
        for r in data["resources"]:
            path = r["url"].replace(base_url, "/", 1).split("?")[0] if r["url"].startswith(base_url) else None
            disk = os.path.join(args.root, path.lstrip("/")) if path else None
            if disk and os.path.isdir(disk):
                disk = os.path.join(disk, "index.html")
            resp = responses.get(r["url"], {})
            resources.append(dict(
                r, url=r["url"].replace(base_url, "/", 1),
                start_ms=round(r["start_ms"], 1), duration_ms=round(r["duration_ms"], 1),
                status=resp.get("status"), content_type=resp.get("content_type"),
                content_length=int(resp["content_length"]) if resp.get("content_length") else None,
                server_bytes_sent=sent.get(path) if path else None,
                gzip6_bytes_estimate=gzip_size(disk) if disk and os.path.isfile(disk) else None))
        tot = lambda k: sum(r[k] or 0 for r in resources)
        urls = " ".join(r["url"] for r in resources)
        out.update(
            url=args.page,
            ready_js_ms=round(ready_js, 1) if ready_js is not None else None,
            ready_pixels_ms=round(ready_pixels, 1) if ready_pixels is not None else None,
            ready_ms=round(ready, 1) if ready is not None else None,
            pixel_probe=pixel_probe,
            wall_s=round(time.monotonic() - wall0, 2),
            engine=data["engine"],
            graphics_path=("webgpu" if "/webgpu/" in urls else "webgl2" if "/webgl2/" in urls
                           else data["engine"]),
            webgpu_probe=data["webgpu"], webgl_renderer=data["webgl"],
            user_agent=data["userAgent"], device_pixel_ratio=data["devicePixelRatio"],
            canvases=data["canvases"],
            resources=resources, resource_count=len(resources),
            transfer_bytes=tot("transferSize"), encoded_bytes=tot("encodedBodySize"),
            decoded_bytes=tot("decodedBodySize"), server_bytes_sent=sum(sent.values()),
            gzip6_bytes_estimate=tot("gzip6_bytes_estimate"),
            js_heap_used_bytes=data["usedJSHeapSize"], js_heap_total_bytes=data["totalJSHeapSize"],
            wasm_memory_bytes=data["wasmMemoryBytes"],
            longtask_supported=data["longtaskSupported"],
            console_errors=console_errors, page_errors=page_errors,
            console_warnings=console_warnings[:20], console_warning_count=len(console_warnings),
            fail_expression_true_at_ms=fail_at_end,
            host_load1_after=round(os.getloadavg()[0], 2),
        )
        if out["error"] is None:
            lt_all = data["longtasks"]
            out["longtasks_to_ready"] = longtask_stats(lt_all, 0, ready)
            out["longtasks_in_window"] = longtask_stats(lt_all, w0, w1)
            out["longtasks_whole_run"] = longtask_stats(lt_all, 0, float("inf"))
            # Heartbeat gaps: a gap that STARTS before ready counts toward ready.
            out["stalls_to_ready"] = longtask_stats(data["gaps"], 0, ready)
            out["stalls_to_startup"] = longtask_stats(data["gaps"], 0, startup)
            out["stalls_in_window"] = longtask_stats(data["gaps"], w0, w1)
            out["window_ms"] = [round(w0, 1), round(w1, 1)]
            out["frames_harness_clock"] = interval_stats(data["frames"], w0, w1)
            starts = [s for s, d in data["pageRaf"]]
            pr = interval_stats(starts, w0, w1)
            durs = sorted(d for s, d in data["pageRaf"] if w0 <= s <= w1)
            pr.update(callback_median_ms=round(statistics.median(durs), 2) if durs else None,
                      callback_p95_ms=round(pct(durs, 0.95), 2) if durs else None,
                      callback_max_ms=round(durs[-1], 2) if durs else None,
                      callback_total_ms=round(sum(durs), 1))
            out["frames_page_raf"] = pr
        context.close()
    except Exception as e:  # a crashed run is a result, not a reason to lose the others
        out["error"] = f"{type(e).__name__}: {e}"[:800]
    finally:
        browser.close()
    return out


SUMMARY_KEYS = [
    ("ready_js_ms", lambda r: r.get("ready_js_ms")),
    ("ready_pixels_ms", lambda r: r.get("ready_pixels_ms")),
    ("ready_ms", lambda r: r.get("ready_ms")),
    ("startup_ms", lambda r: r.get("startup_ms")),
    ("longest_stall_to_startup_ms", lambda r: r.get("stalls_to_startup", {}).get("longest_ms")),
    ("stalled_ms_to_startup", lambda r: r.get("stalls_to_startup", {}).get("total_ms")),
    ("stalls_over_50ms_to_startup", lambda r: r.get("stalls_to_startup", {}).get("count_over_50ms")),
    ("transfer_bytes", lambda r: r.get("transfer_bytes")),
    ("decoded_bytes", lambda r: r.get("decoded_bytes")),
    ("gzip6_bytes_estimate", lambda r: r.get("gzip6_bytes_estimate")),
    ("resource_count", lambda r: r.get("resource_count")),
    ("longest_task_to_ready_ms", lambda r: r.get("longtasks_to_ready", {}).get("longest_ms")),
    ("tasks_over_50ms_to_ready", lambda r: r.get("longtasks_to_ready", {}).get("count_over_50ms")),
    ("blocking_ms_to_ready", lambda r: r.get("longtasks_to_ready", {}).get("blocking_ms")),
    ("longest_stall_to_ready_ms", lambda r: r.get("stalls_to_ready", {}).get("longest_ms")),
    ("stalls_over_50ms_to_ready", lambda r: r.get("stalls_to_ready", {}).get("count_over_50ms")),
    ("stalled_ms_to_ready", lambda r: r.get("stalls_to_ready", {}).get("total_ms")),
    ("longest_stall_in_window_ms", lambda r: r.get("stalls_in_window", {}).get("longest_ms")),
    ("stalls_over_50ms_in_window", lambda r: r.get("stalls_in_window", {}).get("count_over_50ms")),
    ("longest_task_in_window_ms", lambda r: r.get("longtasks_in_window", {}).get("longest_ms")),
    ("tasks_over_50ms_in_window", lambda r: r.get("longtasks_in_window", {}).get("count_over_50ms")),
    ("longest_task_whole_run_ms", lambda r: r.get("longtasks_whole_run", {}).get("longest_ms")),
    ("tasks_over_50ms_whole_run", lambda r: r.get("longtasks_whole_run", {}).get("count_over_50ms")),
    ("frame_median_ms", lambda r: r.get("frames_harness_clock", {}).get("median_ms")),
    ("frame_p95_ms", lambda r: r.get("frames_harness_clock", {}).get("p95_ms")),
    ("frame_max_ms", lambda r: r.get("frames_harness_clock", {}).get("max_ms")),
    ("fps", lambda r: r.get("frames_harness_clock", {}).get("fps")),
    ("page_raf_fps", lambda r: r.get("frames_page_raf", {}).get("fps")),
    ("page_raf_callback_median_ms", lambda r: r.get("frames_page_raf", {}).get("callback_median_ms")),
    ("page_raf_callback_p95_ms", lambda r: r.get("frames_page_raf", {}).get("callback_p95_ms")),
    ("js_heap_used_bytes", lambda r: r.get("js_heap_used_bytes")),
    ("wasm_memory_bytes", lambda r: r.get("wasm_memory_bytes")),
]


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("page", help="page path relative to --root, e.g. atoms3d/index.html (a query string is allowed)")
    ap.add_argument("--label", help="name used in output files (default: the page's directory)")
    ap.add_argument("--ready", default='document.body.dataset.ready === "1"',
                    help="JS expression that is true once the page claims its first frame")
    ap.add_argument("--fail", default=None,
                    help="optional JS expression that is true when the page reports failure")
    ap.add_argument("--mark", action="append", default=[], metavar="NAME=EXPR",
                    help="also time when this JS expression first becomes true (repeatable); the "
                         "frame window waits for every mark, and startup_ms is the latest of them and ready")
    ap.add_argument("--pixels", default=None, metavar="SELECTOR",
                    help="also require that a screenshot of this element is not one flat colour")
    ap.add_argument("--pixel-fraction", type=float, default=0.002,
                    help="share of pixels that must differ from the commonest colour (default 0.002)")
    ap.add_argument("--mode", choices=sorted(MODES), default="desktop")
    ap.add_argument("--runs", type=int, default=5)
    ap.add_argument("--window", type=float, default=10.0, help="seconds of frame timing after ready")
    ap.add_argument("--timeout", type=float, default=240.0, help="seconds to wait for ready")
    ap.add_argument("--root", default=DEFAULT_ROOT)
    ap.add_argument("--out", default=DEFAULT_OUT)
    args = ap.parse_args()
    args.root = os.path.abspath(args.root)
    args.marks = dict(m.split("=", 1) for m in args.mark)
    if not args.label:
        args.label = os.path.basename(os.path.dirname(args.page.split("?")[0])) or "page"
    if not os.path.isfile(os.path.join(args.root, args.page.split("?")[0])):
        sys.exit(f"no such page: {os.path.join(args.root, args.page.split('?')[0])}")
    os.makedirs(args.out, exist_ok=True)

    from playwright.sync_api import sync_playwright
    httpd, thread = serve(args.root)
    base_url = f"http://127.0.0.1:{httpd.server_address[1]}/"
    runs = []
    try:
        with sync_playwright() as pw:
            for i in range(1, args.runs + 1):
                r = one_run(pw, args, args.mode, base_url, i)
                runs.append(r)
                print(f"  run {i}: " + (f"ERROR {r['error']}" if r["error"] else
                      f"ready {r['ready_ms']} ms (claim {r['ready_js_ms']}), "
                      f"fps {r['frames_harness_clock']['fps']}, "
                      f"path {r['graphics_path']}, load {r['host_load1_before']}"),
                      file=sys.stderr, flush=True)
    finally:
        httpd.shutdown()
        httpd.server_close()
        thread.join(timeout=5)

    good = [r for r in runs if not r["error"]]
    medians = {}
    for key, get in SUMMARY_KEYS:
        vals = [get(r) for r in good if get(r) is not None]
        medians[key] = round(statistics.median(vals), 2) if vals else None
    m = MODES[args.mode]
    doc = dict(
        caveat=CAVEAT, label=args.label, page=args.page, mode=args.mode,
        viewport=m["viewport"], device_scale_factor=m["device_scale_factor"],
        cpu_throttle_rate=m["cpu_throttle"],
        cpu_throttle_note=("CDP Emulation.setCPUThrottlingRate slows the renderer main thread "
                           "only; SwiftShader rasterises in the GPU process, unthrottled."),
        chromium_args=CHROMIUM_ARGS, ready_expression=args.ready, fail_expression=args.fail,
        pixels_selector=args.pixels, pixel_fraction=args.pixel_fraction,
        ready_definition=("ready_ms = ready_pixels_ms (first non-flat screenshot of the selector "
                          "after the page's claim; 200 ms polling plus capture cost)"
                          if args.pixels else "ready_ms = ready_js_ms (the page's own claim)"),
        window_s=args.window, runs_requested=args.runs, runs_ok=len(good),
        cold_cache="a new browser process and context for every run",
        transfer_note=("The local server sends files uncompressed, so transfer_bytes is the raw "
                       "size. gzip6_bytes_estimate is what gzip -6 of the same files would be; "
                       "a host that compresses would send about that."),
        host=dict(cpus=os.cpu_count(), load1=[r.get("host_load1_before") for r in runs]),
        graphics_paths=sorted({str(r.get("graphics_path")) for r in good}),
        webgl_renderer=good[0]["webgl_renderer"] if good else None,
        webgpu_probe=good[0]["webgpu_probe"] if good else None,
        engine=good[0]["engine"] if good else None,
        medians=medians, runs=runs,
        command=" ".join([sys.executable] + [a if a.isalnum() else repr(a) for a in sys.argv]),
    )
    path = os.path.join(args.out, f"{args.label}_{args.mode}.json")
    with open(path, "w") as f:
        json.dump(doc, f, indent=1)

    mb = lambda b: "-" if b is None else f"{b / 1e6:.2f} MB"
    ms = lambda v: "-" if v is None else f"{v:.1f} ms"
    num = lambda v: "-" if v is None else f"{v:g}"
    g = medians
    print(f"\n{args.label} [{args.mode}] {args.page}: medians of {len(good)}/{args.runs} runs")
    print(f"  NOTE: {CAVEAT}")
    rows = [
        ("graphics path", ", ".join(doc["graphics_paths"]) or "-"),
        ("webgl renderer", str(doc["webgl_renderer"])),
        ("webgpu probe", str(doc["webgpu_probe"])),
        ("ready (claim / pixels)", f"{ms(g['ready_js_ms'])} / {ms(g['ready_pixels_ms'])}"),
        ("start-up (ready + marks)", f"{ms(g['startup_ms'])}  marks: {list(args.marks) or 'none'}; stalled "
                                     f"{ms(g['stalled_ms_to_startup'])}, longest {ms(g['longest_stall_to_startup_ms'])}"),
        ("transferred / decoded", f"{mb(g['transfer_bytes'])} / {mb(g['decoded_bytes'])}"
                                  f"  ({num(g['resource_count'])} resources)"),
        ("if gzip -6 (estimate)", mb(g["gzip6_bytes_estimate"])),
        ("long tasks to ready", f"{num(g['tasks_over_50ms_to_ready'])} over 50 ms, longest "
                                f"{ms(g['longest_task_to_ready_ms'])}, blocking {ms(g['blocking_ms_to_ready'])}"),
        ("main-thread stalls", f"to ready: {num(g['stalls_over_50ms_to_ready'])} over 50 ms, longest "
                               f"{ms(g['longest_stall_to_ready_ms'])}, total {ms(g['stalled_ms_to_ready'])}; "
                               f"in window: {num(g['stalls_over_50ms_in_window'])}, longest "
                               f"{ms(g['longest_stall_in_window_ms'])}  (10 ms heartbeat; trust over long tasks)"),
        ("long tasks in window", f"{num(g['tasks_over_50ms_in_window'])} over 50 ms, longest "
                                 f"{ms(g['longest_task_in_window_ms'])}"),
        ("frame interval", f"median {ms(g['frame_median_ms'])}, p95 {ms(g['frame_p95_ms'])}, "
                           f"max {ms(g['frame_max_ms'])}, {num(g['fps'])} fps"),
        ("page's own rAF", f"{num(g['page_raf_fps'])} per s, callback median "
                           f"{ms(g['page_raf_callback_median_ms'])}, p95 {ms(g['page_raf_callback_p95_ms'])}"),
        ("JS heap / wasm memory", f"{mb(g['js_heap_used_bytes'])} / {mb(g['wasm_memory_bytes'])}"),
        ("console errors", str(sum(len(r.get('console_errors', [])) + len(r.get('page_errors', []))
                                   for r in runs))),
        ("json", path),
    ]
    for k, v in rows:
        print(f"  {k:<24}{v}")
    for r in runs:
        for k in ("screenshot_ready_plus_1s", "screenshot_end"):
            if r.get(k):
                print(f"  {k:<24}{r[k]}")
    if len(good) < args.runs:
        sys.exit(1)


if __name__ == "__main__":
    main()
