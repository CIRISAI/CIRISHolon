#!/usr/bin/env python3
"""Re-apply render-3d's trimer loader + fence onto holon-render.

WHY THIS EXISTS. The lead's landing order puts two refactors ahead of this work, and both
of them rewrite `lib.rs` and `sim.rs` — the two files these edits live in. A line-oriented
patch generated against today's HEAD will not apply after they land. So the edits are
expressed as ANCHORED replacements instead: each one finds a landmark that survives
reformatting, and each one ASSERTS rather than skipping, so a moved anchor is a loud
failure I re-derive by hand instead of a silent half-application.

Idempotent: running it twice is a no-op, so it is safe to re-run after a partial landing.

    python3 reapply.py /home/emoore/CIRISHolon           # apply
    python3 reapply.py /home/emoore/CIRISHolon --check   # report only, change nothing
"""

import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CHECK = "--check" in sys.argv
ROOT = Path(
    next((a for a in sys.argv[1:] if not a.startswith("-")), "/home/emoore/CIRISHolon")
)
CRATE = ROOT / "engine/crates/holon-render"

# The ABI block's boundaries in lib.rs. These two doc lines open and close the region this
# work owns; nothing else in the file carries them.
ABI_OPEN = "/// Open a shipped surface"
ABI_CLOSE = "/// How many shipped surfaces are admitted."

problems: list[str] = []
applied: list[str] = []
already: list[str] = []


def read(p: Path) -> str:
    return p.read_text()


def write(p: Path, s: str) -> None:
    if not CHECK:
        p.write_text(s)


def copy_new_files() -> None:
    """The three files this work owns outright. No merge hazard: nothing else writes them."""
    for rel in (
        "src/trimer_bank.rs",
        "tests/trimer_door.rs",
        "tests/data/s3_h3_4x4x2.json",
    ):
        src = HERE / Path(rel).name
        dst = CRATE / rel
        if not src.exists():
            problems.append(f"stash is missing {src.name}")
            continue
        if dst.exists() and read(dst) == read(src):
            already.append(rel)
            continue
        if not CHECK:
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(src, dst)
        applied.append(rel)


def module_decl() -> None:
    """`pub mod trimer_bank;` — inserted after `pub mod table;` if absent."""
    p = CRATE / "src/lib.rs"
    s = read(p)
    if "pub mod trimer_bank;" in s:
        already.append("lib.rs: module declaration")
        return
    if "pub mod table;" not in s:
        problems.append("lib.rs: no `pub mod table;` to anchor the module declaration to")
        return
    write(p, s.replace("pub mod table;", "pub mod table;\npub mod trimer_bank;", 1))
    applied.append("lib.rs: module declaration")


def abi_block() -> None:
    """Splice this work's ABI block over whatever occupies the same region.

    Takes the block from the stashed `lib.rs` rather than embedding a 200-line string, so
    the stash stays the single source of truth for what is being landed.
    """
    mine = HERE / "lib.rs"
    if not mine.exists():
        problems.append("stash is missing lib.rs; cannot source the ABI block")
        return
    src = read(mine)
    try:
        block = src[src.index(ABI_OPEN) : src.index(ABI_CLOSE)]
    except ValueError:
        problems.append("stash lib.rs no longer contains the ABI block markers")
        return

    p = CRATE / "src/lib.rs"
    s = read(p)
    if block in s:
        already.append("lib.rs: ABI block")
        return
    if ABI_OPEN not in s or ABI_CLOSE not in s:
        problems.append(
            "lib.rs: the ABI block markers are gone — the region was rewritten. "
            "Re-derive by hand; do NOT let this script guess."
        )
        return
    a, b = s.index(ABI_OPEN), s.index(ABI_CLOSE)
    if a > b:
        problems.append("lib.rs: ABI markers are out of order; refusing to splice")
        return
    write(p, s[:a] + block + s[b:])
    applied.append("lib.rs: ABI block")


