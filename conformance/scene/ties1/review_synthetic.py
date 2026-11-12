# Synthetic respondents and ties; nothing from the survey. Values are mine, not the draft's.
import numpy as np
rng=np.random.default_rng(23)
def sample(R=2300):
    w=rng.lognormal(0,0.55,R); n=np.clip(rng.poisson(3.1,R),0,10)
    rid=np.repeat(np.arange(R),n); T=len(rid)
    re=rng.normal(0,0.5,R)[rid]                      # respondent effect on contact (shared by their ties)
    place=rng.choice(3,R,p=[.3,.5,.2])               # urban, suburban, rural
    hh=rng.random(T)<0.16
    far=(~hh)&(rng.random(T)<(np.array([.26,.22,.22])[place[rid]]+0.08*rng.normal(0,1,R)[rid]).clip(0,1))
    mid=(~hh)&(~far)&(rng.random(T)<0.45)            # 5-50 miles
    kin=rng.random(T)<0.45
    # six electronic means, answer category 0..6 (0 = several times a day)
    ptop=np.where(hh,0.45,np.where(far,0.05,0.12))
    cat=np.where(rng.random((T,6))<ptop[:,None],0,rng.integers(1,7,(T,6)))
    days=np.array([365,365,156,52,12,4,0])[cat]*np.exp(re)[:,None].clip(0,1.0)
    return dict(w=w,rid=rid,hh=hh,far=far,mid=mid,kin=kin,cat=cat,days=days,place=place,R=R)
s=sample(); tw=s['w'][s['rid']]
print("household share of ties %.3f"%(tw[s['hh']].sum()/tw.sum()))
for m in (1,3,10,30):
    sc=np.array([365*m,365,156,52,12,4,0])[s['cat']].sum(1)
    print(f" 'several times a day' = {m:2d} a day: household share of electronic contact {(tw*sc)[s['hh']].sum()/(tw*sc).sum():.3f}")
def boot(stat,B=300):
    R=s['R']; out=[]
    idx_by=[np.where(s['rid']==r)[0] for r in range(R)]
    for _ in range(B):
        pick=rng.integers(0,R,R); ii=np.concatenate([idx_by[r] for r in pick]); out.append(stat(ii))
    return np.std(out)
e=s['days'].sum(1)
def s3(ii):
    a=ii[s['mid'][ii]&s['kin'][ii]]; b=ii[s['mid'][ii]&~s['kin'][ii]]
    ma=np.average(e[a],weights=tw[a]); mb=np.average(e[b],weights=tw[b]); return (ma-mb)/((ma+mb)/2)
def s4(ii):
    nh=ii[~s['hh'][ii]]; u=nh[s['place'][s['rid'][nh]]==0]; r=nh[s['place'][s['rid'][nh]]==2]
    return np.average(s['far'][u],weights=tw[u])-np.average(s['far'][r],weights=tw[r])
allv=np.arange(len(e))
print(f"S3-like (kin against friends at 5-50 miles, no true difference): gap (2 SE) = {2*boot(s3):.3f} of the mean; ties in band {int(s['mid'].sum())}")
print(f"S4-like (urban minus rural share far, true 0.04): read {s4(allv):+.3f}, gap (2 SE) = {2*boot(s4):.3f}")
