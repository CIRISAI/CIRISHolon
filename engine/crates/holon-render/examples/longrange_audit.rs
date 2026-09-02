//! THE LONG-RANGE RESIDUAL AUDIT — GANTT node B1's instrument.
//!
//! Freeze: `conformance/water_observatory/LONGRANGE_PREREG.md`, ADMITTED by
//! `Audit/prereg_audit.py` and committed BEFORE this file existed. Nothing here may add a
//! gate, a cutoff, a denominator or a branch the freeze does not carry; where the freeze's
//! one-line wording needed an execution decision, the decision is commented at the site and
//! reported in `LONGRANGE_RESULTS.md` rather than made silently.
//!
//! ```text
//! cargo run --release -p holon-render --example longrange_audit -- \
//!     --class=hydrogen|fenced [--root=DIR] [--manifest=FILE] [--stride=N] [--plant]
//! ```
//!
//! # What it measures
//!
//! The census scenes declare no pair cutoff (`Sim::pair_switch == None`), so the engine's
//! pair sector runs the complete `N²/2` sum and discards nothing. B1's question is
//! therefore the counterfactual the freeze states: what WOULD a declared truncation drop on
//! these configurations, at the radii this engine actually uses? For each frame and each
//! staked radius `c`:
//!
//! ```text
//! E_band(c)   = Σ over pairs with c < r ≤ r_max(a,b)  of  u_ab(r)
//! E_tail(c)   = Σ over pairs with     r > r_max(a,b)  of  u_ab(r)
//! E_hard(c)   = E_band + E_tail
//! E_switch(c) = Σ over pairs with r > c−W  of  (1 − S₂(r; c−W, c))·u_ab(r)
//! ```
//!
//! `u_ab` comes from the SAME curves the sim served, regenerated through the same
//! `generate_pair_table` call at the same knot count and loaded through the same
//! `load_pair_table` door, then read out of the engine's own bank. There is no second
//! implementation of the pair potential anywhere in this file.
//!
//! **`E_hard` is a LOWER BOUND.** Past the last knot the table is an exponential matched in
//! value and slope (`table.rs:315`) while the true tail is a power law, so the table
//! understates what a real truncation would discard. `E_tail_pow6` and `E_tail_pow3` — the
//! power-law tails matched to the table's own last value, nothing fitted — are printed
//! beside it so the fence has a size and not only a warning.
//!
//! # What it refuses
//!
//! Every `.traj` and every arm log is hashed and compared against
//! `conformance/water_observatory/census_traj_manifest.sha256`. A digest mismatch, or a path
//! the manifest does not list, is REFUSED by name with both digests printed and contributes
//! zero frames. The refusal reason is a printed field, never an omission
//! (M-EXIT-DISCRIMINATOR), and the refusal count is printed whether or not any fired.

use holon_chem::elements::{Species, HYDROGEN, OXYGEN};
use holon_chem::pair::generate_pair_table;
use holon_lens::traj::Trajectory;
use holon_render::bank::Host;
use holon_render::cells::switch_c2;
use holon_render::sim::{Boundary, Dims, Sim, PAIR_SWITCH_WIDTH};
use holon_render::{load_pair_table, TABLE_OK};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

// ============================================================== THE STAKED CONSTANTS
//
// Every one of these is quoted from LONGRANGE_PREREG.md, which is committed. A value that
// disagrees with the freeze is a defect in this file, not a new decision.

/// The staked cutoff ladder, bohr. Provenance, in order: `quaternary_table::R_HI` (the four-body reach the engine derives its cutoff from),
/// `trimer::R_HI`, half the short box edge (the largest cutoff `pbc_ok` would admit on a
/// periodic box this size), `ooh::R_HI` = `ozone::R_HI`, `water::R_HI`, and one radius
/// above the box diagonal as the zero control.
const LADDER: [f64; 6] = [6.0, 9.0, 10.4, 14.0, 15.0, 41.0];

/// Index of `c* = 15.0` in `LADDER` — `three_body_cutoff() = list_cutoff()` at the commit
/// that produced the parked artifacts. The primary.
const C_STAR: usize = 4;

/// Index of the zero control, `c = 41.0` bohr.
const C_ZERO: usize = 5;

/// Knots per curve — the protocol's `CURVE_KNOTS`.
const CURVE_KNOTS: usize = 96;

/// The protocol's box, bohr. Only the bank is used from the `Sim`, but the scene is set up
/// as the protocol set it up so that nothing about the load path differs.
const BOX_W: f64 = 34.6;
const BOX_H: f64 = 20.8;

/// G1's fraction of the scene's own drift bound.
const NEGLIGIBLE_FRACTION: f64 = 0.10;

/// Branch (e)'s bound-looseness threshold: `B_s / D_s` above this is an uninformative bound.
const UNINFORMATIVE_RATIO: f64 = 1.0e3;

/// G4's floor on frames scored per class.
const MIN_FRAMES: usize = 50;

/// G7's floor: the fraction of scored frames that must have a nonempty beyond-`c*`
/// population, or the class was never tested.
const MIN_POPULATED_FRACTION: f64 = 0.01;

/// The plant's staked separation, bohr — just outside `c*`.
const PLANT_R: f64 = 16.0;

/// G2's tolerances against the committed arm log's printed digits. Digits, not a digest:
/// a floating-point summation-order change downstream moves every pinned digest without
/// moving any physics, and a digest gate would fire on a correct curve.
const G2_DR_E: f64 = 1.0e-4;
const G2_DD_E: f64 = 1.0e-6;
const G2_RESIDUAL_FACTOR: f64 = 2.0;

/// B1's price refusal for the mixed class: the O–O curve was measured at 2596.2 s in the
/// committed arm log, and a setup that finishes under half of the priced total is refused
/// as not having generated that curve.
///
/// THIS GATE FIRED AND WAS WRONG TO. It is denominated in wall clock seconds, which cannot
/// separate "did less work" from "did the same work quicker" — see LONGRANGE_RESULTS.md
/// §7.4. It is kept, unchanged and still enforced under `--freeze=b1`, because B1's verdict
/// is a fact about B1 and deleting the gate that produced it would erase the record.
/// `--freeze=b1b` replaces it with W1 + W2 below.
const MIXED_PRICE_FLOOR_S: f64 = 1200.0;

// ---------------------------------------------------------------- B1b's staked constants
//
// From B1B_PREREG.md, committed at `1569288` BEFORE this block existed.

