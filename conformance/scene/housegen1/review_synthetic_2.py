import numpy as np
exec(open('gen.py').read().split('print("row |')[0])
def merge(n,sz,p):
    out=[];left=n
    while left>0:
        s=rng.choice(sz,p=p)
        if s<=left: out.append(s); left-=s
        else:
            if not out or rng.random()<left/s: out.append(left)
            else: out[rng.integers(len(out))]+=left
            left=0
    return out
for name,(sz,p) in rows.items():
    mu=(sz*p).sum()
    for n in (15,30,60,120):
        L=[merge(n,sz,p) for _ in range(int(240000/n))]; a=np.concatenate(L); hh=np.array([len(l) for l in L])
        print(f"{name} | {n:3d} | keep the remainder with chance r/s, else add it to a random earlier household | households {hh.mean():6.2f} ({n/mu:6.2f}) | mean {a.mean():.3f} ({mu:.3f}) {100*(a.mean()/mu-1):+.1f}% | one-person {(a==1).mean():.3f} ({p[0]:.2f}) | two-person {(a==2).mean():.3f} ({p[1]:.2f}) | above the top band {(a>sz[-1]).mean():.4f} | sd households {hh.std():.2f}")
