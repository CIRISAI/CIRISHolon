#!/usr/bin/env python3
"""MOLSEARCH-1 (MOLSEARCH1_PREREG.md): the variational search at the molecule tier on the fine
model's all-atom walk. Dictionary per molecule: r1, r2, theta, their rates (6); COM velocity
(3); angular velocity about the COM (3). Held out by molecule.
  molsearch.py plants | molsearch.py read DIR"""
import sys, os, math
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from view_search import koopman, heldout_score, score_k, r2_on_subspace, inv_sqrt
M_O, M_H = 15.9949146196 * 1822.888486, 1.00782503223 * 1822.888486
def load(d):
    L=open(f"{d}/atoms.walk").readline().split(); Lbox=float(L[3])
    lines=open(f"{d}/atoms.walk").read().splitlines(); z=np.array(lines[1].split()[2:],int)
    P=np.array([np.array(l.split(),float).reshape(-1,3) for l in lines[2:]])
    vl=open(f"{d}/atoms.vwalk").read().splitlines(); dt=float(vl[1].split()[2])
    V=np.array([np.array(l.split(),float).reshape(-1,3) for l in vl[2:]])
    return P,V,z,Lbox,dt
def molecules(z):
    """units are O,H,H in ascending index order (the operator's declared site order)"""
    O=[i for i in range(len(z)) if z[i]==8]; return [(o,o+1,o+2) for o in O]
def features(P,V,z,dt):
    """per molecule (T, 12): [r1, r2, theta, dr1, dr2, dtheta, vcom(3), omega(3)]"""
    out=[]
    for (o,h1,h2) in molecules(z):
        a=P[:,h1]-P[:,o]; b=P[:,h2]-P[:,o]
        r1=np.linalg.norm(a,axis=1); r2=np.linalg.norm(b,axis=1)
        th=np.arccos(np.clip((a*b).sum(1)/(r1*r2),-1,1))
        va=V[:,h1]-V[:,o]; vb=V[:,h2]-V[:,o]
        dr1=(a*va).sum(1)/r1; dr2=(b*vb).sum(1)/r2
        # dtheta from d/dt of cos: -sin(th) dth = d(cos)
        ca=(a*b).sum(1)/(r1*r2); dca=((va*b).sum(1)+(a*vb).sum(1))/(r1*r2) - ca*(dr1/r1+dr2/r2)
        dth=-dca/np.maximum(np.sin(th),1e-9)
        m=np.array([M_O,M_H,M_H]); idx=[o,h1,h2]
        com=(P[:,idx]*m[None,:,None]).sum(1)/m.sum(); vcom=(V[:,idx]*m[None,:,None]).sum(1)/m.sum()
        # angular velocity: L = I omega, with rel positions/velocities about the COM
        rel=P[:,idx]-com[:,None,:]; vrel=V[:,idx]-vcom[:,None,:]
        Lang=(m[None,:,None]*np.cross(rel,vrel)).sum(1)
        om=np.zeros((len(P),3))
        for t in range(len(P)):
            I=np.zeros((3,3))
            for j in range(3): x=rel[t,j]; I+=m[j]*((x@x)*np.eye(3)-np.outer(x,x))
            om[t]=np.linalg.solve(I,Lang[t])
        out.append(np.column_stack([r1,r2,th,dr1,dr2,dth,vcom,om]))
    return out