/// B1b's work-unit price: the expensive curve's cost in units of the cheap curve solved by
/// the same kernel, in the same process. A kernel speedup scales both, contention scales
/// both, core placement scales both — so the ratio is work-proportional and survives all
/// three, which is exactly what B1's wall-clock floor did not.
///
/// The floor sits 5.5× below the lowest ratio measured on three prior runs (555.2, 741.8,
/// 944.1 — spread 1.70× against the absolute second-count's 2.74×), so it catches an
/// artifact with no solve behind it (ratio ≈ 1) and not a legitimate kernel improvement.
const B1B_COST_RATIO_FLOOR: f64 = 100.0;

/// B1b's second denominator: the drift the run actually INCURRED, not the bound it was
/// entitled to. Gated at the same 0.10 fraction, inherited unchanged from B1.
const B1B_UNINFORMATIVE_RATIO: f64 = 1.0e3;

/// B1b's plant tolerance. Looser than B1's 1e-12 because `E_switch` sums a switch function
/// over a band rather than a step over a tail, so its floating-point path is longer.
/// Declared in the freeze rather than discovered afterwards.
const B1B_PLANT_TOL: f64 = 1.0e-9;

/// Which freeze's gates this run is under.
#[derive(Clone, Copy, PartialEq)]
enum Freeze {
    /// LONGRANGE_PREREG.md: `E_hard` gated against `0.10·B_s`, price in wall clock seconds.
    B1,
    /// B1B_PREREG.md: `E_switch` gated against `0.10·B_s` AND `0.10·D_s`, price in work units.
    B1b,
}

impl Freeze {
    fn parse(s: &str) -> Option<Freeze> {
        match s {
            "b1" => Some(Freeze::B1),
            "b1b" => Some(Freeze::B1b),
            _ => None,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Freeze::B1 => "B1 (LONGRANGE_PREREG.md)",
            Freeze::B1b => "B1b (B1B_PREREG.md)",
        }
    }
}

/// Staked curve targets, quoted from LONGRANGE_PREREG.md §5, themselves quoted from the
/// committed arm logs. `(symbol pair, R_e, D_e, worst residual)`.
const CURVE_TARGETS: [(&str, f64, f64, f64); 3] = [
    ("H-H", 1.3887, 0.204142, 8.7e-11),
    ("O-H", 1.9909, 0.122901, 9.9e-11),
    ("O-O", 2.4421, 0.147621, 2.7e-6),
];

// ====================================================================== sha256, inlined
//
// The workspace has no hashing dependency and the discipline that keeps it that way is
// worth more than this function is long. Gated against the standard empty-input vector at
// startup, and — the check that actually matters — against the manifest itself, which was
// produced by `sha256sum`: if this implementation were wrong, every file would be REFUSED
// rather than silently admitted, so its failure mode is loud.

const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
    0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
    0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
    0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
    0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
    0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
    0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
    0xc67178f2,
];

struct Sha256 {
    h: [u32; 8],
    buf: [u8; 64],
    len: usize,
    total: u64,
}

impl Sha256 {
    fn new() -> Self {
        Self {
            h: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
                0x1f83d9ab, 0x5be0cd19,
            ],
            buf: [0u8; 64],
            len: 0,
            total: 0,
        }
    }

    fn block(&mut self) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                self.buf[4 * i],
                self.buf[4 * i + 1],
                self.buf[4 * i + 2],
                self.buf[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) = (
            self.h[0], self.h[1], self.h[2], self.h[3], self.h[4], self.h[5], self.h[6],
            self.h[7],
        );
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K256[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        self.h[0] = self.h[0].wrapping_add(a);
        self.h[1] = self.h[1].wrapping_add(b);
        self.h[2] = self.h[2].wrapping_add(c);
        self.h[3] = self.h[3].wrapping_add(d);
        self.h[4] = self.h[4].wrapping_add(e);
        self.h[5] = self.h[5].wrapping_add(f);
        self.h[6] = self.h[6].wrapping_add(g);
        self.h[7] = self.h[7].wrapping_add(hh);
    }

    fn update(&mut self, mut data: &[u8]) {
        self.total = self.total.wrapping_add(data.len() as u64);
        while !data.is_empty() {
            let take = (64 - self.len).min(data.len());
            self.buf[self.len..self.len + take].copy_from_slice(&data[..take]);
            self.len += take;
            data = &data[take..];
            if self.len == 64 {
                self.block();
                self.len = 0;
            }
        }
    }

    fn hex(mut self) -> String {
        let bits = self.total.wrapping_mul(8);
        self.update(&[0x80]);
        while self.len != 56 {
            self.update(&[0x00]);
        }
        // `update` moved `total`; the length field is the pre-padding one captured above.
        self.buf[56..64].copy_from_slice(&bits.to_be_bytes());
        self.len = 64;
        self.block();
        self.len = 0;
        let mut s = String::with_capacity(64);
        for v in self.h {
            s.push_str(&format!("{v:08x}"));
        }
        s
    }
}

fn sha256_file(p: &Path) -> std::io::Result<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(p)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.hex())
}

fn sha256_selftest() {
    let empty = Sha256::new().hex();
    assert_eq!(
        empty, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "the inlined sha256 fails the standard empty-input vector; every refusal it \
         produced would be meaningless"
    );
    let mut h = Sha256::new();
    h.update(b"abc");
    assert_eq!(
        h.hex(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        "the inlined sha256 fails the standard \"abc\" vector"
    );
}

// ====================================================================== the arm log

/// One seed's row from the committed arm log: the run's own drift peak and drift bound.
#[derive(Clone, Copy, Debug)]
struct SeedBound {
    peak: f64,
    bound: f64,
}

/// Parse `seed 0x…  …  drift <peak>/<bound>  …` out of the committed arm log.
///
/// The bound is READ, never recomputed. Recomputing `Sim::drift_bound()` from a frame would
/// give a DIFFERENT number: the bound is built from running maxima the trajectory earned
/// (`k_pair_max`, `e_ref`, the wall/spring engagement flags), which a single frame does not
/// carry. The number that gates B1 is the one the run itself reported.
fn parse_arm_log(text: &str) -> BTreeMap<u64, SeedBound> {
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let toks: Vec<&str> = line.split_whitespace().collect();
        if toks.first() != Some(&"seed") {
            continue;
        }
        let Some(seed_tok) = toks.get(1) else { continue };
        let Ok(seed) = u64::from_str_radix(seed_tok.trim_start_matches("0x"), 16) else {
            continue;
        };
        let Some(pos) = toks.iter().position(|t| *t == "drift") else {
            continue;
        };
        let Some(field) = toks.get(pos + 1) else { continue };
        let Some((p, b)) = field.split_once('/') else {
            continue;
        };
        if let (Ok(peak), Ok(bound)) = (p.parse::<f64>(), b.parse::<f64>()) {
            out.insert(seed, SeedBound { peak, bound });
        }
    }
    out
}

