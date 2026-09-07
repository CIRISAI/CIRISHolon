//! The closure itself: members, tier, the six-row ledger to each neighbour, and rent.
//!
//! # The six channels are declared HERE, and the fence that keeps them from disagreeing
//!
//! `holon-render::channel` carries the same six as `ChannelId`, with the same plain names
//! and the same kinds, because that crate needs them beside its force law and cannot depend
//! backwards on a crate that depends on it. Two declarations of one table is how the two
//! come to disagree — the lesson `holon-lattice`'s manifest records about `regplus`'s
//! direction set. The fence is not a comment: `holon-render`'s adapter test
//! `the_two_channel_tables_agree` maps the two enums onto each other and asserts the plain
//! name, the kind and the ORDER agree row by row, so a rename on either side fails a test
//! rather than producing two answers about one channel.

/// A closure's identity within one tier's reading. Indices into whatever list the tier
/// hands to [`crate::phase()`]; this crate never invents one.
pub type ClosureId = u32;

/// Which rung of GANTT2's zoom ladder a closure belongs to.
///
/// The integer is the rung and it is the whole of the identity: two closures are on the
/// same tier iff the rung matches. The associated constants are the ladder as `GANTT2.md`
/// lists it, bottom to top, and a tier this engine has not built is simply absent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TierId(pub u16);

impl TierId {
    /// The fold below the atom: the colour-singlet closure (GANTT2, GF nodes).
    pub const HADRON: TierId = TierId(0);
    /// Gated on GF2 and the chemistry tier's machinery; no record yet.
    pub const NUCLEUS: TierId = TierId(1);
    /// The atom tier, `TIERS.md`.
    pub const ATOM: TierId = TierId(2);
    /// The water unit: an oxygen and the two hydrogens its O–H curve holds lowest.
    pub const MOLECULAR: TierId = TierId(3);
    /// The H-bond network (LIQUID-1).
    pub const HBOND_NETWORK: TierId = TierId(4);
    /// The fluid element (FLUID-0, FLUID-1): the orientation lattice's carrier.
    pub const FLUID_ELEMENT: TierId = TierId(5);
}

/// The six channels of the ledger, in `GANTT2.md`'s order, which is rate order.
///
/// The names are the PLAIN ones because that is what the vocabulary table leads with; the
/// physics name is carried beside ([`Channel::physics`]) and is the key `holon-render`'s
/// `ChannelId` uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Channel {
    /// What a whole simply is to others at a distance. Field; `R⁻¹`.
    Presence,
    /// How each reshapes to make room for the other. Induction; `R⁻⁴`.
    Accommodation,
    /// Two flickering in step. Pair dispersion; `R⁻⁶`.
    Attunement,
    /// What three do that no pair does. Three-body dispersion; `R⁻⁹`.
    Concert,
    /// This space is taken. Exchange; `e^{−br}` — identity's hard edge.
    Refusal,
    /// Electrons lent while both stay whole. Charge transfer; `e^{−cr}` — identity's soft
    /// edge.
    Sharing,
}

/// The six, in the order a [`Ledger`]'s rows are stored in.
pub const CHANNELS: [Channel; 6] = [
    Channel::Presence,
    Channel::Accommodation,
    Channel::Attunement,
    Channel::Concert,
    Channel::Refusal,
    Channel::Sharing,
];

impl Channel {
    /// The row this channel occupies in a [`Ledger`]. The order is `CHANNELS`'s.
    pub const fn index(self) -> usize {
        match self {
            Channel::Presence => 0,
            Channel::Accommodation => 1,
            Channel::Attunement => 2,
            Channel::Concert => 3,
            Channel::Refusal => 4,
            Channel::Sharing => 5,
        }
    }

