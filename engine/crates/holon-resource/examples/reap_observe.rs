//! **D10b's instrument: the reaper's false-positive rate, measured against a LIVE generation
//! with the reaper OFF.**
//!
//! RESOURCE_DESIGN D10b says the reaper stays off until its false-positive rate is measured
//! across at least one full real generation, and it says so because the component convicted
//! **1115 live holders** the first time it met real work. The 72-node exercise that produced
//! that number was an exercise. This is the campaign: the observer attaches to a generation
//! that is genuinely running — SATURATION-2's `(O,O,O)` ozone tabulation, 14,025 solves across
//! 32 worker threads — and records, poll by poll, what the reaper WOULD have decided.
//!
//! # Nothing here can reap
//!
//! Only [`Reaper::judge`] is called, never `sweep_one` and never `sweep`; `judge` does not take
//! the arena and cannot convict. That is not left as a promise: the run ends by asserting the
//! arena's own ledger shows `convicted == 0` and `reaped == 0`, so "the reaper stayed off" is a
//! fact the books carry rather than a claim in a comment. Nothing signals, kills, or writes to
//! the observed process; the observer reads `/proc` and an append-only log, and its own rung-3
//! probe writes one dot-file into the probe directory and unlinks it.
//!
//! # The holder, and why there is exactly one of it
//!
//! The holder is the GENERATION, and its receipts are exact: one line appended to the
//! checkpoint log is one node solved — a receipt of REAL WORK, which is what §9 Q1 says rent
//! is. Per-worker-thread holders were considered and REFUSED: from outside the process a
//! thread's receipts are not separable from the shared log, and the only per-thread observable
//! is its CPU time — which is rung 2's sensor. Using it as rung 1's receipt would collapse the
//! two rungs onto one sensor and measure the ladder's own defect rather than the work. One
//! holder with exact receipts is a smaller instrument than 32 with invented ones, and it is an
//! instrument.
//!
//! The denominator is POLLS, not holders: every poll at which the holder is genuinely alive and
//! the policy says REAP is one false positive, which is exactly the quantity §5's table counts.
//!
//! # Four policies, because the question is which rung fixed it
//!
//! §5 measured four configurations on the 72-node exercise and found that the fix was rung 1's
//! grace being sized by the holder's own step, not anything in rung 2. That comparison is
//! re-run here against real work:
//!
//! | policy | rung 1 grace | rung 2 | rung 3 |
//! |---|---|---|---|
//! | `P1_rung2_absent` | flat, declared | ABSENT (always "not scheduling") | absent |
//! | `P2_cpu_tick` | flat, declared | CPU time advanced since the LAST poll | present |
//! | `P3_debounced` | flat, declared | CPU time advanced over the last `debounce` polls | present |
//! | `P4_own_step` | `k` x the holder's OWN observed step | CPU time advanced since the last poll | present |
//!
//! P4 is the design's own rule. If it reads anything but zero on real work, D10b's answer is
//! that the reaper stays off, and this file is the evidence.
//!
//! # Every parameter is echoed
//!
//! The flat grace is a DECLARED constant, not a derived one — it is the shape of the constant
//! that produced 1115 false reaps, and inventing a defensible-looking derivation for it would
//! hide the very thing being measured. It is echoed in the header and in every output record,
//! together with the holder's own measured step distribution, so the ratio between them is
//! visible rather than asserted.
//!
//! Usage:
//! ```text
//! reap_observe --pid N --log PATH --probe-dir DIR [--poll-ms 2000] [--flat-grace-s 10]
//!              [--k 3] [--debounce 3] [--out PREFIX] [--max-polls N]
//! ```

use holon_resource::probe::{AttemptProbe, Probe, ProbeVerdict, ResourceKind};
use holon_resource::reaper::{ReapVerdict, Reaper, ReaperWorld};
use holon_resource::{Arena, LeaseId};
use std::io::Write;
use std::time::{Duration, Instant, SystemTime};

