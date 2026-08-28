//! A reader for the `h2_potential.json` contract, for the NATIVE side only.
//!
//! The browser never uses this: JS parses JSON natively and pushes knots through the
//! same `holon_table_*` ABI the tests use, so both paths feed one interpolator. This
//! module exists so `cargo test` can read the very same file the viewer serves, and it
//! is compiled out of the wasm entirely (see the `cfg` in `lib.rs`) — a JSON parser is
//! pure weight in a build whose host already has one.
//!
//! It is deliberately not a general JSON parser. It scans for the contract's named
//! fields at any depth and reads the number or array of numbers that follows, which is
//! all the schema needs and cannot be fooled by the files it is pointed at. Anything
//! unexpected is an error, never a default.

pub struct PotentialFile {
    pub r: Vec<f64>,
    pub e: Vec<f64>,
    pub f: Vec<f64>,
    /// Optional `d2E_hartree_per_bohr2` column. Empty when the file does not carry one.
    pub d2: Vec<f64>,
    pub r_e: f64,
    pub d_e: f64,
    pub e_asymptote: f64,
    pub provenance: String,
}

fn find_key(src: &str, key: &str) -> Option<usize> {
    let needle = format!("\"{key}\"");
    let at = src.find(&needle)? + needle.len();
    let rest = &src[at..];
    let colon = rest.find(':')?;
    // Reject a match that is really a value, not a key: between the closing quote and
    // the colon there may be whitespace only.
    if !rest[..colon].trim().is_empty() {
        return None;
    }
    Some(at + colon + 1)
}

fn scalar(src: &str, key: &str) -> Result<f64, String> {
    let at = find_key(src, key).ok_or_else(|| format!("missing field \"{key}\""))?;
    let rest = src[at..].trim_start();
    let end = rest
        .find(|c: char| {
            !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E')
        })
        .unwrap_or(rest.len());
    rest[..end].parse::<f64>().map_err(|_| {
        format!(
            "field \"{key}\" is not a number: {:?}",
            &rest[..end.min(32)]
        )
    })
}

fn text(src: &str, key: &str) -> Option<String> {
    let at = find_key(src, key)?;
    let rest = src[at..].trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn array(src: &str, key: &str) -> Result<Vec<f64>, String> {
    let at = find_key(src, key).ok_or_else(|| format!("missing field \"{key}\""))?;
    let rest = src[at..].trim_start();
    let rest = rest
        .strip_prefix('[')
        .ok_or_else(|| format!("field \"{key}\" is not an array"))?;
    let end = rest
        .find(']')
        .ok_or_else(|| format!("field \"{key}\" is an unterminated array"))?;
    let mut out = Vec::new();
    for token in rest[..end].split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        out.push(
            token
                .parse::<f64>()
                .map_err(|_| format!("field \"{key}\" holds a non-number: {token:?}"))?,
        );
    }
    Ok(out)
}

pub fn parse(src: &str) -> Result<PotentialFile, String> {
    let r = array(src, "R_grid_bohr")?;
    let e = array(src, "E_hartree")?;
    let f = array(src, "F_hartree_per_bohr")?;
    if r.len() != e.len() || r.len() != f.len() {
        return Err(format!(
            "contract violation: R_grid_bohr/E_hartree/F_hartree_per_bohr have lengths {}/{}/{}",
            r.len(),
            e.len(),
            f.len()
        ));
    }
    if r.len() < 2 {
        return Err(format!("contract violation: only {} knots", r.len()));
    }
    // Optional: absence is not an error, and a present-but-wrong-length column is.
    //
    // TWO accepted names, which is not indulgence. This reader was written expecting
    // `d2E_hartree_per_bohr2`; the referee curve and `holon-chem`'s emitter both write
    // `E2_hartree_per_bohr2`. A file carrying the column under the other name was parsed
    // without complaint and silently loaded WITHOUT its curvature — an optional field
    // that is present and ignored looks exactly like an optional field that is absent,
    // which is why nothing caught it. Accepting both is the fix; refusing the file for
    // having the wrong spelling of an optional column would not be.
    let (d2_key, d2_raw) = match array(src, "d2E_hartree_per_bohr2") {
        Ok(v) => ("d2E_hartree_per_bohr2", Ok(v)),
        Err(_) => ("E2_hartree_per_bohr2", array(src, "E2_hartree_per_bohr2")),
    };
    let d2 = match d2_raw {
        Ok(v) if v.len() == r.len() => v,
        Ok(v) => {
            return Err(format!(
                "{d2_key} has {} entries against {} grid points",
                v.len(),
                r.len()
            ))
        }
        Err(_) => Vec::new(),
    };
    Ok(PotentialFile {
        r,
        e,
        f,
        d2,
        r_e: scalar(src, "R_e")?,
        d_e: scalar(src, "D_e")?,
        e_asymptote: scalar(src, "E_asymptote")?,
        provenance: text(src, "provenance").unwrap_or_else(|| "unlabelled".to_string()),
    })
}

/// Load a contract file straight into a table.
pub fn load_into(
    table: &mut crate::table::PotentialTable,
    src: &str,
) -> Result<PotentialFile, String> {
    let file = parse(src)?;
    if !table.begin(file.r.len()) {
        return Err(format!("table refused {} knots", file.r.len()));
    }
    for i in 0..file.r.len() {
        if !table.knot(i, file.r[i], file.e[i], file.f[i]) {
            return Err(format!("table refused knot {i}"));
        }
        if !file.d2.is_empty() && !table.knot_curvature(i, file.d2[i]) {
            return Err(format!("table refused the curvature at knot {i}"));
        }
    }
    let status = table.finish(file.r_e, file.d_e, file.e_asymptote);
    if status != crate::table::LoadStatus::Ok {
        return Err(format!("table load failed: {status:?}"));
    }
    Ok(file)
}