    /// The plain name a general reader is given first (`GANTT2.md`).
    pub const fn plain(self) -> &'static str {
        match self {
            Channel::Presence => "presence",
            Channel::Accommodation => "accommodation",
            Channel::Attunement => "attunement",
            Channel::Concert => "concert",
            Channel::Refusal => "refusal",
            Channel::Sharing => "sharing",
        }
    }

    /// The physics name, which stays the key.
    pub const fn physics(self) -> &'static str {
        match self {
            Channel::Presence => "field",
            Channel::Accommodation => "induction",
            Channel::Attunement => "pair dispersion",
            Channel::Concert => "three-body dispersion",
            Channel::Refusal => "exchange",
            Channel::Sharing => "charge transfer",
        }
    }

    /// Which of the account's kinds the channel is a change of.
    pub const fn kind(self) -> &'static str {
        match self {
            Channel::Presence => "Circumstances",
            Channel::Accommodation => "Structure",
            Channel::Attunement => "Process",
            Channel::Concert => "Rules",
            Channel::Refusal => "Identity",
            Channel::Sharing => "Identity",
        }
    }

    /// The leading decay law, as `GANTT2.md`'s `rate` column writes it.
    pub const fn reach(self) -> &'static str {
        match self {
            Channel::Presence => "R^-1",
            Channel::Accommodation => "R^-4",
            Channel::Attunement => "R^-6",
            Channel::Concert => "R^-9",
            Channel::Refusal => "exp(-b r)",
            Channel::Sharing => "exp(-c r)",
        }
    }

    /// How many closures must be present for the channel to exist. Concert is the one
    /// three-body row; every other channel is a pair channel.
    pub const fn arity(self) -> u8 {
        match self {
            Channel::Concert => 3,
            _ => 2,
        }
    }
}

/// One row of the ledger: a channel, what it carries, and whether anybody served it.
///
/// **The flag is not decoration.** A channel that was measured and read zero and a channel
/// nobody evaluated are different facts, and at the molecular tier both exist at once
/// (accommodation and attunement read zero AT THIS BASIS; concert was priced and refused).
/// An unserved row's stored value is `NaN`, so arithmetic that skipped [`LedgerRow::read`]
/// poisons its result instead of quietly adding a zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LedgerRow {
    channel: Channel,
    value: f64,
    served: bool,
}

impl LedgerRow {
    /// A row nobody has served. The value is `NaN`; it is not a zero.
    pub const fn unserved(channel: Channel) -> LedgerRow {
        LedgerRow { channel, value: f64::NAN, served: false }
    }

    /// A row somebody measured. A non-finite value is REFUSED: `NaN` is this type's mark
    /// for "not served" and an infinity is not a measurement.
    pub fn served(channel: Channel, value: f64) -> Option<LedgerRow> {
        value.is_finite().then_some(LedgerRow { channel, value, served: true })
    }

    pub const fn channel(&self) -> Channel {
        self.channel
    }

    pub const fn is_served(&self) -> bool {
        self.served
    }

    /// The value, or `None` where the row is unserved. There is no other way out.
    pub fn read(&self) -> Option<f64> {
        self.served.then_some(self.value)
    }
}

/// The six rows between one closure and one neighbour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ledger {
    rows: [LedgerRow; 6],
}

impl Default for Ledger {
    fn default() -> Self {
        Ledger::unserved()
    }
}

impl Ledger {
    /// Six rows, none of them served. The state a tier starts from.
    pub const fn unserved() -> Ledger {
        Ledger {
            rows: [
                LedgerRow::unserved(Channel::Presence),
                LedgerRow::unserved(Channel::Accommodation),
                LedgerRow::unserved(Channel::Attunement),
                LedgerRow::unserved(Channel::Concert),
                LedgerRow::unserved(Channel::Refusal),
                LedgerRow::unserved(Channel::Sharing),
            ],
        }
    }

    /// Serve one channel. Returns `false` and changes nothing on a non-finite value.
    pub fn serve(&mut self, channel: Channel, value: f64) -> bool {
        match LedgerRow::served(channel, value) {
            Some(r) => {
                self.rows[channel.index()] = r;
                true
            }
            None => false,
        }
    }

    /// Return one channel to the unserved state — a retraction, and the only way back.
    pub fn retract(&mut self, channel: Channel) {
        self.rows[channel.index()] = LedgerRow::unserved(channel);
    }

    pub fn row(&self, channel: Channel) -> &LedgerRow {
        &self.rows[channel.index()]
    }

    /// The value on one channel, or `None` where it is unserved.
    pub fn read(&self, channel: Channel) -> Option<f64> {
        self.rows[channel.index()].read()
    }

