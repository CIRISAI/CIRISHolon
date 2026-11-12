import numpy as np
rng=np.random.default_rng(20261111)
def apportion(total,w):
    w=np.asarray(w,float); q=total*w/w.sum(); f=np.floor(q).astype(int); r=q-f; k=total-f.sum()
    o=np.lexsort((np.arange(len(w)),-r)); f[o[:k]]+=1; return f
def twin(nleaf=16667,hh=60,pph=2.5):
    raw=[];out=[]
    for _ in range(nleaf):
        d=np.clip(np.floor(2.5+1.1*rng.standard_normal(hh)+0.5),1,6).astype(int)
        raw.append(d); out.append(apportion(int(round(hh*pph)),d))
    return np.concatenate(raw),np.concatenate(out)
def dist(v,top=9): return np.array([(v==k).mean() for k in range(0,top+1)])
def w1(p,q): return np.abs(np.cumsum(p-q))[:-1].sum()
raw,app=twin()
pr,pa=dist(raw),dist(app)
print("sizes 0..9   raw clamped draw:",np.round(pr,4)," mean",raw.mean().round(4))
print("             after apportion :",np.round(pa,4)," mean",app.mean().round(4))
print("W1(raw, apportioned) =",w1(pr,pa).round(4)," one-person share",pr[1].round(4),"->",pa[1].round(4)," zero-person households:",pa[0])
# a US national distribution stated from memory (to be replaced by the survey's own): 1..7+
us=np.zeros(10); us[1:8]=[.289,.340,.151,.124,.058,.023,.015]; us/=us.sum()
for topmean in (7,8,9):
    u=us.copy(); 
    if topmean!=7: u[topmean]=u[7]; u[7]=0
    print(f"twin (apportioned) vs remembered national shares, top band at {topmean}: W1 = {w1(pa,u):.3f} persons; one-person share gap = {abs(pa[1]-u[1]):.3f}; means {app.mean():.3f} vs {(u*np.arange(10)).sum():.3f}")
# P-MEAN
A=np.zeros(10);A[2]=A[3]=.5;B=np.zeros(10);B[1]=B[4]=.5
print("P-MEAN: means",(A*np.arange(10)).sum(),(B*np.arange(10)).sum(),"W1 =",w1(A,B),"one-person gap",abs(A[1]-B[1]))
# P-TOP and the two-weights trap
s=np.array([0,.27,.33,.15,.12,.06,.02]); s=s/s.sum()*0.95
for label,share,mean in (("P-TOP 5% at 9",0.05,9.0),("realistic 1.5% at 7.8",0.015,7.8)):
    low=s/s.sum()*(1-share); pop=(low*np.arange(7)).sum()+share*mean
    for eps in (0,0.01,-0.01,0.02,-0.02):
        print(f"{label}: household population off by {eps:+.0%} against the household counts -> derived top mean = {(pop*(1+eps)-(low*np.arange(7)).sum())/share:.2f}")
# budget scale: synthetic counties, random split halves vs a split by state
def scale(ncty,sd_cty,sd_state,nstate=25,reps=400):
    out=[]
    st=rng.integers(0,nstate,ncty)
    for _ in range(reps):
        se=rng.normal(0,sd_state,nstate); x=se[st]+rng.normal(0,sd_cty,ncty)
        h=rng.permutation(ncty)<ncty//2; out.append(abs(x[h].mean()-x[~h].mean()))
    rs=np.quantile(out,.95)
    out2=[]
    for _ in range(reps):
        seA=rng.normal(0,sd_state,nstate); seB=rng.normal(0,sd_state,nstate)
        a=seA[st]+rng.normal(0,sd_cty,ncty); b=seB[st]+rng.normal(0,sd_cty,ncty); out2.append(abs(a.mean()-b.mean()))
    return rs,np.mean(np.array(out2)>rs)
for n in (1500,300,60):
    rs,p=scale(n,0.04,0.015); print(f"one-person share, {n} counties per side, county sd 0.04, state sd 0.015: 95% random-split gap = {rs:.4f}; a held-out half of other states exceeds it in {p:.0%} of draws")