/// Which rung-1 grace and rung-2 sensor a policy uses. Named so the output cannot report a
/// number without saying which configuration produced it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Policy {
    /// Rung 2 absent entirely: rung 1 alone decides. The configuration §5 measured at 1115.
    Rung2Absent,
    /// Rung 2 = "did the CPU tick advance since the last poll" — §5's 108.
    CpuTick,
    /// The same sensor, debounced over several polls — §5's 495.
    Debounced,
    /// Grace sized by the holder's OWN observed step — §5's 0, and the design's rule.
    OwnStep,
    /// **The discriminator.** Own-step grace with rung 2 ABSENT, so nothing but rung 1's rule
    /// separates it from `Rung2Absent`.
    ///
    /// It exists because the first reading against this generation showed P2, P3 and P4 all at
    /// zero: at a 2 s poll against a CPU-saturated holder the CPU tick ALWAYS advances, so
    /// rung 2 masks rung 1 and P4's zero says nothing about the grace rule. That is a finding
    /// about the poll interval, not about the ladder, and the fix is a policy that cannot hide
    /// behind rung 2. If this reads zero while `Rung2Absent` reads high, the grace rule is what
    /// fixed it — which is §5's claim, isolated.
    OwnStepAlone,
}

impl Policy {
    const ALL: [Policy; 5] = [
        Policy::Rung2Absent,
        Policy::CpuTick,
        Policy::Debounced,
        Policy::OwnStep,
        Policy::OwnStepAlone,
    ];
    fn label(self) -> &'static str {
        match self {
            Policy::Rung2Absent => "P1_rung2_absent",
            Policy::CpuTick => "P2_cpu_tick",
            Policy::Debounced => "P3_debounced",
            Policy::OwnStep => "P4_own_step",
            Policy::OwnStepAlone => "P5_own_step_alone",
        }
    }
}

/// One poll's reading of the world, taken ONCE and shared by all four policies.
///
/// Taken once on purpose: four policies each sampling `/proc` at slightly different instants
/// would differ by their sampling as well as by their rule, and the comparison is supposed to
/// isolate the rule.
#[derive(Clone, Copy, Debug)]
struct Reading {
    /// Seconds since a receipt last appeared in the append-only log.
    silence_s: f64,
    /// The holder's own observed step: the largest inter-receipt interval seen so far.
    own_step_max_s: f64,
    /// Did the process's CPU time advance since the previous poll?
    cpu_advanced_now: bool,
    /// Did it advance at any point across the debounce window?
    cpu_advanced_debounced: bool,
    /// Is the holder genuinely alive? The ground truth this measurement is scored against.
    holder_alive: bool,
}

/// The world, answering for ONE policy out of a reading that was taken once.
struct PolicyWorld {
    policy: Policy,
    reading: Reading,
    flat_grace_s: f64,
    k: f64,
}

impl ReaperWorld for PolicyWorld {
    fn grace_expired(&mut self, _id: LeaseId) -> bool {
        let grace = match self.policy {
            // The holder's own step, which is what D10 says a grace must be a multiple of.
            // Until an interval has been observed there is no own-step to be a multiple of,
            // and a policy that reaps before it has measured anything is the flat-constant
            // defect wearing this policy's name — so an unmeasured step never expires.
            Policy::OwnStep | Policy::OwnStepAlone => {
                if self.reading.own_step_max_s <= 0.0 {
                    return false;
                }
                self.k * self.reading.own_step_max_s
            }
            _ => self.flat_grace_s,
        };
        self.reading.silence_s > grace
    }

    fn holder_scheduling(&mut self, _id: LeaseId) -> bool {
        match self.policy {
            Policy::Rung2Absent | Policy::OwnStepAlone => false,
            Policy::Debounced => self.reading.cpu_advanced_debounced,
            _ => self.reading.cpu_advanced_now,
        }
    }
}

/// Running tally per policy.
#[derive(Clone, Copy, Debug, Default)]
struct Tally {
    polls: u64,
    would_reap: u64,
    stood_down: u64,
    kept: u64,
    /// Would-reaps taken while the holder was demonstrably alive — the FALSE positives, which
    /// is the quantity D10b asks for.
    false_reaps: u64,
}

