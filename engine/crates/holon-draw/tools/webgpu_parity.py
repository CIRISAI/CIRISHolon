#!/usr/bin/env python3
"""Draw test.html (WebGL) and test-webgpu.html (WebGPU) the same way and compare their pixels.

    python3 crates/holon-draw/tools/webgpu_parity.py [--out DIR] [--width 960 --height 600]
                                                      [--modes default,map,noshadow,images,resident,...] [--fps 4]

For each mode (default: the 3D scene with shadows; map: ?map=1; noshadow: ?shadow=0; images:
?images=1, a panel of host textures; resident, residentmap, residentimages: the default, map
and images modes with ?resident=1, the world batches, and in residentimages the image panel's
batches, drawn from resident buffers; icons: ?icons=1, the 16 glyph shapes in the world and on
a screen panel, FLOW on lines and arcs (tube, flat ribbon, screen, one resident) at phase
0.25, and a 400-unit ground patch under the RING; iconsmove: the same at phase 0.75; tiles:
?tiles=1, 48 small flat-map tiles each with its own matrix (BATCH MATRIX), drawn in one draw();
tiles3d: ?tiles=2, 42 tiles of lit marks and camera-facing glyphs in one perspective view, each
tile's matrix the camera's times its own translation; add: ?add=1, BLEND 'add'; globe: ?globe=1,
kind 13 TRIANGLE, GLOBE and BLEND 'multiply') and for
the scene alone (?overlay=0) and with the panels, it loads both pages in one headless Google
Chrome at the same size, device pixel ratio 1 and camera, waits for the page's own
`document.body.dataset.ready`, takes a screenshot of the canvas, and compares the two:

  mean_abs     mean absolute difference per channel (RGB, 0..255) over every pixel
  over_8       % of pixels whose largest channel difference is over 8
  over_32      % of pixels whose largest channel difference is over 32
  max          the largest channel difference anywhere

A diff picture (|a - b| x 4) is written beside the two screenshots. With --fps N each page is
also run with ?spin=1 for N seconds and its frames per second counted.

It also checks that the mesh builders copied into draw-webgpu.js are still the same source
text as draw.js's (they are copies; a drift would make the backends draw different meshes),
that each page reported ready with no error, that the WebGPU page's verdict is PASS, and that
both backends skip the same images for an empty slot (one, in the images mode with panels).

RESIDENT MODES ARE GATED against their base mode ON THE SAME BACKEND (the same page without
?resident=1, loaded too if the base mode is not in --modes): the two screenshots must be
identical (max 0), the draw calls and instances the same, and stats().bytesUploaded the base's
less exactly the bytes made resident (so 0 for the scene alone); the page's own checks of
update() (grow, shrink), release() and reuse of a handle, and of the three throws, must hold
(document.body.dataset.resident.ok). For the resident mode with panels, test.html is also
compared with itself on WebGL 1 (?gl=1) and on bare WebGL 1 (?gl=1&novao=1, no vertex array
objects), resident against bytes, with the same gate.
ICONS MODES ARE GATED too: on each backend the page's own check of the resident flow link must
hold (drawn at three phases, stats().bytesUploaded 0 each time, document.body.dataset.flow.ok);
the ground patch under the ring must show at GROUND_PX (green there, not the clear colour);
iconsmove must differ from icons on the same backend (the dashes moved); and test.html?icons=1
with the panels must draw with gl error 0 on WebGL 1, with ?mediump=1, both, ?nohigh=1 (the
twin's no-highp gate; no fragment source may still say highp) and bare WebGL 1 (?novao=1),
each compared with WebGL 2 and the numbers reported.
TILES MODES ARE GATED on each backend: draw() must throw on a batch's own vp with opts.shadow,
on a blend other than 'alpha' or 'add', on a vp of fewer than 16 numbers and on a world batch
with no matrix at all, and must draw a batch's own pixelsPerUnit with shadows
(document.body.dataset.refusals.ok); the one-draw() picture against ?percall=1 (one draw() per
tile, with the twin's GL state restore after each on WebGL), ?resident=1 (each tile's batches
in a resident buffer of its own) and ?resident=1&percall=1 must be identical (max 0), with the
same draw calls and instances, and with the panels off the resident pages upload no record;
with the panels, test.html is also drawn on WebGL 1, with ?mediump=1, both, ?nohigh=1 and bare
WebGL 1, one draw() against per call (max 0) with gl error 0 and no highp left.
THE ADD MODE IS GATED on each backend, with the panels off: at every point the page lists
(document.body.dataset.add.points: each ground alone, each additive disc alone, the two
overlapping, world and screen, over a dark and a bright ground, and the depth check) the
screenshot is the arithmetic's colour within ADD_TOL per channel; on WebGL the blend state read
back after draw() is the one GL STATE promises (dataset.add.ok); ?resident=1 is identical to
the bytes page (max 0), panels on and off; and the WebGL 1 / mediump / nohigh / bare WebGL 1
pages pass the same points with gl error 0.
THE GLOBE MODE IS GATED on each backend (2026-11-23): draw() must throw on a globe with
shadows, with no sun, a zero sun or a short `day`, and on a bad blend, and must draw a multiply
batch and a sun-only globe (dataset.refusals.ok); with the panels off, at every point the page
lists (dataset.globe.points: nine on the lit hemisphere, from the day side through the
terminator to the limb, and sixteen of multiplying discs, world and screen, over a dark and a
bright ground) the screenshot is the arithmetic's colour within GLOBE_TOL per channel; the 18
squares drawn as 36 TRIANGLE records and as 18 QUAD records (?triquads=1) are identical (max 0);
?resident=1 is identical to bytes (max 0), panels on and off; on WebGL the blend state after
draw() is GL STATE's; and the WebGL 1 / mediump / nohigh / bare WebGL 1 pages pass the points
and the triangle check with gl error 0. --plant tri|globe|mul plants a fault in the scene (one
triangle's t moved; the sun turned 12 degrees from the one the points assume; the multiplying
discs drawn with the alpha blend): the gate must then FAIL, and each did when this was written.
THE EYE (2026-11-23): tools/eye_check.mjs runs first and must pass: each backend's eyeFrom against
a plain inverse over 200 orthographic and 200 perspective matrices (see the CORRECTION in INTERFACE.md §16).
Exit status 0 when all of that holds; the pixel numbers are REPORTED, not gated (the two
backends rasterise on different paths, see the README).

WHAT THIS DOES NOT MEASURE. Chrome is headless with SwiftShader for both WebGL (ANGLE) and
WebGPU (Dawn on SwiftShader Vulkan): every GPU operation runs on the CPU. Pixel agreement
here says the WGSL ports compute what the GLSL does on one software rasteriser; it says
nothing about a hardware GPU. Frames per second here are SwiftShader's and say nothing about
any device's frame time; they compare the two pages under the same conditions only.

No network beyond 127.0.0.1. Needs Python Playwright and /usr/bin/google-chrome.
"""