def refusal_codes() -> None:
    """The five coordinate legs need arms in `trimer_refusal_code`.

    A SEPARATE step because that function lives OUTSIDE the ABI block's markers, above
    them next to `TRIMER_REFUSED`. Splicing the ABI block alone left the match
    non-exhaustive and the crate would not compile — found by running this script against
    a clean HEAD and building the result, which is the only way a re-apply script earns
    any trust.
    """
    p = CRATE / "src/lib.rs"
    s = read(p)
    if "T::SideLengthNotPositive => 18," in s:
        already.append("lib.rs: refusal codes")
        return
    anchor = "T::SurfaceNotLoaded => 11,"
    if anchor not in s:
        problems.append(
            "lib.rs: no `T::SurfaceNotLoaded => 11,` to anchor the coordinate refusal "
            "codes to — the code table was rewritten. Re-derive by hand."
        )
        return
    write(
        p,
        s.replace(
            anchor,
            anchor
            + "\n            T::CoordinatesMissing => 12,"
            + "\n            T::CoordinateCountMismatch => 13,"
            + "\n            T::CoordinatesNotMonotone => 14,"
            + "\n            T::AxisRuleContradictsCoordinates => 15,"
            + "\n            T::EnergyCountMismatch => 16,"
            + "\n            T::AngleCosineOutOfRange => 17,"
            + "\n            T::SideLengthNotPositive => 18,",
            1,
        ),
    )
    applied.append("lib.rs: refusal codes")


def fence_dispatch() -> None:
    """Fence the shipped-surface branch of the three-body dispatch.

    Located by its DISTINGUISHING CALL rather than by its formatting: the branch is
    whatever `if` arm contains `surf.table.eval`, and the replacement runs from the arm's
    `let (a, b, c,` binding to the `} else {` that closes it. That survives rustfmt and
    survives the surrounding refactor; what it does not survive is the branch being
    rewritten, and then it says so.
    """
    p = CRATE / "src/sim.rs"
    s = read(p)
    # Detected by a marker from the fence's own comment, NOT by indentation: this script
    # COMPUTES the indentation it writes, so matching on a hardcoded one reported a
    # rewritten dispatch on a tree the script had just correctly edited. A re-apply script
    # that cries wolf on a correct tree costs more than no script at all.
    if FENCE_MARK in s:
        already.append("sim.rs: dispatch fence")
        return
    if "surf.table.eval" not in s:
        problems.append(
            "sim.rs: no `surf.table.eval` to fence, and the fence is not present either — "
            "the dispatch was rewritten. Re-derive by hand."
        )
        return
    start = s.rfind("let (a, b, c,", 0, s.index("surf.table.eval"))
    end = s.index("} else {", s.index("surf.table.eval"))
    if start < 0:
        problems.append("sim.rs: could not find the branch's binding; refusing to edit")
        return
    indent = " " * (start - s.rfind("\n", 0, start) - 1)
    body = FENCE_BODY.replace("\n", "\n" + indent).rstrip() + "\n" + indent
    write(p, s[:start] + body + s[end:])
    applied.append("sim.rs: dispatch fence")


FENCE_MARK = "A SHIPPED surface exists for this composition"

FENCE_BODY = '''let (a, b, c, v, g, env_abs, env_per_grad) = if self
    .trimers
    .find([za, zb, zc])
    .is_some()
{
    // A SHIPPED surface exists for this composition and is deliberately NOT
    // evaluated. It is fenced and counted, which is what the fence counter is for.
    //
    // This branch used to call `TrimerTable::eval` on it. That was wrong in two
    // independent ways, both found by putting a REAL artifact next to the code
    // rather than the schema's example:
    //
    //   1. GRID RULE. `TrimerTable` is this build's 33x33x13 grid with
    //      `r_of_tau`'s STRETCH_A = 2.0 spacing. `s3_tables` emits UNIFORM-LINEAR
    //      spacing on an arbitrary grid -- the first real artifact is 4x4x2.
    //      Interpolating uniform data on stretched axes is smooth, plausible, and
    //      wrong everywhere except the boundary, which is the exact failure
    //      `load_water_table` refuses by construction.
    //   2. COORDINATES. `eval` takes three SIDE LENGTHS; the artifact's axes are
    //      (x, y, u) with `u` an angle-like coordinate. Even on a matching grid
    //      these would not be the same quantities.
    //
    // So a shipped surface is admitted, stored and READABLE -- the fence in
    // `holon_trimer_h_only` lifts off it -- and it is not integrated until an
    // evaluator exists for the geometry the artifact actually ships. Fencing costs
    // a counted truncation; the alternative costs a wrong force nobody would see.
    //
    // A DEFAULT TripleTerm is `live: false`, which is this function's own way of
    // saying "no server for this triple" -- the same exit the untabulated
    // compositions take below. `served` is the other half: it must NOT report a
    // shipped surface as served, or the census would book these as covered
    // rather than fenced and the truncation would stop being counted.
    return TripleTerm::default();
'''


