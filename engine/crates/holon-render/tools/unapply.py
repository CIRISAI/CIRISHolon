#!/usr/bin/env python3
"""Back render-3d's trimer edits OUT of holon-render, leaving every other lane's alone.

WHY NOT `git checkout -- lib.rs sim.rs`. Both files carry ANOTHER lane's uncommitted work
at the same time as mine: the cells lane's `pub mod cells;` in lib.rs and roughly a
thousand refactor lines in sim.rs. Reverting either file wholesale would destroy that. So
this restores HEAD's content for MY REGIONS ONLY and leaves the rest of the working tree
exactly as it stands.

It is the exact inverse of `reapply.py` and shares its discipline: anchored, asserting,
idempotent, with a `--check` mode. The round-trip is verified rather than assumed — apply
then unapply on a clean HEAD must produce a tree identical to HEAD.

    python3 unapply.py /home/emoore/CIRISHolon --check
    python3 unapply.py /home/emoore/CIRISHolon
"""

import subprocess
import sys
from pathlib import Path

CHECK = "--check" in sys.argv
ROOT = Path(
    next((a for a in sys.argv[1:] if not a.startswith("-")), "/home/emoore/CIRISHolon")
)
CRATE = ROOT / "engine/crates/holon-render"
REL = "engine/crates/holon-render"

ABI_OPEN = "/// Open a shipped surface"
ABI_CLOSE = "/// How many shipped surfaces are admitted."
FENCE_MARK = "A SHIPPED surface exists for this composition"
KEEP_MARK = "// KEPT THOUGH UNCALLED: this is the half of the fenced shipped-surface branch"

problems: list[str] = []
done: list[str] = []
already: list[str] = []


