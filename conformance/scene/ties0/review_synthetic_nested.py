import numpy as np, importlib.util,sys
src=open('t2_nest.py').read().split("for conv in 'AB':")[0]
exec(src)
def shares(par,pop,g):
    w=np.empty_like(pop)
    for I in range(par.max()+1):
        a=par==I; x=pop[a]**(1+g); w[a]=x/x.sum()
    return w
def pred(par,S3,w):
    nP=par.max()+1; P=np.zeros((nP,nP)); O=np.zeros(nP); Dg=np.zeros(nP)
    for I in range(nP):
        a=par==I
        for J in range(nP):
            P[I,J]=w[a]@S3[np.ix_(a,par==J)]@w[par==J]
        M=S3[np.ix_(a,a)]; Dg[I]=(w[a]**2*M.diagonal()).sum(); O[I]=P[I,I]-Dg[I]
    return P,O,Dg
for conv in 'AB':
  for noise in (0.0,0.03,0.10):
    par,pop,U,S3,Sp=make(noise=noise,conv=conv); nP=par.max()+1
    off=~np.eye(nP,dtype=bool); best=None
    for g in np.arange(-0.5,0.31,0.02):
        w=shares(par,pop,g); P,O,Dg=pred(par,S3,w)
        lr=np.log(Sp[off]/P[off]); c=np.median(lr); mis=np.sqrt(np.mean((lr-c)**2))
        if best is None or mis<best[0]: best=(mis,g,c,w,P,O,Dg)
        if abs(g)<1e-9: mis0=mis
    mis,g,c,w,P,O,Dg=best
    wt=np.concatenate([U[par==I]/U[par==I].sum() for I in range(nP)]) if False else None
    wt=np.empty_like(pop)
    for I in range(nP): a=par==I; wt[a]=U[a]/U[a].sum()
    e=lambda x:np.sqrt(np.mean((x/wt-1)**2))
    kap=(Sp.diagonal()/np.exp(c)-O)/Dg; kapB=(Sp.diagonal()/np.exp(c)-O/2)/Dg
    print(f"conv {conv} noise {noise:.2f}: tilt found {g:+.2f}; misfit at tilt 0 {mis0:.3f}, at best {mis:.3f}; share error residents {e(shares(par,pop,0)):.3f} -> fitted {e(w):.3f}; diagonal ratio under reading A {np.median(kap):.2f}, under reading B {np.median(kapB):.2f}")
