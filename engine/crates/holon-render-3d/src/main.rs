//! Binary entry point, and the crate's exported ABI.
//!
//! Two jobs, and they are here together because they have the same reason for being here:
//! `lib.rs` carries `#![forbid(unsafe_code)]`, and `forbid` rejects `#[no_mangle]`
//! outright. So every symbol the browser reaches lives in this file — the artifact's own
//! crate, which carries no such attribute — and every one of them is a single call into a
//! safe function in the library. There is no arithmetic here, no state, and no policy.
//!
//! # Which app a page gets, and why the page says so
//!
//! On wasm `main` does NOTHING. That is deliberate and it is the only shape that gives
//! one artifact two apps: wasm-bindgen runs `main` from inside `init()`, before any page
//! code can express a preference, so an app started there is an app the page cannot
//! choose. Instead the page calls `holon3d_run_owned()` or `holon3d_run_hosted()` after
//! `init()` resolves, and gets exactly the one it asked for.
//!
//!   * `holon3d_run_owned` — atoms3d. The app owns an `AtomWorld`, steps it, calibrates
//!     the device, and draws its own HUD.
//!   * `holon3d_run_hosted` — the workbench. No `AtomWorld`, no stepper, no HUD: the
//!     committed cdylib owns the only `Sim` on that page and this app draws what the
//!     bridge hands it.
//!
//! The alternative — a second pair of artifacts under a cargo feature — was rejected
//! because it changes the deployment shape, which is ruled fixed, and because two builds
//! of one renderer can silently diverge in ways one build cannot.
//!
//! Natively `main` still runs the owned app, because a desktop binary has no page to ask.
//!
//! # The ABI
//!
//! Scalars only. No pointers, no buffers, no `wasm-bindgen` dependency — the same raw
//! `extern "C"` shape `holon-render`'s cdylib already exports, which is why neither side
//! of the boundary needs `unsafe` to cross it. `u64` frame numbers and counters cross as
//! `f64` rather than as BigInt: a browser counting frames reaches 2^53 in about four and
//! a half million years, and a number JS can do arithmetic on is worth more than a range
//! nothing will use.

// ── entry points ─────────────────────────────────────────────────────────────────

/// Desktop: no page to ask, so the owned app it is.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    holon_render_3d::run();
}

/// wasm: the page chooses. See the header — this emptiness is the mechanism.
#[cfg(target_arch = "wasm32")]
fn main() {}

/// Start the app that owns its own world (atoms3d).
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn holon3d_run_owned() {
    holon_render_3d::run();
}

/// Start the app fed across the bridge (the workbench).
#[cfg(all(target_arch = "wasm32", feature = "render"))]
#[no_mangle]
pub extern "C" fn holon3d_run_hosted() {
    holon_render_3d::run_hosted();
}

// ── the bridge: a frame in ───────────────────────────────────────────────────────

/// Declare a frame of `n_atoms` atoms and `n_bonds` bonds.
#[no_mangle]
pub extern "C" fn holon3d_frame_begin(n_atoms: u32, n_bonds: u32) {
    holon_render_3d::bridge::frame_begin(n_atoms as usize, n_bonds as usize);
}

/// Place atom `i`. Must arrive in index order; the boundary refuses the frame otherwise.
#[no_mangle]
pub extern "C" fn holon3d_frame_atom(i: u32, x: f64, y: f64, z: f64, radius: f64, z_species: u32) {
    holon_render_3d::bridge::frame_atom(i as usize, x, y, z, radius, z_species);
}

/// Place bond `k` between atoms `i` and `j`, `depth` in 0..1.
#[no_mangle]
pub extern "C" fn holon3d_frame_bond(k: u32, i: u32, j: u32, depth: f64) {
    holon_render_3d::bridge::frame_bond(k as usize, i as usize, j as usize, depth as f32);
}

/// Commit the frame. `grabbed` is the held atom or a negative number for none. Returns 1
/// if the frame was accepted, 0 if it was refused for not matching what it declared.
#[no_mangle]
pub extern "C" fn holon3d_frame_commit(
    grabbed: i32,
    ax: f64,
    ay: f64,
    az: f64,
    frame: f64,
) -> u32 {
    let held = if grabbed < 0 {
        None
    } else {
        Some(grabbed as usize)
    };
    // A NaN or negative frame number would round to something arbitrary. Clamped at zero
    // and truncated, because a frame counter that goes backwards is a producer bug and
    // the buffer's whole reason for carrying one is to make staleness visible.
    let n = if frame.is_finite() && frame >= 0.0 {
        frame as u64
    } else {
        0
    };
    u32::from(holon_render_3d::bridge::frame_commit(held, (ax, ay, az), n))
}

// ── the bridge: the hand out ─────────────────────────────────────────────────────

/// How many hand ops are waiting.
#[no_mangle]
pub extern "C" fn holon3d_hand_len() -> u32 {
    holon_render_3d::bridge::ops_len() as u32
}

/// Kind of op `k`: 0 grab, 1 move anchor, 2 release. Negative if there is no such op.
#[no_mangle]
pub extern "C" fn holon3d_hand_kind(k: u32) -> i32 {
    match holon_render_3d::bridge::op_kind(k as usize) {
        Some(kind) => kind as i32,
        None => -1,
    }
}

/// Argument `which` of op `k`. NaN when the op has no such argument — never zero, which
/// is a legal coordinate and a legal atom index.
#[no_mangle]
pub extern "C" fn holon3d_hand_arg(k: u32, which: u32) -> f64 {
    holon_render_3d::bridge::op_arg(k as usize, which as usize)
}

/// Drop the ops the caller has carried to the cdylib. Failing to call this replays the
/// gesture on every frame, which posts its work again on every frame.
#[no_mangle]
pub extern "C" fn holon3d_hand_clear() {
    holon_render_3d::bridge::ops_clear();
}

// ── the bridge: what it refused ──────────────────────────────────────────────────

/// Frames committed on top of one the renderer had not drawn yet.
#[no_mangle]
pub extern "C" fn holon3d_dropped_frames() -> f64 {
    holon_render_3d::bridge::dropped_frames() as f64
}

/// Frames discarded at commit for not matching what they declared.
#[no_mangle]
pub extern "C" fn holon3d_refused_frames() -> f64 {
    holon_render_3d::bridge::refused_frames() as f64
}
