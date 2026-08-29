"""The pool guard, with the failure it prevents demonstrated first.

scipy's ArpackNoConvergence cannot be reconstructed by the unpickler.  Raised
inside a Pool worker it kills the result-handler THREAD while every worker
process stays up, so the job keeps its process count, keeps its CPU, and never
returns another result.  This campaign lost eighty minutes of N2/CO grid to
exactly that, twice.

Case 1 runs the unguarded path in a subprocess and shows it HANGS.
Case 2 runs the guarded path and shows it raises a plain RuntimeError.
A guard whose failing case has never been produced is indistinguishable from
a guard that cannot fire, so case 1 is not decoration.
"""
import os
import subprocess
import sys
import textwrap

HERE = os.path.dirname(os.path.abspath(__file__))

_UNGUARDED = textwrap.dedent("""
    from multiprocessing import Pool
    from scipy.sparse.linalg import ArpackNoConvergence
    import numpy as np

    def boom(x):
        raise ArpackNoConvergence("no", np.zeros(0), np.zeros((0, 0)))

    if __name__ == "__main__":
        with Pool(2) as p:
            print(p.map(boom, [1, 2]))
""")

_GUARDED = textwrap.dedent("""
    import sys
    sys.path.insert(0, %r)
    import build_curves as B
    from scipy.sparse.linalg import ArpackNoConvergence
    import numpy as np

    def boom(x):
        raise ArpackNoConvergence("no", np.zeros(0), np.zeros((0, 0)))

    if __name__ == "__main__":
        try:
            B.pmap(boom, [1, 2], 2)
        except RuntimeError as e:
            print("RUNTIMEERROR", str(e)[:60])
            sys.exit(0)
        print("NO EXCEPTION AT ALL")
        sys.exit(1)
""") % HERE


def _run(src, timeout):
    pth = os.path.join(HERE, "_tmp_pool_case.py")
    with open(pth, "w") as f:
        f.write(src)
    try:
        return subprocess.run([sys.executable, pth], timeout=timeout,
                              capture_output=True, text=True)
    finally:
        os.remove(pth)


def test_unguarded_pool_hangs():
    try:
        r = _run(_UNGUARDED, 25)
    except subprocess.TimeoutExpired:
        print("  unguarded pool HUNG (the failure is real and reproducible)")
        return
    raise AssertionError(
        "the unguarded pool did NOT hang -- this guard may be guarding "
        "nothing on this scipy/python: rc=%s out=%r err=%r"
        % (r.returncode, r.stdout[-200:], r.stderr[-200:]))


def test_guarded_pool_raises():
    r = _run(_GUARDED, 60)
    assert r.returncode == 0 and "RUNTIMEERROR" in r.stdout, \
        "guarded pool did not surface a plain error: rc=%s out=%r err=%r" \
        % (r.returncode, r.stdout[-300:], r.stderr[-400:])
    print("  guarded pool raised a plain RuntimeError:",
          r.stdout.strip().split("\n")[-1][:70])


def test_every_pooled_call_is_wrapped():
    """The registry version protected only the three workers someone had
    remembered to register; assert the wrapping is unconditional."""
    src = open(os.path.join(HERE, "build_curves.py")).read()
    body = src.split("def pmap(", 1)[1].split("\ndef ", 1)[0]
    assert "_Safe(" in body, "pmap no longer wraps its callable"
    assert "p.map(_Safe(" in body, body
    print("  pmap wraps unconditionally (no registry to forget)")


if __name__ == "__main__":
    for f in (test_unguarded_pool_hangs, test_guarded_pool_raises,
              test_every_pooled_call_is_wrapped):
        f()
    print("test_pmap_safety: 3 passed")
