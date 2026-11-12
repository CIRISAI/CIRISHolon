# What the twin's two theories read today — a read-only survey of its code, 2026-11-11

*By a read-only survey agent over the twin's tree (branch twin/hhsize, `baae03a`), with the
twin session's own code pass agreeing on the points both covered. File and line for each
claim are in the twin's tree and in the agent's full report (kept in this session's
record); this is the author's condensation. Nothing in the twin was changed. It exists to
order the generator's next fields by what decides the two tests.*

## The locality dividend

- **The headline (about 92 % of acts never leave the signer's locality) is arithmetic on
  its inputs:** `0.65 + 0.30 × 0.90`. 65 % of acts are self or family scope and families
  are placed in the signer's locality by construction; 30 % are community acts and
  community rooms are placed 90 % in one locality, 9 % region, 1 % world. The twin's own
  ledger calls it "an input, not a discovery".
- **It reads only where an act's recipients are.** Not where bytes travel (path and hop
  counters exist and are joined to nothing), and not where receivers are by hour.
- **Counted by deliveries the same engine gives 0.58 at 100,000 nodes and 0.21 at a
  million**, because public rooms carry most deliveries.
- **A locality is one of 181 grid localities, not a town.** At a finer grain the share
  would be lower.
- **The people package** generates a social graph and measures the same quantity, with
  these levers, each a labelled guess: kin households per household 4; diaspora share
  0.12; work ties crossing locality 0.05; interest groups local 0.5 ("the main lever");
  public follows local 0.4. Its sweep results were never published.
- **Topology (a model):** of 253 same-metro pairs on different providers 7 leave the
  metro; many distant pairs are faster than the slowest same-metro pair.

**What decides it, in order:** (1) the mix of acts by scope; (2) where a household's kin
and close ties live; (3) where the members of a community or interest group live;
(4) the grain of "local"; (5) whether a local pair's bytes stay local.

## Power savings

- **The page's headline (415 TWh a year down to 27; −93.5 %) has no code behind its
  3.08 GW**: it is a literal labelled a "certified extrapolation", and the twin's own
  ledger says that certificate is not computed.
- **The only power computation** sums users × tier share × device-class share × idle
  watts × a "marginal share", against a data-centre baseline of a fixed 5 MW each. It
  reads no locality, bytes, acts, presence or ties.
- **Its device table carries no source:** phone 2.5 W idle, marginal share 0.05; laptop
  10 W, 0.10; tablet 5 W, 0.07; ARM box 5 W, 1.0; home x86 25 W, 1.0; old desktop 60 W,
  1.0. "Numbers are rough." No router wattage exists anywhere.
- **Devices per person 2.25** ("no device census behind it"); phones per person 1.0;
  laptops per person 0.5; home node in 0.60 of households; server tier 0.08.
- **The twin itself computes no energy.** Its packages emit counts (wake-ups, bytes each
  way, relays, punches) for an energy package that does not exist.
- **Its published figures disagree with each other** (3.08 GW on the page; 5.3 to 16.9 GW
  in the scaling document, against a 50 GW baseline there and 47.4 GW on the page).

**What decides it, in order:** (1) the fleet: devices per household by kind, and which
are on all day anyway; (2) measured idle and active draw per kind, with year; (3) the
"marginal share": how much of a device's draw the mesh adds, which needs duty cycle
(wake-ups, radio tail) and presence by hour; (4) the baseline: energy of the data centres
and networks the mesh would replace, per user or per byte; (5) traffic per device per day.

## What a generator can and cannot settle

- A generator with a reference can replace: household size (done), devices per household,
  presence by hour, kin and tie distance, group membership by distance, buildings.
- It cannot settle the act mix by scope (a product behaviour with no population to read),
  nor the marginal share of a device's power (an engineering measurement of the software).
  Those stay labelled inputs, and the honest output is a range over them.
- So the model can reach "the locality share is X to Y given measured tie distances, for a
  stated act mix" and "the fleet draws A to B watts per household given a measured fleet
  and measured device powers". It cannot reach a single percentage saving.

## The lead's correction, 2026-11-11: power savings is a DELTA from the status quo

In the lead's words: "it is delta from status quo not gross total"; "people use phones
regardless". That is right, and it names what the existing figure does wrong on both sides.

- **The mesh side** should count only what the mesh ADDS: extra wake-ups, radio time and
  relaying on devices people already own and run, plus the whole draw of any device that
  is switched on, or kept on, only for the mesh. The scaling model's "marginal share"
  (phone `0.05`) is a guess at the first; its always-on boxes at share `1.0` assume none
  of them would be on anyway. A home router is on all day already; a node on that
  hardware adds little.
- **The baseline side** should count only what the mesh REMOVES: the data-centre and
  network energy of the workload that moves, not the `415 TWh` of every data-centre
  workload there is.
- So the quantity is: added draw on existing devices + draw of devices added − avoided
  draw upstream, per household or per user. It can be negative or positive, and its sign
  is not known.

**What that changes in the inputs wanted:**
1. Which always-on devices a household ALREADY has (router, set-top box, network storage,
   smart speaker, an old computer left on), not only how many phones.
