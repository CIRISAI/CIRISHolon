//! `qvm_acuity2` — QVM-ACUITY-2's driver: the dictionary of views, one procedure, four
//! unlabelled families (`conformance/qasm/QVM_ACUITY2_PREREG.md`).
//!
//! Per instance, in this order, and the order is the claim:
//!
//! ```text
//! sector: …
//! select[cadence]: view=<V> price=<P> alternatives=[…]     ← printed BEFORE any view runs
//! select[prefix]:  view=<V> price=<P> alternatives=[…]
//! {"id": …, "family": …, …}                                 ← after: the run, all four views by hand, the referee
//! ```
//!
//! The search is handed `views::Query` — gates, observable, acuity — and nothing else. The
//! family is written into the JSON line for the reader and is never passed to the search; PQ-5
//! re-runs every selection with the family revealed in `Query::label` and records whether
//! anything changed.
//!
//! ```text
//! qvm_acuity2 [--seed S] [--count K] [--families CTLD] [--ids 0,5,9] [--no-byhand]
//! ```

use holon::sector::{self, ObsValue, Observable};
use holon::views::{self, Constants, Instance, MpsStop, ProbeMode, Query, Selection, View, ViewRun};
use std::time::Instant;

fn die(msg: &str) -> ! {
    eprintln!("qvm_acuity2: {msg}");
    std::process::exit(2);
}

fn jnum(x: f64) -> String {
    if x.is_finite() {
        format!("{x:.6e}")
    } else {
        "null".into()
    }
}

fn jstr(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "/").replace('"', "'"))
}

fn jval(v: Option<ObsValue>) -> String {
    v.map_or("null".into(), |v| v.as_json())
}

fn run_json(r: &ViewRun) -> String {
    format!(
        "{{\"wall\": {}, \"certificate\": {}, \"reached\": {}, \"value\": {}, \"chi\": {}, \"fraction\": {}, \"note\": {}}}",
        jnum(r.wall),
        jnum(r.certificate),
        r.reached,
        jval(r.value),
        r.chi.map_or("null".into(), |c| c.to_string()),
        r.fraction.map_or("null".into(), jnum),
        jstr(&r.note)
    )
}

fn sel_json(s: &Selection) -> String {
    let priced: Vec<String> = s
        .priced
        .iter()
        .map(|p| {
            format!(
                "{}: {{\"closed\": {}, \"price\": {}, \"detail\": {}}}",
                jstr(p.view.name()),
                p.closed,
                jnum(p.price),
                jstr(&p.detail)
            )
        })
        .collect();
    let probe: Vec<String> = s
        .probe
        .at
        .iter()
        .map(|a| {
            let cuts: Vec<String> = a.per_cut.iter().map(|w| format!("{w:.3e}")).collect();
            format!(
                "{{\"chi\": {}, \"measured\": {}, \"projected\": {}, \"gates_probed\": {}, \"max_bond\": {}, \"closed\": {}, \"per_cut\": [{}]}}",
                a.chi,
                jnum(a.measured),
                jnum(a.projected),
                a.gates_probed,
                a.max_bond,
                a.closed,
                cuts.join(", ")
            )
        })
        .collect();
    format!(
        "{{\"selected\": {}, \"line\": {}, \"prices\": {{{}}}, \"probe_chi\": {}, \"probe\": [{}], \"probe_wall\": {}, \"search_wall\": {}, \"ranking\": [{}]}}",
        jstr(s.selected.name()),
        jstr(&s.line),
        priced.join(", "),
        s.probe.chi.map_or("null".into(), |c| c.to_string()),
        probe.join(", "),
        jnum(s.probe.wall),
        jnum(s.wall_search),
        s.ranking().iter().map(|v| jstr(v.name())).collect::<Vec<_>>().join(", ")
    )
}

/// The same selection, compared field by field (PQ-5): view, every closure, every price to the
/// bit, the probe's chi.
fn same_selection(a: &Selection, b: &Selection) -> bool {
    a.selected == b.selected
        && a.probe.chi == b.probe.chi
        && a.priced.iter().zip(&b.priced).all(|(x, y)| x.closed == y.closed && x.price.to_bits() == y.price.to_bits())
}

fn gates_json(inst: &Instance) -> String {
    use holon::affine::Gate;
    let g: Vec<String> = inst
        .circuit
        .gates
        .iter()
        .map(|g| match *g {
            Gate::X(q) => format!("[\"x\",{q}]"),
            Gate::Z(q) => format!("[\"z\",{q}]"),
            Gate::S(q) => format!("[\"s\",{q}]"),
            Gate::Sdg(q) => format!("[\"sdg\",{q}]"),
            Gate::H(q) => format!("[\"h\",{q}]"),
            Gate::T(q) => format!("[\"t\",{q}]"),
            Gate::Tdg(q) => format!("[\"tdg\",{q}]"),
            Gate::Cx(c, t) => format!("[\"cx\",{c},{t}]"),
        })
        .collect();
    format!("[{}]", g.join(","))
}

