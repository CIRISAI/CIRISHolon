//! THE JOB API — accept a standard job, emit a standard result.
//!
//! Input: a circuit in any accepted dialect plus an OPTIONAL config file
//! beside it (`<circuit>.json`, or an explicit path). The config keys follow
//! the shapes standard runners already use — `shots`, `method`, `seed`,
//! `memory` — so a caller who knows Qiskit/Cirq can drive this engine
//! without learning a new vocabulary, plus a `holon` section for the things
//! no standard has (exactness policy, ring selection, which passes run).
//!
//! Output: the Qiskit `Result` schema, with everything no spec has a field
//! for under `metadata` — see INTERFACE.md for why that is the honest
//! resolution rather than inventing a format.

use crate::qasm::Surface;

/// What a job asks for. Defaults are the certified path.
#[derive(Clone, Debug)]
pub struct JobConfig {
    /// Standard keys.
    pub shots: usize,
    pub seed: Option<u64>,
    /// `"amplitude"` (default), `"sample"`, or `"probabilities"`.
    pub method: String,
    /// The basis state for `amplitude`, as a bit string (leftmost = qubit 0).
    pub target: Option<String>,
    /// Holon-specific.
    pub simplify: bool,
    pub phasepoly: bool,
    /// Hold exactness (default) or allow a declared degradation.
    pub exact: bool,
}

impl Default for JobConfig {
    fn default() -> Self {
        JobConfig {
            shots: 1,
            seed: None,
            method: "amplitude".into(),
            target: None,
            simplify: true,
            phasepoly: true,
            exact: true,
        }
    }
}

/// A minimal, dependency-free JSON reader for the flat config shapes a job
/// file actually uses. Unknown keys are IGNORED (forward compatibility with
/// standard runners' extra fields); malformed values REFUSE (a silent
/// default would be a wrong answer wearing a right one).
pub fn parse_config(src: &str) -> Result<JobConfig, String> {
    let mut cfg = JobConfig::default();
    for (key, val) in flat_pairs(src) {
        let k = key.rsplit('.').next().unwrap_or(&key).to_string();
        match k.as_str() {
            "shots" => cfg.shots = val.parse().map_err(|_| format!("shots: {val}"))?,
            "seed" | "seed_simulator" => {
                cfg.seed = Some(val.parse().map_err(|_| format!("seed: {val}"))?)
            }
            "method" => cfg.method = val.trim_matches('"').to_string(),
            "target" | "bitstring" => cfg.target = Some(val.trim_matches('"').to_string()),
            "simplify" => cfg.simplify = val == "true",
            "phasepoly" => cfg.phasepoly = val == "true",
            "exact" => cfg.exact = val == "true",
            _ => {}
        }
    }
    if !cfg.exact {
        return Err(
            "exact=false is not implemented: this engine has no lawful approximation \
             at the exact tiers. Set a Policy with a declared degradation instead \
             (tune.rs), or leave exact=true."
                .into(),
        );
    }
    Ok(cfg)
}

/// Flatten `"a": {"b": 1}` into `("a.b", "1")` pairs — enough structure for
/// job configs, without a JSON dependency.
fn flat_pairs(src: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut path: Vec<String> = Vec::new();
    let b = src.as_bytes();
    let mut i = 0usize;
    let mut pending_key: Option<String> = None;
    while i < b.len() {
        match b[i] {
            b'"' => {
                let start = i + 1;
                let mut j = start;
                while j < b.len() && b[j] != b'"' {
                    j += 1;
                }
                let s = src[start..j].to_string();
                i = j + 1;
                // is it a key (followed by ':') or a value?
                let mut k = i;
                while k < b.len() && (b[k] as char).is_whitespace() {
                    k += 1;
                }
                if k < b.len() && b[k] == b':' {
                    pending_key = Some(s);
                    i = k + 1;
                } else if let Some(key) = pending_key.take() {
                    let full = path.iter().cloned().chain([key]).collect::<Vec<_>>().join(".");
                    out.push((full, format!("\"{s}\"")));
                }
            }
            b'{' => {
                if let Some(k) = pending_key.take() {
                    path.push(k);
                }
                i += 1;
            }
            b'}' => {
                path.pop();
                i += 1;
            }
            c if c.is_ascii_digit() || c == b'-' || c == b't' || c == b'f' => {
                let start = i;
                while i < b.len()
                    && (b[i].is_ascii_alphanumeric() || b[i] == b'.' || b[i] == b'-')
                {
                    i += 1;
                }
                if let Some(key) = pending_key.take() {
                    let full = path.iter().cloned().chain([key]).collect::<Vec<_>>().join(".");
                    out.push((full, src[start..i].to_string()));
                }
            }
            _ => i += 1,
        }
    }
    out
}

/// One job's outcome, ready to render as a Qiskit `Result`.
pub struct JobResult {
    pub n_qubits: usize,
    pub re: f64,
    pub im: f64,
    pub probability: f64,
    pub residual_zeta16: u8,
    pub magic_before: usize,
    pub magic_local: usize,
    pub magic_after: usize,
    pub gates_before: usize,
    pub gates_after: usize,
    pub simplify_seconds: f64,
    pub seconds: f64,
    pub cfg_simplify: bool,
    pub cfg_phasepoly: bool,
}

