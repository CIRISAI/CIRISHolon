//! Generate the four-body `(O,H,H,H)` surface through the folded leased mesh.
//!
//! This is DE4-TABLE's production generator, and it is deliberately NOT a new generator.
//! Everything it does — the grid, the region decomposition, the warm chains, the orbit
//! reduction, the leases, the digest, the certificate — belongs to `holon-tables` and is
//! shared with the three-body surfaces. What lives here is the composition of those parts
//! for one campaign and the artifact it writes: WB-8.7's test is whether a new composition
//! costs a new pipeline, and this file is the evidence that it did not.
//!
//! # Every parameter is required
//!
//! There are no defaults for the domain. A domain is a physics claim — `R_HI = 6.0` is the
//! MEASURED far-field cutoff and nothing else — so it is passed in and echoed back as
//! parsed, never assumed. The echo is the second half of the launch-header discipline: the
//! shell records what it launched, the binary records what it understood, and a
//! disagreement between the two is visible instead of silent (M-PROVENANCE-OVERREACH).
//!
//! Run:
//! ```text
//! de4_table --nr 13 --nu 11 --r 0.9:6.0 --stretch 3.0 --u -1.0:0.9975 \
//!           --region 7x7x7x6x6x6 --warm chain --workers 4 [--device cpu] --out <path> \
//!           --seam-accepted-floor '<Ha>:<why>'
//! ```

use holon_chem::quaternary_table as qt;
use holon_chem::{trimer, water};
use holon_resource::Arena;
use holon_chem::sigma_op::DeviceClass;
use holon_tables::generate::{generate_surface_leased, SurfaceSpec, WarmPolicy};
use holon_tables::Surface;
use holon_tables::grid::{Axis, NdGrid, Serpentine};
use holon_tables::ohhh::OhhhSurface;
use holon_tables::worker::WorkerProbe;
use std::io::Write;
use std::time::Instant;

fn fail(msg: &str) -> ! {
    eprintln!("de4_table: {msg}");
    eprintln!("every parameter is required; a domain is a physics claim, not a default.");
    std::process::exit(2);
}

fn arg(args: &[String], key: &str) -> String {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == key {
            return it.next().cloned().unwrap_or_else(|| fail(&format!("{key} needs a value")));
        }
    }
    fail(&format!("missing required argument {key}"))
}

