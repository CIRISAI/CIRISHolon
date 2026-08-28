#!/usr/bin/env python3
"""OMEGA-RATCHET-1 — one runnable check of every claim in OMEGA_RATCHET1_NOTE.md.

Exact rational arithmetic throughout (fractions.Fraction).  Floats appear only
in display columns, never in a decision.  Run: python3 verify.py
"""
from fractions import Fraction as F
from itertools import product

# ---- the two faces, transcribed from source -------------------------------
# MAGNITUDE  CIRISHolon/lean/CIRISHolon/Object.lean
def rentStep(lam,q,s0,s): return (1-q)*lam*s + q*s0
def Ginf(lam,q):          return q/((1-lam)+q*lam)
def Wstar(gam,dl):        return (1-dl)*gam/(gam+dl*(1-gam))
# PARTITION  CIRISHolon/conformance/omega/CROSSFACE1_PREREG.md Theorem 1
def W(mu,K):
    return 1 - sum(max(mu[i]*K[i][j] for j in range(len(mu))) for i in range(len(mu)))
def W_minF(mu,K):                       # Theorem 1's (<=) half, independently
    N=len(mu); P=[[mu[i]*K[i][j] for j in range(N)] for i in range(N)]
    return min(1-sum(P[i][f[i]] for i in range(N)) for f in product(range(N),repeat=N))

def registry(gam,q):
    """S={0,1}: 0 = entry gone, 1 = entry stands.  Refresh w.p. q else decay w.p. gam."""
    K=[[1-q, q],[(1-q)*gam, 1-(1-q)*gam]]
    G=q/(q+gam*(1-q)); return [1-G,G], K, (1-q)*(1-gam), G

FAIL=[]
def chk(name,ok,extra=""):
    print(("  PASS  " if ok else "  FAIL  ")+name+(("   "+str(extra)) if extra else ""))
    if not ok: FAIL.append(name)

GAMS=[F(1,10),F(1,4),F(1,3),F(2,5),F(1,2),F(7,10),F(9,10)]
QS  =[F(1,20),F(1,10),F(1,5),F(1,3),F(1,2),F(3,5),F(4,5),F(19,20)]

print("R1  THE THREE IDENTIFICATIONS (magnitude face == the chain, exactly)")
a=b=c=True
for gam in GAMS:
    for q in QS:
        mu,K,lamv,G=registry(gam,q); lam=1-gam
        a &= (Ginf(lam,q)==G); c &= (lamv==(1-q)*lam)
        s=F(0); pr=F(0)
        for _ in range(14):
            s=rentStep(lam,q,F(1),s); pr=pr*K[1][1]+(1-pr)*K[0][1]; b &= (s==pr)
        for i in range(2):
            for j in range(2):
                c &= (K[i][j]==lamv*(i==j)+(1-lamv)*mu[j])
chk("Ginf(lam,q) == mu_on  (stationary occupancy)",a)
chk("rentOrbit(n) == Pr[entry stands at n]  (14 steps)",b)
chk("lambda(v_id) == (1-q)*lam == rent_closed_form's transient base,"
    " and K == lam*I+(1-lam)*Pi so ||M-Pi|| on 1-perp is lam",c)

print("\nR2  THE EXCHANGE RATE  W(v_id) = 2*(1-G)*q = 2*G*(1-G)*(1-lambda)")
d=e=True
for gam in GAMS:
    for q in QS:
        mu,K,lamv,G=registry(gam,q); Wb=W(mu,K); Phi=G*(1-G)*(1-lamv)
        d &= (Wb==W_minF(mu,K)) and (Wb==min(2*Phi,1-G,G,1-2*Phi))
        if q<=F(1,2) and gam*(1-q)<=F(1,2): e &= (Wb==2*(1-G)*q==2*Phi)
chk("Theorem 1's two halves agree; W == min(2Phi, 1-G, G, 1-2Phi) always",d)
chk("in regime R1 (q<=1/2 and gam(1-q)<=1/2): W == 2(1-G)q == 2G(1-G)(1-lam)",e)
cen={"R1":0,"R2":0,"R3":0,"R4":0}; f=True
for gn in range(1,12):
    for qn in range(1,12):
        gam,q=F(gn,12),F(qn,12); mu,K,lamv,G=registry(gam,q); Wb=W(mu,K)
        A,B=(q<=F(1,2)),(gam*(1-q)<=F(1,2))
        if A and B:  cen["R1"]+=1; f&=(Wb==2*G*(1-G)*(1-lamv))
        elif B:      cen["R2"]+=1; f&=(Wb==1-G)
        elif A:      cen["R3"]+=1; f&=(Wb==G)
        else:        cen["R4"]+=1
chk("trichotomy exact on the 11x11 grid; R4 (both>1/2) is EMPTY",f and cen["R4"]==0,cen)