def served_not_shipped() -> None:
    """`served` must NOT report a shipped surface as served while the dispatch fences it.

    The other half of the fence, and the half that keeps the census honest: if `served`
    says yes, every fenced triple is booked as covered and the truncation disappears from
    the count. Found by re-deriving the fence against T3's refactor and asking where the
    counter had gone — the old fence incremented `fence_untabulated` itself, and the new
    `triple_term` takes `&self` and cannot.
    """
    p = CRATE / "src/sim.rs"
    s = read(p)
    if "A SHIPPED SURFACE IS NOT SERVED" in s:
        already.append("sim.rs: served() fence")
        return
    anchor = (
        "    fn served(&self, z: [u8; 3]) -> bool {\n"
        "        if self.trimers.find(z).is_some() {\n"
        "            return true;\n"
        "        }\n"
    )
    if anchor not in s:
        problems.append(
            "sim.rs: `served`'s shipped-surface branch is not where it was. Re-derive by "
            "hand: a shipped surface must not count as served while the dispatch fences it."
        )
        return
    write(
        p,
        s.replace(
            anchor,
            "    fn served(&self, z: [u8; 3]) -> bool {\n"
            "        // A SHIPPED SURFACE IS NOT SERVED. It is admitted, stored and readable, and the\n"
            "        // three-body dispatch deliberately fences it (see `triple_term`) until an\n"
            "        // evaluator exists for the geometry `s3_tables` emits. Reporting it as served here\n"
            "        // would book every such triple as covered, and the truncation the fence creates\n"
            "        // would vanish from the census -- which is the one thing the census exists to\n"
            "        // prevent. This goes back to `true` in the same change that lands the evaluator.\n",
            1,
        ),
    )
    applied.append("sim.rs: served() fence")


def keep_helper() -> None:
    """`match_triple_slots` goes unused once the branch is fenced; keep it with its reason."""
    p = CRATE / "src/sim.rs"
    s = read(p)
    if "fn match_triple_slots(" not in s:
        problems.append("sim.rs: `match_triple_slots` is gone; nothing to keep")
        return
    if "#[allow(dead_code)]\nfn match_triple_slots(" in s:
        already.append("sim.rs: match_triple_slots kept")
        return
    write(
        p,
        s.replace(
            "fn match_triple_slots(",
            "// KEPT THOUGH UNCALLED: this is the half of the fenced shipped-surface branch\n"
            "// that was RIGHT — a surface declares its species in its own order, and the\n"
            "// force loop's i, j, k need permuting to match. The evaluator that is owed will\n"
            "// need it. Deleting it would throw away the correct part of a branch removed for\n"
            "// its incorrect part.\n"
            "#[allow(dead_code)]\nfn match_triple_slots(",
            1,
        ),
    )
    applied.append("sim.rs: match_triple_slots kept")


def main() -> int:
    if not CRATE.is_dir():
        print(f"no holon-render at {CRATE}")
        return 2
    copy_new_files()
    module_decl()
    abi_block()
    refusal_codes()
    fence_dispatch()
    served_not_shipped()
    keep_helper()

    verb = "would apply" if CHECK else "applied"
    for label, items in (("already present", already), (verb, applied)):
        for i in items:
            print(f"  {label:15}  {i}")
    for pr in problems:
        print(f"  PROBLEM          {pr}")
    if problems:
        print("\nRefused to guess on the above. Re-derive those by hand.")
        return 1
    print("\nAll edits accounted for. Now run:")
    print("  cargo test -q -p holon-render --release")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
