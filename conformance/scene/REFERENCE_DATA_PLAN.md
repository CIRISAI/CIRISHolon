# Reference data plan: sizes, storage and compute for the world-model registry

*Provenance, added when this report entered the repository (2026-11-11): written by a planning
subagent briefed to size and plan only, and placed here as it was returned. Four deviations
from its no-download rule, as it reported them: it read Microsoft's 7.2 MB file manifest; a
zero-length range request to APNIC was ignored by the server and a 3.98 MB response was
streamed and discarded; TeleGeography's 41 KB cable list was read; and a fetch tool saved a
275 KB terms PDF into the session's scratch directory. No dataset was kept. Its core-hour
figures rest on four assumed rates, none measured. Free disk on the workstation has since
fallen to 33 GB (97 % used) from the 48 GB it measured.*

*Written 2026-11-11. Sizing and planning only. No dataset file was downloaded, no account
was created, no long computation was run. Sizes were read from publisher listings, bucket
indexes, manifests and HTTP HEAD responses on 2026-11-11; each carries its source. Where no
source could be read the entry says "size not confirmed" and gives a labelled estimate.*

## 0. How to read this

**Units.** GB = 10^9 bytes. GiB = 2^30 bytes. Apache-style directory listings (JRC,
RouteViews, CAIDA, LODES) print binary units: checked on one file (the JRC listing shows
`460M` for the GHS-POP 30 arc-second zip whose exact size is 482,351,880 B). Those figures
are quoted as GiB or MiB.

**The two plans.**
- SAMPLED: the STREETREF-1 design applied to every source. At most 36 classes x 40 parent
  cells x 3 tiles = 1,440 parent cells of 1/8 degree and 4,320 tiles of 0.02 degrees (the
  class count is provisional under the twin's HOLD). For a source that is small or that
  only has global files, "sampled" means the coarsest file or the one layer the tranche
  names.
- FULL: every pixel or record of the product, reduced to the same tables.

**Compute rates assumed (none measured; measure each on the first 20 units before a run).**
- R1 raster: 2 x 10^7 pixels per core-second for decode plus binning into cells. The twin's
  land-cover build read 5.1 x 10^10 pixels with four processes, but logged no rate.
- R2 text features: 2 x 10^4 GeoJSON or CSV building records per core-second.
- R3 columnar features: 1 x 10^5 Parquet building records per core-second.
- R4 routing tables: 10^5 to 10^6 MRT path entries per core-second, depending on parser.

**Deviations from the hard rules, stated.**
- I read the Microsoft footprints manifest `dataset-links.csv` (7.2 MB). It is a manifest.
- A research agent, checking APNIC, sent a zero-length range request that the server
  ignored; a 3.98 MB JSON body streamed to /dev/null. Nothing was kept.
- The same agent read TeleGeography's 41 KB `cable/all.json` to count cables.
- A fetch tool saved a 275 KB MTUS terms PDF into the session's tool-results directory on
  its own. It is a licence document, not data.
- The agent assigned to elevation, hazards and cloud copies stopped on a usage limit. I did
  that part directly, with less coverage: SRTM, ALOS, FABDEM, WRI Aqueduct, GEM and IBTrACS
  are not confirmed.

**Local machine, as measured.** 32 cores, 31 GB memory (24 GB available), 48 GB free on `/`
at 95 % used (the brief says 53 GB).

---

## 1. First tranche (free to use)

### 1.1 OpenStreetMap

| | |
|---|---|
| Publisher, licence | OpenStreetMap Foundation; ODbL 1.0. The reading is in STREETREF-1. |
| Access | Public read API: no account. Planet and extracts: no account. |
| Hosting | `planet.openstreetmap.org` (OSMF). AWS S3 `osm-planet-eu-central-1` (eu-central-1), replica `osm-planet-us-west-2` (us-west-2), and `osm-pds` (us-east-1, analytics formats). Anonymous listing works; not requester-pays per the AWS registry entry. |
| Full size | Planet PBF `planet-261102.osm.pbf`: 95,715,394,709 B (S3 listing; the OSMF page shows 89G). Planet XML bz2: 158 GiB (`planet-260105.osm.bz2`). Uncompressed: not stated. |
| Regional extracts | Geofabrik continents: Africa 7.4 GB, Antarctica 31.6 MB, Asia 15.3 GB, Australia and Oceania 1.5 GB, Central America 757 MB, Europe 33.0 GB, North America 18.2 GB, South America 3.9 GB. Sum about 80.1 GB. |
| Partial reads | API: yes, by bounding box. PBF: no spatial index; a region means an extract. |

Sources: `https://planet.openstreetmap.org/pbf/`, `https://planet.openstreetmap.org/planet/2026/`,
`https://osm-planet-eu-central-1.s3.eu-central-1.amazonaws.com/?list-type=2&prefix=planet/pbf/2026/`,
`https://download.geofabrik.de/`, `https://raw.githubusercontent.com/awslabs/open-data-registry/main/datasets/osm.yaml`.

- **Reduction.** STREETREF-1's ten observables per tile, keyed by region x density class.
  Serves list A (households per street and distance between neighbours, through their
  stand-ins `bps` and `nn`), population rows for buildings, streets and intersections, and
  from list C gathering places and towers where mapped.
- **SAMPLED.** 4,320 requests. Bytes per tile: size not confirmed (the prereg measures it
  on the first 20 tiles). Estimate 0.1 to 2 MB per tile uncompressed, so 0.4 to 9 GB;
  planning figure 2 GB. Reasoning: a 4.9 km^2 tile holds from tens to about 20,000
  buildings at roughly 2 KB each with geometry. The service's caps (2,000 requests and
  500 MB per quota window) make this at least 3 windows by request count. Peak working
  storage under 0.1 GB, since each tile is deleted after its row is written. Committed
  table: 4,320 rows x about 1,015 float64 = 35 MB raw, about 15 MB compressed.
  Compute about 5 core-hours, under 2 GB memory.
- **FULL.** 95.7 GB fetched, 1 request. Peak working storage 250 to 300 GB (PBF, a
  roads-and-buildings filter output, and a node-location index), or 64 to 128 GB of memory
  if the index is held in RAM. Compute 100 to 400 core-hours, planning figure 150: PBF
  decode about 50, building statistics about 35 at R3, street-graph statistics about 35,
  margin. The building and way counts behind this are not confirmed.
