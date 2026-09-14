# The Acuity Sandbox — a toy, and the wall around it

`index.html` is a water **game**. It is not an instrument, it is not a campaign, and nothing
on it may be cited. `wall.mjs` is what makes that true rather than merely stated: it fails the
build if this directory ever cites a record, if any record ever cites this directory, if the
page loses its banner, or if the invented layer stops being called invented. It has already
caught one breach — this file's own sibling, `app.js`, whose header cited two records in
comments and was corrected rather than exempted.

## What is real here and what is not

| layer | what it is |
|---|---|
| the **blobs** | invented. A cheap 2D cohesive kernel with no physical content, tuned to look wet. Its constants were chosen by eye and mean nothing. |
| the **molecules** in the ring | run by the same engine the instrument uses, under a real wall law, at the engine's own derived step (`dt = 1.0775` au, the O–H curve's own). Their positions come out of the engine every frame. |

Even the real half is **not water**: it is a few dozen molecules in a vacuum, with no Ewald
sum, no charge-transfer table and no thermostat, at whatever temperature the seeding left
them. It is real *physics of a real law*, which is a much smaller claim than *water*, and the
banner on the page says so above the fold.

## The point

One idea, made playable: **the cost is governed by where you look.** Everything outside the
ring is a blob and nearly free; everything inside is real and expensive. Drag the ring and the
bill follows it. Scroll to resize it.

## Two things the engine taught this toy, both of which it had wrong

1. **It shipped a still life and called it live.** With no pair curves in the bank the
   timescale derives no step, and `holon_step_frame` returns having advanced nothing:
   `holon_steps()` read `0` after two hundred calls while the page displayed "live". The
   engine was right and the page was lying. The toy now ships the curves, generates the H–H
   one in the browser as the engine's own split expects, and **refuses to display "live" until
   it has watched an atom move**.
2. **It seeded molecules inside each other.** Mapping blob pixels to bohr at the first guessed
   scale put neighbours `3.6` bohr apart — well inside the repulsive wall — and the engine did
   what it should: it blew them apart. The scale is now derived from the blob layer's own
   packing against water's oxygen–oxygen separation, and a guard nudges any pair still too
   close, because handing an overlap to a force law is not the law's fault.

## Running it

Serve the directory (the wasm needs HTTP, not `file://`) and open `index.html`:

```
python3 -m http.server 8731 --bind 127.0.0.1     # from docs/
```

`node wall.mjs` from this directory checks the wall. It runs in `ci-gates.sh`.
