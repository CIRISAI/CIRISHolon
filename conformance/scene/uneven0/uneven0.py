#!/usr/bin/env python3
"""UNEVEN-0: the transition table from a 1/8 degree cell's density class to its tiles' classes.

Prereg conformance/scene/UNEVEN0_PREREG.md. One input: GHS-POP R2023A, epoch 2020, WGS84,
30 arc-seconds. The pixel-to-cell rule, the areas and the class thresholds are the CEWP twin's.

usage: uneven0.py plants <tif>            synthetic plants and the assignment plant (header only)
       uneven0.py run <tif> <twin gd dir> <out dir>    gate, P-SHIFT, P-FLIP, tables, stakes
"""
import os, sys
for _v in ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"): os.environ.setdefault(_v, "1")
import gzip, json, math, tempfile
import numpy as np

R = 6371.0088
THR = np.array([1.0, 250.0, 1500.0, 6000.0, 12000.0])
NAMES = ["empty", "village", "suburban", "terraced", "apartment blocks", "dense fabric"]
NX1, NY1 = 2880, 1440
D8 = math.radians(1.0 / 8.0)

def klass(density):
    """0 empty (under 1 per km2), then the twin's five."""
    return np.searchsorted(THR, density, side="right")

def cell_area(row):
    """Exact band on the sphere for 1/8 degree cell row `row` (row 0 at the north)."""
    top = math.radians(90.0 - row / 8.0); bot = math.radians(90.0 - (row + 1) / 8.0)
    return R * R * D8 * (math.sin(top) - math.sin(bot))

def gini(x):
    """Population form, over the last axis."""
    s = np.sort(x, axis=-1); n = s.shape[-1]; tot = s.sum(-1)
    i = np.arange(1, n + 1)
    g = 2.0 * (s * i).sum(-1) / (n * np.where(tot > 0, tot, 1.0)) - (n + 1.0) / n
    return np.where(tot > 0, g, 0.0)

def shares(P, A):
    """P people and A area per tile, shape (cells, tiles). Returns the cell's people, and per
    tile class the share of tiles and the share of people."""
    cls = klass(P / A); people = P.sum(1); safe = np.where(people > 0, people, 1.0)
    t = np.stack([(cls == k).mean(1) for k in range(6)], 1)
    p = np.stack([(P * (cls == k)).sum(1) / safe for k in range(6)], 1)
    return people, t, p

def blocks(P, a_row, g, wc):
    """Tiles of g pixel rows by wc pixel columns, taken from the cell's north-west corner; a
    ragged last block is kept as a smaller tile with its own area."""
    nr, nc = P.shape[1], P.shape[2]
    re, ce = np.arange(0, nr, g), np.arange(0, nc, wc)
    Pb = np.add.reduceat(np.add.reduceat(P, re, axis=1), ce, axis=2)
    ar = np.add.reduceat(a_row, re)
    ncols = np.diff(np.append(ce, nc))
    Ab = (ar[:, None] * ncols[None, :]).ravel()
    return Pb.reshape(P.shape[0], -1), np.broadcast_to(Ab, (P.shape[0], Ab.size))

def readings(P, a_row, carea, lat_c):
    """Everything read on a batch of cells. P: (cells, pixel rows, pixel cols) people."""
    out = {}
    n = P.shape[0]
    A1 = np.broadcast_to(np.repeat(a_row, P.shape[2]), (n, P.shape[1] * P.shape[2]))
    people, t, p = shares(P.reshape(n, -1), A1)
    out["people"], out["tiles1"], out["people1"] = people, t, p
    out["class"] = klass(people / carea)          # 0 means under the floor: the cell is not read
    out["gini"] = gini(P.reshape(n, -1))
    for g in (2, 4):
        wc = int(min(P.shape[2], max(1, round(g / max(math.cos(math.radians(lat_c)), 1e-6)))))
        Pb, Ab = blocks(P, a_row, g, wc)
        _, t, p = shares(Pb, Ab)
        out[f"tiles{g}"], out[f"people{g}"] = t, p
    return out

class Grid:
    def __init__(self, x0, y0, dx, dy, w, h):
        self.x0, self.y0, self.dx, self.dy, self.w, self.h = x0, y0, dx, dy, w, h
        j, i = np.arange(w), np.arange(h)
        self.lon = x0 + (j + 0.5) * dx; self.lat = y0 - (i + 0.5) * dy
        self.col = np.floor((self.lon + 180.0) * 8).astype(np.int64) % NX1
        self.row = np.clip(np.floor((90.0 - self.lat) * 8).astype(np.int64), 0, NY1 - 1)
        self.parea = R * R * math.radians(dy) * math.radians(dx) * np.cos(np.radians(self.lat))

