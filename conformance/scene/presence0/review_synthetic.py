import numpy as np
rng=np.random.default_rng(5)
# (1) hour state by most minutes against averaged minutes, synthetic diaries
N=4000; tm=np.zeros(24); tp=np.zeros(24); sm=0; sp=0
for _ in range(N):
    day=np.zeros(1440,int)                      # 0 home, 1 away, 2 travel
    t=int(rng.normal(8*60,70)); 
    for k in range(rng.poisson(1.7)):           # round trips
        if t>1300: break
        dur=int(np.clip(rng.lognormal(np.log(19),0.6),4,90)); stay=int(rng.exponential(170))+10
        day[t:t+dur]=2; day[t+dur:t+dur+stay]=1; day[t+dur+stay:t+2*dur+stay]=2
        t=t+2*dur+stay+int(rng.exponential(120))
    h=day[:1440].reshape(24,60)
    tm+=(h==2).mean(1)
    cnt=np.stack([(h==s).sum(1) for s in range(3)]); tp+=(cnt.argmax(0)==2)
print(f"travel, share of person-time: by minutes {tm.mean()/N:.4f} ({tm.mean()/N*1440:.0f} min a day); by the hour's largest state {tp.mean()/N:.4f}; ratio {tp.sum()/tm.sum():.2f}")
# (2) S6 arithmetic under NO change between years: what verdicts can a cell get?
def cell(nfit,nsc,p,cv=0.9,B=300,R=400,budget=0.03):
    out=np.zeros(3,int)
    for _ in range(R):
        wf=rng.lognormal(0,np.sqrt(np.log(1+cv**2)),nfit); ws=rng.lognormal(0,np.sqrt(np.log(1+cv**2)),nsc)
        xf=rng.random(nfit)<p; xs=rng.random(nsc)<p
        miss=abs((wf*xf).sum()/wf.sum()-(ws*xs).sum()/ws.sum())
        i=rng.integers(0,nsc,(B,nsc)); j=rng.integers(0,nfit,(B,nfit))
        bs=(ws[i]*xs[i]).sum(1)/ws[i].sum(1)-(wf[j]*xf[j]).sum(1)/wf[j].sum(1)
        gap=2*bs.std()
        out[0 if miss+gap<=budget else (1 if miss-gap>budget else 2)]+=1
    return out/R, gap
for label,nf,ns in [("one weekday, hour of week",1500,700),("Saturday or Sunday, hour of week",3900,1800),("Mon-Fri pooled, clock hour",7500,3500)]:
    for p in (0.06,0.35):
        o,g=cell(nf,ns,p)
        print(f"{label:34s} n_fit {nf} n_score {ns} share {p:.2f}: gap {g:.3f}; certified {o[0]:.2f} refused {o[1]:.2f} unresolved {o[2]:.2f}")
