# SCOUT: what places look like — buildings, trees, streets, climate — and what the twin's renderer can read

Scouted 2026-11-12, on the lead's word: "can we also bring in what buildings look like,
what trees look like, etc... by location? all the rest of the details of the holons that
the rendering engine needs". Two inputs: a list from the twin's lead of what its renderer
reads (from its code, relayed in a message; not re-read here), and a web scout by a
subagent of open datasets and their licences. No dataset is proposed for fetching except
by name in the last section, and none is approved.

Marks: unmarked = read on the publisher's page through a fetch tool's extraction (re-check
a quote before it enters a licence block); `[summary]` = a search result only; `[my
reading]` = inference; `[not read]`.

**Three overreaches by the scout, said plainly.** It read one live statistics response
(taginfo, Germany, `roof:shape`), one 5-row query of the DHS indicator service, and the
first 100 kB of one summary CSV on GitHub (GEM). These were data, not documentation. One
figure from the first is quoted below as illustration; nothing was saved to this
repository. No account, key or acceptance of terms was involved.

## 1. What the renderer reads today (the twin's lead, from its code)

The renderer is top-down and orthographic. That decides most of what follows.

- **Read per place now:** land-cover class per 1/8° cell (one global palette); household
  density, which sets a class (village, suburban, urban terraced, urban core, dense
  fabric) by global thresholds of 250, 1,500, 6,000 and 12,000 people per km²; elevation
  and relief; a "warmth" figure from latitude and elevation that tints roofs and ground.
- **Global by class, not by place:** street spacing and width; building width, depth,
  storeys, units, gap and setback; the share of barns; tree count and crown size (trees
  are flat green hexagons, none at 6,000 people per km² or over).
- **Could vary by place with little work:** the land-cover palette; class thresholds and
  persons per household; each per-class building and street number; the roof and ground
  tint bands; the tree count and size rule.
- **Cannot vary without new rendering work:** roof colour per building; tree colour; sun
  direction for shadows; any street pattern but an axis-aligned grid.
- **Not drawn at all:** roof form; wall material or colour; height in 3D; tree species or
  form; leaf season; street surface, sidewalks, poles and wires; snow; sky; parks;
  parking and cars; water at street scale. **Tables for these would be read by nothing.**
- **What matters most at street scale:** the density class decides almost everything
  (street spacing and width, footprint, gap, setback, tree count); then ground colour by
  land cover and the roof tint.
- **The tables it could read soonest, in the order they change the picture:**
  1. per density class and region or country: footprint width and depth, storeys, gap,
     setback, street spacing and width;
  2. tree canopy share and crown size by land-cover class and climate zone;
  3. roof tone by region or climate zone;
  4. a ground palette by land-cover class and climate zone.
- **The seam.** Its importer refuses any key named for a coordinate or a real feature and
  requires `holds_no_real_feature`. It can key on country, one of 9 regions, density
  class and land-cover class; it has no climate-zone grid; and tables that change
  geometry need a door into its kernel that does not exist yet.

## 2. Sources, by what they would support

### 2.1 The binning keys: climate zone and biome

- **Köppen-Geiger maps, Beck et al. 2023 (V3).** https://www.gloh2o.org/koppen/ ; file
  `koppen_geiger_tif.zip`, 90 MB `[summary]`, 1991 to 2020 at 1 km with coarser grids
  `[my reading]`. Page: "Freely use, adapt, and share these maps under CC BY 4.0, with
  attribution to Beck et al. (2023)." The scout reports the page's licence link points
  at the NonCommercial deed; the record's own licence field was `[not read]`. To settle
  before any fetch.
- **RESOLVE Ecoregions 2017.** https://ecoregions.appspot.com/ ; `Ecoregions2017.zip`,
  about 150 MB, 846 ecoregions in 14 biomes `[summary]`. "Licensed under CC-BY 4.0".

### 2.2 Buildings: height, storeys, form

- **GHSL, European Commission JRC.** Average building height per 100 m cell (GHS-BUILT-H,
  2018), settlement morphology at 10 m (GHS-BUILT-C), and a table of about 11,000 urban
  centres with 473 indicators (GHS-UCDB R2024A, 1,690 MB). "The GHSL data are provided
  under the Creative Commons Attribution 4.0 International (CC BY 4.0) license." The
  indicator list of the urban-centre table was `[not read]`; raster sizes `[not read]`.
  Heights are modelled averages per cell, not per building.
- **Global map of Local Climate Zones, Demuzere et al.** https://zenodo.org/records/8419340 ;
  `lcz_filter_v3.tif`, 1.4 GB, 100 m, 17 classes of which 10 are built (compact or open,
  high, mid or low rise, lightweight). CC BY 4.0 on the record. The nearest open stand-in
  for attached against detached and for spacing `[my reading]`. A classifier's output.
- **EUBUCCO** (Europe, 322 million buildings; height, floors and type filled, construction
  year 15.9 % filled). ODbL, with two regions under other terms. Sizes `[not read]`.
- **Footprints.** Google Open Buildings v3 (CC BY 4.0 or ODbL at the user's choice; 178 GB;
  Africa, South and South-East Asia, Latin America). Microsoft footprints (CDLA Permissive
  2.0 on its own page; Overture words it as ODbL; unresolved). Overture buildings (ODbL;
  no aggregate; attributes of roof and material come from OpenStreetMap tags). None has a
  pre-aggregated table; each is tens to hundreds of GB.