fn run_selected(q: &Query, s: &Selection) -> ViewRun {
    let chi = if s.selected == View::Mps { s.probe.chi } else { None };
    views::run_view(q, s.selected, q.eps, chi, MpsStop::Never)
}

/// The by-hand SUM in a child process (`--hand-sum FAM IDX SEED EPS`), killed at the cap plus
/// the child's own instance generation. Returns (run JSON, abs-error JSON).
fn hand_sum_child(inst: &Instance, seed: u64, eps: f64) -> (String, String) {
    use std::io::Read;
    use std::process::{Command, Stdio};
    let exe = std::env::current_exe().unwrap_or_else(|_| die("cannot find my own executable"));
    let mut child = Command::new(exe)
        .args([
            "--hand-sum".to_string(),
            inst.family.to_string(),
            inst.index.to_string(),
            seed.to_string(),
            format!("{eps:e}"),
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| die(&format!("spawn: {e}")));
    let t0 = Instant::now();
    loop {
        if child.try_wait().ok().flatten().is_some() {
            break;
        }
        if t0.elapsed().as_secs_f64() > views::CAP_S + 15.0 {
            let _ = child.kill();
            let _ = child.wait();
            let r = ViewRun {
                view: View::Sum,
                value: None,
                certificate: f64::INFINITY,
                wall: f64::INFINITY,
                reached: false,
                note: format!("> {} s (child killed)", views::CAP_S),
                chi: None,
                fraction: None,
            };
            return (run_json(&r), "null".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let mut out = String::new();
    child.stdout.take().map(|mut o| o.read_to_string(&mut out));
    let mut lines = out.lines();
    let j = lines.next().unwrap_or("null").to_string();
    let e = lines.next().unwrap_or("null").to_string();
    (j, e)
}

fn child_main(argv: &[String]) -> ! {
    let fam = argv[0].chars().next().unwrap_or_else(|| die("--hand-sum FAM IDX SEED EPS"));
    let idx: usize = argv[1].parse().unwrap_or_else(|_| die("--hand-sum FAM IDX SEED EPS"));
    let seed: u64 = argv[2].parse().unwrap_or_else(|_| die("--hand-sum FAM IDX SEED EPS"));
    let eps: f64 = argv[3].parse().unwrap_or_else(|_| die("--hand-sum FAM IDX SEED EPS"));
    let inst = views::family_instance(fam, idx, seed);
    let q = inst.query();
    let r = views::run_view(&q, View::Sum, eps, None, MpsStop::Never);
    let referee = sector::referee::value(q.n, &q.gates, &q.obs);
    println!("{}", run_json(&r));
    println!("{}", r.value.map(|v| v.abs_diff(referee)).map_or("null".into(), jnum));
    std::process::exit(0);
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.first().map(String::as_str) == Some("--hand-sum") {
        if argv.len() != 5 {
            die("--hand-sum FAM IDX SEED EPS");
        }
        child_main(&argv[1..]);
    }
    let mut seed = 1u64;
    let mut count = 20usize;
    let mut fams = "CTLD".to_string();
    let mut ids: Option<Vec<usize>> = None;
    let mut byhand = true;
    let mut i = 0;
    while i < argv.len() {
        let val = || argv.get(i + 1).cloned().unwrap_or_else(|| die("a flag is missing its value"));
        match argv[i].as_str() {
            "--seed" => seed = val().parse().unwrap_or_else(|_| die("--seed S")),
            "--count" => count = val().parse().unwrap_or_else(|_| die("--count K")),
            "--families" => fams = val(),
            "--ids" => ids = Some(val().split(',').map(|x| x.parse().unwrap_or_else(|_| die("--ids"))).collect()),
            "--no-byhand" => {
                byhand = false;
                i += 1;
                continue;
            }
            f => die(&format!("unknown flag {f}")),
        }
        i += 2;
    }

    let t_all = Instant::now();
    let k: Constants = views::calibrate();
    println!("{}", k.line());
    println!("{{\"calibration\": {}}}", k.json());

    let instances = views::campaign(count, seed);
    for inst in &instances {
        if !fams.contains(inst.family) {
            continue;
        }
        if ids.as_ref().is_some_and(|v| !v.contains(&inst.id)) {
            continue;
        }
        let q = inst.query(); // no family in here
        let sel_c = views::select(&q, &k, ProbeMode::Cadence);
        let sel_p = views::select(&q, &k, ProbeMode::Prefix);
        println!("{}", sel_c.sector_line);
        println!("select[cadence]: {}", &sel_c.line["select: ".len()..]);
        println!("select[prefix]: {}", &sel_p.line["select: ".len()..]);

        // PQ-5: the family revealed
        let sel_l = views::select(&inst.query_labelled(), &k, ProbeMode::Cadence);
        let pq5 = same_selection(&sel_c, &sel_l);

        // the run of the selection (both readings; the prefix one only if it differs)
        let run_c = run_selected(&q, &sel_c);
        let differs = sel_p.selected != sel_c.selected || (sel_p.selected == View::Mps && sel_p.probe.chi != sel_c.probe.chi);
        let run_p = if differs { run_selected(&q, &sel_p) } else { run_c.clone() };

        // the referee — the search never sees it
        let t_ref = Instant::now();
        let referee = sector::referee::value(q.n, &q.gates, &q.obs);
        let wall_ref = t_ref.elapsed().as_secs_f64();
        let err = |r: &ViewRun| r.value.map(|v| v.abs_diff(referee));

        // all four by hand
        let mut hand_json: Vec<String> = Vec::new();
        let mut hand_err: Vec<String> = Vec::new();
        let mut mps_search_wall = f64::NAN;
        let mut extra = Vec::new();
        if byhand {
            let tab = views::run_view(&q, View::Tableau, q.eps, None, MpsStop::Never);
            // the sum's branch construction cannot be interrupted, so it runs in a child
            // process that is killed at the cap
            let (sum_j, sum_e) = hand_sum_child(inst, seed, q.eps);
            let (mps, msw) = views::mps_by_hand(&q);
            mps_search_wall = msw;
            let dense = views::run_view(&q, View::Dense, q.eps, None, MpsStop::Never);
            for (name, j, e) in [
                ("tableau", run_json(&tab), err(&tab).map_or("null".into(), jnum)),
                ("sum", sum_j, sum_e),
                ("mps", run_json(&mps), err(&mps).map_or("null".into(), jnum)),
                ("dense", run_json(&dense), err(&dense).map_or("null".into(), jnum)),
            ] {
                hand_json.push(format!("{}: {}", jstr(name), j));
                hand_err.push(format!("{}: {}", jstr(name), e));
            }
            // S5's comparison class and S4's full method (the family is used HERE, for the
            // record's comparison columns, after the search has run)
            if inst.family == 'T' {
                let (fj, fe) = hand_sum_child(inst, seed, 0.0);
                extra.push(format!("\"sum_full\": {fj}, \"sum_full_abs_error\": {fe}"));
            }
            if inst.family == 'L' {
                let m64 = views::run_view(&q, View::Mps, q.eps, Some(64), MpsStop::Never);
                extra.push(format!("\"mps64\": {}, \"mps64_abs_error\": {}", run_json(&m64), err(&m64).map_or("null".into(), jnum)));
            }
        }

        let obs_kind = match inst.observable {
            Observable::Amplitude(_) => "amplitude",
            Observable::Marginal { .. } => "marginal",
        };
        let mut line = format!(
            "{{\"id\": {}, \"family\": {}, \"seed\": {}, \"n\": {}, \"t\": {}, \"gates\": {}, \"t_eff\": {}, \
             \"observable\": {}, \"observable_kind\": {}, \"eps_rel\": {}, \"eps\": {}, \
             \"select_cadence\": {}, \"select_prefix\": {}, \"pq5_label_revealed_same\": {}, \
             \"selected\": {}, \"run\": {}, \"abs_error\": {}, \
             \"selected_prefix\": {}, \"run_prefix\": {}, \"abs_error_prefix\": {}, \
             \"referee\": {}, \"wall_referee\": {}, \"hand\": {{{}}}, \"hand_abs_error\": {{{}}}, \"mps_hand_search_wall\": {}",
            inst.id,
            jstr(&inst.family.to_string()),
            inst.seed,
            inst.n,
            inst.t,
            inst.circuit.gates.len(),
            sel_c.t_eff,
            jstr(&inst.observable.describe()),
            jstr(obs_kind),
            jnum(inst.eps_rel),
            jnum(inst.eps),
            sel_json(&sel_c),
            sel_json(&sel_p),
            pq5,
            jstr(sel_c.selected.name()),
            run_json(&run_c),
            err(&run_c).map_or("null".into(), jnum),
            jstr(sel_p.selected.name()),
            run_json(&run_p),
            err(&run_p).map_or("null".into(), jnum),
            referee.as_json(),
            jnum(wall_ref),
            hand_json.join(", "),
            hand_err.join(", "),
            jnum(mps_search_wall),
        );
        for e in extra {
            line.push_str(", ");
            line.push_str(&e);
        }
        if inst.family == 'C' {
            line.push_str(&format!(", \"circuit\": {}", gates_json(inst)));
        }
        line.push('}');
        println!("{line}");
    }
    eprintln!("qvm_acuity2: done in {:.1} s", t_all.elapsed().as_secs_f64());
}