- **Risks.** ODbL reach on the table (branch (l) of STREETREF-1). Building completeness is
  uneven, so buildings cannot certify without a second source (1.4 to 1.6). The API caps.

### 1.2 GHSL R2023A (EC JRC)

| | |
|---|---|
| Licence | CC BY 4.0, per the `copyright.txt` beside each product. |
| Access | Open HTTPS directory, no account. |
| Hosting | `jeodpp.jrc.ec.europa.eu/ftp/jrc-opendata/GHSL/` (JRC). **No public cloud bucket found.** Catalogue pages exist in Google Earth Engine for BUILT-S, -H, -V, SMOD and POP (`JRC/GHSL/P2023A/...`); that needs an account and is not a bucket. Not in the Planetary Computer catalogue. |
| Format | One zip per product, epoch, grid, holding a tiled GeoTIFF. Per-tile zips also exist (375 tiles for BUILT-S 100 m epoch 2020: sum 1.70 GiB, median 1.6 MiB, largest 40 MiB). |
| Partial reads | By tile file, yes. Range reads inside a zip: no. |

Sizes of the global zips, as listed. Epochs are 1975 to 2030 in 5-year steps (12).

| product | 10 m (Mollweide) | 100 m (Mollweide) | 1 km (Mollweide) | 3 arcsec | 30 arcsec | all epochs, all grids |
|---|---|---|---|---|---|---|
| BUILT-S, epoch 2020 | 30 GiB (2018 only) | 1.9 GiB | 145 MiB | 3.0 GiB | 178 MiB | 84.6 GiB |
| BUILT-S non-residential | 1.2 GiB (2018 only) | 173 MiB | 32 MiB | 252 MiB | 35 MiB | 6.7 GiB |
| BUILT-H, 2018 only: gross (AGBH) / net (ANBH) | none | 4.3 / 1.8 GiB | none | 7.3 / 5.5 GiB | none | 18.9 GiB |
| BUILT-V, epoch 2020 | none | 2.4 GiB | 156 MiB | 4.1 GiB | 193 MiB | 71.1 GiB |
| BUILT-V non-residential | none | 227 MiB | 34 MiB | 359 MiB | 38 MiB | 6.9 GiB |
| SMOD, epoch 2020 | none | none | 28 and 34 MiB (two versions) | none | 32 MiB | 1.2 GiB |
| POP, epoch 2020 | none | 4.7 GiB | 307 MiB | 12 GiB | 460 MiB | 180.5 GiB |

- One epoch at every grid: about 81 GiB. All epochs at every grid: about 370 GiB (397 GB)
  in 284 zips. Uncompressed: not stated; the one file the twin holds is smaller unzipped
  (383 MB) than zipped (482 MB), so on-disk size is about the same.
- BUILT-H has no coarse grid. SMOD has no fine grid. 10 m exists for BUILT-S 2018 only.

Source: recursive read of the listing under
`https://jeodpp.jrc.ec.europa.eu/ftp/jrc-opendata/GHSL/GHS_{BUILT_S,BUILT_H,BUILT_V,SMOD,POP}_GLOBE_R2023A/`.

- **Reduction.** Per 1/8 degree cell: built-up surface, volume, mean height (volume over
  surface), settlement-class shares. Per class: the distribution of those, and their
  spread inside a cell at 100 m. Serves population rows (buildings, storeys), the class
  label "settlement" for every other reduction, list C rural settlement patterns, and the
  twin's in-cell weights.
- **SAMPLED.** Epoch 2020, 30 arcsec BUILT-S, BUILT-V, SMOD (403 MiB), plus 100 m BUILT-S,
  BUILT-V and net height for in-cell spread (6.1 GiB). Total 6.5 GiB (7.0 GB), 6 requests.
  The sample's 1,440 parents touch most land tiles, so tile files save little. Peak
  working storage about 13 GiB (zip plus unzipped). Compute about 3 core-hours at R1
  (6.5 x 10^10 pixels per 100 m layer), under 4 GB memory. Table: three 2880 x 1440 uint8
  or uint16 grids, 4 to 8 MB raw each, 0.3 to 0.6 MB compressed, plus a class table of
  tens of KB.
- **FULL.** 397 GB, 284 requests. Peak working storage about 60 GiB streamed (the 10 m zip
  plus its raster). Compute about 320 core-hours at R1 over about 2.3 x 10^13 pixels; an
  upper figure, since ocean tiles decode faster.
- **Risks.** GHS-POP is disaggregated by built-up volume, so BUILT-V and POP are not
  independent readings. Mollweide grids need a per-pixel inverse projection. Epoch 2020 is
  modelled; height is 2018.

### 1.3 Copernicus Global Land Cover 100 m, collection 3, epoch 2019

| | |
|---|---|
| Licence | CC BY 4.0 (Zenodo record). Statement "Contains modified Copernicus Service information 2019". |
| Hosting | Zenodo record 3939050 (CERN). Not a public cloud bucket. A catalogue page exists in Earth Engine. |
| Format | One global GeoTIFF per layer, 362,880 x 141,120 pixels, tiled 256 x 256, DEFLATE. |
| Partial reads | Internally tiled, so range reads are possible in principle; Zenodo's support for them was not tested. A per-cell statistic needs every pixel anyway. |

| layer | bytes |
|---|---|
| Discrete classification map (the twin already used this file) | 1,713,345,228 |
| Tree cover fraction | 5,650,453,608 |
| Built-up cover fraction | 585,129,418 |
| Crops cover fraction | 2,978,277,982 |
| Grass cover fraction | 7,262,041,349 |
| Shrub cover fraction | 5,640,237,415 |
| Bare cover fraction | 3,204,609,492 |
| Permanent water cover fraction | 493,759,033 |
| Seasonal water cover fraction | 603,501,616 |
| Snow cover fraction | 111,788,764 |
| Moss and lichen cover fraction | 306,691,757 |
| Discrete classification probability | 7,708,773,201 |
| Data density indicator | 3,757,728,304 |
| Forest type | 1,037,889,222 |
| Change confidence | 151,834,438 |
| **Whole record (16 files)** | **41,207,135,373** |

The ten fraction layers sum to 26.84 GB; with the discrete map, 28.55 GB.
Source: `https://zenodo.org/api/records/3939050`.

- **Reduction.** Mean cover fraction per 1/8 degree cell and its distribution per class.
  Serves the in-cell weights, the terrain class label, list C material and
  indoor-outdoor context, and trees as decoration with a real density.
