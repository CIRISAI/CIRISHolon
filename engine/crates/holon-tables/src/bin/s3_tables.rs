//! **SATURATION-3: the cheap-class tables, generated through the resource layer.**
//!
//! One table per invocation. Every parameter is supplied on the command line and ECHOED back
//! before any work starts — that echo is half of the provenance discipline (M-PROVENANCE-
//! OVERREACH: the launch header pins the bytes, the binary's own echo says what those bytes were
//! asked to do, and the two catch each other's gaps).
//!
//! # There are no default domains, and that is deliberate
//!
//! A domain is a PHYSICS claim: it comes from each species pair's own curve, A1-style
//! second-smallest-side logic with the pair's measured tail. This lane owns the mesh, not the
//! chemistry, so inventing a default here would be a physics assertion smuggled in as a
//! convenience — and a plausible-looking wrong domain is exactly the shape that produced the
//! `homonuclear_radius` near-miss (a fallback returning hydrogen's radius for chlorine). The
//! binary REFUSES rather than defaulting.
//!
//! # Usage
//!
//! ```text
//! s3_tables --species H,H,Cl \
//!           --x 2.0:6.0 --y 2.2:6.4 --u -1.0:0.6 \
//!           --grid 33x33x13 --region 3x3x13 \
//!           --warm cold|chain --workers 32 \
//!           --out engine/output/saturation3/hhcl.tbl
//! ```

use std::io::Write;
use std::time::Instant;

use holon_chem::elements::{by_symbol, Species};
use holon_resource::{Arena, ResourceKind};
use holon_tables::generate::generate_leased;
use holon_tables::{GenSpec, TableGrid, WarmPolicy, WorkerProbe};

fn fail(msg: &str) -> ! {
    eprintln!("REFUSED: {msg}");
    std::process::exit(2);
}

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn required(args: &[String], name: &str, what: &str) -> String {
    arg(args, name).unwrap_or_else(|| {
        fail(&format!(
            "{name} is required and has no default. {what} There is no sensible fallback: a \
             wrong-but-plausible value here would be a physics claim this binary is not \
             entitled to make."
        ))
    })
}

/// `lo:hi`
fn span(s: &str, name: &str) -> (f64, f64) {
    let (a, b) = s
        .split_once(':')
        .unwrap_or_else(|| fail(&format!("{name} must be lo:hi, got '{s}'")));
    let (lo, hi) = (
        a.parse::<f64>().unwrap_or_else(|_| fail(&format!("bad {name} low bound '{a}'"))),
        b.parse::<f64>().unwrap_or_else(|_| fail(&format!("bad {name} high bound '{b}'"))),
    );
    if !(hi > lo) {
        fail(&format!("{name} is not increasing: {lo}:{hi}"));
    }
    (lo, hi)
}

