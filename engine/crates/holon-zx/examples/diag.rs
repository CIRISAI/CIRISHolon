use holon::qasm::Surface::*;
use holon::run::amplitude;
struct Rng(u64);
impl Rng { fn next(&mut self)->u64{self.0=self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);self.0>>11}
           fn below(&mut self,n:usize)->usize{(self.next()%n as u64) as usize} }
fn main() {
    let mut rng = Rng(0xC0FFEE);
    for trial in 0..5 {
        let n = 3; let mut g = Vec::new();
        for _ in 0..25 {
            let q = rng.below(n); let mut q2 = rng.below(n); while q2==q { q2=rng.below(n); }
            g.push(match rng.below(7) {0=>H(q),1=>S(q),2=>T(q),3=>Tdg(q),4=>Cx(q,q2),5=>Cz(q,q2),_=>Z(q)});
        }
        let Ok((simp, red)) = holon_zx::canonicalize(n, &g) else { println!("trial {trial}: canonicalize failed"); continue };
        let (a,_) = holon::qasm::lower(&g); let (b,_) = holon::qasm::lower(&simp);
        let mut ratios = Vec::new();
        let mut probs_match = true;
        for k in 0..(1u32<<n) {
            let y: Vec<bool> = (0..n).map(|q| k>>q&1==1).collect();
            let (ar,ai) = amplitude(n,&a,&y).to_complex();
            let (br,bi) = amplitude(n,&b,&y).to_complex();
            let (pa,pb) = (ar*ar+ai*ai, br*br+bi*bi);
            if (pa-pb).abs() > 1e-9 { probs_match = false; }
            if pa > 1e-12 && pb > 1e-12 {
                let d = br*br+bi*bi;
                ratios.push(((ar*br+ai*bi)/d, (ai*br-ar*bi)/d));
            }
        }
        let consistent = ratios.windows(2).all(|w| (w[0].0-w[1].0).abs()<1e-9 && (w[0].1-w[1].1).abs()<1e-9);
        println!("trial {trial}: t {}->{}  probs_match={probs_match}  phase_consistent={consistent}  ratio={:?}",
                 red.t_before, red.t_after, ratios.first().map(|r| ((r.0*1e6).round()/1e6,(r.1*1e6).round()/1e6)));
    }
}