- **SAMPLED.** Tree and built-up fractions: 6.24 GB, 2 requests, peak storage 6.3 GB.
  Compute 1.4 core-hours at R1 (5.1 x 10^10 pixels per layer). Table: two 2880 x 1440
  uint8 grids, 4.1 MB raw each.
- **FULL.** 41.2 GB, 16 requests, peak 8 GB streamed. Compute about 11 core-hours.
- **Risks.** 2019. No data south of 60 S. Built-up fraction overlaps GHSL BUILT-S: two
  readings of one thing, useful as a cross-check, not as two independent supports.

### 1.4 Microsoft GlobalMLBuildingFootprints

| | |
|---|---|
| Licence | README: "freely available for download and use under CDLA Permissive 2.0". The Planetary Computer copy (an older vintage, 999 million buildings) is labelled ODbL-1.0. The two differ; the README governs the current files. |
| Hosting | Azure blob static site `minedbuildings.z5.web.core.windows.net` (region not confirmed). Planetary Computer collection `ms-buildings` in westeurope. |
| Full size | 116.8 GB compressed (sum of the manifest's rounded sizes), 30,344 files, 225 locations, 25,606 distinct level-9 quadkeys. 1.4 billion buildings per the README. Uncompressed: not stated; estimate 350 to 470 GB at 3 to 4 times. |
| Format | `.csv.gz` of GeoJSON lines, one file per location and level-9 quadkey (about 0.7 degrees wide). Median file 0.37 MB, 90th percentile 10.6 MB, largest 292 MB. |
| Partial reads | Per-quadkey files, yes. Inside a file, no. |

Sources: `https://minedbuildings.z5.web.core.windows.net/global-buildings/dataset-links.csv`,
`https://raw.githubusercontent.com/microsoft/GlobalMLBuildingFootprints/main/README.md`,
`https://planetarycomputer.microsoft.com/api/stac/v1/collections/ms-buildings`.

- **Reduction.** For the same tiles as 1.1: buildings per km^2, footprint area, nearest
  building distance, cluster shares, and height where given. Serves the building
  completeness argument STREETREF-1 owes, and list C storeys.
- **SAMPLED.** One file per parent cell, sometimes up to four: at most about 1,700
  requests. A draw weighted by people lands on large files: the size-weighted mean file is
  31.5 MB, against a plain mean of 3.85 MB. So 15 to 50 GB, planning figure 30 GB. Peak
  working storage under 2 GB (one file at a time). Compute about 5 core-hours at R2.
  Table about 15 MB compressed.
- **FULL.** 116.8 GB, 30,344 requests, peak under 2 GB streamed. Compute 25 to 40
  core-hours.
- **Risks.** Machine-detected footprints from imagery of 2014 to 2025: misses and merges
  differ by region. The sample saves only a factor of about 4 over the whole.

### 1.5 Google Open Buildings v3 (not global)

| | |
|---|---|
| Licence | Publisher page: shared under CC BY 4.0 and ODbL 1.0, the user picks one. |
| Hosting | Google Cloud Storage `gs://open-buildings-data`, location `EU`, multi-region, standard class. Anonymous listing works. Also in Earth Engine. |
| Full size | Polygons, gzip, by S2 level 4: 178.3 GB in 333 files. By S2 level 6: 175.5 GB in 3,330 files (median 9.2 MB, largest 1.72 GB). Uncompressed polygons: 466.3 GB. Points only: 48.1 GB (level 4) or 46.2 GB (level 6). |
| Coverage | Africa, South and South-East Asia, Latin America and the Caribbean. |
| Partial reads | Per S2 cell file. A level-6 cell is roughly 100 km across. |

Sources: `https://storage.googleapis.com/storage/v1/b/open-buildings-data/o?prefix=v3/`,
`https://sites.research.google/gr/open-buildings/`.

- **SAMPLED.** Poor fit. The size-weighted mean level-6 file is 337 MB, so a
  people-weighted draw pulls most of the product: 60 to 120 GB of polygons, or 20 to 30 GB
  using the points files (centroid, area, confidence), which are enough for density, area
  and nearest-neighbour distance. About 800 requests. Compute about 5 core-hours.
- **FULL.** 178.3 GB polygons, 333 requests; compute about 40 core-hours.
- **Recommendation.** Do not fetch this directly. Overture (1.6) carries it with a source
  column and spatial row groups.

### 1.6 Overture Maps, release 2026-10-21.0 (share-alike for buildings)

| | |
|---|---|
| Licence | AWS registry entry: CDLA Permissive v2 "unless derived from a source that requires publishing under a different license, such as data derived from OpenStreetMap", which is ODbL. Buildings and transportation are ODbL in effect. |
| Hosting | AWS S3 `overturemaps-us-west-2` (us-west-2). Azure blob `overturemapswestus2`, container `release` (the same theme folders are listed). Anonymous listing works on both. |
| Format | GeoParquet, spatially sorted, with a bounding-box column. |
| Partial reads | Yes: Parquet row-group statistics on the bounding box allow range reads. Row-group size was not confirmed, because that needs a file footer read. |

| theme | files | GB |
|---|---|---|
| buildings (building 512 files 276.94; building_part 1 file 0.62) | 513 | 277.6 |
| transportation (segment 72.47; connector 24.93) | 160 | 97.4 |
| places | 16 | 11.9 |
| base (land_cover 109.06; land 30.52; water 28.82; land_use 18.93; infrastructure 13.93; bathymetry 0.05) | 257 | 201.3 |
| addresses | 64 | 27.7 |
| divisions | 10 | 5.7 |
| **all themes** | 1,020 | **621.6** |

Source: `https://overturemaps-us-west-2.s3.amazonaws.com/?list-type=2&prefix=release/2026-10-21.0/`.

- **SAMPLED.** Estimate only: about 1,440 parents x 2 row groups x 13 MB (bracket 5 to
  60 MB) = about 40 GB, bracket 15 to 170 GB, in 512 footer reads plus 3,000 to 9,000
  range requests. Compute about 5 core-hours at R3.
- **FULL.** 277.6 GB for buildings; compute 30 to 60 core-hours; 8 GB per worker.
- **Use.** As the alternative to 1.4 and 1.5 in one read, and as the easier form of 1.1
  FULL (segments and buildings as Parquet instead of PBF).
- **Risks.** Not a second independent source: it is OSM, Microsoft and Google conflated.
  The completeness argument needs the source column kept apart.

### 1.7 World Bank indicators mirroring ITU

- Licence CC BY 4.0 per the indicator pages (earlier check). No account. Host
  `api.worldbank.org`; not a cloud bucket.
- Sizes by HEAD: mobile subscriptions per 100 (`IT.CEL.SETS.P2`) 84,865 B zip; fixed
  broadband per 100 (`IT.NET.BBND.P2`) 62,296 B zip. Other indicators (internet users,
  fixed telephone) were not HEADed; estimate 60 to 90 KB each. The whole WDI bulk file is
  283,089,754 B.
- **Reduction.** Country x year values, mapped to the twin's regions. Serves list B
  adoption and device counts per household at country grain.
- **SAMPLED = FULL.** About 0.5 MB in about 6 requests. Table about 50 KB. Compute nil.
- **Risk.** The source is ITU, whose own terms differ. Country grain only.

### 1.8 RouteViews

| | |
|---|---|
| Licence | CC BY 4.0 with a revocation clause (earlier check). |
| Hosting | `archive.routeviews.org` (University of Oregon). Not a cloud bucket. |
| Format | MRT, bz2, one RIB every 2 hours per collector. No partial reads. |
| One RIB, one collector | `route-views2`: listed 78M on 2026-10-01; mean 72.8 MiB over October 2026. Largest collector (`route-views.linx`): mean 193 MiB. |
| A month, all collectors | October 2026: 10,790 RIB files, 597.4 GiB (641 GB), from 29 collectors that returned a listing. Four collector names returned 404 at the expected path. |
| Uncompressed | Not stated; estimate 10 to 15 times. |

Source: `https://archive.routeviews.org/<collector>/bgpdata/2026.10/RIBS/` for each collector.

- **Reduction.** The AS graph: nodes, links, inferred relationships, degree and
  customer-cone distributions, by region. Serves the relationship row AS to AS, and list B
  "the AS graph".
- **SAMPLED.** One RIB from each collector at one timestamp: about 1.6 GiB (1.73 GB), 29
  requests, peak storage about 3 GB. Compute 2 to 12 core-hours (about 4 x 10^8 path
  entries at R4, plus relationship inference), 8 to 16 GB memory. Table: graph statistics
  about 1 MB; the inferred link list about 2 MB compressed.
- **FULL.** 641 GB, 10,790 requests, streamed. Compute 40 to 4,000 core-hours depending on
  the parser; planning figure 400. Little is gained: paths repeat across a month.
- **Risks.** Collector bias (the graph seen is the graph near the collectors; peering
  links are under-seen). Inferred relationships are a model.

### 1.9 Gridfinder

- Licence CC BY 4.0 (Zenodo record 3628142). No account. Host Zenodo; not a cloud bucket.
- Sizes: `grid.gpkg` 724,566,016 B; `lv.tif` 251,153,160 B; `targets.tif` 28,981,506 B;
  total 1,005,674,147 B in 4 files. Source `https://zenodo.org/api/records/3628142`.
- **Reduction.** Per 1/8 degree cell: line length, distance to the nearest line,
  low-voltage density; connected components as candidate power regions. Serves list B
  failure domains for power.
- **SAMPLED = FULL.** 1.0 GB, 3 requests. Compute about 3 core-hours, 8 GB memory. Table
  about 5 MB.
- **Risks.** A predictive model built on OpenStreetMap lines and night lights: check the
  share-alike reach before publishing. About 2020.

### 1.10 American Time Use Survey

- Public domain: "everything that we publish ... is in the public domain". No account.
  Host `bls.gov`; not a cloud bucket. **The host returned HTTP 403 to automated HEAD
  requests**; the listing page was readable through a fetch tool.
- Multi-year 2003 to 2025: 593.3 MB in 10 zips (2003 to 2024: 569.9 MB). Of that the
  replicate weights are 292.6 + 56.1 MB. The six files a rhythm needs (respondent, roster,
  activity, who, summary, CPS) sum to 239.1 MB. Uncompressed: not stated; estimate 4 to
  8 GB with weights. Rows: over 258,000 interviews; activity rows not stated, estimate 4.5
  to 5.5 million. Sources: `https://www.bls.gov/tus/data/datafiles-0325.htm`,
  `https://www.bls.gov/tus/data/datafiles-0324.htm`.
- **Reduction.** Weighted share of people present, by 168 hour-of-week bins x place kind x
  person kind. Activity codes for telephone, television and computer use give a first
  activity-by-traffic-kind rhythm. Serves every rhythm row, for one country.
- **SAMPLED = FULL.** 0.24 GB (0.59 GB with weights), 6 to 10 requests. Compute under 1
  core-hour, 4 to 8 GB memory. Table: 168 x about 10 x about 24 float32 = about 0.2 MB.
- **Risks.** One country. One diary day per respondent. No device or network attachment:
  phone on Wi-Fi or cellular by hour is inferred from place, not read.

---

## 2. Second tranche (restricted or unconfirmed)

| # | dataset | barrier | size (source) | sampled plan | full plan |
|---|---|---|---|---|---|
| 11 | Measurement Lab NDT | Google account plus membership of the M-Lab discuss group; M-Lab pays for queries run in its project, with a per-user quota reported as 10 TiB per day. Licence CC0. | **Size not confirmed**: anonymous bucket listing is denied and no table size is published. | One year, per-country and per-AS aggregate in BigQuery. Estimated scan 0.05 TB (ideal column pruning: 2.9 million tests a day x 365 x about 44 B) to 5 TB (the unified views are reported as far heavier). Nothing fetched but the result, about 20 MB. | Raw archive: estimate 20 to 50 TB per year. Not advised. |
| 12 | Ookla open tiles | Licence non-commercial share-alike per the earlier check; the licence line was not re-read here. No account. | Parquet, S3 `ookla-open-data` (us-west-2). Latest quarter: fixed 365,146,218 B, mobile 189,644,612 B. All quarters since 2019: fixed 9.63 GB, mobile 6.05 GB (32 files each). Shapefile copies 7.76 and 4.93 GB. | One quarter, both: 0.555 GB, 2 requests, about 1 core-hour. | 15.7 GB, 64 requests, about 5 core-hours. |
| 13 | OpenCelliD world export | Free account and API token. CC BY-SA 4.0. | **105 MB gz from a third-party mirror only**; publisher size, rows and uncompressed size not confirmed. | = full | 0.1 GB, 1 request, under 1 core-hour. |
| 14 | PeeringDB; CAIDA AS Relationships; AS Rank | PeeringDB AUP forbids passing data on in bulk and excludes commercial use. CAIDA: non-transferable licence, citation. AS Rank: no terms found. No accounts. | PeeringDB daily dump 111 MiB uncompressed JSON (CAIDA archive). AS relationships serial-2: 1.9 to 2.0 MiB per month. Serial-1: relationships 1.6 MiB, cones 1.6 MiB, all paths 2.2 GiB. AS organisations 4.4 MiB. AS Rank API: 123,372 ASNs, 526,512 links. | One day plus one month: about 0.13 GB, about 5 requests; AS Rank by API about 125 requests, estimate 50 MB. | A year of PeeringDB about 40 GB; all-paths 2.2 GiB per month. |
| 15 | TeleGeography cables | Geocoded data is an annual paid licence; price not public. | Site JSON answers without an account but states no licence: 713 cables, files of 41 KB to 755 KB. | Order of magnitude: 1 to 2 MB. | Same. |
| 16 | Meta Social Connectedness Index | None. HDX metadata: "Public Domain / No restrictions (CC0)". | About 16.0 GB: 1.22 GB on HDX (country 0.56 MB, GADM1 273 MB, others) plus three zips on Google Drive (4.6, 8.4, 1.8 GB). | Country and GADM1: 0.27 GB, 2 requests. | 16 GB, 9 requests; estimate 45 to 75 GB unzipped; about 5 core-hours. |
| 16 | UN household size and composition | No account. UN terms: no redistribution, no derivative works. | 342,176 B xlsx (HEAD). | = full | 0.34 MB. |
| 16 | MTUS | Free registration and acceptance of terms: no third-party transfer, no published cell under 30 cases. | **Size not confirmed.** Estimate 0.1 to 2 GB per extract. | = full | |
| 17 | APNIC IPv6 tables | No account. **No licence stated.** | Per-economy JSON, one read at 3.98 MB (US). Estimate 0.2 to 1 GB for about 240 economies; per-AS tables are embedded in HTML. | About 480 requests, estimate 0.6 GB. | Same. |
| 18 | IODA | No account; terms and limits not found. API only, no bulk file. | Estimate tens of MB per year (about 300,000 events from one sample window). | One year: hundreds to low thousands of paged requests. | |
| 18 | DOE OE-417 | **Not reached**: the host answered only over IPv6 and this machine has no IPv6 route. | Size not confirmed; estimate under 20 MB for all years. | | |
| 18 | Access Now STOP | Public spreadsheet; no licence sentence naming the dataset. | Size not confirmed; estimate under 5 MB, 1,500 to 2,500 rows, 46 columns. | | |
| 18 | Cloudflare Radar | Account and API token. CC BY-NC 4.0. 1,200 requests per 5 minutes. | API only; estimate under 50 MB for hourly series by country. | | |

**Extras the consumer notes name.** FCC Measuring Broadband America: validated sets about
29 GB, raw about 390 GB (2012 to 2023), licence wording not read. RIPE Atlas daily dumps:
about 137 GB per day compressed, kept about one month, no redistribution and no
commercial use.

**No dataset with a stated licence was found for NAT or CGN behaviour.**

### 2.19 List C sources

| source | licence | host | size (source) | partial reads |
|---|---|---|---|---|
| Copernicus DEM GLO-30 | Copernicus DEM licence, free with conditions (linked from the AWS registry; not quoted here). Some tiles withheld. | AWS S3 `copernicus-dem-30m` (eu-central-1), anonymous. Also Planetary Computer `cop-dem-glo-30` (westeurope). | 26,450 tiles. **Extrapolated** from a 40-tile listing sample: elevation files about 540 GB (mean 20.4 MB, range 0.1 to 49 MB); all objects about 1,260 GB. | Cloud-optimised GeoTIFF, per 1 degree tile. |
| Copernicus DEM GLO-90 | same | `copernicus-dem-90m` (eu-central-1); Planetary Computer `cop-dem-glo-90`. | 26,475 tiles. **Extrapolated**: elevation files about 64 GB (mean 2.4 MB); all objects about 170 GB. | same |
| SRTM / NASADEM | not read | Planetary Computer `nasadem` (westeurope); NASA Earthdata needs a login. | **Size not confirmed.** Estimate 150 to 350 GB at 30 m. | per tile |
| US LODES 8 (commute flows) | US public domain | `lehd.ces.census.gov`; not a cloud bucket. | Origin-destination, all years and job types: 12,200 files, 56.06 GiB. Residence and workplace characteristics: 61.72 and 22.91 GiB. Latest year, all jobs, origin-destination only: 102 files, 0.78 GiB. | per state and year |
| Google Community Mobility Reports | not read | `gstatic.com` | Global CSV 1,144,240,623 B; regional zip 98,167,694 B. Frozen 2022-10-18. | no |
| Overture places (gathering places) | CDLA Permissive v2 per the registry's general statement; per-theme text not read | S3 us-west-2, Azure westus2 | 11.9 GB, 16 files | row groups |
| Overture base/infrastructure (towers, power) | ODbL in effect | same | 13.93 GB, 16 files | row groups |
| CEMS-GloFAS global river flood hazard v2.1.2 | CC BY 4.0 (`copyright.txt`) | JRC host; not a cloud bucket | 3 arcsec. The 100-year layer: 542 files, 4.0 GiB. Seven return periods: about 28 GiB, extrapolated. | per tile |
| WRI Aqueduct floods, GEM seismic hazard, IBTrACS | not read | | **Not confirmed.** | |

- **Sampled.** Elevation windows around the 4,320 tiles by range request from GLO-30:
  1.5 to 4 GB in about 30,000 requests; or whole GLO-90 tiles, about 2.4 GB in about 1,000
  requests. LODES latest year 0.84 GB. Overture places for the sample about 2 GB
  (estimate). Flood 100-year layer 4.3 GB. Mobility 1.14 GB. About 10 core-hours.
- **Full.** GLO-30 540 GB and GLO-90 64 GB, 3.4 x 10^11 and 3.8 x 10^10 pixels: 5 to 20
  core-hours for terrain class per cell at R1. LODES 151 GB. Places and infrastructure
  25.8 GB. Flood about 30 GB.
- **What it serves.** Terrain class label (a class dimension for every other table); line
  of sight per building at the sample tiles; commute-flow relationship rows (one country);
  gathering-place population rows; disaster exposure attribute.
- **Gap.** No open global commute or daily-mobility aggregate was found. The mobility
  reports are a frozen pandemic-era index, not flows.

---

## 3. Summary table

| # | dataset | tranche | licence class | full size | sampled plan | hosting cloud |
|---|---|---|---|---|---|---|
| 1 | OpenStreetMap | 1 | share-alike | 95.7 GB planet PBF | 2 GB est. (0.4 to 9), 4,320 requests | AWS eu-central-1, us-west-2, us-east-1 |
| 2 | GHSL R2023A | 1 | free | 397 GB all; 87 GB one epoch | 7.0 GB | none (JRC host); Earth Engine catalogue |
| 3 | Land cover 100 m | 1 | free | 41.2 GB | 6.24 GB | none (Zenodo); Earth Engine catalogue |
| 4 | Microsoft footprints | 1 | free | 116.8 GB | 30 GB est. (15 to 50) | Azure |
| 5 | Google Open Buildings v3 | 1 | free (or share-alike, user's choice) | 178.3 GB | 20 to 30 GB points est.; poor fit | Google Cloud, EU multi-region |
| 6 | Overture buildings | 1 | share-alike | 277.6 GB (all themes 621.6) | 40 GB est. (15 to 170) | AWS us-west-2; Azure westus2 |
| 7 | World Bank indicators | 1 | free | 0.5 MB (WDI bulk 0.28 GB) | same | none |
| 8 | RouteViews | 1 | free | 641 GB per month, all collectors | 1.73 GB | none |
| 9 | Gridfinder | 1 | free (check share-alike reach) | 1.0 GB | same | none (Zenodo) |
| 10 | ATUS | 1 | free | 0.59 GB | 0.24 GB | none |
| 11 | M-Lab NDT | 2 | free, behind an account | not confirmed | 0.05 to 5 TB scanned, est. | Google Cloud (BigQuery, GCS) |
| 12 | Ookla tiles | 2 | restricted (non-commercial) | 15.7 GB | 0.56 GB | AWS us-west-2 |
| 13 | OpenCelliD | 2 | share-alike, behind a token | about 0.1 GB, unconfirmed | same | not determined |
| 14 | PeeringDB, CAIDA, AS Rank | 2 | restricted | 40 GB per year; 2.2 GiB per month | 0.18 GB | none |
| 15 | TeleGeography | 2 | paid | MBs | MBs | none (paid delivery on S3) |
| 16 | SCI; UN households; MTUS | 2 | free; restricted; restricted | 16.0 GB; 0.34 MB; not confirmed | 0.27 GB | none (HDX, Google Drive) |
| 17 | APNIC IPv6 | 2 | unconfirmed | 0.2 to 1 GB est. | same | none |
| 18 | IODA; OE-417; STOP; Radar | 2 | unconfirmed; free (unread); unconfirmed; restricted | under 0.2 GB est. | same | none |
| 19 | Elevation, commute, places, hazards | 2 | free with conditions; free; free; free | 540 + 64 + 151 + 26 + 30 GB | about 11 GB | AWS eu-central-1 and Azure westeurope (elevation); AWS and Azure (Overture) |

---

## 4. Totals

The base first tranche is items 1, 2, 3, 4, 7, 8, 9, 10 (the eight rows of FSD section
10.5). Items 5 and 6 are alternatives to item 4 and are shown separately.

| plan | bytes to fetch | requests | peak working storage | core-hours | committed tables |
|---|---|---|---|---|---|
| (a) SAMPLED first tranche, base | about 49 GB (bracket 32 to 76) | about 6,100 | 15 GB sequential; 20 GB with the rasters kept | about 25 (10 to 60) | 70 to 90 MB raw, 30 to 40 MB compressed |
| (a) with 5 and 6 sampled as well | about 114 GB (65 to 330) | about 16,000 | same | about 35 | adds about 30 MB |
| (b) FULL first tranche, items 1 to 10 | about 1,750 GB (1,294 GB without 5 and 6) | about 42,000 | 300 to 400 GB streamed; 2.0 to 2.5 TB if held | about 1,000 (400 to 5,000) | 150 to 300 MB |
| (c) both tranches SAMPLED | about 62 GB (45 to 95), plus a BigQuery scan of 0.05 to 5 TB | about 39,000 | 15 to 20 GB | about 40 | under 150 MB |
| (d) both tranches FULL | about 2,640 GB, plus a BigQuery scan of up to tens of TiB | about 130,000 | 300 to 400 GB streamed; 3 TB if held | about 1,100 (500 to 5,000) | 0.3 to 1 GB |

- (d) leaves out the M-Lab raw archive (estimate 20 to 50 TB per year) and RIPE Atlas
  dumps (137 GB per day). Neither is advised.
- The widest brackets come from four unmeasured things: bytes per API tile (item 1), the
  MRT parser's rate (item 8), Overture's row-group size (item 6), and R1.
- Memory: every sampled reduction fits in 16 GB. The one full reduction that does not fit
  in 31 GB comfortably is the OSM planet node index (64 to 128 GB in memory, or about
  100 GB of fast disk).

---

## 5. Does (a) fit on the workstation

**Yes, if it is streamed. No, if all raw data is held at once.**

- Bytes: 49 GB against 48 GB free. Held all at once it does not fit, and the upper bracket
  (76 GB) does not fit under either figure for free disk. Streamed, with each source
  reduced and deleted before the next, the peak is 15 to 20 GB and it fits with room.
- Compute: about 25 core-hours on 32 cores. Not a constraint.
- Memory: 24 GB available is enough for every sampled reduction.
- What constrains it instead: the public API's quota windows for item 1 (at least 3 by
  request count), and the 30 GB of Microsoft files over the workstation's own link.

**What does not fit on the workstation.**
- Item 5 or 6 sampled together with the rest, unless streamed one file at a time.
- Anything in (b): the OSM planet alone needs 250 to 300 GB of working disk; GHSL at all
  grids is 397 GB; a month of RouteViews is 641 GB.
- (d) in any form.
- (c) fits like (a), since the second tranche's sampled reductions are small. Its M-Lab
  part runs in BigQuery, not locally.

---

## 6. Cloud options

### 6.1 What is already hosted where

| provider | already there as public data | not there |
|---|---|---|
| **AWS** | OSM planet (eu-central-1, us-west-2; analytics copy us-east-1). Overture, all themes (us-west-2). Ookla tiles (us-west-2). Copernicus DEM 30 m and 90 m (eu-central-1). | GHSL, land cover, Microsoft footprints, Google Open Buildings, RouteViews, M-Lab |
| **Azure** | Microsoft footprints (blob static site; Planetary Computer copy in westeurope). Overture, all themes (westus2). Copernicus DEM 30 m and 90 m, NASADEM, ALOS DEM (Planetary Computer, westeurope). | GHSL, land cover, OSM planet, Google Open Buildings, Ookla, RouteViews, M-Lab |
| **Google Cloud** | Google Open Buildings v3 (GCS, EU multi-region). M-Lab NDT (BigQuery and GCS, behind an account). GHSL, land cover and Open Buildings in Earth Engine (account; a catalogue, not a bucket). | OSM planet as files, Microsoft footprints, Overture, Ookla, Copernicus DEM, RouteViews |
| **none of the three** | GHSL (JRC host), land cover (Zenodo), Gridfinder (Zenodo), RouteViews, ATUS, World Bank, CAIDA, LODES, SCI, APNIC, flood hazard (JRC host) | |

- The two largest first-tranche items by bytes (GHSL at 397 GB, RouteViews at 641 GB per
  month) are in no cloud. They come over the public internet wherever compute runs;
  inbound transfer is free on all three.
- Reading a public bucket from compute in the same region carries no transfer charge on
  any of the three. On Google, a region is not the same location as a multi-region: a
  regional machine reading the EU multi-region bucket is not automatically free.
- None of the public buckets above was found to be requester-pays. A cross-cloud read is
  then paid by the bucket's owner, not the reader, so the reader's bill does not depend
  on the choice. Speed and courtesy do.
- A BigQuery scan is billed by Google whichever cloud holds the compute.

### 6.2 Shapes

| plan | machine | storage | pattern |
|---|---|---|---|
| (a), (c) | 4 vCPU / 16 GB is enough; 32 vCPU / 128 GB finishes the rasters in one pass | 60 to 100 GB block disk | one bursty job, deleted after; or the workstation |
| (b) | 32 vCPU / 128 GB, or 32 vCPU / 256 GB if the OSM node index is held in memory | 500 GB block disk or local NVMe, streamed; 2.5 TB if held | one bursty job per source |
| (d) | 64 vCPU / 256 GB | 500 GB streamed; 3 TB if held | several bursty jobs; the raw data never needs to persist |

**What must come back out:** only the statistics tables and manifests: tens of MB for (a)
and (c), under 1 GB for (d). That is inside the monthly free egress allowance on AWS and
Azure (100 GB each), and about 0.12 USD per GiB after the first GiB on Google's default
network tier.

### 6.3 Order-of-magnitude cost, public list prices

**List prices read on 2026-11-11. They change, and spot prices change hourly.** US
regions (AWS us-east-1, Google us-central1, Azure westus2). A core-hour is the 32-vCPU
machine's price divided by 32.

| | AWS | Google Cloud | Azure |
|---|---|---|---|
| 32 vCPU / 128 GB, on-demand, USD per hour | 1.6128 (m7i.8xlarge) | 1.5539 (n2-standard-32) | 1.536 (D32s_v5) |
| same, spot | 0.529 | 0.932 | 0.284 |
| per core-hour, on-demand / spot | 0.0504 / 0.0165 | 0.0486 / 0.0291 | 0.0480 / 0.0089 |
| 32 vCPU / 256 GB, on-demand | 2.1168 (r7i.8xlarge) | 2.0962 (n2-highmem-32) | 2.016 (E32s_v5) |
| 4 vCPU / 16 GB, on-demand | 0.2016 (m7i.xlarge) | 0.1942 (n2-standard-4); 0.1340 (e2-standard-4) | 0.192 (D4s_v5) |
| block disk, per GB-month | 0.08 (gp3) | 0.10 (pd-balanced) | about 0.08 (Premium SSD v2) |
| object storage, per GB-month | 0.023 | 0.020 regional | 0.0184 |
| internet egress | first 100 GB a month free, then 0.09 per GB | first 1 GiB free, then 0.12 per GiB (default tier) | first 100 GB free, then 0.087 per GB |
| SQL scan | Athena 5.00 per TB | BigQuery 6.25 per TiB, first TiB a month free | not read |

| plan | arithmetic | AWS | Google Cloud | Azure |
|---|---|---|---|---|
| (a) 25 core-hours, 60 GB disk for 72 hours | compute + disk | 1.26 + 0.47 = about 2 | 1.21 + 0.59 + 1 egress = about 3 | 1.20 + 0.48 = about 2 |
| (b) 1,000 core-hours, 500 GB disk for 168 hours | compute + disk | 50 + 9 = about 60 (spot about 26) | 49 + 12 = about 60 (spot about 41) | 48 + 9 = about 57 (spot about 18) |
| (c) 40 core-hours, 100 GB for 120 hours, plus a 2 TiB scan | compute + disk + 6.25 | about 10 | about 10 | about 10 |
| (d) 1,100 core-hours, 500 GB for 336 hours, plus a 10 TiB scan | compute + disk + 56.25 | 55 + 18 + 56 = about 130 | 53 + 23 + 56 = about 132 | 53 + 18 + 56 = about 127 |
| (d) with 3 TB held for 336 hours instead | disk becomes about 110 to 138 | about 220 | about 250 | about 220 |
| small standing machine, 4 vCPU / 16 GB, 200 GB disk, per 730-hour month | | about 163 | about 162 (118 on e2) | about 156 |

- At the upper bracket of 5,000 core-hours, (b) and (d) are about 250 USD of on-demand
  compute on any of the three.
- On-demand compute is at parity across the three (0.048 to 0.050 USD per core-hour). The
  differences are in spot prices on the day and in the small-egress rules.
- **Bursty job against standing machine.** Every plan here is under about 250 USD as a
  bursty job at list price. A small standing machine costs more per month than plan (b)
  costs once. A standing machine earns its cost only if the reductions are to be re-run
  on a schedule, or if the capped API fetch of item 1 should run unattended across its
  quota windows; a 4 vCPU machine does that.
- Not included: taxes, public IP and NAT charges, boot disks, snapshot storage, request
  charges (about 0.0004 USD per 1,000 reads), committed-use discounts.

Sources for prices: the pages behind `aws.amazon.com/ec2/pricing/on-demand/`,
`aws.amazon.com/ec2/spot/pricing/`, `aws.amazon.com/ebs/pricing/`,
`aws.amazon.com/s3/pricing/`, `aws.amazon.com/athena/pricing/`;
`cloud.google.com/products/compute/pricing/general-purpose`,
`cloud.google.com/spot-vms/pricing`, `cloud.google.com/compute/disks-image-pricing`,
`cloud.google.com/storage/pricing`, `cloud.google.com/vpc/network-pricing`,
`cloud.google.com/bigquery/pricing`; the Azure Retail Prices API at
`prices.azure.com/api/retail/prices` and `azure.microsoft.com/en-us/pricing/details/bandwidth/`.

### 6.4 The choice, stated neutrally

- **Price does not decide it.** The plans cost single dollars to low hundreds on any
  provider.
- **Data position.** AWS holds the most of the large items (OSM planet, Overture, Ookla,
  elevation). Azure holds the Microsoft footprints, Overture and elevation. Google holds
  Open Buildings and is the only home of M-Lab.
- **By plan.** (a) and (c) need no cloud. For (b) and (d), us-west-2 on AWS sits beside
  OSM, Overture and Ookla; westus2 or westeurope on Azure sits beside Overture, Microsoft
  footprints and elevation. Either way GHSL and RouteViews arrive over the internet, and
  M-Lab is a query sent to Google.

---

## 7. Recommended order (dependencies, no dates)

1. **Class labels first.** Every table is keyed by class, and the twin holds the class
   rules (density thresholds, region rule, tile weights). Nothing below freezes before
   they arrive. *Blocks: all.*
2. **GHSL sampled** (SMOD, BUILT-S, BUILT-V; net height). Gives the settlement class and
   the per-cell building quantities. *Depends on 1. Feeds the class dimension of 3 to 9.*
3. **Land cover fractions** (tree, built-up). Gives the in-cell weights the twin may
   adopt. *Depends on 1. If the twin weights tiles by land cover, the tile draw in 4
   depends on this.*
4. **The draw.** `draw.json` from the population grid, the class labels and the tile
   weights, committed alone. *Depends on 1, and on 3 if weights are adopted.*
5. **OSM sampled (STREETREF-1)**. *Depends on 4.*
6. **A second buildings source on the same draw**: Microsoft footprints, or Overture with
   its source column. *Depends on 4. Unblocks the completeness argument, without which
   the building observables of 5 cannot certify.*
7. **Independent of the draw, in any order:** World Bank indicators; Gridfinder; ATUS;
   RouteViews. *Each depends only on 1 for its region key. ATUS needs a person-kind and
   place-kind registry before its bins are frozen.*
8. **Terrain class** from Copernicus DEM 90 m, if terrain is to be a class dimension.
   *If adopted it belongs before 4, since it changes the classes of the draw.*
9. **Second tranche, as decisions land** (section 8).

**Where a licence decision blocks.**
- Persons per household: UN terms forbid derivatives. No free substitute was sized.
- Measured access rates: M-Lab needs an account and group membership; Ookla is
  non-commercial.
- NAT and IPv6: APNIC states no licence; no NAT dataset with a licence exists.
- Exchanges and cables: PeeringDB and TeleGeography need approval or payment.
- Towers: OpenCelliD is share-alike and needs a token.
- Overture or OSM as full products: share-alike reach on the table, the same question as
  STREETREF-1 branch (l).
- Gridfinder: share-alike reach through its OpenStreetMap input.

---

## 8. Open questions for the lead

1. **Sampled or full for the first tranche?** Sampled runs on the workstation, streamed.
   Full needs a cloud burst or about 400 GB of free disk.
2. **Which second buildings source:** Microsoft (permissive, 30 GB sampled) or Overture
   (share-alike, one read that also carries Google's and OSM's buildings)? Or both and
   compare.
3. **May the reductions use the per-cell grids, not only class tables?** A 2880 x 1440
   grid per layer is 4 MB raw. It is far more than a class table and still not raw data.
4. **Is terrain a class dimension?** If yes, elevation moves ahead of the draw.
5. **M-Lab:** may an account join its discuss group so the access-rate query can be sized
   exactly (a dry run reports bytes scanned without running)?
6. **Is the use commercial or research?** One answer decides Ookla, Cloudflare Radar,
   PeeringDB, RIPE Atlas and MTUS.
7. **Buy or ask:** TeleGeography (price not public), PeeringDB approval for aggregates,
   APNIC for a licence statement, UN DESA for permission on household size.
8. **ATUS is one country.** Is a second time-use source wanted before any rhythm
   certifies outside it? MTUS needs registration and a 30-case cell floor.
9. **May one Parquet footer be read** (metadata, 1 to 5 MB) to confirm Overture's
   row-group size? It was not read under the no-data rule.
10. **Free disk:** the brief says 53 GB; the machine showed 48 GB at 95 % used.

---

## 9. Not confirmed

- M-Lab: any table or archive size, partitioning, region.
- OSM bytes per API tile (measured on the first 20 tiles by design).
- Overture row-group size, so its sampled figure is an estimate.
- Uncompressed sizes of Microsoft footprints, RouteViews RIBs, ATUS, SCI zips, GHSL
  rasters as decoded.
- Azure region of the Microsoft footprints blob.
- OpenCelliD size from its publisher; rows; download limits.
- DOE OE-417 (host unreachable from this machine); Access Now STOP size and licence;
  IODA terms and limits; APNIC licence; AS Rank terms.
- Ookla's licence line (taken from the earlier check, not re-read); Copernicus DEM
  licence text (linked, not quoted); Overture per-theme licence text.
- SRTM, NASADEM, ALOS and FABDEM sizes and licences; WRI Aqueduct, GEM, IBTrACS.
- Copernicus DEM totals are extrapolated from 40 sampled tiles per bucket, not summed.
- Whether the four RouteViews collectors that returned 404 hold October data elsewhere.
- Every core-hour figure: the four rates in section 0 are assumptions.
- MTUS size; ATUS activity row count; World Bank indicators other than the two HEADed.
