import math
M=160; INF=float('inf')
def dp(base, chain=None, magic5=False):
    c=[INF]*(M+1); c[0]=1
    for m in range(1,M+1):
        best=base.get(m,INF)
        if chain:
            r=chain(m)
            if r: best=min(best,r)
        for a in range(1,m//2+1):
            if c[a]<INF and c[m-a]<INF: best=min(best,c[a]*c[m-a])
        if magic5 and m>=5 and c[m-4]<INF: best=min(best,3*c[m-4])
        c[m]=best
    return c
def chain(m):
    return 2*3**((m-2)//4) if m>=10 and (m-2)%4==0 else None
base_qpg={1:2,2:2,3:3,4:4,5:6,6:6,7:12,8:12}
ours  = dp({1:2,2:2})
bg16  = dp({1:2,2:2,3:3,4:4,5:6,6:7})
qpg   = dp(base_qpg, chain)
m5    = dp(base_qpg, chain, magic5=True)   # + Magic5FromCat 4-to-3 (quizx)

print(f"{'t':>4} {'ours 2^0.5t':>18} {'BG16 6->7':>16} {'QPG chain':>14} {'+Magic5 4to3':>14}  {'gain(M5)':>9}")
for t in [16,24,32,40,48,56,64,80,100]:
    print(f"{t:>4} {ours[t]:>18,.0f} {bg16[t]:>16,.0f} {qpg[t]:>14,.0f} {m5[t]:>14,.0f}  {ours[t]/m5[t]:>8.1f}x")
print("\neffective alpha = log2(terms)/t")
for t in [40,64,100]:
    print(f"  t={t:>3}: ours {math.log2(ours[t])/t:.4f}  BG16 {math.log2(bg16[t])/t:.4f}  "
          f"QPGchain {math.log2(qpg[t])/t:.4f}  Magic5 {math.log2(m5[t])/t:.4f}")
print("\nsanity: log2(3)/4 =",round(math.log2(3)/4,5)," log2(7)/6 =",round(math.log2(7)/6,5),
      " log2(6)/6 =",round(math.log2(6)/6,5))
