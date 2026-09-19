#!/usr/bin/env python3
"""GF1's x^2 law, DERIVED (GF1_RESULTS.md section 5). At strong coupling the massless Schwinger
vacuum is the staggered stabilizer state; the hopping x(s+ s- + h.c.) creates one pair across a
link at electric cost 1, so to first order |psi> = |vac> - x sum_n |pair_n>. On one link that is
the two-qubit state |01> - x|10>, whose stabilizer 2-Renyi entropy is EXACTLY
    M2(theta) = -log2(1 - 1/4 sin^2 4 theta),  tan theta = x,
(Pauli expectations: ZZ = -1, Z1 = -Z2 = cos 2theta, XX = YY = sin 2theta), so per link
    M2 = (4 / ln 2) x^2 + O(x^4) = 5.7708 x^2.
Checked here by brute-force SRE on the two-qubit state and on the first-order vacuum at N = 6, 8.
The ladder read c(1/64) = 0.00141 = 5.775 x^2 and c(1/16) = 0.02161 = 5.53 x^2 (second order).
"""
import numpy as np, itertools, math
I=np.eye(2); X=np.array([[0,1],[1,0]]); Y=np.array([[0,-1j],[1j,0]]); Z=np.diag([1,-1]); P=[I,X,Y,Z]
def sre2(psi):
    n=int(round(math.log2(len(psi)))); s=0.0
    for idx in itertools.product(range(4),repeat=n):
        op=P[idx[0]]
        for k in idx[1:]: op=np.kron(op,P[k])
        s+=np.vdot(psi,op@psi).real**4
    return -math.log2(s/2**n)
def first_order_vacuum(N,x):
    conf=[1 if k%2==0 else 0 for k in range(N)]        # q_k = 0: sz_k = -(-1)^k
    idx=lambda c:int("".join(map(str,c)),2)
    psi=np.zeros(2**N); psi[idx(conf)]=1.0
    for n in range(N-1):
        c=conf.copy(); c[n],c[n+1]=c[n+1],c[n]; psi[idx(c)]+=-x   # one pair on link n, cost 1
    return psi/np.linalg.norm(psi)
if __name__=="__main__":
    for x in [1/64,1/16,1/4]:
        psi=np.zeros(4); psi[1]=1; psi[2]=-x; psi/=np.linalg.norm(psi)
        th=math.atan(x)
        print(f"x={x:g}: two-qubit M2 {sre2(psi):.6f} = -log2(1-1/4 sin^2 4θ) {-math.log2(1-0.25*math.sin(4*th)**2):.6f}; leading 4x²/ln2 {4*x*x/math.log(2):.6f}")
    for N in [6,8]:
        for x in [1/64,1/16]:
            m=sre2(first_order_vacuum(N,x)); print(f"N={N} x={x:g}: first-order vacuum M2/link {m/(N-1):.6f} vs 4x²/ln2 {4*x*x/math.log(2):.6f}")
