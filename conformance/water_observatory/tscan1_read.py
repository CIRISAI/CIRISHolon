#!/usr/bin/env python3
"""TSCAN-1 reader (TSCAN1_PREREG.md): D(T) from the oxygen MSD, the density-pattern persistence
r = |offset|/sd over the first-harmonic density modes, the rigid-mode temperature check.
  tscan1_read.py plants | tscan1_read.py read"""
import sys, os, math, glob, re
import numpy as np
BOHR = 5.29177210903e-11; FS = 20.0
def load(base, tag):
    L=None; out=[]
    for ext in ("walk","vwalk"):
        lines=open(f"{base}/{tag}.{ext}").read().splitlines(); h=lines[0].split(); L=float(h[3])
        out.append(np.array([np.array(l.split(),float).reshape(-1,3) for l in lines[1:]]))
    return out[0], out[1], L
def msd(P, lags):
    return np.array([((P[l:]-P[:-l])**2).sum(2).mean() for l in lags])   # bohr^2, over atoms and origins
def read_D(P, dt_fs=FS):
    lags=[l for l in range(25,126,5)]                                    # 0.5-2.5 ps
    m=msd(P,lags); t=np.array(lags)*dt_fs*1e-15
    slope, icpt = np.polyfit(t, m*BOHR**2, 1)
    lin = 1.0 - np.sqrt(((m*BOHR**2-(slope*t+icpt))**2).mean())/ (m*BOHR**2).mean()
    return slope/6.0, lin, m*BOHR**2
def persistence(P, L):
    k=2*math.pi/L; X=P%L; r=[]
    for ax in range(3):
        for f in (np.cos,np.sin):
            m=f(k*X[...,ax]).sum(1); r.append(abs(m.mean())/max(m.std(),1e-300))
    return float(np.mean(r))
def plants():
    rng=np.random.default_rng(3); D=2.3e-9; n=64; T=2000
    step=math.sqrt(2*D*FS*1e-15)/BOHR
    P=np.cumsum(rng.normal(size=(T,n,3))*step,axis=0)
    d,lin,_=read_D(P); ok1=abs(d/D-1)<0.03
    print(f"PT-1: synthetic D {D:.2e} read {d:.3e} (linearity {lin:.3f}) -> {'PASS' if ok1 else 'FAIL'}")
    L=30.0; X=rng.uniform(0,L,size=(1,n,3))+rng.normal(size=(T,n,3))*0.3     # frozen positions + jitter
    r=persistence(X,L)
    k=2*math.pi/L; exp=[]
    for ax in range(3):
        for f in (np.cos,np.sin): m=f(k*(X[...,ax]%L)).sum(1); exp.append(abs(m.mean())/m.std())
    ok2=abs(r/np.mean(exp)-1)<0.05 and r>3
    print(f"PT-2: frozen pattern r read {r:.2f} vs direct {np.mean(exp):.2f} -> {'PASS' if ok2 else 'FAIL'}")
    return ok1 and ok2
def read():
    B=os.path.join(os.path.dirname(__file__),"replace0","tscan1")
    print("# TSCAN-1 read\n#  T(K) seed  T_rigid(K)  D (m^2/s)  linearity  r=|offset|/sd  MSD(2.5ps) A^2")
    res={}
    for d in sorted(glob.glob(f"{B}/T*_seed*")):
        T=int(re.search(r"T(\d+)_",d).group(1)); seed=int(d[-1])
        if not os.path.exists(f"{d}/rigid.walk") or os.path.getsize(f"{d}/rigid.walk")<1000: print(f"  {T} {seed}: no walk yet"); continue
        try: P,V,L=load(d,"rigid")
        except Exception as e: print(f"  {T} {seed}: unreadable ({e})"); continue
        err=open(f"{d}/scout.err").read(); m=re.findall(r"scout readout\s+\d+ at .*?: T\s+([\d.]+) K", err); Tr=float(np.mean([float(x) for x in m])) if m else float('nan')
        if len(P)<130: print(f"  {T} {seed}: {len(P)} rows, too few for the 2.5 ps lag"); continue
        D,lin,ms=read_D(P); r=persistence(P,L)
        flag="" if abs(Tr/T-1)<=0.10 else "  REFUSED (temperature off target by > 10 %)"
        void="" if lin>=0.7 else "  VOID (MSD not linear to 30 %)"
        print(f"  {T:4d} {seed}   {Tr:7.1f}   {D:.3e}   {lin:.3f}    {r:.2f}    {ms[-1]/1e-20:.2f}{flag}{void}")
        res.setdefault(T,[]).append((D,r,lin,Tr))
    if 293 in res and 500 in res:
        D293=np.mean([x[0] for x in res[293]]); D500=np.mean([x[0] for x in res[500]]); r500=np.mean([x[1] for x in res[500]])
        print(f"\nS1: D(293) = {D293:.2e} vs 0.3e-9 (kill >= 1.0e-9): {'MET' if D293<0.3e-9 else ('KILL' if D293>=1e-9 else 'between - neither met nor killed')}")
        print(f"S2: D(500)/D(293) = {D500/max(D293,1e-300):.1f} (needs > 5) and r(500) = {r500:.2f} (needs < 0.5): {'MET' if D500/max(D293,1e-300)>5 and r500<0.5 else ('KILL (flat within 2x)' if D500/max(D293,1e-300)<2 else 'not met')}")
if __name__=="__main__": sys.exit(0 if (plants() if sys.argv[1]=="plants" else (read() or True)) else 1)