def head_text(rel: str) -> str | None:
    r = subprocess.run(
        ["git", "show", f"HEAD:{REL}/{rel}"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    return r.stdout if r.returncode == 0 else None


def write(p: Path, s: str) -> None:
    if not CHECK:
        p.write_text(s)


def region(s: str, open_mark: str, close_mark: str) -> str | None:
    if open_mark not in s or close_mark not in s:
        return None
    a, b = s.index(open_mark), s.index(close_mark)
    return s[a:b] if a < b else None


def restore_abi() -> None:
    """Put HEAD's version of the ABI block back."""
    p = CRATE / "src/lib.rs"
    cur = p.read_text()
    head = head_text("src/lib.rs")
    if head is None:
        problems.append("cannot read HEAD's lib.rs")
        return
    theirs = region(head, ABI_OPEN, ABI_CLOSE)
    mine = region(cur, ABI_OPEN, ABI_CLOSE)
    if theirs is None:
        problems.append("HEAD's lib.rs has no ABI block to restore")
        return
    if mine is None:
        problems.append("working lib.rs has no ABI block markers; refusing to guess")
        return
    if mine == theirs:
        already.append("lib.rs: ABI block")
        return
    a = cur.index(ABI_OPEN)
    b = cur.index(ABI_CLOSE)
    write(p, cur[:a] + theirs + cur[b:])
    done.append("lib.rs: ABI block")


def restore_refusal_codes() -> None:
    """Drop the five coordinate arms this work added to `trimer_refusal_code`."""
    p = CRATE / "src/lib.rs"
    cur = p.read_text()
    arms = (
        "\n            T::CoordinatesMissing => 12,"
        "\n            T::CoordinateCountMismatch => 13,"
        "\n            T::CoordinatesNotMonotone => 14,"
        "\n            T::AxisRuleContradictsCoordinates => 15,"
        "\n            T::EnergyCountMismatch => 16,"
    )
    if arms not in cur:
        already.append("lib.rs: refusal codes")
        return
    write(p, cur.replace(arms, "", 1))
    done.append("lib.rs: refusal codes")


def restore_dispatch() -> None:
    """Put HEAD's shipped-surface branch back in place of the fence."""
    p = CRATE / "src/sim.rs"
    cur = p.read_text()
    if FENCE_MARK not in cur:
        already.append("sim.rs: dispatch fence")
        return
    head = head_text("src/sim.rs")
    if head is None:
        problems.append("cannot read HEAD's sim.rs")
        return
    if "surf.table.eval" not in head:
        problems.append(
            "HEAD's sim.rs has no `surf.table.eval` branch to restore — HEAD moved past "
            "it. Restore by hand, or leave the fence in place and tell the lead."
        )
        return
    # HEAD's branch: from its `let (a, b, c,` binding to the `} else {` that closes it.
    h_start = head.rfind("let (a, b, c,", 0, head.index("surf.table.eval"))
    h_end = head.index("} else {", head.index("surf.table.eval"))
    theirs = head[h_start:h_end]
    # Mine: same span, located by the fence marker instead.
    c_start = cur.rfind("let (a, b, c,", 0, cur.index(FENCE_MARK))
    c_end = cur.index("} else {", cur.index(FENCE_MARK))
    if c_start < 0:
        problems.append("sim.rs: could not bound the fence; refusing to edit")
        return
    write(p, cur[:c_start] + theirs + cur[c_end:])
    done.append("sim.rs: dispatch fence")


def restore_helper() -> None:
    """Remove whatever this work put between `#[inline]` and `fn match_triple_slots(`.

    Anchored on the SPAN rather than on a comment string, because two different wordings
    of that note exist: the one this script's sibling writes and the one I typed by hand
    into the shared tree. Matching either literal reported "already at HEAD" on a tree that
    still carried the other — a false all-clear that would have left my residue behind in
    a back-out whose whole purpose is to leave none.
    """
    p = CRATE / "src/sim.rs"
    cur = p.read_text()
    if "fn match_triple_slots(" not in cur:
        already.append("sim.rs: match_triple_slots (absent)")
        return
    at = cur.index("fn match_triple_slots(")
    tag = "#[inline]\n"
    inline = cur.rfind(tag, 0, at)
    if inline < 0:
        problems.append("sim.rs: no `#[inline]` above match_triple_slots to anchor on")
        return
    head_of_span = inline + len(tag)
    between = cur[head_of_span:at]
    if "KEPT THOUGH" not in between:
        already.append("sim.rs: match_triple_slots")
        return
    write(p, cur[:head_of_span] + cur[at:])
    done.append("sim.rs: match_triple_slots")


def restore_owned_files() -> None:
    """Return this work's own files to HEAD, and drop the untracked artifact.

    They carry no other lane's work, but they must go back anyway: HEAD's `lib.rs` calls
    the v1 `trimer_bank` API, so leaving the v2 module behind would hand t3-engine a tree
    that does not compile — the opposite of landing against clean files.
    """
    for rel in ("src/trimer_bank.rs", "tests/trimer_door.rs"):
        head = head_text(rel)
        p = CRATE / rel
        if head is None:
            problems.append(f"cannot read HEAD's {rel}")
            continue
        if p.read_text() == head:
            already.append(rel)
            continue
        write(p, head)
        done.append(rel)
    artifact = CRATE / "tests/data/s3_h3_4x4x2.json"
    if artifact.exists():
        if not CHECK:
            artifact.unlink()
            try:
                artifact.parent.rmdir()
            except OSError:
                pass  # other data files live there; leave the directory
        done.append("tests/data/s3_h3_4x4x2.json (removed; it is in the stash)")
    else:
        already.append("tests/data/s3_h3_4x4x2.json")


def main() -> int:
    if not CRATE.is_dir():
        print(f"no holon-render at {CRATE}")
        return 2
    restore_abi()
    restore_refusal_codes()
    restore_dispatch()
    restore_helper()
    restore_owned_files()

    verb = "would restore" if CHECK else "restored"
    for label, items in (("already at HEAD", already), (verb, done)):
        for i in items:
            print(f"  {label:16}  {i}")
    for pr in problems:
        print(f"  PROBLEM           {pr}")
    if problems:
        print("\nRefused to guess on the above. Resolve by hand.")
        return 1
    print("\nBacked out. Other lanes' uncommitted work in these files is untouched.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