import argparse
import base64
import functools
import http.server
import json
import os
import re
import sys
import threading

from playwright.sync_api import sync_playwright

HERE = os.path.dirname(os.path.abspath(__file__))
WEB = os.path.abspath(os.path.join(HERE, "..", "web"))
CHROME = "/usr/bin/google-chrome"
# What runs WebGPU headless on this Linux box: Dawn's SwiftShader Vulkan adapter. Without
# --use-vulkan=swiftshader requestAdapter() returned null in most launches here; with
# --disable-vulkan-surface the adapter works but the canvas is not composited, so a screenshot
# is blank. These flags gave an adapter and a composited canvas in every launch tried.
CHROME_ARGS = [
    "--enable-unsafe-webgpu",
    "--enable-features=Vulkan",
    "--use-vulkan=swiftshader",
    "--use-webgpu-adapter=swiftshader",
    "--use-angle=swiftshader",
    "--enable-unsafe-swiftshader",
]
MODES = {"default": "", "map": "map=1", "noshadow": "shadow=0", "images": "images=1",
         "resident": "resident=1", "residentmap": "map=1&resident=1", "residentimages": "images=1&resident=1",
         "icons": "icons=1", "iconsmove": "icons=1&phase=0.75",
         "tiles": "tiles=1", "tiles3d": "tiles=2", "add": "add=1", "globe": "globe=1"}
TILES_MODES = ("tiles", "tiles3d")
GL_VARIANTS = ("gl=1", "mediump=1", "gl=1&mediump=1", "nohigh=1", "gl=1&novao=1")
ADD_TOL = 2
GLOBE_TOL = 3   # the globe's varyings are interpolated over 3-degree triangles; the points assume the true sphere
PLANTS = ("tri", "globe", "mul")
RESIDENT_BASE = {"resident": "default", "residentmap": "map", "residentimages": "images"}
MESH_FUNCS = ["sphereMesh", "tubeMesh", "capsuleMesh", "arcCapsuleMesh", "arcRibbonMesh", "triMesh",
              "ribbonMesh", "boxMesh", "quadMesh", "discMesh", "polyMesh", "prismGableXMesh",
              "prismGableYMesh", "prismHippedMesh", "prismShedMesh", "prismPyramidMesh", "prismCourtyardMesh"]

