#!/usr/bin/env python3
"""The audit is a gate, and gates prove they can fire.

`prereg_audit.py` refuses a freeze that CONTACTS a registered misfit's shape without
citing it. That refusal had never been demonstrated for a single keyword — the table was
believed, not tested — and on 2026-08-30 the registry had grown five ids past the table
entirely, so those entries were decoration.

This file demonstrates the refusal **for every entry in `CONTACT`, derived from `CONTACT`
itself**. That derivation is the point: a future keyword added to the table without a
failing case is impossible, because the case is generated from the table. Adding a
keyword automatically adds its test.

Each keyword gets the split pair the mutation discipline requires:

  * a synthetic freeze that CONTACTS the shape and does NOT cite the id  -> must be REFUSED
  * the same freeze WITH the citation                                     -> must be ADMITTED

Without the second half, "refuses everything" would pass. Without the first, the gate
could be a no-op.

Run: python3 Audit/test_prereg_audit.py
"""
import pathlib
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import prereg_audit as A  # noqa: E402


def skeleton(body: str, cite: str = "") -> str:
    """A minimal ADMISSIBLE freeze, with `body` spliced in.

    It has to clear every OTHER check or a refusal would be ambiguous — that is the whole
    difficulty of testing one rule inside a gate that applies several. Gate rows carry a
    digit; the witness is `none` with a reason on the same line; a `misfits:` line exists.
    """
    return f"""# Synthetic freeze

misfits: contacts nothing else {cite}

- **G1 — a gate with a numeric criterion**: the measured value is within 1e-3.
  witness: none (measured, no Lean object)

{body}
"""


def run(text: str):
    with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as f:
        f.write(text)
        path = f.name
    try:
        return A.audit(path)
    finally:
        pathlib.Path(path).unlink(missing_ok=True)


def main() -> int:
    failures = []

    # ---- 0. The table must not name an id the registry lacks. A typo here disables that
    #         keyword's gate forever and no freeze could ever reveal it.
    unknown = A.contact_table_is_sound()
    if unknown:
        failures.append(f"CONTACT maps to unregistered id(s): {unknown}")

    # ---- 1. The skeleton itself must be ADMITTED, or every refusal below is ambiguous.
    base = run(skeleton("Nothing here contacts a registered shape."))
    if base:
        failures.append(f"the empty skeleton was refused, so no test below isolates anything: {base}")

    # ---- 2. THE SPLIT PAIR, for every keyword in the table.
    checked = 0
    for kw, mid in sorted(A.CONTACT.items()):
        body = f"The design's approach to {kw} is described here at length."

        uncited = run(skeleton(body))
        hit = [e for e in uncited if "keyword contact" in e and mid in e]
        if not hit:
            failures.append(
                f"NOT REFUSED: a freeze contacting '{kw}' without citing {mid} was admitted "
                f"— that entry is decoration"
            )

        cited = run(skeleton(body, cite=mid))
        still = [e for e in cited if "keyword contact" in e and mid in e]
        if still:
            failures.append(
                f"REFUSED ANYWAY: citing {mid} did not clear the '{kw}' contact — the gate "
                f"cannot be satisfied, which is a gate people switch off"
            )
        checked += 1

    # ---- 3. M-VACUOUS-SUCCESS: this file must have done work proportional to the table.
    if checked != len(A.CONTACT):
        failures.append(f"checked {checked} keywords against a table of {len(A.CONTACT)}")
    if checked == 0:
        failures.append("the contact table is empty, so this suite proved nothing")

    # ---- 4. The five ids registered on 2026-08-30, named explicitly. The loop above
    #         covers them, but naming them makes their absence a RED TEST rather than a
    #         silently smaller loop — which is exactly how they went missing in the first
    #         place.
    owed = {
        "M-DEVICE-CLASS",
        "M-PROBE-THE-RESOURCE",
        "M-PROVENANCE-OVERREACH",
        "M-MAINTENANCE-LENS",
        "M-IDLE-CALIBRATED-TIMEOUT",
    }
    missing = sorted(owed - set(A.CONTACT.values()))
    if missing:
        failures.append(f"registered but NOT grep-armed: {missing}")

    # ---- 5. Every registry id should eventually be reachable. Reported, not enforced:
    #         some misfits are about process rather than about a word a freeze would use,
    #         and forcing a keyword for those would manufacture false refusals.
    unarmed = sorted(A.REG_IDS - set(A.CONTACT.values()))

    print(f"contact keywords checked: {checked}  (ids armed: {len(set(A.CONTACT.values()))})")
    print(f"registry ids with no contact keyword: {len(unarmed)}")
    for u in unarmed:
        print(f"    unarmed: {u}")

    if failures:
        print("\nFAILED")
        for f in failures:
            print(f"    - {f}")
        return 1
    print("\nPASSED: every contact keyword refuses uncited and admits cited")
    return 0


if __name__ == "__main__":
    sys.exit(main())