    pub fn rows(&self) -> &[LedgerRow; 6] {
        &self.rows
    }

    pub fn served_count(&self) -> usize {
        self.rows.iter().filter(|r| r.is_served()).count()
    }

    /// The sum over the SERVED rows, and how many rows were not served.
    ///
    /// Both numbers, always. A total quoted without the second one is a total over an
    /// unknown fraction of the ledger, and at the molecular tier that fraction is not
    /// small: concert was priced and refused, so a six-channel total there is a
    /// five-channel total wearing a six-channel name.
    pub fn served_total(&self) -> (f64, usize) {
        let mut total = 0.0;
        let mut missing = 0usize;
        for r in &self.rows {
            match r.read() {
                Some(v) => total += v,
                None => missing += 1,
            }
        }
        (total, missing)
    }
}

/// The ledger from one closure to one of its neighbours.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeighbourLedger {
    pub to: ClosureId,
    pub rows: Ledger,
}

/// What a closure pays per step to stay closed.
///
/// `CIRISOntology/Core/Maintenance.lean`'s `step γ α S = S − γ·S + α`, transcribed and not
/// reinterpreted. `decay` is `γ`, `payment` is `α`. The three theorems there are the three
/// verdicts here: `α = γS` holds the entry for ever (`rent_holds`), `α < γS` strictly loses
/// (`underpaid_shrinks`), `α = 0` tends to zero (`unpaid_decays`).
///
/// **It is a law about the model.** Nothing here is a claim that a physical binding obeys
/// it; a tier that says so must measure it, and the molecular tier did (CT-2's retention).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rent {
    /// `γ`: the fraction of the held amount that decays per step.
    pub decay: f64,
    /// `α`: what is paid back per step.
    pub payment: f64,
}

/// Which side of the rent clause a holding is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RentVerdict {
    /// `α = 0`: nothing is paid, and the entry tends to zero.
    Unpaid,
    /// `α < γS`: strictly loses.
    Underpaid,
    /// `α = γS` within the stated tolerance: steady, for ever.
    Holds,
    /// `α > γS`: grows. Not a failure — the mint (`Core/Creation.lean`) is exactly this.
    Overpaid,
}

impl Rent {
    /// One step of the rent clause on an entry currently worth `held`.
    pub fn step(&self, held: f64) -> f64 {
        held - self.decay * held + self.payment
    }

    /// What holding `held` steady costs per step: `γS`.
    pub fn due(&self, held: f64) -> f64 {
        self.decay * held
    }

    /// Which side of the clause `held` is on, within an ABSOLUTE tolerance on the payment.
    ///
    /// The tolerance is a parameter and never a default: "holds" is an exact equality in
    /// the Lean and any tolerance here is a measurement decision, so the caller states it.
    pub fn verdict(&self, held: f64, tol: f64) -> RentVerdict {
        let due = self.due(held);
        if self.payment == 0.0 {
            return RentVerdict::Unpaid;
        }
        if (self.payment - due).abs() <= tol {
            RentVerdict::Holds
        } else if self.payment < due {
            RentVerdict::Underpaid
        } else {
            RentVerdict::Overpaid
        }
    }
}

/// Why a closure could not be constructed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClosureRefusal {
    /// No members. A reading about nothing.
    NoMembers,
    /// A member index appears twice; the carrier would be double-counted.
    RepeatedMember(u32),
}

/// Why two closures could not be merged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MergeRefusal {
    /// Two tiers. A merge across rungs is a category error, not a bigger object.
    DifferentTiers(TierId, TierId),
    /// A member is in both. The merged member set would lose a member to deduplication and
    /// `part` could not put it back.
    SharedMember(u32),
}

/// Why a closure could not be parted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PartRefusal {
    /// A named member is not in the closure.
    NotAMember(u32),
    /// The part is empty, or is the whole closure. Neither is a part.
    NotAProperPart,
    /// The part names a member twice.
    RepeatedMember(u32),
}

