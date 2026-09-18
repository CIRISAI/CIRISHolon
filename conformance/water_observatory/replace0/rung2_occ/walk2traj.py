#!/usr/bin/env python3
# REPLACE-0's banked oxygen walks, written as HLNTRAJ1 v1 trajectories so the FROZEN rung-2
# instrument (holon-lens/examples/rung2.rs, unchanged) can read them.
#
# DECLARED, because it bypasses a guard: TrajWriter::create asserts n_atoms <= 16 (the v1
# bond bitset is C(16,2) bits). The READER has no such check and the occupancy chart never
# reads bonds, so a file with 128 oxygens and an EMPTY bond set is read exactly and the
# position chart is honest. Consequences a reader of the log must carry:
#   - bonds are EMPTY: rung1 (the network chart) must never be pointed at these files;
#   - velocities are ZERO: the Mom and Ene rungs are degenerate copies of Occ and are NOT
#     readings — only "Spatial Occ" lines mean anything;
#   - positions are the walk's, wrapped into the box (cell_series refuses atoms outside);
#   - the grid is the freeze's 2D ladder, so at 128 waters in 3D the cells are COLUMNS
#     through the box (x,y only) — a coarse view, declared as such.
import struct, sys
S=sys.argv[1]; B=sys.argv[2]
SEEDS=[0x5245504c41434530,0x5245504c41434531,0x5245504c41434532]
FS=20.0; AU_FS=0.024188843265857; dt_au=FS/AU_FS
import os
def walk(p, tag):
    L=open(p).read().splitlines(); h=L[0].split(); assert h[1]==tag, (p, h)
    return float(h[3]),[[float(x) for x in ln.split()] for ln in L[1:]]
SRC=sys.argv[3] if len(sys.argv)>3 else "transport_seed"
NS=int(sys.argv[4]) if len(sys.argv)>4 else 3
ARMS=sys.argv[6].split(",") if len(sys.argv)>6 else ["flexible","rigid"]
for k in range(NS):
    for arm in ARMS:
        base=f"{B}/{SRC}{k}/{arm}"
        if not os.path.exists(base+".walk"): print(f"{arm}/seed{k}: no walk, skipped"); continue
        l,fr=walk(base+".walk","walk"); n=len(fr[0])//3
        vw=base+".vwalk"; has_v=os.path.exists(vw)
        vel=walk(vw,"vwalk")[1] if has_v else None
        if has_v: assert len(vel)==len(fr), "vwalk and walk disagree on readout count"
        # readout spacing in fs is not in the walk header; it is passed as argv[5] or defaults to 20
        fs=float(sys.argv[5]) if len(sys.argv)>5 else 20.0; dt=fs/AU_FS
        os.makedirs(f"{S}/{arm}",exist_ok=True)
        o=open(f"{S}/{arm}/seed{k}.traj","wb")
        o.write(b"HLNTRAJ1"+struct.pack("<IIII",1,n,3,1)+struct.pack("<QQ",SEEDS[k],len(fr))+struct.pack("<dddd",dt,l,l,l)+struct.pack("<%dI"%n,*([8]*n)))
        for i,f in enumerate(fr):
            o.write(struct.pack("<Qdd",i,i*dt,0.0)+(0).to_bytes(16,"little"))
            v=vel[i] if has_v else None
            for a in range(n):
                o.write(struct.pack("<ddd",*(f[3*a+c]%l for c in range(3))))
                o.write(struct.pack("<ddd",*(v[3*a:3*a+3] if has_v else (0.0,0.0,0.0))))
        o.close()
        print(f"{arm}/seed{k}.traj: {n} oxygens, {len(fr)} readouts at {fs} fs, velocities {'BANKED' if has_v else 'ZERO (older bundle; Mom/Ene degenerate)'}")
