import numpy as np, itertools, math
I=1j
def ket(m,vec):
    out=vec
    for _ in range(m-1): out=np.kron(out,vec)
    return out
t=np.array([1,np.exp(I*np.pi/4)])/math.sqrt(2)          # |T> pi/8 magic state
tp=np.array([1,-np.exp(I*np.pi/4)])/math.sqrt(2)        # |T_perp> = Z|T>

def cat(m):
    return (ket(m,t)+ket(m,tp))/math.sqrt(2)

def E(m):
    v=np.zeros(2**m,complex)
    for x in range(2**m):
        if bin(x).count('1')%2==0: v[x]=2**(-(m-1)/2)
    return v
def K(m):
    v=E(m).copy()
    for x in range(2**m):
        b=[(x>>(m-1-k))&1 for k in range(m)]
        ph=sum(b[i]*b[j] for i in range(m) for j in range(i+1,m))
        v[x]*=(-1)**ph
    return v

# --- cat2 ---
c2=cat(2)
claim2=np.zeros(4,complex); claim2[0]=2**-0.5; claim2[3]=I*2**-0.5
print("cat2 == 2^-1/2(|00>+i|11>) ? maxdiff",round(np.max(np.abs(c2-claim2)),12))

# --- cat6 : 3 stabilizer terms ---
c6=cat(6)
z0=np.zeros(64,complex); z0[0]=1.0
z1=np.zeros(64,complex); z1[63]=1.0
claim6 = 2**-1.5*(z0-I*z1) + 2**-0.5*np.exp(3j*np.pi/4)*(E(6)+I*K(6))
print("cat6 3-term identity  maxdiff",round(np.max(np.abs(c6-claim6)),12),
      " |<.|.>|",round(abs(np.vdot(c6,claim6)),12))
for nm,v in [("E6",E(6)),("K6",K(6))]: print("   ",nm,"norm",round(np.linalg.norm(v),10))

# --- chi(T^6) <= 6 :  |T>^6  propto |cat6> + (A tensor I)|cat6> ---
A=np.exp(-1j*np.pi/4)*np.array([[1,0],[0,1j]])@np.array([[0,1],[1,0]])  # e^{-i pi/4} S X
AI=np.kron(A,np.eye(32))
v=c6+AI@c6
T6=ket(6,t)
ov=abs(np.vdot(v/np.linalg.norm(v),T6))
print("T^6 from cat6+(A x I)cat6 : fidelity",round(ov,12),"  => chi(T^6)<=6")

# --- chain contraction: <cat2|_{6,7} |cat6>|cat6>  propto |cat10> ---
c66=np.kron(c6,c6)                       # 12 qubits, order q1..q6,q7..q12
# contract qubits 6 and 7 (0-indexed 5 and 6) with <cat2|
psi=c66.reshape([2]*12)
bra=np.conj(c2).reshape(2,2)
out=np.einsum('abcdeFGhijkl,FG->abcdehijkl', psi, bra)
out=out.reshape(-1)
c10=cat(10)
print("chain: <cat2|_{6,7}(cat6 x cat6) propto cat10 ? fidelity",
      round(abs(np.vdot(out/np.linalg.norm(out),c10)),12))
