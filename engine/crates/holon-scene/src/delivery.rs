//! Who holds the message (MESH-0, `conformance/scene/MESH0_PREREG.md`): the first composition
//! of three shipped generators. For a population of households, each adult with a phone and a
//! day by the hour, it counts where a message goes: taken at once, held by the sender until
//! both phones are on, handed to a host at home, or left to someone else.
//!
//! Nothing here says a mesh works. It counts how much must be held, by whom, for how long.
//! Integers only. The law it inherits: one state per person per hour, so one outcome per message.

use crate::seed::{child_seed, Stream};
use crate::verdict::Verdict;
use crate::{devices, household, presence};

/// Hours looked ahead for a delivery: the rest of the day sent and all of the next.
pub const HORIZON: usize = 48;

/// One addressee: an adult holding a phone, with the hours the phone is on over two days.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Person {
    pub household: u32,
    /// A label for the breakdown by kind of sender and addressee (here: employed or not).
    pub kind: u8,
    pub on: [bool; HORIZON],
}

/// A population: addressees, and for each household whether it has a host that is on.
#[derive(Clone, Debug, Default)]
pub struct Population {
    pub persons: Vec<Person>,
    /// `members[h]` are indices into `persons`.
    pub members: Vec<Vec<u32>>,
}

/// What `simulate` counts. Every message has exactly one of `at_once`, or a wait.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Counts {
    pub messages: u64,
    /// Messages whose sender was of the addressee's household.
    pub from_household: u64,
    /// Addressees skipped because no sender could be drawn or the sender's phone is never on.
    pub no_sender: u64,
    pub at_once: u64,
    /// `at_once` and `messages` by (sender kind, addressee kind), kinds 0 and 1.
    pub at_once_by_kind: [[u64; 2]; 2],
    pub messages_by_kind: [[u64; 2]; 2],
    /// Of messages not taken at once: the hours until both phones are on (index `h`, 1-based
    /// in meaning: index 0 is unused), with index `HORIZON` for "not within the horizon".
    pub peer_wait: [u64; HORIZON + 1],
    /// The same with a host holding: hours until the addressee's phone is next on.
    pub host_wait: [u64; HORIZON + 1],
    /// Messages not taken at once whose household has no host: left to someone else.
    pub left_to_someone_else: u64,
    /// Those messages, by each clock hour (of 24) they are held in.
    pub held_elsewhere_by_hour: [u64; 24],
}

impl Counts {
    fn new() -> Counts {
        Counts {
            messages: 0, from_household: 0, no_sender: 0, at_once: 0, at_once_by_kind: [[0; 2]; 2], messages_by_kind: [[0; 2]; 2],
            peer_wait: [0; HORIZON + 1], host_wait: [0; HORIZON + 1], left_to_someone_else: 0, held_elsewhere_by_hour: [0; 24],
        }
    }
    /// Messages delivered within `t` hours of the hour sent when the sender holds.
    pub fn peer_within(&self, t: usize) -> u64 {
        self.at_once + self.peer_wait[1..=t.min(HORIZON - 1)].iter().sum::<u64>()
    }
    /// The same when a host at home holds (every household given one).
    pub fn host_within(&self, t: usize) -> u64 {
        self.at_once + self.host_wait[1..=t.min(HORIZON - 1)].iter().sum::<u64>()
    }
}

