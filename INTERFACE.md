# The interface — what crosses a seam between this engine and a consumer

*Written 2026-11-15. DRAFT: sent to the client and constitution sessions for correction;
nothing below is built. It states (1) the draw layer's primitives, (2) the obligations a
consumer's laws put on a draw layer, (3) how the draw primitives compose to the 24 UX
primitives of the screen standard, and (4) the seam contract learned from the CEWP twin.
Sources: CIRISConstitution `manifests/ui_primitives.json`, CSD/5 draft 3 at `181c48a` (24
primitives, 17 states, 12 laws); the answers of those two sessions on 2026-11-15, as they
gave them; the twin's `FSD/twin/texture.md`. The doctrine behind it is `OBJECT.md`,
CORRECTION 2026-11-15: the consumer selects.*

## 0. The one rule

**A drawn scene is a view of a list.** Every mark a person can pick or act on is one row of a
list the consumer holds, under one stable tag. A mark with no row is DECORATION: it cannot be
picked, carries no value, and is drawn labelled as decoration where a person could mistake it
for data. This is the closed-versus-carried split at the screen: a mark is carried when a
consumer reads it. (The standard implies the first two sentences through L1 and GraphView's
`twin` slot; the labelling is this file's addition and binds this engine only.)

## 0a. Two ways a draw layer is hosted (revised the same day, before any reply to the draft)

The first draft assumed the client was the host. The client session then relayed that the
lead means `holon-draw` for CEWPOS, not the client (as relayed; not said to this session).
CEWPOS's own guide (`AGENTS.md`, read at `d6ba311`) says its scene IS the whole screen,
projected from a closed set of UX primitives (12 there; the standard now has 24). So there
are two profiles, and the engine is one core with a layer above it for the second.

- **Profile A, a canvas in a host** (the twin's map; a GraphView inside a client). The host
  draws text, the 17 state forms and every control. The draw layer draws marks. §1 to §3 as
  first written are this profile.
- **Profile B, the whole screen** (CEWPOS). There is no host toolkit: the draw layer draws
  all 24 primitives, so it needs text and a form for each of the 17 states. This adds two
  marks (D11, D12) and a UI layer (`holon-draw-ui`, §3a) that composes the 24 from marks. The
  core does not learn about cards; the UI layer is a client of it, as the map and the atoms
  are.

## 1. The draw primitives (`holon-draw`)

Ten marks, one container, four services. A mark is an INSTANCE: a fixed-size record, drawn
by instancing, never a per-vertex build in the host.

| # | mark | what it is | fields beyond the common ones |
|---|---|---|---|
| D1 | `quad` | a flat rectangle in a plane; optional corner radius | size (2), radius |
| D2 | `line` | an anti-aliased segment of stated width; a polyline is a run of them | end (3), width |
| D3 | `arc` | a great-circle or Bezier segment of stated width | control (3), end (3), width |
| D4 | `disc` | a filled or ringed circle facing the camera | radius, ring width |
| D5 | `box` | an axis box on a ground plane, rotated about the vertical | size (3), angle |
| D6 | `prism` | a footprint extruded to a height, with one of a closed list of tops | size (3), angle, top |
| D7 | `sphere` | a lit sphere | radius |
| D8 | `rod` | a lit cylinder between two points | end (3), radius |
| D9 | `patch` | a ground or surface patch with a mean colour and a procedural field | size (2), field id |
| D10 | `glyph` | one shape from a closed set (arrow, tick, cross, dot, ring, bar) | size, glyph id |

Common to every mark: `tag` (u32, 0 = decoration), position (3, in its tile's local frame),
colour (rgba), `kind` (which mark), `detail` (the coarsest level at which it is still
drawn), `pin` (never elided or coarsened).

- **C1 `tile`** — the container: a local origin (f64 pair outside the GPU, f32 inside), a
  level, a version, and instance buffers by mark. One upload and one draw call per tile and
  mark. A tile's contents are a function of its inputs and its seed; a seed touches nothing
  outside its tile.
- **S1 `camera`** — orthographic or perspective, as an input; orbit, map and globe are
  hosts' choices over one matrix.
- **S2 `pick(x, y) -> tag | none`** — pure, no side effect, exactly one tag (L1).
- **S3 `report`** — for each tagged mark: on screen or not, its screen box, and whether it
  is legible at the present level (the host needs this for L12 and for its labels).
- **S4 `allocate`** — the level-of-detail allocator: given a work budget, which tiles and
  marks are drawn fine. Behind one interface; the ring (distance) and the bits allocator
  (issue #4 §3a) are two implementations of it.

**Text is drawn, never shaped, by the core.** In profile A the draw layer draws no text: it
returns anchors (S3) and the host draws labels. In profile B two more marks exist:

| # | mark | what it is | fields |
|---|---|---|---|
| D11 | `run` | one already-shaped run of text: a strip of glyph quads from an atlas | atlas rectangle per glyph, advance, baseline, colour |
| D12 | `image` | a rectangle of a host-supplied texture (a QR grid, a media frame the host decoded) | texture id, source rectangle |

The core holds no strings, no fonts and no language. A `run` arrives from a TEXT PROVIDER
behind one trait: given a localised string, a style and a width, return shaped, positioned
glyphs and their coverage. On the web the provider is the platform (the browser shapes and
rasterises, so right-to-left and complex scripts are the platform's); on bare metal it is a
shaping library and a rasteriser compiled in. Which library, and its size, is an open
decision (§5). The laws' "only localised labels" holds in both: the core cannot invent a
word.

**State forms are the UI layer's, not the core's.** In profile A the host draws them. In
profile B `holon-draw-ui` holds one form per state, all 17, and a missing form is a build
error (the constitution session's reading of L3). The core is given marks either way.

## 2. What the screen standard's laws put on a draw layer

| law | binds the draw layer how | where it lands |
|---|---|---|
| L1 one owner | a pick returns exactly one tagged instance; a tag names one instance | `tag`, S2; a duplicate tag in a frame is refused whole |
| L2 exactly one state | no cross-fade leaves two forms legible at once | the draw layer draws no state forms; a host transition swaps scenes on one frame |
| L3 an honest zero | never substitutes one state's form for another; a missing form is a build error | not in GL: the host draws states (§1) |
| L4 an honest number | a value not sent gets no mark, or a mark of the "not sent" form; never zero, never a default size that reads as data | a mark's data fields are required; absent data is `glyph` "not sent" or no instance |
| L8 a function of its inputs | level of detail, animation phase and cache may change how finely, never what; no clock and no randomness in what is drawn; a layout seed is an input; reduced motion is an input | S4 may coarsen, may not add, drop or move a tagged mark's meaning; determinism gate |
| L10 media never bare | the draw layer has no media mark | Media stays the host's |
| L11 an answer is about its question | when the host drops an answer, its marks go on that frame; a retained tile is valid only for the version of the inputs it was built from | C1 `version`; a stale tile is not drawn |
| L12 shown is sent | an instance a write can be sent from is never elided | `pin`; S3 reports legibility so the host can refuse a send |
| L5, L6, L7, L9 | the card's; a pick never submits, confirms or navigates | S2 is pure |

## 3. The draw-to-client map: how the draw primitives compose to the 24

*Hosted* says whether a draw layer would actually draw it. The client's answer is that all
24 are drawn by its own toolkit today and nothing waits on this engine; the one plausible
host is GraphView's `canvas`. The other rows are vocabulary: what the composition WOULD be,
so the two sets of primitives can be checked against each other.

| UX primitive (prefix) | hosted | composition from draw primitives | what stays the host's |
|---|---|---|---|
| **GraphView** (`canvas`) | **yes** | nodes `disc`/`sphere`, edges `line`/`arc`/`rod`, groups `quad`/`patch`, in tiles; `pick` returns the row's tag | the twin (ItemRow rows), labels, the StateBlock around it |
| ItemRow (`row`) | as a mark | each row is one tagged mark in a canvas; the list form is the host's | the row itself, its `flag` and `act` |
| Metric (`stat`) | optional | a chart: `quad` bars, `line` series, `glyph` "not sent" | the number, unit and source as text; the em dash |
| Chip (`chip`) | as a mark | a status token on a mark: colour plus `glyph` (never colour alone) | the token's text |
| ScopePill (`scope`) | as a mark | a group boundary: `quad` or `patch` outline, tag 0 unless the group is a row | the pill |
| LineageChain (`chain`) | optional | links `line`/`arc` between tagged `disc`s; a cascade as a run | link text, order |
| PolicyLens (`lens`) | optional | a filter over marks: hidden marks are ABSENT, with a count returned (never dimmed to look present) | `hidden_count`, helpers |
| QrCode (`qr`) | no | would be a grid of `quad`s; exactness matters more than speed | all of it; scanning is the platform's |
| CodeCompare (`code`) | no | — | text |
| Transcript (`msg`, `note`) | no | — | text |
| Document (`doc`) | no | — | text; a recanted claim struck, never removed |
| Media (`media`) | no | — | all of it (L10) |
| CardShell (`card`) | no | a canvas sits inside one | the shell |
| StateBlock (`state`) | no | the draw layer is given a scene only in `populated` | all 17 forms |
| FieldRow (`txt`), Field (`input`), Choice (`opt`), DestinationPicker (`pick`) | no | — | all of it; where one overlays a canvas its anchor is `pin`ned (L12) |
| Action (`btn`, `mi`), ConfirmSheet (`sheet`), ReceiptSheet (`receipt`), Ceremony (`ceremony`), StepFlow (`step`), ConscienceStep (`conscience`) | no | — | all of it; a pick never acts |

### 3a. Profile B: the same 24 when the draw layer is the whole screen

Each primitive is a composition in `holon-draw-ui`; `run` is D11, and every row carries its
tag on the mark a person would touch.

| UX primitive | composition | notes |
|---|---|---|
| CardShell | `quad` (radius) + `run` title | a tile of its own: a card is a tile |
| FieldRow | `run` label + `run` value, or the fixed "not sent" form | never blank (its note) |
| ItemRow | `quad` + `run`s + `glyph` flag + Action | the twin of any canvas mark with the same tag |
| ScopePill, Chip | `quad` (radius) + `run`; Chip adds a `glyph` | never colour alone |
| Action | `quad` + `run`; disabled draws its reason `run` (L5) | `pin`ned (L12) |
| ReceiptSheet, ConfirmSheet | a tile above the scene: `quad` + fact `run`s + Actions | one open at a time; a sheet's `shows:` is `pin`ned |
| StateBlock | exactly one of 17 forms, swapped on one frame (L2) | each form is `glyph` + `run`; `loading_known` adds a `quad` bar |
| Metric | `run` number + `run` unit + `run` source, or an em dash `run` | a chart is optional (§3) |
| Choice, DestinationPicker | option rows: `quad` + `run` + `glyph` mark; nothing preselected | `pin`ned |
| Field | `quad` + `run` + caret `line` + ceiling `run` | text INPUT (keys, an input method, selection) is the platform's or a further layer: not in this draft |
| StepFlow | step `disc`s + `line`s + `run`s; a skipped step is absent | |
| Ceremony | `run` prices and window + signer rows + Action | numbers only from the node's answer |
| LineageChain | `disc`s + `arc`s + `run`s | |
| CodeCompare | large `run`s (words, digits) + verdict `glyph` | |
| QrCode | `image` (exact, nearest sampling) or a grid of `quad`s; scanning is the platform's camera | |
| Transcript, Document | `run`s in a scrolling tile; a recanted claim is a `run` with a `line` through it and its reason beside it | long text needs line breaking from the provider |
| ConscienceStep, PolicyLens | rows of `run` + `glyph`; hidden items absent with a count `run` | |
| GraphView | as §3 | its twin is ItemRows drawn by this same layer |
| Media | `image` with its alt `run` and disclosure `run`, or not drawn (L10) | decoding is the platform's |

What profile B costs beyond profile A, stated plainly: a text provider on each platform; line
breaking and scrolling; text input; focus order and a screen reader's tree built from the
tags. None of these is the 3D core, and each is its own piece of work.

**Which draw primitives no UX primitive uses:** `box`, `prism`, `patch` with a field. They
exist for worlds (streets, terrain, atoms), where the consumer is a map or a workbench, not
a card. A world view shown inside a client is a GraphView whose rows are places.

**Which UX needs the draw set lacks in profile A:** text and images, which are the host's.
In profile B they are D11 and D12. A curved text path is not proposed.

## 4. The seam contract (from the CEWP twin)

1. **A frozen record and side lists.** A consumer's record (the twin's 8 numbers per
   building) is never widened. A new attribute is a separate list at the same index, outside
   every digest of the record. `DATA_OBJECT.md`, CORRECTION 2026-11-15.
2. **Geometry is built on the engine's side of the seam.** The consumer hands layout
   buffers; the draw layer packs instances. No per-vertex or per-cell loop in the host per
   frame.
3. **A seed touches only what lies wholly within its tile.** A field shared by neighbours
   (ground, canopy) is a function of world position alone and agrees from both sides of
   every edge.
4. **Stable under zoom.** A component is drawn only where it spans at least 2 pixels and
   fades in continuously; nothing pops.
5. **Legible under data.** A procedural field modulates luminance by a bounded share and
   yields to a data map.
6. **The floor is the consumer's device.** WebGL 1 with instancing, medium precision in the
   fragment stage survivable, no long task on the main thread.
7. **Digests do not move.** A replacement draws the same instances: the gate is a digest of
   the instance buffers against the renderer it replaces, before a pixel test.
8. **No real feature crosses.** A table from this engine carries no coordinate, address or
   feature identifier; a cell index and class codes only.

## 5. Open, for the two sessions and the lead

- Whether "decoration is labelled" should be a law of the standard (a draft bump; an issue
  on the constitution's repository).
- CEWPOS as the host of profile B: its targets (a browser canvas today, bare metal later),
  its size budget, and whether its 12 primitives are to be re-based on the standard's 24.
  These are the lead's and the CEWPOS side's; nobody from that side has been asked yet.
- The text provider off the web: which shaping library and rasteriser, and what they weigh.
- Text input, focus and the screen reader's tree in profile B.
- The first draft's line "Ten marks" in §1 now reads twelve in profile B.

## 6. CORRECTION 2026-11-15 — from the client session's check against the registry (the tables above kept legible; where a row above and a line here disagree, this section holds)

**Framing.** The client is a host in neither profile: it draws all 24 with its own toolkit
and gave requirements only. Profile A's host is the twin or any canvas host; profile B's is
CEWPOS. §3's lead-in ("the one plausible host is GraphView's canvas") is read as: of the 24,
GraphView's `canvas` is the one a draw layer would host under ANY toolkit.

**Tags.** A tag in the standard is a derived STRING (`prefix '_' surface '_' instance
['__' slot] ['.' key]`), built by the primitives layer and never by hand. The `tag` of a
mark here is a HANDLE (u32) into a table the consumer owns from handle to derived tag. This
engine never mints a tag. Keyed instances take the element id as `.key`, so two rows do not
collide. In profile B each composition emits its own primitive's prefix and no other: an
Action in an ItemRow is `btn_…`; `act` is the row's SLOT that holds it.

**Laws.**
- L9 binds the draw layer too: the allocator must not cull a tagged mark at every zoom and
  in every camera state. Stated as a property of S4: with budget enough, every mark of a
  live tile is drawn (tested in `holon-draw`); a consumer's zoom range must reach it.
- L3 is wider than state forms: anything drawn from a read is present only when that read
  was answered. C1's `version` therefore covers L3 as well as L11: a tile retained from an
  unanswered or failed read is not drawn.
- L12: "drawn" means LEGIBLE, not merely in a buffer. S3's `legible` is the test a host
  uses before a send.

**Rows that contradicted a note.**
- ScopePill is ONE pill naming the group. A group's boundary or outline on a canvas is not
  a ScopePill: it is decoration (handle 0) or a grouping GraphView owns.
- Choice: nothing preselected UNLESS the card names the default and its `cost`.
- Ceremony: numbers from the node's answer OR the governance policy manifest, never the card.
- QrCode, both profiles: a scan fills a Field and never submits.

**Slots §3a missed** (closed lists; every one is a `run`, a `glyph` or an Action unless
said): FieldRow `gloss`; StateBlock's root tag always present with the state's name as its
value, `freshness`, one block per read; ReceiptSheet `open`, `holders`, `notes`, `acts`
(keyed); Field `error`; StepFlow `next`, `back`; QrCode `paste`, prefixes `qr` and `scan`;
Transcript `receipt`, `fetch`; ConscienceStep `cost`, `faculty` (4 faculties); PolicyLens
`depth`; Document `revision`, `cite`; DestinationPicker `group`; Media `blur`, `class`;
Action's second prefix `mi` inside a menu container, which §3a did not compose. And the
four shell prefixes (`circle`, `tab`, `nav`, `screen`), which a whole-screen layer must draw:
each is `quad` + `run` (+ `glyph`), in a tile of its own that is `pin`ned.

**Text.** One provider behind one trait stands, with three changes to what crosses it.
(a) The unit is the PARAGRAPH with a base direction, not a run: bidirectional reordering is
paragraph-level (UAX #9), and the provider returns visual runs. (b) Layout direction is an
input to the UI LAYER, not to text alone: StepFlow's next and back, list alignment and the
arrows of D10 mirror, so `glyph` carries a mirror flag. (c) Line breaking is the
provider's, including dictionary breaking where words are not spaced (Thai, Burmese) and
per-character rules (Japanese, Chinese). Scripts the client ships that need real shaping,
as the client reports them: Arabic script, the Indic scripts, Thai, Burmese; Ethiopic needs
its syllabary in a font; the large fonts of Chinese, Japanese and Korean weigh on any
bare-metal size budget. Text input stays outside the core.

**From the twin's lead, the same day (as relayed).** The timing protocol is 390 px wide at
6× CPU throttle, cold, with a 200 ms longest-task budget on the dive (this file's §4 did not
state a number; an earlier message of mine said 4×, which was wrong). The witnesses a
replacement must run green are the twin's own (its story, layer, texture-switch, no-highp
and service-worker tests). And the twin was wound down today at the lead's request: whether
it switches to this engine is the lead's decision, and gating a switch is the twin's.

## 7. CORRECTION 2026-11-15 — from the twin's street-renderer owner (as relayed by the twin's lead; refs are the twin's, at its `5de796a`)

**§4.6 was wrong for the twin as it stands.** Medium precision is NOT survivable in its
street tier today: a uniform declared in both stages links only if its precision agrees,
and one does not, so a phone with no high precision in the fragment stage has no street
tier (the twin is fixing it). Rules this engine takes from that: no uniform is declared in
both stages unless its precision is pinned; and the no-high-precision test undefines the
`GL_FRAGMENT_PRECISION_HIGH` macro in the shader source, since the precision query alone
does not show the failure.

**§4.7, the digest.** The twin's digest is over the LAYOUT record against its native
simulation, not over vertices. The gate for a replacement is two digests: the layout's,
unchanged, and this engine's instance digest against what the twin's leaf builder emits.

**§4.4.** "Spans 2 pixels" is tested nowhere in the twin. What it has is an alpha per kind,
a radial fade at the edge of its ring, and a floor that keeps a street at least 1 pixel.
A replacement reproduces those three; the 2-pixel rule stays a rule of this file, untested.

**What the instance record of §1 loses, and so must carry** (it has room: `aux` and
`variant` are free on a box or prism, and a tile needs a slot it does not yet have):
- dwelling units per building (they choose the roof colour where there is no tone);
- the leaf's seed with the building's index (they place yards and garden trees), and the
  lot density that switches yards on;
- the canopy flag;
- RAW tone, not final colour: ground and roof colours are mixed per tile from terrain and
  night values. So a tile carries per-tile UNIFORMS (origin and scale, night, two terrain
  colours and a roof factor, a noise origin, a texture switch), and a mark's colour may be
  an index into them. **Owed in `holon-draw`: a uniform block on the tile, and a rule for
  which of a mark's fields a consumer may define.**
- streets are not buildings: segments with a width class and end kinds (cap or flush), and
  a casing pass. That is `line` with a `variant` for its ends, drawn twice.

**A street-tier-only first slice brings these with it:** the mosaic uses the same program;
there is one GL context whose land stencil and draw order are shared with the relief and
data-map passes, so a second canvas loses the clip and the order (this engine must draw
INTO a host's context between the host's passes, not own the canvas); the layout cache is
read by five other places in the twin; picking is not in the street tier at all (it is a 2D
overlay); cache keys and the service worker's stamp must name any new file.

**A worker:** the layout may move to one only if the worker's kernel holds the same
population chain and table versions as the main one; uploads, version checks, the story's
timing and picking stay on the main thread; a worker adds a frame of latency per leaf.

**Added to §4, the twin's numbers:** a hold drift of at most 3,000 ms; at least 0.98 of
street frames drawn with their neighbours; the street step's waits (60 s, 15 s); a longest
task of 200 ms on the dive and the pull-back, never widened; timing at 390 px, 6× throttle,
cold, 5 runs, each started under a 1-minute load of 2 with the load logged. And §4.8's
inverse: a generated feature never lands on real geography.

**The gate before any switch** is the twin's list, all asserted: its story, zoom (plain and
delayed), planet-arc, layer-zoom (3 forms), texture-switch and every page test including
the layout cross-check and the service worker's; its evidence crate's tests; the
no-high-precision boot with the macro undefined; and texture-off frames by eye with a
tolerance, since two loads already differ by leaf order.
