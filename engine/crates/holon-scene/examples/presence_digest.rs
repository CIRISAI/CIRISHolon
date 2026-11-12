//! Prints the digest of the shipped presence generators; built native and for wasm32-wasip1.
fn main() {
    println!("{}", holon_scene::presence::digest());
}
