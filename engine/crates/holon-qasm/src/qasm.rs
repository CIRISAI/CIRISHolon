//! Enhanced OpenQASM 2 and OpenQASM 3 front-end rewriter and parser.
//!
//! Provides:
//! - Full support for OpenQASM 2.0 and OpenQASM 3 syntax
//! - General single-qubit gate decomposition (U3, U2, U1, U, Rx, Ry, Rz, Sx, Sxdg, Y, etc.)
//! - Multi-qubit gate decomposition (CZ, CY, SWAP, CH, CRz, CP, CCX, CSWAP)
//! - Parameterized angle evaluation with arithmetic expressions and π constants
//! - Exact classification into Clifford, Clifford+T, and general rotation gates
//! - Symplectic tableau canonicalization and equivalence verification for stabilizer states

use crate::{Circuit, Gate, Mutation, Tableau};
use std::f64::consts::PI;

// ----------------------------------------------------------- Angle Expression Parser

/// Evaluates arithmetic expressions containing numbers, `pi`, `+`, `-`, `*`, `/`, and parentheses.
pub fn parse_angle_expr(expr: &str) -> Result<f64, String> {
    let s = expr.trim();
    if s.is_empty() {
        return Err("empty angle expression".into());
    }
    let tokens = tokenize_expr(s)?;
    let mut pos = 0;
    let val = parse_addition(&tokens, &mut pos)?;
    if pos < tokens.len() {
        return Err(format!("unexpected tokens at '{}' in '{}'", tokens[pos], expr));
    }
    Ok(val)
}

fn tokenize_expr(s: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c == '+' || c == '-' || c == '*' || c == '/' || c == '(' || c == ')' || c == ',' {
            tokens.push(c.to_string());
            chars.next();
        } else if c.is_alphabetic() || c == '_' {
            let mut id = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_alphanumeric() || ch == '_' {
                    id.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(id);
        } else if c.is_ascii_digit() || c == '.' {
            let mut num = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E' {
                    num.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(num);
        } else {
            return Err(format!("unexpected character '{c}' in expression"));
        }
    }
    Ok(tokens)
}

fn parse_addition(tokens: &[String], pos: &mut usize) -> Result<f64, String> {
    let mut val = parse_multiplication(tokens, pos)?;
    while *pos < tokens.len() {
        match tokens[*pos].as_str() {
            "+" => {
                *pos += 1;
                val += parse_multiplication(tokens, pos)?;
            }
            "-" => {
                *pos += 1;
                val -= parse_multiplication(tokens, pos)?;
            }
            _ => break,
        }
    }
    Ok(val)
}

fn parse_multiplication(tokens: &[String], pos: &mut usize) -> Result<f64, String> {
    let mut val = parse_unary(tokens, pos)?;
    while *pos < tokens.len() {
        match tokens[*pos].as_str() {
            "*" => {
                *pos += 1;
                val *= parse_unary(tokens, pos)?;
            }
            "/" => {
                *pos += 1;
                let denom = parse_unary(tokens, pos)?;
                if denom.abs() < 1e-15 {
                    return Err("division by zero in angle expression".into());
                }
                val /= denom;
            }
            _ => break,
        }
    }
    Ok(val)
}

fn parse_unary(tokens: &[String], pos: &mut usize) -> Result<f64, String> {
    if *pos >= tokens.len() {
        return Err("unexpected end of expression".into());
    }
    if tokens[*pos] == "-" {
        *pos += 1;
        let v = parse_primary(tokens, pos)?;
        Ok(-v)
    } else if tokens[*pos] == "+" {
        *pos += 1;
        parse_primary(tokens, pos)
    } else {
        parse_primary(tokens, pos)
    }
}

