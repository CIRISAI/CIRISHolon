//! The detail allocator, behind one interface (`INTERFACE.md` S4; issue #4 §3a).
//!
//! An allocator is given, for each live tile, how far it is from the view's focus, how many
//! marks drawing it fine would add (`work`), and how many of those a consumer READS (`read`:
//! here, the tagged ones, since a mark with no tag is decoration that nothing reads). It
//! returns which tiles are drawn fine, spending at most `budget` marks.
//!
//! Two allocators are here so that they can be compared on one scene at equal work:
//!
//! * [`Ring`]: nearest first. The frame as a box, which is what a distance ring is.
//! * [`Bits`]: most read per unit of work first, nearest on a tie. This is the selection
//!   rule of `OBJECT.md` (CORRECTION 2026-11-15) with the crudest possible price, a count of
//!   what is read. A measured price per tile (what the frame loses, in held-out description
//!   length, when the tile is coarse) replaces the count without changing this interface.
//!
//! Neither has been compared on a consumer's scene. That comparison is a staked reading
//! (issue #4 §3a: at equal work the second must do better on what the frame reads, or the
//! rule is killed there), not a property of this file.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    pub distance: f64,
    pub work: u64,
    pub read: u64,
}

pub trait Allocator {
    /// Which candidates are drawn fine. The sum of `work` over those chosen is at most
    /// `budget`. A candidate with no work is always fine: there is nothing to save.
    fn allocate(&self, cands: &[Candidate], budget: u64) -> Vec<bool>;
}

fn spend(cands: &[Candidate], budget: u64, mut order: Vec<usize>, skip_unfit: bool) -> Vec<bool> {
    let mut out = vec![false; cands.len()];
    let mut left = budget;
    order.retain(|&i| {
        if cands[i].work == 0 {
            out[i] = true;
            false
        } else {
            true
        }
    });
    for i in order {
        if cands[i].work <= left {
            left -= cands[i].work;
            out[i] = true;
        } else if !skip_unfit {
            break;
        }
    }
    out
}

/// Nearest first, stopping at the first tile that does not fit: a ring has no holes.
pub struct Ring;

impl Allocator for Ring {
    fn allocate(&self, cands: &[Candidate], budget: u64) -> Vec<bool> {
        let mut order: Vec<usize> = (0..cands.len()).collect();
        order.sort_by(|&a, &b| cands[a].distance.total_cmp(&cands[b].distance).then(a.cmp(&b)));
        spend(cands, budget, order, false)
    }
}

/// Most read per unit of work first; nearest on a tie; a tile that does not fit is passed
/// over and a smaller one behind it may still be taken.
pub struct Bits;

impl Allocator for Bits {
    fn allocate(&self, cands: &[Candidate], budget: u64) -> Vec<bool> {
        let mut order: Vec<usize> = (0..cands.len()).collect();
        // read_a / work_a > read_b / work_b, compared without division
        order.sort_by(|&a, &b| {
            let (l, r) = (cands[a].read as u128 * cands[b].work as u128, cands[b].read as u128 * cands[a].work as u128);
            r.cmp(&l).then(cands[a].distance.total_cmp(&cands[b].distance)).then(a.cmp(&b))
        });
        spend(cands, budget, order, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(distance: f64, work: u64, read: u64) -> Candidate {
        Candidate { distance, work, read }
    }

    #[test]
    fn neither_overspends_and_a_tile_with_no_work_is_fine() {
        let cands = [c(1.0, 50, 0), c(2.0, 0, 0), c(3.0, 40, 40), c(4.0, 30, 10), c(5.0, 20, 20)];
        for a in [&Ring as &dyn Allocator, &Bits] {
            for budget in [0u64, 20, 60, 90, 1000] {
                let f = a.allocate(&cands, budget);
                let spent: u64 = cands.iter().zip(&f).filter(|x| *x.1).map(|x| x.0.work).sum();
                assert!(spent <= budget && f[1], "{budget}: {f:?}");
            }
        }
        assert_eq!(Ring.allocate(&cands, 1000), vec![true; 5]);
        assert_eq!(Bits.allocate(&cands, 1000), vec![true; 5]);
    }

    #[test]
    fn the_ring_takes_the_nearest_and_the_other_takes_what_is_read() {
        // the nearest tile is all decoration; two farther ones are all read
        let cands = [c(1.0, 50, 0), c(3.0, 40, 40), c(5.0, 20, 20)];
        assert_eq!(Ring.allocate(&cands, 60), vec![true, false, false]);
        assert_eq!(Bits.allocate(&cands, 60), vec![false, true, true]);
        let read = |f: &[bool]| cands.iter().zip(f).filter(|x| *x.1).map(|x| x.0.read).sum::<u64>();
        assert!(read(&Bits.allocate(&cands, 60)) > read(&Ring.allocate(&cands, 60)));
    }

    #[test]
    fn where_everything_is_read_alike_the_two_agree() {
        // the plant against a free win: with read proportional to work, no order beats distance
        let cands = [c(1.0, 30, 30), c(2.0, 30, 30), c(3.0, 30, 30), c(4.0, 30, 30)];
        assert_eq!(Ring.allocate(&cands, 60), Bits.allocate(&cands, 60));
    }
}