- **Excluded by licence:** GlobalBuildingAtlas (NonCommercial, terms to accept); the GEM
  exposure model (NonCommercial ShareAlike); the UN housing compilation (no derivative
  works); TABULA (no formal open licence; one would have to ask).

### 2.3 Roof form, materials, colours, street surface: OpenStreetMap tags

- **Per-country tag statistics exist without a bulk download,** on Geofabrik's regional
  taginfo instances, a few kB per key per country. Illustration from the one response
  read: Germany, `roof:shape`: gabled 62.3 %, flat 23.1 %, hipped 5.4 %.
- **Three problems.** The shares are of TAGGED objects, heavily skewed to Europe and a few
  well-mapped cities. The licence is ODbL, and whether a table of tag shares may be
  CC BY is unresolved in OSM's own guidance. The usage policy says the service "is
  intended for the use of the OpenStreetMap community. Do not use it for other services"
  and asks for a contact in the request, which this campaign's rules do not allow.
- These are also the attributes the renderer does not draw.

### 2.4 Trees and land cover

- **Canopy share.** Hansen/UMD global forest change (tree cover in 2000 at 30 m, by 10°
  tile; CC BY 4.0). Copernicus land cover 100 m, 2019 (CC BY 4.0): discrete classes
  1.7 GB, forest type 1.0 GB, tree-cover fraction 5.7 GB. The twin already holds this
  product's class per 1/8° cell.
- **Height.** ETH and Meta/WRI canopy-height maps (CC BY 4.0; tiles only, large; modelled;
  not specific to street trees).
- **Species.** A Dryad set of 5.66 million street trees in 63 US cities (CC0, 1.24 GB,
  one CSV per city). A global urban tree inventory of 4,734 species in 473 urban areas
  (10.2 MB `[summary]`; licence `[not read]`). GBIF needs an account. BGCI's list is
  NonCommercial.
- **Season.** No product. Derivable from leaf type and climate class `[my reading]`.

### 2.5 Streets and snow

- Street form: an indicator set for 10,351 urban areas (Boeing), licence and sizes
  `[not read]`; no widths. Width and sidewalks: sparse OpenStreetMap tags alone.
- Snow normals: each source found needs an account. Köppen's cold classes are the proxy.

## 3. What has no adequate open source

- Facade and roof colour. Nothing beyond sparse, Europe-heavy tags.
- Roof and cladding material as seen from the street, world-wide.
- Construction era outside Europe.
- Street width and sidewalks with usable completeness.
- Street-tree species by country outside the United States (unless the global inventory's
  licence reads open).
- Attached against detached, and spacing, as direct measures: only derivable from
  footprints, or approximated from Local Climate Zone classes.

## 4. How the two lists meet

| What the renderer could read soonest | Open source that could feed it | Gap |
|---|---|---|
| Storeys by density class and region | GHSL building height (cell averages) | per cell, not per building; a storey height must be assumed |
| Footprint, gap, setback by class and region | Local Climate Zones as form classes; footprints for size | no direct measure; footprints are very large files |
| Street spacing and width by class and region | none for width | spacing derivable from street-network indicators, licence unread |
| Canopy share by land cover and climate zone | Hansen tree cover with Köppen | the twin has no climate-zone key yet |
| Roof tone by region or climate | none | no colour source; would need imagery, not scouted |
| Ground palette by land cover and climate | none as a table | as above |

- **The honest reading:** the renderer's first need (form by density class and region)
  is served by modelled rasters of 1 to 2 GB, not by small tables; its third and fourth
  (roof and ground tone) have no table source at all; and the richest published detail
  (roof shape, materials, species) is for things the renderer does not draw.
- A table here would be a prior by region, read from a model's output over real
  buildings and summarised so that no real feature remains. The seam requires that.

## 5. For approval by name (four; none is approved)

1. **Köppen-Geiger V3, Beck et al. 2023** — `koppen_geiger_tif.zip`, 90 MB, gloh2o.org —
   CC BY 4.0 as the page words it, with a link discrepancy to settle first — the climate
   key for canopy, tint and palette tables; the twin would need a small grid of it.
2. **Global map of Local Climate Zones, Demuzere et al.** — `lcz_filter_v3.tif`, 1.4 GB,
   Zenodo record 8419340 — CC BY 4.0 — shares of built-form classes by country and
   density class: the stand-in for attached against detached, rise and spacing.
3. **GHSL building height (GHS-BUILT-H R2023A), JRC** — global raster, size to be read
   first — CC BY 4.0 — storeys by country and density class.
4. **Hansen/UMD tree cover 2000** — 10° tiles, a stated sample of tiles and not the globe —
   CC BY 4.0 — canopy share by land-cover class and climate zone.

Not offered: OpenStreetMap tag statistics (the service's policy and the contact it asks
for; shares of tagged objects are not shares of buildings; and nothing draws them yet);
any footprint set (size); anything NonCommercial.

Two things only the lead can decide: whether rasters of this size may be processed on
this machine under "no long runtimes"; and whether to ask the twin to draw roof form,
materials and species, which is where published detail is richest and which would need
its rendering work before any table.
