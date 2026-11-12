import numpy as np
rng=np.random.default_rng(11)
rows={"small households":(np.array([1,2,3,4,5,7]),np.array([.50,.30,.10,.07,.02,.01])),
      "large households":(np.array([1,2,3,4,5,6,8]),np.array([.24,.33,.16,.14,.08,.03,.02]))}
def seq(n,sz,p):
    out=[];left=n
    while left>0:
        s=min(rng.choice(sz,p=p),left); out.append(s); left-=s
    return out
def count_first(n,sz,p):
    mu=(sz*p).sum(); x=n/mu; h=int(x)+(rng.random()<x-int(x)); h=max(1,min(h,n))
    # systematic sample of h sizes from the row (random start), then repair the sum by +-1 moves
    u=(rng.random()+np.arange(h))/h; s=sz[np.searchsorted(np.cumsum(p),u,side='right').clip(max=len(sz)-1)].astype(int)
    rng.shuffle(s); d=n-s.sum(); i=0
    while d!=0:
        j=rng.integers(h)
        if d>0: s[j]+=1; d-=1
        elif s[j]>1: s[j]-=1; d+=1
        i+=1
    return list(s)
print("row | leaf people | scheme | households per leaf (n/mu) | mean size (row) | one-person share (row) | two-person (row) | sd of households per leaf")
for name,(sz,p) in rows.items():
    mu=(sz*p).sum()
    for n in (15,30,60,120,300):
        for sch,f in (("draw until used up",seq),("count first, systematic, repair",count_first)):
            L=[f(n,sz,p) for _ in range(int(240000/n))]
            a=np.concatenate(L); hh=np.array([len(l) for l in L])
            print(f"{name} | {n:3d} | {sch:31s} | {hh.mean():6.2f} ({n/mu:6.2f}) | {a.mean():.3f} ({mu:.3f}) {100*(a.mean()/mu-1):+.1f}% | {(a==1).mean():.3f} ({p[0]:.2f}) | {(a==2).mean():.3f} ({p[1]:.2f}) | {hh.std():.2f}")
