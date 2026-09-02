#!/usr/bin/env python3
"""
mutate_tcap.py -- plant defects in the tier walls and confirm tests/refusal.rs
refuses to stay green.

A gate that cannot fail on a planted defect is not a gate.  The battlerig found
the original defect by running into it (upstream/BATTLERIG.md caveat 1: the
`amp` path had no T-cap guard and enumerated 2^28 branches into a TIMEOUT
instead of refusing by name).  Enumerating the doors turned up two siblings of
the same shape.  This script plants the removal of each guard, one at a time,
and requires the NAMED test that holds that door to go red.

Two failure modes are held apart, because they are not the same news:

  UNDETECTED -- the mutant built and every test stayed green.  The gate is
                theatre and the exit code says so.
  INVALID    -- the mutant did not build, so nothing was tested.  A mutation
                that cannot compile has not been observed to be caught; it has
                only been observed to be a syntax error.  (Three of seven
                mutations in a prior campaign stayed silent for reasons that
                had nothing to do with the check under test.)

Sources are edited IN PLACE and restored in a `finally`, with the pre-mutation
bytes hashed and the restoration verified against that hash.  The script
refuses to start if any target string is missing -- that means a guard moved
and the mutation is stale, which is a different problem than a gate that does
not fire.

Usage:
    ./mutate_tcap.py              run every mutation
    ./mutate_tcap.py --dry-run    show the sites and the expectations, edit nothing
    ./mutate_tcap.py --only M2    run one mutation

Environment:
    HOLON_ENGINE   engine root (default: resolved from this script's location)
    CARGO          cargo binary (default: "cargo")
    MUTANT_MEM_GB  address-space cap for the test process (default: 8)
"""
import argparse
import hashlib
import os
import re
import resource
import signal
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ENGINE = os.environ.get(
    "HOLON_ENGINE", os.path.abspath(os.path.join(HERE, "..", "..", "engine"))
)
CARGO = os.environ.get("CARGO", "cargo")
MEM_GB = int(os.environ.get("MUTANT_MEM_GB", "8"))
# A mutant that removed a T guard does not fail fast -- it ENUMERATES, which is
# the original defect's exact signature (a TIMEOUT where a refusal belonged).
# Waiting it out is not a test strategy, so a mutant that runs past this is
# recorded as caught, by the same evidence the battlerig had.
TEST_TIMEOUT = int(os.environ.get("MUTANT_TIMEOUT_S", "30"))

QASM = os.path.join(ENGINE, "crates", "holon-qasm")
LIB = os.path.join(QASM, "src", "lib.rs")
MAGIC = os.path.join(QASM, "src", "magic.rs")

# An unguarded carrier asks for 2^n amplitudes.  The cap keeps a mutant that
# gets past its guard from thrashing the machine instead of failing fast; it is
# comfortably above what building and running this crate needs.
LIMIT = MEM_GB * 1024**3