/// `axbxc`
fn triple(s: &str, name: &str) -> [usize; 3] {
    let p: Vec<&str> = s.split('x').collect();
    if p.len() != 3 {
        fail(&format!("{name} must be AxBxC, got '{s}'"));
    }
    let mut out = [0usize; 3];
    for (i, v) in p.iter().enumerate() {
        out[i] = v
            .parse()
            .unwrap_or_else(|_| fail(&format!("bad {name} component '{v}'")));
        if out[i] == 0 {
            fail(&format!("{name} has a zero component"));
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        eprintln!("{}", include_str!("s3_tables_usage.txt"));
        std::process::exit(0);
    }

    // ---- parameters, all required, all echoed.
    let species_s = required(&args, "--species", "Three symbols, comma separated.");
    let species: Vec<Species> = species_s
        .split(',')
        .map(|s| by_symbol(s).unwrap_or_else(|| fail(&format!("unknown species '{s}'"))))
        .collect();
    if species.len() != 3 {
        fail("--species must name exactly three");
    }

    let (x_lo, x_hi) = span(
        &required(&args, "--x", "The SHORT side's domain, from the pair's own curve."),
        "--x",
    );
    let (y_lo, y_hi) = span(
        &required(&args, "--y", "The MIDDLE side's domain (A1 bounds this one, not s3)."),
        "--y",
    );
    let (u_lo, u_hi) = span(
        &required(&args, "--u", "The cosine domain between the two short sides."),
        "--u",
    );
    let grid = triple(&required(&args, "--grid", "Sized by this table's own T1 kill."), "--grid");
    let region = triple(
        &required(
            &args,
            "--region",
            "Part of the TABLE's identity (it fixes the warm chains, hence the trailing bits).",
        ),
        "--region",
    );
    let warm = match required(&args, "--warm", "cold | chain.").as_str() {
        "cold" => WarmPolicy::AllCold,
        "chain" => WarmPolicy::CanonicalChain,
        other => fail(&format!("--warm must be cold|chain, got '{other}'")),
    };
    let workers: usize = required(&args, "--workers", "Worker count.")
        .parse()
        .unwrap_or_else(|_| fail("--workers must be a positive integer"));
    if workers == 0 {
        fail("--workers must be at least 1");
    }
    let out = required(&args, "--out", "Where the table is written.");

    let tg = TableGrid::new(
        grid[0],
        grid[1],
        grid[2],
        region,
        (x_lo, x_hi),
        (y_lo, y_hi),
        (u_lo, u_hi),
    );
    let spec = GenSpec::new([species[0], species[1], species[2]], tg).with_warm(warm);

    // ---- THE ECHO. Every parameter, as the binary actually parsed it — not as the launcher
    // believes it passed them. This is the half of the provenance discipline that closed
    // M-PROVENANCE-OVERREACH's gap: the header pins the bytes, the echo says what they were
    // asked to do.
    println!("=== s3_tables: parameters as parsed ===");
    println!("species       {species_s}");
    println!("x (short)     {x_lo} .. {x_hi}");
    println!("y (middle)    {y_lo} .. {y_hi}");
    println!("u (cosine)    {u_lo} .. {u_hi}");
    println!("grid          {}x{}x{}", grid[0], grid[1], grid[2]);
    println!("region        {}x{}x{}", region[0], region[1], region[2]);
    println!("nodes         {}", tg.n_nodes());
    println!("regions       {}  (cold seeds; cold fraction {:.4})",
        tg.n_regions(),
        tg.n_regions() as f64 / tg.n_nodes() as f64);
    println!("warm          {warm:?}");
    println!("workers       {workers}");
    println!("out           {out}");
    println!("device class  Cpu (DECLARED — this table is bit-gated, so D0 pins it here)");
    println!("loadavg       {:.2}  (at launch; the machine runs other campaigns)", loadavg());

    // A region shape that cuts into fewer regions than there are workers is not an error, but it
    // means the extra workers idle, and saying so beats discovering it in a wall-clock figure.
    if tg.n_regions() < workers {
        println!(
            "NOTE          {} regions against {workers} workers: {} worker(s) will idle. The \
             region shape is part of the table's identity, so it is NOT adjusted to fit the \
             machine.",
            tg.n_regions(),
            workers - tg.n_regions()
        );
    }
    println!();

    // ---- the run, through the resource layer.
    let mut arena = Arena::new();
    let mut probe = WorkerProbe::new();
    println!("available_parallelism reports {} (REPORTED, not the admission test — every \
              worker is admitted by a probe that spawns, runs and joins a thread)",
        probe.reported_parallelism);

    let t0 = Instant::now();
    let run = match generate_leased(&spec, workers, &mut arena, &mut probe) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("REFUSED at the resource layer: {}", e.message());
            std::process::exit(3);
        }
    };
    let secs = t0.elapsed().as_secs_f64();

    // ---- the verdict.
    let o = &run.outcome;
    let voided = o.voided;
    println!();
    println!("=== result ===");
    println!("nodes         {}", o.records.len());
    println!("wall          {secs:.1} s  ({:.3} s/node)", secs / o.records.len() as f64);
    println!("cold / warm   {} / {}", o.cold_solves, o.warm_solves);
    println!("davidson      {} iterations total", o.total_davidson_iters);
    println!("VOIDed        {voided}");
    println!("digest        {}", o.digest().hex());
    println!("certificate   {}", if o.certificate.is_clean() { "CLEAN" } else { "CONVICTED" });
    println!("ledger        opened {} released {} convicted {} rent {}",
        arena.ledger().opened, arena.ledger().released, arena.ledger().convicted,
        arena.ledger().rent.0);
    println!("books balance {}", arena.balances());
    println!("per-worker    {:?}", run.progress());

    // M-BUDGET-LAUNDER: a VOIDed node is reported at full prominence, never averaged away.
    if voided > 0 {
        println!();
        println!("!! {voided} node(s) VOIDED. Each is listed; none is scored.");
        for r in o.records.iter().filter(|r| !r.is_ok()) {
            println!("   node {:6}  {:?}", r.node, r.status);
        }
    }

    // Refuse to write a table whose certificate is dirty or whose books leaked.
    if !o.certificate.is_clean() {
        eprintln!("REFUSED: the merge digest convicted this run; the table is not written.");
        std::process::exit(4);
    }
    if !arena.balances() {
        eprintln!("REFUSED: the lease books did not balance; the table is not written.");
        std::process::exit(5);
    }

    let mut f = std::fs::File::create(&out)
        .unwrap_or_else(|e| fail(&format!("cannot create {out}: {e}")));
    writeln!(f, "# SATURATION-3 table: {species_s}").unwrap();
    writeln!(f, "# grid {}x{}x{} region {}x{}x{} warm {warm:?}",
        grid[0], grid[1], grid[2], region[0], region[1], region[2]).unwrap();
    writeln!(f, "# x {x_lo}:{x_hi} y {y_lo}:{y_hi} u {u_lo}:{u_hi}").unwrap();
    writeln!(f, "# digest {}", o.digest().hex()).unwrap();
    writeln!(f, "# nodes {} voided {voided}", o.records.len()).unwrap();
    writeln!(f, "# node energy d1 d2 iters exit status").unwrap();
    for r in &o.records {
        writeln!(f, "{} {:.17e} {:.17e} {:.17e} {} {} {}",
            r.node, r.energy(), f64::from_bits(r.d1_bits), f64::from_bits(r.d2_bits),
            r.davidson_iters, r.exit_code, r.status_code()).unwrap();
    }
    println!();
    println!("wrote {out}");
}

fn loadavg() -> f64 {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok())
        .unwrap_or(f64::NAN)
}
