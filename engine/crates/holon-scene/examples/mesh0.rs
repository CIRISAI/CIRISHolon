//! MESH-0's carrier: the generated population and every count the freeze names, one JSON
//! object per line on stdout. `mesh0 [households_per_class] [seed]` (defaults 20000, 20261112).
use holon_scene::delivery::{closed_form_ppm, compose, hosts, mesh0_inputs, simulate, us_population, Counts, HostSetting, Join, UsPopulation};
use holon_scene::seed::{child_seed, mix};

fn arr(v: &[u64]) -> String {
    format!("[{}]", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
}

fn counts(c: &Counts) -> String {
    format!(
        "\"messages\":{},\"from_household\":{},\"no_sender\":{},\"at_once\":{},\"at_once_by_kind\":[{},{}],\"messages_by_kind\":[{},{}],\"peer_wait\":{},\"host_wait\":{},\"left\":{},\"held_by_hour\":{}",
        c.messages, c.from_household, c.no_sender, c.at_once, arr(&c.at_once_by_kind[0]), arr(&c.at_once_by_kind[1]), arr(&c.messages_by_kind[0]), arr(&c.messages_by_kind[1]),
        arr(&c.peer_wait), arr(&c.host_wait), c.left_to_someone_else, arr(&c.held_elsewhere_by_hour)
    )
}

const SETTINGS: [(&str, HostSetting); 7] = [
    ("NONE", HostSetting::None), ("ANY-COMPUTER", HostSetting::AnyComputer), ("DECLARED", HostSetting::Declared), ("TWIN", HostSetting::Twin { ppm: 600_000 }),
    ("TWIN-IN-DECLARED", HostSetting::TwinInDeclared { ppm: 600_000 }), ("HOSTLESS-OLD", HostSetting::HostlessOld), ("ALL", HostSetting::All),
];

fn describe(us: &UsPopulation, label: &str) {
    let d = &us.declared;
    let n = d.len() as u64;
    let c = |f: &dyn Fn(&holon_scene::delivery::Declared) -> bool| d.iter().filter(|x| f(x)).count() as u64;
    let hourly: Vec<u64> = (0..24).map(|h| us.pop.persons.iter().filter(|p| p.on[h]).count() as u64).collect();
    // unbroken sleep: runs of hours off, over the two days joined, in whole hours
    let mut runs = vec![0u64; 49];
    for p in &us.pop.persons {
        let mut len = 0usize;
        for x in 0..48 {
            if !p.on[x] {
                len += 1;
            } else if len > 0 {
                runs[len] += 1;
                len = 0;
            }
        }
    }
    let in_hostless = us.pop.persons.iter().filter(|p| { let x = &d[p.household as usize]; !(x.fixed_line || x.desktop || x.laptop) }).count();
    let twin = hosts(us, HostSetting::Twin { ppm: 600_000 }, 77);
    let twin_on_undeclared = twin.iter().zip(d).filter(|(&t, x)| t && !(x.fixed_line || x.desktop)).count();
    println!(
        "{{\"kind\":\"population\",\"label\":\"{label}\",\"households\":{n},\"addressees\":{},\"awake_hours\":{},\"person_hours\":{},\"closed_form_ppm\":{},\"hourly_on\":{},\"desktop\":{},\"fixed_line\":{},\"desktop_or_line\":{},\"nothing_to_host_on\":{},\"addressees_in_hostless\":{},\"twin_hosts\":{},\"twin_hosts_on_undeclared\":{},\"children_phones\":{},\"adults_without_phone\":{},\"off_runs\":{}}}",
        us.pop.persons.len(), us.awake_hours, us.person_hours, closed_form_ppm(&us.pop), arr(&hourly), c(&|x| x.desktop), c(&|x| x.fixed_line), c(&|x| x.desktop || x.fixed_line),
        c(&|x| !(x.fixed_line || x.desktop || x.laptop)), in_hostless, twin.iter().filter(|&&x| x).count(), twin_on_undeclared,
        d.iter().map(|x| x.children_phones as u64).sum::<u64>(), d.iter().map(|x| x.adults_without_phone as u64).sum::<u64>(), arr(&runs)
    );
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let per_class: u32 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(20_000);
    let seed: u64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(20_261_112);
    let (v, names) = compose(&mesh0_inputs());
    println!("{{\"kind\":\"verdict\",\"verdict\":\"{v:?}\",\"set_by\":{:?},\"inputs\":[{}]}}", names, mesh0_inputs().iter().map(|i| format!("{{\"name\":\"{}\",\"verdict\":\"{:?}\",\"outside_its_scoring\":{},\"reason\":\"{}\"}}", i.name, i.verdict, i.outside_its_scoring, i.reason)).collect::<Vec<_>>().join(","));
    let mut digest = 0u64;
    for (day_type, dname) in [(0u8, "Mon-Thu"), (2u8, "Saturday")] {
        for (join, jname) in [(Join::Afresh, "afresh"), (Join::Same, "same")] {
            for off in [0u32, 150_000] {
                for shared in [false, true] {
                    if shared && !(day_type == 0 && join == Join::Afresh && off == 0) {
                        continue;
                    }
                    let us = us_population(seed, per_class, day_type, join, off, shared);
                    let label = format!("{dname}|{jname}|off{off}|{}", if shared { "shared-day" } else { "independent" });
                    describe(&us, &label);
                    for t in [237_000u32, 290_000, 520_000] {
                        for (sname, s) in SETTINGS {
                            let c = simulate(&us.pop, &hosts(&us, s, child_seed(seed, 77)), t, 4, child_seed(seed, 1));
                            digest = mix(digest ^ c.at_once ^ (c.left_to_someone_else << 20) ^ c.peer_wait[1] ^ (c.host_wait[2] << 7));
                            println!("{{\"kind\":\"run\",\"label\":\"{label}\",\"t_ppm\":{t},\"host\":\"{sname}\",{}}}", counts(&c));
                        }
                    }
                }
            }
        }
    }
    // the product reading over 20 seeds (ANY-COMPUTER, Monday to Thursday, afresh, phone on when awake)
    for k in 0..20u64 {
        let s = child_seed(seed, 1000 + k);
        let us = us_population(s, per_class, 0, Join::Afresh, 0, false);
        let c = simulate(&us.pop, &hosts(&us, HostSetting::AnyComputer, 0), 237_000, 4, s);
        let d = &us.declared;
        let hostless = us.pop.persons.iter().filter(|p| { let x = &d[p.household as usize]; !(x.fixed_line || x.desktop || x.laptop) }).count();
        println!("{{\"kind\":\"seed\",\"k\":{k},\"messages\":{},\"at_once\":{},\"left\":{},\"addressees\":{},\"addressees_in_hostless\":{}}}", c.messages, c.at_once, c.left_to_someone_else, us.pop.persons.len(), hostless);
    }
    println!("{{\"kind\":\"digest\",\"value\":\"{digest}\"}}");
}
