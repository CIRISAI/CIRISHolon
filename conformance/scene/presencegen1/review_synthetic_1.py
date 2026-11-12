# Synthetic days (mine, no survey): 0 home, 1 work, 2 elsewhere, 3 travel. Minute 0 = 4 a.m.
import numpy as np
rng=np.random.default_rng(31)
def heap(t): return int(round(t/30.0)*30) if rng.random()<0.8 else int(t)
def day():
    d=np.zeros(1440,int); u=rng.random()
    if u<0.24: return d
    if u<0.66:                                   # a working day, short or long
        a=heap(rng.normal(250,55)); L=heap(rng.normal(270,40) if rng.random()<0.3 else rng.normal(530,45)); tr=int(rng.integers(12,40))
        a=max(a,tr+1); e=min(a+L,1380)
        d[a-tr:a]=3; d[a:e]=1; d[e:e+tr]=3
        if rng.random()<0.3: s=e+tr; d[s:s+int(rng.integers(20,80))]=2; d[s+60:s+75]=3 if False else d[s+60:s+75]
    else:
        for k in range(rng.integers(1,4)):
            a=heap(rng.uniform(300,1000)); L=int(rng.integers(15,150)); tr=int(rng.integers(8,25))
            d[a-tr:a]=3; d[a:a+L]=2; d[a+L:a+L+tr]=3
    return d
N=12000; M=np.array([day() for _ in range(N)]); H=M[:,::60]
print(f"travel: share of minutes {np.mean(M==3):.4f}; share of hour-starts {np.mean(H==3):.4f}; at a seeded random minute in each hour {np.mean(M.reshape(N,24,60)[np.arange(N)[:,None],np.arange(24)[None,:],rng.integers(0,60,(N,24))]==3):.4f}")
dep_min=np.mean(((M[:,1:]!=0)&(M[:,:-1]==0)).sum(1)); dep_hr=np.mean(((H[:,1:]!=0)&(H[:,:-1]==0)).sum(1))
print(f"departures from home a day: by minutes {dep_min:.2f}; seen at hour-starts {dep_hr:.2f}")
def fit_sim(H,flag,n=40000):
    S=4*(2 if flag else 1); X=H.copy()
    if flag:
        left=np.cumsum(H!=0,1)>0; X=H+4*left
    T=np.zeros((23,S,S)); 
    for h in range(23): np.add.at(T[h],(X[:,h],X[:,h+1]),1)
    p0=np.bincount(X[:,0],minlength=S)/len(X); out=np.zeros((n,24),int); out[:,0]=rng.choice(S,n,p=p0)
    for h in range(23):
        P=T[h]/np.maximum(T[h].sum(1,keepdims=True),1); c=P.cumsum(1); r=rng.random(n)
        out[:,h+1]=np.minimum((r[:,None]>c[out[:,h]]).sum(1),S-1)
    return out%4
def stats(H):
    away=H!=0; stay=np.mean(~away.any(1)); hrs=away.sum(1)
    sp=[]
    for row in away:
        z=np.diff(np.concatenate([[0],row.astype(int),[0]])); sp+=list(np.where(z==-1)[0]-np.where(z==1)[0])
    sp=np.array(sp); w=(H==1).sum(1)
    return stay,sp.mean(),np.mean(sp<=2),np.mean(sp>=8),np.mean(w>=6),np.mean(w==0),np.mean((H[:,5]==0)&(H[:,10]==0)&(H[:,14]==0)),np.percentile(hrs,[25,50,75,90])
names="stays home | mean away spell h | spells <=2 h | spells >=8 h | work >=6 hour-starts | work none | home at 9,14,18 | away hours 25/50/75/90"
print(names)
for tag,X in (("synthetic days",H),("chain (G1)",fit_sim(H,False)),("chain + has-left bit",fit_sim(H,True))):
    s=stats(X); print(f"{tag:22s} {s[0]:.3f} | {s[1]:.2f} | {s[2]:.3f} | {s[3]:.3f} | {s[4]:.3f} | {s[5]:.3f} | {s[6]:.3f} | {s[7]}")
