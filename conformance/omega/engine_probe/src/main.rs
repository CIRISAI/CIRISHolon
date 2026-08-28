//! OMEGA-CIRCUITS-1 gate G2: read the staked circuits on the ENGINE's own
//! PackedTableau and report, per computational basis input and per qubit,
//! whether the outcome is determinate and what it is.  No sampling, no seed:
//! `measure_peek` is a pure read of the tableau (adaptive.rs's own
//! deterministic branch), so this is a universal reading, not a trajectory.
use holon::tableau::PackedTableau;

#[derive(Clone, Copy)]
enum G {
    H(usize),
    S(usize),
    X(usize),
    Z(usize),
    Cx(usize, usize),
    Cz(usize, usize),
}

fn apply(t: &mut PackedTableau, g: G) {
    match g {
        G::H(q) => t.h(q),
        G::S(q) => t.s(q),
        G::X(q) => t.x_gate(q),
        G::Z(q) => t.z_gate(q),
        G::Cx(c, x) => t.cx(c, x),
        G::Cz(c, x) => {
            t.h(x);
            t.cx(c, x);
            t.h(x);
        }
    }
}

fn circuits() -> Vec<(&'static str, usize, Vec<G>)> {
    vec![
        ("U_CX", 2, vec![G::Cx(0, 1)]),
        ("U_SWAP", 2, vec![G::Cx(0, 1), G::Cx(1, 0), G::Cx(0, 1)]),
        ("U_H0_2", 2, vec![G::H(0)]),
        ("U_GHZ", 3, vec![G::H(0), G::Cx(0, 1), G::Cx(1, 2)]),
        ("U_W", 3, vec![G::Cx(0, 1), G::Cx(0, 2)]),
        ("U_H01", 3, vec![G::H(0), G::H(1)]),
        ("U_H012", 3, vec![G::H(0), G::H(1), G::H(2)]),
        ("U_W_H3", 4, vec![G::Cx(0, 1), G::Cx(0, 2), G::H(3)]),
        (
            "U_TEL",
            3,
            vec![
                G::H(0),
                G::H(1),
                G::Cx(1, 2),
                G::Cx(0, 1),
                G::H(0),
                G::Cx(1, 2),
                G::Cz(0, 2),
            ],
        ),
        (
            "U_REP",
            5,
            vec![
                G::X(1),
                G::Cx(0, 3),
                G::Cx(1, 3),
                G::Cx(1, 4),
                G::Cx(2, 4),
                G::Cx(3, 1),
            ],
        ),
    ]
}

fn main() {
    let mut out = String::from("{");
    let cs = circuits();
    for (ci, (name, n, gates)) in cs.iter().enumerate() {
        if ci > 0 {
            out.push(',');
        }
        out.push_str(&format!("\"{name}\":["));
        for s in 0..(1usize << n) {
            if s > 0 {
                out.push(',');
            }
            let mut t = PackedTableau::new(*n);
            // prepare |s>: qubit q carries bit (n-1-q) of s, the freeze's
            // left-to-right convention.
            for q in 0..*n {
                if (s >> (n - 1 - q)) & 1 == 1 {
                    t.x_gate(q);
                }
            }
            for g in gates {
                apply(&mut t, *g);
            }
            out.push('[');
            for q in 0..*n {
                if q > 0 {
                    out.push(',');
                }
                let v = match t.measure_peek(q) {
                    None => -1i32,
                    Some(false) => 0,
                    Some(true) => 1,
                };
                out.push_str(&v.to_string());
            }
            out.push(']');
        }
        out.push(']');
    }
    out.push('}');
    println!("{out}");
}
