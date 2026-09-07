# LIQUID-2's size check — and why one box has a log and no record

Four boxes, one process each, at the state point's density on an `n^3` body-centred lattice
carrying `2 n^3` waters. One process per box on purpose: the largest one is killed by the
operating system, and run together it would have taken the other three records with it.

| file | waters | verdict |
|---|---|---|
| `cells3.json` | 54 | door REFUSED by the image rule: the half-edge `11.0976` bohr is under the law's reach `14.0000` by `2.9024` |
| `cells4.json` | 128 | ADMITTED — the campaign's own box, measured here again so every other size is read against a number this phase took itself |
| `cells5.json` | 250 | ADMITTED and priced |
| `cells6.log` | 432 | **no record, and that is the reading**: the process is killed with exit `137` (SIGKILL, the OOM killer) while building the box, BEFORE its door runs, on three independent attempts with 25 GB free |

The 432 fence is measured and not estimated. Peak resident set, by `/usr/bin/time -v` on this
host: `0.929` GiB at 384 atoms and `6.845` GiB at 750 — a factor of `7.37` for an atom ratio
of `1.953`, whose cube is `7.45`. The periodic three-body enumeration is cubic in the atom
count, confirmed to 1 % at two points; carried to 1,296 atoms that is `38.4 x 0.929 = 35.7`
GiB against this host's `31.8`. It is the ENUMERATION that is cubic, not the physics, so the
fence's owner is whoever gives the seam a three-body neighbour list and its exit is a 432-water
box that builds, here or on a larger host.

`LIQUID2_PREREG.md` section 3a carries all of this with the campaign's decision: 128 waters is
the smallest periodic box CT-3's law admits at all, and 250 waters is pre-committed as a
one-seed size check inside the price ceiling.