/// Send `per_addressee` messages to each addressee and count where each goes.
///
/// The sender is, with `tie_share_ppm` parts per million, another addressee of the same
/// household when there is one, otherwise an addressee of another household; the hour is
/// drawn evenly over the sender's on-hours of the first day. `host[h]` says whether
/// household `h` has a host that is on.
pub fn simulate(pop: &Population, host: &[bool], tie_share_ppm: u32, per_addressee: u32, seed: u64) -> Counts {
    assert_eq!(host.len(), pop.members.len(), "one host flag per household");
    let n = pop.persons.len() as u64;
    let mut c = Counts::new();
    for (a, addressee) in pop.persons.iter().enumerate() {
        let mut st = Stream::new(child_seed(seed, a as u64));
        for _ in 0..per_addressee {
            let house = &pop.members[addressee.household as usize];
            let inside = house.len() > 1 && st.below(1_000_000) < tie_share_ppm as u64;
            let s = if inside {
                let mut j = st.below(house.len() as u64 - 1) as usize;
                if house[j] as usize == a {
                    j = house.len() - 1;
                }
                house[j] as usize
            } else {
                if n as usize <= house.len() {
                    c.no_sender += 1;
                    continue;
                }
                loop {
                    let j = st.below(n) as usize;
                    if pop.persons[j].household != addressee.household {
                        break j;
                    }
                }
            };
            let sender = &pop.persons[s];
            let hours = sender.on[..24].iter().filter(|&&x| x).count() as u64;
            if hours == 0 {
                c.no_sender += 1;
                continue;
            }
            let mut pick = st.below(hours);
            let mut h = 0usize;
            for (i, &x) in sender.on[..24].iter().enumerate() {
                if x {
                    if pick == 0 {
                        h = i;
                        break;
                    }
                    pick -= 1;
                }
            }
            c.messages += 1;
            c.from_household += inside as u64;
            let (ks, ka) = ((sender.kind & 1) as usize, (addressee.kind & 1) as usize);
            c.messages_by_kind[ks][ka] += 1;
            if addressee.on[h] {
                c.at_once += 1;
                c.at_once_by_kind[ks][ka] += 1;
                continue;
            }
            let peer = (h + 1..HORIZON).find(|&x| addressee.on[x] && sender.on[x]);
            c.peer_wait[peer.map_or(HORIZON, |x| x - h)] += 1;
            let next = (h + 1..HORIZON).find(|&x| addressee.on[x]);
            c.host_wait[next.map_or(HORIZON, |x| x - h)] += 1;
            if !host[addressee.household as usize] {
                c.left_to_someone_else += 1;
                for x in h..next.unwrap_or(HORIZON) {
                    c.held_elsewhere_by_hour[x % 24] += 1;
                }
            }
        }
    }
    c
}

/// The share taken at once that the hourly on-shares alone give for an independent sender:
/// `Σ a(h)² / Σ a(h)`, in parts per million. (MESH-0's C1, here from a population's own hours.)
pub fn closed_form_ppm(pop: &Population) -> u64 {
    let n = pop.persons.len() as u128;
    if n == 0 {
        return 0;
    }
    let (mut num, mut den) = (0u128, 0u128);
    for h in 0..24 {
        let a = pop.persons.iter().filter(|p| p.on[h]).count() as u128;
        num += a * a;
        den += a;
    }
    if den == 0 {
        0
    } else {
        (num * 1_000_000 / (den * n)) as u64
    }
}

// ---------------------------------------------------------------------------------------------
// The United States population MESH-0 reads, from the shipped generators
// ---------------------------------------------------------------------------------------------

/// How the second day follows the first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Join {
    /// A day drawn afresh, same day type.
    Afresh,
    /// The same day again.
    Same,
}

/// What a generated household declares, beside its addressees.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Declared {
    pub fixed_line: bool,
    pub desktop: bool,
    pub laptop: bool,
    pub seniors: u8,
    /// Phones held by children, and adults with no phone: counted, not addressed.
    pub children_phones: u8,
    pub adults_without_phone: u8,
}

/// A generated population with what each household declares.
#[derive(Clone, Debug, Default)]
pub struct UsPopulation {
    pub pop: Population,
    pub declared: Vec<Declared>,
    /// Person-hours awake and person-hours in all, first day, before any phone rule.
    pub awake_hours: u64,
    pub person_hours: u64,
}

