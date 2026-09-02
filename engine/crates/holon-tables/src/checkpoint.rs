//! Survive a death: a region-granular, bit-exact checkpoint for the leased generator.
//!
//! # Why this exists, priced
//!
//! `generate_surface_*` runs a whole grid in one call, so before this module a death was a
//! TOTAL LOSS rather than a setback. The four-body production run made that concrete: at a
//! measured 1.24 effective cores it projected to ~78 hours of wall time with no resume and
//! no live progress — its only health signal was that the pid still existed — and it was
//! killed at 0.96% of its priced budget precisely because that is the cheapest the decision
//! could ever be taken. A run that survives nothing violates the detached-compute rule
//! outright, and M-PROBE-THE-RESOURCE names the blindness half.
//!
//! # Why REGIONS and not nodes
//!
//! The obvious design — checkpoint each node as it lands, replay the ones already done — is
//! WRONG here, and quietly so. Under `WarmPolicy::CanonicalChain` a region's first solved
//! node is cold and every later one starts from its canonical predecessor IN THAT SAME
//! REGION. Skipping a node on replay would deprive its successor of the vector it was
//! supposed to start from, so the successor would converge from somewhere else and land on
//! different last bits. The table would still be correct physics and would no longer be the
//! same artifact, which is the one thing this crate exists to prevent.
//!
//! So the unit is the REGION: it is either fully replayed from the log or fully re-solved
//! from cold, and in both cases its warm chain is exactly the chain it would have had in an
//! uninterrupted run. Bit-identity is preserved by construction rather than by tolerance.
//! The cost is bounded and known: a crash loses at most one in-flight region per worker.
//!
//! # Why hex — and a claim about the sibling that I had to retract
//!
//! Every field is written as raw hex, because the log has to carry every field
//! `GenOutcome::table_bytes()` compares and every field `Digest::of_record` hashes, and hex
//! makes that self-evidently exact rather than argued. `tests/checkpoint.rs` is the claim's
//! proof rather than its assertion.
//!
//! It is NOT because decimal would lose bits. I asserted that about `s2_ozone_table`'s
//! `{:.16e}` resume log and it is false: measured over 2,999,033 finite values — random bit
//! patterns and the stored energy range both — `{:.16e}` round-trips f64 exactly, zero
//! failures, because it is one digit before the point plus sixteen after and seventeen
//! significant digits is precisely f64's round-trip width. The reasoning was
//! "decimal, therefore lossy" without checking the width.
//!
//! What IS true of that generator is worse, and it is the reason this module's unit is the
//! region. Its carrier is updated only inside the branch that actually solves, so a knot
//! replayed from the log leaves the carrier stale and the NEXT knot on that ray starts from
//! the wrong vector — or cold. Different start, different convergence path, different last
//! bits and iteration count. The defect is the warm chain, not the number format, and a
//! per-node checkpoint here would have reproduced it exactly.
//!
//! # Torn writes
//!
//! A region is committed by an `END` line carrying its own digest. A region whose `END` is
//! missing or whose digest does not match its replayed records is DISCARDED and re-solved,
//! because a process killed mid-write leaves exactly that shape and a half-region silently
//! accepted is worse than the crash.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::Mutex;

use crate::digest::Digest;
use crate::grid::{NodeId, RegionId};
use crate::node::{NodeRecord, NodeStatus, VoidReason};

/// One region's committed records, plus the append handle that commits later ones.
pub struct Checkpoint {
    sink: Mutex<File>,
    replay: HashMap<RegionId, Vec<NodeRecord>>,
    /// Regions found in the log whose `END` line was missing or whose digest disagreed.
    torn: usize,
}

fn status_fields(r: &NodeRecord) -> (u64, u64, u64) {
    match r.status {
        NodeStatus::Ok => (0, 0, 0),
        NodeStatus::Void(VoidReason::BudgetExhausted) => (1, 0, 0),
        NodeStatus::Void(VoidReason::AboveLowestDiagonal {
            energy_bits,
            min_diagonal_bits,
        }) => (2, energy_bits, min_diagonal_bits),
        NodeStatus::Void(VoidReason::NotFinite) => (3, 0, 0),
        NodeStatus::Void(VoidReason::NotAGeometry) => (4, 0, 0),
        NodeStatus::Void(VoidReason::Unrealisable) => (5, 0, 0),
    }
}

