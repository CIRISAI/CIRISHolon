//! Look at ONE block's held series directly, because a verdict is easier to trust when the
//! series behind it has been read rather than summarised.
//!
//! `cargo run -p holon-lens --example block_probe -- <traj> <block-hex>`

use holon_lens::partition;
use holon_lens::traj::Trajectory;

fn main() {
    let mut a = std::env::args().skip(1);
    let path = a.next().expect("trajectory");
    let block = u16::from_str_radix(a.next().expect("block hex").trim_start_matches("0x"), 16)
        .expect("hex block");
    let t = Trajectory::read(std::path::Path::new(&path)).expect("readable");
    let n = t.header.n_atoms;
    let times: Vec<f64> = t.frames.iter().map(|f| f.time * holon_lens::traj::AU_TIME_FS).collect();
    let series: Vec<bool> = t
        .frames
        .iter()
        .map(|f| partition::blocks(&partition::labels_from_bonds(n, f.bonded)).contains(&block))
        .collect();

    let members: Vec<usize> = (0..n).filter(|i| block >> i & 1 == 1).collect();
    println!(
        "# block {:#06x} = {} ; atoms {:?} ; Z {:?}",
        block,
        partition::formula(block, &t.header.z),
        members,
        members.iter().map(|&i| t.header.z[i]).collect::<Vec<_>>()
    );
    let held = series.iter().filter(|b| **b).count();
    println!("# held in {held} of {} frames ({:.1}%)", series.len(), 100.0 * held as f64 / series.len() as f64);

    // Runs of held and of breach, in simulated time.
    let mut runs: Vec<(bool, usize, f64)> = Vec::new();
    let mut i = 0usize;
    while i < series.len() {
        let v = series[i];
        let s = i;
        while i < series.len() && series[i] == v { i += 1; }
        let end_t = if i < series.len() { times[i] } else { times[series.len() - 1] };
        runs.push((v, i - s, end_t - times[s]));
    }
    let held_runs: Vec<&(bool, usize, f64)> = runs.iter().filter(|r| r.0).collect();
    let brk_runs: Vec<&(bool, usize, f64)> = runs.iter().filter(|r| !r.0).collect();
    let stat = |v: &[&(bool, usize, f64)]| {
        if v.is_empty() { return (0.0, 0.0, 0.0); }
        let mut d: Vec<f64> = v.iter().map(|r| r.2).collect();
        d.sort_by(|a, b| a.partial_cmp(b).unwrap());
        (d[0], d[d.len() / 2], d[d.len() - 1])
    };
    let (hl, hm, hh) = stat(&held_runs);
    let (bl, bm, bh) = stat(&brk_runs);
    println!("# HELD runs  : {:>5}  min {:.1} fs  median {:.1} fs  max {:.1} fs", held_runs.len(), hl, hm, hh);
    println!("# BREACH runs: {:>5}  min {:.1} fs  median {:.1} fs  max {:.1} fs", brk_runs.len(), bl, bm, bh);
    let over = brk_runs.iter().filter(|r| r.2 > 8.4).count();
    println!("# breaches longer than the 8.4 fs flicker cap: {over}");

    // The first window the budget would accept, spelled out frame by frame.
    if let Some((s, e, breach, worst)) =
        holon_lens::census::budgeted_window(&series, &times, 834.0, 0.02, 8.4)
    {
        println!(
            "# BUDGETED WINDOW: frames {s}..{e} ({:.1} fs), {breach} breached frames, longest breach {:.2} fs",
            times[e - 1] - times[s],
            worst
        );
        let inside: Vec<&(bool, usize, f64)> = runs
            .iter()
            .filter(|_| true)
            .collect::<Vec<_>>()
            .into_iter()
            .collect();
        let _ = inside;
        let n_breach = series[s..e].iter().filter(|b| !**b).count();
        println!("# inside that window: {} held, {} breached", (e - s) - n_breach, n_breach);
    } else {
        println!("# NO budgeted window");
    }
}
