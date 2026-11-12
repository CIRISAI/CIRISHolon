# The next free references — a design-stage attack on two plans, 2026-11-11

*Before drafting, the reviewer was asked to attack the PLAN for two references from the free
list: buildings per tile from the settlement grids (BUILT-0) and device ownership by country
from World Bank mirrors of ITU indicators (DEVICES-0). It fetched nothing, ran nothing, and
opened no documentation page: every statement about a product or a licence below is from its
memory and is to be read at the publisher's page before any freeze. The author's condensation.*

**Verdict.** Neither plan as stated reaches a row the first consumer reads. A different
reference should come next.

## BUILT-0: only in a small form, and not under that name

- **The per-person readings would be circular.** The population grid is, to the reviewer's
  knowledge, made by spreading each census unit's count over that unit's residential built-up
  volume. So people per built volume is constant within a census unit by construction, and a
  "volume per person by tile class" table would read each country's census geography, not
  its buildings. The tile's class is circular the same way.
- **What is independent and worth reading:** how built volume divides into coverage and
  height (surface fraction, and mean height as volume over surface), and the non-residential
  share, which bears on where people are by day.
- **It gives no building count** and so cannot reach "households per building".
- **Two stakes that decide something:** (1) the circularity itself: within a cell, the
  coefficient of variation of people over residential volume across populated pixels is
  below `0.1` in at least 80 % of cells; if met, no per-person reading is published. (2) The
  twin's own stated storeys by class against mean height over 3 m, which the reviewer expects
  the apartment class to miss.
- It would feed decoration: no model of the twin reads storeys.

## DEVICES-0 from country indicators: an appendix table at most

- **It repeats the even-spread error one level up.** UNEVEN-0 measured that a cell's mean
  applied to every tile mislabels most people; a country's mean applied to every household
  does the same, and fixed broadband is concentrated in towns.
- **No honest mapping to a household's device mix exists.** Mobile subscriptions count SIMs.
  Fixed broadband per 100 people is the most useful of the set and still needs persons per
  household and a business-line share to become a household share.
- **The honest product:** a per-country rank with its unit in full and two flags,
  `not_devices` and `not_households`; any household or sub-country request returns
  `unresolved: reference at country grain only`.

## What it recommends instead

**The US census household tables** (American Community Survey; US government work; to be
confirmed at the publisher's page). One source, at tract grain, for three of the four things
the twin reads: the device mix per household (computer types and internet subscription type),
persons per household, and households per structure type. NAT class stays unreferenced.

Its traps, stated before any draft:
1. A tract is sized by population, not area. Joining tracts to tile classes is a unit
   mismatch of the kind that sank STREETREF-1; either key by the tract's own density and say
   so, or reduce to the tiles a tract covers and state the assumption.
2. Every tract estimate has a published margin of error; the resolution floor must use it.
3. "Units in structure" is a banded structure TYPE with an open top band, not a count per
   building.
4. The device tables are marginals and a few joints, not the full joint with household
   size: `SceneCert.marginals_match_joint_differs` applies.
5. The envelope is the United States. Everywhere else is `unresolved`.

A stake that can fail: device mix depends on density class (the share of households with a
fixed subscription differs between the lowest and highest density classes by a stated
amount). If not, density is no key for devices.

**Second choice, and the first if the census tables are out of bounds:** the time-use survey,
for "who is home when". Its traps: location is not recorded for sleep; one person per
household and one day per person, so "nobody home" cannot be read for a household; nobody
under 15; and the week is a composite of different people, so continuity holds only for
totals across each day's seam, which is itself a stake.

**Ranking:** census household tables; the time-use survey; BUILT-0 small; DEVICES-0 only as
an appendix to the first.