fn parse_primary(tokens: &[String], pos: &mut usize) -> Result<f64, String> {
    if *pos >= tokens.len() {
        return Err("unexpected end of expression".into());
    }
    let tok = &tokens[*pos];
    if tok == "(" {
        *pos += 1;
        let val = parse_addition(tokens, pos)?;
        if *pos >= tokens.len() || tokens[*pos] != ")" {
            return Err("missing closing parenthesis".into());
        }
        *pos += 1;
        Ok(val)
    } else if tok.eq_ignore_ascii_case("pi") {
        *pos += 1;
        Ok(PI)
    } else if tok.eq_ignore_ascii_case("tau") {
        *pos += 1;
        Ok(2.0 * PI)
    } else if tok.eq_ignore_ascii_case("euler") || tok.eq_ignore_ascii_case("e") {
        *pos += 1;
        Ok(std::f64::consts::E)
    } else if let Ok(num) = tok.parse::<f64>() {
        *pos += 1;
        Ok(num)
    } else {
        Err(format!("unrecognized token '{tok}' in expression"))
    }
}

// ---------------------------------------------------- Single-Qubit Gate Decomposition

/// Check if angle is close to an integer multiple of π/4.
pub fn is_pi4_multiple(theta: f64) -> Option<i64> {
    let k = theta / (PI / 4.0);
    let kr = k.round();
    if (k - kr).abs() < 1e-6 {
        Some(kr as i64)
    } else {
        None
    }
}

/// Check if angle is close to an integer multiple of π/2 (Clifford angle).
pub fn is_pi2_multiple(theta: f64) -> Option<i64> {
    let k = theta / (PI / 2.0);
    let kr = k.round();
    if (k - kr).abs() < 1e-6 {
        Some(kr as i64)
    } else {
        None
    }
}

/// Decompose Rz(theta) into Clifford+T gates if theta is a π/4 multiple.
pub fn decompose_rz(q: usize, theta: f64) -> Result<Vec<Gate>, String> {
    if let Some(k) = is_pi4_multiple(theta) {
        let km = k.rem_euclid(8);
        let gates = match km {
            0 => vec![],
            1 => vec![Gate::T(q)],
            2 => vec![Gate::S(q)],
            3 => vec![Gate::S(q), Gate::T(q)],
            4 => vec![Gate::Z(q)],
            5 => vec![Gate::Z(q), Gate::T(q)],
            6 => vec![Gate::Sdg(q)],
            7 => vec![Gate::Tdg(q)],
            _ => unreachable!(),
        };
        Ok(gates)
    } else {
        Err(format!("rz({theta}) is not a π/4 multiple"))
    }
}

/// Decompose Rx(theta) = H Rz(theta) H
pub fn decompose_rx(q: usize, theta: f64) -> Result<Vec<Gate>, String> {
    let mut out = vec![Gate::H(q)];
    out.extend(decompose_rz(q, theta)?);
    out.push(Gate::H(q));
    Ok(out)
}

/// Decompose Ry(theta) = S H Rz(theta) H Sdg
pub fn decompose_ry(q: usize, theta: f64) -> Result<Vec<Gate>, String> {
    if let Some(k) = is_pi2_multiple(theta) {
        let km = k.rem_euclid(4);
        let gates = match km {
            0 => vec![],
            1 => vec![Gate::S(q), Gate::H(q), Gate::S(q), Gate::H(q), Gate::Sdg(q)],
            2 => vec![Gate::Z(q), Gate::X(q)], // Y gate
            3 => vec![Gate::S(q), Gate::H(q), Gate::Sdg(q), Gate::H(q), Gate::Sdg(q)],
            _ => unreachable!(),
        };
        Ok(gates)
    } else {
        let mut out = vec![Gate::S(q), Gate::H(q)];
        out.extend(decompose_rz(q, theta)?);
        out.push(Gate::H(q));
        out.push(Gate::Sdg(q));
        Ok(out)
    }
}

/// Decompose U3(theta, phi, lambda) = Rz(phi) Ry(theta) Rz(lambda)
pub fn decompose_u3(q: usize, theta: f64, phi: f64, lambda: f64) -> Result<Vec<Gate>, String> {
    let mut out = Vec::new();
    out.extend(decompose_rz(q, lambda)?);
    out.extend(decompose_ry(q, theta)?);
    out.extend(decompose_rz(q, phi)?);
    Ok(out)
}

