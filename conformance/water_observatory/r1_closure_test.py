#!/usr/bin/env python3
"""The face-centred and lagged closures for R1 (2026-09-21): on a full RESPONSE-1 arm, predict each
cell's aligned occupancy change over the lead windows three ways and report the amplitude
fidelity alpha = <obs.pred>/<pred^2> and R^2 of the aligned coherent parts:
  A  slab-mean momentum at the faces (the chart as staked; rung2's integral form)
  B  face-centred: the x-velocity sum of molecules within +/-h of the face, over 2h  (h in A)
  C  slab-mean momentum lagged by ell readouts (10 fs each)
usage: r1_closure_test.py DIR   (DIR holds rigid.walk / rigid.vwalk)"""
import sys, numpy as np
AU_FS = 0.024188843265857; DT_FS = 10.0; DT_AU = DT_FS / AU_FS; BOHR_A = 0.529177
def load(d):
    P=[];V=[]
    for ext,store in (("walk",P),("vwalk",V)):
        L=open(f"{d}/rigid.{ext}").read().splitlines(); Lbox=float(L[0].split()[3])
        for ln in L[1:]: store.append(np.array(ln.split(),float).reshape(-1,3))
    return np.array(P),np.array(V),Lbox
def run(d, cycles=12, relax=314, lead=2):
    P,V,L=load(d); T=len(P); n=P.shape[1]; X=P[...,0]%L; vx=V[...,0]
    print(f"# {d}: {T} rows, {n} oxygens, L = {L:.2f} bohr = {L*BOHR_A:.1f} A")
    for nx in (4,8,16):
        edge=L/nx; w=int(round((edge*BOHR_A*1e-10/1500.0)/(DT_FS*1e-15)))   # window = a / c_s(water), as staked
        cell=(np.floor(X/edge).astype(int))%nx
        occ=np.zeros((T,nx)); pc=np.zeros((T,nx))
        for c in range(nx): m=(cell==c); occ[:,c]=m.sum(1); pc[:,c]=(vx*m).sum(1)
        faces=(np.arange(nx)+1)*edge   # face f sits between cell f and cell f+1 (periodic)
        def face_series(h_bohr):
            F=np.zeros((T,nx))
            for f in range(nx):
                dx=X-faces[f]; dx-=L*np.round(dx/L); m=np.abs(dx)<h_bohr; F[:,f]=(vx*m).sum(1)/(2*h_bohr)
            return F   # number flux density per unit time at each face (molecules / au time)
        methods={"A slab-mean":None}
        for hA in (0.25,0.5,1.0,2.0): methods[f"B face h={hA} A"]=face_series(hA/BOHR_A)
        def pred_from_face_flux(F, t0, t1, shift=0):
            # net inflow to each cell over [t0,t1]: flux at left face minus flux at right face, trapezoid in time
            out=np.zeros(nx)
            for c in range(nx):
                fl=(c-1)%nx; fr=c
                s=0.0
                for i in range(t0,t1):
                    a,b=max(i-shift,0),max(i+1-shift,0)
                    s+=0.5*((F[a,fl]+F[b,fl])-(F[a,fr]+F[b,fr]))
                out[c]=s*DT_AU
            return out
        Fslab=np.zeros((T,nx))
        for f in range(nx): Fslab[:,f]=0.5*(pc[:,f]+pc[:,(f+1)%nx])/edge
        rows=[]
        for name,F in methods.items():
            Fuse=Fslab if F is None else F
            for shift in ([0,3,5,10] if F is None else [0]):
                obs_al=np.zeros((lead,nx)); pred_al=np.zeros((lead,nx)); tail_o=np.zeros((lead,nx)); tail_p=np.zeros((lead,nx))
                for cyc in range(cycles):
                    sgn=1 if cyc%2==0 else -1; start=1+cyc*relax
                    for k in range(lead):
                        t0=start+k*w; t1=t0+w
                        obs_al[k]+=sgn*(occ[t1]-occ[t0]); pred_al[k]+=sgn*pred_from_face_flux(Fuse,t0,t1,shift)
                        tt0=start+relax-1-(lead-k)*w; tt1=tt0+w
                        tail_o[k]+=sgn*(occ[tt1]-occ[tt0]); tail_p[k]+=sgn*pred_from_face_flux(Fuse,tt0,tt1,shift)
                o=obs_al.ravel()/cycles; p=pred_al.ravel()/cycles; to=tail_o.ravel()/cycles; tp=tail_p.ravel()/cycles
                alpha=float(o@p/max(p@p,1e-300)); r2=float((o@p)**2/max((o@o)*(p@p),1e-300)); D=float(np.sqrt(((o-p)**2).sum()/max((o**2).sum(),1e-300)))
                Dt=float(np.sqrt(((to-tp)**2).sum()/max((to**2).sum(),1e-300)))
                lab=name if shift==0 else f"C slab-mean lagged {shift*DT_FS:.0f} fs"
                rows.append((lab,alpha,r2,D,Dt,np.sqrt((o**2).mean()),np.sqrt((to**2).mean())))
        print(f"\n== {nx} cells (slab {edge*BOHR_A:.2f} A, window {w*DT_FS:.0f} fs): aligned lead-window obs RMS {rows[0][5]:.3f}, tail RMS {rows[0][6]:.3f} counts")
        print(f"   {'closure':28} {'alpha':>6} {'R2':>6} {'D_lead':>7} {'D_tail':>7}")
        for lab,a,r2,D,Dt,_,_ in rows: print(f"   {lab:28} {a:6.2f} {r2:6.2f} {D:7.3f} {Dt:7.3f}")
if __name__=="__main__": run(sys.argv[1])
