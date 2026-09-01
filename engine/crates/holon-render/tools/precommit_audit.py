#!/usr/bin/env python3
"""Account for every hunk in the files this lane is about to commit.

THE RULE THIS ENFORCES (lead, 2026-09-01, after four within-file sweeps in one day):
a pathspec fences ACROSS files, not WITHIN one. `git commit -- <file>` takes the whole
WORKTREE file, including a sibling lane's in-flight hunks sitting in it. So before
committing, every hunk in every named file has to be accounted for as this lane's.

HOW IT ANSWERS THAT EXACTLY, RATHER THAN GUESSING. The first version of this tool
classified a hunk as mine if its added lines carried one of a list of markers. That is a
heuristic and it failed its positive control on the first run: a comment rewording, a
removed `use`, a field's type change are all mine and carry no marker, so it refused
twelve of my own hunks and would have refused everything at landing. A check that always
says no is worth exactly as much as one that always says yes.

So it does not guess. It builds a PRISTINE worktree at the same HEAD, re-applies this
lane's work there, and takes `git diff HEAD -- <path>` from it. That diff IS this lane's
change, by construction — nothing else has ever touched that worktree. Comparing it to the
same diff taken from the shared tree answers the question exactly: identical means the
shared file holds this lane's work and nothing else; any difference IS the foreign
content, and it gets printed.

    python3 precommit_audit.py <repo> <path> [<path> ...]

WHEN IT APPLIES, AND WHEN IT CANNOT. It answers "is this file's change mine alone" by
re-applying a HELD change onto a pristine worktree and comparing. That reference only
exists while the change is held OUT of the tree — during a landing-order hold, with the
held files sitting beside `reapply.py`. Once the work is landed, the reference IS HEAD and
there is nothing to re-apply: run it here and `reapply.py` correctly reports that it
cannot find the held sources, which reads like a failure and is not one.

For an incremental edit on top of a landed change, the check is the direct one instead:
`git diff HEAD -- <path>` and read every hunk, which is what the rule asked for in the
first place. This tool earns its place on the case a person cannot do reliably by eye — a
multi-file change held across other lanes' landings — not on a one-file edit.

Exit 0 = safe to `git commit -- <those paths>`. Exit 1 = at least one file holds content
this lane did not put there; the paths to leave out are named.
"""

import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent


def git(cwd: Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True)


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    repo = Path(sys.argv[1]).resolve()
    paths = sys.argv[2:]

    # The shared index first: staged content is invisible to `git diff` and would ride
    # along on any commit. Another lane's staged work is never this lane's to carry.
    staged = git(repo, "diff", "--cached", "--stat").stdout.strip()
    if staged:
        print("STAGED CONTENT IN THE SHARED INDEX — leave all of it alone:")
        for line in staged.splitlines():
            print(f"    {line}")
        print()

    head = git(repo, "rev-parse", "HEAD").stdout.strip()
    verdict = 0
    with tempfile.TemporaryDirectory() as tmp:
        clean = Path(tmp) / "pristine"
        add = git(repo, "worktree", "add", "--detach", str(clean), head)
        if add.returncode != 0:
            print(f"could not make a pristine worktree at {head[:9]}:\n{add.stderr}")
            return 2
        try:
            ra = subprocess.run(
                [sys.executable, str(HERE / "reapply.py"), str(clean)],
                capture_output=True,
                text=True,
            )
            if ra.returncode != 0:
                print("re-apply refused on a pristine HEAD, so there is no reference to")
                print("compare against. Resolve that first:\n")
                print(ra.stdout)
                return 2

            for path in paths:
                mine = git(clean, "diff", "HEAD", "--", path).stdout
                actual = git(repo, "diff", "HEAD", "--", path).stdout
                if mine == actual:
                    n = mine.count("\n@@") + mine.count("@@", 0, 3)
                    print(f"  OK      {path}: identical to this lane's change alone")
                    continue
                verdict = 1
                mine_lines = set(mine.splitlines())
                extra = [
                    l
                    for l in actual.splitlines()
                    if l.startswith(("+", "-"))
                    and not l.startswith(("+++", "---"))
                    and l not in mine_lines
                ]
                print(f"  REFUSE  {path}: holds content this lane did not put there")
                for line in extra[:8]:
                    print(f"            {line[:96]}")
                if len(extra) > 8:
                    print(f"            ... {len(extra) - 8} more foreign lines")
        finally:
            git(repo, "worktree", "remove", "--force", str(clean))
            git(repo, "worktree", "prune")

    print()
    if verdict:
        print(
            "Do NOT `git commit -- <path>` the REFUSED files: a pathspec takes the whole\n"
            "worktree file, foreign content included. Leave those paths out and hand the\n"
            "lead the hunks, or ask for the private-index recipe."
        )
    else:
        print("Every named file holds this lane's change and nothing else. Pathspec is safe.")
    return verdict


if __name__ == "__main__":
    raise SystemExit(main())
