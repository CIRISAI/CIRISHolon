//! The sub-atom band's gates: the door IS the campaign's exact arm, the density it draws
//! conserves quark number, and every refusal fires by name rather than hanging the page.

use holon_render::hadron::*;

#[test]
fn the_band_is_the_exact_colour_lane_arm_to_the_bit() {
    for (n, x) in [(4usize, 4.0f64), (6, 4.0), (6, 9.0)] {
        for b in 0..=2 {
            let q = holon_chem::qcd2::Qcd2::new(n, x);
            let n_q = q.quarks(b);
            if n_q % 3 != 0 || n_q / 3 > n {
                continue;
            }
            let referee = q.ground_lanes(b, 1);
            let code = holon_hadron_solve(n as u32, x, b);
            // 0 converged, 3 trivial — a one-determinant sector (N=4, B=2 is C(4,4)^3 = 1)
            // has nothing to iterate, and reporting that honestly is the exit's job
            assert!(code == 0 || code == 3, "N={n} x={x} B={b}: the door failed (code {code})");
            let e = holon_hadron_energy(b);
            assert_eq!(
                e.to_bits(),
                referee.energy.to_bits(),
                "N={n} x={x} B={b}: door {e:.12} vs the exact arm {:.12}",
                referee.energy
            );
            assert!(holon_hadron_margin(b) >= 0.0, "N={n} B={b}: negative variational margin");
            assert_eq!(holon_hadron_n_det(b) as usize, q.lane_space(b).n_det);
            assert_eq!(holon_hadron_dim_for(n as u32, b), holon_hadron_n_det(b), "the priced dimension is the real one");
            // the density it draws conserves quark number
            let total: f64 = (0..n).map(|k| holon_hadron_occ(b, k as u32)).sum();
            assert!(
                (total - n_q as f64).abs() <= 1e-9,
                "N={n} B={b}: the drawn density sums to {total:.9}, not the sector's {n_q} quarks"
            );
            for k in 0..n {
                let o = holon_hadron_occ(b, k as u32);
                assert!((-1e-12..=3.0 + 1e-12).contains(&o), "N={n} B={b} site {k}: density {o} outside [0, 3]");
            }
            assert!(holon_hadron_occ(b, n as u32).is_nan(), "off the end of the chain must be NaN, not a number");
        }
        // the mass needs BOTH sectors from the SAME box, and says nothing otherwise
        holon_hadron_solve(n as u32, x, 0);
        holon_hadron_solve(n as u32, x, 1);
        let m = holon_hadron_baryon_mass();
        let q = holon_chem::qcd2::Qcd2::new(n, x);
        let want = holon_chem::qcd2::Qcd2::baryon_mass(q.ground_lanes(0, 1).energy, q.ground_lanes(1, 1).energy, x);
        assert!((m - want).abs() <= 1e-12, "N={n} x={x}: baryon mass {m:.12} vs {want:.12}");
        println!("N={n} x={x}: E0 {:.9}  E1 {:.9}  baryon mass {m:.9}  ({} and {} determinants)",
            holon_hadron_energy(0), holon_hadron_energy(1), holon_hadron_n_det(0), holon_hadron_n_det(1));
    }
}

#[test]
fn every_refusal_fires_by_name_and_changes_nothing() {
    // a good solve to have something to protect
    assert_eq!(holon_hadron_solve(4, 4.0, 1), 0);
    let good = holon_hadron_energy(1);
    // bad parameters: odd chain, too small, baryon number out of range, non-positive coupling
    for (n, x, b, why) in [(5u32, 4.0f64, 1i32, "odd chain"), (0, 4.0, 1, "empty chain"), (4, 4.0, 7, "no such sector"), (4, 0.0, 1, "zero coupling"), (4, -1.0, 1, "negative coupling")] {
        assert_eq!(holon_hadron_solve(n, x, b), 5, "{why} must be refused with code 5");
    }
    // too big: N=10 B=1 is 9.3 million determinants, far over the cap
    assert_eq!(holon_hadron_solve(10, 4.0, 1), 6, "a sector over the cap must be refused with code 6, not attempted");
    assert!(holon_hadron_dim_for(10, 1) > holon_hadron_max_det(), "the page can price the refusal before asking for it");
    // and none of that disturbed the good solve
    assert_eq!(holon_hadron_energy(1).to_bits(), good.to_bits(), "a refusal changed the stored band");
    // an unsolved sector reads as unsolved, never as zero
    assert_eq!(holon_hadron_exit(9), 4);
    assert!(holon_hadron_energy(9).is_nan(), "an out-of-range sector must read NaN, not 0.0");
    println!("refusals: bad parameters -> 5, over the {:.0}-determinant cap -> 6, unsolved -> NaN and exit 4", holon_hadron_max_det());
}
