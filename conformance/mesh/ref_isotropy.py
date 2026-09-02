import numpy as np, itertools
def tensors(c, name):
    c=np.array(c,dtype=float); n=len(c)
    T2=np.einsum('ia,ib->ab',c,c)
    T4=np.einsum('ia,ib,ig,id->abgd',c,c,c,c)
    d=np.eye(2)
    iso=np.einsum('ab,gd->abgd',d,d)+np.einsum('ag,bd->abgd',d,d)+np.einsum('ad,bg->abgd',d,d)
    A=T4[0,0,0,0]/3.0
    resid=np.abs(T4-A*iso).max()
    print(f"{name}: n={n}  T2={T2.round(10).tolist()}  T4_xxxx={T4[0,0,0,0]:.6f} T4_xxyy={T4[0,0,1,1]:.6f} A={A:.6f}  max|T4-A*iso|={resid:.3e}")
    return resid
hexd=[(np.cos(np.pi/3*k), np.sin(np.pi/3*k)) for k in range(6)]
sq=[(1,0),(0,1),(-1,0),(0,-1)]
r6=tensors(hexd,"FHP-6 hex (Euclidean embedding)")
r4=tensors(sq,"HPP-4 square")
# axial <-> euclidean map
AX=[(1,0),(0,1),(-1,1),(-1,0),(0,-1),(1,-1)]
M=np.array([[1.0,0.5],[0.0,np.sqrt(3)/2]])
emb=[tuple((M@np.array(a,dtype=float)).round(12)) for a in AX]
print("axial->euclidean via M=[[1,1/2],[0,sqrt3/2]]:", [tuple(round(x,6) for x in e) for e in emb])
print("matches hex unit vectors:", np.allclose(np.array(emb), np.array(hexd)))
print("det M =", round(float(np.linalg.det(M)),12), "(invertible => axial conservation <=> euclidean conservation)")
