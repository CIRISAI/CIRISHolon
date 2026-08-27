import numpy as np, itertools, math
I=1j
def bits(x): return [(x>>k)&1 for k in range(4)]  # careful with ordering

# index x as 4-bit tuple (x1,x2,x3,x4); basis index = x1*8+x2*4+x3*2+x4
def idx(x1,x2,x3,x4): return x1*8+x2*4+x3*2+x4

s1=np.zeros(16,complex); s2=np.zeros(16,complex); s3=np.zeros(16,complex)
for x1,x2,x3,x4 in itertools.product([0,1],repeat=4):
    j=idx(x1,x2,x3,x4)
    if x1==x3: s1[j]=2**-1.5 * (I**x1) * ((-1)**(x2*x4))
    s2[j]=2**-2 * (I**(x2+x4)) * ((-1)**(x1*x3+x2*x4))
    if x2==x4: s3[j]=2**-1.5 * (I**(x1+x2+x3)) * ((-1)**(x1*x3))

for n,s in [("s1",s1),("s2",s2),("s3",s3)]:
    print(n,"norm",round(np.linalg.norm(s),10))

c1=(2/3)*np.exp(I*np.pi/12); c2=2/3+0j; c3=(2/3)*np.exp(-I*np.pi/12)
combo=c1*s1+c2*s2+c3*s3

# FACE state (BK05 "T-type" = QPG |F>): cos(2b)=1/sqrt3
b=0.5*math.acos(1/math.sqrt(3))
face=np.array([math.cos(b), np.exp(I*np.pi/4)*math.sin(b)])
F4=np.kron(np.kron(face,face),np.kron(face,face))
# PI/8 state (BG16/QPG |T>, the Clifford+T one)
t=np.array([1,np.exp(I*np.pi/4)])/math.sqrt(2)
T4=np.kron(np.kron(t,t),np.kron(t,t))

def fid(a,b_):
    ov=abs(np.vdot(a,b_)); return ov/(np.linalg.norm(a)*np.linalg.norm(b_))
print("\ncombo norm       :",round(np.linalg.norm(combo),10))
print("|<combo|FACE^4>| :",round(fid(combo,F4),12), " max-abs-diff:",round(np.max(np.abs(combo-F4)),12))
print("|<combo|T_pi8^4>|:",round(fid(combo,T4),12))