/// `households_per_class` households in each of the five density classes.
/// `off_ppm`: the share of awake person-hours in which the phone is off, seeded.
/// `shared_day`: members of a household all get the first member's day (a swing, not a claim).
pub fn us_population(seed: u64, households_per_class: u32, day_type: u8, join: Join, off_ppm: u32, shared_day: bool) -> UsPopulation {
    let mut out = UsPopulation::default();
    for class in 1..=5u8 {
        let mut made = 0u32;
        let mut block = 0u64;
        while made < households_per_class {
            // HOUSEGEN-1 draws the sizes of the households of a unit holding `people`
            let hs = household::generate(child_seed(seed, class as u64 * 1_000_003 + block), 2_500, class, household::Reference::UnitedStates2024, true);
            block += 1;
            for &size in &hs.sizes {
                if made == households_per_class {
                    break;
                }
                let hseed = child_seed(seed, (class as u64) << 40 | (made as u64));
                made += 1;
                let (d, _, _) = devices::generate(hseed, size as u32, devices::BUILDING_UNKNOWN, true);
                let h = out.pop.members.len() as u32;
                let mut members = Vec::new();
                let mut first_day: Option<[bool; HORIZON]> = None;
                let mut person = 0u64;
                let mut without = 0u8;
                for (group, (&count, &phones)) in [d.adults, d.seniors].iter().zip(&d.phones[..2]).enumerate() {
                    for i in 0..count {
                        let pseed = child_seed(hseed, 100 + person);
                        person += 1;
                        let mut st = Stream::new(pseed);
                        let employed = st.below(1_000_000) < presence::employed_ppm(if group == 0 { 1 } else { 2 }) as u64;
                        let mut on = [false; HORIZON];
                        for day in 0..2usize {
                            let dseed = if day == 1 && join == Join::Same { child_seed(pseed, 0) } else { child_seed(pseed, day as u64) };
                            let (dd, _) = presence::day(presence::Generator::Chain, dseed, employed, day_type, 0, false, true);
                            for (s, &state) in dd.states.iter().enumerate() {
                                on[day * 24 + s] = state != 0 && state != 4;
                            }
                        }
                        out.awake_hours += on[..24].iter().filter(|&&x| x).count() as u64;
                        out.person_hours += 24;
                        if shared_day {
                            on = *first_day.get_or_insert(on);
                        }
                        if off_ppm > 0 {
                            let mut os = Stream::new(child_seed(pseed, 7));
                            for x in on.iter_mut() {
                                if *x && os.below(1_000_000) < off_ppm as u64 {
                                    *x = false;
                                }
                            }
                        }
                        if i < phones {
                            members.push(out.pop.persons.len() as u32);
                            out.pop.persons.push(Person { household: h, kind: employed as u8, on });
                        } else {
                            without += 1;
                        }
                    }
                }
                out.pop.members.push(members);
                out.declared.push(Declared {
                    fixed_line: d.gateway_declared(), desktop: d.has_desktop(), laptop: d.counts[0] > 0, seniors: d.seniors,
                    children_phones: d.phones[2], adults_without_phone: without,
                });
            }
        }
    }
    out
}

/// The settings for the host that is on, which nobody knows for any household.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostSetting {
    None,
    /// A fixed line, or any laptop or desktop. Its complement is DEVICEGEN-1's certified event.
    AnyComputer,
    /// A fixed line or a desktop: everything DEVICEGEN-1 declares able to host.
    Declared,
    /// A seeded share of all households (the twin's 0.60), independent of everything.
    Twin { ppm: u32 },
    /// The same number of households, drawn among the declared ones.
    TwinInDeclared { ppm: u32 },
    /// As many hostless households as `AnyComputer` leaves, placed on households of seniors first.
    HostlessOld,
    All,
}

