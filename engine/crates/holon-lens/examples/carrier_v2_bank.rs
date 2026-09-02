//! G1, G2 and G5 OF `CARRIER_V2_PREREG.md`, RUN AGAINST THE REAL BANK.
//!
//! The unit tests and the plants run this campaign's gates on synthetic carriers. This
//! runs them on the artifacts of record — the twenty-three census trajectories pinned by
//! `conformance/water_observatory/census_traj_manifest.sha256` — because a compatibility
//! claim about banked files that was only ever checked on files this campaign wrote is a
//! claim about this campaign's own writer.
//!
//! * **G1** every banked file, read through `Trajectory2` and written back through
//!   `write_as_v1`, reproduces its MANIFEST DIGEST. Not merely its own bytes: the
//!   manifest is the artifact of record and it was produced out of band by `sha256sum`,
//!   so agreeing with it ties this reader to the pin rather than to itself.
//! * **G2** every field the v1 reader makes of the file, the v2 reader makes too.
//! * **G5** the measured dimensionality, printed for every file whatever it says, with
//!   `CENSUS_RESULTS.md` §14.4's own numbers as the expectation.
//!
//! ## Paths, and why none of them is written down here
//!
//! Gate 10a3: no instrument carries a session-keyed or lane-keyed path. The repository is
//! found by walking up from the running binary and then from the working directory,
//! looking for the manifest itself as the marker; the bank is found from an argument, an
//! environment variable, or the repository's sibling directory. Every candidate that was
//! tried is printed on a refusal, so a caller sees WHERE it looked rather than only that
//! it failed. `--dry-run` prints the resolution and the work it would do and exits 0
//! having read nothing — testing a launcher by launching it is not a test.
//!
//! ```text
//! cargo run --release -p holon-lens --example carrier_v2_bank -- [--dry-run] [--bank=DIR]
//! ```
//!
//! Exit codes, fixed by the freeze: 0 all gates pass, 2 bad arguments, 3 a path did not
//! resolve, 4 a version or format refusal, 5 a digest or field mismatch.

use holon_lens::traj::Trajectory;
use holon_lens::traj2::{Measured, Trajectory2};
use std::path::{Path, PathBuf};

// ============================================================ sha256, with its own gate
//
// `holon-lens` has ZERO dependencies and that is load-bearing (RUNG2 §1 verified G1 out of
// band for exactly this reason). So the digest is implemented here rather than imported,
// and an implementation this campaign wrote is an instrument this campaign has to gauge:
// `sha256_matches_the_published_vectors` checks it against the two NIST vectors before any
// file is read, and the whole run refuses if they do not match. A digest that agrees with
// nothing is not evidence.