fn main() {
    let mut pid: Option<i32> = None;
    let mut log = String::new();
    let mut probe_dir = String::from("/tmp");
    let mut poll_ms: u64 = 2000;
    let mut flat_grace_s: f64 = 10.0;
    let mut k: f64 = 3.0;
    let mut debounce: usize = 3;
    let mut out_prefix = String::from("reap_observe");
    let mut max_polls: u64 = u64::MAX;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        let need = |i: usize| -> String {
            args.get(i + 1)
                .unwrap_or_else(|| panic!("{} needs a value", args[i]))
                .clone()
        };
        match args[i].as_str() {
            "--pid" => pid = Some(need(i).parse().expect("--pid must be an integer")),
            "--log" => log = need(i),
            "--probe-dir" => probe_dir = need(i),
            "--poll-ms" => poll_ms = need(i).parse().expect("--poll-ms"),
            "--flat-grace-s" => flat_grace_s = need(i).parse().expect("--flat-grace-s"),
            "--k" => k = need(i).parse().expect("--k"),
            "--debounce" => debounce = need(i).parse().expect("--debounce"),
            "--out" => out_prefix = need(i),
            "--max-polls" => max_polls = need(i).parse().expect("--max-polls"),
            other => panic!(
                "unknown argument {other}. This binary refuses arguments it does not \
                 understand rather than ignoring them: a CLI that silently drops a flag makes \
                 one command mean two things in two trees."
            ),
        }
        i += 2;
    }
    let pid = pid.expect("--pid is required: this observer attaches to a REAL generation");
    assert!(!log.is_empty(), "--log is required (the receipt stream)");
    assert!(debounce >= 1, "--debounce must be at least one poll");

    // ---- the launch header. Parameters echoed as PARSED, never as intended.
    let started = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let header = format!(
        "# reap_observe — RESOURCE_DESIGN D10b, the reaper's false-positive rate on REAL work\n\
         # started_unix    {started}\n\
         # pid             {pid}\n\
         # log             {log}\n\
         # probe_dir       {probe_dir}\n\
         # poll_ms         {poll_ms}\n\
         # flat_grace_s    {flat_grace_s}   (DECLARED, not derived — the shape of the constant \
         that convicted 1115 live holders)\n\
         # k               {k}   (P4's multiplier of the holder's OWN observed step)\n\
         # debounce        {debounce} polls\n\
         # max_polls       {max_polls}\n\
         # THE REAPER IS OFF: only Reaper::judge is called; the ledger is asserted at exit\n"
    );
    print!("{header}");

    let records_path = format!("{out_prefix}.jsonl");
    let mut records = std::io::BufWriter::new(
        std::fs::File::create(&records_path).expect("cannot create the record file"),
    );
    records.write_all(header.as_bytes()).unwrap();

    // ---- the arena. One real lease, admitted by a real probe, never convicted.
    let mut arena = Arena::new();
    let mut admit = AttemptProbe::new(&probe_dir);
    // Disk, not Worker: the holder's operation class is an APPEND to the checkpoint log, and
    // rung 3 must attempt the same class it is about to convict someone for failing at. A
    // Worker lease would also be refused here for the right reason — this crate has no thread
    // pool and its probe says so rather than passing on nothing.
    let lease = arena
        .lease(&mut admit, None, ResourceKind::Disk, 1)
        .unwrap_or_else(|e| {
            panic!(
                "the observer could not lease its own holder record: {}. It refuses to \
                 measure a reaper it could not admit a lease through.",
                e.message()
            )
        });

    let mut tallies = [Tally::default(); 5];
    let mut last_receipts = receipt_count(&log)
        .unwrap_or_else(|| panic!("cannot read the receipt stream at {log}"));
    let mut last_receipt_at = Instant::now();
    let mut own_step_max_s = 0.0f64;
    let mut own_step_intervals: Vec<f64> = Vec::new();
    let first_receipts = last_receipts;
    let mut last_cpu = proc_cpu_ticks(pid);
    let mut cpu_history: Vec<bool> = Vec::new();
    let mut rung3_failures: u64 = 0;
    let mut unreadable_polls: u64 = 0;
    let started_at = Instant::now();
    let mut polls: u64 = 0;

    while polls < max_polls {
        std::thread::sleep(Duration::from_millis(poll_ms));

        // The holder is alive iff the process still exists AND is not a zombie. This is the
        // GROUND TRUTH the would-have-reaps are scored against; without it a reap after the
        // run legitimately ended would be counted as a false positive.
        let Some(cpu_now) = proc_cpu_ticks(pid) else {
            println!("# holder {pid} is gone; the generation ended or was stopped. Stopping.");
            break;
        };
        let holder_alive = true;

        // An unreadable log is an unknown reading, and a poll with an unknown reading is
        // SKIPPED rather than scored: a policy judged on a reading the instrument did not take
        // would put the instrument's own gaps into the false-positive rate.
        let Some(now_receipts) = receipt_count(&log) else {
            unreadable_polls += 1;
            continue;
        };
        let receipt_arrived = now_receipts > last_receipts;
        if receipt_arrived {
            let interval = last_receipt_at.elapsed().as_secs_f64();
            own_step_intervals.push(interval);
            if interval > own_step_max_s {
                own_step_max_s = interval;
            }
            last_receipt_at = Instant::now();
            // The receipts ARE the rent (§9 Q1): a node solved is a work product, so it is
            // paid into the lease rather than being merely counted here.
            let paid = now_receipts - last_receipts;
            arena
                .pay_rent(lease, holon_resource::Receipt(paid))
                .expect("the observer's own lease refused rent");
            last_receipts = now_receipts;
        }

        let cpu_advanced_now = match (last_cpu, cpu_now) {
            (Some(a), b) => b > a,
            (None, _) => true,
        };
        last_cpu = Some(cpu_now);
        cpu_history.push(cpu_advanced_now);
        if cpu_history.len() > debounce {
            let excess = cpu_history.len() - debounce;
            cpu_history.drain(0..excess);
        }
        let cpu_advanced_debounced = cpu_history.iter().any(|b| *b);

        let reading = Reading {
            silence_s: last_receipt_at.elapsed().as_secs_f64(),
            own_step_max_s,
            cpu_advanced_now,
            cpu_advanced_debounced,
            holder_alive,
        };

        // Rung 3 is a REAL probe of the same operation class the holder is failing at: the
        // generation's work product is an append to a file, so the reaper attempts a write.
        let mut verdicts: [&'static str; 5] = ["", "", "", "", ""];
        for (slot, policy) in Policy::ALL.iter().enumerate() {
            let world = PolicyWorld {
                policy: *policy,
                reading,
                flat_grace_s,
                k,
            };
            // Rung 3 is absent for P1 by construction (that IS the configuration), and a probe
            // that always passes is how "absent" is expressed to a reaper that always has one.
            let mut reaper = Reaper::new(
                world,
                if matches!(policy, Policy::Rung2Absent | Policy::OwnStepAlone) {
                    Rung3::Absent
                } else {
                    Rung3::Attempt(AttemptProbe::new(&probe_dir))
                },
            );
            let v = reaper.judge(lease, ResourceKind::Disk);
            let t = &mut tallies[slot];
            t.polls += 1;
            match &v {
                ReapVerdict::Reap { .. } => {
                    t.would_reap += 1;
                    if reading.holder_alive {
                        t.false_reaps += 1;
                    }
                    verdicts[slot] = "REAP";
                }
                ReapVerdict::StandDown { .. } => {
                    t.stood_down += 1;
                    verdicts[slot] = "STAND_DOWN";
                }
                ReapVerdict::Keep { .. } => {
                    t.kept += 1;
                    verdicts[slot] = "KEEP";
                }
            }
            if !v.evidence().reaper_own_probe.passed() {
                rung3_failures += 1;
            }
        }

        polls += 1;
        writeln!(
            records,
            "{{\"poll\":{},\"t_s\":{:.3},\"receipts\":{},\"silence_s\":{:.3},\
             \"own_step_max_s\":{:.3},\"cpu_now\":{},\"cpu_deb\":{},\
             \"P1\":\"{}\",\"P2\":\"{}\",\"P3\":\"{}\",\"P4\":\"{}\",\"P5\":\"{}\"}}",
            polls,
            started_at.elapsed().as_secs_f64(),
            now_receipts,
            reading.silence_s,
            reading.own_step_max_s,
            cpu_advanced_now,
            cpu_advanced_debounced,
            verdicts[0],
            verdicts[1],
            verdicts[2],
            verdicts[3],
            verdicts[4]
        )
        .unwrap();
        if polls % 30 == 0 {
            records.flush().unwrap();
            let mut interim = String::new();
            summarise(
                &tallies,
                polls,
                now_receipts - first_receipts,
                &own_step_intervals,
                own_step_max_s,
                rung3_failures,
                flat_grace_s,
                k,
                &mut interim,
            );
            print!("{interim}");
            let _ = std::io::stdout().flush();
        }
    }

    records.flush().unwrap();

    // ---- THE REAPER STAYED OFF, and the books say so rather than the comments.
    let l = arena.ledger();
    assert_eq!(
        l.convicted, 0,
        "the observer convicted a lease. It is observe-only; this is a defect in the observer, \
         not a reading about the reaper."
    );
    assert_eq!(l.reaped, 0, "the observer reaped. Same: a defect in the observer.");
    assert!(
        arena.balances(),
        "the observer's own lease books do not balance, so nothing it reports about accounting \
         can be trusted"
    );

    let mut out = String::new();
    summarise(
        &tallies,
        polls,
        last_receipts - first_receipts,
        &own_step_intervals,
        own_step_max_s,
        rung3_failures,
        flat_grace_s,
        k,
        &mut out,
    );
    print!("{out}");
    std::fs::write(format!("{out_prefix}.summary.txt"), &out).unwrap();
    println!("# records  {records_path}");
    println!("# ledger   opened={} released={} convicted={} reaped={} rent={}",
             l.opened, l.released, l.convicted, l.reaped, l.rent.0);
    println!("# skipped  {unreadable_polls} polls on an unreadable receipt stream");
}