pub fn hosts(us: &UsPopulation, setting: HostSetting, seed: u64) -> Vec<bool> {
    let d = &us.declared;
    let any = |x: &Declared| x.fixed_line || x.desktop || x.laptop;
    let dec = |x: &Declared| x.fixed_line || x.desktop;
    match setting {
        HostSetting::None => vec![false; d.len()],
        HostSetting::All => vec![true; d.len()],
        HostSetting::AnyComputer => d.iter().map(any).collect(),
        HostSetting::Declared => d.iter().map(dec).collect(),
        HostSetting::Twin { ppm } => (0..d.len()).map(|i| Stream::new(child_seed(seed, i as u64)).below(1_000_000) < ppm as u64).collect(),
        HostSetting::TwinInDeclared { ppm } => {
            let want = (d.len() as u64 * ppm as u64 / 1_000_000) as usize;
            let mut idx: Vec<usize> = (0..d.len()).filter(|&i| dec(&d[i])).collect();
            let mut st = Stream::new(seed);
            for i in (1..idx.len()).rev() {
                idx.swap(i, st.below(i as u64 + 1) as usize);
            }
            let mut out = vec![false; d.len()];
            for &i in idx.iter().take(want) {
                out[i] = true;
            }
            out
        }
        HostSetting::HostlessOld => {
            let hostless = d.iter().filter(|x| !any(x)).count();
            let mut idx: Vec<usize> = (0..d.len()).collect();
            idx.sort_by_key(|&i| (core::cmp::Reverse(d[i].seniors), i));
            let mut out = vec![true; d.len()];
            for &i in idx.iter().take(hostless) {
                out[i] = false;
            }
            out
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The verdict of a composed count
// ---------------------------------------------------------------------------------------------

/// One input a composed count rests on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    pub name: &'static str,
    pub verdict: Verdict,
    /// The input is used for something it was not scored on.
    pub outside_its_scoring: bool,
    pub reason: &'static str,
}

fn rank(v: Verdict) -> u8 {
    match v {
        Verdict::Certified => 2,
        Verdict::Unresolved => 1,
        Verdict::Refused => 0,
    }
}

/// Certified ranks above unresolved above refused; the composition takes the lowest; an
/// input used outside what it was scored on enters as unresolved. Returns the verdict and
/// the inputs that set it.
pub fn compose(inputs: &[Input]) -> (Verdict, Vec<&'static str>) {
    let eff = |i: &Input| if i.outside_its_scoring && i.verdict == Verdict::Certified { Verdict::Unresolved } else { i.verdict };
    let low = inputs.iter().map(|i| rank(eff(i))).min().unwrap_or(1);
    let v = match low {
        2 => Verdict::Certified,
        1 => Verdict::Unresolved,
        _ => Verdict::Refused,
    };
    (v, inputs.iter().filter(|i| rank(eff(i)) == low).map(|i| i.name).collect())
}

/// What MESH-0's counts rest on, as the inputs stood on 2026-11-12.
pub fn mesh0_inputs() -> Vec<Input> {
    vec![
        Input { name: "household sizes (HOUSEGEN-1, United States)", verdict: Verdict::Unresolved, outside_its_scoring: false, reason: "read, not certified, in each density class" },
        Input { name: "nothing to host on (DEVICEGEN-1)", verdict: Verdict::Certified, outside_its_scoring: false, reason: "certified for 2020" },
        Input { name: "desktop and laptop counts (DEVICEGEN-1)", verdict: Verdict::Unresolved, outside_its_scoring: false, reason: "counts by household size unresolved" },
        Input { name: "a person's day (PRESENCEGEN-2, chain)", verdict: Verdict::Certified, outside_its_scoring: false, reason: "certified on spells away, far-apart hours and sleep" },
        Input { name: "two days of one person joined", verdict: Verdict::Certified, outside_its_scoring: true, reason: "the chain was not scored across the day's seam or on unbroken sleep" },
        Input { name: "members of a household drawn independently", verdict: Verdict::Certified, outside_its_scoring: true, reason: "the household joint is unresolved by design" },
        Input { name: "whether a host is on", verdict: Verdict::Unresolved, outside_its_scoring: false, reason: "no reference for any household" },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn planted(n_house: usize, per_house: usize, on: impl Fn(usize, usize) -> ([bool; HORIZON], u8)) -> Population {
        let mut pop = Population::default();
        for h in 0..n_house {
            let mut m = Vec::new();
            for i in 0..per_house {
                let (o, kind) = on(h, i);
                m.push(pop.persons.len() as u32);
                pop.persons.push(Person { household: h as u32, kind, on: o });
            }
            pop.members.push(m);
        }
        pop
    }

    fn random_hours(seed: u64, k: u64) -> [bool; HORIZON] {
        let mut st = Stream::new(seed);
        let mut o = [false; HORIZON];
        for day in 0..2 {
            let mut left = k;
            for h in 0..24u64 {
                if st.below(24 - h) < left {
                    o[day * 24 + h as usize] = true;
                    left -= 1;
                }
            }
        }
        o
    }

    fn block(start: usize, len: usize) -> [bool; HORIZON] {
        let mut o = [false; HORIZON];
        for x in 0..HORIZON {
            o[x] = (x % 24 + 24 - start) % 24 < len;
        }
        o
    }

    /// P-SUM: every message has exactly one outcome, under any host setting.
    #[test]
    fn outcomes_sum_to_messages() {
        let pop = planted(400, 2, |h, i| (random_hours((h * 2 + i) as u64, 15), (h % 2) as u8));
        for host in [vec![false; 400], vec![true; 400], (0..400).map(|h| h % 3 == 0).collect()] {
            let c = simulate(&pop, &host, 237_000, 4, 1);
            assert_eq!(c.messages + c.no_sender, 400 * 2 * 4);
            assert_eq!(c.at_once + c.peer_wait.iter().sum::<u64>(), c.messages);
            assert_eq!(c.at_once + c.host_wait.iter().sum::<u64>(), c.messages);
            assert_eq!(c.messages_by_kind.iter().flatten().sum::<u64>(), c.messages);
            assert!(c.left_to_someone_else <= c.messages - c.at_once);
        }
        let all = simulate(&pop, &vec![true; 400], 237_000, 4, 1);
        assert_eq!(all.left_to_someone_else, 0);
        // a household with no addressee adds nothing
        let mut more = pop.clone();
        more.members.push(Vec::new());
        assert_eq!(simulate(&more, &vec![true; 401], 237_000, 4, 1).messages, all.messages);
    }

    /// P-FLAT: people awake in a random 15 of 24 hours: measured is the closed form.
    #[test]
    fn a_flat_population_is_the_closed_form() {
        let pop = planted(6000, 1, |h, _| (random_hours(h as u64 + 99, 15), 0));
        let c = simulate(&pop, &vec![false; 6000], 0, 4, 2);
        let measured = c.at_once * 1_000_000 / c.messages;
        assert!((measured as i64 - closed_form_ppm(&pop) as i64).abs() < 10_000, "{measured} against {}", closed_form_ppm(&pop));
    }

    /// P-KIND: two kinds awake in disjoint halves, senders of the addressee's own kind.
    #[test]
    fn senders_like_their_addressee_beat_the_closed_form() {
        let pop = planted(3000, 2, |h, _| (block(if h % 2 == 0 { 0 } else { 12 }, 12), (h % 2) as u8));
        let c = simulate(&pop, &vec![false; 3000], 1_000_000, 4, 3);
        let measured = c.at_once * 1_000_000 / c.messages;
        assert!(measured > closed_form_ppm(&pop) + 200_000, "{measured} against {}", closed_form_ppm(&pop));
        assert_eq!(c.at_once_by_kind[0][1] + c.at_once_by_kind[1][0], 0);
    }

    /// P-WAIT: one block of 8 hours asleep, the same on both days: with a host no wait is over 8.
    #[test]
    fn a_host_never_waits_longer_than_the_sleep() {
        let pop = planted(2000, 1, |h, _| (block((h * 7) % 24, 16), 0));
        let c = simulate(&pop, &vec![true; 2000], 0, 4, 4);
        assert!(c.messages > c.at_once, "some messages are held");
        assert_eq!(c.host_wait[9..].iter().sum::<u64>(), 0);
        assert!(c.host_wait[1..=8].iter().sum::<u64>() > 0);
    }

    /// P-PEER: sender and addressee awake in disjoint halves: two phones alone deliver nothing.
    #[test]
    fn two_phones_never_on_together_deliver_nothing() {
        let pop = planted(1000, 2, |_, i| (block(i * 12, 12), i as u8));
        let c = simulate(&pop, &vec![true; 1000], 1_000_000, 4, 5);
        assert_eq!(c.peer_within(24), 0);
        assert_eq!(c.peer_wait[HORIZON], c.messages);
        // a host at home delivers all of them within 12 hours
        assert_eq!(c.host_within(12), c.messages);
    }

    /// P-HOUSE: with the tie share at one and two adults a household, each sender is of the household.
    #[test]
    fn the_tie_share_puts_senders_in_the_household() {
        let pop = planted(500, 2, |h, i| (random_hours((h * 2 + i) as u64, 15), 0));
        let c = simulate(&pop, &vec![false; 500], 1_000_000, 4, 6);
        assert_eq!(c.from_household, c.messages);
        assert_eq!(simulate(&pop, &vec![false; 500], 0, 4, 6).from_household, 0);
        // one-person households have nobody inside to send
        let solo = planted(500, 1, |h, _| (random_hours(h as u64, 15), 0));
        assert_eq!(simulate(&solo, &vec![false; 500], 1_000_000, 4, 6).from_household, 0);
    }

    /// P-JOIN and P-SEED, on the generated population.
    #[test]
    fn the_join_and_the_seed() {
        let a = us_population(11, 60, 0, Join::Afresh, 0, false);
        let s = us_population(11, 60, 0, Join::Same, 0, false);
        assert_eq!(a.pop.persons.len(), s.pop.persons.len());
        assert!(a.pop.persons.iter().zip(&s.pop.persons).all(|(x, y)| x.on[..24] == y.on[..24]), "the first day is the same");
        assert!(s.pop.persons.iter().all(|p| p.on[..24] == p.on[24..]), "the same day again");
        assert!(a.pop.persons.iter().any(|p| p.on[..24] != p.on[24..]), "a day afresh differs");
        let h = hosts(&a, HostSetting::All, 0);
        let (ca, cs) = (simulate(&a.pop, &h, 237_000, 4, 3), simulate(&s.pop, &hosts(&s, HostSetting::All, 0), 237_000, 4, 3));
        assert_eq!(ca.at_once, cs.at_once);
        assert_ne!(ca.host_wait, cs.host_wait);
        assert_eq!(simulate(&a.pop, &h, 237_000, 4, 3), ca);
        assert_ne!(simulate(&a.pop, &h, 237_000, 4, 4), ca);
        assert_ne!(us_population(12, 60, 0, Join::Afresh, 0, false).pop.persons, a.pop.persons);
        assert_eq!(a.pop.members.len(), 300);
    }

    /// The host settings keep their promises.
    #[test]
    fn host_settings() {
        let us = us_population(5, 400, 0, Join::Afresh, 0, false);
        let n = |s| hosts(&us, s, 9).iter().filter(|&&x| x).count();
        assert_eq!(n(HostSetting::None), 0);
        assert_eq!(n(HostSetting::All), 2000);
        assert!(n(HostSetting::Declared) <= n(HostSetting::AnyComputer));
        assert_eq!(n(HostSetting::HostlessOld), n(HostSetting::AnyComputer));
        assert_eq!(n(HostSetting::TwinInDeclared { ppm: 600_000 }), 1200);
        let (t, d) = (hosts(&us, HostSetting::TwinInDeclared { ppm: 600_000 }, 9), hosts(&us, HostSetting::Declared, 9));
        assert!(t.iter().zip(&d).all(|(&a, &b)| !a || b));
        let tw = n(HostSetting::Twin { ppm: 600_000 });
        assert!((1100..1300).contains(&tw), "{tw}");
        // a phone that is off some of the time is on less
        let off = us_population(5, 400, 0, Join::Afresh, 150_000, false);
        let hours = |u: &UsPopulation| u.pop.persons.iter().map(|p| p.on.iter().filter(|&&x| x).count()).sum::<usize>();
        assert!(hours(&off) * 100 < hours(&us) * 88 && hours(&off) * 100 > hours(&us) * 82);
        // members sharing one day
        let sh = us_population(5, 400, 0, Join::Afresh, 0, true);
        assert!(sh.pop.members.iter().all(|m| m.windows(2).all(|w| sh.pop.persons[w[0] as usize].on == sh.pop.persons[w[1] as usize].on)));
    }

    /// P-WEAK: the composed verdict and what it names.
    #[test]
    fn the_composed_verdict_takes_the_lowest_and_names_it() {
        let i = |name, verdict, outside| Input { name, verdict, outside_its_scoring: outside, reason: "" };
        assert_eq!(compose(&[i("a", Verdict::Certified, false), i("b", Verdict::Certified, false)]).0, Verdict::Certified);
        assert_eq!(compose(&[i("a", Verdict::Certified, false), i("b", Verdict::Unresolved, false), i("c", Verdict::Certified, false)]), (Verdict::Unresolved, vec!["b"]));
        assert_eq!(compose(&[i("a", Verdict::Unresolved, false), i("b", Verdict::Refused, false)]), (Verdict::Refused, vec!["b"]));
        assert_eq!(compose(&[i("a", Verdict::Certified, true), i("b", Verdict::Certified, false)]), (Verdict::Unresolved, vec!["a"]));
        let (v, names) = compose(&mesh0_inputs());
        assert_eq!(v, Verdict::Unresolved);
        assert!(names.len() >= 4 && names.contains(&"whether a host is on"));
    }
}