// ====================================================================== the classes

#[derive(Clone, Copy, PartialEq)]
enum Class {
    Hydrogen,
    MixFenced,
}

impl Class {
    fn parse(s: &str) -> Option<Class> {
        match s {
            "hydrogen" => Some(Class::Hydrogen),
            "fenced" => Some(Class::MixFenced),
            _ => None,
        }
    }
    /// The freeze's class name, used in every printed row.
    fn label(self) -> &'static str {
        match self {
            Class::Hydrogen => "CLASS-H",
            Class::MixFenced => "CLASS-MIX-FENCED",
        }
    }
    fn dir(self) -> &'static str {
        match self {
            Class::Hydrogen => "hydrogen",
            Class::MixFenced => "fenced",
        }
    }
    fn curves(self) -> Vec<(Species, Species)> {
        match self {
            Class::Hydrogen => vec![(HYDROGEN, HYDROGEN)],
            Class::MixFenced => vec![
                (HYDROGEN, HYDROGEN),
                (OXYGEN, HYDROGEN),
                (OXYGEN, OXYGEN),
            ],
        }
    }
    /// The freeze's price floor for the curve setup, seconds.
    fn price_floor(self) -> f64 {
        match self {
            // The freeze prices CLASS-H's single H–H curve at 3.5–7.0 s and states no
            // refusal for it: a floor below a few seconds cannot discriminate a real solve
            // from a stub on this box, so none is claimed. The mixed class's O–O curve is
            // where the price has teeth.
            Class::Hydrogen => 0.0,
            Class::MixFenced => MIXED_PRICE_FLOOR_S,
        }
    }
}

// ====================================================================== the estimator

/// One frame's readings, at every staked radius.
#[derive(Clone, Default)]
struct FrameRow {
    band: [f64; 6],
    tail: [f64; 6],
    hard: [f64; 6],
    switched: [f64; 6],
    /// Pairs with `r > c*` — the population the discard is drawn from (G7).
    beyond_cstar: usize,
    /// Largest pair separation in the frame, for the zero control's own audit.
    r_max_seen: f64,
    /// The two power-law fence tails at `c*`, matched to the table's own last value.
    tail_pow6: f64,
    tail_pow3: f64,
}

/// The pair-table view the estimator reads: one slot index and one `r_max` per species pair.
struct Curves {
    /// `slot[a][b]` — index into the engine's own bank.
    slot: Vec<Vec<usize>>,
    /// `rmax[a][b]` — the last knot of that curve.
    rmax: Vec<Vec<f64>>,
    /// `u_edge[a][b]` — the table's value AT the last knot, the fence tails' only input.
    u_edge: Vec<Vec<f64>>,
    /// Distinct nuclear charges, in ascending order; an atom's index into these arrays.
    zs: Vec<u32>,
}

impl Curves {
    fn idx(&self, z: u32) -> usize {
        self.zs.iter().position(|v| *v == z).expect("a nuclear charge the class did not load")
    }
}

/// Score one frame. `pos` is in bohr; the separation is the scene's own convention.
///
/// MINIMUM IMAGE: the census scenes are `Boundary::Walls`, so `delta` is `b − a` and the
/// minimum-image convention degenerates to it (freeze §1.3). The wrap is written out anyway
/// and switched by `periodic`, so the estimator transfers unchanged to a periodic scene and
/// nobody has to remember to add it later.
#[allow(clippy::too_many_arguments)]
fn score_frame(
    pos: &[[f64; 3]],
    zidx: &[usize],
    cur: &Curves,
    bank: &holon_render::bank::PairBank,
    periodic: bool,
    box_whd: [f64; 3],
) -> FrameRow {
    let n = pos.len();
    let mut row = FrameRow::default();
    for i in 0..n {
        for j in (i + 1)..n {
            let mut d = [
                pos[j][0] - pos[i][0],
                pos[j][1] - pos[i][1],
                pos[j][2] - pos[i][2],
            ];
            if periodic {
                for k in 0..3 {
                    let l = box_whd[k];
                    if l > 0.0 {
                        d[k] -= l * (d[k] / l).round();
                    }
                }
            }
            let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if r > row.r_max_seen {
                row.r_max_seen = r;
            }
            let (a, b) = (zidx[i], zidx[j]);
            let t = bank.table_slot(cur.slot[a][b]);
            let u = t.u(r);
            let rmax = cur.rmax[a][b];
            if r > LADDER[C_STAR] {
                row.beyond_cstar += 1;
                if r > rmax {
                    // The fence, sized: the maximal power-law tails consistent with the
                    // table's own last value. Matched, never fitted.
                    let e = cur.u_edge[a][b];
                    row.tail_pow6 += e * (rmax / r).powi(6);
                    row.tail_pow3 += e * (rmax / r).powi(3);
                }
            }
            for (c, &cut) in LADDER.iter().enumerate() {
                if r > cut {
                    row.hard[c] += u;
                    if r <= rmax {
                        row.band[c] += u;
                    } else {
                        row.tail[c] += u;
                    }
                }
                let r_in = cut - PAIR_SWITCH_WIDTH;
                if r > r_in {
                    let (sw, _, _) = switch_c2(r, r_in, cut);
                    row.switched[c] += (1.0 - sw) * u;
                }
            }
        }
    }
    row
}

// ====================================================================== main

fn arg<'a>(args: &'a [String], key: &str) -> Option<&'a str> {
    args.iter().find_map(|a| a.strip_prefix(key))
}

