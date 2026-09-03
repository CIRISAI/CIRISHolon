//! A2's ladder on the DEVICE: the same `q8_mps::qcd2::run_sym_ladder` the host driver runs,
//! with `GpuTwoSite` as the two-site executor. Checkpointed and resumable through `--ckpt`.
//!
//!   qcd2_sym_device --n 8 --x 4 --b 1 --chi 64,128,256 --sweeps 60 --mix 1e-4 --variance --ckpt DIR [--reseed] [--mutant] [--reserve-mib 512]
use holon_gpu::mps_blocks::GpuTwoSite;
use q8_mps::qcd2::{run_sym_ladder, LadderOpts};
use std::sync::Arc;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let get = |k: &str| a.iter().position(|s| s == k).map(|i| a[i + 1].clone());
    let opts = LadderOpts {
        n: get("--n").expect("--n").parse().unwrap(),
        x: get("--x").expect("--x").parse().unwrap(),
        b: get("--b").expect("--b").parse().unwrap(),
        chis: get("--chi").map_or(vec![64, 128, 256], |v| v.split(',').map(|c| c.parse().unwrap()).collect()),
        sweeps: get("--sweeps").map_or(60, |v| v.parse().unwrap()),
        mixing: get("--mix").map_or(0.0, |v| v.parse().unwrap()),
        reseed: a.iter().any(|s| s == "--reseed"),
        mutant: a.iter().any(|s| s == "--mutant"),
        variance: a.iter().any(|s| s == "--variance"),
        ckpt_dir: get("--ckpt").map(std::path::PathBuf::from),
        seed: 7,
        label_cap: 256,
    };
    if let Some(d) = &opts.ckpt_dir { std::fs::create_dir_all(d).expect("checkpoint dir"); }
    let reserve: u64 = get("--reserve-mib").map_or(512, |v| v.parse().unwrap());
    let gpu = Arc::new(GpuTwoSite::new(0, reserve).expect("a CUDA device"));
    let json = run_sym_ladder(&opts, Some(gpu.clone()));
    let refused = gpu.refusals.lock().unwrap().len();
    // the refusal count is part of the row: a device run that fell back to the host on some
    // bonds says so, rather than wearing a class it did not earn everywhere
    println!("{}", json.replacen("\"class\":\"device\"", &format!("\"class\":\"device\",\"device_refusals\":{refused}"), 1));
}