print("\nR3  W = 2*delta*Wstar(gam,delta), and the maximum")
g=True; h=True
for gam in GAMS:
    for dl in [F(1,20),F(1,10),F(1,4),F(1,3),F(1,2),F(2,3),F(4,5)]:
        q=Wstar(gam,dl); mu,K,lamv,G=registry(gam,q)
        g &= (G==1-dl)
        if q<=F(1,2) and gam*(1-q)<=F(1,2): g &= (W(mu,K)==2*dl*q)
for r in [F(1,2),F(1,3),F(1,4),F(2,5),F(3,4)]:
    gam=r*r; ds=r/(1+r)
    h &= (Wstar(gam,ds)==ds)
    h &= (max(2*F(k,400)*Wstar(gam,F(k,400)) for k in range(1,400)) <= 2*gam/(1+r)**2)
chk("Ginf_at_Wstar re-derived, and W == 2*delta*Wstar exactly in R1",g)
chk("argmax: delta* = q* = sqrt(gam)/(1+sqrt(gam)); W_max = 2gam/(1+sqrt(gam))^2",h)

print("\nR4  GINI RENT — exact rent of M = lam*I + (1-lam)*Pi on N blocks")
def urelax(mu,lam): return [[lam*(i==j)+(1-lam)*mu[j] for j in range(len(mu))]
                            for i in range(len(mu))]
def gini_closed(mu,lam):
    mm=max(mu); return 1-sum(m*max(lam+(1-lam)*m,(1-lam)*mm) for m in mu)
MUS=[[F(1,2)]*2,[F(3,4),F(1,4)],[F(1,3)]*3,[F(1,2),F(1,3),F(1,6)],
     [F(1,8),F(1,8),F(1,4),F(1,2)],[F(1,5)]*5,[F(9,10),F(1,20),F(1,20)],
     [F(1,10),F(2,10),F(3,10),F(4,10)],[F(1,4)]*4,[F(2,5),F(2,5),F(1,5)]]
i1=i2=i3=True; nin=nout=0
for mu in MUS:
    for lam in [F(k,12) for k in range(13)]:
        K=urelax(mu,lam); Wb=W(mu,K)
        i1 &= (Wb==gini_closed(mu,lam)==W_minF(mu,K))
        # CROSS-FACE-1 Theorem 2 must not be violated
        sig=[ (m*(1-m)) for m in mu]
        import math
        i3 &= (float(Wb) >= (1-max(mu)) - float(lam)*sum(math.sqrt(float(s)) for s in sig)
                                          *max(math.sqrt(float(s)) for s in sig) - 1e-12)
        if lam >= (1-lam)*(max(mu)-min(mu)):
            nin+=1; i2 &= (Wb==(1-sum(m*m for m in mu))*(1-lam))
        else: nout+=1
chk("closed form W = 1 - sum_i mu_i*max(lam+(1-lam)mu_i, (1-lam)mu_max) exact on 130 cells",i1)
chk("== (1 - sum mu_i^2)(1-lam) under the fence lam >= (1-lam)(mu_max-mu_min)",i2,
    f"{nin} in-fence, {nout} out")
chk("CROSS-FACE-1 Theorem 2's inequality is respected on every cell",i3)
chk("contains Theorem 3 (uniform mu): (1-1/N)(1-lam)",
    all(gini_closed([F(1,N)]*N,l)==(1-F(1,N))*(1-l) for N in (2,3,4,5)
        for l in [F(k,12) for k in range(13)]))
chk("contains Theorem 6's lam=0 form: 1 - mu_max",
    all(gini_closed(m,F(0))==1-max(m) for m in MUS))
muD4=[F(1,8),F(1,8),F(2,8),F(2,8),F(2,8)]
chk("reproduces CROSS-FACE-1's MEASURED D4 v_classA = 3/4 at lam=0",
    gini_closed(muD4,F(0))==F(3,4),
    f"naive Gini form would give {(1-sum(m*m for m in muD4))} -- the fence is load-bearing")

print("\nR5  THE FIBER MODEL — S = V x Phi, measure-preserving motion, uniform mu")
S=[(v,f) for v in (0,1) for f in (0,1)]; ix={s:i for i,s in enumerate(S)}
def motion(gam,p):
    K=[[F(0)]*4 for _ in range(4)]
    for (v,f) in S:
        for v2,pv in [(v,1-gam),(1-v,gam)]:
            for f2,pf in [(f,1-p),(1-f,p)]: K[ix[(v,f)]][ix[(v2,f2)]]+=pv*pf
    return K
def coarse(K,mu,blocks):
    mb=[sum(mu[i] for i in b) for b in blocks]
    return mb,[[sum(mu[i]*sum(K[i][j] for j in b2) for i in b1)/mb[k]
                for b2 in blocks] for k,b1 in enumerate(blocks)]
