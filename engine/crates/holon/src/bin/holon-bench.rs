//! The benchmark contract's instrument: measured ratios or it didn't happen.
use holon::tableau::PackedTableau;

fn main() {
    let n = 256;
    let depth = 5120;
    let mut seed = 99u64;
    let mut rand = move || {
        seed = seed.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = seed;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    };
    let gates: Vec<(u8, usize, usize)> = (0..depth)
        .map(|_| {
            let g = (rand() % 6) as u8;
            let q = (rand() % n as u64) as usize;
            let mut q2 = (rand() % n as u64) as usize;
            while q2 == q {
                q2 = (rand() % n as u64) as usize;
            }
            (g, q, q2)
        })
        .collect();
    let t0 = std::time::Instant::now();
    let mut t = PackedTableau::new(n);
    for &(g, q, q2) in &gates {
        match g {
            0 => t.x_gate(q),
            1 => t.z_gate(q),
            2 => t.h(q),
            3 => t.s(q),
            4 => t.sdg(q),
            _ => t.cx(q, q2),
        }
    }
    for q in 0..n {
        if t.measure_peek(q).is_none() {
            t.collapse(q, false);
        }
    }
    println!(
        "packed tableau n={n} depth={depth} + full measure: {:.4}s",
        t0.elapsed().as_secs_f64()
    );
}
