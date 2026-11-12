#!/usr/bin/env python3
"""SCOPE-0: a filmstrip of the cloud through the model's 13 stages, as one PNG.
   filmstrip.py SHAPES.json OUT.png [yaw pitch]      (needs Pillow; nothing is computed but a projection)"""
import json
import math
import sys
from PIL import Image, ImageDraw

COL = [(150, 150, 146), (98, 80, 214), (42, 120, 214), (27, 175, 122), (235, 104, 52), (237, 161, 0)]


def main():
    t = json.load(open(sys.argv[1]))
    yaw, pitch = (float(sys.argv[3]), float(sys.argv[4])) if len(sys.argv) > 4 else (0.6, 0.35)
    F, cols, K = 330, 7, len(t["stages"])
    rows = (K + 1 + cols - 1) // cols
    im = Image.new("RGB", (F * cols, F * rows + 4), (252, 252, 251))
    d = ImageDraw.Draw(im)
    cy, sy, cp, sp = math.cos(yaw), math.sin(yaw), math.cos(pitch), math.sin(pitch)
    sc = F / 5.4
    for k in range(K):
        ox, oy = (k % cols) * F, (k // cols) * F
        pts = []
        for n, m in enumerate(t["points"]):
            if m["pos"] < 13:
                continue
            x, y, z = t["xyz"][n][k]
            xr, zr = x * cy - z * sy, x * sy + z * cy
            pts.append((zr * cp + y * sp, ox + F / 2 + sc * xr, oy + F / 2 + 8 - sc * (y * cp - zr * sp), m["place"]))
        pts.sort()
        for _, px, py, pl in pts:
            if ox + 2 < px < ox + F - 2 and oy + 16 < py < oy + F - 2:
                d.ellipse([px - 2, py - 2, px + 2, py + 2], fill=COL[pl])
        name = t["stages"][k]
        d.text((ox + 6, oy + 4), "%d  %s   kept %d%%  by place %d%%  by char %d%%" % (k + 1, name, round(100 * t["share_of_spread_kept_by_the_projection"][k]),
               round(100 * t["separation"]["place_in_word_3d"][k]), round(100 * t["separation"]["which_character_3d"][k])), fill=(11, 11, 11))
        d.rectangle([ox, oy, ox + F - 1, oy + F - 1], outline=(220, 219, 213))
    ox, oy = (K % cols) * F, (K // cols) * F
    for i, n in enumerate(t["places"]):
        d.ellipse([ox + 14, oy + 30 + i * 22, ox + 24, oy + 40 + i * 22], fill=COL[i])
        d.text((ox + 32, oy + 29 + i * 22), n, fill=(11, 11, 11))
    d.text((ox + 14, oy + 180), "yaw %.2f pitch %.2f; opening marker omitted" % (yaw, pitch), fill=(82, 81, 78))
    im.save(sys.argv[2])
    print(sys.argv[2], im.size)


if __name__ == "__main__":
    main()