2. The added energy per event on a phone (a wake-up, a radio tail, a relayed byte), which
   is a measurement of the software on hardware and not a scene field.
3. Hours awake and on which network, since the protocol's fixed cadence (about 127
   wake-ups per device-hour while awake, by the twin's count) multiplies it.
4. The upstream energy attributable to the workload displaced, per user or per byte, with
   its year.

## The twin's corrected order of fields (its code pass, parts 3 to 6)

Neither test can consume most new fields until the twin changes: the locality test reads
only room membership against one of 181 localities, and no energy pass exists.

1. **Traffic share by kind of group, with the distance band of each group's members**
   (household, kin, work or school, neighbourhood, interest, public; bands at household,
   street, tile, city, the twin's locality, region, world). This is the only field that
   moves the locality headline. Its own first-order model: locality ≈ family share × 1 +
   community share × the local share of community groups. If a share `f` of family
   deliveries left the locality the headline falls by about `0.65 f`.
2. **Presence by hour of week:** awake, on which network, charging. Every duty cycle is
   flat today.
3. **The device census by kind with always-on share**, which also has to reconcile three
   censuses in the twin that disagree (2.25 devices per person on the page; 1 phone, 0.5
   laptop and 0.6 home node per household in the fine tier; tier and fleet shares in the
   scaling model).
4. **Uplink class and NAT** (a consumer already exists).
5. **Device power** as one sourced table per class and radio state, not a scene field.
6. **Buildings** last: nothing reads them yet.

Its three least-trusted inputs: room placement and the act mix; the device power table
and server-tier share; the 181-locality partition itself (the largest holds 8.4 % of the
grid's population, so "never leaves the locality" is a sub-continental statement).

## The lead's second correction, 2026-11-12: the scenario is FULL REPLACEMENT

In the lead's words: "the whole assumption is if we replaced the internet entirely with
cewp"; "We do not need massive datacenters"; "or big tech".

**CORRECTION of this session's reading.** After the first correction (a delta from the
status quo) this session added, on its own, an adoption question: what fraction of
platform use a person gives up on installing a private messenger. It sourced a prior for
that (0.005 to 0.16) and concluded that on those priors the mesh costs energy. That
answers a question the theory does not ask. In the theory's scenario substitution is 1 by
definition: the data centres are not partly displaced, they are not needed.

**The two corrections together:**
- Mesh side: only what the mesh ADDS (people use phones and run gateways regardless).
- Baseline side: the WHOLE data-centre estate the replaced internet no longer needs.

**Arithmetic on priors (this session's; the user count is from memory):**
- All data centres worldwide, 2024: 415 TWh over about 5.5 billion internet users is
  about 75 kWh per user per year (8.6 W).
- Added by the mesh: a home node on the gateway a home already runs, near zero; on its
  own 5 W box shared by a household of 2.5, about 17 kWh per person per year; the
  30-second idle round on a carrier network if left as it is, up to about 3.6 kWh a year.
- On that framing the saving is large and the sign is not close.

**What is still open under full replacement:**
1. What must still be computed and stored somewhere. The 415 TWh is every workload.
   Harvesting, ranking and advertising vanish; people's own data, media and the AI they
   use relocate. No operator publishes the split.
2. How efficiently the edge does the work that relocates (a hyperscale centre runs near
   full with 9 % overhead; home hardware is mostly idle and already on).
3. Whether home nodes ride on hardware a home already runs. A separate always-on box in
   each home is about a quarter of the baseline on its own.
4. The networks stay; fixed access is mostly draw per line.
5. Whether "entirely" includes AI, the fastest-growing part of the baseline.

**The adoption-path question** (how much a person gives up on the way there) is a
different question. Its priors stay in `tables/upstream0/` under that label and are not
the theory's test.

## The AI branch, as the twin's session reports the lead's decision (2026-11-12)

Reported by the twin's session, not said in this one: in the full-replacement scenario AI
is a HYBRID. Inference is not moved to people's devices; agents call hosted models through
a router with zero data retention. So inference people ask for stays in data centres at
data-centre efficiency, neither removed nor relocated. What vanishes is retention, and the
training, ranking and advertising work done on harvested data. "All AI on devices" and
"no AI in scope" stay as labelled counterfactuals.

**What that changes in the inputs wanted:**
- Out: the energy of comparable inference on a device against a data centre.
- In: of data-centre AI energy, the share that is inference people ask for against
  ranking, advertising and training on retained data; AI's share of the estate by year;
  and how inference demand per person changes when people run agents (carried by the twin
  as a multiple of 1, 2 or 5, added back on the mesh side).

**The twin's first result under full replacement** (its own, before this change; kWh per
person-year against a baseline of 75.5): with AI left out, a saving of 16.5 at its
defaults and a range of −65 to +111 over its inputs; with AI included and moved to
devices, a cost of 6.2 at its defaults. The mesh's own additions are 2.8. Four guessed
inputs decide the sign; a scout is sourcing them (`SCOUT_ESTATE.md`, when filed).
