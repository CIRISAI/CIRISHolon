//! Compile every `kernels/*.cu` to PTX with nvcc, once, at build time.
//!
//! The PTX is CHECKED IN at `kernels/fold.ptx` so the crate builds on a machine
//! with no CUDA toolkit (only running needs a driver). But a checked-in artifact
//! generated from a source file is a stale-artifact trap of exactly the kind
//! LESSONS records — an edited `.cu` that silently keeps shipping yesterday's
//! `.ptx` is a diagnostic that does not echo its parameters. So:
//!
//! * nvcc present -> always recompile, and fail the build if nvcc fails;
//! * nvcc absent  -> use the checked-in PTX, but FAIL if the `.cu` is newer than
//!   the `.ptx`, because then the checked-in one is known to be stale and using
//!   it would be a silent lie.
//!
//! `-arch=compute_89` emits PTX only (no cubin): the installed nvcc is 12.0 and
//! the driver is 580 / CUDA 13.0, so the driver JIT-compiles the PTX at load.
//! That is the supported forward direction; the reverse (13.0 cubin, 12.0
//! driver) is not, and is not what happens here.

use std::path::Path;
use std::process::Command;

/// Every kernel translation unit, by stem. NAMED rather than globbed: a glob would compile
/// whatever happened to be in the directory, so a stray file becomes a build input and a
/// deleted one stops being checked without anything saying so.
const KERNELS: [&str; 2] = ["fold", "lanes_sigma"];

/// Per-kernel flags. `lanes_sigma` is the transliteration of a host body and is gated
/// bit-identical to it, so fused multiply-add — which the host never performs — is off.
fn extra_flags(stem: &str) -> &'static [&'static str] {
    match stem {
        "lanes_sigma" => &["-fmad=false"],
        _ => &[],
    }
}

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rerun-if-changed=build.rs");

    let nvcc = std::env::var("NVCC").unwrap_or_else(|_| "nvcc".to_string());
    let have_nvcc = Command::new(&nvcc)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    for stem in KERNELS {
        let cu = dir.join(format!("kernels/{stem}.cu"));
        let ptx = dir.join(format!("kernels/{stem}.ptx"));
        println!("cargo:rerun-if-changed={}", cu.display());
        compile_or_check(&nvcc, have_nvcc, stem, &cu, &ptx);
    }
}

fn compile_or_check(nvcc: &str, have_nvcc: bool, stem: &str, cu: &Path, ptx: &Path) {
    if have_nvcc {
        let out = Command::new(nvcc)
            .args(["-ptx", "-O3", "-arch=compute_89", "-lineinfo"])
            .args(extra_flags(stem))
            .arg(cu)
            .arg("-o")
            .arg(ptx)
            .output()
            .expect("nvcc reported a version and then failed to run");
        if !out.status.success() {
            panic!(
                "nvcc failed to compile kernels/{stem}.cu:\n{}\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
        }
        return;
    }

    // No nvcc. The checked-in PTX is the only option — and it is only honest if
    // it is not older than the source it claims to be a compilation of.
    let cu_t = std::fs::metadata(cu).and_then(|m| m.modified());
    let ptx_t = std::fs::metadata(ptx).and_then(|m| m.modified());
    match (cu_t, ptx_t) {
        (Ok(a), Ok(b)) if a > b => panic!(
            "no nvcc on PATH, and kernels/{stem}.cu is NEWER than the checked-in \
             kernels/{stem}.ptx. The checked-in PTX is stale; install the CUDA \
             toolkit or regenerate it on a machine that has one."
        ),
        (_, Err(e)) => panic!(
            "no nvcc on PATH and no checked-in kernels/{stem}.ptx ({e}); this crate \
             cannot be built without one of the two."
        ),
        _ => {
            println!("cargo:warning=nvcc not found; using the checked-in kernels/{stem}.ptx");
        }
    }
}