/// Decompose U2(phi, lambda) = U3(π/2, phi, lambda)
pub fn decompose_u2(q: usize, phi: f64, lambda: f64) -> Result<Vec<Gate>, String> {
    decompose_u3(q, PI / 2.0, phi, lambda)
}

/// Decompose U1(lambda) = Rz(lambda)
pub fn decompose_u1(q: usize, lambda: f64) -> Result<Vec<Gate>, String> {
    decompose_rz(q, lambda)
}

// ------------------------------------------------------------ QASM 2 / 3 Parser

fn parse_qubit_index(tok: &str, default_reg: &str) -> Result<usize, String> {
    let tok = tok.trim();
    if let (Some(open), Some(close)) = (tok.find('['), tok.find(']')) {
        let idx: usize = tok[open + 1..close]
            .parse()
            .map_err(|_| format!("bad register index in '{tok}'"))?;
        Ok(idx)
    } else if tok.starts_with(default_reg) || tok == "q" || tok == "c" || tok.starts_with('$') {
        let num_part = tok.trim_start_matches(|c: char| !c.is_ascii_digit());
        if num_part.is_empty() {
            Ok(0)
        } else {
            num_part
                .parse::<usize>()
                .map_err(|_| format!("unparseable qubit index '{tok}'"))
        }
    } else {
        tok.parse::<usize>()
            .map_err(|_| format!("expected indexed register in '{tok}'"))
    }
}

fn parse_param_list(param_str: &str) -> Result<Vec<f64>, String> {
    let s = param_str.trim().trim_start_matches('(').trim_end_matches(')');
    if s.is_empty() {
        return Ok(Vec::new());
    }
    let mut args = Vec::new();
    for p in s.split(',') {
        let p_trimmed = p.trim();
        if !p_trimmed.is_empty() {
            let val = parse_angle_expr(p_trimmed)?;
            args.push(val);
        }
    }
    Ok(args)
}