/// A closed whole at one tier: which of the tier's carrier it is made of, what it carries
/// to each neighbour, and what it pays to stay closed.
///
/// The members are ascending and unique. That is enforced at construction and preserved by
/// [`Closure::merge`] and [`Closure::part`], which are exact inverses on the member set.
#[derive(Clone, Debug, PartialEq)]
pub struct Closure {
    members: Vec<u32>,
    /// Which rung of the ladder this closure lives on.
    pub tier: TierId,
    /// The six rows to each neighbour. Empty where no neighbour has been read; a neighbour
    /// listed with an all-unserved [`Ledger`] is a DIFFERENT statement — that neighbour
    /// exists and nothing between them has been measured.
    pub ledger: Vec<NeighbourLedger>,
    /// What it pays per step to stay closed. `None` where the tier has no rent rule, which
    /// is not the same as a rent of zero.
    pub rent: Option<Rent>,
}

impl Closure {
    /// A closure of the given members. Refuses an empty member set and a repeated member;
    /// sorts the rest.
    pub fn new(tier: TierId, members: impl Into<Vec<u32>>) -> Result<Closure, ClosureRefusal> {
        let mut m: Vec<u32> = members.into();
        if m.is_empty() {
            return Err(ClosureRefusal::NoMembers);
        }
        m.sort_unstable();
        for w in m.windows(2) {
            if w[0] == w[1] {
                return Err(ClosureRefusal::RepeatedMember(w[0]));
            }
        }
        Ok(Closure { members: m, tier, ledger: Vec::new(), rent: None })
    }

    /// The members, ascending and unique. Indices into the tier's own carrier.
    pub fn members(&self) -> &[u32] {
        &self.members
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        // A constructed closure never is; the method exists because clippy asks for it
        // beside `len`, and answering honestly is cheaper than an allow.
        self.members.is_empty()
    }

    pub fn contains(&self, member: u32) -> bool {
        self.members.binary_search(&member).is_ok()
    }

    /// Record the six rows to one neighbour, replacing any rows already recorded to it.
    pub fn set_neighbour(&mut self, to: ClosureId, rows: Ledger) {
        match self.ledger.iter_mut().find(|n| n.to == to) {
            Some(n) => n.rows = rows,
            None => self.ledger.push(NeighbourLedger { to, rows }),
        }
    }

    pub fn neighbour(&self, to: ClosureId) -> Option<&Ledger> {
        self.ledger.iter().find(|n| n.to == to).map(|n| &n.rows)
    }

    /// Two closures into one.
    ///
    /// **Exact on the member set, and DELIBERATELY empty on everything else.** The merged
    /// closure carries the union of the members and NO ledger and NO rent. That is not an
    /// omission: `Core/Closure.lean`'s `macro_law_forced` says the coarse law of a view is
    /// forced by the view, so a merged closure's ledger to its neighbours and its rent are
    /// measurements OF THE MERGED THING, and a ledger carried up by adding the parts' rows
    /// would be a claim nobody made. [`Closure::part`] undoes this exactly on the members
    /// and makes no promise about the rest, which is the whole of what the round-trip test
    /// asserts.
    pub fn merge(&self, other: &Closure) -> Result<Closure, MergeRefusal> {
        if self.tier != other.tier {
            return Err(MergeRefusal::DifferentTiers(self.tier, other.tier));
        }
        let mut m = Vec::with_capacity(self.members.len() + other.members.len());
        let (mut i, mut j) = (0usize, 0usize);
        while i < self.members.len() && j < other.members.len() {
            let (a, b) = (self.members[i], other.members[j]);
            if a == b {
                return Err(MergeRefusal::SharedMember(a));
            } else if a < b {
                m.push(a);
                i += 1;
            } else {
                m.push(b);
                j += 1;
            }
        }
        m.extend_from_slice(&self.members[i..]);
        m.extend_from_slice(&other.members[j..]);
        Ok(Closure { members: m, tier: self.tier, ledger: Vec::new(), rent: None })
    }