RIGID=[6,7,8,9,10,11]; INTERNAL=[0,1,2,3,4,5]
def search(feats, dt, lags_fs, label=""):
    n=len(feats); k_all=feats[0].shape[1]
    for tau in lags_fs:
        lag=max(1,int(round(tau/dt)))
        # centre per molecule, standardise globally
        F=[f-f.mean(0) for f in feats]
        allF=np.concatenate(F); sc=allF.std(0); F=[f/sc for f in F]; allF=allF/sc
        m=koopman(allF,lag); s=m["s"]
        w=[]
        for i in range(3):
            v=m["W0"]@m["U"][:,i]; w.append((r2_on_subspace(v,allF,RIGID), r2_on_subspace(v,allF,INTERNAL)))
        # held out by molecule: 4 folds of 4
        folds=[list(range(j,n,4)) for j in range(4)]
        def ho(cols,k):
            sc_=[];bd=[]
            for test in folds:
                tr=np.concatenate([F[i][:,cols] for i in range(n) if i not in test]); te=np.concatenate([F[i][:,cols] for i in test])
                trD=np.concatenate([F[i] for i in range(n) if i not in test]); teD=np.concatenate([F[i] for i in test])
                sc_.append(heldout_score(tr,te,lag,k)[0]); bd.append(heldout_score(trD,teD,lag,k)[0])
            return np.mean(sc_),np.mean(bd)
        rs,rb=ho(RIGID,6); is_,ib=ho(INTERNAL,6)
        s_int=koopman(allF[:,INTERNAL],lag)["s"][0]; s_rig=koopman(allF[:,RIGID],lag)["s"][0]
        # Amendment 1: purity of each top-6 singular function (S1') and the cross-block norm (S2')
        pur=[]
        for i in range(6):
            v=m["W0"]@m["U"][:,i]; pur.append(max(r2_on_subspace(v,allF,RIGID), r2_on_subspace(v,allF,INTERNAL)))
        # the whitened Koopman in the standardised feature basis: K_feat = W0^-1 K Wt^-1 -> use the regression matrix C00^-1 C0t on standardised features
        (mean,scale),C00,C0t,Ctt=__import__('view_search').covs(allF,lag)
        Kreg=np.linalg.solve(C00+1e-6*np.trace(C00)/len(C00)*np.eye(len(C00)),C0t)
        cross=np.sqrt(np.linalg.norm(Kreg[np.ix_(RIGID,INTERNAL)],'fro')**2+np.linalg.norm(Kreg[np.ix_(INTERNAL,RIGID)],'fro')**2)/np.linalg.norm(Kreg,'fro')
        print(f"    A1: purity of top-6 singular functions {[f'{x:.2f}' for x in pur]} (S1' stake: every one ≥ 0.9; kill: any < 0.7) | cross-block ‖K_RI‖/‖K‖ = {cross:.3f} (S2' stake < 0.1; kill ≥ 0.3)")
        print(f"{label}lag {tau:5.1f} fs: dictionary σ {np.array2string(s[:6],precision=3)} | top-3 R² rigid {[f'{a:.2f}' for a,b in w]} internal {[f'{b:.2f}' for a,b in w]} | rigid view held-out {rs:.2f}/bound {rb:.2f} = {rs/rb:.2f}; internal {is_:.2f}/{ib:.2f} = {is_/ib:.2f} | own top σ: rigid {s_rig:.3f} internal {s_int:.3f}")
        if abs(tau-50)<1e-9:
            wr=np.mean([a for a,b in w])
            print(f"  S1: top-3 weight in the rigid subspace {wr:.2f} (stake ≥ 0.9, kill < 0.7): {'MET' if wr>=0.9 else ('KILL' if wr<0.7 else 'between')}")
            print(f"  S2: internal view's own top σ at 50 fs {s_int:.3f} (stake < 0.5, kill ≥ 0.8): {'MET' if s_int<0.5 else ('KILL' if s_int>=0.8 else 'between')}")
            print(f"  S3: rigid view fraction of the bound {rs/rb:.2f} (stake ≥ 0.8, kill < 0.6): {'MET' if rs/rb>=0.8 else ('KILL' if rs/rb<0.6 else 'between')}")
def plants():
    rng=np.random.default_rng(7); dt=1.0; T=2000; n=16
    # PM-3: angular velocity on a rigid synthetic molecule
    om0=np.array([0.3,-0.2,0.5])*1e-3; r_oh=1.81; th0=1.82
    geom=np.array([[0,0,0],[r_oh,0,0],[r_oh*math.cos(th0),r_oh*math.sin(th0),0]])
    m=np.array([M_O,M_H,M_H]); com=(geom*m[:,None]).sum(0)/m.sum(); rel=geom-com
    P=np.zeros((2,3,3)); V=np.zeros((2,3,3))
    for t in range(2):
        P[t]=geom; V[t]=np.cross(om0,rel)
    f=features(P,V,np.array([8,1,1]),dt)[0]; ok3=np.allclose(f[0,9:12],om0,atol=1e-9)
    print(f"PM-3: omega read {f[0,9:12]} vs imposed {om0} -> {'PASS' if ok3 else 'FAIL'}")
    # PM-1: rigid rotors (slow OU angular/COM velocities, tau 200 fs) + fast stretches (period 9 fs)
    feats=[]
    for i in range(n):
        x=np.zeros((T,12)); v=np.zeros(6)
        for t in range(T):
            v=v*math.exp(-dt/200)+rng.normal(size=6)*math.sqrt(1-math.exp(-2*dt/200)); x[t,6:]=v
        ph=rng.uniform(0,2*math.pi,3); w=2*math.pi/9.0; tt=np.arange(T)*dt
        for j in range(3): x[:,j]=np.cos(w*tt+ph[j]); x[:,3+j]=-w*np.sin(w*tt+ph[j])
        x+=rng.normal(size=x.shape)*0.05
        feats.append(x)
    print("PM-1 (synthetic rotors + 9 fs stretches; expect rigid weight ≥ 0.95 at 50 fs, stretch σ ≈ |cos(ω·50 fs)| = %.3f):"%abs(math.cos(w*50)))
    search(feats,dt,[50.0],label="  ")
    # PM-2: shuffled rows
    sh=[f[rng.permutation(T)] for f in feats]
    allF=np.concatenate([f-f.mean(0) for f in sh]); sc=allF.std(0); allF/=sc
    lag=50; half=len(allF)//2
    scsh=heldout_score(allF[:half],allF[half:],lag,6)[0]; bound=score_k(koopman(np.concatenate([f-f.mean(0) for f in feats])/sc,lag)["s"],6)
    ok2=scsh<0.05*bound; print(f"PM-2: shuffled held-out {scsh:.3f} vs bound {bound:.3f} ({scsh/bound:.3f}, stake < 0.05) -> {'PASS' if ok2 else 'FAIL'}")
    return ok3 and ok2
if __name__=="__main__":
    if sys.argv[1]=="plants": sys.exit(0 if plants() else 1)
    P,V,z,L,dt=load(sys.argv[2]); print(f"# MOLSEARCH-1 on {sys.argv[2]}: {len(P)} rows at {dt:.3f} fs, {len(z)} atoms, {len(molecules(z))} molecules")
    search(features(P,V,z,dt),dt,[5,10,20,50,100])
