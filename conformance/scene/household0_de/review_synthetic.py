import numpy as np
rng=np.random.default_rng(5)
shares=np.array([.36,.34,.14,.11,.035,.015])   # a made-up size distribution, 1..6+
def perturb(c,rule):
    c=c.copy(); out=c.copy()
    big=c>=3
    noise=rng.choice([-2,-1,0,1,2],size=c.shape,p=[.05,.125,.65,.125,.05])
    out[big]=np.maximum(c[big]+noise[big],3)            # stays publishable
    u=rng.random(c.shape)
    if rule=="unbiased":    # expectation kept: 1 -> 3 w.p. 1/3, 2 -> 3 w.p. 2/3
        out[c==1]=np.where(u[c==1]<1/3,3,0); out[c==2]=np.where(u[c==2]<2/3,3,0)
    elif rule=="even":      # 1 and 2 each go to 0 or 3 with equal chance
        out[c==1]=np.where(u[c==1]<.5,3,0); out[c==2]=np.where(u[c==2]<.5,3,0)
    elif rule=="down":      # 1 -> 0 always; 2 -> 0 or 3 evenly
        out[c==1]=0; out[c==2]=np.where(u[c==2]<.5,3,0)
    return out
print("cell size | rule     | pooled parts/total | one-person share (true .360) | 5-person (true .035) | 6+ (true .015) | cells excluded under 10")
for H in (20,40,80,150):
    n=40000
    true=rng.multinomial(H,shares,size=n)
    tot_true=true.sum(1)
    for rule in ("unbiased","even","down"):
        pub=perturb(true,rule); tot_pub=perturb(tot_true[:,None],rule)[:,0]
        keep=pub.sum(1)>=10
        s=pub[keep].sum(0)/pub[keep].sum()
        print(f"   {H:4d}   | {rule:8s} |      {pub[keep].sum()/tot_pub[keep].sum():.4f}        |          {s[0]:.4f}             |       {s[4]:.4f}        |     {s[5]:.4f}     | {1-keep.mean():.4f}")
