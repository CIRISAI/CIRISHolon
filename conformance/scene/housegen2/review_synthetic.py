import numpy as np
from scipy.optimize import brentq
rng=np.random.default_rng(3)
k=np.array([1,2,3,4,5,7.]); s=np.array([.30,.37,.15,.12,.04,.02])   # a made-up row
mu=(k*s).sum(); m2=(k[1:]*s[1:]).sum()/s[1:].sum()
def tilt(th): p=s*np.exp(-th*k); return p/p.sum()
print(f"made-up row: one-person {s[0]:.2f}, mean {mu:.3f}, mean of multi-person households {m2:.3f}")
print("target one-person | tilt: mean, multi-person mean, share of 4+ | mixture (multi-person shape fixed): mean, 4+ | mean difference")
for t in (.20,.40,.50,.60):
    th=brentq(lambda x: tilt(x)[0]-t,-5,5); p=tilt(th); q=np.r_[t,s[1:]/s[1:].sum()*(1-t)]
    print(f"      {t:.2f}        |   {(k*p).sum():.3f}   {(k[1:]*p[1:]).sum()/p[1:].sum():.3f}   {p[3:].sum():.3f}   |   {(k*q).sum():.3f}   {q[3:].sum():.3f}   | {100*((k*p).sum()/(k*q).sum()-1):+.1f}%")
# a moment estimator of the tile-effect spread, tiles of very different size, with count noise
def trial(tau,n=3000,noise=True,family="mix"):
    H=np.exp(rng.uniform(np.log(90),np.log(4000),n)).astype(int)
    p=np.clip(s[0]+rng.normal(0,tau,n),.02,.95)
    n1=rng.binomial(H,p).astype(float); Hp=H.astype(float)
    if noise:
        n1+=rng.choice([-2,-1,0,1,2],n,p=[.05,.125,.65,.125,.05]); Hp+=rng.choice([-2,-1,0,1,2],n,p=[.05,.125,.65,.125,.05])
    ph=n1/Hp; pb=ph.mean()
    raw=ph.var(ddof=1); binom=(pb*(1-pb)/Hp).mean(); ck=(0.85*(1+pb**2)/Hp**2).mean()
    # household-weighted version
    w=Hp/Hp.sum(); pw=(w*ph).sum(); raww=(w*(ph-pw)**2).sum()/(1-(w**2).sum()); binw=(w*pw*(1-pw)/Hp).sum()
    return np.sqrt(raw),np.sqrt(max(raw-binom,0)),np.sqrt(max(raw-binom-ck,0)),np.sqrt(max(raww-binw,0))
for tau in (0.04,0.07,0.11):
    r=np.mean([trial(tau) for _ in range(40)],0)
    print(f"true spread {tau:.2f}: raw SD {r[0]:.4f}; minus binomial {r[1]:.4f}; minus binomial and count noise {r[2]:.4f}; household-weighted, minus binomial {r[3]:.4f}")
# when the true spread depends on tile size
def trial2(n=3000):
    H=np.exp(rng.uniform(np.log(90),np.log(4000),n)).astype(int); tau=0.14*(H/90.)**-0.25
    p=np.clip(s[0]+rng.normal(0,tau,n),.02,.95); ph=rng.binomial(H,p)/H; pb=ph.mean()
    un=np.sqrt(ph.var(ddof=1)-(pb*(1-pb)/H).mean()); w=H/H.sum(); pw=(w*ph).sum()
    hw=np.sqrt((w*(ph-pw)**2).sum()-(w*pw*(1-pw)/H).sum())
    return un,hw,np.sqrt((tau**2).mean()),np.sqrt((w*tau**2).sum())
r=np.mean([trial2() for _ in range(40)],0)
print(f"spread falling with tile size: each tile once estimates {r[0]:.4f} (truth for that weight {r[2]:.4f}); by households {r[1]:.4f} (truth {r[3]:.4f})")