    /// One closure into two, at the named part.
    ///
    /// The first returned closure has exactly the members of `first`; the second has the
    /// complement, in ascending order. Refuses a part that is empty, that is the whole, or
    /// that names something the closure does not hold — the three ways a "part" is not one.
    /// Like [`Closure::merge`], neither result inherits a ledger or a rent.
    pub fn part(&self, first: &[u32]) -> Result<(Closure, Closure), PartRefusal> {
        if first.is_empty() || first.len() >= self.members.len() {
            return Err(PartRefusal::NotAProperPart);
        }
        let mut a: Vec<u32> = first.to_vec();
        a.sort_unstable();
        for w in a.windows(2) {
            if w[0] == w[1] {
                return Err(PartRefusal::RepeatedMember(w[0]));
            }
        }
        for &x in &a {
            if !self.contains(x) {
                return Err(PartRefusal::NotAMember(x));
            }
        }
        let b: Vec<u32> = self.members.iter().copied().filter(|x| a.binary_search(x).is_err()).collect();
        Ok((
            Closure { members: a, tier: self.tier, ledger: Vec::new(), rent: None },
            Closure { members: b, tier: self.tier, ledger: Vec::new(), rent: None },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_six_channels_are_gantt2s_six_in_gantt2s_order() {
        let plain: Vec<&str> = CHANNELS.iter().map(|c| c.plain()).collect();
        assert_eq!(
            plain,
            ["presence", "accommodation", "attunement", "concert", "refusal", "sharing"]
        );
        for (i, c) in CHANNELS.iter().enumerate() {
            assert_eq!(c.index(), i, "{} is stored out of order", c.plain());
        }
        assert_eq!(Channel::Concert.arity(), 3, "concert is the three-body row");
        assert_eq!(Channel::Refusal.kind(), Channel::Sharing.kind(), "both edges are Identity");
    }

    /// An unserved row is not a zero, and cannot be turned into one by reading it.
    #[test]
    fn an_unserved_row_reads_none_and_not_zero() {
        let l = Ledger::unserved();
        for c in CHANNELS {
            assert_eq!(l.read(c), None);
            assert!(l.row(c).read().is_none());
        }
        let (total, missing) = l.served_total();
        assert_eq!(total, 0.0, "the sum over no rows is zero");
        assert_eq!(missing, 6, "and the ledger says so, which is the point");
    }

    #[test]
    fn a_served_zero_and_an_unserved_row_are_different_facts() {
        let mut l = Ledger::unserved();
        assert!(l.serve(Channel::Attunement, 0.0));
        assert_eq!(l.read(Channel::Attunement), Some(0.0));
        assert_eq!(l.read(Channel::Concert), None);
        assert_eq!(l.served_total(), (0.0, 5));
        l.retract(Channel::Attunement);
        assert_eq!(l.read(Channel::Attunement), None);
    }

    #[test]
    fn a_row_refuses_a_non_finite_value() {
        let mut l = Ledger::unserved();
        assert!(!l.serve(Channel::Presence, f64::NAN));
        assert!(!l.serve(Channel::Presence, f64::INFINITY));
        assert_eq!(l.read(Channel::Presence), None, "a refused serve changed nothing");
    }

    /// `Core/Maintenance.lean`'s three theorems, on the transcription.
    #[test]
    fn the_rent_clause_holds_shrinks_and_decays() {
        let s = 3.5f64;
        let gamma = 0.25;
        let holds = Rent { decay: gamma, payment: gamma * s };
        assert_eq!(holds.step(s), s, "rent_holds: paying the decay holds the entry exactly");
        assert_eq!(holds.verdict(s, 0.0), RentVerdict::Holds);

        let under = Rent { decay: gamma, payment: gamma * s - 0.1 };
        assert!(under.step(s) < s, "underpaid_shrinks");
        assert_eq!(under.verdict(s, 1e-12), RentVerdict::Underpaid);

        let unpaid = Rent { decay: gamma, payment: 0.0 };
        let mut held = s;
        for _ in 0..400 {
            held = unpaid.step(held);
        }
        assert!(held < 1e-12, "unpaid_decays: {held}");
        assert_eq!(unpaid.verdict(s, 1e-12), RentVerdict::Unpaid);

        let over = Rent { decay: gamma, payment: gamma * s + 0.1 };
        assert_eq!(over.verdict(s, 1e-12), RentVerdict::Overpaid);
    }

    /// Forever, not for a while: the held entry is bit-steady over many steps.
    #[test]
    fn paying_the_rent_holds_for_ever() {
        let s = 1.0f64;
        let r = Rent { decay: 0.5, payment: 0.5 };
        let mut held = s;
        for _ in 0..100_000 {
            held = r.step(held);
        }
        assert_eq!(held, s);
    }

    #[test]
    fn a_closure_refuses_an_empty_or_repeated_member_set() {
        assert_eq!(Closure::new(TierId::MOLECULAR, vec![]), Err(ClosureRefusal::NoMembers));
        assert_eq!(
            Closure::new(TierId::MOLECULAR, vec![4, 1, 4]),
            Err(ClosureRefusal::RepeatedMember(4))
        );
        let c = Closure::new(TierId::MOLECULAR, vec![9, 2, 5]).unwrap();
        assert_eq!(c.members(), &[2, 5, 9], "members are held ascending");
    }

    /// THE ROUND TRIP, on the member set and stated as such.
    #[test]
    fn merge_and_part_are_exact_inverses_on_the_member_set() {
        let a = Closure::new(TierId::FLUID_ELEMENT, vec![7, 1, 4]).unwrap();
        let b = Closure::new(TierId::FLUID_ELEMENT, vec![9, 2]).unwrap();
        let m = a.merge(&b).unwrap();
        assert_eq!(m.members(), &[1, 2, 4, 7, 9]);
        let (a2, b2) = m.part(a.members()).unwrap();
        assert_eq!(a2.members(), a.members());
        assert_eq!(b2.members(), b.members());
        // and the other way round: part then merge
        let (p, q) = m.part(&[2, 9]).unwrap();
        assert_eq!(p.merge(&q).unwrap().members(), m.members());
    }

    #[test]
    fn a_merged_closure_inherits_no_ledger_and_no_rent() {
        let mut a = Closure::new(TierId::MOLECULAR, vec![0, 1, 2]).unwrap();
        let mut l = Ledger::unserved();
        assert!(l.serve(Channel::Presence, -0.01));
        a.set_neighbour(1, l);
        a.rent = Some(Rent { decay: 0.1, payment: 0.01 });
        let b = Closure::new(TierId::MOLECULAR, vec![3, 4, 5]).unwrap();
        let m = a.merge(&b).unwrap();
        assert!(m.ledger.is_empty(), "a merged ledger is a measurement, never a sum");
        assert_eq!(m.rent, None, "a merged rent is a measurement, never inherited");
    }

    #[test]
    fn merge_refuses_two_tiers_and_a_shared_member() {
        let a = Closure::new(TierId::MOLECULAR, vec![0, 1]).unwrap();
        let b = Closure::new(TierId::FLUID_ELEMENT, vec![2]).unwrap();
        assert_eq!(
            a.merge(&b),
            Err(MergeRefusal::DifferentTiers(TierId::MOLECULAR, TierId::FLUID_ELEMENT))
        );
        let c = Closure::new(TierId::MOLECULAR, vec![1, 5]).unwrap();
        assert_eq!(a.merge(&c), Err(MergeRefusal::SharedMember(1)));
    }

    #[test]
    fn part_refuses_the_three_things_that_are_not_parts() {
        let c = Closure::new(TierId::MOLECULAR, vec![0, 1, 2]).unwrap();
        assert_eq!(c.part(&[]), Err(PartRefusal::NotAProperPart));
        assert_eq!(c.part(&[0, 1, 2]), Err(PartRefusal::NotAProperPart));
        assert_eq!(c.part(&[0, 7]), Err(PartRefusal::NotAMember(7)));
        assert_eq!(c.part(&[0, 0]), Err(PartRefusal::RepeatedMember(0)));
    }

    #[test]
    fn a_neighbour_ledger_is_replaced_and_not_duplicated() {
        let mut c = Closure::new(TierId::MOLECULAR, vec![0]).unwrap();
        let mut l = Ledger::unserved();
        assert!(l.serve(Channel::Sharing, 1.0));
        c.set_neighbour(3, l);
        let mut l2 = Ledger::unserved();
        assert!(l2.serve(Channel::Sharing, 2.0));
        c.set_neighbour(3, l2);
        assert_eq!(c.ledger.len(), 1);
        assert_eq!(c.neighbour(3).unwrap().read(Channel::Sharing), Some(2.0));
        assert_eq!(c.neighbour(4), None);
    }
}
