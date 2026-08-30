//! Where two fixture curves differ in sign, measured rather than assumed.
use holon_chem::elements::{HYDROGEN, LITHIUM, HELIUM};
use holon_chem::pair::generate_pair_table;
fn main() {
    let hh = generate_pair_table(HYDROGEN, HYDROGEN, 24);
    let hli = generate_pair_table(HYDROGEN, LITHIUM, 24);
    let hhe = generate_pair_table(HYDROGEN, HELIUM, 24);
    for (name, pt) in [("H-H", &hh), ("H-Li", &hli), ("H-He", &hhe)] {
        println!("{name}: r in [{:.4}, {:.4}]  well {:?}", pt.meta.r_min, pt.meta.r_max,
            pt.meta.well.map(|w| (w.r_e, w.d_e)));
    }
    println!("\n     R      u(H-H)        u(H-Li)       u(H-He)");
    let za = hh.meta.e_asymptote; let zb = hli.meta.e_asymptote; let zc = hhe.meta.e_asymptote;
    let mut r = 0.8;
    while r <= 6.01 {
        let a = holon_chem::pair::pair_point(HYDROGEN, HYDROGEN, r).e - za;
        let b = holon_chem::pair::pair_point(HYDROGEN, LITHIUM, r).e - zb;
        let c = holon_chem::pair::pair_point(HYDROGEN, HELIUM, r).e - zc;
        println!("{r:6.2}  {a:+.6e}  {b:+.6e}  {c:+.6e}");
        r += 0.2;
    }
}
