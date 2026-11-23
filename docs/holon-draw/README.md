# docs/holon-draw: the engine the ported pages draw with

`draw.js` and `text.js` here are **byte copies** of holon-draw's browser backend and text
provider, taken from `engine/crates/holon-draw/web/` at commit **f206a4f** (2026-11-23):

| file | sha256 | source |
|---|---|---|
| draw.js | f2bc0c7ed460cfa21634ff10140ea1803ca9a617d65c961f82547ff86cde4bca | engine/crates/holon-draw/web/draw.js @ f206a4f |
| text.js | 9ec85f6db6f36613432d07a477eff59338ff52bca46ff2e84523381fc4dcda40 | engine/crates/holon-draw/web/text.js @ f206a4f (last changed 22b2a2e) |

They are vendored, not copied at deploy time (as docs/atoms-draw and docs/ladder are), so a
page that imports them draws with the engine it was checked against. To move them forward,
copy both files again, update this table, and rerun the checks.

`screen.js` is **not** engine code. It is the pages' own helper. It packs holon-draw screen
records (lines, discs, quads, text runs) in CSS pixels and hands them to `draw.js` in paint
order. Its header lists what it builds from those kinds that Canvas 2D had natively: dashes,
round caps, circle strokes, and radial gradients as stacked discs.

`checks/` holds one headless check per ported page (`check_<page>.py`, Python + Playwright,
real GPU Chrome). Each states its pass conditions in its header. It compares the page with
its Canvas-2D version at a named commit, renders both in the same seeded state, and saves a
before/after pair with a pixel-difference summary.