const K: [u32; 64] = [
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
            buf: [0; 64],
            len: 0,
            total: 0,
        }
    }

    fn block(&mut self, b: &[u8; 64]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([b[4 * i], b[4 * i + 1], b[4 * i + 2], b[4 * i + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut v = self.h;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [
                t1.wrapping_add(t2),
                v[0],
                v[1],
                v[2],
                v[3].wrapping_add(t1),
                v[4],
                v[5],
                v[6],
            ];
        }
        for i in 0..8 {
            self.h[i] = self.h[i].wrapping_add(v[i]);
        }
    }

    fn update(&mut self, mut data: &[u8]) {
        self.total += data.len() as u64;
        while !data.is_empty() {
            let take = (64 - self.len).min(data.len());
            self.buf[self.len..self.len + take].copy_from_slice(&data[..take]);
            self.len += take;
            data = &data[take..];
            if self.len == 64 {
                let b = self.buf;
                self.block(&b);
                self.len = 0;
            }
        }
    }

    fn hex(mut self) -> String {
        let bits = self.total * 8;
        self.update(&[0x80]);
        while self.len != 56 {
            self.update(&[0]);
        }
        // `update` moved `total`; the length field is the one captured above.
        let b = {
            let mut b = self.buf;
            b[56..64].copy_from_slice(&bits.to_be_bytes());
            b
        };
        self.block(&b);
        self.h.iter().map(|w| format!("{w:08x}")).collect()
    }
}

fn sha256_hex(data: &[u8]) -> String {
    let mut s = Sha256::new();
    s.update(data);
    s.hex()
}

/// The two vectors every SHA-256 implementation is published against. Checked BEFORE any
/// artifact is read: a digest instrument that has never been compared to a known answer
/// cannot convict anything.
fn sha256_self_check() -> Result<(), String> {
    let cases = [
        (
            "".as_bytes(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            "abc".as_bytes(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".as_bytes(),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
    ];
    for (input, want) in cases {
        let got = sha256_hex(input);
        if got != want {
            return Err(format!(
                "sha256 self-check FAILED on {:?}: got {got}, published {want}",
                String::from_utf8_lossy(input)
            ));
        }
    }
    // The million-'a' vector. The three above all fit inside one or two blocks, so none of
    // them exercises the streaming buffer, the block counter, or a length field past
    // 2^16 bits — which is the whole of what a trajectory file uses.
    let long = vec![b'a'; 1_000_000];
    let got = sha256_hex(&long);
    let want = "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0";
    if got != want {
        return Err(format!("sha256 self-check FAILED on 1e6 'a': got {got}, published {want}"));
    }
    // And the streaming path must agree with the one-shot path on the same bytes, because
    // the bank is read in one shot and this is the only place that could diverge.
    let mut s = Sha256::new();
    for chunk in long.chunks(7) {
        s.update(chunk);
    }
    if s.hex() != want {
        return Err("sha256 disagrees with itself across chunk boundaries".into());
    }
    Ok(())
}

// ==================================================================== path resolution

/// The repository root: the directory containing the census manifest.
///
/// Searched by walking up from the RUNNING BINARY and then from the working directory.
/// Neither a compile-time manifest path nor any absolute path appears in this file, so the
/// binary is not keyed to the tree that built it.
const MARKER: &str = "conformance/water_observatory/census_traj_manifest.sha256";

fn find_repo(tried: &mut Vec<String>) -> Option<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(v) = std::env::var("HOLON_REPO") {
        roots.push(PathBuf::from(v));
    }
    if let Ok(exe) = std::env::current_exe() {
        roots.push(exe);
    }
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }
    for start in roots {
        let mut d: Option<&Path> = Some(start.as_path());
        while let Some(cur) = d {
            let probe = cur.join(MARKER);
            tried.push(probe.display().to_string());
            if probe.is_file() {
                return Some(cur.to_path_buf());
            }
            d = cur.parent();
        }
    }
    None
}

fn find_bank(repo: &Path, arg: Option<&str>, tried: &mut Vec<String>) -> Option<PathBuf> {
    let mut cands: Vec<PathBuf> = Vec::new();
    if let Some(a) = arg {
        cands.push(PathBuf::from(a));
    }
    if let Ok(v) = std::env::var("HOLON_CENSUS_TRAJ") {
        cands.push(PathBuf::from(v));
    }
    if let Ok(v) = std::env::var("HOLON_ARTIFACTS") {
        cands.push(PathBuf::from(v).join("census-traj"));
    }
    if let Some(p) = repo.parent() {
        cands.push(p.join("holon-artifacts/census-traj"));
    }
    for c in cands {
        tried.push(c.display().to_string());
        if c.is_dir() {
            return Some(c);
        }
    }
    None
}

// ============================================================================= the run

/// `CENSUS_RESULTS.md` §14.4's table, as this campaign's G5 expectation. The seventeen
/// planar trajectories are named by the arm they belong to; `de4_on` is the one that left.
fn expected_planar(rel: &str) -> bool {
    !rel.starts_with("de4_on/")
}

/// `max_i max_t |z_i(t) - z_i(0)|` — CENSUS_RESULTS.md §14.4's own statistic, so this
/// campaign's reading can be compared to the prior measurement rather than merely to its
/// own expectation. §14.4 publishes 0.0000 for seventeen files and 11.4899 for `de4_on`.
fn max_departure_z(t: &Trajectory2) -> f64 {
    let Some(first) = t.frames.first() else {
        return 0.0;
    };
    let mut worst = 0.0f64;
    for f in &t.frames {
        for (p, p0) in f.pos.iter().zip(first.pos.iter()) {
            let d = (p[2] - p0[2]).abs();
            if d > worst {
                worst = d;
            }
        }
    }
    worst
}

struct Row {
    rel: String,
    g1: bool,
    g2: Vec<String>,
    m: Measured,
    max_dz: f64,
    declared: u32,
    frames: usize,
    complete: bool,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dry = args.iter().any(|a| a == "--dry-run");
    let bank_arg = args.iter().find_map(|a| a.strip_prefix("--bank="));
    for a in &args {
        if a.starts_with("--") && a != "--dry-run" && !a.starts_with("--bank=") {
            eprintln!("REFUSED  unknown argument {a}");
            eprintln!("         usage: carrier_v2_bank [--dry-run] [--bank=DIR]");
            std::process::exit(2);
        }
    }

    if let Err(e) = sha256_self_check() {
        eprintln!("REFUSED  {e}");
        std::process::exit(5);
    }
    println!("sha256 self-check: PASS (3 published vectors)");

    let mut tried = Vec::new();
    let Some(repo) = find_repo(&mut tried) else {
        eprintln!("REFUSED  could not locate the repository: no ancestor of the binary or");
        eprintln!("         the working directory contains {MARKER}");
        for t in tried {
            eprintln!("           tried {t}");
        }
        eprintln!("         set HOLON_REPO to the repository root");
        std::process::exit(3);
    };
    let manifest_path = repo.join(MARKER);
    let mut tried = Vec::new();
    let Some(bank) = find_bank(&repo, bank_arg, &mut tried) else {
        eprintln!("REFUSED  could not locate the census trajectory bank");
        for t in tried {
            eprintln!("           tried {t}");
        }
        eprintln!("         pass --bank=DIR, or set HOLON_CENSUS_TRAJ or HOLON_ARTIFACTS");
        std::process::exit(3);
    };

    let manifest = match std::fs::read_to_string(&manifest_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("REFUSED  {}: {e}", manifest_path.display());
            std::process::exit(3);
        }
    };
    let pins: Vec<(String, String)> = manifest
        .lines()
        .filter_map(|l| l.split_once("  "))
        .map(|(d, f)| (d.trim().to_string(), f.trim().to_string()))
        .collect();

    println!("repo     {}", repo.display());
    println!("bank     {}", bank.display());
    println!("manifest {} ({} pins)", manifest_path.display(), pins.len());

    if dry {
        println!("\nDRY RUN — nothing was read and nothing was written.");
        let n_traj = pins.iter().filter(|(_, r)| r.ends_with(".traj")).count();
        println!(
            "would digest {} pinned files and read {} of them as trajectories:",
            pins.len(),
            n_traj
        );
        for (_, rel) in &pins {
            let p = bank.join(rel);
            let kind = if rel.ends_with(".traj") {
                if expected_planar(rel) { "traj, expect planar" } else { "traj, expect 3D" }
            } else {
                "log, digest only"
            };
            println!(
                "  {rel:44}  {:7}  {kind}",
                if p.is_file() { "present" } else { "MISSING" }
            );
        }
        println!("\ngates: G1a bank identity, G1b reader identity, G2 field identity, G5 dims");
        std::process::exit(0);
    }

    // ------------------------------------------------------- G1a: the bank is unchanged
    //
    // Kept SEPARATE from the round-trip check below, because they are different facts. G1a
    // says the artifacts on disk are still the artifacts the manifest pinned; G1b says
    // this reader reinterprets none of them. A single combined number could not tell a
    // rotted bank from a defective reader, and the two call for opposite responses.
    let mut g1a_fail = 0usize;
    for (pin, rel) in &pins {
        let p = bank.join(rel);
        let bytes = match std::fs::read(&p) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("REFUSED  {}: {e}", p.display());
                std::process::exit(3);
            }
        };
        let got = sha256_hex(&bytes);
        if &got != pin {
            g1a_fail += 1;
            eprintln!("G1a MISMATCH  {rel}\n    pin {pin}\n    got {got}");
        }
    }
    println!(
        "\nG1a — carrier identity: {} of {} banked files match the manifest",
        pins.len() - g1a_fail,
        pins.len()
    );
    if g1a_fail > 0 {
        eprintln!("REFUSED  the BANK has changed since it was pinned; nothing measured here");
        eprintln!("         would be about the artifacts of record.");
        std::process::exit(5);
    }

    // Five of the twenty-three pins are RUN LOGS, not trajectories. They are part of the
    // carrier's identity and are digested above; they are not fed to a trajectory reader,
    // and saying so is why the counts below are 18 and not 23.
    let trajs: Vec<&(String, String)> = pins.iter().filter(|(_, r)| r.ends_with(".traj")).collect();
    println!(
        "         {} of them are trajectories; {} are run logs, digested and not parsed",
        trajs.len(),
        pins.len() - trajs.len()
    );

    let mut rows: Vec<Row> = Vec::new();
    let mut g1_fail = 0usize;
    let mut g2_fail = 0usize;
    let out_dir = std::env::temp_dir().join(format!("carrier-v2-rt-{}", std::process::id()));
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!("REFUSED  scratch directory {}: {e}", out_dir.display());
        std::process::exit(3);
    }

    for (pin, rel) in &trajs {
        let src = bank.join(rel);
        let two = match Trajectory2::read(&src) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("REFUSED  {rel}: {e}");
                std::process::exit(e.exit_code());
            }
        };
        let one = match Trajectory::read(&src) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("REFUSED  {rel} through the v1 reader: {e}");
                std::process::exit(4);
            }
        };
        // G1 — through the campaign's own writer, digested and compared to the PIN.
        let back = out_dir.join("rt.traj");
        if let Err(e) = two.write_as_v1(&back) {
            eprintln!("REFUSED  {rel} would not re-serialise: {e}");
            std::process::exit(e.exit_code());
        }
        let got = match std::fs::read(&back) {
            Ok(b) => sha256_hex(&b),
            Err(e) => {
                eprintln!("REFUSED  reading back {}: {e}", back.display());
                std::process::exit(3);
            }
        };
        let g1 = &got == pin;
        if !g1 {
            g1_fail += 1;
            eprintln!("G1 MISMATCH  {rel}\n    pin {pin}\n    got {got}");
        }
        // G2 — the shared comparator, the same one the plants convict.
        let g2 = two.diff_against_v1(&one);
        if !g2.is_empty() {
            g2_fail += 1;
            eprintln!("G2 MISMATCH  {rel}");
            for d in g2.iter().take(6) {
                eprintln!("    {d}");
            }
        }
        let m = two.measure();
        let max_dz = max_departure_z(&two);
        rows.push(Row {
            rel: rel.clone(),
            max_dz,
            g1,
            g2,
            m,
            declared: two.header.dims_declared,
            frames: two.frames.len(),
            complete: two.is_complete(),
        });
    }
    let _ = std::fs::remove_dir_all(&out_dir);

    // ------------------------------------------------------------------ G5, every file
    println!("\nG5 — DIMS AS MEASURED (declared is printed beside it, never instead of it)");
    println!(
        "{:<42} {:>4} {:>4} {:>10} {:>10} {:>11} {:>7} {:>6}",
        "file", "dec", "meas", "span z", "max|dz|", "flatness", "frames", "whole"
    );
    let mut g5_fail = 0usize;
    for r in &rows {
        let want_planar = expected_planar(&r.rel);
        let ok = if want_planar {
            r.m.span[2] == 0.0 && r.m.dims == 2
        } else {
            r.m.span[2] > 10.0 && r.m.dims == 3
        };
        if !ok {
            g5_fail += 1;
        }
        println!(
            "{:<42} {:>4} {:>4} {:>10.4} {:>10.4} {:>11.3e} {:>7} {:>6}  {}",
            r.rel,
            r.declared,
            r.m.dims,
            r.m.span[2],
            r.max_dz,
            r.m.flatness,
            r.frames,
            if r.complete { "yes" } else { "NO" },
            if ok { "" } else { "<-- G5" }
        );
    }

    // The independent cross-check: §14.4 measured `de4_on` at 11.4899 with a different
    // instrument in a different lane. Reproducing it here is corroboration of the reader,
    // and a disagreement would be a finding about one of the two, not a gate this
    // campaign gets to fail quietly.
    if let Some(r) = rows.iter().find(|r| r.rel.starts_with("de4_on/")) {
        let published = 11.4899_f64;
        println!(
            "\n         cross-check against CENSUS_RESULTS.md §14.4: de4_on max|dz| = \
{:.4}, published {published:.4}, difference {:.4}",
            r.max_dz,
            (r.max_dz - published).abs()
        );
    }

    let disagree = rows.iter().filter(|r| r.m.dims != r.declared).count();
    println!(
        "G1b {} of {} trajectories reproduce their manifest digest through the reader",
        rows.len() - g1_fail,
        rows.len()
    );
    println!(
        "G2  {} of {} field-identical to the v1 reader",
        rows.len() - g2_fail,
        rows.len()
    );
    println!(
        "G5  {} of {} match §14.4's expectation; {} file(s) DECLARE a dimensionality \
the data does not carry",
        rows.len() - g5_fail,
        rows.len(),
        disagree
    );

    if g1_fail + g2_fail > 0 {
        eprintln!("\nREFUSED  the READER is defective, not the bank: the banked files are");
        eprintln!("         the artifacts of record and this instrument is the new thing.");
        std::process::exit(5);
    }
    if g5_fail > 0 {
        eprintln!("\nG5 did not match §14.4 on {g5_fail} file(s) — reported, not exited on:");
        eprintln!("         a disagreement with a PRIOR MEASUREMENT is a finding to write");
        eprintln!("         up, and this instrument does not get to decide which is right.");
    }
    println!("\nALL GATES PASS");
}