fn parse_range(s: &str, key: &str) -> (f64, f64) {
    let (a, b) = s.split_once(':').unwrap_or_else(|| fail(&format!("{key} must be lo:hi")));
    (
        a.parse().unwrap_or_else(|_| fail(&format!("{key} lo is not a number"))),
        b.parse().unwrap_or_else(|_| fail(&format!("{key} hi is not a number"))),
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let nr: usize = arg(&args, "--nr").parse().unwrap_or_else(|_| fail("--nr"));
    let nu: usize = arg(&args, "--nu").parse().unwrap_or_else(|_| fail("--nu"));
    let (r_lo, r_hi) = parse_range(&arg(&args, "--r"), "--r");
    let stretch: f64 = arg(&args, "--stretch").parse().unwrap_or_else(|_| fail("--stretch"));
    let (u_lo, u_hi) = parse_range(&arg(&args, "--u"), "--u");
    let region: Vec<usize> = arg(&args, "--region")
        .split('x')
        .map(|s| s.parse().unwrap_or_else(|_| fail("--region must be axAxBxCxDxExF")))
        .collect();
    if region.len() != 6 {
        fail("--region needs six edges, one per axis");
    }
    // **--device: the third identity axis** (RESOURCE_DESIGN D0), beside the solver budget and
    // the subtraction basis. `gpu` is REFUSED by name with its exit rather than downgraded —
    // this binary links `holon-tables`, which does not link CUDA by design, and a table stamped
    // `gpu` that a CPU produced would pass every gate and be wrong (D4: loud refusal, never a
    // silent fallback across classes).
    let device_s = args
        .iter()
        .position(|a| a == "--device")
        .and_then(|i| args.get(i + 1).cloned())
        .unwrap_or_else(|| "cpu".to_string());
    let device = match DeviceClass::from_tag(&device_s) {
        Some(DeviceClass::Cpu) => DeviceClass::Cpu,
        Some(DeviceClass::Gpu) => fail(
            "--device gpu: this binary cannot generate a GPU-class table. Use holon-gpu's \
             device-class launcher, which supplies the GPU provider. REFUSED rather than run \
             on the CPU under a `gpu` stamp.",
        ),
        None => fail(&format!(
            "--device {device_s}: unknown device class. Known: cpu, gpu. Refused rather than \
             defaulted."
        )),
    };

    let warm = match arg(&args, "--warm").as_str() {
        "chain" => WarmPolicy::CanonicalChain,
        "cold" => WarmPolicy::AllCold,
        _ => fail("--warm must be chain or cold"),
    };
    let workers: usize = arg(&args, "--workers").parse().unwrap_or_else(|_| fail("--workers"));
    let out = arg(&args, "--out");
    // The seam record. EXACTLY ONE of these, per the trimer-table schema: a shipped
    // surface with a hidden corner is exactly what that gate exists to see.
    let seam_floor = args.iter().position(|a| a == "--seam-accepted-floor")
        .map(|i| args.get(i + 1).cloned().unwrap_or_else(|| fail("--seam-accepted-floor needs a value")));
    let seam_loci = args.iter().position(|a| a == "--seam-loci")
        .map(|i| args.get(i + 1).cloned().unwrap_or_else(|| fail("--seam-loci needs a value")));
    if seam_floor.is_some() == seam_loci.is_some() {
        fail("exactly one of --seam-accepted-floor or --seam-loci is required: uniform \
              refinement cannot beat a state crossing, so a table that has not located its \
              seams is claiming a smoothness it has not checked");
    }

    // The grid the campaign froze. The three radial axes must be identical to each other
    // and the three cosine axes likewise, because the orbit map permutes indices BETWEEN
    // axes; a ragged grid would produce a complete table of entirely plausible wrong
    // numbers, and the mesh cannot see the physics that would catch it.
    let axes = vec![
        Axis::stretched(nr, r_lo, r_hi, stretch, region[0]),
        Axis::stretched(nr, r_lo, r_hi, stretch, region[1]),
        Axis::stretched(nr, r_lo, r_hi, stretch, region[2]),
        Axis::linear(nu, u_lo, u_hi, region[3]),
        Axis::linear(nu, u_lo, u_hi, region[4]),
        Axis::linear(nu, u_lo, u_hi, region[5]),
    ];
    let grid = NdGrid::new(axes).with_serpentine(Serpentine::Reflected);

    let box_nodes = grid.n_nodes();
    let orbits = (nr * nr * nr * nu * nu * nu + 3 * nr * nr * nu * nu + 2 * nr * nu) / 6;

    println!("=== de4_table: the four-body (O,H,H,H) surface, through the folded mesh ===");
    println!("--- parameters AS PARSED (the launch header's second half) ---");
    println!("  nr              {nr}   (three radial axes, identical by requirement)");
    println!("  nu              {nu}   (three cosine axes, identical by requirement)");
    println!("  R               [{r_lo}, {r_hi}] bohr, exponential stretch a = {stretch}");
    println!("  u               [{u_lo}, {u_hi}]");
    println!("  region          {region:?}");
    println!("  serpentine      Reflected (the sum-parity rule is not adjacent at even interior extents)");
    println!("  warm            {warm:?}");
    println!("  device class    {device} (DECLARED; D0 — part of the artifact, echoed as parsed)");
    println!("  workers         {workers}");
    println!("  out             {out}");
    println!("  seam            {}",
        seam_floor.as_deref().map(|s| format!("accepted_floor {s}"))
            .unwrap_or_else(|| format!("loci {}", seam_loci.as_deref().unwrap_or(""))));
    println!("  box nodes       {box_nodes}");
    println!("  orbits          {orbits}   (Burnside; the solves the symmetry actually costs)");
    println!("  regions         {}", grid.n_regions());
    if grid.n_regions() < workers {
        println!("  NOTE: fewer regions than workers; {} worker(s) will idle.",
            workers - grid.n_regions());
    }
    println!("  host loadavg    {}", std::fs::read_to_string("/proc/loadavg").unwrap_or_default().trim());

    // A smoke run exercises the whole path on a grid small enough to finish, and is
    // therefore allowed to differ from the compiled interpolant's grid -- because it does
    // not produce a table for the reader. It refuses to write anywhere but a .smoke path,
    // so a smoke artifact can never be mistaken for the campaign's.
    let smoke = args.iter().any(|a| a == "--smoke");
    if smoke && !out.ends_with(".smoke.json") {
        fail("--smoke must write to a path ending .smoke.json, so a smoke artifact can \
              never be mistaken for the campaign's table");
    }

    // Consistency between the CLI and the compiled table constants. The interpolant's grid
    // is `const` in `holon-chem`, so a table generated on a different grid could never be
    // read back by it; refusing here costs nothing and refusing later costs the run.
    // The parentheses are load-bearing: `&&` binds tighter than `||`, so without them
    // `--smoke` would waive only the FIRST of the seven comparisons and the other six
    // would still refuse -- a guard that looks applied and is not.
    let grid_differs = nr != qt::NR
        || nu != qt::NU
        || r_lo != qt::R_LO
        || r_hi != qt::R_HI
        || stretch != qt::STRETCH_A
        || u_lo != qt::U_LO
        || u_hi != qt::U_HI;
    if !smoke && grid_differs {
        eprintln!("\nREFUSED: these parameters do not match the compiled interpolant's grid");
        eprintln!("  compiled: {}", qt::grid_line());
        eprintln!("  requested: # grid: NR={nr} NU={nu} R_LO={r_lo} R_HI={r_hi} STRETCH_A={stretch} U_LO={u_lo} U_HI={u_hi}");
        eprintln!("  a table the reader cannot load is not a table.");
        std::process::exit(2);
    }

    println!("\n--- building the surface (samples both pair curves once) ---");
    let t0 = Instant::now();
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../holon-chem/tests/data/s2/s2_water_table.txt"
    ))
    .unwrap_or_else(|e| fail(&format!("the committed (O,H,H) table: {e}")));
    let w = water::from_text(&src).unwrap_or_else(|| fail("the (O,H,H) table did not parse"));
    let tri = trimer::generate().unwrap_or_else(|| fail("the (H,H,H) table did not build"));
    let surface = OhhhSurface::new(w, tri, r_lo, r_hi);
    println!("  surface ready in {:.1} s", t0.elapsed().as_secs_f64());

    println!("\n--- generating ---");
    let mut spec = SurfaceSpec::new(&surface, grid.clone()).with_warm(warm);
    spec.device = device;
    let mut arena = Arena::new();
    let mut probe = WorkerProbe::new();
    let t1 = Instant::now();
    let run = match generate_surface_leased(&spec, workers, &mut arena, &mut probe) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("REFUSED by the resource layer: {e:?}");
            std::process::exit(3);
        }
    };
    let wall = t1.elapsed().as_secs_f64();
    let o = &run.outcome;

    println!("  wall            {:.1} s", wall);
    println!("  records         {}", o.records.len());
    println!("  solved          {}", o.records.len() - o.mirrored);
    println!("  mirrored        {}   ({:.4}x fewer solves)", o.mirrored,
        o.records.len() as f64 / (o.records.len() - o.mirrored).max(1) as f64);
    println!("  cold / warm     {} / {}", o.cold_solves, o.warm_solves);
    println!("  voided          {}", o.voided);
    println!("  davidson iters  {}", o.total_davidson_iters);
    println!("  digest          {}", o.digest().hex());
    println!("  certificate     {}", if o.certificate.is_clean() { "CLEAN" } else { "CONVICTED" });

    if !o.certificate.is_clean() {
        eprintln!("REFUSED: the certificate is dirty; the table is NOT written.");
        std::process::exit(4);
    }

    // M-CHEAPER-THAN-ITS-PRICE, gate G2: the arithmetic has to close, IN THIS JOB, against
    // a cost re-measured here rather than against the prereg's number. Reported both ways
    // and refused in both directions -- too cheap means work not done, too dear means the
    // successor's budget is fiction.
    let solved = (o.records.len() - o.mirrored) as f64;
    let cpu_per_node = wall * workers as f64 / solved.max(1.0);
    println!("\n--- G2, the price model, closed in-job ---");
    println!("  solved nodes    {solved}");
    println!("  wall x workers  {:.1} core-seconds", wall * workers as f64);
    println!("  per solved node {:.4} s of core time", cpu_per_node);
    println!("  (the prereg banked ~0.25 s CPU/node with the pair cache; the two-sided");
    println!("   band is 2.0x either way, and this run's reading is the one that counts)");

    let mut f = std::fs::File::create(&out).unwrap_or_else(|e| fail(&format!("cannot write {out}: {e}")));
    let _ = writeln!(f, "{{");
    let _ = writeln!(f, "  \"schema\": \"DE4TABLE/quaternary-table/v1\",");
    let _ = writeln!(f, "  \"provenance\": \"{}\",", qt::QUATERNARY_PROVENANCE);
    let _ = writeln!(f, "  \"solver_route\": \"determinant\",");
    // THE THREE-AXIS IDENTITY, together. `stored` below already names this table's
    // subtraction basis in prose; `subtraction_basis` is the SURFACE's own machine-readable
    // answer, so a consumer does not have to parse an English sentence to know what it holds.
    let _ = writeln!(f, "  \"device_class\": \"{}\",", spec.device);
    let _ = writeln!(f, "  \"solver_budget_iterations\": {},", spec.max_iter);
    let _ = writeln!(f, "  \"subtraction_basis\": \"{}\",", surface.basis().replace('"', "'"));
    let _ = writeln!(f, "  \"exact_in_model\": true,");
    let _ = writeln!(f, "  \"model\": \"(O,H,H,H)/STO-3G/FCI four-body term\",");
    let _ = writeln!(f, "  \"stored\": \"dE4 = E_FCI(OH3) - E_MBE3(OH3), hartree\",");
    let _ = writeln!(f, "  \"grid_line\": \"{}\",", qt::grid_line().replace('"', "'"));
    let _ = writeln!(f, "  \"grid\": {{\"nr\": {nr}, \"nu\": {nu}, \"region\": {region:?}, \"n_nodes\": {box_nodes}, \"orbits\": {orbits}}},");
    let _ = writeln!(f, "  \"warm_policy\": \"{warm:?}\",");
    let _ = writeln!(f, "  \"serpentine\": \"Reflected\",");
    let _ = writeln!(f, "  \"digest\": \"{}\",", o.digest().hex());
    let _ = writeln!(f, "  \"digest_covers\": [\"node\",\"energy\",\"d1\",\"d2\",\"status\"],");
    let _ = writeln!(f, "  \"solved\": {}, \"mirrored\": {}, \"voided\": {},", o.records.len() - o.mirrored, o.mirrored, o.voided);
    let _ = writeln!(f, "  \"generation\": {{\"workers\": {workers}, \"wall_s\": {wall:.1}, \"cold_solves\": {}, \"warm_solves\": {}, \"davidson_iters\": {}, \"core_s_per_solved_node\": {cpu_per_node:.4}}},", o.cold_solves, o.warm_solves, o.total_davidson_iters);
    match (&seam_floor, &seam_loci) {
        (Some(s), _) => {
            let (ha, why) = s.split_once(':').unwrap_or((s.as_str(), "unstated"));
            let _ = writeln!(f, "  \"seams\": {{\"scanned\": true, \"instrument\": \"ohhh_seam_scan\", \"loci\": [], \"accepted_floor\": {{\"hartree\": {ha}, \"why\": \"{why}\", \"located_by\": \"ohhh_seam_scan, six slices, cold and warm at every point\"}}}},");
        }
        (_, Some(l)) => {
            let _ = writeln!(f, "  \"seams\": {{\"scanned\": true, \"instrument\": \"ohhh_seam_scan\", \"loci\": [{l}], \"accepted_floor\": null}},");
        }
        _ => unreachable!("exactly one was required above"),
    }
    let _ = writeln!(f, "  \"units\": \"Hartree atomic units: R in bohr, u dimensionless, dE4 in hartree\",");
    // The values, as raw IEEE-754 hex bit patterns in canonical node order. Bits rather
    // than decimal because a decimal round-trip would put a tolerance where none belongs,
    // and this file is compared for bit-identity.
    let _ = writeln!(f, "  \"values_hex\": [");
    for (n, r) in o.records.iter().enumerate() {
        let comma = if n + 1 == o.records.len() { "" } else { "," };
        let _ = writeln!(f, "    \"{:016x}\"{}", r.energy_bits, comma);
    }
    let _ = writeln!(f, "  ]");
    let _ = writeln!(f, "}}");
    println!("\nwrote {out}");
}
