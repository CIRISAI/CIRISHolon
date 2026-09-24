"""Export stabrank's known qubit_H witnesses to a flat text format the Rust tests read.

One decomposition per block: a line `dec <m> <rank> <source>`, then one line per term:
`k=<k> x0=<bits> W=<row;row;...> Q=<row;row;...> l=<l,l,...>` with bit strings indexed by
qubit (stabrank's list order), Q the full k x k matrix as stabrank stores it.
"""
import json, sys

def term_line(t):
    k = t['k']
    x0 = ''.join(str(b) for b in t['x0'])
    W = ';'.join(''.join(str(b) for b in r) for r in t.get('W', [])) or '-'
    Q = ';'.join(''.join(str(b) for b in r) for r in t.get('Q', [])) or '-'
    l = ','.join(str(v) for v in t.get('l', [])) or '-'
    return f'k={k} x0={x0} W={W} Q={Q} l={l}'

out = []
root = sys.argv[1]
for rel in ['research/constructions/data/qubit_H_m4_rank4.json']:
    d = json.load(open(f'{root}/{rel}'))
    for i, dec in enumerate(d['decompositions']):
        out.append(f'dec {d["m"]} {d["rank"]} stabrank:{rel}#{i}')
        out += [term_line(t) for t in dec]
for rel in ['bounds/qubit_H-m4-upper-4.json', 'bounds/qubit_H-m6-upper-6.json', 'bounds/qubit_H-m7-upper-9.json',
            'bounds/qubit_H-m3-upper-3.json', 'bounds/qubit_H-m2-upper-2.json', 'bounds/qubit_H-m5-upper-6.json']:
    try:
        d = json.load(open(f'{root}/{rel}'))
    except FileNotFoundError:
        continue
    if 'witness' not in d:
        continue
    out.append(f'dec {d["m"]} {d["rank"]} stabrank:{rel}')
    out += [term_line(t) for t in d['witness']['terms']]
print('# Known qubit_H decompositions exported from unitaryfoundation/stabrank (Apache-2.0) at 2ceb676,')
print('# by conformance/rank/export_known.py. Terms only; coefficients are recomputed exactly by the reader.')
print('\n'.join(out))