fn main() {
    sha256_selftest();
    let args: Vec<String> = std::env::args().skip(1).collect();

    let class = arg(&args, "--class=")
        .and_then(Class::parse)
        .expect("--class=hydrogen|fenced is REQUIRED and has no default: the two classes \
                 are different scenes and a runner that picked one silently would make its \
                 own output unattributable");
    let root = PathBuf::from(
        arg(&args, "--root=").unwrap_or("/home/emoore/holon-artifacts/census-traj"),
    );
    let manifest_path = PathBuf::from(arg(&args, "--manifest=").unwrap_or(
        "conformance/water_observatory/census_traj_manifest.sha256",
    ));
    let stride: usize = arg(&args, "--stride=")
        .and_then(|s| s.parse().ok())
        .unwrap_or(400);
    let want_plant = args.iter().any(|a| a == "--plant");
    let freeze = arg(&args, "--freeze=")
        .and_then(Freeze::parse)
        .expect(
            "--freeze=b1|b1b is REQUIRED and has no default: the two freezes gate different \
             estimators against different denominators under different prices, and a runner \
             that picked one silently would make its own output unattributable",
        );

    println!("# longrange_audit — GANTT B1, freeze {}", freeze.label());
    println!("# class = {}   arm dir = {}", class.label(), class.dir());
    println!("# traj root = {}", root.display());
    println!("# manifest  = {}", manifest_path.display());

    // --- the manifest, and its own digest (M-PROVENANCE-OVERREACH: this names the FILE
    //     that was hashed and infers nothing about which run produced it).
    let manifest_text =
        std::fs::read_to_string(&manifest_path).expect("the committed manifest opens");
    println!(
        "# manifest sha256 = {}",
        sha256_file(&manifest_path).expect("the manifest hashes")
    );
    let mut want: BTreeMap<String, String> = BTreeMap::new();
    for line in manifest_text.lines() {
        let mut it = line.split_whitespace();
        if let (Some(d), Some(p)) = (it.next(), it.next()) {
            want.insert(p.to_string(), d.to_string());
        }
    }

    // --- the arm log, hashed like everything else, then read for the drift bounds.
    let log_key = format!("{}.log", class.dir());
    let log_path = root.join(&log_key);
    let mut refusals: Vec<(String, String)> = Vec::new();
    let bounds = match (sha256_file(&log_path), want.get(&log_key)) {
        (Ok(got), Some(w)) if &got == w => {
            println!("# ADMITTED {log_key}  sha256 {got}");
            parse_arm_log(&std::fs::read_to_string(&log_path).expect("the arm log reads"))
        }
        (Ok(got), Some(w)) => {
            println!("# REFUSED  {log_key}: sha256 {got} != manifest {w}");
            refusals.push((log_key.clone(), format!("sha256 {got} != manifest {w}")));
            BTreeMap::new()
        }
        (Ok(got), None) => {
            println!("# REFUSED  {log_key}: sha256 {got} but the manifest does not list this path");
            refusals.push((log_key.clone(), "not listed in manifest".to_string()));
            BTreeMap::new()
        }
        (Err(e), _) => {
            println!("# REFUSED  {log_key}: {e}");
            refusals.push((log_key.clone(), format!("{e}")));
            BTreeMap::new()
        }
    };

    // --- the curves. Same call, same knot count, same load door as the protocol.
    let mut sim = Box::new(Sim::empty());
    sim.boundary = Boundary::Walls;
    sim.dims = Dims::Two;
    sim.width = BOX_W;
    sim.height = BOX_H;
    let mut setup_s = 0.0f64;
    let mut g2_fail = 0usize;
    // W1/W2 state (B1b). `curve_secs` keys the in-run cost ratio; `w1_fail` counts curves
    // that could not show a solver certificate.
    let mut w1_fail = 0usize;
    let mut curve_secs: BTreeMap<String, f64> = BTreeMap::new();
    for (a, b) in class.curves() {
        let t = Instant::now();
        let pt = generate_pair_table(a, b, CURVE_KNOTS);
        let secs = t.elapsed().as_secs_f64();
        setup_s += secs;
        assert_eq!(
            load_pair_table(&mut sim, &pt, Host::Native),
            TABLE_OK,
            "the {}-{} curve did not load",
            a.symbol,
            b.symbol
        );
        let name = format!("{}-{}", a.symbol, b.symbol);
        curve_secs.insert(name.clone(), secs);
        // W1 — THE SOLVER CERTIFICATE (B1b). An analytic stub has no determinant space, no
        // route, no exit and no residual; a real curve carries all four on its own meta.
        // The three counts are PRINTED AND NOT GATED ON A VALUE: no prior record carries
        // them for these curves, and a freeze cannot gate a value it would have to invent.
        // Recording them here is what lets a successor gate them.
        println!(
            "# W1 {name}: route {:?}  exit {:?}  n_det {}  n_basis {}  solver_budget {}  \
             worst_residual {:.3e}",
            pt.meta.route,
            pt.meta.exit,
            pt.meta.n_det,
            pt.meta.n_basis,
            pt.meta.solver_budget,
            pt.meta.worst_residual
        );
        let w1_ok = pt.meta.route == holon_chem::fci::SolverRoute::Determinant
            && pt.meta.n_det >= 2
            && pt.meta.n_basis >= 2
            && pt.meta.solver_budget >= 1;
        if freeze == Freeze::B1b && !w1_ok {
            println!("# GATE W1 {name}: FAIL — no solver certificate (route/space/budget)");
            w1_fail += 1;
        }
        let (r_e, d_e) = match pt.meta.well {
            Some(w) => (w.r_e, w.d_e),
            None => (0.0, 0.0),
        };
        let slot = sim.bank.slot_of_z(a.z, b.z).expect("the curve registered a slot");
        let rmax = sim.bank.table_slot(slot).r_max();
        println!(
            "# curve {name}: {CURVE_KNOTS} knots, R_e = {r_e:.4} bohr, D_e = {d_e:.6} Ha, \
             worst residual {:.1e}, r_max = {rmax:.4} bohr, {secs:.1} s",
            pt.meta.worst_residual
        );
        // G2 — curve identity against the committed arm log's PRINTED digits.
        let Some(&(_, tr, td, tres)) = CURVE_TARGETS.iter().find(|c| c.0 == name) else {
            println!("# GATE G2 {name}: NO STAKED TARGET — refusing to grade an unstaked curve");
            g2_fail += 1;
            continue;
        };
        let (dr, dd) = ((r_e - tr).abs(), (d_e - td).abs());
        let rres = pt.meta.worst_residual / tres;
        let ok = dr <= G2_DR_E
            && dd <= G2_DD_E
            && rres <= G2_RESIDUAL_FACTOR
            && rres >= 1.0 / G2_RESIDUAL_FACTOR;
        println!(
            "# GATE G2 {name}: {}  dR_e {dr:.1e} (<= {G2_DR_E:.0e})  dD_e {dd:.1e} \
             (<= {G2_DD_E:.0e})  residual x{rres:.2} (within {G2_RESIDUAL_FACTOR:.0}x)",
            if ok { "PASS" } else { "FAIL" }
        );
        if !ok {
            g2_fail += 1;
        }
    }
    // THE PRICE (M-CHEAPER-THAN-ITS-PRICE), which is the whole difference between the two
    // freezes. B1 asks wall clock seconds; B1b asks work units.
    let price_ok = match freeze {
        Freeze::B1 => {
            let ok = setup_s >= class.price_floor();
            println!(
                "# GATE PRICE {}: setup {setup_s:.1} s against floor {:.0} s — {}",
                class.label(),
                class.price_floor(),
                if ok { "PASS" } else { "REFUSED (the curve was not generated)" }
            );
            ok
        }
        Freeze::B1b => {
            // W2 — the expensive curve in units of the cheap one, both solved by the same
            // kernel in the same process. Kernel speed, contention and placement all cancel.
            let hh = curve_secs.get("H-H").copied().unwrap_or(0.0);
            let oo = curve_secs.get("O-O").copied();
            let w2_ok = match (oo, hh > 0.0) {
                (Some(oo), true) => {
                    let ratio = oo / hh;
                    let ok = ratio >= B1B_COST_RATIO_FLOOR;
                    println!(
                        "# GATE W2 {}: t(O-O)/t(H-H) = {oo:.1}/{hh:.1} = {ratio:.1} against \
                         floor {B1B_COST_RATIO_FLOOR:.0} — {}",
                        class.label(),
                        if ok { "PASS" } else { "REFUSED (no solve behind the curve)" }
                    );
                    ok
                }
                _ => {
                    // A single-curve class has no ratio to take. The freeze says so rather
                    // than manufacturing a floor that cannot discriminate.
                    println!(
                        "# GATE W2 {}: NOT APPLICABLE — one curve, and a ratio needs two; \
                         W1 is this class's price evidence",
                        class.label()
                    );
                    true
                }
            };
            let ok = w2_ok && w1_fail == 0;
            println!(
                "# GATE W1 {}: {} ({w1_fail} curves without a solver certificate)",
                class.label(),
                if w1_fail == 0 { "PASS" } else { "FAIL" }
            );
            ok
        }
    };
    // NOT an abort. The freeze says the reading is "void with it", and VOID in this
    // campaign means NOT SCORED — it does not mean NOT COMPUTED. Aborting here would throw
    // away the VOID structure that M-BUDGET-LAUNDER exists to make visible, and would leave
    // the class with no number at all to look at. So the condition sets a VOID flag that
    // the verdict line reads, exactly as V1–V5 do, and everything is still printed.
    let price_void = !price_ok;

    // --- admit the trajectories.
    let dir = root.join(class.dir());
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|x| x == "traj").unwrap_or(false))
                .collect()
        })
        .unwrap_or_default();
    paths.sort();
    println!("# parked .traj files in {}: {}", dir.display(), paths.len());

    let mut admitted: Vec<PathBuf> = Vec::new();
    for p in &paths {
        let key = format!("{}/{}", class.dir(), p.file_name().unwrap().to_string_lossy());
        let got = match sha256_file(p) {
            Ok(g) => g,
            Err(e) => {
                println!("# REFUSED  {key}: unreadable: {e}");
                refusals.push((key, format!("unreadable: {e}")));
                continue;
            }
        };
        match want.get(&key) {
            Some(w) if *w == got => {
                println!("# ADMITTED {key}  sha256 {got}");
                admitted.push(p.clone());
            }
            Some(w) => {
                println!("# REFUSED  {key}: sha256 {got} != manifest {w}");
                refusals.push((key, format!("sha256 {got} != manifest {w}")));
            }
            None => {
                println!("# REFUSED  {key}: sha256 {got} but the manifest does not list this path");
                refusals.push((key, "not listed in manifest".to_string()));
            }
        }
    }
    println!(
        "# admitted {} of {} parked files; refusals {}",
        admitted.len(),
        paths.len(),
        refusals.len()
    );
    for (k, why) in &refusals {
        println!("# REFUSAL-REASON {k}: {why}");
    }
    if admitted.is_empty() {
        println!(
            "VOID {} — V1/V2: no admitted trajectory. frames scored 0.",
            class.label()
        );
        return;
    }

    // --- the species map for this class.
    let head = Trajectory::read(&admitted[0]).expect("the first admitted trajectory reads");
    let mut zs: Vec<u32> = head.header.z.clone();
    zs.sort_unstable();
    zs.dedup();
    let nz = zs.len();
    let mut cur = Curves {
        slot: vec![vec![0usize; nz]; nz],
        rmax: vec![vec![0.0f64; nz]; nz],
        u_edge: vec![vec![0.0f64; nz]; nz],
        zs: zs.clone(),
    };
    for a in 0..nz {
        for b in 0..nz {
            let slot = sim
                .bank
                .slot_of_z(zs[a], zs[b])
                .expect("the class loaded a curve for every species pair in its scenes");
            cur.slot[a][b] = slot;
            let t = sim.bank.table_slot(slot);
            cur.rmax[a][b] = t.r_max();
            cur.u_edge[a][b] = t.u(t.r_max());
            println!(
                "# curve slot z{}-z{} = {slot}, r_max = {:.4} bohr, u(r_max) = {:.6e} Ha",
                zs[a], zs[b], cur.rmax[a][b], cur.u_edge[a][b]
            );
        }
    }

    // --- plants (M-PLANT-OBS: re-derived here and pre-checked to fire).
    if want_plant {
        run_plant(&head, &cur, &sim.bank, freeze);
    }

    // --- the sweep.
    println!("# ladder (bohr): {LADDER:?}   c* = {} bohr   stride = {stride}", LADDER[C_STAR]);
    println!(
        "COLUMNS FRAME class seed frame E_band E_tail E_hard E_switch bound ratio_to_bound"
    );

    let mut total_frames = 0usize;
    let mut populated_frames = 0usize;
    let mut ladder_sum = [0.0f64; 6];
    let mut ladder_max = [0.0f64; 6];
    let mut zero_control_max = 0.0f64;
    let mut monotone_violations = 0usize;
    let mut r_max_seen_all = 0.0f64;
    let mut fence_tail = 0.0f64;
    let mut fence_pow6 = 0.0f64;
    let mut fence_pow3 = 0.0f64;
    // The fence's MAX as well as its mean. A mean fence beside a max verdict compares two
    // different statistics, and the verdict's own quantity is a max.
    let mut fence_tail_max = 0.0f64;
    let mut fence_pow6_max = 0.0f64;
    let mut fence_pow3_max = 0.0f64;
    // THE FREEZE GOT THIS BACKWARDS AND THE DATA SAID SO. §3 asserts "where they disagree,
    // `E_hard` is the larger and is the one gated, which is the conservative direction".
    // It is not: the C² switch starts removing energy at `c − W`, INSIDE the cutoff, so
    // `|E_switch(c)| ≥ |E_hard(c)|` and gating `E_hard` is the LESS conservative choice.
    // The gate stays on `E_hard` because that is what was staked; this tracks `E_switch`
    // so the results document can report the size of the freeze's error rather than
    // inherit its claim.
    let mut switch_max = 0.0f64;
    let mut switch_arg = (0u64, 0u64);
    let mut seed_rows: Vec<String> = Vec::new();
    let mut g1_fail = 0usize;
    // Split apart so branch (c) can be told from branch (b): a class that passes on the
    // entitled bound and fails on the incurred drift is a different finding from one that
    // fails both, and the freeze stakes them as different branches.
    let mut g1a_fail = 0usize;
    let mut g1b_fail = 0usize;
    let mut uninformative = 0usize;
    let mut worst_overall = (0.0f64, 0u64, 0u64); // (|E_hard(c*)|, seed, frame)

    for p in &admitted {
        let traj = Trajectory::read(p).expect("an admitted trajectory reads");
        let seed = traj.header.seed;
        let n = traj.header.n_atoms;
        let zidx: Vec<usize> = traj.header.z.iter().map(|z| cur.idx(*z)).collect();
        let periodic = false; // freeze §1.3: the census protocol is Boundary::Walls.
        let boxwhd = [traj.header.box_w, traj.header.box_h, traj.header.box_d];

        let Some(sb) = bounds.get(&seed).copied() else {
            println!(
                "# REFUSED  seed {seed:#018x}: the admitted arm log carries no drift bound for it"
            );
            refusals.push((format!("{seed:#018x}"), "no drift bound in arm log".into()));
            continue;
        };

        let mut smax = 0.0f64;
        let mut sarg = 0u64;
        let mut sframes = 0usize;
        // B1b's primary: the per-seed max of the estimator the engine would ACTUALLY apply.
        let mut swmax = 0.0f64;
        let mut swarg = 0u64;
        // Per-seed ladder maxima, so the radius at which THIS seed would cross its own
        // 0.10·B_s can be named. The freeze pre-commits this as branch (b)'s follow-up; it
        // is printed whatever the branch, labelled as beyond the staked verdict when the
        // class passes, because it is B2's design input either way.
        let mut sladder = [0.0f64; 6];
        for f in &traj.frames {
            assert_eq!(f.pos.len(), n, "frame atom count disagrees with the header");
            let row = score_frame(&f.pos, &zidx, &cur, &sim.bank, periodic, boxwhd);
            total_frames += 1;
            sframes += 1;
            if row.beyond_cstar > 0 {
                populated_frames += 1;
            }
            if row.r_max_seen > r_max_seen_all {
                r_max_seen_all = row.r_max_seen;
            }
            if row.hard[C_ZERO].abs() > zero_control_max {
                zero_control_max = row.hard[C_ZERO].abs();
            }
            for c in 0..LADDER.len() {
                let v = row.hard[c].abs();
                ladder_sum[c] += v;
                if v > ladder_max[c] {
                    ladder_max[c] = v;
                }
                if v > sladder[c] {
                    sladder[c] = v;
                }
                // G5 — |E_hard(c)| non-increasing along the ladder, per frame.
                if c > 0 && v > row.hard[c - 1].abs() {
                    monotone_violations += 1;
                }
            }
            fence_tail += row.tail[C_STAR];
            fence_pow6 += row.tail_pow6;
            fence_pow3 += row.tail_pow3;
            fence_tail_max = fence_tail_max.max(row.tail[C_STAR].abs());
            fence_pow6_max = fence_pow6_max.max(row.tail_pow6.abs());
            fence_pow3_max = fence_pow3_max.max(row.tail_pow3.abs());
            if row.switched[C_STAR].abs() > switch_max {
                switch_max = row.switched[C_STAR].abs();
                switch_arg = (seed, f.index);
            }
            let e = row.hard[C_STAR].abs();
            if e > smax {
                smax = e;
                sarg = f.index;
            }
            let sw = row.switched[C_STAR].abs();
            if sw > swmax {
                swmax = sw;
                swarg = f.index;
            }
            if f.index as usize % stride == 0 {
                println!(
                    "FRAME {} {seed:#018x} {} {:.6e} {:.6e} {:.6e} {:.6e} {:.6e} {:.6e}",
                    class.label(),
                    f.index,
                    row.band[C_STAR],
                    row.tail[C_STAR],
                    row.hard[C_STAR],
                    row.switched[C_STAR],
                    sb.bound,
                    e / sb.bound
                );
            }
        }
        let allow = NEGLIGIBLE_FRACTION * sb.bound;
        // B1: E_hard against 0.10·B_s. B1b: E_switch against 0.10·B_s AND 0.10·D_s, both
        // required. The 0.10 fraction is the same number in both freezes, deliberately.
        let allow_drift = NEGLIGIBLE_FRACTION * sb.peak;
        let (g1a, g1b) = match freeze {
            Freeze::B1 => (smax < allow, true),
            Freeze::B1b => (swmax < allow, swmax < allow_drift),
        };
        let pass = g1a && g1b;
        if !pass {
            g1_fail += 1;
        }
        if !g1a {
            g1a_fail += 1;
        }
        if !g1b {
            g1b_fail += 1;
        }
        if freeze == Freeze::B1b {
            println!(
                "SEEDB1B {} {seed:#018x} {sframes} maxEswitch {swmax:.6e} at {swarg} \
                 | G1a {swmax:.6e} vs {allow:.6e} {} | G1b {swmax:.6e} vs {allow_drift:.6e} {} \
                 | {}",
                class.label(),
                if g1a { "PASS" } else { "FAIL" },
                if g1b { "PASS" } else { "FAIL" },
                if pass { "PASS" } else { "FAIL" }
            );
        }
        let bd = sb.bound / sb.peak;
        if bd > UNINFORMATIVE_RATIO {
            uninformative += 1;
        }
        if smax > worst_overall.0 {
            worst_overall = (smax, seed, sarg);
        }
        let row = format!(
            "SEED {} {seed:#018x} {sframes} {smax:.6e} {sarg} {:.6e} {:.6e} {:.6e} {:.6e} {:.4} {}",
            class.label(),
            sb.bound,
            allow,
            smax / allow,
            sb.peak,
            bd,
            if pass { "PASS" } else { "FAIL" }
        );
        println!("{row}");
        seed_rows.push(row);
        for (c, &cut) in LADDER.iter().enumerate() {
            println!(
                "SEEDLADDER {} {seed:#018x} {cut:.1} {:.6e} {:.6e} {}",
                class.label(),
                sladder[c],
                allow,
                if sladder[c] < allow { "under" } else { "OVER" }
            );
        }
    }

    // ------------------------------------------------------------------ the gates
    println!("# ---- GATES, {} ----", class.label());
    for (c, &cut) in LADDER.iter().enumerate() {
        println!(
            "LADDER {} {cut:.1} {:.6e} {:.6e}",
            class.label(),
            ladder_sum[c] / total_frames.max(1) as f64,
            ladder_max[c]
        );
    }
    println!(
        "FENCE {} E_tail_mean {:.6e} E_tail_pow6_mean {:.6e} E_tail_pow3_mean {:.6e}",
        class.label(),
        fence_tail / total_frames.max(1) as f64,
        fence_pow6 / total_frames.max(1) as f64,
        fence_pow3 / total_frames.max(1) as f64
    );
    println!(
        "FENCEMAX {} E_tail_max {:.6e} E_tail_pow6_max {:.6e} E_tail_pow3_max {:.6e}",
        class.label(),
        fence_tail_max,
        fence_pow6_max,
        fence_pow3_max
    );
    println!("# max pair separation seen: {r_max_seen_all:.4} bohr (zero control sits at {:.1})", LADDER[C_ZERO]);
    println!(
        "SWITCHMAX {} max|E_switch(c*)| {switch_max:.6e} at seed {:#018x} frame {} \
         (ratio to max|E_hard(c*)| = {:.2})",
        class.label(),
        switch_arg.0,
        switch_arg.1,
        if worst_overall.0 > 0.0 { switch_max / worst_overall.0 } else { 0.0 }
    );

    let g4 = total_frames >= MIN_FRAMES;
    let g5 = monotone_violations == 0;
    let g6 = zero_control_max == 0.0;
    let pop_frac = populated_frames as f64 / total_frames.max(1) as f64;
    let g7 = pop_frac >= MIN_POPULATED_FRACTION;
    let g1 = g1_fail == 0;
    let g2 = g2_fail == 0;
    println!("GATE G1 {} {} (seeds failing: {g1_fail})", class.label(), if g1 { "PASS" } else { "FAIL" });
    println!("GATE G2 {} {} (curves failing: {g2_fail})", class.label(), if g2 { "PASS" } else { "FAIL" });
    println!("GATE G3 {} refusals {} of {} files hashed", class.label(), refusals.len(), paths.len() + 1);
    println!("GATE G4 {} {} (frames scored: {total_frames}, floor {MIN_FRAMES})", class.label(), if g4 { "PASS" } else { "VOID" });
    println!("GATE G5 {} {} (monotonicity violations: {monotone_violations})", class.label(), if g5 { "PASS" } else { "VOID" });
    println!("GATE G6 {} {} (max |E_hard(41.0)| = {zero_control_max:.3e}, EXACT 0.0 required)", class.label(), if g6 { "PASS" } else { "VOID" });
    println!("GATE G7 {} {} (frames with a nonempty beyond-c* population: {populated_frames}/{total_frames} = {:.4})", class.label(), if g7 { "PASS" } else { "VOID" }, pop_frac);

    // NAME THE PRICE THAT WAS ACTUALLY APPLIED. Printing B1's wall-clock floor beside a
    // B1b verdict produced a line reading "PASS (setup 973.3 s, floor 1200 s)" — false on
    // its face, since 973.3 is under 1200. The GATING was correct (B1b prices with W1+W2,
    // whose own lines are right); the RECEIPT was not, and a receipt that misstates its own
    // gate is worse than no receipt.
    println!(
        "GATE PRICE {} {} ({})",
        class.label(),
        if price_void { "VOID" } else { "PASS" },
        match freeze {
            Freeze::B1 => format!(
                "wall-clock: setup {setup_s:.1} s against floor {:.0} s",
                class.price_floor()
            ),
            Freeze::B1b => format!(
                "work units: W1 solver certificate + W2 in-run cost ratio; setup \
                 {setup_s:.1} s is REPORTED, not gated"
            ),
        }
    );
    if freeze == Freeze::B1b {
        println!(
            "GATE G1a {} {} (seeds failing the ENTITLED BOUND 0.10*B_s: {g1a_fail})",
            class.label(),
            if g1a_fail == 0 { "PASS" } else { "FAIL" }
        );
        println!(
            "GATE G1b {} {} (seeds failing the INCURRED DRIFT 0.10*D_s: {g1b_fail})",
            class.label(),
            if g1b_fail == 0 { "PASS" } else { "FAIL" }
        );
    }
    let verdict = if price_void {
        match freeze {
            Freeze::B1 => {
                "VOID — the freeze's price refusal fired: the curve setup finished under the floor"
            }
            Freeze::B1b => "VOID — the work-unit price refused (W1 or W2)",
        }
    } else if !(g2 && g4 && g5 && g6 && g7) {
        "VOID"
    } else if freeze == Freeze::B1b && g1a_fail == 0 && g1b_fail > 0 {
        // BRANCH (c) — the split. Small against what the integrator was ENTITLED to lose,
        // not against what it ACTUALLY lost. Staked as NON-NEGLIGIBLE because the
        // conjunction is what was staked, and never rounded to either side.
        "NON-NEGLIGIBLE — branch (c) SPLIT: passes the entitled bound 0.10*B_s, FAILS the \
         incurred drift 0.10*D_s. The discard is small against what the integrator was \
         entitled to lose and NOT against what it actually lost. The B2 Ewald requirement \
         FIRES for this class"
    } else if freeze == Freeze::B1b && g1b_fail == 0 && g1a_fail > 0 {
        "NON-NEGLIGIBLE — branch (c) SPLIT: passes the incurred drift 0.10*D_s, FAILS the \
         entitled bound 0.10*B_s. The B2 Ewald requirement FIRES for this class"
    } else if !g1 {
        "NON-NEGLIGIBLE — branch (b): FAILS BOTH denominators. The B2 Ewald requirement \
         FIRES for this class"
    } else if uninformative > 0 {
        "NEGLIGIBLE (G1a uninformative) — branch (e): passes both, but B_s/D_s > 1e3 so the \
         bound gate carried no information and the verdict rests on the incurred-drift gate"
    } else {
        "NEGLIGIBLE — branch (a)"
    };
    println!(
        "VERDICT {} {verdict} | worst frame: {} | LOWER BOUND: past the table's last knot the \
         curve is an exponential while the true tail is a power law, so the discard is at \
         least this and never at most this.",
        class.label(),
        match freeze {
            Freeze::B1 => format!(
                "seed {:#018x} frame {} at |E_hard(c*)| = {:.6e} Ha",
                worst_overall.1, worst_overall.2, worst_overall.0
            ),
            Freeze::B1b => format!(
                "seed {:#018x} frame {} at |E_switch(c*)| = {switch_max:.6e} Ha",
                switch_arg.0, switch_arg.1
            ),
        }
    );
}