def weighted_median(x, w):
    o = np.argsort(x); c = np.cumsum(w[o]); return float(x[o][np.searchsorted(c, 0.5 * c[-1])])

# ---------------------------------------------------------------- plants on synthetic rasters
def synth(top_lat, fill):
    """One 15 x 15 cell whose top edge is at top_lat. fill(a_row) returns the people array."""
    lat = top_lat - (np.arange(15) + 0.5) / 120.0
    a_row = R * R * math.radians(1 / 120) ** 2 * np.cos(np.radians(lat))
    row = int(round((90.0 - top_lat) * 8))
    P = fill(a_row)
    return readings(P[None], a_row, cell_area(row), top_lat - 1 / 16), P.sum() / cell_area(row), a_row

def plants(tif):
    ok = True
    def say(name, text, fired):
        nonlocal ok; ok &= bool(fired); print(f"{name} {text}  {'FIRES' if fired else 'FAILS'}", flush=True)
    r, d, a = synth(0.125, lambda a: np.outer(400.0 * a, np.ones(15)))
    say("P-FLAT ", f"density {d:.3f}, class {NAMES[r['class'][0]]}, own-class tile share {r['tiles1'][0][2]:.3f}, "
        f"gini {r['gini'][0]:.4f}, empty {r['tiles1'][0][0]:.3f}",
        NAMES[r["class"][0]] == "suburban" and abs(r["tiles1"][0][2] - 1) < 1e-9 and r["gini"][0] < 1e-3 and r["tiles1"][0][0] == 0)
    def town(a):
        P = np.zeros((15, 15)); P[6:9, 6:9] = (3000.0 * a[6:9])[:, None]; return P
    r, d, a = synth(0.125, town); c = r["class"][0]
    up = r["people1"][0][c + 1:].sum()
    say("P-TOWN ", f"cell density {d:.3f} ({NAMES[c]}); own-class tile share {r['tiles1'][0][c]:.3f}; empty {r['tiles1'][0][0]:.3f}; "
        f"own-class people {r['people1'][0][c]:.3f}; higher-class people {up:.3f}; gini {r['gini'][0]:.3f}",
        abs(d - 120.0) < 5e-4 and NAMES[c] == "village" and r["tiles1"][0][c] == 0 and abs(r["tiles1"][0][0] - 0.960) < 5e-4
        and r["people1"][0][c] == 0 and abs(up - 1) < 1e-12 and abs(r["gini"][0] - 0.960) < 5e-4)
    def thousand(a):
        P = np.zeros((15, 15)); P[6:9, 6:9] = 1000.0; return P
    got = []
    for top in (0.125, 60.125):
        r, d, a = synth(top, thousand); k = int(np.argmax(r["people1"][0]))
        got.append((1000.0 / a[7], NAMES[k], d, NAMES[r["class"][0]]))
    say("P-LAT  ", f"pixel {got[0][0]:.1f} per km2 ({got[0][1]}) at 0.125 N, {got[1][0]:.1f} ({got[1][1]}) at 60.125 N; "
        f"cell {got[0][2]:.2f} and {got[1][2]:.2f} ({got[0][3]}, {got[1][3]})",
        abs(got[0][0] - 1164.6) < 0.1 and got[0][1] == "suburban" and abs(got[1][0] - 2333.7) < 0.1 and got[1][1] == "terraced"
        and abs(got[0][2] - 46.59) < 0.01 and abs(got[1][2] - 93.35) < 0.01 and got[0][3] == got[1][3] == "village")
    def nodata(a):
        P = np.outer(400.0 * a, np.ones(15)); P.ravel()[:10] = -200.0; return np.where(P > 0, P, 0.0)
    r, d, a = synth(0.125, nodata)
    say("P-NODATA", f"10 no-data pixels: empty tile share {r['tiles1'][0][0]:.4f} (10 of 225 = {10/225:.4f}), people {r['people'][0]:.1f}",
        abs(r["tiles1"][0][0] - 10 / 225) < 1e-12 and r["people"][0] > 0)
    import tifffile
    with tifffile.TiffFile(tif) as t:
        p = t.pages[0]; sx, sy, _ = p.tags["ModelPixelScaleTag"].value; tp = p.tags["ModelTiepointTag"].value
        G = Grid(tp[3], tp[4], sx, sy, p.shape[1], p.shape[0])
    print(f"         file geometry: {G.w} x {G.h} pixels, origin {G.x0:.7f} E {G.y0:.7f} N, pixel {G.dx:.10f} x {G.dy:.10f} degrees")
    cnt_c = np.bincount(G.col, minlength=NX1); cnt_r = np.bincount(G.row, minlength=NY1)
    off = ((G.lon[16] + 180.0) * 8) % 1.0 * 15
    say("P-ASSIGN", f"pixel columns 0, 15, 16, {G.w-1} land in cell columns {G.col[0]}, {G.col[15]}, {G.col[16]}, {G.col[-1]}; "
        f"pixel rows 0, {G.h-1} in cell rows {G.row[0]}, {G.row[-1]}; a pixel centre sits {off:.2f} pixel past its cell edge",
        (G.col[0], G.col[15], G.col[16], G.col[-1], G.row[0], G.row[-1]) == (2879, 0, 1, 0, 7, 1432))
    edge_c = sorted(set(cnt_c.tolist())); edge_r = sorted(set(cnt_r[cnt_r > 0].tolist()))
    say("P-EDGE ", f"pixel columns per cell column: {edge_c} (cells 0 and 2879 hold {cnt_c[0]} and {cnt_c[2879]}); "
        f"pixel rows per cell row: {edge_r} (rows 7 and 1432 hold {cnt_r[7]} and {cnt_r[1432]}); rows with none: {int((cnt_r == 0).sum())}",
        cnt_c[0] == 16 and cnt_c[2879] == 16 and set(cnt_c[1:2879].tolist()) == {15} and cnt_r[7] == 12 and cnt_r[1432] == 12)
    return ok