COMPARE_JS = """
async ([a, b]) => {
  const load = async (b64) => {
    const blob = await (await fetch('data:image/png;base64,' + b64)).blob();
    const bmp = await createImageBitmap(blob, { colorSpaceConversion: 'none', premultiplyAlpha: 'none' });
    const c = new OffscreenCanvas(bmp.width, bmp.height);
    const x = c.getContext('2d');
    x.drawImage(bmp, 0, 0);
    return x.getImageData(0, 0, bmp.width, bmp.height);
  };
  const A = await load(a), B = await load(b);
  if (A.width !== B.width || A.height !== B.height) return { error: 'sizes differ' };
  const n = A.width * A.height, da = A.data, db = B.data;
  const out = new ImageData(A.width, A.height);
  let sum = 0, o8 = 0, o32 = 0, mx = 0;
  for (let i = 0; i < n; i++) {
    let m = 0;
    for (let c = 0; c < 3; c++) {
      const d = Math.abs(da[4 * i + c] - db[4 * i + c]);
      sum += d; if (d > m) m = d;
      out.data[4 * i + c] = Math.min(255, d * 4);
    }
    out.data[4 * i + 3] = 255;
    if (m > 8) o8++;
    if (m > 32) o32++;
    if (m > mx) mx = m;
  }
  const c = new OffscreenCanvas(A.width, A.height);
  c.getContext('2d').putImageData(out, 0, 0);
  const png = await c.convertToBlob({ type: 'image/png' });
  const buf = new Uint8Array(await png.arrayBuffer());
  let s = '';
  for (let i = 0; i < buf.length; i += 0x8000) s += String.fromCharCode.apply(null, buf.subarray(i, i + 0x8000));
  return { width: A.width, height: A.height, mean_abs: +(sum / (3 * n)).toFixed(4),
           over_8: +(100 * o8 / n).toFixed(4), over_32: +(100 * o32 / n).toFixed(4), max: mx, diff_png: btoa(s) };
}
"""

PIXEL_JS = """
async ([b64, x, y]) => {
  const bmp = await createImageBitmap(await (await fetch('data:image/png;base64,' + b64)).blob(), { colorSpaceConversion: 'none', premultiplyAlpha: 'none' });
  const c = new OffscreenCanvas(bmp.width, bmp.height);
  const g = c.getContext('2d');
  g.drawImage(bmp, 0, 0);
  return Array.from(g.getImageData(x, y, 1, 1).data);
}
"""
# ?icons=1: places where the 400-unit ground patch under the ring must show (green, not the
# clear colour), at 960x600: between the panels, inside the ring, and near its rim.
GROUND_PX = [(480, 180), (480, 250), (300, 330)]

PIXELS_JS = """
async ([b64, pts]) => {
  const bmp = await createImageBitmap(await (await fetch('data:image/png;base64,' + b64)).blob(), { colorSpaceConversion: 'none', premultiplyAlpha: 'none' });
  const c = new OffscreenCanvas(bmp.width, bmp.height);
  const g = c.getContext('2d');
  g.drawImage(bmp, 0, 0);
  return pts.map(([x, y]) => Array.from(g.getImageData(x, y, 1, 1).data));
}
"""

FPS_JS = """
(ms) => new Promise((done) => {
  let n = 0; const t0 = performance.now();
  const f = () => { n++; const t = performance.now() - t0; if (t < ms) requestAnimationFrame(f); else done(n * 1000 / t); };
  requestAnimationFrame(f);
})
"""


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass


def serve():
    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Quiet, directory=WEB))
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv


def mesh_drift():
    """Names of the mesh builders whose source differs between draw.js and draw-webgpu.js."""
    def funcs(path):
        src = open(path).read()
        out = {}
        for name in MESH_FUNCS:
            m = re.search(r"^function " + name + r"\(.*?^}\n", src, re.S | re.M)
            out[name] = m.group(0) if m else None
        return out
    a, b = funcs(os.path.join(WEB, "draw.js")), funcs(os.path.join(WEB, "draw-webgpu.js"))
    return [n for n in MESH_FUNCS if a[n] is None or a[n] != b[n]]


