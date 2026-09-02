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
//!           --warm cold|chain --workers 32 [--device cpu] \
//!           --out engine/output/saturation3/hhcl.tbl
//! ```

use std::io::Write;
use std::time::Instant;

use holon_chem::elements::{by_symbol, Species};
use holon_chem::sigma_op::DeviceClass;
use holon_tables::Surface;
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

    // THE SEAM RECORD. Required, and one of the two forms, because a shipped surface with a
    // HIDDEN CORNER is exactly what the provenance gate exists to see. A state crossing is a
    // slope discontinuity: a cubic interpolant across one has an error floor set by the jump,
    // so uniform refinement cannot beat it and paying for resolution past a seam buys nothing.
    // Every trimer type this campaign tabulates has a reactive channel.
    let seam_loci = arg(&args, "--seam-loci");
    let seam_floor = arg(&args, "--seam-accepted-floor");
    let seam_instrument = arg(&args, "--seam-instrument");
    if seam_loci.is_none() && seam_floor.is_none() {
        fail(
            "a seam record is required: pass --seam-loci 'axis=value,…' for located state \
             crossings, or --seam-accepted-floor '<hartree>:<why>' to declare a floor taken \
             knowingly. Uniform refinement cannot beat a state crossing, so a table with \
             neither has not checked the smoothness it is about to claim.",
        );
    }
    if seam_loci.is_some() && seam_floor.is_some() {
        fail("--seam-loci and --seam-accepted-floor are alternatives; pass exactly one");
    }
    if seam_loci.is_some() && seam_instrument.is_none() {
        fail("--seam-loci needs --seam-instrument: loci from an unnamed source are not a scan");
    }

    let tg = TableGrid::new(
        grid[0],
        grid[1],
        grid[2],
        region,
        (x_lo, x_hi),
        (y_lo, y_hi),
        (u_lo, u_hi),
    );
    // **--device: THE THIRD IDENTITY AXIS ON THE COMMAND LINE** (RESOURCE_DESIGN D0).
    //
    // Beside the solver budget and the subtraction basis, and for the same reason both of
    // those are named rather than defaulted: it is a property of the artifact, and an
    // artifact whose regime is unrecorded cannot be re-derived from. Defaults to `cpu`,
    // which is what every committed table declares.
    //
    // `gpu` is REFUSED HERE, by name, with its exit — not silently downgraded. This binary
    // links `holon-tables`, which does not link CUDA (deliberately: it sits inside the
    // workspace whose isolation gates keep CUDA out). A GPU-class table is generated through
    // `holon-gpu`'s device-class launcher, which can supply the provider. D4: a failed path
    // produces a loud refusal naming what was asked, what was found, and what to do instead —
    // never a quiet run on the other class.
    let device_s = args
        .iter()
        .position(|a| a == "--device")
        .and_then(|i| args.get(i + 1).cloned())
        .unwrap_or_else(|| "cpu".to_string());
    let device = match DeviceClass::from_tag(&device_s) {
        Some(DeviceClass::Cpu) => DeviceClass::Cpu,
        Some(DeviceClass::Gpu) => fail(
            "--device gpu: this binary cannot generate a GPU-class table. It links \
             holon-tables, which does not link CUDA by design. Use holon-gpu's device-class \
             launcher, which supplies the GPU provider. REFUSED rather than run on the CPU: a \
             table stamped `gpu` that a CPU produced would pass every gate and be wrong.",
        ),
        None => fail(&format!(
            "--device {device_s}: unknown device class. Known: cpu, gpu. An unrecognised class \
             is refused rather than defaulted — defaulting would stamp this build's class onto \
             an artifact the caller meant for another one."
        )),
    };

    let spec = GenSpec::new([species[0], species[1], species[2]], tg)
        .with_warm(warm)
        .with_device(device);

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
    // ECHOED AS PARSED, not as a constant. This line used to read a hardcoded "Cpu" — correct
    // while the class was not selectable and a lie the moment `--device` existed, which is the
    // trap "a diagnostic must echo its parameters" names: the string was right for a year and
    // wrong the instant the thing it described became a variable.
    println!("device class  {device} (DECLARED — this table is bit-gated, so D0 pins it)");
    match (&seam_loci, &seam_floor) {
        (Some(l), _) => println!("seams         LOCATED: {l}  (instrument {})",
                                 seam_instrument.as_deref().unwrap_or("?")),
        (_, Some(f)) => println!("seams         ACCEPTED FLOOR: {f}"),
        _ => unreachable!("checked above"),
    }
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

    // ---- the shipped artifact: SATURATION3/trimer-table/v1.
    //
    // JSON with the same provenance discipline the pair tables carry, and deliberately WITHOUT
    // their `converged: true` field -- that is the M-EXIT-DISCRIMINATOR shape. Every heavy
    // all-electron solve here exits `stagnated` at a residual just under the expansion floor,
    // so a boolean derived from a residual would be a threshold masquerading as an outcome.
    // Exit reasons ship as a histogram instead.
    let mut hist = [0u64; 4];
    for r in &o.records {
        hist[(r.exit_code as usize).min(3)] += 1;
    }
    let esc = |v: &str| v.replace('\\', "\\\\").replace('"', "\\\"");
    let mut f = std::fs::File::create(&out)
        .unwrap_or_else(|e| fail(&format!("cannot create {out}: {e}")));
    writeln!(f, "{{").unwrap();
    writeln!(f, "  \"schema\": \"SATURATION3/trimer-table/v1\",").unwrap();
    writeln!(f, "  \"provenance\": \"engine-computed STO-3G FCI (determinant, Knowles-Handy), f64, generated through holon-tables' leased mesh\",").unwrap();
    writeln!(f, "  \"solver_route\": \"determinant\",").unwrap();
    // THE THREE-AXIS IDENTITY, together, because they are one identity rather than three
    // diagnostics: the DEVICE CLASS (which arithmetic produced the bits), the SOLVER BUDGET
    // (which regime the solves ran under) and the SUBTRACTION BASIS (what the stored number
    // is a residual of). Each was learned the same way — a number moved and nothing recorded
    // which regime made it. A consumer comparing two tables cannot know they are comparable
    // unless the file says so on all three.
    writeln!(f, "  \"device_class\": \"{}\",", spec.device).unwrap();
    writeln!(f, "  \"solver_budget_iterations\": {},", spec.max_iter).unwrap();
    writeln!(f, "  \"subtraction_basis\": \"{}\",",
        esc(holon_tables::TrimerSurface::new(spec.species).basis())).unwrap();
    writeln!(f, "  \"exact_in_model\": true,").unwrap();
    writeln!(f, "  \"model\": \"({})/STO-3G/FCI\",", esc(&species_s)).unwrap();
    writeln!(f, "  \"species\": [{}],",
        species.iter().zip(species_s.split(','))
            .map(|(sp, sym)| format!("{{\"symbol\": \"{}\", \"Z\": {}}}", esc(sym), sp.z))
            .collect::<Vec<_>>().join(", ")).unwrap();
    writeln!(f, "  \"domain\": {{\"x_bohr\": [{x_lo}, {x_hi}], \"y_bohr\": [{y_lo}, {y_hi}], \"u\": [{u_lo}, {u_hi}]}},").unwrap();
    // THE AXIS RULE, AND THE NODE COORDINATES THEMSELVES.
    //
    // Spans plus counts do NOT determine node positions -- they say nothing about interior
    // spacing -- and a consumer handed only corners has to GUESS uniform. That guess is
    // wrong for this build's H3 table, which places r by `trimer::r_of_tau` with
    // STRETCH_A = 2.0 and its angle axis by `node_c`. A loader assuming uniform on a
    // stretched grid interpolates smoothly, plausibly, and wrongly everywhere except the
    // boundary.
    //
    // THIS generator is uniform-linear, which is NOT the H3 rule despite the node counts
    // matching. So the rule is named AND the coordinates ship: 79 floats against 14,157
    // energies costs nothing and makes the artifact self-describing, so a loader verifies
    // instead of assuming and can refuse a rule it cannot reproduce.
    writeln!(f, "  \"axis_rule\": {{\"x\": \"uniform-linear\", \"y\": \"uniform-linear\", \"u\": \"uniform-linear\", \"note\": \"NOT trimer.rs's tau-stretch; the coordinates below are authoritative\"}},").unwrap();
    {
        let axis = |lo: f64, hi: f64, n: usize| -> String {
            (0..n)
                .map(|i| {
                    let v = if n == 1 { lo } else { lo + (hi - lo) * (i as f64) / ((n - 1) as f64) };
                    format!("{v:?}")
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        writeln!(f, "  \"x_nodes\": [{}],", axis(x_lo, x_hi, grid[0])).unwrap();
        writeln!(f, "  \"y_nodes\": [{}],", axis(y_lo, y_hi, grid[1])).unwrap();
        writeln!(f, "  \"u_nodes\": [{}],", axis(u_lo, u_hi, grid[2])).unwrap();
    }
    writeln!(f, "  \"grid\": {{\"nx\": {}, \"ny\": {}, \"nu\": {}, \"region\": [{}, {}, {}], \"n_nodes\": {}}},",
        grid[0], grid[1], grid[2], region[0], region[1], region[2], tg.n_nodes()).unwrap();
    writeln!(f, "  \"warm_policy\": \"{warm:?}\",").unwrap();
    match (&seam_loci, &seam_floor) {
        (Some(l), _) => writeln!(f, "  \"seams\": {{\"scanned\": true, \"instrument\": \"{}\", \"loci\": \"{}\", \"accepted_floor\": null}},",
            esc(seam_instrument.as_deref().unwrap_or("")), esc(l)).unwrap(),
        (_, Some(fl)) => writeln!(f, "  \"seams\": {{\"scanned\": true, \"loci\": [], \"accepted_floor\": \"{}\"}},", esc(fl)).unwrap(),
        _ => unreachable!(),
    }
    writeln!(f, "  \"digest\": \"{}\",", o.digest().hex()).unwrap();
    writeln!(f, "  \"digest_covers\": [\"node\", \"energy\", \"d1\", \"d2\", \"status\"],").unwrap();
    writeln!(f, "  \"exit_histogram\": {{\"converged\": {}, \"iteration_cap\": {}, \"stagnated\": {}, \"trivial\": {}}},",
        hist[0], hist[1], hist[2], hist[3]).unwrap();
    writeln!(f, "  \"voided\": {{\"count\": {}, \"nodes\": [{}]}},", voided,
        o.records.iter().filter(|r| !r.is_ok()).map(|r| r.node.to_string())
            .collect::<Vec<_>>().join(", ")).unwrap();
    writeln!(f, "  \"units\": \"Hartree atomic units: x,y in bohr, u dimensionless, energies in hartree\",").unwrap();
    writeln!(f, "  \"generation\": {{\"workers\": {workers}, \"wall_s\": {secs:.3}, \"cold_solves\": {}, \"warm_solves\": {}, \"davidson_iters\": {}}},",
        o.cold_solves, o.warm_solves, o.total_davidson_iters).unwrap();
    let col = |name: &str, f2: &dyn Fn(&holon_tables::NodeRecord) -> f64, last: bool| {
        format!("  \"{name}\": [{}]{}",
            o.records.iter().map(|r| format!("{:?}", f2(r))).collect::<Vec<_>>().join(", "),
            if last { "" } else { "," })
    };
    writeln!(f, "  \"node\": [{}],",
        o.records.iter().map(|r| r.node.to_string()).collect::<Vec<_>>().join(", ")).unwrap();
    writeln!(f, "{}", col("energy_hartree", &|r| r.energy(), false)).unwrap();
    writeln!(f, "{}", col("d1", &|r| f64::from_bits(r.d1_bits), false)).unwrap();
    writeln!(f, "{}", col("d2", &|r| f64::from_bits(r.d2_bits), true)).unwrap();
    writeln!(f, "}}").unwrap();

    println!();
    println!("wrote {out}");
}

fn loadavg() -> f64 {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok())
        .unwrap_or(f64::NAN)
}
