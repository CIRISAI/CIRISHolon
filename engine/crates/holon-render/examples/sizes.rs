//! The memory budget behind the bank's declared species cap.
//!
//! ```text
//! cargo run -p holon-render --release --example sizes
//! ```
//!
//! `Sim` is constructed BY VALUE on the stack by every test, example and shell here, so
//! the bank's size is a stack budget and not only a wasm-memory one. `bank.rs` states the
//! cap and its reason; this prints the numbers that reason is made of, so a future change
//! to `MAX_KNOTS` or `MAX_SPECIES` shows its cost immediately rather than as an
//! unexplained `stack overflow` in an unrelated fixture.
fn main() {
    let table = std::mem::size_of::<holon_render::table::PotentialTable>();
    let bank = std::mem::size_of::<holon_render::bank::PairBank>();
    let sim = std::mem::size_of::<holon_render::sim::Sim>();
    println!("MAX_KNOTS          {}", holon_render::table::MAX_KNOTS);
    println!("MAX_SPECIES        {}", holon_render::bank::MAX_SPECIES);
    println!("MAX_TABLES         {}", holon_render::bank::MAX_TABLES);
    println!("PotentialTable     {table:>9} bytes");
    println!("TrimerTable        {:>9} bytes", std::mem::size_of::<holon_chem::trimer::TrimerTable>());
    println!("PairBank           {bank:>9} bytes  ({:.0} KB)", bank as f64 / 1024.0);
    println!("Sim                {sim:>9} bytes  ({:.0} KB)", sim as f64 / 1024.0);
    println!();
    println!("A spawned Rust thread gets 2 MiB of stack by default, and constructing a");
    println!("`Sim` by value can need two frames of it. The measured budget above is what");
    println!("holds `MAX_SPECIES` where it is.");
}
