//! THE CHANNEL LEDGER'S DOOR (WB-9.1): the five exports the workbench page reads the six
//! channels through, gated against the record they claim to serve.
//!
//! `channel.rs` is the record — six channels, each with a plain name, a kind, a derived
//! rate and the rows that carry it — and `Sim::channel_standing` reads it against a running
//! scene. The page must not retype any of that, so the doors hand it over: the count, the
//! two names as NUL-terminated C strings in the module's static memory, the reach, and the
//! value.
//!
//! WHAT IS ACTUALLY CHECKED, and why each is here rather than assumed:
//!
//!   1. The two static name tables ARE `CHANNELS[i].plain` and the word for
//!      `CHANNELS[i].kind`, field for field. The tables exist only because `&'static str`
//!      is not NUL-terminated; a name edited in the record and not in the table would
//!      otherwise ship a page saying one thing and an engine holding another, and nothing
//!      else in the tree compares them.
//!   2. Every pointer reads back as the name it stands for, through the same walk-to-NUL a
//!      wasm host performs. A table whose terminator was dropped reads the NEXT name too,
//!      and the bounded walk is what says so instead of running off the statics.
//!   3. Off-the-end indices return null and NaN rather than panicking or reading a
//!      neighbour. A door that traps on a bad index takes the page's frame loop down.
//!   4. `holon_channel_value` serves a channel ONLY where a ledger row carries it wholly
//!      and alone, and returns NaN otherwise — checked in BOTH directions, against
//!      `Row::carries` itself rather than against a list of names. The direction that bites
//!      is the second: a value door that summed folded rows would hand the page some other
//!      channel's energy wearing this channel's label, which is precisely the number WB-7
//!      forbids. The served arm is pinned bit-for-bit against `holon_field_energy`, an
//!      independent export of the same row.
//!   5. THE READOUTS MOVE. A door checked once on a scene where every arm reads the same
//!      sentinel would pass while returning a constant. So two switches are thrown and both
//!      readings are required to change with them: the field takes presence's reach from
//!      NaN (no row) to `+inf` (the whole scene), and the seam takes sharing's from NaN to
//!      `+inf`. Both directions are asserted, so a door that answered `+inf` always would
//!      fail on the first reading and one that answered NaN always on the second.
//!
//! One test function on purpose: the doors act on the crate's one global `Sim`, and two
//! tests in this binary would race for it (the same reason `water_door.rs` gives).

use holon_render::channel::{Carriage, Kind, Row, CHANNELS};
use holon_render::{
    holon_channel_count, holon_channel_kind, holon_channel_plain, holon_channel_reach,
    holon_channel_value, holon_field_energy, holon_reset, holon_set_field, holon_set_seam,
    holon_table_generate,
};

/// The kind's own word, as the door spells it. Written out here rather than imported so the
/// gate compares two independent statements of the same thing; deriving it from `Debug`
/// would make the check pass by construction and establish nothing.
fn kind_word(k: Kind) -> &'static str {
    match k {
        Kind::Circumstances => "Circumstances",
        Kind::Structure => "Structure",
        Kind::Process => "Process",
        Kind::Rules => "Rules",
        Kind::Identity => "Identity",
    }
}

/// The host's side of a name door: walk to the NUL, exactly as a wasm host walks the
/// module's linear memory.
///
/// # Safety
/// `p` is a pointer the door returned for an in-range index, so it addresses a
/// NUL-terminated static in this module.
unsafe fn c_str(p: *const u8) -> String {
    assert!(!p.is_null(), "an in-range channel has a name");
    let mut n = 0usize;
    // A missing terminator is the failure this walk exists to expose, so the walk is
    // BOUNDED: without the guard a dropped NUL would run off the statics and either read a
    // neighbour or fault, and neither says which name lost its terminator.
    while n < 64 && *p.add(n) != 0 {
        n += 1;
    }
    assert!(n < 64, "the name is NUL-terminated within 64 bytes");
    String::from_utf8(std::slice::from_raw_parts(p, n).to_vec()).expect("the name is UTF-8")
}

/// Which rows carry this channel WHOLLY and carry nothing else — the door's rule, restated
/// from the record rather than from the door, so the two can disagree.
fn sole_whole_rows(id: holon_render::channel::ChannelId) -> Vec<Row> {
    Row::ALL
        .iter()
        .copied()
        .filter(|r| {
            let carried = r.carries();
            carried.len() == 1 && carried[0].0 == id && carried[0].1 == Carriage::Whole
        })
        .collect()
}

