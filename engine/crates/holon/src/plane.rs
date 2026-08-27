//! The PLANE: one observable across many degrees of freedom, bit-packed.
//! Tier 0's SOTA layout, and the building block of every tier above it
//! (a Pauli row is two planes and a sign; an affine stabilizer state's R and
//! J matrices are planes; an ℝ-tier's SoA columns are f64 planes).

#[derive(Clone, PartialEq, Debug)]
pub struct BitPlane {
    pub words: Vec<u64>,
    pub len: usize,
}

impl BitPlane {
    pub fn zeros(len: usize) -> Self {
        BitPlane { words: vec![0; len.div_ceil(64)], len }
    }

    #[inline]
    pub fn get(&self, i: usize) -> bool {
        self.words[i >> 6] >> (i & 63) & 1 == 1
    }

    #[inline]
    pub fn set(&mut self, i: usize, v: bool) {
        let (w, b) = (i >> 6, i & 63);
        if v {
            self.words[w] |= 1 << b;
        } else {
            self.words[w] &= !(1 << b);
        }
    }

    #[inline]
    pub fn flip(&mut self, i: usize) {
        self.words[i >> 6] ^= 1 << (i & 63);
    }

    /// Word-parallel XOR: the workhorse of rowsum and Gaussian pivoting.
    pub fn xor_assign(&mut self, o: &BitPlane) {
        for (a, b) in self.words.iter_mut().zip(&o.words) {
            *a ^= *b;
        }
    }

    pub fn any(&self) -> bool {
        self.words.iter().any(|&w| w != 0)
    }

    pub fn popcount(&self) -> u32 {
        self.words.iter().map(|w| w.count_ones()).sum()
    }
}
