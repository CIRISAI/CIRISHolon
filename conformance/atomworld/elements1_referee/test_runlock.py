"""The run lock, with its failing case demonstrated.

A gate that has never fired is indistinguishable from a gate that cannot, so
every case here either fires the refusal or proves the refusal stays silent
where it should.  The duplicate-pool defect this guards against is real and
recorded: two identical pools over the same six species ran for three hours,
each healthy, together holding 28 of 32 cores.
"""
import json
import os
import shutil
import subprocess
import sys
import tempfile

import build_curves as B


def _fresh_lockdir():
    d = tempfile.mkdtemp(prefix="locks_test_")
    B.LOCKDIR = d
    del B._HELD[:]
    return d


def _live_helper():
    """A real live process to own a lock -- not a fabricated pid."""
    return subprocess.Popen([sys.executable, "-c",
                             "import time; time.sleep(600)"])


def test_duplicate_is_refused():
    d = _fresh_lockdir()
    holder = _live_helper()
    try:
        with open(os.path.join(d, "Li2.stencil.lock"), "w") as f:
            json.dump(dict(pid=holder.pid, started="now",
                           argv=["build_curves.py", "--stencil", "Li2"],
                           species="Li2", stage="--stencil"), f)
        fired = False
        try:
            B.acquire_run_locks(["--stencil"], ["Li2"])
        except SystemExit as e:
            fired, code = True, e.code
        assert fired, "THE REFUSAL DID NOT FIRE on a live duplicate"
        assert code == 3, code
    finally:
        holder.kill(), holder.wait(), shutil.rmtree(d, ignore_errors=True)
    print("  duplicate refused (the gate fires)")


def test_dead_holder_is_taken_over():
    d = _fresh_lockdir()
    holder = _live_helper()
    pid = holder.pid
    holder.kill()
    holder.wait()
    try:
        with open(os.path.join(d, "Li2.stencil.lock"), "w") as f:
            json.dump(dict(pid=pid, started="then", argv=[], species="Li2",
                           stage="--stencil"), f)
        held = B.acquire_run_locks(["--stencil"], ["Li2"])
        assert len(held) == 1
        assert B.read_lock(held[0])["pid"] == os.getpid()
        B.release_run_locks()
        assert not os.path.exists(held[0])
    finally:
        shutil.rmtree(d, ignore_errors=True)
    print("  dead holder taken over, lock released on exit")


def test_disjoint_work_is_not_blocked():
    d = _fresh_lockdir()
    holder = _live_helper()
    try:
        with open(os.path.join(d, "Li2.stencil.lock"), "w") as f:
            json.dump(dict(pid=holder.pid, started="now", argv=[],
                           species="Li2", stage="--stencil"), f)
        B.acquire_run_locks(["--stencil"], ["N2"])       # other species
        B.acquire_run_locks(["--spin"], ["Li2"])         # other stage
        B.release_run_locks()
    finally:
        holder.kill(), holder.wait(), shutil.rmtree(d, ignore_errors=True)
    print("  disjoint species/stage not blocked (the gate is not a blanket)")


def test_override_is_available():
    d = _fresh_lockdir()
    holder = _live_helper()
    try:
        with open(os.path.join(d, "Li2.stencil.lock"), "w") as f:
            json.dump(dict(pid=holder.pid, started="now", argv=[],
                           species="Li2", stage="--stencil"), f)
        os.environ["ALLOW_DUPLICATE_RUN"] = "1"
        B.acquire_run_locks(["--stencil"], ["Li2"])
        B.release_run_locks()
    finally:
        os.environ.pop("ALLOW_DUPLICATE_RUN", None)
        holder.kill(), holder.wait(), shutil.rmtree(d, ignore_errors=True)
    print("  documented override works")


def test_partial_assembly_merges_instead_of_narrowing():
    """The failing case, stated as the pre-fix behaviour: a run over ONE
    species must not delete the others from the accumulated partial file."""
    prev = dict(species={"H2": {"E": 1}, "LiH": {"E": 2}},
                incomplete_species={"He2": "no cache"})
    out = dict(model="x")
    B.merge_partial(prev, out, {"He2": {"E": 3}}, {})
    assert sorted(out["species"]) == ["H2", "He2", "LiH"], out["species"]
    assert "incomplete_species" not in out, out
    narrowing = dict(species={"He2": {"E": 3}})
    assert sorted(narrowing["species"]) == ["He2"], "control"
    print("  partial assembly merges (the narrowing case is the one that bit)")


if __name__ == "__main__":
    for f in (test_duplicate_is_refused, test_dead_holder_is_taken_over,
              test_disjoint_work_is_not_blocked, test_override_is_available,
              test_partial_assembly_merges_instead_of_narrowing):
        f()
    print("test_runlock: 5 passed")
