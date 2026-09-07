//! A whole campaign — gate, run, read — in one file, so a new one has a shape to copy.
//! The physics is the smallest thing that is still a measurement: an entry under the rent
//! clause (`Core/Maintenance.lean`), `S -> S - g*S + a`, and where an underpaid one settles.
//! Branches: (a) it holds, (b) it settles above zero, (c) it goes to zero. Phases are
//! `gate [dir]` (gates, the plant's pre-check, price.json, gate.json), `run [dir]` (the
//! counted steps -> run.json, run.done) and `read [dir]` (the branch -> read.json).

use holon_campaign::{is_done, read_record, Gate, Plant, Price, Record, RecordWriter, Report, Stake};
use std::time::Instant;

const GAMMA: f64 = 0.02;
const ALPHA: f64 = 0.01;
const S0: f64 = 1.0;
const PRICE_STEPS: u64 = 200;
const COUNTED_STEPS: u64 = 200_000;
const PRICE_BAND: (f64, f64) = (0.1, 10.0);
/// The plant's carrier floor and its stake: the freeze's own two numbers.
const CARRIER_FLOOR: f64 = 0.05;
const PLANT_STAKE: f64 = 0.20;

fn step(g: f64, a: f64, s: f64) -> f64 {
    s - g * s + a
}

/// The clause's stationary level, closed form.
fn settles_at(g: f64, a: f64) -> f64 {
    a / g
}

fn run_steps(g: f64, a: f64, n: u64) -> f64 {
    (0..n).fold(S0, |s, _| step(g, a, s))
}

/// Does every underpaid step strictly lose, over `n` of them?
fn strictly_loses(n: u64) -> bool {
    let one = |s: f64| { let t = step(GAMMA, ALPHA, s); (t < s).then_some(t) };
    (0..n).try_fold(S0, |s, _| one(s)).is_some()
}

/// Measured on the first steps, so the caller can write it before counting anything.
fn measure_price() -> Price {
    let t = Instant::now();
    let _ = run_steps(GAMMA, ALPHA, PRICE_STEPS);
    let secs = t.elapsed().as_secs_f64().max(1e-9);
    Price::measure("the counted arm", PRICE_STEPS, secs, COUNTED_STEPS, PRICE_BAND)
        .expect("a price needs a denominator")
}

/// Double the decay. The carrier is the level in the sector the plant acts on; the reach is
/// computed from the closed form BEFORE anything runs, and printed beside the stake.
fn plant() -> Plant {
    let (one, two) = (settles_at(GAMMA, ALPHA), settles_at(2.0 * GAMMA, ALPHA));
    let stake =
        Stake::derived("the level moves by a fifth", PLANT_STAKE, &[], "the freeze's own number");
    let why = "|a/g - a/2g| / (a/g) = 1/2, from the closed form";
    let sector = "the stationary level";
    Plant::new("double the decay", sector, one, CARRIER_FLOOR, stake, (one - two).abs() / one, why)
}

fn gate(dir: &str) {
    let w = RecordWriter::new(dir);
    let mut r = Report::new();

    let held = run_steps(GAMMA, GAMMA * S0, 10_000);
    let g0 = Gate::new("G0").work(10_000).detail("rent_holds, on the instrument");
    r.gate(g0.leg_at("a = g*S holds S exactly", held == S0, held - S0));

    let g1 = Gate::new("G1").work(1_000).detail("underpaid_shrinks, measured not asserted");
    r.gate(g1.leg("every underpaid step strictly loses", strictly_loses(1_000)));

    let one = settles_at(GAMMA, ALPHA);
    let p = plant().measured((one - run_steps(2.0 * GAMMA, ALPHA, 10_000)).abs() / one);
    println!("{}", p.print());
    r.gate(p.gate());

    let price = measure_price();
    println!("{}", price.print());
    price.write(&w, "price.json").expect("the price writes before anything is counted");

    let rec = Record::new("gate").raw("gates", r.json()).plant(&p)
        .number("gamma", GAMMA).number("alpha", ALPHA).flag("admits", r.admits());
    w.write("gate.json", &rec).expect("gate.json writes");
    r.refusals().iter().for_each(|l| println!("REFUSED {l}"));
}

fn run(dir: &str) {
    let w = RecordWriter::new(dir);
    // The counted phase takes a `Priced`, and only writing a price makes one.
    let priced = measure_price().write(&w, "price.json").expect("the price writes");
    let t0 = Instant::now();
    let final_s = run_steps(GAMMA, ALPHA, COUNTED_STEPS);
    let check = priced.check(t0.elapsed().as_secs_f64());
    println!("{}", check.line());
    let rec = Record::new("run").int("counted_steps", COUNTED_STEPS as i64)
        .number("final_level", final_s).number("closed_form_level", settles_at(GAMMA, ALPHA));
    w.write("run.json", &rec.gate(&check)).expect("run.json writes");
    w.done("run.done", &format!("{COUNTED_STEPS} steps")).expect("run.done writes");
}

fn read(dir: &str) {
    if !is_done(dir, "run.done") {
        println!("run.done is ABSENT: the read below is on a run that did not finish.");
    }
    let reading = read_record(format!("{dir}/run.json")).expect("run.json reads");
    let counted = reading.count().unwrap_or_else(|e| panic!("{e}"));
    let level = counted.field("final_level").expect("final_level");
    let closed = counted.field("closed_form_level").expect("closed_form_level");
    println!("  {}\n  {}", level.cite(), closed.cite());
    let d = level.value - closed.value;
    let branch = match level.value {
        v if (v - S0).abs() < 1e-12 => 'a',
        v if v > 1e-9 => 'b',
        _ => 'c',
    };
    let g = Gate::new("R").work(1).branch(branch).leg_at("agrees with the closed form", d.abs() < 1e-9, d)
        .detail("(a) holds, (b) settles above zero, (c) goes to zero");
    println!("{}", g.line());
    RecordWriter::new(dir)
        .write("read.json", &Record::new("read").gate(&g).number("level", level.value))
        .expect("read.json writes");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let fallback = std::env::temp_dir().join("tiny-campaign").to_string_lossy().to_string();
    let dir = args.get(2).cloned().unwrap_or(fallback);
    match args.get(1).map(String::as_str).unwrap_or("gate") {
        "gate" => gate(&dir),
        "run" => run(&dir),
        "read" => read(&dir),
        other => panic!("unknown phase {other:?}: gate | run | read"),
    }
}
