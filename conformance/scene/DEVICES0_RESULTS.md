# DEVICES-0 (United States) — results, 2026-11-11. Branch (a). One smartphone per adult, not per person; a device table fitted in 2020 fails in 2024; the building matters; and the households without a computer are the ones without fixed broadband.

*Prereg `DEVICES0_PREREG.md`, the record before the parse `devices0/BEFORE_THE_PARSE.md`,
instrument `devices0/devices0.py`, log `devices0/run2.log`, numbers
`devices0/results.json`, table `tables/devices0_us/device_table.json` with its sidecar.
18,496 households of 2020 and 16,676 of 2024. Errors are the agency's jackknife.*

## The twin's censuses (S1): one refused, two certified, for the US

| the twin says | the survey, 2020 | verdict |
|---|---|---|
| 1.0 phone per person | 0.813 smartphones per person | **refused** |
| 0.5 laptop per person | 0.576 | certified |
| 2.25 devices per person | 2.15 phones, laptops, desktops and tablets | certified |

- **Smartphones per ADULT are 0.999.** The twin's number is right as "one per adult" and
  wrong as "one per person": households hold children.
- These are verdicts about the United States in 2020. The twin's figures are world ones.

## Who lives there, not how many (S2): neither met nor killed

Three rates fitted to each kind, then asked to predict the count at each household size:

| kind | per adult 18–64 | per adult 65+ | per child | predicts sizes 1 to 6 |
|---|---|---|---|---|
| smartphone | 0.93 | 0.67 | 0.52 | within budget at 1 to 5; unresolved at 6 |
| laptop | 0.65 | 0.40 | 0.42 | refused at 1 and at 6 |
| tablet | 0.39 | 0.43 | 0.41 | certified at 3 and 4 only |

- For smartphones, who lives in the household is enough. For laptops it is not: people
  living alone have a fifth fewer laptops than their age group's rate predicts, and
  households of six have more.

## A table fitted in another year (S3): KILLED

The 2020 table against 2024, by household size:

- **Televisions:** certified at each size from 1 to 6.
- **Smartphones:** certified from 2 to 6; **refused for one-person households** (0.88 in
  2020, 1.22 in 2024).
- **Laptops:** refused in households of 6 and of 7 or more; unresolved at 1, 4 and 5.
- The kill was "refused at some size for 2 of the 3", and it fired.
- The two years differ by a pandemic, by release status (2024 is a first release) and by
  top-coding as well as by time. The one-person smartphone jump of 39 % in four years is
  larger than drift alone would suggest, and its cause is not established here.

## Drift, 2020 to 2024 (S4, a reading, two points only)

| kind | per household 2020 | 2024 | change |
|---|---|---|---|
| set-top box | 1.21 | 0.72 | −40 % |
| streaming device | 1.07 | 1.42 | +33 % |
| games console | 0.49 | 0.59 | +21 % |
| other mobile phone | 0.28 | 0.23 | −18 % |
| smart speaker | 0.64 | 0.75 | +18 % |
| smartphone | 1.95 | 2.11 | +8 % |
| television | 2.29 | 2.15 | −6 % |
| laptop | 1.38 | 1.30 | −6 % |
| desktop | 0.52 | 0.49 | −6 % |
| tablet | 1.01 | 0.99 | −2 % |

The always-on boxes are the ones moving fastest: set-top boxes are disappearing and
streaming devices and smart speakers replacing them. The 2015 file was not read.

## Does the building matter (S5): KILLED. It does

Detached houses against apartments in buildings of five or more units, at equal household
size (apartment relative to house):

| kind | difference | |
|---|---|---|
| set-top box | −60 % | ± 8 |
| desktop | −45 % | ± 9 |
| television | −36 % | ± 3 |
| smart speaker | −30 % | ± 10 |
| tablet | −25 % | ± 6 |
| streaming device | −18 % | ± 8 |
| smartphone | −3 % | ± 3 |
| laptop, games console, other phone | about 0 | |

- Four kinds differ by more than 20 % beyond their error; the kill needed four.
- **Personal devices follow people; fixed devices follow the dwelling.** A device
  generator needs the building type for desktops, televisions, set-top boxes and speakers,
  and does not need it for phones and laptops.
- Urban against rural at equal household size is smaller: laptops −19 % and smart
  speakers −29 % in rural homes, other phones +22 %.

## Do the have-nots cluster (S7): MET

- 11.1 % of households have no laptop, desktop or tablet. 27.2 % have no fixed broadband.
- If those were independent, 3.0 % would have neither. **8.7 % have neither**: a ratio of
  `2.88 ± 0.04`.
- Kinds must be drawn jointly. A generator that draws each kind on its own would give
  nearly every home something to host on, and three times too few homes with nothing.

## What a household already has (S6, a reading)

| | all | one person | four persons |
|---|---|---|---|
| fixed broadband (a gateway, by declaration) | 72.8 % | 60.9 % | 81.9 % |
| cellular data only | 14.6 % | 16.0 % | 12.1 % |
| no internet at home | 7.1 % | 16.2 % | 1.3 % |
| a desktop (plugged in, could host) | 40.8 % | 27.4 % | 47.5 % |
| no laptop, desktop or tablet | 11.1 % | 22.8 % | 3.6 % |
| someone home all five weekdays | 74.7 % | 65.2 % | 79.3 % |
| nobody home on any weekday | 7.9 % | 13.6 % | 5.3 % |

- So about a quarter of US households have no fixed line for an always-on node to sit
  behind, and in one-person households it is two fifths.
- The at-home figures are from 2020, a pandemic year, and are high for that reason.
- The main television is on about 6.6 hours on a weekday.

## The shape of a count (S8, a reading)

Variance over mean, at household sizes 1 to 4: smartphones `0.35` to `0.47` (far narrower
than a Poisson: nearly everyone has exactly one), laptops `0.65` to `0.88`, televisions
`0.54` to `0.74`, desktops and tablets near `1`, set-top boxes `1.4` to `2.3`, smart
speakers `2.2` to `2.4` (most homes none, some many).

## Gates, plants and what went wrong

- G0: weighted households 123.53 million and 132.54 million against the agency's stated
  123.5 and 132.5; members equal their three age groups in each household below the top
  codes. Eight plants pass.
- The first run computed everything and then crashed writing its output file (a number
  type the writer did not accept). Nothing was printed or seen from it
  (`run1_crashed_writing.log`); the second run is the one read.
- Cells were pooled upward for four kinds (other phones, desktops, set-top boxes, smart
  speakers) to keep each cell's error within half its budget.

## What this is not

- Not which devices are on. Power state and can-host are declared attributes per kind.
- Not a router count: the survey never asks. A gateway is declared present with fixed
  broadband.
- Not outside the United States, and not a current year.

## What follows from it

- The device generator's rule: phones by who lives in the household (three rates);
  fixed devices by household size AND building type; all kinds drawn jointly; each table
  carrying its year.
- For the energy delta: in the US about 73 % of households have an always-on gateway
  already, 41 % have a desktop that could host if it is on, and 9 % have neither a
  computer nor a fixed line.
