//! THE CHANNEL LEDGER'S RECEIPT WRITER. Prints every reading of
//! `tests/common/channel_scenes.rs` as raw bits; with `HOLON_CHANNEL_RECEIPT=write` it
//! writes them to `tests/data/channel_ledger.receipt`, which `tests/channel_ledger.rs`
//! then requires the engine to reproduce exactly.
//!
//! Write the receipt ONLY at the commit whose force law is the one being frozen — it was
//! first written at the parent of the commit that introduced `channel.rs`, by this
//! example checked out against that parent's library. Regenerating it after a change is
//! how a change hides.

#[path = "../tests/common/channel_scenes.rs"]
#[allow(dead_code)]
mod channel_scenes;

fn main() {
    let text = channel_scenes::receipt();
    if std::env::var("HOLON_CHANNEL_RECEIPT").as_deref() == Ok("write") {
        std::fs::write(channel_scenes::RECEIPT, &text).expect("receipt written");
        eprintln!("wrote {}", channel_scenes::RECEIPT);
    }
    print!("{text}");
}