#[test]
fn the_channel_doors_serve_the_record_and_refuse_what_the_ledger_does_not_carry() {
    // A scene with a curve and atoms in it: the pair switch is derived from the curve, so
    // without one every reach arm would collapse to the same sentinel and the reach checks
    // below would not be able to tell the arms apart.
    assert_eq!(holon_table_generate(0.6, 12.0, 96), holon_render::TABLE_OK, "the H-H curve generates");
    holon_reset(8);

    // ---- 1 & 2: the count and the two name tables, against the record ----------
    assert_eq!(holon_channel_count() as usize, CHANNELS.len());
    assert_eq!(holon_channel_count(), 6, "the five, plus the sixth appended (CT-1)");
    for (i, c) in CHANNELS.iter().enumerate() {
        let plain = unsafe { c_str(holon_channel_plain(i as u32)) };
        let kind = unsafe { c_str(holon_channel_kind(i as u32)) };
        assert_eq!(plain, c.plain, "the plain-name table is CHANNELS[{i}].plain");
        assert_eq!(kind, kind_word(c.kind), "the kind table is CHANNELS[{i}].kind");
    }
    // The six plain names as Backpass V fixes them, in the record's order. Pinned here as
    // well as against the record because a rename could edit both together, and these six
    // words are the whole of the page's vocabulary for the ledger.
    let plains: Vec<String> = (0..holon_channel_count())
        .map(|i| unsafe { c_str(holon_channel_plain(i)) })
        .collect();
    assert_eq!(
        plains,
        ["presence", "accommodation", "attunement", "concert", "refusal", "sharing"]
    );

    // ---- 3: off the end refuses rather than reading a neighbour ----------------
    let n = holon_channel_count();
    assert!(holon_channel_plain(n).is_null());
    assert!(holon_channel_kind(n).is_null());
    assert!(holon_channel_plain(u32::MAX).is_null());
    assert!(holon_channel_kind(u32::MAX).is_null());
    assert!(holon_channel_value(n).is_nan());
    assert!(holon_channel_reach(n).is_nan());

    // ---- 4: the value door, both directions, against Row::carries --------------
    let mut served_any = false;
    let mut refused_any = false;
    for (i, c) in CHANNELS.iter().enumerate() {
        let sole = sole_whole_rows(c.id);
        let got = holon_channel_value(i as u32);
        if sole.is_empty() {
            refused_any = true;
            assert!(
                got.is_nan(),
                "{:?} has no row that carries it alone and wholly, so the door must not \
                 serve a number for it (got {got})",
                c.id
            );
        } else {
            served_any = true;
            assert!(!got.is_nan(), "{:?} is carried alone and wholly by {sole:?}", c.id);
        }
    }
    assert!(served_any, "some channel is served, or the served arm is untested");
    assert!(refused_any, "some channel is refused, or the refusing arm is untested");
    // The served arm's ARITHMETIC, against an independent export of the same row. Without
    // this the loop above would pass on a door that served the right channels out of the
    // wrong rows.
    assert_eq!(sole_whole_rows(CHANNELS[0].id), vec![Row::Field]);
    assert_eq!(
        holon_channel_value(0).to_bits(),
        holon_field_energy().to_bits(),
        "presence IS the field row, bit for bit"
    );
    // The two the sectors serve today and the four they do not, named so that a change in
    // the row map fails here rather than silently changing what the page draws.
    assert!(!holon_channel_value(0).is_nan(), "presence is served by the field row");
    assert!(!holon_channel_value(2).is_nan(), "attunement is served by the far row");
    assert!(holon_channel_value(1).is_nan(), "accommodation: FIELD-2 is named, not built");
    assert!(holon_channel_value(3).is_nan(), "concert is folded into the three-body row");
    assert!(holon_channel_value(4).is_nan(), "refusal is folded, and the seam row is three channels");
    assert!(holon_channel_value(5).is_nan(), "sharing rides the seam row with two others");

    // ---- 5: the reach arms, and that they MOVE ---------------------------------
    //
    // Induction is the Absent arm in every scene this engine can build: FIELD-2 is named
    // and not built, so nothing can make it move, and that is the point of asserting it
    // beside two readings that do move.
    assert!(holon_channel_reach(1).is_nan(), "induction has no row at all");

    // The field switch: presence goes from no row to the whole scene, and back.
    assert_eq!(holon_set_field(0), 0);
    assert!(holon_channel_reach(0).is_nan(), "field off: presence has no row");
    assert_eq!(holon_set_field(1), 0);
    let on = holon_channel_reach(0);
    assert!(on.is_infinite() && on > 0.0, "field on: presence reaches the whole scene (got {on})");
    assert_eq!(holon_set_field(0), 0);
    assert!(holon_channel_reach(0).is_nan(), "and back: the door tracks the switch");

    // The seam switch: sharing goes from no row to the whole scene, and back. The
    // amplitudes are CT-2's admitted law only in the sense that the transfer term is
    // non-zero — this gate is about the door, and the one property it needs is that
    // `p_ct != 0`, which is what puts channel 6 in the scene at all.
    assert!(holon_channel_reach(5).is_nan(), "seam off: sharing has no row");
    let code = holon_set_seam(
        1, 948.0, 2.40, 0.0, 2.06, 0.0, 22.59, 2.20, 1.525, 1.75, 0.0, 1.0, 1.371, 1.40, 1, 0,
        0.0, 12.0,
    );
    assert_eq!(code, 0, "the seam door admits this scene (refusal code {code})");
    let on = holon_channel_reach(5);
    assert!(on.is_infinite() && on > 0.0, "seam on: sharing reaches the whole scene (got {on})");
    // And the value door does NOT follow it: the seam row carries three channels wholly,
    // so it is no one of their numbers, and turning the seam on must not start serving a
    // number for sharing. This is the check that separates "has a row" from "has a value".
    assert!(
        holon_channel_value(5).is_nan(),
        "the seam row is exchange, dispersion and charge transfer added together — not any \
         one channel's value, however present the channel is"
    );
    assert_eq!(holon_set_seam(0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0, 0, 0.0, 12.0), 0);
    assert!(holon_channel_reach(5).is_nan(), "and back: the door tracks the switch");
}