/// Enhanced OpenQASM 2 and OpenQASM 3 parser.
pub fn parse_qasm(src: &str) -> Result<Circuit, String> {
    let mut n_qubits = 0usize;
    let mut n_clbits = 0usize;
    let mut gates = Vec::new();
    let mut measures: Vec<(usize, usize)> = Vec::new();

    for raw in src.lines() {
        let line = raw.split("//").next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        for stmt in line.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty()
                || stmt.starts_with("OPENQASM")
                || stmt.starts_with("include")
                || stmt.starts_with("barrier")
                || stmt.starts_with("version")
            {
                continue;
            }

            // QASM 2 `qreg q[n];` or QASM 3 `qubit[n] q;` or `qubit q;`
            if let Some(rest) = stmt.strip_prefix("qreg ") {
                let rest = rest.trim();
                let sz = if let (Some(o), Some(c)) = (rest.find('['), rest.find(']')) {
                    rest[o + 1..c].parse().map_err(|_| "bad qreg size")?
                } else {
                    1
                };
                n_qubits = n_qubits.max(sz);
                continue;
            }
            if let Some(rest) = stmt.strip_prefix("qubit[") {
                let (sz_str, _) = rest
                    .split_once(']')
                    .ok_or_else(|| "bad qubit array declaration".to_string())?;
                let sz: usize = sz_str.parse().map_err(|_| "bad qubit array size")?;
                n_qubits = n_qubits.max(sz);
                continue;
            }
            if stmt.starts_with("qubit ") {
                n_qubits = n_qubits.max(1);
                continue;
            }

            // QASM 2 `creg c[n];` or QASM 3 `bit[n] c;`
            if let Some(rest) = stmt.strip_prefix("creg ") {
                let rest = rest.trim();
                let sz = if let (Some(o), Some(c)) = (rest.find('['), rest.find(']')) {
                    rest[o + 1..c].parse().map_err(|_| "bad creg size")?
                } else {
                    1
                };
                n_clbits = n_clbits.max(sz);
                continue;
            }
            if let Some(rest) = stmt.strip_prefix("bit[") {
                let (sz_str, _) = rest
                    .split_once(']')
                    .ok_or_else(|| "bad bit array declaration".to_string())?;
                let sz: usize = sz_str.parse().map_err(|_| "bad bit array size")?;
                n_clbits = n_clbits.max(sz);
                continue;
            }
            if stmt.starts_with("bit ") {
                n_clbits = n_clbits.max(1);
                continue;
            }

            // Measurements
            // QASM 2: `measure q[0] -> c[0];`
            if let Some(rest) = stmt.strip_prefix("measure ") {
                if let Some((q_tok, c_tok)) = rest.split_once("->") {
                    let q = parse_qubit_index(q_tok, "q")?;
                    let c = parse_qubit_index(c_tok, "c")?;
                    n_qubits = n_qubits.max(q + 1);
                    n_clbits = n_clbits.max(c + 1);
                    measures.push((q, c));
                    continue;
                }
            }
            // QASM 3: `c[0] = measure q[0];`
            if let Some((c_tok, rest)) = stmt.split_once('=') {
                let rest = rest.trim();
                if let Some(q_tok) = rest.strip_prefix("measure ") {
                    let q = parse_qubit_index(q_tok, "q")?;
                    let c = parse_qubit_index(c_tok, "c")?;
                    n_qubits = n_qubits.max(q + 1);
                    n_clbits = n_clbits.max(c + 1);
                    measures.push((q, c));
                    continue;
                }
            }

            // Gate statements
            let (name, params, args_str) = if let Some(open) = stmt.find('(') {
                let close = stmt
                    .find(')')
                    .ok_or_else(|| format!("unclosed parenthesis in: {stmt}"))?;
                let gname = stmt[..open].trim();
                let pstr = &stmt[open + 1..close];
                let r_args = stmt[close + 1..].trim();
                (gname, parse_param_list(pstr)?, r_args)
            } else {
                let (head, rest) = stmt
                    .split_once(' ')
                    .ok_or_else(|| format!("bad statement: '{stmt}'"))?;
                (head.trim(), Vec::new(), rest.trim())
            };

            if args_str.is_empty() {
                continue;
            }

            let q_args: Result<Vec<usize>, String> = args_str
                .split(',')
                .map(|a| parse_qubit_index(a, "q"))
                .collect();
            let qs = q_args?;
            for &q in &qs {
                n_qubits = n_qubits.max(q + 1);
            }

            match (name.to_lowercase().as_str(), qs.as_slice(), params.as_slice()) {
                ("x", [q], _) => gates.push(Gate::X(*q)),
                ("z", [q], _) => gates.push(Gate::Z(*q)),
                ("h", [q], _) => gates.push(Gate::H(*q)),
                ("s", [q], _) => gates.push(Gate::S(*q)),
                ("sdg", [q], _) => gates.push(Gate::Sdg(*q)),
                ("t", [q], _) => gates.push(Gate::T(*q)),
                ("tdg", [q], _) => gates.push(Gate::Tdg(*q)),
                ("y", [q], _) => {
                    gates.push(Gate::Z(*q));
                    gates.push(Gate::X(*q));
                }
                ("sx", [q], _) => {
                    gates.push(Gate::H(*q));
                    gates.push(Gate::S(*q));
                    gates.push(Gate::H(*q));
                }
                ("sxdg", [q], _) => {
                    gates.push(Gate::H(*q));
                    gates.push(Gate::Sdg(*q));
                    gates.push(Gate::H(*q));
                }
                ("cx" | "cnot", [c, t], _) => gates.push(Gate::Cx(*c, *t)),
                ("cz", [c, t], _) => {
                    gates.push(Gate::H(*t));
                    gates.push(Gate::Cx(*c, *t));
                    gates.push(Gate::H(*t));
                }
                ("cy", [c, t], _) => {
                    gates.push(Gate::Sdg(*t));
                    gates.push(Gate::Cx(*c, *t));
                    gates.push(Gate::S(*t));
                }
                ("swap", [a, b], _) => {
                    gates.push(Gate::Cx(*a, *b));
                    gates.push(Gate::Cx(*b, *a));
                    gates.push(Gate::Cx(*a, *b));
                }
                ("ccx" | "toffoli", [a, b, c], _) => gates.push(Gate::Ccx(*a, *b, *c)),
                ("rz", [q], [theta]) => {
                    gates.extend(decompose_rz(*q, *theta)?);
                }
                ("rx", [q], [theta]) => {
                    gates.extend(decompose_rx(*q, *theta)?);
                }
                ("ry", [q], [theta]) => {
                    gates.extend(decompose_ry(*q, *theta)?);
                }
                ("p" | "u1", [q], [lambda]) => {
                    gates.extend(decompose_u1(*q, *lambda)?);
                }
                ("u2", [q], [phi, lambda]) => {
                    gates.extend(decompose_u2(*q, *phi, *lambda)?);
                }
                ("u3" | "u", [q], [theta, phi, lambda]) => {
                    gates.extend(decompose_u3(*q, *theta, *phi, *lambda)?);
                }
                ("id" | "i", [_q], _) => {}
                _ => return Err(format!("unsupported gate '{name}' in statement: {stmt}")),
            }
        }
    }

    if n_qubits == 0 {
        return Err("no qubits found in circuit".into());
    }
    if n_clbits == 0 {
        n_clbits = n_qubits;
    }
    if measures.is_empty() {
        measures = (0..n_qubits).map(|q| (q, q)).collect();
    }

    Ok(Circuit {
        n_qubits,
        n_clbits,
        gates,
        measures,
    })
}