fn status_from(code: u64, a: u64, b: u64) -> Option<NodeStatus> {
    Some(match code {
        0 => NodeStatus::Ok,
        1 => NodeStatus::Void(VoidReason::BudgetExhausted),
        2 => NodeStatus::Void(VoidReason::AboveLowestDiagonal {
            energy_bits: a,
            min_diagonal_bits: b,
        }),
        3 => NodeStatus::Void(VoidReason::NotFinite),
        4 => NodeStatus::Void(VoidReason::NotAGeometry),
        5 => NodeStatus::Void(VoidReason::Unrealisable),
        _ => return None,
    })
}

fn hx(s: &str) -> Option<u64> {
    u64::from_str_radix(s, 16).ok()
}

impl Checkpoint {
    /// Open a checkpoint at `path`, replaying whatever complete regions it already holds.
    ///
    /// A missing file is not an error: it is the first run.
    pub fn open(path: &Path) -> std::io::Result<Checkpoint> {
        let mut replay: HashMap<RegionId, Vec<NodeRecord>> = HashMap::new();
        let mut torn = 0usize;
        if path.exists() {
            let mut pending: Option<(RegionId, Vec<NodeRecord>)> = None;
            for line in BufReader::new(File::open(path)?).lines() {
                let line = line?;
                let f: Vec<&str> = line.split_whitespace().collect();
                match f.first().copied() {
                    Some("BEGIN") => {
                        // A BEGIN with a region still pending means the previous region
                        // never committed: the process died inside it.
                        if pending.take().is_some() {
                            torn += 1;
                        }
                        if let Some(r) = f.get(1).and_then(|x| x.parse::<RegionId>().ok()) {
                            pending = Some((r, Vec::new()));
                        }
                    }
                    Some("NODE") => {
                        if let Some((_, recs)) = pending.as_mut() {
                            if f.len() >= 11 {
                                let node = f[1].parse::<NodeId>().ok();
                                let e = hx(f[2]);
                                let d1 = hx(f[3]);
                                let d2 = hx(f[4]);
                                let di = f[5].parse::<u32>().ok();
                                let ci = f[6].parse::<u32>().ok();
                                let ec = f[7].parse::<u8>().ok();
                                let sc = hx(f[8]);
                                let sa = hx(f[9]);
                                let sb = hx(f[10]);
                                let wm = f.get(11).map(|x| *x == "1");
                                let mr = f.get(12).map(|x| *x == "1");
                                if let (
                                    Some(node),
                                    Some(energy_bits),
                                    Some(d1_bits),
                                    Some(d2_bits),
                                    Some(davidson_iters),
                                    Some(cg_iters),
                                    Some(exit_code),
                                    Some(sc),
                                    Some(sa),
                                    Some(sb),
                                    Some(warm),
                                    Some(mirrored),
                                ) = (node, e, d1, d2, di, ci, ec, sc, sa, sb, wm, mr)
                                {
                                    if let Some(status) = status_from(sc, sa, sb) {
                                        recs.push(NodeRecord {
                                            node,
                                            energy_bits,
                                            d1_bits,
                                            d2_bits,
                                            davidson_iters,
                                            cg_iters,
                                            exit_code,
                                            status,
                                            warm,
                                            mirrored,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    Some("END") => {
                        let claimed = f.get(2).copied().unwrap_or("");
                        match pending.take() {
                            Some((r, recs))
                                if f.get(1).and_then(|x| x.parse::<RegionId>().ok()) == Some(r) =>
                            {
                                let d = recs.iter().fold(Digest::ZERO, |a, x| {
                                    a.merge(Digest::of_record(x))
                                });
                                if d.hex() == claimed {
                                    replay.insert(r, recs);
                                } else {
                                    // A torn or corrupted region is re-solved, never used.
                                    torn += 1;
                                }
                            }
                            Some(_) => torn += 1,
                            None => {}
                        }
                    }
                    _ => {}
                }
            }
            if pending.is_some() {
                torn += 1;
            }
        }
        let sink = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Checkpoint {
            sink: Mutex::new(sink),
            replay,
            torn,
        })
    }

    /// How many complete regions were replayed from an earlier run.
    pub fn replayed_regions(&self) -> usize {
        self.replay.len()
    }

    /// How many regions in the log were incomplete or failed their digest, and so will be
    /// re-solved. Non-zero after any death; reported rather than swallowed.
    pub fn torn_regions(&self) -> usize {
        self.torn
    }

    /// How many node records were replayed.
    pub fn replayed_nodes(&self) -> usize {
        self.replay.values().map(|v| v.len()).sum()
    }

    /// This region's committed records, if it completed in an earlier run.
    pub fn region(&self, r: RegionId) -> Option<&Vec<NodeRecord>> {
        self.replay.get(&r)
    }

    /// Commit one region. Called after every node in it has landed, never before: the `END`
    /// line is what makes the region replayable, and writing it early is how a half-region
    /// would be accepted as whole.
    pub fn commit(&self, r: RegionId, recs: &[NodeRecord]) {
        let mut out = String::with_capacity(64 * recs.len() + 64);
        out.push_str(&format!("BEGIN {r}\n"));
        let mut d = Digest::ZERO;
        for x in recs {
            let (sc, sa, sb) = status_fields(x);
            d = d.merge(Digest::of_record(x));
            out.push_str(&format!(
                "NODE {} {:016x} {:016x} {:016x} {} {} {} {:016x} {:016x} {:016x} {} {}\n",
                x.node,
                x.energy_bits,
                x.d1_bits,
                x.d2_bits,
                x.davidson_iters,
                x.cg_iters,
                x.exit_code,
                sc,
                sa,
                sb,
                u8::from(x.warm),
                u8::from(x.mirrored),
            ));
        }
        out.push_str(&format!("END {r} {}\n", d.hex()));
        if let Ok(mut f) = self.sink.lock() {
            let _ = f.write_all(out.as_bytes());
            // Flushed per region, not per run: an unflushed checkpoint is not a checkpoint.
            let _ = f.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(node: NodeId, e: u64) -> NodeRecord {
        NodeRecord {
            node,
            energy_bits: e,
            d1_bits: e ^ 0x5555,
            d2_bits: e ^ 0xAAAA,
            davidson_iters: 7,
            cg_iters: 3,
            exit_code: 2,
            status: NodeStatus::Void(VoidReason::AboveLowestDiagonal {
                energy_bits: e,
                min_diagonal_bits: e + 1,
            }),
            warm: true,
            mirrored: false,
        }
    }

    #[test]
    fn a_committed_region_round_trips_every_field() {
        let dir = std::env::temp_dir().join(format!("ckpt_rt_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let p = dir.join("c.log");
        let _ = std::fs::remove_file(&p);
        let recs = vec![rec(1, 0x3FF0_0000_0000_0000), rec(2, 0xBFE0_0000_0000_0001)];
        {
            let c = Checkpoint::open(&p).unwrap();
            c.commit(4, &recs);
        }
        let c = Checkpoint::open(&p).unwrap();
        assert_eq!(c.replayed_regions(), 1);
        assert_eq!(c.torn_regions(), 0);
        // Every field, not just the energy: the record is compared for bit-identity and
        // hashed, so a field that does not survive the round trip is a silent divergence.
        assert_eq!(c.region(4).unwrap(), &recs);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn a_region_whose_end_never_landed_is_discarded_not_half_accepted() {
        let dir = std::env::temp_dir().join(format!("ckpt_torn_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let p = dir.join("c.log");
        let _ = std::fs::remove_file(&p);
        {
            let c = Checkpoint::open(&p).unwrap();
            c.commit(1, &[rec(1, 9)]);
        }
        // Simulate a death mid-region: a BEGIN and a NODE with no END, which is exactly
        // what a killed process leaves behind.
        {
            let mut f = OpenOptions::new().append(true).open(&p).unwrap();
            writeln!(f, "BEGIN 2").unwrap();
            writeln!(
                f,
                "NODE 5 {:016x} {:016x} {:016x} 1 1 0 {:016x} {:016x} {:016x} 0 0",
                1u64, 2u64, 3u64, 0u64, 0u64, 0u64
            )
            .unwrap();
        }
        let c = Checkpoint::open(&p).unwrap();
        assert_eq!(c.replayed_regions(), 1, "only region 1 committed");
        assert!(c.region(2).is_none(), "a region with no END must not replay");
        assert_eq!(c.torn_regions(), 1, "the torn region must be REPORTED, not swallowed");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn a_corrupted_region_fails_its_own_digest() {
        let dir = std::env::temp_dir().join(format!("ckpt_corrupt_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let p = dir.join("c.log");
        let _ = std::fs::remove_file(&p);
        {
            let c = Checkpoint::open(&p).unwrap();
            c.commit(3, &[rec(1, 0x1234), rec(2, 0x5678)]);
        }
        // Flip one hex digit in a NODE line, leaving the END digest as it was.
        let src = std::fs::read_to_string(&p).unwrap();
        let bad = src.replacen("0000000000001234", "0000000000001235", 1);
        assert_ne!(src, bad, "the corruption must actually change the bytes");
        std::fs::write(&p, bad).unwrap();
        let c = Checkpoint::open(&p).unwrap();
        assert_eq!(c.replayed_regions(), 0, "a region that fails its digest must not replay");
        assert_eq!(c.torn_regions(), 1);
        let _ = std::fs::remove_file(&p);
    }
}
