import numpy as np
a=open('t2_nest.py').read().split("for conv in 'AB':")[0]
b=open('t2b_nest.py').read().split("exec(src)")[1].split("for conv in 'AB':")[0]
exec(a);exec(b)
# (1) scale fixed first at g=0, against scale and tilt fitted together; and the convention statistics R_A, R_B
for conv in 'AB':
  for noise in (0.04,):
    par,pop,U,S3,Sp=make(nP=30,noise=noise,conv=conv); nP=par.max()+1; ctrue=np.log(9.1e3/3.7e5)
    off=~np.eye(nP,dtype=bool); res={}
    for g in np.arange(-0.5,0.31,0.02):
        w=shares(par,pop,g); P,O,Dg=pred(par,S3,w); lr=np.log(Sp[off]/P[off]); c=np.median(lr)
        res[round(g,2)]=(np.sqrt(np.mean((lr-c)**2)),c,O,Dg)
    gb=min(res,key=lambda k:res[k][0])
    # tilt fitted with the scale held at its g=0 value
    c0=res[0.0][1]; gfix=min(res,key=lambda k:np.mean((np.log(Sp[off]/pred(par,S3,shares(par,pop,k))[0][off])-c0)**2)) if False else None
    for tag,g in (("g=0",0.0),("fitted g",gb)):
        m,c,O,Dg=res[g]; d=Sp.diagonal()/np.exp(c)
        RA=(d-O)/Dg; RB=(d-O/2)/Dg
        print(f"published convention {conv}, {tag} ({g:+.2f}): scale error {np.exp(c-ctrue)-1:+.3f}; R_A median {np.median(RA):.2f} (10-90: {np.percentile(RA,10):.2f} to {np.percentile(RA,90):.2f}); R_B median {np.median(RB):.2f} ({np.percentile(RB,10):.2f} to {np.percentile(RB,90):.2f}); off/diag median {np.median(O/Dg):.2f}")
# (2) how much the floor b moves the kernel's mean over a disc, and how little pairs beyond 25 km see it
rng=np.random.default_rng(3); R=13.0; c=1.7
r=R*np.sqrt(rng.random((200000,2))); th=rng.random((200000,2))*2*np.pi
d=np.sqrt((r[:,0]*np.cos(th[:,0])-r[:,1]*np.cos(th[:,1]))**2+(r[:,0]*np.sin(th[:,0])-r[:,1]*np.sin(th[:,1]))**2)
D=np.exp(np.linspace(np.log(25),np.log(400),60))
base=None
for b in (0.2,1.0,3.0,6.0):
    own=((b+d)**-c).mean()
    # best power law A d^-c' (floor zero) to this kernel on 25..400 km: residual a fit must detect to see b
    y=np.log((b+D)**-c); sl,ic=np.polyfit(np.log(D),y,1); rms=np.sqrt(np.mean((y-(sl*np.log(D)+ic))**2))
    own0=np.exp(ic)*(np.maximum(d,1e-3)**sl).mean()
    print(f"floor b {b:3.1f} km: kernel mean over a disc of radius {R:.0f} km = {own:.5f}; a floor-zero power law fitted to it on 25-400 km (slope {sl:.2f}, rms log residual {rms:.3f}) predicts {own0:.5f}, ratio {own0/own:.2f}; own share 0.35 would read {0.35*own0/own/(0.35*own0/own+0.65):.2f}")