def load(browser, url, w, h, shot):
    page = browser.new_page(viewport={"width": w, "height": h}, device_scale_factor=1)
    errs = []
    page.on("pageerror", lambda e: errs.append(str(e)))
    page.goto(url)
    try:
        page.wait_for_function("document.body.dataset.ready === '1' || !!document.body.dataset.error", timeout=90000)
    except Exception as e:  # noqa: BLE001
        errs.append("not ready: " + str(e).splitlines()[0])
    info = page.evaluate("({ error: document.body.dataset.error || null, verdict: document.body.dataset.verdict || null,"
                         "  stats: document.body.dataset.stats ? JSON.parse(document.body.dataset.stats) : null,"
                         "  resident: document.body.dataset.resident ? JSON.parse(document.body.dataset.resident) : null,"
                         "  flow: document.body.dataset.flow ? JSON.parse(document.body.dataset.flow) : null,"
                         "  add: document.body.dataset.add ? JSON.parse(document.body.dataset.add) : null,"
                         "  globe: document.body.dataset.globe ? JSON.parse(document.body.dataset.globe) : null,"
                         "  refusals: document.body.dataset.refusals ? JSON.parse(document.body.dataset.refusals) : null })")
    png = page.locator("#c").screenshot(type="png") if shot else None
    return page, info, errs, png


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--out", default=os.path.join(os.environ.get("TMPDIR", "/tmp"), "holon-draw-webgpu-parity"))
    ap.add_argument("--width", type=int, default=960)
    ap.add_argument("--height", type=int, default=600)
    ap.add_argument("--modes", default=",".join(MODES))
    ap.add_argument("--fps", type=float, default=0, help="seconds of ?spin=1 per page to count frames (0: skip)")
    ap.add_argument("--plant", default="", choices=("",) + PLANTS, help="a fault planted in the globe mode's scene (?plant=); the gate must then FAIL")
    args = ap.parse_args()
    os.makedirs(args.out, exist_ok=True)

    ok = True
    report = {"chrome_args": CHROME_ARGS, "size": [args.width, args.height], "results": [], "fps": []}
    drift = mesh_drift()
    report["mesh_drift"] = drift
    if drift:
        ok = False
        print("FAIL mesh builders differ between draw.js and draw-webgpu.js:", ", ".join(drift))
    else:
        print("ok   mesh builders in draw-webgpu.js are draw.js's, source for source")

    # the eye each backend derives from vp, against a plain inverse (tools/eye_check.mjs; the CORRECTION of 2026-11-23)
    import subprocess
    ec = subprocess.run(["node", os.path.join(HERE, "eye_check.mjs")], capture_output=True, text=True)
    report["eye_check"] = ec.stdout.strip().splitlines()
    print(("ok   " if ec.returncode == 0 else "FAIL ") + "eye from vp (tools/eye_check.mjs): " + " | ".join(report["eye_check"]))
    if ec.returncode != 0:
        ok = False

    srv = serve()
    base = "http://127.0.0.1:%d/" % srv.server_address[1]
    with sync_playwright() as pw:
        browser = pw.chromium.launch(executable_path=CHROME, headless=True, args=CHROME_ARGS)
        report["chrome"] = browser.version
        print("chrome %s, %dx%d at dpr 1" % (browser.version, args.width, args.height))
        print("%-10s %-8s %10s %9s %9s %5s  %s" % ("mode", "overlay", "mean_abs", "over_8%", "over_32%", "max", "draw calls gl/gpu"))
        cache = {}

        def shot(name, qs, tag):
            """(png as base64, info) of a page, loaded once per (name, query)."""
            nonlocal ok
            key = (name, qs)
            if key not in cache:
                page, info, errs, png = load(browser, base + name + ("?" + qs if qs else ""), args.width, args.height, True)
                page.close()
                open(os.path.join(args.out, tag + ".png"), "wb").write(png)
                if errs or info["error"] or (name == "test-webgpu.html" and info["verdict"] != "PASS"):
                    ok = False
                    print("FAIL %s?%s: %s" % (name, qs, errs or info["error"] or info["verdict"]))
                cache[key] = (base64.b64encode(png).decode(), info)
            return cache[key]

        def compare(a, b):
            cmp_page = browser.new_page()
            cmp_page.goto(base + "test-scene.js")   # any same-origin page; the comparison is self-contained
            r = cmp_page.evaluate(COMPARE_JS, [a, b])
            cmp_page.close()
            return r

        def resident_gate(label, res, bas, unshadowed=None):
            """Gate a resident page (png, info) against its base on the same backend; one report row."""
            nonlocal ok
            r = compare(res[0], bas[0])
            r.pop("diff_png", None)
            rs, bs, log = res[1]["stats"] or {}, bas[1]["stats"] or {}, res[1]["resident"] or {}
            want = (bs.get("bytesUploaded") or 0) - (log.get("worldBytes") or 0) - (log.get("imageBytes") or 0)
            row = dict(label=label, max=r.get("max"), mean_abs=r.get("mean_abs"), bytes_base=bs.get("bytesUploaded"),
                       bytes_resident=rs.get("bytesUploaded"), bytes_expected=want, world_bytes=log.get("worldBytes"),
                       image_bytes=log.get("imageBytes"), resident_buffers=rs.get("residentBuffers"),
                       resident_bytes=rs.get("residentBytes"), checks_ok=log.get("ok"),
                       draw_calls=[rs.get("drawCalls"), bs.get("drawCalls")], instances=[rs.get("instances"), bs.get("instances")],
                       missing_images=[rs.get("missingImages"), bs.get("missingImages")])
            bad = []
            if r.get("max") != 0: bad.append("pixels differ (max %s, mean_abs %s)" % (r.get("max"), r.get("mean_abs")))
            if rs.get("bytesUploaded") != want: bad.append("bytesUploaded %s, want %s" % (rs.get("bytesUploaded"), want))
            if not log.get("ok"): bad.append("the page's resident checks failed: %s" % json.dumps(log))
            if rs.get("drawCalls") != bs.get("drawCalls") or rs.get("instances") != bs.get("instances"): bad.append("draw calls/instances differ")
            if rs.get("missingImages") != bs.get("missingImages"): bad.append("missingImages differ")
            if unshadowed is not None:
                # the resident picture against the same scene with no shadow map: it must differ,
                # or "identical to the shadowed bytes page" would not show that resident batches cast
                u = compare(res[0], unshadowed[0])
                row["vs_noshadow"] = {"max": u.get("max"), "over_8": u.get("over_8")}
                if not (u.get("max") or 0) > 0: bad.append("no different from the page without shadows")
                print("     resident vs ?shadow=0 %-34s max %s, over_8 %s%% (the shadows it casts)" % (label, u.get("max"), u.get("over_8")))
            if bad:
                ok = False
                print("FAIL resident %s: %s" % (label, "; ".join(bad)))
            print("     resident vs bytes %-34s max %s  bytesUploaded %s (bytes page %s, made resident %s)  resident %s buffers %s B" % (
                label, r.get("max"), rs.get("bytesUploaded"), bs.get("bytesUploaded"),
                (log.get("worldBytes") or 0) + (log.get("imageBytes") or 0), rs.get("residentBuffers"), rs.get("residentBytes")))
            report.setdefault("resident", []).append(row)

        def pixel(png, x, y):
            pg = browser.new_page()
            pg.goto(base + "test-scene.js")
            v = pg.evaluate(PIXEL_JS, [png, x, y])
            pg.close()
            return v

        def icons_gate(mode, overlay, qs, pngs, infos):
            """?icons=1: the resident flow link's draws uploaded nothing while its phase moved; the huge
            patch under the ring is drawn; at another phase the dashes moved (iconsmove against icons,
            same backend); WebGL 1 and the mediump paths draw it with no error (test.html only)."""
            nonlocal ok
            tag = "scene" if overlay == "0" else "panels"
            bad = []
            for k, be in enumerate(("webgl", "webgpu")):
                fl = infos[k].get("flow") or {}
                if not fl.get("ok"):
                    bad.append("%s: the resident flow link's check failed: %s" % (be, json.dumps(fl)))
                for (x, y) in GROUND_PX:
                    r, g, b, _ = pixel(pngs[k], x, y)
                    if not (g > r + 12 and g > b + 4):
                        bad.append("%s: the ground under the ring is not drawn at (%d, %d): rgb %d %d %d" % (be, x, y, r, g, b))
                if mode == "iconsmove":
                    bqs = "&".join(x for x in (MODES["icons"], "overlay=0" if overlay == "0" else "") if x)
                    still = shot("test.html" if k == 0 else "test-webgpu.html", bqs, "icons_%s_%s" % (tag, be))
                    r = compare(pngs[k], still[0])
                    r.pop("diff_png", None)
                    print("     flow phase 0.75 vs 0.25 %-6s %-6s max %s, over_8 %s%% (the dashes moved)" % (tag, be, r.get("max"), r.get("over_8")))
                    report.setdefault("icons", []).append(dict(check="phase moved", tag=tag, backend=be, max=r.get("max"), over_8=r.get("over_8")))
                    if not (r.get("max") or 0) > 0:
                        bad.append("%s: the picture did not change with the phase" % be)
            if mode == "icons" and overlay == "1":
                for extra in ("gl=1", "mediump=1", "gl=1&mediump=1", "nohigh=1", "gl=1&novao=1"):
                    png, info = shot("test.html", qs + "&" + extra, "icons_panels_webgl_" + extra.replace("&", "_").replace("=", ""))
                    st = info["stats"] or {}
                    r = compare(png, pngs[0])
                    r.pop("diff_png", None)
                    fl = info.get("flow") or {}
                    print("     icons test.html?%-16s glError %s, %s, screen %s; vs WebGL 2 highp: max %s, over_8 %s%%" % (
                        extra, st.get("glError"), st.get("version"), st.get("screenPrecision"), r.get("max"), r.get("over_8")))
                    report.setdefault("icons", []).append(dict(check="variant", query=extra, glError=st.get("glError"), version=st.get("version"),
                                                               screenPrecision=st.get("screenPrecision"), fragHighp=st.get("fragHighp"), max=r.get("max"), over_8=r.get("over_8")))
                    if st.get("glError") != 0 or not fl.get("ok"):
                        bad.append("test.html?%s: glError %s, flow %s" % (extra, st.get("glError"), json.dumps(fl)))
                    if st.get("fragHighp"):
                        bad.append("test.html?%s: a fragment source still says highp outside the gate" % extra)
            if bad:
                ok = False
                for x in bad:
                    print("FAIL icons %s %s: %s" % (mode, tag, x))

        def pixels(png, pts):
            pg = browser.new_page()
            pg.goto(base + "test-scene.js")
            v = pg.evaluate(PIXELS_JS, [png, pts])
            pg.close()
            return v

        def same_as(label, png, info, ref, want_bytes=None):
            """A page that must draw ref's picture exactly (max 0) with the same draw calls and instances."""
            nonlocal ok
            r = compare(png, ref[0])
            r.pop("diff_png", None)
            st, rs = info["stats"] or {}, ref[1]["stats"] or {}
            bad = []
            if r.get("max") != 0: bad.append("pixels differ (max %s, mean_abs %s)" % (r.get("max"), r.get("mean_abs")))
            if st.get("drawCalls") != rs.get("drawCalls") or st.get("instances") != rs.get("instances"):
                bad.append("draw calls/instances %s/%s, want %s/%s" % (st.get("drawCalls"), st.get("instances"), rs.get("drawCalls"), rs.get("instances")))
            if want_bytes is not None and st.get("bytesUploaded") != want_bytes: bad.append("bytesUploaded %s, want %s" % (st.get("bytesUploaded"), want_bytes))
            if "glError" in st and st.get("glError") != 0: bad.append("glError %s" % st.get("glError"))
            if st.get("fragHighp"): bad.append("a fragment source still says highp outside the gate")
            print("     %-58s max %s  draw calls %s (in %s draw() calls)  bytesUploaded %s" % (label, r.get("max"), st.get("drawCalls"), st.get("calls"), st.get("bytesUploaded")))
            report.setdefault("same", []).append(dict(label=label, max=r.get("max"), mean_abs=r.get("mean_abs"), drawCalls=st.get("drawCalls"),
                                                      calls=st.get("calls"), bytesUploaded=st.get("bytesUploaded"), glError=st.get("glError")))
            if bad:
                ok = False
                print("FAIL %s: %s" % (label, "; ".join(bad)))

        def tiles_gate(mode, overlay, qs, pngs, infos):
            """One draw() with a matrix per batch draws what one draw() per tile does, resident or not."""
            nonlocal ok
            tag = "scene" if overlay == "0" else "panels"
            for k, name in enumerate(("test.html", "test-webgpu.html")):
                be = "webgl" if k == 0 else "webgpu"
                ref = (pngs[k], infos[k])
                rf = infos[k].get("refusals") or {}
                print("     %-58s %s" % ("%s %s %s refusals" % (mode, tag, be), "ok: throws on a vp with shadows, a bad blend, a short vp, no vp at all; a batch's pixelsPerUnit with shadows draws" if rf.get("ok") else "FAILED"))
                report.setdefault("refusals", []).append(dict(mode=mode, tag=tag, backend=be, **rf))
                if not rf.get("ok"):
                    ok = False
                    print("FAIL %s %s %s refusals: %s" % (mode, tag, be, json.dumps(rf)))
                for extra in ("percall=1", "resident=1", "resident=1&percall=1"):
                    png, info = shot(name, qs + "&" + extra, "%s_%s_%s_%s" % (mode, tag, be, extra.replace("&", "_").replace("=", "")))
                    wb = None
                    if "resident" in extra:
                        wb = (infos[k]["stats"] or {}).get("bytesUploaded", 0) - ((info.get("resident") or {}).get("worldBytes") or 0)
                    same_as("%s %s %s ?%s vs one draw()" % (mode, tag, be, extra), png, info, ref, wb)
            if overlay == "1":
                for extra in GL_VARIANTS:
                    one = shot("test.html", qs + "&" + extra, "%s_%s_webgl_%s" % (mode, tag, extra.replace("&", "_").replace("=", "")))
                    per = shot("test.html", qs + "&" + extra + "&percall=1", "%s_%s_webgl_%s_percall" % (mode, tag, extra.replace("&", "_").replace("=", "")))
                    same_as("%s %s test.html?%s: per call vs one draw()" % (mode, tag, extra), per[0], per[1], one)
                    r = compare(one[0], pngs[0])
                    r.pop("diff_png", None)
                    print("     %-58s vs WebGL 2 highp: max %s, over_8 %s%%" % ("%s %s test.html?%s (%s)" % (mode, tag, extra, (one[1]["stats"] or {}).get("version")), r.get("max"), r.get("over_8")))

        def add_points(label, png, info, gl_state, key="add", tol=ADD_TOL):
            nonlocal ok
            a = info.get(key) or {}
            pts = a.get("points") or []
            got = pixels(png, [[p["x"], p["y"]] for p in pts])
            worst, bad = 0, []
            for p, g in zip(pts, got):
                d = max(abs(g[c] - p["want"][c]) for c in range(3))
                worst = max(worst, d)
                if d > tol: bad.append("%s at (%d, %d): rgb %s, want %s" % (p["what"], p["x"], p["y"], g[:3], p["want"]))
            if not pts: bad.append("the page listed no points")
            if gl_state and not a.get("ok"): bad.append("the blend state after draw() is not GL STATE's: %s" % json.dumps({k: a.get(k) for k in ("afterMain", "afterDepthA", "after")}))
            st = info["stats"] or {}
            if "glError" in st and st.get("glError") != 0: bad.append("glError %s" % st.get("glError"))
            print("     %-58s %d points, largest difference from the arithmetic %d (tolerance %d)%s" % (label, len(pts), worst, tol, "; blend state after draw() as promised" if gl_state and a.get("ok") else ""))
            report.setdefault(key, []).append(dict(label=label, points=len(pts), worst=worst, state=(a.get("afterMain") or a.get("after")) if gl_state else None, state_ok=a.get("ok") if gl_state else None))
            if bad:
                ok = False
                for x in bad: print("FAIL %s %s: %s" % (key, label, x))

        def add_gate(overlay, qs, pngs, infos):
            tag = "scene" if overlay == "0" else "panels"
            for k, name in enumerate(("test.html", "test-webgpu.html")):
                be = "webgl" if k == 0 else "webgpu"
                if overlay == "0":
                    add_points("add %s %s" % (tag, be), pngs[k], infos[k], k == 0)
                png, info = shot(name, qs + "&resident=1", "add_%s_%s_resident" % (tag, be))
                wb = (infos[k]["stats"] or {}).get("bytesUploaded", 0) - ((info.get("resident") or {}).get("worldBytes") or 0)
                same_as("add %s %s ?resident=1 vs bytes" % (tag, be), png, info, (pngs[k], infos[k]), wb)
            if overlay == "0":
                for extra in GL_VARIANTS:
                    png, info = shot("test.html", qs + "&" + extra, "add_%s_webgl_%s" % (tag, extra.replace("&", "_").replace("=", "")))
                    add_points("add %s test.html?%s (%s)" % (tag, extra, (info["stats"] or {}).get("version")), png, info, True)

        def globe_gate(overlay, qs, pngs, infos):
            """?globe=1: the TRIANGLE squares draw what the same squares as QUAD records draw (max 0), the
            globe and the multiplying discs give the header's arithmetic at every listed point, draw()
            refuses what GLOBE and BLEND say it refuses, resident draws what bytes draw, and on WebGL the
            blend state after draw() is GL STATE's; WebGL 1, mediump, nohigh and bare WebGL 1 pass the points."""
            nonlocal ok
            tag = "scene" if overlay == "0" else "panels"
            for k, name in enumerate(("test.html", "test-webgpu.html")):
                be = "webgl" if k == 0 else "webgpu"
                rf = infos[k].get("refusals") or {}
                print("     %-58s %s" % ("globe %s %s refusals" % (tag, be), "ok: throws on a globe with shadows, no sun, a zero sun, a short day, a bad blend; a multiply batch and a sun-only globe draw" if rf.get("ok") else "FAILED"))
                report.setdefault("refusals", []).append(dict(mode="globe", tag=tag, backend=be, **rf))
                if not rf.get("ok"):
                    ok = False
                    print("FAIL globe %s %s refusals: %s" % (tag, be, json.dumps(rf)))
                if overlay == "0":
                    add_points("globe %s %s" % (tag, be), pngs[k], infos[k], k == 0, "globe", GLOBE_TOL)
                if overlay == "0":
                    png, info = shot(name, qs + "&triquads=1", "globe_%s_%s_triquads" % (tag, be))
                    tri_quads("globe %s %s: the squares as QUAD records vs TRIANGLE" % (tag, be), png, info, (pngs[k], infos[k]))
                png, info = shot(name, qs + "&resident=1", "globe_%s_%s_resident" % (tag, be))
                wb = (infos[k]["stats"] or {}).get("bytesUploaded", 0) - ((info.get("resident") or {}).get("worldBytes") or 0)
                same_as("globe %s %s ?resident=1 vs bytes" % (tag, be), png, info, (pngs[k], infos[k]), wb)
            if overlay == "0":
                for extra in GL_VARIANTS:
                    png, info = shot("test.html", qs + "&" + extra, "globe_%s_webgl_%s" % (tag, extra.replace("&", "_").replace("=", "")))
                    add_points("globe %s test.html?%s (%s)" % (tag, extra, (info["stats"] or {}).get("version")), png, info, True, "globe", GLOBE_TOL)
                    tq = shot("test.html", qs + "&" + extra + "&triquads=1", "globe_%s_webgl_%s_triquads" % (tag, extra.replace("&", "_").replace("=", "")))
                    tri_quads("globe %s test.html?%s: QUAD vs TRIANGLE" % (tag, extra), tq[0], tq[1], (png, info))

        def tri_quads(label, png, info, ref):
            """The 18 squares as QUAD records (png) against the same as 36 TRIANGLE records (ref): the
            pictures identical (max 0), the draw calls the same, 18 instances fewer, gl error 0."""
            nonlocal ok
            r = compare(png, ref[0])
            r.pop("diff_png", None)
            st, rs = info["stats"] or {}, ref[1]["stats"] or {}
            bad = []
            if r.get("max") != 0: bad.append("pixels differ (max %s, mean_abs %s)" % (r.get("max"), r.get("mean_abs")))
            if st.get("drawCalls") != rs.get("drawCalls") or (rs.get("instances") or 0) - (st.get("instances") or 0) != 18:
                bad.append("draw calls %s vs %s, instances %s vs %s (want the same calls, 18 instances more as triangles)" % (st.get("drawCalls"), rs.get("drawCalls"), st.get("instances"), rs.get("instances")))
            if "glError" in st and st.get("glError") != 0: bad.append("glError %s" % st.get("glError"))
            print("     %-58s max %s  instances %s as quads, %s as triangles" % (label, r.get("max"), st.get("instances"), rs.get("instances")))
            report.setdefault("triquads", []).append(dict(label=label, max=r.get("max"), mean_abs=r.get("mean_abs")))
            if bad:
                ok = False
                print("FAIL %s: %s" % (label, "; ".join(bad)))

        for mode in args.modes.split(","):
            for overlay in ("0", "1"):
                qs = "&".join(x for x in (MODES[mode], "overlay=0" if overlay == "0" else "", ("plant=" + args.plant) if args.plant and mode == "globe" else "") if x)
                pngs, infos = [], []
                for name in ("test.html", "test-webgpu.html"):
                    tag = "%s_%s_%s" % (mode, "scene" if overlay == "0" else "panels", "webgl" if name == "test.html" else "webgpu")
                    png, info = shot(name, qs, tag)
                    pngs.append(png)
                    infos.append(info)
                r = compare(pngs[0], pngs[1])
                diff = r.pop("diff_png", None)
                if diff:
                    open(os.path.join(args.out, "%s_%s_diff.png" % (mode, "scene" if overlay == "0" else "panels")), "wb").write(base64.b64decode(diff))
                dc = [i["stats"]["drawCalls"] if i["stats"] else None for i in infos]
                inst = [i["stats"]["instances"] if i["stats"] else None for i in infos]
                if dc[0] != dc[1] or inst[0] != inst[1]:
                    ok = False
                    print("FAIL %s overlay=%s: draw calls %s, instances %s differ" % (mode, overlay, dc, inst))
                miss = [i["stats"].get("missingImages") if i["stats"] else None for i in infos]
                if miss[0] != miss[1] or (mode == "images" and overlay == "1" and miss[0] != 1):
                    ok = False
                    print("FAIL %s overlay=%s: images skipped for an empty slot %s (want 1 on both with the panels)" % (mode, overlay, miss))
                print("%-10s %-8s %10s %9s %9s %5s  %s/%s" % (mode, "on" if overlay == "1" else "off", r.get("mean_abs"), r.get("over_8"),
                                                          r.get("over_32"), r.get("max"), dc[0], dc[1]))
                report["results"].append(dict(mode=mode, overlay=overlay == "1", query=qs, **r,
                                              stats={"webgl": infos[0]["stats"], "webgpu": infos[1]["stats"]}))
                if mode in ("icons", "iconsmove"):
                    icons_gate(mode, overlay, qs, pngs, infos)
                if mode in TILES_MODES:
                    tiles_gate(mode, overlay, qs, pngs, infos)
                if mode == "add":
                    add_gate(overlay, qs, pngs, infos)
                if mode == "globe":
                    globe_gate(overlay, qs, pngs, infos)
                if mode in RESIDENT_BASE:
                    bm = RESIDENT_BASE[mode]
                    bqs = "&".join(x for x in (MODES[bm], "overlay=0" if overlay == "0" else "") if x)
                    for k, name in enumerate(("test.html", "test-webgpu.html")):
                        be = "webgl" if k == 0 else "webgpu"
                        bas = shot(name, bqs, "%s_%s_%s" % (bm, "scene" if overlay == "0" else "panels", be))
                        uns = None
                        if mode == "resident":
                            uqs = "&".join(x for x in (MODES["noshadow"], "overlay=0" if overlay == "0" else "") if x)
                            uns = shot(name, uqs, "noshadow_%s_%s" % ("scene" if overlay == "0" else "panels", be))
                        resident_gate("%s %s %s" % (mode, "panels" if overlay == "1" else "scene", be), (pngs[k], infos[k]), bas, uns)
                    if mode == "resident" and overlay == "1":
                        for extra in ("gl=1", "gl=1&novao=1"):
                            res = shot("test.html", "&".join((qs, extra)), "resident_panels_webgl1_" + extra.replace("&", "_").replace("=", ""))
                            bas = shot("test.html", "&".join(x for x in (bqs, extra) if x), "default_panels_webgl1_" + extra.replace("&", "_").replace("=", ""))
                            ver = (res[1]["stats"] or {}).get("version"), (res[1]["stats"] or {}).get("vertexArrays")
                            resident_gate("%s panels test.html?%s (%s, vao %s)" % (mode, extra, ver[0], ver[1]), res, bas)
        if args.fps > 0:
            for mode in args.modes.split(","):
                row = {"mode": mode}
                for name in ("test.html", "test-webgpu.html"):
                    qs = "&".join(x for x in (MODES[mode], "spin=1") if x)
                    page, info, errs, _ = load(browser, base + name + "?" + qs, args.width, args.height, False)
                    fps = page.evaluate(FPS_JS, int(args.fps * 1000)) if not errs and not info["error"] else None
                    page.close()
                    row["webgl" if name == "test.html" else "webgpu"] = round(fps, 2) if fps else None
                report["fps"].append(row)
                print("fps %-10s webgl %s  webgpu %s  (SwiftShader, ?spin=1, %g s)" % (mode, row["webgl"], row["webgpu"], args.fps))
        browser.close()
    srv.shutdown()
    with open(os.path.join(args.out, "parity.json"), "w") as f:
        json.dump(report, f, indent=1)
    print("pictures and parity.json in", args.out)
    print("PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