/// Rung 3, including the configuration that HAS no rung 3.
///
/// `Absent` is a named variant rather than a scripted always-pass so that reading the record
/// cannot mistake "not consulted" for "consulted and passed" — the verdict string it produces
/// says which it was.
enum Rung3 {
    Absent,
    Attempt(AttemptProbe),
}

impl Probe for Rung3 {
    fn probe(&mut self, kind: ResourceKind, amount: u64) -> ProbeVerdict {
        match self {
            Rung3::Absent => ProbeVerdict::Pass("rung 3 absent in this configuration"),
            Rung3::Attempt(p) => p.probe(kind, amount),
        }
    }
}

fn summarise<W: std::fmt::Write + ?Sized>(
    tallies: &[Tally; 5],
    polls: u64,
    receipts: u64,
    intervals: &[f64],
    own_step_max_s: f64,
    rung3_failures: u64,
    flat_grace_s: f64,
    k: f64,
    out: &mut W,
) {
    let mut sorted: Vec<f64> = intervals.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = if sorted.is_empty() {
        f64::NAN
    } else {
        sorted[sorted.len() / 2]
    };
    let _ = writeln!(
        out,
        "\n=== reap_observe: {polls} polls, {receipts} receipts (nodes solved) ===\n\
         holder's own step   median {median:.3} s, max {own_step_max_s:.3} s over {} intervals\n\
         flat grace          {flat_grace_s:.3} s   (ratio to the holder's own max step: {:.2}x)\n\
         P4's grace          {:.3} s   ({k} x the holder's own max step)\n\
         rung-3 probe fails  {rung3_failures}\n",
        sorted.len(),
        flat_grace_s / own_step_max_s.max(f64::MIN_POSITIVE),
        k * own_step_max_s
    );
    let _ = writeln!(
        out,
        "| policy | polls | would-reap | FALSE reaps | stood down | kept |"
    );
    let _ = writeln!(out, "|---|---:|---:|---:|---:|---:|");
    for (slot, p) in Policy::ALL.iter().enumerate() {
        let t = tallies[slot];
        let _ = writeln!(
            out,
            "| {} | {} | {} | **{}** | {} | {} |",
            p.label(),
            t.polls,
            t.would_reap,
            t.false_reaps,
            t.stood_down,
            t.kept
        );
    }
}

/// Lines in the append-only receipt stream. One line is one node solved.
///
/// `None` when the log cannot be read. That is an UNKNOWN, not zero receipts: returning zero
/// would manufacture a silence the policies would then act on, which is the same shape as a
/// control value asserted about the instrument's coverage rather than about the scene.
fn receipt_count(path: &str) -> Option<u64> {
    std::fs::read(path)
        .ok()
        .map(|b| b.iter().filter(|c| **c == b'\n').count() as u64)
}

/// utime + stime in clock ticks, or `None` if the process is gone.
fn proc_cpu_ticks(pid: i32) -> Option<u64> {
    let s = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // Field 14 (utime) and 15 (stime), 1-based, AFTER the comm field — which can itself
    // contain spaces and parentheses, so the split starts past the last ')'.
    let close = s.rfind(')')?;
    let rest: Vec<&str> = s[close + 1..].split_whitespace().collect();
    // rest[0] is state (field 3), so field 14 is rest[11] and field 15 is rest[12].
    let ut: u64 = rest.get(11)?.parse().ok()?;
    let st: u64 = rest.get(12)?.parse().ok()?;
    Some(ut + st)
}