# ---------------------------------------------------------------- the run
def run(tif, gd, out):
    import tifffile
    os.makedirs(out, exist_ok=True)
    log = open(os.path.join(out, "uneven0.log"), "w")
    def say(*a):
        s = " ".join(str(x) for x in a); print(s, flush=True); log.write(s + "\n"); log.flush()
    say("# UNEVEN-0 run")
    if not plants(tif): say("BRANCH (e): a synthetic plant failed; the grid is not read"); return
    tempfile.tempdir = os.path.dirname(os.path.abspath(tif))
    with tifffile.TiffFile(tif) as t:
        p = t.pages[0]; sx, sy, _ = p.tags["ModelPixelScaleTag"].value; tp = p.tags["ModelTiepointTag"].value
        G = Grid(tp[3], tp[4], sx, sy, p.shape[1], p.shape[0])
        mm = p.asarray(out="memmap", maxworkers=2)
    shift_col = np.roll(G.col, -1)                 # P-SHIFT: every pixel assigned one column east
    flip_row = G.row[::-1]                         # P-FLIP: rows reversed
    g1 = np.zeros((NY1, NX1)); gs = np.zeros((NY1, NX1)); gf = np.zeros((NY1, NX1))
    cls = np.zeros((NY1, NX1), np.uint8); ppl = np.zeros((NY1, NX1)); gin = np.zeros((NY1, NX1), np.float32)
    T1 = np.zeros((NY1, NX1, 6), np.float32); P1 = np.zeros((NY1, NX1, 6), np.float32)
    T24 = {g: np.zeros((NY1, NX1, 6), np.float32) for g in (2, 4)}; P24 = {g: np.zeros((NY1, NX1, 6), np.float32) for g in (2, 4)}
    j1 = int(np.argmax(G.col == 1)); assert np.all(G.col[j1:j1 + 2878 * 15] == np.repeat(np.arange(1, 2879), 15))
    idx0 = np.nonzero(G.col == 0)[0]; idx9 = np.nonzero(G.col == 2879)[0]
    idx0 = idx0[np.argsort(((G.lon[idx0] + 180.0) % 360.0))]; idx9 = idx9[np.argsort(((G.lon[idx9] + 180.0) % 360.0))]
    bounds = np.nonzero(np.diff(G.row))[0] + 1; starts = np.append(0, bounds); ends = np.append(bounds, G.h)
    total_parts = []; nodata = neg = npix = 0; extra = [0.0, 0.0]
    for i0, i1 in zip(starts, ends):
        r = int(G.row[i0]); raw = np.asarray(mm[i0:i1, :], dtype=np.float64)
        nodata += int(np.count_nonzero(raw == -200)); neg += int(np.count_nonzero((raw < 0) & (raw != -200)))
        band = np.where(raw > 0, raw, 0.0); npix += band.size
        extra[0] += float(band[:, 0].sum()); extra[1] += float(band[:, -1].sum())
        for k in range(i1 - i0):
            g1[r] += np.bincount(G.col, weights=band[k], minlength=NX1)
            gs[r] += np.bincount(shift_col, weights=band[k], minlength=NX1)
            gf[int(flip_row[i0 + k])] += np.bincount(G.col, weights=band[k], minlength=NX1)
        total_parts.append(math.fsum(band.sum(1).tolist()))
        if not band.any(): continue
        a_row = G.parea[i0:i1]; ca = cell_area(r); lat_c = 90.0 - (r + 0.5) / 8.0
        for cols, sl in ((slice(1, 2879), band[:, j1:j1 + 2878 * 15].reshape(i1 - i0, 2878, 15).transpose(1, 0, 2)),
                         (slice(0, 1), band[:, idx0][None]), (slice(2879, 2880), band[:, idx9][None])):
            if not sl.any(): continue
            o = readings(np.ascontiguousarray(sl), a_row, ca, lat_c)
            cls[r, cols] = o["class"]; ppl[r, cols] = o["people"]; gin[r, cols] = o["gini"]
            T1[r, cols] = o["tiles1"]; P1[r, cols] = o["people1"]
            for g in (2, 4): T24[g][r, cols] = o[f"tiles{g}"]; P24[g][r, cols] = o[f"people{g}"]
    total = math.fsum(total_parts)
    del mm
    say(f"pixels read: {npix:,}; no-data pixels: {nodata:,}; negative pixels: {neg:,}; "
        f"people in pixel column 0: {extra[0]:,.2f}; in pixel column {G.w-1}: {extra[1]:,.2f}")
    # ---- G0
    def gz(name, dtype, shape):
        return np.frombuffer(gzip.open(os.path.join(gd, name)).read(), dtype=dtype).reshape(shape)
    pop0 = gz("pop0.bin.gz", "<f4", (360, 720)).astype(np.float64); pop1 = gz("pop1.bin.gz", np.uint8, (NY1, NX1))
    def half_ok(grid):
        h = grid.reshape(360, 4, 720, 4).sum((1, 3)); return int(np.count_nonzero(np.abs(h - pop0) > np.maximum(1e-6 * np.abs(pop0), 0.5)))
    def encode(grid):
        pos = grid >= 0.1; code = np.floor((np.log10(np.where(pos, grid, 1.0)) + 1.0) * 255 / 8.0).astype(np.int64) + 1
        k = np.zeros(grid.shape, np.uint8); k[pos] = np.clip(code[pos], 1, 255); return k
    a = abs(total - 7840952947.01); b = half_ok(g1); c = int(np.count_nonzero(encode(g1) != pop1))
    g0 = a <= 0.01 and b == 0 and c <= 100
    say(f"G0  (1) world total {total:,.2f}, off the publisher's figure by {a:.3f}; (2) half-degree totals differing from the twin's: "
        f"{b} of 259,200; (3) 1/8 degree codes differing from the twin's: {c} of {NX1*NY1:,}  {'MET' if g0 else 'FAILED'}")
    if not g0: say("BRANCH (e): the gate failed; nothing is read"); return
    bs = half_ok(gs); bf = half_ok(gf); tf = abs(float(gf.sum()) - float(g1.sum()))
    f1 = bs > 0; f2 = bf > 0 and tf < 1.0
    say(f"P-SHIFT every pixel one column east: {bs} half-degree totals differ  {'FIRES' if f1 else 'FAILS'}")
    say(f"P-FLIP  rows reversed: the world total moves by {tf:.3f} and {bf} half-degree totals differ  {'FIRES' if f2 else 'FAILS'}")
    if not (f1 and f2): say("BRANCH (e): a plant on the real grid failed; nothing is read"); return
    # ---- tables
    read_ = cls > 0
    say(f"cells read (at or above the floor): {int(read_.sum()):,} holding {float(ppl[read_].sum()):,.0f} people; "
        f"cells under the floor holding people: {int(((cls == 0) & (ppl > 0)).sum()):,} with {float(ppl[cls == 0].sum()):,.0f}")
    near = 0
    for r in range(NY1):
        d = ppl[r] / cell_area(r)
        near += int(np.count_nonzero(np.min(np.abs(d[:, None] / THR[None, :] - 1.0), axis=1) < 0.005))
    say(f"cells within 0.5 % of a class threshold: {near:,}")
    tables = {}
    for g, T, P in ((1, T1, P1), (2, T24[2], P24[2]), (4, T24[4], P24[4])):
        tables[g] = {}
        for c_ in range(1, 6):
            m = cls == c_; n = int(m.sum()); w = ppl[m]
            row = {"cells": n, "people": float(w.sum())}
            if n:
                row["tiles_each_cell_once"] = T[m].mean(0, dtype=np.float64).tolist(); row["people_each_cell_once"] = P[m].mean(0, dtype=np.float64).tolist()
                row["tiles_by_people"] = (T[m] * w[:, None]).sum(0).__truediv__(w.sum()).tolist()
                row["people_by_people"] = (P[m] * w[:, None]).sum(0).__truediv__(w.sum()).tolist()
            tables[g][NAMES[c_]] = row
    allt = T1[read_].mean(0)
    say("share of ALL tiles of read cells in each tile class (1 pixel grain): " + ", ".join(f"{NAMES[k]} {allt[k]:.3f}" for k in range(6)))
    for g in (1, 2, 4):
        say(f"--- transition table, grain {g} pixel{'s' if g > 1 else ''}; rows = cell class, columns = tile class {NAMES}")
        for c_ in range(1, 6):
            row = tables[g][NAMES[c_]]
            if not row["cells"]: say(f"  {NAMES[c_]:17s} cells 0"); continue
            flag = "" if row["cells"] >= 30 else "  (fewer than 30 cells: reported, not summarised further)"
            say(f"  {NAMES[c_]:17s} cells {row['cells']:>7,}{flag}")
            for key in ("tiles_each_cell_once", "people_each_cell_once", "tiles_by_people", "people_by_people"):
                say(f"      {key:22s} " + " ".join(f"{v:6.3f}" for v in row[key]))
    for c_ in range(1, 6):
        m = cls == c_
        if m.sum() >= 30:
            say(f"gini by class (1 pixel): {NAMES[c_]:17s} median {float(np.median(gin[m])):.3f} over {int(m.sum()):,} cells")
    # ---- stakes, suburban class, 1 pixel grain
    m = cls == 2; w = ppl[m]; one = np.ones(int(m.sum()))
    own_t = T1[m][:, 2].astype(np.float64); own_p = P1[m][:, 2].astype(np.float64); up_p = P1[m][:, 3:].sum(1).astype(np.float64)
    a1, a2 = weighted_median(own_t, one), weighted_median(own_t, w)
    s1 = a1 < 0.5 and a2 < 0.5; k1 = a1 > 0.8 or a2 > 0.8
    say(f"S1  suburban cells ({int(m.sum()):,}): median share of tiles in the cell's own class {a1:.3f} (each cell once), {a2:.3f} (by people)  "
        f"{'MET' if s1 else ('KILLED' if k1 else 'NOT MET')}")
    b1, b2 = weighted_median(own_p, w), weighted_median(up_p, w)
    s2 = b1 < 0.5 and b2 > 0.4; k2 = b1 > 0.8
    say(f"S2  suburban cells, by people: median share of people in own-class tiles {b1:.3f}; in higher-class tiles {b2:.3f}  "
        f"{'MET' if s2 else ('KILLED' if k2 else 'NOT MET')}")
    mv = cls == 1
    say(f"reported: village cells ({int(mv.sum()):,}): median own-class tile share {weighted_median(T1[mv][:,1].astype(np.float64), np.ones(int(mv.sum()))):.3f} "
        f"(each cell once), {weighted_median(T1[mv][:,1].astype(np.float64), ppl[mv]):.3f} (by people); median empty tile share "
        f"{float(np.median(T1[mv][:,0])):.3f}")
    say("BRANCH (a)" if s1 and s2 else ("BRANCH (k)" if (k1 or k2) else "BRANCH (b)"))
    json.dump({"credit": "GHS-POP R2023A, European Commission, Joint Research Centre; Schiavina, Freire, Carioli and MacManus (2023), "
               "doi:10.2905/2FF68A52-5B5B-4A22-8F40-C41DA8332CFE; CC BY 4.0.", "tile_classes": NAMES,
               "tables": {str(g): tables[g] for g in tables}}, open(os.path.join(out, "transition_tables.json"), "w"), indent=1)
    rr, cc = np.nonzero(read_)
    np.savez_compressed(os.path.join(out, "cells.npz"), row=rr.astype(np.int16), col=cc.astype(np.int16), klass=cls[read_],
                        people=ppl[read_].astype(np.float32), tiles=T1[read_].astype(np.float16), people_share=P1[read_].astype(np.float16),
                        gini=gin[read_].astype(np.float16))
    say(f"written: transition_tables.json and cells.npz ({len(rr):,} rows; no pixel value kept)")

if __name__ == "__main__":
    if sys.argv[1] == "plants": plants(sys.argv[2])
    elif sys.argv[1] == "run": run(sys.argv[2], sys.argv[3], sys.argv[4])