impl JobResult {
    /// The Qiskit `Result` schema, with our extras under `metadata`.
    pub fn to_qiskit_json(&self) -> String {
        format!(
            "{{\"backend_name\": \"cirisholon\", \"backend_version\": \"0.1.0\", \
             \"success\": true, \"results\": [{{\"shots\": 1, \"status\": \"DONE\", \
             \"data\": {{\"amplitude\": {{\"re\": {:.12}, \"im\": {:.12}}}, \
             \"probability\": {:.12}}}, \"metadata\": {{\"exact\": true, \
             \"ring\": \"Z[omega]\", \"residual_zeta16\": {}, \"n_qubits\": {}, \
             \"passes\": {{\"simplify\": {}, \"phasepoly\": {}, \"seconds\": {:.6}}}, \
             \"magic\": {{\"before\": {}, \"after_local\": {}, \"after\": {}}}, \
             \"gates\": {{\"before\": {}, \"after\": {}}}, \"seconds\": {:.6}}}}}]}}",
            self.re,
            self.im,
            self.probability,
            self.residual_zeta16,
            self.n_qubits,
            self.cfg_simplify,
            self.cfg_phasepoly,
            self.simplify_seconds,
            self.magic_before,
            self.magic_local,
            self.magic_after,
            self.gates_before,
            self.gates_after,
            self.seconds
        )
    }
}

/// Run a parsed surface program under a config, returning the job result.
pub fn run_surface(n: usize, surface: &[Surface], cfg: &JobConfig) -> Result<JobResult, String> {
    let gates_before = surface.len();
    let magic_before = crate::simplify::magic_weight(surface);
    let t0 = std::time::Instant::now();
    let tp = std::time::Instant::now();
    let s1 = if cfg.simplify { crate::simplify::simplify(surface) } else { surface.to_vec() };
    let magic_local = crate::simplify::magic_weight(&s1);
    let s2 = if cfg.simplify && cfg.phasepoly {
        crate::simplify::simplify(&crate::phasepoly::optimize(n, &s1))
    } else {
        s1
    };
    let simplify_seconds = tp.elapsed().as_secs_f64();
    let magic_after = crate::simplify::magic_weight(&s2);
    if s2.iter().any(|g| matches!(g, Surface::Face(..) | Surface::Rot(_))) {
        return Err("program carries face/generic rotations: route to face::amplitude_face \
                    or face::amplitude_poly (see INTERFACE.md)"
            .into());
    }
    let (core, phase16) = crate::qasm::lower(&s2);
    let p16 = phase16.rem_euclid(16);
    let prog = crate::qasm::Program {
        n_qubits: n,
        gates: core,
        measured: vec![],
        phase_omega: (p16 / 2) as u8,
        residual_zeta16: (p16 % 2) as u8,
    };
    let y: Vec<bool> = match &cfg.target {
        Some(bits) => {
            if bits.len() != n {
                return Err(format!("target width {} != qubit count {n}", bits.len()));
            }
            bits.chars().map(|c| c == '1').collect()
        }
        None => vec![false; n],
    };
    let (amp, residual) = crate::run::amplitude_program(&prog, &y);
    let (re, im) = amp.to_complex();
    Ok(JobResult {
        n_qubits: n,
        re,
        im,
        probability: re * re + im * im,
        residual_zeta16: residual,
        magic_before,
        magic_local,
        magic_after,
        gates_before,
        gates_after: s2.len(),
        simplify_seconds,
        seconds: t0.elapsed().as_secs_f64(),
        cfg_simplify: cfg.simplify,
        cfg_phasepoly: cfg.phasepoly,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_reads_standard_and_holon_keys() {
        let src = r#"{"shots": 1024, "seed_simulator": 42, "method": "amplitude",
                      "target": "1011", "holon": {"phasepoly": false}}"#;
        let c = parse_config(src).unwrap();
        assert_eq!(c.shots, 1024);
        assert_eq!(c.seed, Some(42));
        assert_eq!(c.target.as_deref(), Some("1011"));
        assert!(!c.phasepoly);
        assert!(c.simplify, "unspecified keys keep their certified default");
    }

    #[test]
    fn unknown_keys_are_ignored_and_bad_values_refuse() {
        let ok = parse_config(r#"{"shots": 2, "some_future_runner_key": "x"}"#).unwrap();
        assert_eq!(ok.shots, 2);
        let bad = parse_config(r#"{"shots": "many"}"#);
        assert!(bad.is_err(), "a malformed value must refuse, not default");
        let inexact = parse_config(r#"{"exact": false}"#);
        assert!(inexact.unwrap_err().contains("no lawful approximation"));
    }

    #[test]
    fn a_job_runs_end_to_end_and_renders_the_schema() {
        use crate::qasm::Surface::*;
        let surface = vec![H(0), Cx(0, 1), T(0), Tdg(0), H(0)];
        let cfg = JobConfig { target: Some("00".into()), ..Default::default() };
        let r = run_surface(2, &surface, &cfg).unwrap();
        // H(0) CX(0,1) H(0) on |00>: amplitude 1/2, so p = 1/4.
        assert!((r.probability - 0.25).abs() < 1e-12, "p = {}", r.probability);
        assert_eq!(r.magic_after, 0, "T·T† must cancel");
        let j = r.to_qiskit_json();
        assert!(j.contains("\"backend_name\": \"cirisholon\""));
        assert!(j.contains("\"success\": true"));
        assert!(j.contains("\"metadata\""));
        assert!(j.contains("\"after_local\""));
    }
}
