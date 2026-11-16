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

**No text.** The draw layer draws no text and holds no strings. It returns anchors (S3) and
the host draws labels with its own already-localised strings and its own shaping. A label
is therefore never in a language the host did not choose, and right-to-left and complex
scripts are not this engine's work. (The one exception a host may ask for later is digits
on a `glyph`; it is not in this draft.)

**No state forms.** Loading, error, empty, refused and the other states of a read are the
host's to draw. The draw layer is given a scene only when its read is `populated`.

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

**Which draw primitives no UX primitive uses:** `box`, `prism`, `patch` with a field. They
exist for worlds (streets, terrain, atoms), where the consumer is a map or a workbench, not
a card. A world view shown inside a client is a GraphView whose rows are places.

**Which UX needs the draw set lacks:** text (deliberately), images (Media), and a curved
text path. None is proposed.

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
- Whether a draw layer in the client is wanted at all, and if so on which surface: the
  client reports a native surface per platform fits its architecture and a WebGL surface in
  a web view would be new. That is the lead's decision, with a size budget.
- Digits on a mark without host text.