# (id, description, file, old, new, tests that MUST go red)
MUTATIONS = [
    (
        "M1",
        "the T budget predicate always passes (both amp doors at once)",
        LIB,
        "    let t = t_count(c);\n    if t <= T_MAX_MAGIC {\n        return Ok(());\n    }",
        "    let t = t_count(c);\n    if true {\n        return Ok(());\n    }",
        ["amp_path_refuses_past_the_t_cap", "amp_library_call_refuses_past_the_t_cap"],
    ),
    (
        "M2",
        "the ORIGINAL defect: magic_amplitude's guard removed",
        MAGIC,
        "    if let Err(reason) = crate::check_t_budget(c) {\n        panic!(\"{reason}\");\n    }\n    let t_count = crate::t_count(c);\n    let mut acc = Cyc::ZERO;",
        "    let t_count = crate::t_count(c);\n    let mut acc = Cyc::ZERO;",
        ["amp_library_call_refuses_past_the_t_cap"],
    ),
    (
        "M3",
        "the checked amp wrapper stops checking",
        MAGIC,
        "    crate::check_t_budget(c)?;\n    Ok(magic_amplitude(c, y, drop_s_cross, wrong_gauss))",
        "    Ok(magic_amplitude(c, y, drop_s_cross, wrong_gauss))",
        ["amp_path_refuses_past_the_t_cap"],
    ),
    (
        "M4",
        "the 2^n budget predicate always passes",
        LIB,
        "    let n = c.n_qubits;\n    if n <= N_MAX_STATEVECTOR {\n        return Ok(());\n    }",
        "    let n = c.n_qubits;\n    if true {\n        return Ok(());\n    }",
        [
            "distribution_path_refuses_past_the_n_wall",
            "distribution_library_call_refuses_past_the_n_wall",
            "carrier_refuses_past_the_n_wall_by_name",
            "carrier_library_call_refuses_past_the_n_wall",
        ],
    ),
    (
        "M5",
        "the router routes to magic on T-count alone (the sibling defect)",
        LIB,
        "    if t <= T_ROUTE_MAGIC\n        && !c.gates.iter().any(|g| matches!(g, Gate::Ccx(..)))\n        && c.n_qubits <= N_MAX_STATEVECTOR\n    {",
        "    if t <= T_ROUTE_MAGIC\n        && !c.gates.iter().any(|g| matches!(g, Gate::Ccx(..)))\n    {",
        ["router_does_not_send_a_wide_circuit_to_magic"],
    ),
    (
        "M6",
        "the carrier trusts the router again",
        LIB,
        "    if let Err(reason) =\n        check_n_budget(c, \"the statevector carrier stores 2^n amplitudes\")\n    {\n        panic!(\"{reason}\");\n    }\n    let dim = 1usize << n;",
        "    let dim = 1usize << n;",
        ["carrier_library_call_refuses_past_the_n_wall"],
    ),
    (
        "M7",
        "the cap is tightened by one (the wall must sit AT the cap)",
        LIB,
        "pub const T_MAX_MAGIC: usize = 24;",
        "pub const T_MAX_MAGIC: usize = 23;",
        ["amp_path_is_open_at_the_cap"],
    ),
    (
        "M8",
        "the refusal drops the numbers it is supposed to name",
        LIB,
        "        \"REFUSED by the magic wall: T-count {t} > {T_MAX_MAGIC}, the magic \\\n         tier's branch budget.",
        "        \"REFUSED by the magic wall: this circuit is too expensive. \\\n         Ignore the following:",
        ["amp_path_refuses_past_the_t_cap"],
    ),
]


# Sources are edited in place, so an interrupt between the edit and the restore
# would BANK a mutation in the tree. `finally` does not cover that: SIGTERM
# terminates the interpreter without unwinding, and the first run of this
# battery was killed by an outer timeout and left `T_MAX_MAGIC = 23` behind.
_PENDING = {}


def _restore_all():
    for path, text in list(_PENDING.items()):
        with open(path, "w") as f:
            f.write(text)
    _PENDING.clear()


def _on_signal(signum, _frame):
    _restore_all()
    print(f"\nINTERRUPTED by signal {signum} -- sources restored.", file=sys.stderr)
    sys.exit(130)


for _sig in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
    signal.signal(_sig, _on_signal)


