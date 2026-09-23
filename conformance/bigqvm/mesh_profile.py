#!/usr/bin/env python3
"""MESH-CLIFFORD-1 profile driver: d in {141, 45}, arms interleaved and rotated per repetition,
pinned to cores 21-27. Writes conformance/bigqvm/profile_d{d}_{arm}.json (raw flagship JSON per rep)."""
import json, os, subprocess, time

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..'))
BIN = ROOT + '/engine/target/release/examples/surface_flagship'
OUT = ROOT + '/conformance/bigqvm'
CORES = '21-27'
REPS = 3

ARMS = {
    # name: extra flags
    'unsharded':                ['--shards', '1', '--profile'],
    'S1':                       ['--shards', '1', '--sharded', '--profile'],
    'S8':                       ['--shards', '8', '--profile'],
    'S8_transpose_parallel':    ['--shards', '8', '--profile', '--transpose-parallel'],
    # timers OFF, for the overhead read
    'unsharded_noprof':         ['--shards', '1'],
    'S1_noprof':                ['--shards', '1', '--sharded'],
    'S8_noprof':                ['--shards', '8'],
}

def loadavg():
    return open('/proc/loadavg').read().split()[:3]

def run(d, flags):
    cmd = ['taskset', '-c', CORES, BIN, '--d', str(d), '--mode', 'bench', '--rounds', '3',
           '--seed', '1', '--layout', 'banded'] + flags
    p = subprocess.run(cmd, capture_output=True, text=True)
    if p.returncode != 0:
        raise SystemExit(f'failed: {cmd}\n{p.stderr}')
    return cmd, json.loads(p.stdout.strip().splitlines()[-1]), p.stderr

for d in [141, 45]:
    names = list(ARMS)
    if d == 45:
        names = [n for n in names if not n.endswith('_noprof')] + ['unsharded_noprof', 'S8_noprof']
    rec = {n: {'distance': d, 'arm': n, 'cores': CORES, 'reps': []} for n in names}
    for rep in range(REPS):
        order = names[rep % len(names):] + names[:rep % len(names)]
        for n in order:
            la0 = loadavg()
            t0 = time.time()
            cmd, js, err = run(d, ARMS[n])
            rec[n]['command'] = ' '.join(cmd)
            rec[n]['reps'].append({'rep': rep, 'loadavg_start': la0, 'loadavg_end': loadavg(),
                                   'started_unix': t0, 'result': js,
                                   'stderr_tail': err.strip().splitlines()[-24:]})
            md = js['results'][0]['metadata']
            print(d, n, rep, md['timing_seconds']['wall'], md['record_hash'], flush=True)
    for n in names:
        with open(f'{OUT}/profile_d{d}_{n}.json', 'w') as f:
            json.dump(rec[n], f, indent=1)