/// P2 — the injected-pair plant, against G8 and the estimator's own bookkeeping.
///
/// The freeze's wording is "one atom is displaced so that its separation from a chosen
/// partner is exactly 16.0 bohr [...] `E_hard(c*)` must change by exactly `u_ab(16.0)` minus
/// that pair's prior contribution". EXECUTION NOTE, recorded rather than made silently:
/// displacing an atom moves EVERY pair it is in, not only the target pair, so the predicted
/// change is the target pair's term PLUS the change in the displaced atom's other pairs.
/// Both components are computed by a direct per-pair path and compared against the full
/// estimator run twice — which is what makes this a check of the estimator's inclusion
/// bookkeeping rather than an identity.
fn run_plant(
    traj: &Trajectory,
    cur: &Curves,
    bank: &holon_render::bank::PairBank,
    freeze: Freeze,
) {
    let f = &traj.frames[0];
    let n = traj.header.n_atoms;
    let zidx: Vec<usize> = traj.header.z.iter().map(|z| cur.idx(*z)).collect();
    let boxwhd = [traj.header.box_w, traj.header.box_h, traj.header.box_d];

    let (a, b) = (zidx[0], zidx[1]);
    // The carrier is the estimator's OWN term at the plant separation, so it moves with the
    // estimator: B1's is the bare table value, B1b's is what the switch actually removes
    // there. A plant re-derived for the instrument is the whole of M-PLANT-OBS.
    let u_plant = bank.table_slot(cur.slot[a][b]).u(PLANT_R);
    let carrier = match freeze {
        Freeze::B1 => u_plant,
        Freeze::B1b => {
            let r_in = LADDER[C_STAR] - PAIR_SWITCH_WIDTH;
            let (sw, _, _) = switch_c2(PLANT_R, r_in, LADDER[C_STAR]);
            (1.0 - sw) * u_plant
        }
    };
    println!(
        "# PLANT P2 carrier ({}): {} at {PLANT_R} bohr = {carrier:.6e} Ha",
        freeze.label(),
        match freeze {
            Freeze::B1 => "u_ab",
            Freeze::B1b => "(1-S2)*u_ab",
        }
    );
    assert!(
        carrier != 0.0,
        "the plant's carrier is zero in the sector the plant acts on (the beyond-cutoff \
         pair sector); a plant that could not have fired is not run"
    );

    let base = score_frame(&f.pos, &zidx, cur, bank, false, boxwhd);

    // Displace atom 1 along the line from atom 0, to exactly PLANT_R bohr.
    let mut pos = f.pos.clone();
    let d = [
        pos[1][0] - pos[0][0],
        pos[1][1] - pos[0][1],
        pos[1][2] - pos[0][2],
    ];
    let r0 = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    for k in 0..3 {
        pos[1][k] = pos[0][k] + d[k] / r0 * PLANT_R;
    }

    let planted = score_frame(&pos, &zidx, cur, bank, false, boxwhd);
    let measured = match freeze {
        Freeze::B1 => planted.hard[C_STAR] - base.hard[C_STAR],
        Freeze::B1b => planted.switched[C_STAR] - base.switched[C_STAR],
    };

    // The independent prediction: every pair the displaced atom is in, evaluated directly,
    // through the SAME inclusion rule the gated estimator uses.
    let contrib = |p: &[[f64; 3]], i: usize, j: usize| -> f64 {
        let dd = [p[j][0] - p[i][0], p[j][1] - p[i][1], p[j][2] - p[i][2]];
        let r = (dd[0] * dd[0] + dd[1] * dd[1] + dd[2] * dd[2]).sqrt();
        let t = bank.table_slot(cur.slot[zidx[i]][zidx[j]]);
        match freeze {
            Freeze::B1 => {
                if r > LADDER[C_STAR] {
                    t.u(r)
                } else {
                    0.0
                }
            }
            Freeze::B1b => {
                let r_in = LADDER[C_STAR] - PAIR_SWITCH_WIDTH;
                if r > r_in {
                    let (sw, _, _) = switch_c2(r, r_in, LADDER[C_STAR]);
                    (1.0 - sw) * t.u(r)
                } else {
                    0.0
                }
            }
        }
    };
    let mut d_target = 0.0f64;
    let mut d_others = 0.0f64;
    for k in 0..n {
        if k == 1 {
            continue;
        }
        let (i, j) = if k < 1 { (k, 1) } else { (1, k) };
        let delta = contrib(&pos, i, j) - contrib(&f.pos, i, j);
        if k == 0 {
            d_target = delta;
        } else {
            d_others += delta;
        }
    }
    let predicted = d_target + d_others;
    let rel = if predicted != 0.0 {
        ((measured - predicted) / predicted).abs()
    } else {
        (measured - predicted).abs()
    };
    println!(
        "# PLANT P2: r_01 {r0:.4} -> {PLANT_R:.4} bohr | target-pair change {d_target:.9e} \
         | other pairs of the displaced atom {d_others:.9e} | predicted {predicted:.9e} \
         | measured {measured:.9e} | relative {rel:.3e}"
    );
    let tol = match freeze {
        Freeze::B1 => 1e-12,
        Freeze::B1b => B1B_PLANT_TOL,
    };
    println!(
        "GATE G8 PLANT-P2 {} (relative {rel:.3e} against {tol:.0e})",
        if rel <= tol {
            "PASS — the plant fired and the estimator's bookkeeping matches"
        } else {
            "FAIL"
        }
    );
}