def sha(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def limit_mem():
    resource.setrlimit(resource.RLIMIT_AS, (LIMIT, LIMIT))


def cargo(args, cap_mem, timeout=None):
    return subprocess.run(
        [CARGO, *args],
        cwd=ENGINE,
        capture_output=True,
        text=True,
        preexec_fn=limit_mem if cap_mem else None,
        timeout=timeout,
    )


def build_tests():
    """Compile the test target. A mutant that does not compile has not been
    observed to be caught."""
    p = cargo(["build", "--release", "-p", "holon-qasm", "--tests"], cap_mem=False)
    return p.returncode == 0, p.stderr[-3000:]


def run_one(name):
    """Run ONE named test in its own process and say whether it stayed green.

    Isolation is not tidiness here, it is correctness. Removing a 2^n guard
    lets the mutant reach a real 2^n allocation, which ABORTS the whole test
    binary (SIGABRT) rather than failing one test -- so a batch run prints no
    per-test verdict at all, and a parser that looks only for "... FAILED"
    reads a loudly-caught mutation as an undetected one. That happened on this
    battery's first run and cost two false UNDETECTED verdicts.

    Returns (green, how) where `how` distinguishes the ways it went red.
    """
    try:
        p = cargo(
            ["test", "--release", "-p", "holon-qasm", "--test", "refusal",
             "--", "--exact", name],
            cap_mem=True,
            timeout=TEST_TIMEOUT,
        )
    except subprocess.TimeoutExpired:
        return False, f"the mutant enumerated past {TEST_TIMEOUT}s instead of refusing"
    log = p.stdout + p.stderr
    if re.search(rf"^test {re.escape(name)} .*\.\.\. ok$", log, re.M):
        return True, "green"
    if re.search(rf"^test {re.escape(name)} .*\.\.\. FAILED$", log, re.M):
        return False, "assertion"
    if "SIGABRT" in log or "signal:" in log or "memory allocation of" in log:
        return False, "the mutant aborted the process"
    if "0 filtered out" in log and " 0 passed" in log:
        return False, "the test did not run (name not found)"
    return False, f"nonzero exit ({p.returncode})"


def run_all():
    """Baseline only: every test in the file, batched."""
    p = cargo(["test", "--release", "-p", "holon-qasm", "--test", "refusal"],
              cap_mem=True)
    log = p.stdout + p.stderr
    names = re.findall(r"^test (\S+) .*\.\.\. (ok|FAILED)$", log, re.M)
    return p.returncode == 0, dict(names), log


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--only", default=None)
    args = ap.parse_args()

    if not os.path.isdir(QASM):
        sys.exit(f"REFUSED: no holon-qasm crate at {QASM}. Set HOLON_ENGINE.")

    chosen = [m for m in MUTATIONS if args.only in (None, m[0])]
    if not chosen:
        sys.exit(f"REFUSED: no mutation named {args.only!r}")

    # Every target string must be present BEFORE anything is edited. A stale
    # mutation is a different failure than an undetected one and must not be
    # reported as either.
    stale = []
    for mid, desc, path, old, _new, _tests in chosen:
        if not os.path.exists(path):
            stale.append(f"{mid}: no such file {path}")
        elif open(path).read().count(old) != 1:
            n = open(path).read().count(old)
            stale.append(f"{mid}: target text occurs {n}x (want exactly 1) in {path}")
    if stale:
        print("REFUSED -- the mutations are stale, the guards have moved:")
        for s in stale:
            print("  " + s)
        sys.exit(2)

    if args.dry_run:
        print(f"engine   {ENGINE}")
        print(f"mem cap  {MEM_GB} GB on the test process")
        print(f"{len(chosen)} mutation(s), all sites present:\n")
        for mid, desc, path, _old, _new, tests in chosen:
            print(f"  {mid}  {desc}")
            print(f"      {os.path.relpath(path, ENGINE)}")
            print(f"      must fire: {', '.join(tests)}")
        return 0

    print("BASELINE -- the unmutated crate must be green before anything is planted.")
    ok, err = build_tests()
    if not ok:
        sys.exit(f"REFUSED: the unmutated crate does not build.\n{err}")
    green, base, log = run_all()
    reds = [k for k, v in base.items() if v != "ok"]
    if not green or reds or not base:
        sys.exit(f"REFUSED: baseline is not green ({len(base)} tests, red: {reds}).\n{log[-2000:]}")
    print(f"  {len(base)} tests green.\n")

    verdicts = []
    for mid, desc, path, old, new, tests in chosen:
        before = open(path).read()
        h = hashlib.sha256(before.encode()).hexdigest()
        _PENDING[path] = before
        try:
            open(path, "w").write(before.replace(old, new, 1))
            compiled, err = build_tests()
            if not compiled:
                verdict = "INVALID"
                detail = "the mutant does not compile; nothing was tested"
            else:
                outcomes = {t: run_one(t) for t in tests}
                missed = [t for t, (g, _) in outcomes.items() if g]
                if missed:
                    verdict = "UNDETECTED"
                    detail = f"stayed green: {', '.join(missed)}"
                else:
                    verdict = "CAUGHT"
                    detail = "; ".join(f"{t} ({how})" for t, (_, how) in outcomes.items())
        finally:
            _restore_all()
            if sha(path) != h:
                sys.exit(f"FATAL: {path} not restored after {mid}. Fix the tree by hand.")
        verdicts.append((mid, verdict, desc, detail))
        print(f"{verdict:11s} {mid}  {desc}")
        print(f"            {detail}")

    bad = [v for v in verdicts if v[1] != "CAUGHT"]
    print()
    print(f"{len(verdicts) - len(bad)}/{len(verdicts)} planted defects caught.")
    if bad:
        print("NOT CAUGHT:")
        for mid, verdict, desc, detail in bad:
            print(f"  {verdict} {mid} {desc} -- {detail}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
