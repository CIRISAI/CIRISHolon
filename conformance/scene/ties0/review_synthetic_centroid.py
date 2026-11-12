# Synthetic: one individual-level kernel, two unit sizes. Centre-point exponent and band shares.
import numpy as np
rng=np.random.default_rng(7)
def run(L,b=1.5,c=1.4,half=600.0,n=500):
    k=lambda d:(b+d)**(-c)
    m=int(half//L)
    x=rng.uniform(0,L,(n,2))
    rows=[]
    for i in range(-m,m+1):
        for j in range(-m,m+1):
            y=rng.uniform(0,L,(n,2))+np.array([i*L,j*L])
            d=np.sqrt(((x[:,None,:]-y[None,:,:])**2).sum(-1))
            if i==0 and j==0: d=d[d>0]
            rows.append((np.hypot(i,j)*L,k(d).mean(),(d<=25).mean(),(d<=50).mean(),k(d)[d<=25].sum()/k(d).size if True else 0,(k(d)*(d<=50)).mean()))
    r=np.array(rows); D,S=r[:,0],r[:,1]
    tot=S.sum()
    own=S[D==0][0]/tot
    cen25=S[D<=25].sum()/tot; cen50=S[D<=50].sum()/tot
    true25=r[:,4].sum()/tot; true50=r[:,5].sum()/tot
    out=[]
    for lo,hi in [(0,200),(100,400)]:
        sel=(D>lo)&(D<hi)
        out.append(np.polyfit(np.log(D[sel]),np.log(S[sel]),1)[0])
    return own,cen25,true25,cen50,true50,out
for L in (22,34,65,110):
    own,c25,t25,c50,t50,o=run(L)
    print(f"side {L:4d} km: own {own:.3f} | <=25 km centre {c25:.3f} true {t25:.3f} | <=50 km centre {c50:.3f} true {t50:.3f} | slope <200 km {o[0]:.2f}, 100-400 km {o[1]:.2f} (individual law -1.40)")
