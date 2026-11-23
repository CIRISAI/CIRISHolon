# The screen standard's registry, vendored

`ui_primitives.json` is CSD/5 draft 3's registry of UX primitives, copied byte for byte from
CIRISAI/CIRISConstitution `manifests/ui_primitives.json` at tag `v1.0-rc21-wire`
(commit `c8afde40fe4d334078b2c5e7c816a11b64327fa0`). It is the same file as at `181c48a`, the
commit `INTERFACE.md` §9 was first generated against. It has 24 primitives and 60 slots.

The registry says to vendor it this way ("a client vendors this file at a pinned commit of this
repository and checks its screens against it"). Both repositories are AGPL-3.0.

SHA-256: `6e8d058c14083cc8d26210d29e5bb0c93989b11306fd80a2c2a81f5adfc1510d`

`../tools/check_registry.py` checks `INTERFACE.md` §9 against this file. It refuses if the file's
hash is not the one above, so a re-vendor has to change both together.
