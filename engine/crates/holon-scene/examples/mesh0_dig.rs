//! MESH-0, a diagnosis after the read (2026-11-12): WHERE the waits come from, and who has no phone.
//! Exploratory; nothing here is a stake. `mesh0_dig [households_per_class] [seed]`.
//! It repeats `delivery::simulate`'s message loop so that the hour sent can be kept.
use holon_scene::delivery::{us_population, Join, HORIZON};
use holon_scene::seed::{child_seed, Stream};
use holon_scene::{devices, household};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let per_class: u32 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(20_000);
    let seed: u64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(20_261_112);
    let us = us_population(seed, per_class, 0, Join::Afresh, 0, false);
    let pop = &us.pop;
    let n = pop.persons.len() as u64;
    // by the slot sent (slot s is clock hour (4 + s) mod 24): messages, held, and of the held: summed wait, waits over 8 h
    let (mut sent, mut held, mut wait_sum, mut over8) = ([0u64; 24], [0u64; 24], [0u64; 24], [0u64; 24]);
    // what the mesh ADDS when nothing holds a message: (hours until both phones are on) minus (hours until the addressee's is on)
    let mut excess = [0u64; 6]; // 0, 1, 2-3, 4-8, over 8, never within the horizon
    let mut asleep_when_sent_then_awake_at = [0u64; 24]; // the slot in which a held message is finally read
    for (ai, addressee) in pop.persons.iter().enumerate() {
        let mut st = Stream::new(child_seed(child_seed(seed, 1), ai as u64));
        for _ in 0..4 {
            let house = &pop.members[addressee.household as usize];
            let inside = house.len() > 1 && st.below(1_000_000) < 237_000;
            let s = if inside {
                let mut j = st.below(house.len() as u64 - 1) as usize;
                if house[j] as usize == ai { j = house.len() - 1; }
                house[j] as usize
            } else {
                loop { let j = st.below(n) as usize; if pop.persons[j].household != addressee.household { break j; } }
            };
            let sender = &pop.persons[s];
            let hours = sender.on[..24].iter().filter(|&&x| x).count() as u64;
            if hours == 0 { continue; }
            let mut pick = st.below(hours);
            let mut h = 0usize;
            for (i, &x) in sender.on[..24].iter().enumerate() { if x { if pick == 0 { h = i; break; } pick -= 1; } }
            sent[h] += 1;
            if addressee.on[h] { continue; }
            held[h] += 1;
            let host = (h + 1..HORIZON).find(|&x| addressee.on[x]);
            let peer = (h + 1..HORIZON).find(|&x| addressee.on[x] && sender.on[x]);
            if let Some(x) = host {
                wait_sum[h] += (x - h) as u64;
                over8[h] += ((x - h) > 8) as u64;
                asleep_when_sent_then_awake_at[x % 24] += 1;
            }
            let e = match (host, peer) { (Some(a), Some(b)) => b - a, _ => usize::MAX };
            excess[match e { 0 => 0, 1 => 1, 2..=3 => 2, 4..=8 => 3, usize::MAX => 5, _ => 4 }] += 1;
        }
    }
    let arr = |v: &[u64]| format!("[{}]", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","));
    println!("{{\"kind\":\"by_slot_sent\",\"sent\":{},\"held\":{},\"wait_sum\":{},\"over8\":{},\"read_at_slot\":{}}}", arr(&sent), arr(&held), arr(&wait_sum), arr(&over8), arr(&asleep_when_sent_then_awake_at));
    println!("{{\"kind\":\"excess_when_nothing_holds\",\"bins\":[\"0\",\"1\",\"2-3\",\"4-8\",\"over 8\",\"never in 48 h\"],\"counts\":{}}}", arr(&excess));
    // who has no phone: adults (18 and over) by the number of adults in the household, from the device generator as shipped
    let mut adults_by = [0u64; 7];
    let mut without_by = [0u64; 7];
    let mut hh_by = [0u64; 7];
    let mut hh_short = [0u64; 7];
    let (mut phones_adults, mut phones_children, mut adults_total, mut seniors_alone, mut seniors_alone_without) = (0u64, 0u64, 0u64, 0u64, 0u64);
    // by household size (1..=7, 7 = 7 or more), as the device reference counts: households, those with no smartphone at all, smartphones
    let (mut size_hh, mut size_none, mut size_phones) = ([0u64; 8], [0u64; 8], [0u64; 8]);
    for class in 1..=5u8 {
        let mut made = 0u32;
        let mut block = 0u64;
        while made < per_class {
            let hs = household::generate(child_seed(seed, class as u64 * 1_000_003 + block), 2_500, class, household::Reference::UnitedStates2024, true);
            block += 1;
            for &size in &hs.sizes {
                if made == per_class { break; }
                let hseed = child_seed(seed, (class as u64) << 40 | (made as u64));
                made += 1;
                let (d, _, _) = devices::generate(hseed, size as u32, devices::BUILDING_UNKNOWN, true);
                let sz = (size as usize).min(7);
                let all = d.phones[0] as u64 + d.phones[1] as u64 + d.phones[2] as u64;
                size_hh[sz] += 1; size_none[sz] += (all == 0) as u64; size_phones[sz] += all;
                let ad = (d.adults + d.seniors) as usize;
                let b = ad.min(6);
                let have = (d.adults.min(d.phones[0]) + d.seniors.min(d.phones[1])) as u64;
                adults_by[b] += ad as u64;
                without_by[b] += ad as u64 - have;
                hh_by[b] += 1;
                hh_short[b] += ((d.phones[0] + d.phones[1]) < (d.adults + d.seniors)) as u64;
                phones_adults += (d.phones[0] + d.phones[1]) as u64;
                phones_children += d.phones[2] as u64;
                adults_total += ad as u64;
                if d.adults == 0 && d.seniors == 1 && d.children == 0 { seniors_alone += 1; seniors_alone_without += (d.phones[1] == 0) as u64; }
            }
        }
    }
    println!("{{\"kind\":\"smartphones_by_household_size\",\"households\":{},\"with_none\":{},\"smartphones\":{}}}", arr(&size_hh), arr(&size_none), arr(&size_phones));
    println!("{{\"kind\":\"phones\",\"adults_by_adults_in_household\":{},\"adults_without_by\":{},\"households_by\":{},\"households_with_fewer_phones_than_adults\":{},\"phones_held_by_adults\":{},\"phones_held_by_children\":{},\"adults\":{},\"seniors_living_alone\":{},\"of_them_without\":{}}}",
        arr(&adults_by), arr(&without_by), arr(&hh_by), arr(&hh_short), phones_adults, phones_children, adults_total, seniors_alone, seniors_alone_without);
}
