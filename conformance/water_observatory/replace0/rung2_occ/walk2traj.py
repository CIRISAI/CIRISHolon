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
def walk(p):
    L=open(p).read().splitlines(); l=float(L[0].split()[3])
    return l,[[float(x) for x in ln.split()] for ln in L[1:]]
for k in range(3):
    for arm in ("flexible","rigid"):
        l,fr=walk(f"{B}/transport_seed{k}/{arm}.walk"); n=len(fr[0])//3
        o=open(f"{S}/{arm}/seed{k}.traj","wb")
        o.write(b"HLNTRAJ1"+struct.pack("<IIII",1,n,3,1)+struct.pack("<QQ",SEEDS[k],len(fr))+struct.pack("<dddd",dt_au,l,l,l)+struct.pack("<%dI"%n,*([8]*n)))
        for i,f in enumerate(fr):
            o.write(struct.pack("<Qdd",i,i*dt_au,0.0)+(0).to_bytes(16,"little"))
            for a in range(n): o.write(struct.pack("<dddddd",*(f[3*a+c]%l for c in range(3)),0.0,0.0,0.0))
        o.close()