mu4=[F(1,4)]*4; j1=True
for gam in [F(1,10),F(1,5),F(1,3),F(1,2)]:
    for p in [F(1,20),F(1,10),F(1,4),F(1,2)]:
        K=motion(gam,p)
        WV=W(*coarse(K,mu4,[[ix[(0,0)],ix[(0,1)]],[ix[(1,0)],ix[(1,1)]]]))
        WF=W(mu4,K)
        j1 &= (WV==gam) and (WF==gam+p*(1-gam)) and (WF-WV==p*(1-gam)) and (WF==W_minF(mu4,K))
chk("W(v_view) = gam ; W(v_full) = gam + p(1-gam) ; surcharge = p(1-gam) exactly",j1)

def maint(gam,p,q,mode):
    K=[[F(0)]*4 for _ in range(4)]
    for (v,f) in S:
        for v2,pv in [(v,1-gam),(1-v,gam)]:
            for f2,pf in [(f,1-p),(1-f,p)]:
                for rep,pq in [(True,q),(False,1-q)]:
                    v3=1 if rep else v2
                    f3=(f if rep else f2) if mode=='pol' else (0 if rep else f2)
                    K[ix[(v,f)]][ix[(v3,f3)]]+=pv*pf*pq
    return K
def align(K,n):
    v=[F(0)]*4; v[ix[(1,0)]]=F(1); o=[]
    for _ in range(n):
        v=[sum(v[i]*K[i][j] for i in range(4)) for j in range(4)]
        o.append(v[ix[(0,0)]]+v[ix[(1,0)]])
    return o
k1=k2=True
for gam in [F(1,5),F(1,3)]:
    for p in [F(3,20),F(1,10),F(2,5)]:
        for q in [F(1,50),F(1,10),F(1,3),F(7,10),F(19,20)]:
            r=1-2*p*(1-q); rd=(1-q)*(1-2*p); fs=(q+(1-q)*p)/(1-(1-q)*(1-2*p))
            A=align(maint(gam,p,q,'pol'),120); D=align(maint(gam,p,q,'des'),120)
            k1 &= all(A[n]==(1+r**(n+1))/2 for n in range(120))
            k2 &= all(D[n]==fs+rd**(n+1)*(1-fs) for n in range(120))
chk("design-BLIND deposit: alignment(n) = (1+(1-2p(1-q))^n)/2 -> 1/|Phi| at every q<1",k1)
chk("design-KNOWING deposit: alignment(n) = f* + ((1-q)(1-2p))^n (1-f*),"
    " f* = (q+(1-q)p)/(1-(1-q)(1-2p)) > 1/|Phi| for every q>0",k2)

print("\n" + ("ALL CHECKS PASS" if not FAIL else "FAILURES: "+", ".join(FAIL)))

# --------------------------------------------------------------------------
# The display tables quoted in OMEGA_RATCHET1_NOTE.md, regenerated from source.
# --------------------------------------------------------------------------
print("\nT1  sec.3 anti-correlation table (gam = 1/4)")
print("     delta      W* (magnitude price)        W (partition rent)")
for dl in [F(1,100),F(1,20),F(1,10),F(1,5),F(1,3),F(1,2),F(2,3),F(99,100)]:
    q=Wstar(F(1,4),dl); Wv=2*dl*q
    print(f"     {str(dl):>7}   {str(q):>10} = {float(q):.4f}   {str(Wv):>12} = {float(Wv):.4f}")
print("     crossing W == W* iff 2*delta == 1:", all(
    (2*dl*Wstar(F(1,4),dl)==Wstar(F(1,4),dl)) == (dl==F(1,2))
    for dl in [F(k,20) for k in range(1,20)]))

print("\nT2  sec.3 maximum, at rational squares gam = r^2")
for r in [F(1,2),F(1,3),F(1,4),F(2,5),F(3,4)]:
    gam=r*r; ds=r/(1+r)
    print(f"     gam={str(gam):>5}  delta* = q* = {str(ds):>4}   W_max = {str(2*gam/(1+r)**2):>6}")

print("\nT3  sec.6 fiber limits (gam = 1/5, p = 3/20)")
p=F(3,20)
print("     q        design-BLIND limit   design-KNOWING plateau f*(q)")
for q in [F(1,50),F(1,10),F(1,3),F(7,10),F(19,20)]:
    fs=(q+(1-q)*p)/(1-(1-q)*(1-2*p))
    print(f"     {str(q):>5}      1/2 = 0.500000     {str(fs):>10} = {float(fs):.6f}")
print("     f*(q=0) =", (0+1*p)/(1-1*(1-2*p)), " (the floor)   f*(q=1) = 1")

print("\nT4  sec.7.5 the disanalogy that does NOT pass"
      "  (measured slopes, HOLONOMY_RENT_RESULTS.md sec.6)")
print("     q        |d log f/d log R|    slope/(1-q)   [flat iff the toy rate law held]")
for q,s in [(0.0345,1.5540),(0.1,1.1084),(0.3,0.2758),(0.7,0.0270)]:
    print(f"     {q:<7}  {s:>10.4f}          {s/(1-q):.4f}")
print("     not flat: 1.6095 -> 0.0900, an 18x fall.  Shape reproduced, rate law NOT.")
