#[no_mangle]
pub extern "C" fn digest() -> u64 {
    holon_scene::household::digest(&[434_010, 305_330, 124_260, 91_074, 29_674, 15_652], &[1, 2, 3, 4, 5, 7])
}