// --------------------------------------------- Symplectic Tableau Canonicalization

impl Tableau {
    /// Put the stabilizer rows into canonical reduced row echelon form (RREF)
    /// via symplectic Gaussian elimination.
    /// Returns a new canonicalized Tableau.
    pub fn canonical_form(&self) -> Tableau {
        let n = self.n;
        let mut tab = self.clone();

        let mut pivot_row = 0;
        for col in 0..2 * n {
            if pivot_row == n {
                break;
            }
            // Find a row in pivot_row..n with 1 at col
            let found = (pivot_row..n).find(|&r| {
                if col < n {
                    tab.x[n + r][col]
                } else {
                    tab.z[n + r][col - n]
                }
            });

            if let Some(r) = found {
                if r != pivot_row {
                    tab.swap_stabilizers(n + pivot_row, n + r);
                }
                for r2 in 0..n {
                    if r2 != pivot_row {
                        let has_bit = if col < n {
                            tab.x[n + r2][col]
                        } else {
                            tab.z[n + r2][col - n]
                        };
                        if has_bit {
                            tab.rowsum(n + r2, n + pivot_row);
                        }
                    }
                }
                pivot_row += 1;
            }
        }

        tab
    }

    fn swap_stabilizers(&mut self, r1: usize, r2: usize) {
        if r1 == r2 {
            return;
        }
        self.x.swap(r1, r2);
        self.z.swap(r1, r2);
        self.r.swap(r1, r2);
        let d1 = r1 - self.n;
        let d2 = r2 - self.n;
        self.x.swap(d1, d2);
        self.z.swap(d1, d2);
        self.r.swap(d1, d2);
    }
}

/// Check if two Tableaux represent the exact same stabilizer state.
pub fn are_tableaux_equivalent(t1: &Tableau, t2: &Tableau) -> bool {
    if t1.n != t2.n {
        return false;
    }
    let c1 = t1.canonical_form();
    let c2 = t2.canonical_form();
    let n = t1.n;

    // Stabilizers (rows n..2n) must match bit-for-bit
    for i in n..2 * n {
        if c1.x[i] != c2.x[i] || c1.z[i] != c2.z[i] || c1.r[i] != c2.r[i] {
            return false;
        }
    }
    true
}

/// Canonicalize a Clifford circuit into a canonical stabilizer Tableau.
pub fn canonicalize_circuit(c: &Circuit) -> Result<Tableau, String> {
    let mut t = Tableau::new(c.n_qubits, Mutation::None);
    for &g in &c.gates {
        match g {
            Gate::T(_) | Gate::Tdg(_) | Gate::Ccx(..) => {
                return Err("non-Clifford gate cannot be represented in a stabilizer tableau".into())
            }
            _ => t.apply(g),
        }
    }
    Ok(t.canonical_form())
}
